//! `CPhysicsObj` moving through a `CTransition` and taking its result: `transition`,
//! `get_object_info`, `SetPositionInternal(CTransition*)`, `handle_all_collisions`,
//! `set_on_walkable` and `HandleHitGround`.

use crate::cells::ObjCell;
use crate::fpu::{ld, st, F_EPSILON};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND, ELEMENT_SEA_FLOOR};
use crate::physics_obj::{
    PhysicsObj, CONTACT_TS, EDGE_SLIDE_PS, INELASTIC_PS, LANDSCAPE_CONTACT_TS, ON_WALKABLE_TS,
    SLIDING_TS, STATIONARY_FALL_TS, STATIONARY_STOP_TS, STATIONARY_STUCK_TS, SYNC_TRANSIT_PS,
};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::transition::collision::{
    CollisionInfo, ObjectInfo, CONTACT_OI, EDGE_SLIDE_OI, FREE_ROTATE_OI, ON_WALKABLE_OI,
    PREDETERMINED_TRANSITION_OI,
};
use crate::transition::{Transition, TransitionObject};
use crate::vector::Vector;

/// `WATER_CONTACT_TS` and `ENVIRONMENT_CONTACT_TS`.
const WATER_CONTACT_TS: u32 = 0x8;
const ENVIRONMENT_CONTACT_TS: u32 = 0x800;
/// `handle_all_collisions` reflects velocity with this elasticity (`1 + 0.005`).
const ELASTICITY: f32 = 0.005;

impl PhysicsObj {
    /// `CPhysicsObj::can_jump`: in contact for at least half a second, on ground or sea floor.
    pub(crate) fn can_jump(&self) -> bool {
        let grounded = |element| element == ELEMENT_GROUND || element == ELEMENT_SEA_FLOOR;
        self.has_contact() && 0.5 <= self.time_on_ground && grounded(self.element)
    }

    /// `CPhysicsObj::Jump`: leave the ground with the ground values' jump impulse times the
    /// jump scale and the body's z scale (at least 0.5), and enter the air element.
    pub(crate) fn jump(&mut self) {
        if !self.has_contact() {
            return;
        }
        if self.element != ELEMENT_GROUND && self.element != ELEMENT_SEA_FLOOR {
            return;
        }
        self.transient_state &= !CONTACT_TS;
        let mut scale = self.scale.z;
        if ld(scale) < 0.5 {
            scale = 0.5;
        }
        let impulse = self
            .motion
            .motion_values_for_element(ELEMENT_GROUND)
            .jump_impulse;
        self.velocity.z = st(ld(self.jump_scale) * ld(impulse) * ld(scale));
        self.element = ELEMENT_AIR;
        self.times_jumped += 1;
    }

    /// `CPhysicsObj::get_object_info` (no admin move, no missile).
    fn object_info_state(&self) -> u32 {
        let mut state = if self.state & SYNC_TRANSIT_PS != 0 {
            PREDETERMINED_TRANSITION_OI
        } else if self.state & EDGE_SLIDE_PS != 0 {
            EDGE_SLIDE_OI
        } else {
            0
        };
        if self.has_contact() {
            state |= CONTACT_OI;
            if self.on_walkable() {
                state |= ON_WALKABLE_OI;
            }
        }
        if self.shape.allow_free_heading {
            state |= FREE_ROTATE_OI;
        }
        state
    }

    /// `CPhysicsObj::GetStepDownHeight`: the Setup step-down height, at most the body height
    /// less a centimetre.
    pub(crate) fn step_down_height(&self) -> f32 {
        let below_head = st(ld(self.height()) - ld(0.01_f32));
        let setup = self.shape.step_down_height;
        if ld(below_head) < ld(setup) {
            below_head
        } else {
            setup
        }
    }

    /// `CPhysicsObj::transition` (from the current position, not an admin move) and, on
    /// success, `SetPositionInternal(CTransition*)`. Returns whether the body moved.
    pub(crate) fn transition_to(&mut self, end: Position, terrain: &Terrain) -> bool {
        let contact = self.has_contact().then_some((
            self.contact_plane_cell_id,
            self.contact_plane,
            self.transient_state & WATER_CONTACT_TS != 0,
            self.transient_state & LANDSCAPE_CONTACT_TS != 0,
            self.transient_state & ENVIRONMENT_CONTACT_TS != 0,
        ));
        let frames_stationary_fall = if self.transient_state & STATIONARY_STUCK_TS != 0 {
            3
        } else if self.transient_state & STATIONARY_STOP_TS != 0 {
            2
        } else if self.transient_state & STATIONARY_FALL_TS != 0 {
            1
        } else {
            0
        };
        let spheres = self.shape.spheres.clone();
        let object = TransitionObject {
            state: self.object_info_state(),
            spheres: &spheres,
            scale: st(ld(self.shape.scale) * ld(self.scale.z)),
            step_up_height: self.shape.step_up_height,
            step_down_height: self.step_down_height(),
            affected_by_gravity: self.state & crate::physics_obj::GRAVITY_PS != 0,
            contact,
            sliding_normal: (self.transient_state & SLIDING_TS != 0).then_some(self.sliding_normal),
            frames_stationary_fall,
            containment_radius: self.containment_radius(),
        };
        let begin_cell =
            ObjCell::get(terrain, self.position.cell_id).map(|_| self.position.cell_id);
        let mut transition = Transition::new(&object, begin_cell, Some(self.position), end);
        let found = transition.find_valid_position(terrain);
        if transition.object_info.kill_velocity {
            self.kill_velocity();
        }
        if !found {
            return false;
        }
        self.set_position_internal(&transition);
        true
    }

