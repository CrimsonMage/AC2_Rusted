//! `CPhysicsObj`'s activity and transient state: the movement, animation and misc active bits
//! (`set_*_active`), the `*_can_be_inactive` tests the activity manager asks, the transient
//! state a body drops on leaving the world (`clear_transient_states`) and the gravity a cell's
//! material sets (`UpdateGravity`).
//!
//! Retail's active bits also add the object to the world's active lists
//! (`is_movement_active` and friends are the bit tests); the Rust step has no such list, so only
//! the bits and the position bookkeeping are modelled.

use crate::fpu::ld;
use crate::motion_interp::ELEMENT_WATER;
use crate::objects::STATIC_PS;
use crate::physics_material::PhysicsMaterial;
use crate::vector::Vector;

use super::{
    PhysicsObj, CONTACT_TS, LANDSCAPE_CONTACT_TS, ON_WALKABLE_TS, SLIDING_TS, STATIONARY_FALL_TS,
    STATIONARY_STOP_TS, STATIONARY_STUCK_TS,
};

/// `TransientState` bits: `MOVEMENT_ACTIVE_TS`, `ANIMATION_ACTIVE_TS`, `MISC_ACTIVE_TS`.
pub(crate) const MOVEMENT_ACTIVE_TS: u32 = 0x80;
pub(crate) const ANIMATION_ACTIVE_TS: u32 = 0x100;
pub(crate) const MISC_ACTIVE_TS: u32 = 0x200;
/// `TransientState` water contact (`contact_is_water`).
const WATER_TS: u32 = 0x8;
/// The property `CPhysicsObj::UpdateGravity` reads from a cell's material: a gravity vector.
pub(crate) const GRAVITY_PROPERTY: u32 = 0x8000_0001;
/// The magnitude below which every velocity and acceleration counts as zero
/// (`movement_can_be_inactive`).
const REST_EPSILON: f64 = 0.0002;

fn at_rest(v: Vector) -> bool {
    [v.x, v.y, v.z]
        .into_iter()
        .all(|c| ld(c).abs() < REST_EPSILON)
}

impl PhysicsObj {
    /// `CPhysicsObj::set_movement_active`: refused for a static body. Going inactive while
    /// active records the current position as the previous one.
    pub(crate) fn set_movement_active(&mut self, active: bool) -> bool {
        if self.state & STATIC_PS != 0 {
            return false;
        }
        if active {
            self.transient_state |= MOVEMENT_ACTIVE_TS;
            return true;
        }
        if self.transient_state & MOVEMENT_ACTIVE_TS != 0 {
            self.prev_position = self.position;
        }
        self.transient_state &= !MOVEMENT_ACTIVE_TS;
        true
    }

    /// `CPhysicsObj::set_animation_active`: refused for a static body.
    pub(crate) fn set_animation_active_to(&mut self, active: bool) -> bool {
        if self.state & STATIC_PS != 0 {
            return false;
        }
        if active {
            self.transient_state |= ANIMATION_ACTIVE_TS;
        } else {
            self.transient_state &= !ANIMATION_ACTIVE_TS;
        }
        true
    }

    /// `CPhysicsObj::set_misc_active`: refused for a static body.
    pub(crate) fn set_misc_active(&mut self, active: bool) -> bool {
        if self.state & STATIC_PS != 0 {
            return false;
        }
        if active {
            self.transient_state |= MISC_ACTIVE_TS;
        } else {
            self.transient_state &= !MISC_ACTIVE_TS;
        }
        true
    }

    /// `CPhysicsObj::clear_transient_states` (`LeaveWorld`): contact, water contact, walkable,
    /// sliding, the stationary frame counters and the landscape contact are dropped; leaving
    /// walkable ground outside water is a `LeaveGround`; the movement, animation and misc
    /// activity ends.
    pub(crate) fn clear_transient_states(&mut self) {
        self.transient_state &= !CONTACT_TS;
        self.set_misc_active(true);
        self.transient_state &= !WATER_TS;
        let was_walkable = self.on_walkable();
        self.transient_state &= !ON_WALKABLE_TS;
        if was_walkable && self.desired_element() != ELEMENT_WATER {
            self.leave_ground();
        }
        self.transient_state &= !SLIDING_TS;
        self.transient_state &= !(STATIONARY_FALL_TS | STATIONARY_STOP_TS | STATIONARY_STUCK_TS);
        self.set_movement_active(false);
        self.set_animation_active_to(false);
        self.set_misc_active(false);
        self.transient_state &= !LANDSCAPE_CONTACT_TS;
    }

    /// `CPhysicsObj::animation_can_be_inactive`: an object not in the world.
    pub(crate) fn animation_can_be_inactive(&self, in_world: bool) -> bool {
        !in_world
    }

