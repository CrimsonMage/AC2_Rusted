//! `CPhysicsObj::ApplyPhysObjProfile`: the weenie's physics profile (state bits, placement and
//! scale: `CWPhysicsObject::InitPhysics` reads it from the weenie) applied to the body, and the
//! `PhysicsState` setters it calls through the object's vtable.
//!
//! The profile's report-collisions bit goes through `set_report_collisions` (body.rs, vtable slot
//! 0x1A0, which runs `ReportCollisions`); the other setters here are plain state-bit sets.

use std::sync::Arc;

use super::super::PhysicsBody;
use crate::collision_types::{CollisionParameters, PhysicsTypeMask};

const IGNORE_COLLISIONS_PS: u32 = 0x8;
const LIGHTS_PS: u32 = 0x400;
const INIT_BEGIN_PS: u32 = 0x8_0000;
/// The property `InitObjectBegin` sets in a dynamic object's default movement and placement masks.
const DEFAULT_COLLISION_PROPERTY: u32 = 0x4000_0024;
/// The bool `MasterProperty` type word.
const TYPE_BOOL: u32 = 0x4000_0001;

/// `PhysObjProfile`: the part of the weenie's physics description `ApplyPhysObjProfile` reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysObjProfile {
    /// `_state`: `PhysicsState` bits; bits 2, 3, 9, 10, 13 and 15 are applied.
    pub state: u32,
    /// `_placement`: the Setup placement frame id.
    pub placement: u32,
    /// `_scale`: the uniform object scale.
    pub scale: f32,
}

impl PhysicsBody {
    /// `CPhysicsObj::InitObjectBegin`: a static object gets `STATIC_PS`, a dynamic one clears
    /// it and, without a collision manager, takes the default `CollisionParameters`
    /// (`SetCollisionState`: physics type empty, the movement and placement masks holding the bit
    /// of property `0x40000024`, which `MasterProperty::InqPropertyBit` gives; no bit without the
    /// installed property table); `set_mobile`; and the state's `0x80000` bit is set. Always true.
    pub fn init_object_begin(&mut self, dynamic: bool, mobile: bool) -> bool {
        self.object
            .set_state_bit(crate::physics_obj::STATIC_PS, !dynamic);
        if dynamic && !self.object.has_collision_manager {
            let mut mask = PhysicsTypeMask::default();
            if let Some(desc) = ac2_db::ac2_property_desc(DEFAULT_COLLISION_PROPERTY)
                .filter(|desc| desc.value_type == TYPE_BOOL)
            {
                mask.set_bit(u64::from(desc.data), true);
            }
            let mut collide = CollisionParameters::default();
            collide.set_bit(0x10);
            collide.set_physics_movement_type(mask);
            collide.set_physics_placement_type(mask);
            self.object.set_collision_state(&collide, &|_| None);
        }
        self.set_mobile(mobile);
        self.object.set_state_bit(INIT_BEGIN_PS, true);
        true
    }

    /// `CPhysicsObj::IsVisible`: hidden while the animation opacity is under 0.0002.
    pub fn is_visible(&self) -> bool {
        crate::object_visual::is_visible(Some(self.anim_opacity()))
    }

    /// `CPhysicsObj::m_state` (`PhysicsState` bits).
    pub fn state_bits(&self) -> u32 {
        self.object.state
    }

    /// `CPhysicsObj::set_ignore_collisions`: sets or clears `IGNORE_COLLISIONS_PS`.
    pub fn set_ignore_collisions_bit(&mut self, ignore: bool) {
        self.object.set_state_bit(IGNORE_COLLISIONS_PS, ignore);
    }

    /// `CPhysicsObj::set_lights`: sets or clears `LIGHTING_ON_PS` (the object lights its
    /// surroundings).
    pub fn set_lights(&mut self, on: bool) {
        self.object.set_state_bit(LIGHTS_PS, on);
    }

    /// `CPhysicsObj::get_lights`: `LIGHTING_ON_PS`.
    pub fn lights_on(&self) -> bool {
        self.state_bits() & LIGHTS_PS != 0
    }

