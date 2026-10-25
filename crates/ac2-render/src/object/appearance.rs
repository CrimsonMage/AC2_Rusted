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

use ac2_assets::{
    ac2_appearance_atoms, Ac2Atoms, Ac2TotalGroup, Ac2TotalVisualDesc, Ac2VisualDesc,
    GLOBAL_PART_GROUP,
};
use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

const SETUP_PREFIX: u32 = 0x0400_0000;
const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
/// `VisualDesc` parent chains are short; this bounds a malformed cycle.
const MAX_CHAIN: usize = 16;
/// `AppearanceInfo::AugmentAtomCollection` skips keys whose scalar is this.
const UNAPPLIED: f32 = -1.0;

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
/// `VisualDesc` `visual` alone. A link of the chain that does not load fails it, as
/// `GetTotalVisualDescInternal` does.
pub fn total_visual_desc(
    store: &ResourceStore,
    visual: DataId,
    local: Option<&Ac2VisualDesc>,
) -> ResourceResult<Option<Ac2TotalVisualDesc>> {
    let mut chain = Vec::new();
    let mut current = match local {
        Some(local) => local.parent,
        None => Some(visual),
    };
    for _ in 0..MAX_CHAIN {
        let Some(did) = current else {
            break;
        };
        if did.raw() & 0xFF00_0000 != VISUAL_DESC_PREFIX {
            return Ok(None);
        }
        let Some(desc) = store.load_cached_packed::<Ac2VisualDesc>(
            ac2_db::TypedResourceRef::new(ac2_dat::DatFileKind::Portal, did, 0),
        )?
        else {
            return Ok(None);
        };
        current = desc.parent;
        chain.push(desc);
    }
    let mut total = Ac2TotalVisualDesc::default();
    for desc in chain.iter().rev() {
        if !total.apply_additive(desc) {
            return Ok(None);
        }
    }
    if let Some(local) = local {
        if !total.apply_additive(local) {
            return Ok(None);
        }
    }
    modify_tree_by_appearance(store, &mut total)?;
    Ok(Some(total))
}

/// `PartGroupDataDesc::AugmentAtomCollection` of one group's appearance hash.
fn augment_atoms(
    store: &ResourceStore,
    group: &Ac2TotalGroup,
    only_part_groups: bool,
    atoms: &mut Ac2Atoms,
) -> ResourceResult<()> {
    for (table, keys) in &group.appearance {
        if table.is_null() {
            continue;
        }
        for &(key, scalar) in keys {
            if scalar != UNAPPLIED {
                ac2_appearance_atoms(store, *table, key, scalar, only_part_groups, atoms)?;
            }
        }
    }
    Ok(())
}

/// `VisualDesc::ModifyPGDTreeBasedOnApr`.
fn modify_tree_by_appearance(
    store: &ResourceStore,
    total: &mut Ac2TotalVisualDesc,
) -> ResourceResult<()> {
    let mut atoms = Ac2Atoms::default();
    augment_atoms(store, &total.global, true, &mut atoms)?;
    for atom in &atoms.part_groups {
        let group = &atom.group;
        let replacement = Ac2TotalGroup {
            key: group.key,
            parent_key: group.parent_key,
            connection_point: group.connection_point,
            setup: group.setup,
            anim_map: group.anim_map,
            appearance: atom
                .appearance
                .iter()
                .map(|(table, keys)| (*table, keys.to_vec()))
                .collect(),
        };
        // `PartGroupDataDesc::FindPGDDWithPartGroupKey` looks among the root's children only.
        let swap = total
            .root
            .is_some_and(|root| total.children(root).contains(&group.key));
        if swap {
            total.swap_group(group.key, &replacement);
        } else {
            total.add_group(&replacement);
        }
    }
    Ok(())
}

/// `CPartGroup::InitVisualDesc` over the total description: the groups from the root down, each
/// with its atoms. `None` when the tree has no root or a group names no Setup (`SubInit` fails,
/// and with it the whole build).
pub fn drawn_groups(
    store: &ResourceStore,
    total: &Ac2TotalVisualDesc,
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
        let Some(setup) = group.setup.filter(|_| key != GLOBAL_PART_GROUP && key != 0) else {
            return Ok(None);
        };
        // `VisualDesc::BuildAtomCollection`: the global modifiers, then the group's own.
        let mut atoms = Ac2Atoms::default();
        augment_atoms(store, &total.global, false, &mut atoms)?;
        augment_atoms(store, group, false, &mut atoms)?;
        let index = groups.len();
        groups.push(DrawnGroup {
            key,
            setup,
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
    local: Option<&Ac2VisualDesc>,
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