    /// `CPhysicsObj::movement_can_be_inactive`. A body in the world, mobile and without a
    /// parent stays active while any velocity, acceleration, omega or external field is not
    /// negligible, or a move-to, turn-to or interpolation runs; one with a missile manager
    /// never rests. (`m_vVelocityField` and `m_vAccelerationField` are not modelled: zero.)
    pub(crate) fn movement_can_be_inactive(
        &self,
        in_world: bool,
        has_parent: bool,
        has_missile_manager: bool,
    ) -> bool {
        if has_missile_manager {
            return false;
        }
        if !(in_world && self.is_mobile() && !has_parent) {
            return true;
        }
        let still = at_rest(self.velocity)
            && at_rest(self.acceleration)
            && at_rest(self.omega)
            && at_rest(self.omega_acceleration)
            && at_rest(self.external_field);
        if !still {
            return false;
        }
        let moving_to = self.move_to.as_ref().is_some_and(|m| m.moving_to);
        let interpolating = self.interp.as_ref().is_some_and(|i| i.active);
        !(moving_to || self.motion.is_turning_to() || interpolating)
    }

    /// `CPhysicsObj::misc_can_be_inactive`: an object neither in the world nor in a workspace
    /// can; otherwise it cannot while its collision manager lists anything
    /// (`CCollisionManager::IsEmpty`) or its detection manager is active
    /// (`CDetectionManager::IsActive`); the caller passes both answers.
    pub(crate) fn misc_can_be_inactive(
        &self,
        in_world: bool,
        in_workspace: bool,
        collision_manager_empty: bool,
        detection_active: bool,
    ) -> bool {
        if !in_world && !in_workspace {
            return true;
        }
        collision_manager_empty && !detection_active
    }

    /// `CPhysicsObj::UpdateGravity`: the gravity the material's `0x80000001` property holds
    /// becomes the body's local gravity. `true` when there is a material and either it has no
    /// such property (gravity stays) or the property is a vector (taken); `false` without a
    /// material or when the property is not a vector.
    pub(crate) fn update_gravity(&mut self, material: Option<&PhysicsMaterial>) -> bool {
        let Some(material) = material else {
            return false;
        };
        let property = material
            .groups
            .iter()
            .flat_map(|group| group.properties.iter())
            .find(|property| property.name == GRAVITY_PROPERTY);
        let Some(property) = property else {
            return true;
        };
        match property.vector() {
            Some([x, y, z]) => {
                self.local_gravity = Vector::new(x, y, z);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use ac2_db::{Ac2PropertyGroup, Ac2PropertyValue};

    use crate::physics_material::PhysicsMaterial;
    use crate::test_support::body_at;

    use super::GRAVITY_PROPERTY;

    fn gravity_material(z: f32) -> PhysicsMaterial {
        let mut bytes = Vec::new();
        for c in [0.0_f32, 0.0, z] {
            bytes.extend(c.to_le_bytes());
        }
        PhysicsMaterial {
            id: 1,
            groups: vec![Ac2PropertyGroup {
                name: 0,
                properties: vec![Ac2PropertyValue {
                    name: GRAVITY_PROPERTY,
                    value_type: 0x4000_0004,
                    bytes: bytes.into(),
                }],
            }],
        }
    }

    #[test]
    fn a_material_gravity_property_sets_the_local_gravity() {
        let mut body = body_at(70.0, 70.0, 50.0);
        assert!(!body.object_mut().update_gravity(None));
        assert!(body
            .object_mut()
            .update_gravity(Some(&PhysicsMaterial::with_id(1))));
        assert_eq!(body.object_mut().local_gravity.z, -9.8);
        assert!(body
            .object_mut()
            .update_gravity(Some(&gravity_material(-1.5))));
        assert_eq!(body.object_mut().local_gravity.z, -1.5);
    }

    #[test]
    fn leaving_the_world_clears_contact_and_activity() {
        let mut body = body_at(70.0, 70.0, 50.0);
        body.object_mut().transient_state |=
            super::CONTACT_TS | super::ON_WALKABLE_TS | 0x400 | 0x4;
        assert_eq!(body.object_mut().transient_state & 0x380, 0x380);
        body.object_mut().clear_transient_states();
        assert_eq!(body.object_mut().transient_state & 0x587, 0);
        // Not active any more, so a static body is the only one that refuses.
        assert_eq!(body.object_mut().transient_state & 0x380, 0);
    }

    #[test]
    fn an_idle_mobile_body_can_rest_and_a_moving_one_cannot() {
        let mut body = body_at(70.0, 70.0, 50.0);
        assert!(body
            .object_mut()
            .movement_can_be_inactive(true, false, false));
        body.object_mut().velocity = crate::vector::Vector::new(0.0, 1.0, 0.0);
        assert!(!body
            .object_mut()
            .movement_can_be_inactive(true, false, false));
        // Not in the world, or held by a parent: it rests regardless.
        assert!(body
            .object_mut()
            .movement_can_be_inactive(false, false, false));
        assert!(body
            .object_mut()
            .movement_can_be_inactive(true, true, false));
        assert!(!body
            .object_mut()
            .movement_can_be_inactive(true, false, true));
        assert!(body.object_mut().animation_can_be_inactive(false));
        assert!(!body.object_mut().animation_can_be_inactive(true));
    }

    #[test]
    fn a_static_body_refuses_activity_changes() {
        let mut body = body_at(70.0, 70.0, 50.0);
        body.object_mut().state |= crate::objects::STATIC_PS;
        assert!(!body.object_mut().set_movement_active(true));
        assert!(!body.object_mut().set_misc_active(false));
    }
}
