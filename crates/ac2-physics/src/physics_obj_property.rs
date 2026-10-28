//! `CPhysicsObj`'s property appliers: `ApplyPropertyCollection` walks a description's property
//! groups, `ApplyProperty` hands each property to the applier of its group (physics, render,
//! ethereal, gameplay, pusher), and every applier but the gameplay one also keeps the property in
//! the object's `m_pPropertyInfo` (a `PhysicsMaterial` collection that `InqCurrentProperties`
//! reads back).
//!
//! `BaseProperty::IsDefaultValue` needs the `MasterProperty` default values, which the Rust
//! table does not hold: every property is kept, so `m_pPropertyInfo` can list a property
//! retail would have skipped as default.

use ac2_db::{
    ac2_property_desc, ethereal_mask_with, Ac2PropertyDesc, Ac2PropertyGroup, Ac2PropertyValue,
};

use crate::collision_types::PhysicsTypeMask;

use super::PhysicsObj;

/// `CPhysicsObj::ApplyProperty`'s property groups.
const PHYSICS_GROUP: u32 = 0x4000_0001;
const RENDER_GROUP: u32 = 0x4000_0002;
const ETHEREAL_GROUP: u32 = 0x4000_0003;
const GAMEPLAY_GROUP: u32 = 0x4000_0007;
const PUSHER_GROUP: u32 = 0x4000_000D;

/// The render properties `ApplyRenderProperty` acts on, and the `GraphicsState` bit each sets
/// (`SetGraphicsState(bit, value)`); the invisible property clears the game-draw state bit
/// instead.
const RENDER_GRAPHICS_STATE_2: u32 = 0x4000_0014;
const RENDER_GRAPHICS_STATE_1: u32 = 0x4000_0015;
const RENDER_INVISIBLE: u32 = 0x4000_004D;
const RENDER_GRAPHICS_STATE_4: u32 = 0x4000_005C;
/// The physics property that names the object's physics material.
const PHYSICS_MATERIAL_PROPERTY: u32 = 0x4000_0001;
/// `PhysicsState` `GAMEDRAW_PS` (`set_gamedraw`).
const GAMEDRAW_PS: u32 = 0x8_0000;
/// Value types: bool, data file.
const TYPE_BOOL: u32 = 0x4000_0001;
const TYPE_DATA_FILE: u32 = 0x4000_0008;

fn bool_value(property: &Ac2PropertyValue) -> Option<bool> {
    (property.value_type == TYPE_BOOL)
        .then(|| property.word().map(|w| w != 0))
        .flatten()
}

/// The `MasterProperty` descs the appliers read (the installed table in the client).
type DescOf<'a> = &'a dyn Fn(u32) -> Option<Ac2PropertyDesc>;

impl PhysicsObj {
    /// `PhysicsMaterial::UpdateProperty` on `m_pPropertyInfo`: the property replaces the one of
    /// its name in its group, or is appended (a new group when it has none).
    fn keep_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) {
        let group = desc_of(property.name).map_or(0, |desc| desc.group);
        ac2_db::update_ac2_property_collection(
            &mut self.property_info,
            &[Ac2PropertyGroup {
                name: group,
                properties: vec![property.clone()],
            }],
        );
    }

    /// `CPhysicsObj::ApplyPropertyCollection`: `m_pPropertyInfo` starts empty, every group is
    /// applied, and the result is true only when every property applied.
    pub(crate) fn apply_property_collection(&mut self, groups: &[Ac2PropertyGroup]) -> bool {
        self.apply_property_collection_with(groups, &ac2_property_desc)
    }

    pub(crate) fn apply_property_collection_with(
        &mut self,
        groups: &[Ac2PropertyGroup],
        desc_of: DescOf,
    ) -> bool {
        self.property_info.clear();
        let mut applied = true;
        for group in groups {
            applied &= self.apply_property_group(group, desc_of);
        }
        applied
    }

    /// `CPhysicsObj::ApplyPropertyGroup`: every property is applied; the result is true only when
    /// every one applied (an empty group is true).
    pub(crate) fn apply_property_group(
        &mut self,
        group: &Ac2PropertyGroup,
        desc_of: DescOf,
    ) -> bool {
        let mut applied = true;
        for property in &group.properties {
            applied &= self.apply_property(property, desc_of);
        }
        applied
    }

    /// `CPhysicsObj::ApplyProperty`: by the property's group; a property of any other group (or
    /// without a desc) is not applied.
    pub(crate) fn apply_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) -> bool {
        match desc_of(property.name).map(|desc| desc.group) {
            Some(PHYSICS_GROUP) => self.apply_physics_property(property, desc_of),
            Some(RENDER_GROUP) => self.apply_render_property(property, desc_of),
            Some(ETHEREAL_GROUP) => self.apply_ethereal_property(property, desc_of),
            Some(GAMEPLAY_GROUP) => self.apply_gameplay_property(property, desc_of),
            Some(PUSHER_GROUP) => self.apply_pusher_property(property, desc_of),
            _ => false,
        }
    }

    /// `CPhysicsObj::ApplyPhysicsProperty`: kept, and the physics-material property sets
    /// `m_physics_material`.
    fn apply_physics_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) -> bool {
        self.keep_property(property, desc_of);
        if property.name != PHYSICS_MATERIAL_PROPERTY || property.value_type != TYPE_DATA_FILE {
            return false;
        }
        match property.word() {
            Some(did) => {
                self.physics_material = did;
                true
            }
            None => false,
        }
    }

    /// `CPhysicsObj::ApplyRenderProperty`: kept; the graphics-state properties set or clear
    /// their `SetGraphicsState` bit, and the invisible property sets `set_gamedraw(!invisible)`.
    fn apply_render_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) -> bool {
        self.keep_property(property, desc_of);
        let Some(value) = bool_value(property) else {
            return false;
        };
        let bit = match property.name {
            RENDER_GRAPHICS_STATE_2 => 2,
            RENDER_GRAPHICS_STATE_1 => 1,
            RENDER_GRAPHICS_STATE_4 => 4,
            RENDER_INVISIBLE => {
                if value {
                    self.state &= !GAMEDRAW_PS;
                } else {
                    self.state |= GAMEDRAW_PS;
                }
                return true;
            }
            _ => return false,
        };
        self.set_graphics_state(bit, value);
        true
    }

    /// `CPhysicsObj::SetGraphicsState`.
    pub(crate) fn set_graphics_state(&mut self, state: u32, enable: bool) {
        if enable {
            self.graphics_state |= state;
        } else {
            self.graphics_state &= !state;
        }
    }

    /// `CPhysicsObj::ApplyEtherealProperty`: an ethereal bool property sets its bit in the
    /// blocking mask (a false value sets nothing).
    fn apply_ethereal_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) -> bool {
        let bits = ethereal_mask_with(
            &[Ac2PropertyGroup {
                name: ETHEREAL_GROUP,
                properties: vec![property.clone()],
            }],
            desc_of,
        );
        if bits != 0 {
            self.blocking = PhysicsTypeMask::from_bits(self.blocking.bits() | bits);
        }
        bits != 0
    }

    /// `CPhysicsObj::ApplyGameplayProperty`: always kept.
    fn apply_gameplay_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) -> bool {
        self.keep_property(property, desc_of);
        true
    }

    /// `CPhysicsObj::ApplyPusherProperty`: kept; the pusher values are the collision manager's
    /// (`UpdatePusherValues`), so nothing applies here.
    fn apply_pusher_property(&mut self, property: &Ac2PropertyValue, desc_of: DescOf) -> bool {
        self.keep_property(property, desc_of);
        false
    }
}

