//! What an object draws: its total `VisualDesc` and each part group's appearance atoms.
//!
//! `VisualDesc::GetTotalVisualDesc` builds the drawn description into an empty one: the parent
//! chain from the DATs root first, then the object's local description on top
//! (`GetTotalVisualDescInternal`, each step `ApplyAdditiveVDesc`), then
//! `ModifyPGDTreeBasedOnApr` swaps in the part groups the global appearance keys name
//! (`PGDReplacementAtom`s: a child of the root with that key is `SwapPGDD`'d, any other is
//! `AddPGDDesc`'d). `CPartGroup::Init` walks the tree from the root, each group's `SubInit`
//! loading its Setup, hanging it from its parent's connection point (`AddChild`) and applying
//! `VisualDesc::BuildAtomCollection` for its key: the global modifiers' atoms, then the group's
//! own (`PartGroupDataDesc::AugmentAtomCollection` over its `AppearanceInfo` hash, each applied
//! key with a scalar other than -1 through `AppearanceTable::GetAtoms`).

use ac2_assets::{ac2_appearance_atoms, Ac2AppearanceHash, Ac2Atoms, Ac2VisualDesc};
use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};
use ac2_net::protocol::{PartGroupDesc, VisualDesc};
use ac2_weenie::{Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};

const SETUP_PREFIX: u32 = 0x0400_0000;
const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
/// `VisualDesc` parent chains are short; this bounds a malformed cycle.
const MAX_CHAIN: usize = 16;
/// `AppearanceInfo::AugmentAtomCollection` skips keys whose scalar is this.
const UNAPPLIED: f32 = -1.0;

/// A DAT `VisualDesc` as the additive delta `ApplyAdditiveVDesc` reads (FX override variants and
/// icon layers, which do not change what is drawn, are left out).
fn as_delta(desc: &Ac2VisualDesc) -> VisualDesc {
    let hash = |hash: &Ac2AppearanceHash| -> Vec<(u32, Vec<(u32, f32)>)> {
        hash.iter()
            .map(|(table, keys)| (table.raw(), keys.to_vec()))
            .collect()
    };
    let raw = |did: Option<DataId>| did.map(DataId::raw);
    let global = PartGroupDesc {
        appearance: hash(&desc.global_appearance),
        fx_table: desc.fx.fx_table.map_or(0, DataId::raw),
        startup_fx: desc.fx.startup.to_vec(),
        ..PartGroupDesc::default()
    };
    VisualDesc {
        parent: raw(desc.parent),
        motion_interp: raw(desc.motion_interp),
        behavior_table: raw(desc.behavior_table),
        modes: raw(desc.modes),
        scale: desc.scale.map(|scale| scale.to_array()),
        child_scale: desc.child_scale,
        particle_scale: desc.particle_scale,
        global_mods: Some(global),
        part_groups: desc
            .part_groups
            .iter()
            .zip(desc.part_group_appearance.iter())
            .map(|(group, appearance)| {
                (
                    group.key,
                    PartGroupDesc {
                        key: group.key,
                        parent_key: group.parent_key,
                        connection_point: group.connection_point,
                        setup: group.setup.map_or(0, DataId::raw),
                        anim_map: group.anim_map.map_or(0, DataId::raw),
                        appearance: hash(appearance),
                        ..PartGroupDesc::default()
                    },
                )
            })
            .collect(),
        ..VisualDesc::default()
    }
}

/// One part group as `CPartGroup::Init` builds it.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnGroup {
    pub key: u32,
    pub setup: DataId,
    /// The parent group (an index into the list) and the connection point it hangs from.
    pub parent: Option<(usize, u32)>,
    /// `BuildAtomCollection` for the group's key.
    pub atoms: Ac2Atoms,
}

/// `VisualDesc::GetTotalVisualDesc` of `local` (its parent chain, then itself), or of the DAT
/// `VisualDesc` `visual` alone. A link of the chain that does not load ends it, as
/// `GetTotalVisualDescInternal` fails there.
pub fn total_visual_desc(
    store: &ResourceStore,
    visual: DataId,
    local: Option<&Appearance>,
) -> ResourceResult<Option<Appearance>> {
    let mut chain = Vec::new();
    let mut current = local.map_or(visual.raw(), |local| local.parent);
    for _ in 0..MAX_CHAIN {
        if current == 0 {
            break;
        }
        if current & 0xFF00_0000 != VISUAL_DESC_PREFIX {
            return Ok(None);
        }
        let Some(desc) = store.load_cached_packed::<Ac2VisualDesc>(
            ac2_db::TypedResourceRef::new(ac2_dat::DatFileKind::Portal, DataId::new(current), 0),
        )?
        else {
            return Ok(None);
        };
        current = desc.parent.map_or(0, DataId::raw);
        chain.push(as_delta(&desc));
    }
    let mut total = Appearance::default();
    for desc in chain.iter().rev() {
        if !total.apply_additive(desc) {
            return Ok(None);
        }
    }
    if let Some(local) = local {
        if !total.apply_additive(&local.to_visual_desc()) {
            return Ok(None);
        }
    }
    modify_tree_by_appearance(store, &mut total)?;
    Ok(Some(total))
}

