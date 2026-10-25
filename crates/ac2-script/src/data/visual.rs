//! A DAT `VisualDesc` (DB type 0x40000021) as the script's appearance: the parent it adds to,
//! the scales, the part groups with their appearance tables and the global group, in the model
//! `VisualDesc::DBObj` decodes to (ac2-weenie's [`Appearance`]). The DAT record's icon and FX
//! overrides are not carried (the appearance efuns do not read them).

use ac2_assets::Ac2VisualDesc;
use ac2_core::DataId;
use ac2_weenie::{Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};

fn raw(id: Option<DataId>) -> u32 {
    id.map_or(0, DataId::raw)
}

/// The DAT description as an [`Appearance`] (`VisualDesc` decoded from the record).
pub(crate) fn appearance_of_dat(desc: &Ac2VisualDesc) -> Appearance {
    let tables = |hash: &ac2_assets::Ac2AppearanceHash| -> Vec<(u32, Vec<(u32, f32)>)> {
        hash.iter()
            .map(|(table, keys)| (table.raw(), keys.to_vec()))
            .collect()
    };
    let part_groups: Vec<PartGroupAppearance> = desc
        .part_groups
        .iter()
        .enumerate()
        .map(|(i, group)| PartGroupAppearance {
            key: group.key,
            parent_key: group.parent_key,
            connection_point: group.connection_point,
            setup: raw(group.setup),
            anim_map: raw(group.anim_map),
            appearance: desc
                .part_group_appearance
                .get(i)
                .map_or_else(Vec::new, tables),
            ..PartGroupAppearance::default()
        })
        .collect();
    let global = PartGroupAppearance {
        key: GLOBAL_PART_GROUP,
        appearance: tables(&desc.global_appearance),
        fx_table: raw(desc.fx.fx_table),
        startup_fx: desc.fx.startup.to_vec(),
        ..PartGroupAppearance::default()
    };
    let mut appearance = Appearance {
        parent: raw(desc.parent),
        motion_interp: raw(desc.motion_interp),
        behavior_table: raw(desc.behavior_table),
        modes: raw(desc.modes),
        scale: desc.scale.map_or([1.0; 3], |v| v.to_array()),
        child_scale: desc.child_scale.unwrap_or(1.0),
        particle_scale: desc.particle_scale.unwrap_or(1.0),
        global,
        part_groups,
        ..Appearance::default()
    };
    appearance.reset_tree();
    appearance
}
