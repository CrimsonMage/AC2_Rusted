//! `CPhysicsObj`'s movement step: `UpdateMovement` -> `UpdateMovementInternal(quantum)` ->
//! `UpdatePositionInternal` -> `UpdateMovementInternal(quantum, offset_frame)`, which runs
//! `CalcElement`, the motion interpreter, `CalcAcceleration`, the velocity/omega integration and
//! `CalcFriction`, then moves the body through a `CTransition`.

use crate::fpu::{ld, st, F_EPSILON};
use crate::frame::Frame;
use crate::heading::Heading;
use crate::motion_interp::{
    MotionContext, MotionInterp, ELEMENT_AIR, ELEMENT_GROUND, ELEMENT_SEA_FLOOR,
    ELEMENT_WALK_ON_AIR, ELEMENT_WATER,
};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Plane, Quaternion, Sphere, Vector};

/// `PhysicsState` bits this port reads.
pub(crate) const GRAVITY_PS: u32 = 0x200;
pub(crate) const INELASTIC_PS: u32 = 0x2000;
pub(crate) const EDGE_SLIDE_PS: u32 = 0x8000;
pub(crate) const MOVEMENT_FROZEN_PS: u32 = 0x2_0000;
pub(crate) const MOBILE_PS: u32 = 0x20_0000;
pub(crate) const SYNC_TRANSIT_PS: u32 = 0x8000_0000;
/// `CPhysicsObj::CPhysicsObj`'s `m_state`: reports collisions, gravity, lighting, edge
/// sliding, mobile.
pub(crate) const DEFAULT_STATE: u32 = 0x0020_8604;

/// `TransientState` bits.
pub(crate) const CONTACT_TS: u32 = 0x1;
pub(crate) const ON_WALKABLE_TS: u32 = 0x2;
pub(crate) const SLIDING_TS: u32 = 0x4;
pub(crate) const STATIONARY_FALL_TS: u32 = 0x10;
pub(crate) const STATIONARY_STOP_TS: u32 = 0x20;
pub(crate) const STATIONARY_STUCK_TS: u32 = 0x40;
pub(crate) const LANDSCAPE_CONTACT_TS: u32 = 0x400;
/// `CPhysicsObj::CPhysicsObj`'s `m_transient_state`: movement, animation and misc active.
const DEFAULT_TRANSIENT_STATE: u32 = 0x380;

/// The global `gravity` vector `CPhysicsObj::CPhysicsObj` copies into `m_vLocalGravity`.
const GRAVITY: Vector = Vector::new(0.0, 0.0, -9.8);

/// A body's collision shape and size: its Setup's `CSphere`s (`CPartArray::GetSphere`, at most
/// two used), height and step heights, and the total scale `SPHEREPATH::init_sphere` applies.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyShape {
    pub spheres: Vec<Sphere>,
    /// `CSetup::height` (unscaled).
    pub height: f32,
    pub step_up_height: f32,
    pub step_down_height: f32,
    /// `CPhysicsObj::GetTotalScale` z.
    pub scale: f32,
    /// `CSetup::allow_free_heading` (`AllowsFreeHeading` -> `FREE_ROTATE_OI`).
    pub allow_free_heading: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PhysicsObj {
    pub position: Position,
    pub prev_position: Position,
    pub anim_position: Position,
    pub state: u32,
    pub transient_state: u32,
    pub velocity: Vector,
    pub anim_velocity: Vector,
    pub acceleration: Vector,
    pub acceleration_field_vel: Vector,
    pub external_field: Vector,
    pub omega: Vector,
    pub omega_acceleration: Vector,
    pub local_gravity: Vector,
    pub distance_cap_2d: f32,
    pub velocity_rail: Option<Vector>,
    pub clamp_heading: Option<Heading>,
    pub clamp_omega: Option<f32>,
    pub time_on_ground: f64,
    pub walk_on_air: bool,
    pub depth: f32,
    pub has_water: bool,
    pub contact_plane: Plane,
    pub contact_plane_cell_id: u32,
    pub sliding_normal: Vector,
    pub scale: Vector,
    /// `ModeTracker::mDesiredElement`; without an animated mode tracker the current element is
    /// the desired one (`ModeTracker::GetElement` with `mCanAnimate` 0).
    pub element: u32,
    pub motion: MotionInterp,
    pub shape: BodyShape,
}

