//! An object's local `VisualDesc` (`CPhysicsObj::m_local_vdesc`, the session's
//! [`Appearance`]) as the additive delta the renderer merges onto its DAT chain
//! (`VisualDesc::GetTotalVisualDesc`, `ac2_render::object::load_object_model_with`).

use std::sync::Arc;

use crate::appearance::{Appearance, PartGroupAppearance};
use ac2_assets::{Ac2PartGroup, Ac2VisualDesc, Ac2VisualFx};
use ac2_core::DataId;
use ac2_math::Vec3;

fn did(raw: u32) -> Option<DataId> {
    (raw != 0).then(|| DataId::new(raw))
}

fn hash(group: &PartGroupAppearance) -> ac2_assets::Ac2AppearanceHash {
    group
        .appearance
        .iter()
        .map(|(table, keys)| (DataId::new(*table), Arc::from(keys.as_slice())))
        .collect()
}

/// The local description as a delta: its parent, DIDs, scales, global appearance keys and part
/// groups in join order, and its icon layers (its FX are left out: they are not drawn).
pub fn local_visual_desc(appearance: &Appearance) -> Ac2VisualDesc {
    Ac2VisualDesc {
        parent: did(appearance.parent),
        motion_interp: did(appearance.motion_interp),
        behavior_table: did(appearance.behavior_table),
        modes: did(appearance.modes),
        scale: Some(Vec3::from_array(appearance.scale)),
        child_scale: Some(appearance.child_scale),
        particle_scale: Some(appearance.particle_scale),
        icon: ac2_assets::Ac2IconDesc {
            layers: appearance
                .icon
                .iter()
                .map(|&(layer_id, image, shift_color)| ac2_assets::Ac2IconLayer {
                    layer_id,
                    image,
                    shift_color,
                })
                .collect(),
        },
        part_groups: appearance
            .part_groups
            .iter()
            .map(|group| Ac2PartGroup {
                key: group.key,
                parent_key: group.parent_key,
                connection_point: group.connection_point,
                setup: did(group.setup),
                anim_map: did(group.anim_map),
            })
            .collect(),
        part_group_appearance: appearance.part_groups.iter().map(hash).collect(),
        global_appearance: hash(&appearance.global),
        fx: Ac2VisualFx::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_local_description_carries_its_tree_and_keys() {
        let mut local = Appearance {
            parent: 0x1F00_0023,
            ..Appearance::default()
        };
        local.global.appearance = vec![(0x2000_0000, vec![(1, 0.5)])];
        local.part_groups.push(PartGroupAppearance {
            key: 0x8100_0001,
            setup: 0x0400_0029,
            appearance: vec![(0x2000_0002, vec![(0xF, 0.25)])],
            ..PartGroupAppearance::default()
        });
        let delta = local_visual_desc(&local);
        assert_eq!(delta.parent, Some(DataId::new(0x1F00_0023)));
        assert_eq!(delta.part_groups[0].setup, Some(DataId::new(0x0400_0029)));
        assert_eq!(&*delta.part_group_appearance[0][0].1, &[(0xF, 0.25)]);
        assert_eq!(delta.global_appearance[0].0, DataId::new(0x2000_0000));
        assert_eq!(delta.motion_interp, None);
    }
}
