//! Appearance and quality messages as a 7160 server packs them (test tooling): `VisualDesc` and
//! `PartGroupDataDesc` in `StreamPack` order, `CreateObject` with a full visual description,
//! `UpdateVisualDesc` (`0x1009F`), the quality updates and `Contain` (`0x100CB`).

use ac2_net::protocol::{IconLayer, PartGroupDesc, VisualDesc};
use ac2_weenie::{Appearance, PartGroupAppearance};

use crate::{words, ObjectSpawn};

/// A `PHashTable` header: the count in the low 24 bits, the bucket-size index above.
fn hash_header(count: usize) -> u32 {
    count as u32 | (3 << 24)
}

fn float_table(b: &mut Vec<u8>, pairs: &[(u32, f32)]) {
    words(b, &[hash_header(pairs.len())]);
    for &(key, value) in pairs {
        words(b, &[key]);
        b.extend(value.to_le_bytes());
    }
}

/// `PartGroupDataDesc::StreamPack`: the pack bitfield (from the fields set), key, parent key,
/// connection point, Setup, anim map, the appearance hash, the FX table and the startup FX
/// (FX overrides are not written).
pub fn part_group_bytes(b: &mut Vec<u8>, g: &PartGroupDesc) {
    let mut bits = 0;
    for (bit, value) in [
        (0x1, g.key),
        (0x2, g.parent_key),
        (0x4, g.connection_point),
        (0x8, g.setup),
        (0x10, g.anim_map),
    ] {
        if value != 0 {
            bits |= bit;
        }
    }
    if !g.appearance.is_empty() {
        bits |= 0x20;
    }
    if g.fx_table != 0 {
        bits |= 0x40;
    }
    if !g.startup_fx.is_empty() {
        bits |= 0x80;
    }
    words(b, &[bits]);
    for value in [g.key, g.parent_key, g.connection_point, g.setup, g.anim_map] {
        if value != 0 {
            words(b, &[value]);
        }
    }
    if !g.appearance.is_empty() {
        words(b, &[hash_header(g.appearance.len())]);
        for (table, keys) in &g.appearance {
            words(b, &[*table]);
            float_table(b, keys);
        }
    }
    if g.fx_table != 0 {
        words(b, &[g.fx_table]);
    }
    if !g.startup_fx.is_empty() {
        float_table(b, &g.startup_fx);
    }
}

/// `VisualDesc::StreamPack` (no DID): the pack bitfield from the fields set, then each field.
pub fn visual_desc_bytes(d: &VisualDesc) -> Vec<u8> {
    let mut b = Vec::new();
    let mut bits = 0;
    let dids = [
        (0x2, d.parent),
        (0x4, d.motion_interp),
        (0x8, d.behavior_table),
        (0x10, d.modes),
    ];
    for (bit, value) in dids {
        if value.is_some() {
            bits |= bit;
        }
    }
    let flags = [
        (0x20, d.scale.is_some()),
        (0x40, d.child_scale.is_some()),
        (0x400, d.particle_scale.is_some()),
        (0x80, !d.icon.is_empty()),
        (0x100, d.global_mods.is_some()),
        (0x200, !d.part_groups.is_empty()),
    ];
    for (bit, set) in flags {
        if set {
            bits |= bit;
        }
    }
    words(&mut b, &[bits]);
    for (_, value) in dids {
        if let Some(value) = value {
            words(&mut b, &[value]);
        }
    }
    for v in d.scale.iter().flatten() {
        b.extend(v.to_le_bytes());
    }
    for v in d.child_scale.iter().chain(d.particle_scale.iter()) {
        b.extend(v.to_le_bytes());
    }
    if !d.icon.is_empty() {
        words(&mut b, &[d.icon.len() as u32]);
        for layer in &d.icon {
            words(&mut b, &[layer.layer, layer.image, layer.color]);
        }
    }
    if let Some(global) = &d.global_mods {
        part_group_bytes(&mut b, global);
    }
    if !d.part_groups.is_empty() {
        words(&mut b, &[hash_header(d.part_groups.len())]);
        for (key, group) in &d.part_groups {
            words(&mut b, &[*key]);
            part_group_bytes(&mut b, group);
        }
    }
    b
}

