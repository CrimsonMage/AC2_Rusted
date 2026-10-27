//! `CPhysicsObj`'s movement step: `UpdateMovement` -> `UpdateMovementInternal(quantum)` ->
//! `UpdatePositionInternal` -> `UpdateMovementInternal(quantum, offset_frame)`, which runs
//! `CalcElement`, the motion interpreter, `CalcAcceleration`, the velocity/omega integration and
//! `CalcFriction`, then moves the body through a `CTransition`.

use crate::anim_rep::AnimRep;
use crate::fpu::{ld, st, F_EPSILON};
use crate::frame::Frame;
use crate::heading::Heading;
use crate::look_at::{LookAtManager, LookTarget};
use crate::motion_interp::{
    MotionContext, MotionInterp, ELEMENT_AIR, ELEMENT_GROUND, ELEMENT_SEA_FLOOR,
    ELEMENT_WALK_ON_AIR, ELEMENT_WATER,
};
use crate::objects::ObjectShape;
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
    /// `PhysicsGlobals::windmag`: the wind's magnitude as last set (`PhysicsBody::set_wind`).
    pub wind_magnitude: f32,
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
    /// `CPhysicsObj::m_fJumpScale` and `m_nTimesJumped`.
    pub jump_scale: f32,
    pub times_jumped: u32,
    /// The collision impact minimums (`CCollisionManager::m_current_info`), the last impact
    /// speed the environment contact recorded (`m_impact_speed`), and the impact not yet taken
    /// by the collision report (`m_has_normal` with `m_collision_normal`).
    pub(crate) impact_minimums: crate::collision_impact::ImpactMinimums,
    /// The body's `CollisionParameters` (`CPhysicsObj::SetCollisionState`): the ethereal and
    /// physics-type sets the transition tests objects against.
    pub(crate) collision: crate::collision_types::CollisionParameters,
    pub(crate) impact_speed: Option<f32>,
    pub(crate) impact_report: Option<crate::collision_impact::ImpactReport>,
    /// The element of a body without a behaviour table. Retail's `ModeTracker` never changes
    /// element without one; this port takes the element at once instead so tableless bodies
    /// still swim and fall.
    pub element: u32,
    pub motion: MotionInterp,
    /// `PHYSICSOBJREP`'s mode and slider trackers.
    pub(crate) anim: AnimRep,
    /// `CPhysicsObj::m_pLookAtManager`.
    pub(crate) look: LookAtManager,
    /// The look target's position and size, refreshed by the client before each animation step.
    pub(crate) look_snapshot: Option<LookTarget>,
    /// `CPhysicsObj::m_fHeadX` and `m_fHeadZ`: the head direction a server `LookAtDir` sent.
    pub(crate) head_x: f32,
    pub(crate) head_z: f32,
    /// `CSetup::radius` (unscaled), for move-to distances.
    pub setup_radius: f32,
    /// `CMotionInterp::mMoveToManager`.
    pub(crate) move_to: Option<crate::moveto::MoveToManager>,
    /// `CMotionInterp::mInterpolationManager`, made by the first server position.
    pub(crate) interp: Option<crate::interp::InterpolationManager>,
    /// `CMotionInterp::mConstraintManager`, made by the first `ConstrainTo`.
    pub(crate) constraint: Option<crate::constraint::ConstraintManager>,
    /// `Timer::cur_time` as interpolation reads it (seconds, the clock positions are sent on).
    pub cur_time: f64,
    pub shape: BodyShape,
    /// `CSetup::m_physics_containment_radius` of the body's spheres (unscaled).
    pub setup_containment: f32,
    /// `m_fViewDistance` (`UpdateViewerDistance`: the camera's distance to the body) and
    /// `m_fMinVisibleDistance` (written by the draw), which degrade the animation update rate.
    pub view_distance: f32,
    pub min_visible_distance: f32,
    /// `m_dLastAnimationUpdateTime`: `Timer::local_time` of the last animation update frame.
    pub last_animation_update_time: f64,
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
            wind_magnitude: 0.0,
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
            jump_scale: 1.0,
            times_jumped: 0,
            impact_minimums: crate::collision_impact::ImpactMinimums::default(),
            collision: crate::collision_types::CollisionParameters::default(),
            impact_speed: None,
            impact_report: None,
            element: ELEMENT_GROUND,
            motion,
            anim: AnimRep::unanimated(),
            interp: None,
            constraint: None,
            setup_radius: 0.0,
            move_to: None,
            cur_time: 0.0,
            setup_containment: ObjectShape::new(Vec::new(), Vec::new(), shape.spheres.clone())
                .containment_radius,
            shape,
            view_distance: 1.0,
            min_visible_distance: f32::MAX,
            last_animation_update_time: 0.0,
            look: LookAtManager::default(),
            look_snapshot: None,
            head_x: 0.0,
            head_z: 0.0,
        }
    }

    /// `CPhysicsObj::SetPhysicsContainmentRadius`: the Setup's containment radius times the
    /// length of the total scale.
    pub(crate) fn containment_radius(&self) -> f32 {
        let s = ld(self.shape.scale);
        let (x, y, z) = (
            ld(st(s * ld(self.scale.x))),
            ld(st(s * ld(self.scale.y))),
            ld(st(s * ld(self.scale.z))),
        );
        st(ld(st((x * x + y * y) + z * z)).sqrt() * ld(self.setup_containment))
    }

    /// `CPhysicsObj::InitObjectEnd`'s collision impact minimums: the XY minimum from the hover
    /// height and the standard values' forward speed, the Z minimum from the jump impulse and the
    /// jump scale (both plus 0.1).
    pub(crate) fn init_object_end(&mut self) {
        let hover = self.motion.hover_height(self.desired_element());
        let standard = self.motion.standard_motion_values();
        self.impact_minimums
            .init_hover(hover, standard.max_forward_velocity);
        self.impact_minimums
            .set_jump_scale(self.jump_scale, standard.jump_impulse);
    }

    /// `PHYSICSOBJREP::GetDesiredElement` (`ModeTracker::mDesiredElement`).
    pub(crate) fn desired_element(&self) -> u32 {
        if self.anim.can_animate() {
            self.anim.modes.desired_element()
        } else {
            self.element
        }
    }

    /// `PHYSICSOBJREP::GetElement`: the element the animation has reached.
    pub(crate) fn current_element(&self) -> u32 {
        if self.anim.can_animate() {
            self.anim.modes.element()
        } else {
            self.element
        }
    }

    /// `CPhysicsObj::DoElement` (`ModeTracker::DoElement`: a mode transition toward `element`).
    pub(crate) fn do_element(&mut self, element: u32) {
        if self.anim.can_animate() {
            self.anim.do_element(element);
            self.sync_mode();
        } else {
            self.element = element;
        }
    }

    /// `CMotionInterp::GetDesiredModeState` reads the mode tracker's desired mode.
    pub(crate) fn sync_mode(&mut self) {
        if self.anim.can_animate() {
            self.motion.set_mode(self.anim.modes.desired_mode());
        }
    }

    pub(crate) fn has_contact(&self) -> bool {
        self.transient_state & CONTACT_TS != 0
    }

    pub(crate) fn on_walkable(&self) -> bool {
        self.transient_state & ON_WALKABLE_TS != 0
    }

    pub(crate) fn is_mobile(&self) -> bool {
        self.state & MOBILE_PS != 0
    }

    /// `CPhysicsObj::GetRadius`: the Setup radius times the visual and object z scales.
    pub(crate) fn radius(&self) -> f32 {
        st(ld(st(ld(self.setup_radius) * ld(self.shape.scale))) * ld(self.scale.z))
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
            self.time_on_ground += f64::from(quantum);
        }
        self.calc_element(terrain);
        self.sync_mode();
        // The head of `CMotionInterp::UpdateMovement`: an active interpolation steers, then the
        // player's move-to.
        let mut managed = match self.interp_update(quantum, terrain) {
            Some(steering) => {
                self.motion.steer(steering.motion, steering.heading);
                true
            }
            None => false,
        };
        if self.anim.is_player() {
            if let Some((motion, heading)) = self.move_to_update(quantum) {
                self.motion.steer(motion, heading);
                managed = true;
            }
        }
        let context = self.motion_context(terrain);
        let clamp = self
            .motion
            .update_movement(quantum, &context, managed, &mut self.anim);
        self.sync_mode();
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
        if self.desired_element() != ELEMENT_AIR {
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
            desired_element: self.desired_element(),
            element: self.current_element(),
            is_sliding_down: self.has_contact() && !self.on_walkable(),
            constraint_factor: self.constraint_factor(),
            vector_heading: self.position.frame.vector_heading(),
        }
    }

    /// `CPhysicsObj::GetConstraintFactor` -> `CMotionInterp::GetConstraintFactor` at the
    /// body's position (1 without a constraint manager).
    fn constraint_factor(&self) -> f32 {
        self.constraint
            .as_ref()
            .map_or(1.0, |constraint| constraint.factor(&self.position))
    }

    /// `CPhysicsObj::GetHeightAboveWalkable`: zero in contact or walking on air, else the drop
    /// to the terrain below (`LOSCheck` against terrain only), negative when below it.
    pub(crate) fn height_above_walkable(&self, terrain: &Terrain) -> f32 {
        if self.desired_element() == ELEMENT_WALK_ON_AIR || self.has_contact() {
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
            .desired_motion_values(self.desired_element(), self.motion.mode())
            .gravity;
        if self.walk_on_air {
            self.do_element(ELEMENT_WALK_ON_AIR);
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
                self.do_element(ELEMENT_GROUND);
            } else {
                self.do_element(ELEMENT_AIR);
            }
            return;
        };
        let o = self.position.frame.origin;
        let n = plane.n;
        let below = -(((ld(n.z) * ld(o.z) + ld(n.y) * ld(o.y)) + ld(n.x) * ld(o.x)) + ld(plane.d));
        self.depth = st(below - water_height);
        if below - sea_height > 0.0 && ld(self.depth) < 0.0 {
            if self.has_contact() {
                self.do_element(ELEMENT_SEA_FLOOR);
            } else if ld(self.velocity.z) > 0.0 && ld(self.depth) < -0.3 {
                self.do_element(ELEMENT_AIR);
            }
            return;
        }
        if ld(self.depth) <= 0.0 {
            if self.has_contact() {
                self.do_element(ELEMENT_GROUND);
                let mode = self.motion.mode();
                if !self
                    .motion
                    .desired_motion_values(self.desired_element(), mode)
                    .gravity
                {
                    self.transient_state &= !CONTACT_TS;
                }
            } else {
                self.do_element(ELEMENT_AIR);
            }
            return;
        }
        if self.desired_element() != ELEMENT_WATER {
            self.handle_hit_water();
        }
        self.do_element(ELEMENT_WATER);
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

#[path = "physics_obj_anim.rs"]
mod anim;

#[path = "physics_obj_impulse.rs"]
mod impulse;

pub use anim::DescSlider;