/// `PartGroupDataDesc::AugmentAtomCollection` of one group's appearance hash.
fn augment_atoms(
    store: &ResourceStore,
    group: &PartGroupAppearance,
    only_part_groups: bool,
    atoms: &mut Ac2Atoms,
) -> ResourceResult<()> {
    for (table, keys) in &group.appearance {
        if *table == 0 {
            continue;
        }
        for &(key, scalar) in keys {
            if scalar != UNAPPLIED {
                ac2_appearance_atoms(
                    store,
                    DataId::new(*table),
                    key,
                    scalar,
                    only_part_groups,
                    atoms,
                )?;
            }
        }
    }
    Ok(())
}

/// `VisualDesc::ModifyPGDTreeBasedOnApr`.
fn modify_tree_by_appearance(store: &ResourceStore, total: &mut Appearance) -> ResourceResult<()> {
    let mut atoms = Ac2Atoms::default();
    augment_atoms(store, &total.global, true, &mut atoms)?;
    for atom in &atoms.part_groups {
        let group = &atom.group;
        let replacement = PartGroupAppearance {
            key: group.key,
            parent_key: group.parent_key,
            connection_point: group.connection_point,
            setup: group.setup.map_or(0, DataId::raw),
            anim_map: group.anim_map.map_or(0, DataId::raw),
            appearance: atom
                .appearance
                .iter()
                .map(|(table, keys)| (table.raw(), keys.to_vec()))
                .collect(),
            ..PartGroupAppearance::default()
        };
        // `PartGroupDataDesc::FindPGDDWithPartGroupKey` looks among the root's children only.
        let swap = total
            .root
            .is_some_and(|root| total.children(root).contains(&group.key));
        if swap {
            total.swap_part_group(group.key, &replacement);
        } else {
            total.add_part_group(&replacement);
        }
    }
    Ok(())
}

/// `CPartGroup::InitVisualDesc` over the total description: the groups from the root down, each
/// with its atoms. `None` when the tree has no root or a group names no Setup (`SubInit` fails,
/// and with it the whole build).
pub fn drawn_groups(
    store: &ResourceStore,
    total: &Appearance,
) -> ResourceResult<Option<Vec<DrawnGroup>>> {
    let Some(root) = total.root else {
        return Ok(None);
    };
    let mut groups = Vec::new();
    // Depth first, children in join order (`CPartGroup::Init`).
    let mut pending = vec![(root, None)];
    while let Some((key, parent)) = pending.pop() {
        let Some(group) = total.group(key) else {
            return Ok(None);
        };
        if key == GLOBAL_PART_GROUP || key == 0 || group.setup == 0 {
            return Ok(None);
        }
        // `VisualDesc::BuildAtomCollection`: the global modifiers, then the group's own.
        let mut atoms = Ac2Atoms::default();
        augment_atoms(store, &total.global, false, &mut atoms)?;
        augment_atoms(store, group, false, &mut atoms)?;
        let index = groups.len();
        groups.push(DrawnGroup {
            key,
            setup: DataId::new(group.setup),
            parent: parent.map(|parent| (parent, group.connection_point)),
            atoms,
        });
        for child in total.children(key).iter().rev() {
            pending.push((*child, Some(index)));
        }
    }
    Ok(Some(groups))
}

/// The groups a visual draws: a bare `CSetup` is one group with no atoms; a `VisualDesc` (with
/// the object's local description on top, when it has one) goes through
/// [`total_visual_desc`] and [`drawn_groups`].
pub fn resolve_drawn_groups(
    store: &ResourceStore,
    visual: DataId,
    local: Option<&Appearance>,
) -> ResourceResult<Option<Vec<DrawnGroup>>> {
    if local.is_none() && visual.raw() & 0xFF00_0000 == SETUP_PREFIX {
        return Ok(Some(vec![DrawnGroup {
            key: 0,
            setup: visual,
            parent: None,
            atoms: Ac2Atoms::default(),
        }]));
    }
    match total_visual_desc(store, visual, local)? {
        Some(total) => drawn_groups(store, &total),
        None => Ok(None),
    }
}