    /// `CPhysicsObj::set_inelastic`: sets or clears `INELASTIC_PS`.
    pub fn set_inelastic(&mut self, inelastic: bool) {
        self.object
            .set_state_bit(crate::physics_obj::INELASTIC_PS, inelastic);
    }

    /// `CPhysicsObj::set_affected_by_gravity`: sets or clears `GRAVITY_PS`.
    pub fn set_affected_by_gravity(&mut self, affected: bool) {
        self.object
            .set_state_bit(crate::physics_obj::GRAVITY_PS, affected);
    }

    /// `CPhysicsObj::SetPlacementFrame` (`PHYSICSOBJREP::SetPlacementFrame`): the part tree takes
    /// the Setup placement frame `frame_id`. False without a representation or without that
    /// frame.
    pub fn set_placement_frame(&mut self, frame_id: u32) -> bool {
        let Some(tree) = self.object.part_group.as_mut() else {
            return false;
        };
        Arc::make_mut(tree).set_placement_frame_id(0, frame_id)
    }

    /// `CPhysicsObj::ApplyPhysObjProfile`: report-collisions (bit 2), ignore-collisions (3),
    /// lights (10), inelastic (13), edge-slide (15) and gravity (9) from the profile's state,
    /// then the placement frame and the uniform scale. Retail returns 1; here the result is the
    /// report `set_report_collisions` (vtable slot 0x1A0, the one that runs `ReportCollisions`)
    /// makes when the profile turns reporting on or off.
    pub fn apply_phys_obj_profile(
        &mut self,
        profile: &PhysObjProfile,
        materials: &crate::collision_report::ContactMaterials,
    ) -> Option<crate::collision_report::CollisionReport> {
        let bit = |n: u32| profile.state >> n & 1 != 0;
        let report = self.set_report_collisions(bit(2), materials);
        self.set_ignore_collisions_bit(bit(3));
        self.set_lights(bit(10));
        self.set_inelastic(bit(13));
        self.set_edge_slide(bit(15));
        self.set_affected_by_gravity(bit(9));
        self.set_placement_frame(profile.placement);
        self.set_uniform_scale(profile.scale);
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::body_at;

    #[test]
    fn the_profile_sets_each_state_bit_and_the_scale() {
        let mut body = body_at(70.0, 30.0, 50.0);
        let profile = PhysObjProfile {
            state: 1 << 2 | 1 << 10 | 1 << 13 | 1 << 9,
            placement: 0,
            scale: 2.0,
        };
        let materials = crate::collision_report::ContactMaterials::default();
        body.apply_phys_obj_profile(&profile, &materials);
        assert!(body.reports_collisions());
        assert!(body.lights_on());
        assert!(body.state_bits() & crate::physics_obj::INELASTIC_PS != 0);
        assert!(body.state_bits() & crate::physics_obj::GRAVITY_PS != 0);
        assert_eq!(body.state_bits() & IGNORE_COLLISIONS_PS, 0);
        assert_eq!(body.state_bits() & crate::physics_obj::EDGE_SLIDE_PS, 0);
        assert_eq!(body.own_total_scale().x, 2.0);
        // Clearing the bits clears the state, and the report bit reports its edge.
        let clear = PhysObjProfile {
            state: 1 << 3 | 1 << 15,
            placement: 0,
            scale: 1.0,
        };
        body.apply_phys_obj_profile(&clear, &materials);
        assert!(!body.reports_collisions());
        assert!(!body.lights_on());
        assert!(body.state_bits() & IGNORE_COLLISIONS_PS != 0);
        assert!(body.state_bits() & crate::physics_obj::EDGE_SLIDE_PS != 0);
        assert!(body.init_object_begin(false, false));
        assert!(body.state_bits() & crate::physics_obj::STATIC_PS != 0);
        assert!(!body.is_mobile());
        assert!(body.init_object_begin(true, true));
        assert!(body.state_bits() & crate::physics_obj::STATIC_PS == 0);
        assert!(body.is_mobile());
        // A dynamic object takes the default collision state (`SetCollisionState`).
        assert!(body.has_collision_manager());
    }
}