/// A local appearance (`CPhysicsObj`'s `m_local_vdesc`) as the `VisualDesc` a server sends:
/// the parent, the scales and icon layers, the global and the part-group modifications. The
/// server computes it from a character's creation result (`gmCharGenResult::InqAppliedAppearanceVDesc`).
pub fn appearance_visual(appearance: &Appearance) -> VisualDesc {
    let did = |value: u32| (value != 0).then_some(value);
    VisualDesc {
        parent: did(appearance.parent),
        motion_interp: did(appearance.motion_interp),
        behavior_table: did(appearance.behavior_table),
        modes: did(appearance.modes),
        scale: Some(appearance.scale),
        child_scale: Some(appearance.child_scale),
        particle_scale: Some(appearance.particle_scale),
        icon: appearance
            .icon
            .iter()
            .map(|&(layer, image, color)| IconLayer {
                layer,
                image,
                color,
            })
            .collect(),
        global_mods: Some(part_group(&appearance.global)),
        part_groups: appearance
            .part_groups
            .iter()
            .map(|group| (group.key, part_group(group)))
            .collect(),
        ..VisualDesc::default()
    }
}

fn part_group(group: &PartGroupAppearance) -> PartGroupDesc {
    PartGroupDesc {
        key: group.key,
        parent_key: group.parent_key,
        connection_point: group.connection_point,
        setup: group.setup,
        anim_map: group.anim_map,
        appearance: group.appearance.clone(),
        fx_table: group.fx_table,
        startup_fx: group.startup_fx.clone(),
        ..PartGroupDesc::default()
    }
}

/// `CreateObject` with `visual` as its `VisualDesc` (the spawn's `visual` field is ignored).
pub fn create_object_with_visual(object: &ObjectSpawn, visual: &VisualDesc) -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(object.id.to_le_bytes());
    b.extend(visual_desc_bytes(visual));
    words(&mut b, &[0x8000]);
    b.extend(object.instance.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    words(&mut b, &[object.cell]);
    for v in object.origin.iter().chain(object.rotation.iter()) {
        b.extend(v.to_le_bytes());
    }
    for stamp in object.stamps {
        b.extend(stamp.to_le_bytes());
    }
    words(&mut b, &[0]);
    b
}

/// `UpdateVisualDesc` (`0x1009F`): the sender, then the delta `VisualDesc`.
pub fn update_visual_desc_bytes(id: u64, instance: u16, delta: &VisualDesc) -> Vec<u8> {
    let mut b = 0x0001_009Fu32.to_le_bytes().to_vec();
    b.extend(id.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    b.extend(visual_desc_bytes(delta));
    b
}

/// A quality update: the opcode, the sender for the visual form, the quality type, its value
/// bytes (already in their pack layout), aligned.
pub fn quality_bytes(
    opcode: u32,
    object: Option<(u64, u16)>,
    quality: u32,
    value: &[u8],
) -> Vec<u8> {
    let mut b = opcode.to_le_bytes().to_vec();
    if let Some((id, instance)) = object {
        b.extend(id.to_le_bytes());
        b.extend(instance.to_le_bytes());
        b.extend(0u16.to_le_bytes());
    }
    words(&mut b, &[quality]);
    b.extend(value);
    while !b.len().is_multiple_of(4) {
        b.push(0);
    }
    b
}

/// `Contain` (`0x100CB`): the child with its instance and position stamp.
pub fn contain_bytes(child: u64, instance: u16, position_stamp: u16) -> Vec<u8> {
    let mut b = 0x0001_00CBu32.to_le_bytes().to_vec();
    b.extend(child.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(position_stamp.to_le_bytes());
    b
}