#[cfg(test)]
mod tests {
    use ac2_db::{Ac2PropertyDesc, Ac2PropertyGroup, Ac2PropertyValue};

    use crate::test_support::body_at;

    fn desc_of(name: u32) -> Option<Ac2PropertyDesc> {
        let (value_type, group, data) = match name {
            0x4000_0001 => (0x4000_0008, 0x4000_0001, 0),
            0x1000_0002 => (0x4000_0001, 0x4000_0002, 0),
            0x4000_004D => (0x4000_0001, 0x4000_0002, 0),
            0x1000_0003 => (0x4000_0001, 0x4000_0003, 5),
            0x1000_0004 => (0x4000_0002, 0x4000_0007, 0),
            _ => return None,
        };
        Some(Ac2PropertyDesc {
            value_type,
            group,
            data,
            report_to_weenie: 0,
        })
    }

    fn word(name: u32, value_type: u32, word: u32) -> Ac2PropertyValue {
        Ac2PropertyValue {
            name,
            value_type,
            bytes: word.to_le_bytes().to_vec().into(),
        }
    }

    #[test]
    fn a_collection_is_applied_by_group_and_kept_as_property_info() {
        let mut body = body_at(70.0, 70.0, 50.0);
        let groups = [
            Ac2PropertyGroup {
                name: 0x4000_0001,
                properties: vec![word(0x4000_0001, 0x4000_0008, 0x3700_0042)],
            },
            Ac2PropertyGroup {
                name: 0x4000_0002,
                properties: vec![
                    word(0x4000_004D, 0x4000_0001, 1),
                    word(0x1000_0002, 0x4000_0001, 1),
                ],
            },
            Ac2PropertyGroup {
                name: 0x4000_0003,
                properties: vec![word(0x1000_0003, 0x4000_0001, 1)],
            },
        ];
        let object = body.object_mut();
        object.state |= 0x8_0000;
        // The unknown render property (no graphics-state bit) is the only one not applied.
        assert!(!object.apply_property_collection_with(&groups, &desc_of));
        assert_eq!(object.physics_material, 0x3700_0042);
        assert_eq!(
            object.state & 0x8_0000,
            0,
            "invisible clears the game-draw state"
        );
        assert_eq!(object.blocking.bits(), 1 << 5);
        // The physics and render groups are kept; the ethereal bit goes to the blocking mask only.
        assert_eq!(object.property_info.len(), 2);
        // A second collection starts the kept properties over.
        assert!(object.apply_property_collection_with(&[], &desc_of));
        assert!(object.property_info.is_empty());
    }

    #[test]
    fn gameplay_properties_apply_and_unknown_ones_do_not() {
        let mut body = body_at(70.0, 70.0, 50.0);
        let object = body.object_mut();
        assert!(object.apply_property(&word(0x1000_0004, 0x4000_0002, 7), &desc_of));
        assert!(!object.apply_property(&word(0x1000_0999, 0x4000_0002, 7), &desc_of));
        object.set_graphics_state(4, true);
        assert_eq!(object.graphics_state, 4);
        object.set_graphics_state(4, false);
        assert_eq!(object.graphics_state, 0);
    }
}