impl PhysicsObj {
    /// `CPhysicsObj::CPhysicsObj` defaults for a body placed at `position`.
    pub(crate) fn new(position: Position, shape: BodyShape, motion: MotionInterp) -> Self {
        Self {
            position,
            prev_position: position,
            anim_position: position,
            state: DEFAULT_STATE,
            transient_state: DEFAULT_TRANSIENT_STATE,
            velocity: Vector::ZERO,
            anim_velocity: Vector::ZERO,
            acceleration: Vector::ZERO,
            acceleration_field_vel: Vector::ZERO,
            external_field: Vector::ZERO,
            omega: Vector::ZERO,
            omega_acceleration: Vector::ZERO,
            local_gravity: GRAVITY,
            distance_cap_2d: -1.0,
            velocity_rail: None,
            clamp_heading: None,
            clamp_omega: None,
            time_on_ground: 0.0,
            walk_on_air: false,
            depth: 0.0,
            has_water: false,
            contact_plane: Plane::default(),
            contact_plane_cell_id: 0,
            sliding_normal: Vector::ZERO,
            scale: Vector::new(1.0, 1.0, 1.0),
            element: ELEMENT_GROUND,
            motion,
            shape,
        }
    }

    pub(crate) fn has_contact(&self) -> bool {
        self.transient_state & CONTACT_TS != 0
    }

    pub(crate) fn on_walkable(&self) -> bool {
        self.transient_state & ON_WALKABLE_TS != 0
    }

    fn is_mobile(&self) -> bool {
        self.state & MOBILE_PS != 0
    }

    /// `CPhysicsObj::GetHeight`: the Setup height times the object's z scale.
    pub(crate) fn height(&self) -> f32 {
        st(ld(self.shape.height) * ld(self.shape.scale) * ld(self.scale.z))
    }

    /// `CPhysicsObj::UpdateMovement` for an object in the world, mobile and not frozen.
    pub(crate) fn update_movement(&mut self, quantum: f32, terrain: &Terrain) {
        if self.state & MOVEMENT_FROZEN_PS != 0 || !self.is_mobile() {
            return;
        }
        self.anim_position = self.prev_position;
        self.prev_position = self.position;
        self.update_movement_internal(quantum, terrain);
    }

    /// `CPhysicsObj::UpdateMovementInternal(float)`: integrate a frame offset, then move there
    /// (rotation only when the origin moved less than `F_EPSILON` on every axis).
    fn update_movement_internal(&mut self, quantum: f32, terrain: &Terrain) {
        let mut new_frame = Frame::IDENTITY;
        self.update_position_internal(quantum, &mut new_frame, terrain);
        if !self.is_mobile() {
            return;
        }
        let current = self.position.frame.origin;
        let near = |a: f32, b: f32| (ld(a) - ld(b)).abs() < ld(F_EPSILON);
        if near(new_frame.origin.x, current.x)
            && near(new_frame.origin.y, current.y)
            && near(new_frame.origin.z, current.z)
        {
            new_frame.origin = current;
            self.position.frame = new_frame;
            return;
        }
        let new_position = Position::new(self.position.cell_id, new_frame);
        if !self.transition_to(new_position, terrain) {
            new_frame.origin = self.position.frame.origin;
            self.position.frame = new_frame;
        }
    }

    /// `CPhysicsObj::UpdatePositionInternal`: the frame offset of this quantum applied to the
    /// current frame (origin added, rotation `current * offset`).
    fn update_position_internal(&mut self, quantum: f32, new_frame: &mut Frame, terrain: &Terrain) {
        let mut offset = Frame::IDENTITY;
        self.update_movement_offset(quantum, &mut offset, terrain);
        let frame = self.position.frame;
        new_frame.origin = Vector::new(
            st(ld(offset.origin.x) + ld(frame.origin.x)),
            st(ld(offset.origin.y) + ld(frame.origin.y)),
            st(ld(offset.origin.z) + ld(frame.origin.z)),
        );
        let q = Quaternion::multiply(frame.q, offset.q);
        new_frame.set_rotate(q);
    }