    /// `OBJECTINFO::kill_velocity` -> `CPhysicsObj::set_velocity(0)`: only a velocity that
    /// differs from zero by `F_EPSILON` on some axis is replaced.
    fn kill_velocity(&mut self) {
        let v = self.velocity;
        if ld(v.x).abs() >= ld(F_EPSILON)
            || ld(v.y).abs() >= ld(F_EPSILON)
            || ld(v.z).abs() >= ld(F_EPSILON)
        {
            self.velocity = Vector::ZERO;
        }
    }

    /// `CPhysicsObj::SetPositionInternal(CTransition*)`.
    pub(crate) fn set_position_internal(&mut self, transition: &Transition) {
        let prev_on_walkable = self.on_walkable();
        let path = &transition.sphere_path;
        if path.curr_cell.is_none() {
            // `LeaveVisibility`: the body has left the resident cells and is not moved.
            return;
        }
        self.position = path.curr_pos;
        let c = &transition.collision_info;
        if self.state & SYNC_TRANSIT_PS == 0 {
            self.contact_plane_cell_id = c.contact_plane_cell_id;
            self.contact_plane = c.contact_plane;
            if c.contact_plane_valid {
                self.transient_state |= CONTACT_TS;
            } else {
                self.transient_state &= !CONTACT_TS;
            }
            if c.contact_plane_is_landscape {
                self.transient_state |= LANDSCAPE_CONTACT_TS;
            } else {
                self.transient_state &= !LANDSCAPE_CONTACT_TS;
            }
        }
        if self.has_contact() {
            let walkable = ObjectInfo::is_valid_walkable(self.contact_plane.n);
            self.set_on_walkable(walkable);
        } else {
            self.transient_state &= !ON_WALKABLE_TS;
        }
        self.sliding_normal = c.sliding_normal;
        if c.sliding_normal_valid {
            self.transient_state |= SLIDING_TS;
        } else {
            self.transient_state &= !SLIDING_TS;
        }
        self.handle_all_collisions(c, prev_on_walkable);
    }

    /// `CPhysicsObj::set_on_walkable`: landing absorbs the field velocity
    /// (`HandleHitGround`); leaving the ground is the motion interpreter's `LeaveGround` (an
    /// animation/FX event, not ported).
    fn set_on_walkable(&mut self, on_walkable: bool) {
        let was = self.on_walkable();
        if on_walkable {
            self.transient_state |= ON_WALKABLE_TS;
        } else {
            self.transient_state &= !ON_WALKABLE_TS;
        }
        if !was && on_walkable {
            self.handle_hit_ground();
        }
    }

    /// `CPhysicsObj::HandleHitGround` (the `HitGround` landing FX is not ported).
    fn handle_hit_ground(&mut self) {
        self.time_on_ground = 0.0;
        self.absorb_field_velocity();
    }

    /// `CPhysicsObj::handle_all_collisions` for the environment (object collisions and the
    /// collision manager's sounds and materials are not ported): a body that left or never had
    /// walkable ground reflects its velocity off the collision normal; falling in place stops
    /// it, and the stationary state follows the frame count.
    pub(crate) fn handle_all_collisions(
        &mut self,
        collisions: &CollisionInfo,
        prev_on_walkable: bool,
    ) {
        let left_ground = !prev_on_walkable || !self.on_walkable();
        let frames = collisions.frames_stationary_fall;
        if frames < 2 {
            if left_ground && collisions.collision_normal_valid {
                if self.state & INELASTIC_PS != 0 {
                    self.velocity = Vector::ZERO;
                } else {
                    let n = collisions.collision_normal;
                    let v = self.velocity;
                    let into = (ld(v.z) * ld(n.z) + ld(v.y) * ld(n.y)) + ld(n.x) * ld(v.x);
                    let scale = -(into * (ld(ELASTICITY) + 1.0));
                    let py = ld(st(scale * ld(n.y)));
                    let pz = ld(st(scale * ld(n.z)));
                    self.velocity = Vector::new(
                        st(scale * ld(n.x) + ld(v.x)),
                        st(py + ld(v.y)),
                        st(pz + ld(v.z)),
                    );
                }
            }
        } else {
            self.velocity = Vector::ZERO;
        }
        let stationary = STATIONARY_FALL_TS | STATIONARY_STOP_TS | STATIONARY_STUCK_TS;
        match frames {
            0 | 3 => self.transient_state &= !stationary,
            1 => self.transient_state |= STATIONARY_FALL_TS,
            2 => self.transient_state |= STATIONARY_STOP_TS,
            _ => self.transient_state |= STATIONARY_STUCK_TS,
        }
    }
}