    /// `CPhysicsObj::UpdateMovementInternal(float, Frame&)`.
    fn update_movement_offset(&mut self, quantum: f32, offset: &mut Frame, terrain: &Terrain) {
        let q = ld(quantum);
        if self.has_contact() {
            self.time_on_ground = f64::from(quantum) + self.time_on_ground;
        }
        self.calc_element(terrain);
        let context = self.motion_context(terrain);
        let clamp = self.motion.update_movement(quantum, &context);
        self.clamp_heading = clamp.heading;
        self.clamp_omega = Some(clamp.omega_z);
        self.calc_acceleration(terrain);
        self.omega_acceleration = self.motion.omega_acceleration();
        // `CalcExternalForces` without pushers (`CCollisionManager::UpdatePusherValues`): the
        // fields stay zero and the residual field velocity decays.
        self.calc_external_forces();
        let v = self.velocity;
        let velocity_mag2 = (ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y)) + ld(v.z) * ld(v.z);
        let velocity_mag2_stored = st(velocity_mag2);
        let e = self.external_field;
        let field_mag2 = (ld(e.x) * ld(e.x) + ld(e.y) * ld(e.y)) + ld(e.z) * ld(e.z);
        if velocity_mag2 > 0.0 || field_mag2 > 0.0 {
            self.cull_velocity();
            let small = ld(0.05_f32) * ld(0.05_f32);
            let a = self.acceleration;
            if ld(velocity_mag2_stored) - small < ld(F_EPSILON)
                && ld(a.x).abs() < ld(F_EPSILON)
                && ld(a.y).abs() < ld(F_EPSILON)
                && ld(a.z).abs() < ld(F_EPSILON)
            {
                self.velocity = Vector::ZERO;
                self.acceleration = Vector::ZERO;
            }
            let displacement = self.integrate_offset(q);
            offset.origin = Vector::new(
                st(ld(displacement.x) + ld(offset.origin.x)),
                st(ld(displacement.y) + ld(offset.origin.y)),
                st(ld(displacement.z) + ld(offset.origin.z)),
            );
        }
        let to_current = self.anim_position.offset_to(&self.position);
        let scaled = Vector::new(
            st(ld(10.0) * ld(to_current.x)),
            st(ld(10.0) * ld(to_current.y)),
            st(ld(10.0) * ld(to_current.z)),
        );
        self.anim_velocity = self.anim_position.frame.global_to_local_vec(scaled);
        let w = self.omega;
        let omega_mag2 = (ld(w.x) * ld(w.x) + ld(w.y) * ld(w.y)) + ld(w.z) * ld(w.z);
        let wa = self.omega_acceleration;
        let turning = omega_mag2 > 0.0
            || (ld(wa.x) * ld(wa.x) + ld(wa.y) * ld(wa.y)) + ld(wa.z) * ld(wa.z) > 0.0;
        let small = ld(0.05_f32) * ld(0.05_f32);
        if omega_mag2 - small < ld(F_EPSILON)
            && ld(wa.x).abs() < ld(F_EPSILON)
            && ld(wa.y).abs() < ld(F_EPSILON)
            && ld(wa.z).abs() < ld(F_EPSILON)
        {
            self.omega = Vector::ZERO;
            self.omega_acceleration = Vector::ZERO;
        } else if turning {
            self.update_rotation(quantum, offset);
        }
        let a = self.acceleration;
        let az = st(q * ld(a.z));
        self.velocity = Vector::new(
            st(q * ld(a.x) + ld(self.velocity.x)),
            st(q * ld(a.y) + ld(self.velocity.y)),
            st(ld(az) + ld(self.velocity.z)),
        );
        self.calc_friction(quantum, velocity_mag2_stored);
    }

    /// The displacement `v q + field + a q^2 / 2` of `UpdateMovementInternal`, with its float
    /// stores, the 2D distance cap and the velocity rail.
    fn integrate_offset(&mut self, q: f64) -> Vector {
        let a = self.acceleration;
        let v = self.velocity;
        let e = self.external_field;
        let half_z = ld(st(ld(a.z) * 0.5));
        let ax = ld(st(ld(st(ld(a.x) * 0.5 * q)) * q));
        let ay = ld(st(ld(a.y) * 0.5 * q * q));
        let az = half_z * q * q;
        let vz = ld(st(q * ld(v.z)));
        let ox = ld(st(q * ld(v.x) + ld(e.x)));
        let oy = q * ld(v.y) + ld(e.y);
        let oz = ld(st(vz + ld(e.z)));
        let mut x = st(ox + ax);
        let mut y = st(oy + ay);
        let z = st(oz + az);
        if self.element != ELEMENT_AIR {
            if ld(self.distance_cap_2d) >= 0.0 {
                let travelled = (ld(y) * ld(y) + ld(x) * ld(x)).sqrt();
                let travelled_stored = st(travelled);
                if travelled > ld(self.distance_cap_2d) {
                    if travelled < ld(F_EPSILON) {
                        (x, y) = (0.0, 0.0);
                    } else {
                        let inverse = 1.0 / travelled;
                        let cap = ld(self.distance_cap_2d);
                        let ux = ld(st(ld(x) * inverse));
                        let uy = ld(st(inverse * ld(y)));
                        (x, y) = (st(cap * ux), st(cap * uy));
                    }
                }
                self.distance_cap_2d = st(ld(self.distance_cap_2d) - ld(travelled_stored));
            }
            if let Some(rail) = self.velocity_rail {
                let length = ((ld(y) * ld(y) + ld(x) * ld(x)) + ld(z) * ld(z)).sqrt();
                (x, y) = (st(length * ld(rail.x)), st(length * ld(rail.y)));
            }
        }
        Vector::new(x, y, z)
    }

    /// What `PHYSICSOBJREP` answers the motion interpreter.
    fn motion_context(&self, terrain: &Terrain) -> MotionContext {
        MotionContext {
            local_velocity: self.position.frame.global_to_local_vec(self.velocity),
            omega: self.omega,
            scale: self.scale,
            height_above_walkable: self.height_above_walkable(terrain),
            has_water: self.has_water,
            depth: self.depth,
            desired_element: self.element,
            element: self.element,
            constraint_factor: 1.0,
            vector_heading: self.position.frame.vector_heading(),
        }
    }

    /// `CPhysicsObj::GetHeightAboveWalkable`: zero in contact or walking on air, else the drop
    /// to the terrain below (`LOSCheck` against terrain only), negative when below it.
    pub(crate) fn height_above_walkable(&self, terrain: &Terrain) -> f32 {
        if self.element == ELEMENT_WALK_ON_AIR || self.has_contact() {
            return 0.0;
        }
        terrain.height_above_terrain(&self.position).unwrap_or(0.0)
    }

    /// `CPhysicsObj::CalcElement`: ground or air by contact, water and sea floor by the depth
    /// of the body under its cell's water plane.
    fn calc_element(&mut self, terrain: &Terrain) {
        if !self.is_mobile() {
            return;
        }
        let height = ld(self.height());
        let water_height = ld(st(height * ld(self.motion_water_multiplier())));
        let sea_height = ld(st(height * ld(self.motion_sea_floor_multiplier())));
        let gravity = self
            .motion
            .desired_motion_values(self.element, self.motion.mode())
            .gravity;
        if self.walk_on_air {
            self.element = ELEMENT_WALK_ON_AIR;
            if !gravity {
                self.transient_state &= !CONTACT_TS;
            }
            return;
        }
        let water = terrain.water_plane(self.position.cell_id);
        self.has_water = water.is_some();
        let Some(plane) = water else {
            if self.has_contact() {
                if !gravity {
                    self.transient_state &= !CONTACT_TS;
                }
                self.element = ELEMENT_GROUND;
            } else {
                self.element = ELEMENT_AIR;
            }
            return;
        };
        let o = self.position.frame.origin;
        let n = plane.n;
        let below = -(((ld(n.z) * ld(o.z) + ld(n.y) * ld(o.y)) + ld(n.x) * ld(o.x)) + ld(plane.d));
        self.depth = st(below - water_height);
        if below - sea_height > 0.0 && ld(self.depth) < 0.0 {
            if self.has_contact() {
                self.element = ELEMENT_SEA_FLOOR;
            } else if ld(self.velocity.z) > 0.0 && ld(self.depth) < -0.3 {
                self.element = ELEMENT_AIR;
            }
            return;
        }
        if ld(self.depth) <= 0.0 {
            if self.has_contact() {
                self.element = ELEMENT_GROUND;
                let mode = self.motion.mode();
                if !self
                    .motion
                    .desired_motion_values(self.element, mode)
                    .gravity
                {
                    self.transient_state &= !CONTACT_TS;
                }
            } else {
                self.element = ELEMENT_AIR;
            }
            return;
        }
        if self.element != ELEMENT_WATER {
            self.handle_hit_water();
        }
        self.element = ELEMENT_WATER;
        if !gravity {
            self.transient_state &= !CONTACT_TS;
        }
    }

    fn motion_water_multiplier(&self) -> f32 {
        self.motion.water_multiplier()
    }

    fn motion_sea_floor_multiplier(&self) -> f32 {
        self.motion.sea_floor_multiplier()
    }

    /// `CPhysicsObj::HandleHitWater`.
    fn handle_hit_water(&mut self) {
        self.absorb_field_velocity();
    }

    /// The field-velocity hand-off `HandleHitGround` and `HandleHitWater` share.
    pub(crate) fn absorb_field_velocity(&mut self) {
        let f = self.acceleration_field_vel;
        self.velocity = Vector::new(
            st(ld(f.x) + ld(self.velocity.x)),
            st(ld(f.y) + ld(self.velocity.y)),
            st(ld(f.z) + ld(self.velocity.z)),
        );
        self.acceleration_field_vel = Vector::ZERO;
    }
}

#[path = "physics_obj_forces.rs"]
mod forces;
