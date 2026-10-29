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

use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_core::DataId;
use ac2_db::{CachedResource, ResourceResult, ResourceStore};
use ac2_math::Vec3;

use crate::{
    ac2_appearance_atoms, Ac2Atoms, Ac2TotalGroup, Ac2TotalVisualDesc, Ac2VisualDesc,
    GLOBAL_PART_GROUP,
};

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
        // An `EntityDesc` takes its template's visual (`ICPhysicsObj::makeObject`).
        None => Some(crate::resolve_ac2_entity_visual(store, visual)?),
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

/// `VisualDesc::GetTotalIconDesc` of `local` (its parent chain, then itself), or of the DAT
/// `VisualDesc` `visual` alone: the merged icon description, with the icon atoms of this
/// description's own global appearance keys applied on top (`IconDesc::ApplyAdditiveIconLayer`
/// of each `IconModAtom` that `PartGroupDataDesc::AugmentAtomCollection` gathers). `None` when
/// the chain does not load or there is no icon layer.
pub fn ac2_total_icon_desc(
    store: &ResourceStore,
    visual: DataId,
    local: Option<&Ac2VisualDesc>,
) -> ResourceResult<Option<crate::Ac2IconDesc>> {
    let Some(total) = total_visual_desc(store, visual, local)? else {
        return Ok(None);
    };
    let mut icon = total.icon;
    // The atoms come from this description's own global group, not the merged one.
    let dat = match local {
        Some(_) => None,
        None => store.load_cached_packed::<Ac2VisualDesc>(ac2_db::TypedResourceRef::new(
            ac2_dat::DatFileKind::Portal,
            visual,
            0,
        ))?,
    };
    let own = local.or(dat.as_deref());
    if let Some(own) = own {
        let group = Ac2TotalGroup {
            appearance: own
                .global_appearance
                .iter()
                .map(|(table, keys)| (*table, keys.to_vec()))
                .collect(),
            ..Ac2TotalGroup::default()
        };
        let mut atoms = Ac2Atoms::default();
        augment_atoms(store, &group, false, &mut atoms)?;
        for layer in &atoms.icons {
            icon.apply_additive_layer(layer);
        }
    }
    Ok((!icon.layers.is_empty()).then_some(icon))
}

/// The FX atoms of a visual's global appearance keys, in set order: what
/// `PartGroupDataDesc::FillFXTable` adds to the FX table after the part group's overrides
/// (`FXTable::AddEntry` keeps the first entry added for an id). The global group is the total
/// description's (`VisualDesc::GetTotalVisualDesc`).
pub(crate) fn global_fx_atoms(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Vec<crate::Ac2FxAtom>> {
    let Some(total) = total_visual_desc(store, visual, None)? else {
        return Ok(Vec::new());
    };
    let mut atoms = Ac2Atoms::default();
    augment_atoms(store, &total.global, false, &mut atoms)?;
    Ok(atoms.fx)
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
    // `ICPhysicsObj::makeObject`: an `EntityDesc` (a temporary object's) draws its template's visual.
    let visual = crate::resolve_ac2_entity_visual(store, visual)?;
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

/// The groups in `CPartGroup::GetHoldingLocation`'s search order: a group's own holding list
/// first (`CPartArray::GetHoldingLocation`), then each child group's search in turn, children in
/// list order and depth first. Entries are indices into `groups`; a child always follows its
/// parent in the list, which bounds the walk.
pub fn holding_search_order(groups: &[DrawnGroup]) -> Vec<usize> {
    fn visit(groups: &[DrawnGroup], index: usize, order: &mut Vec<usize>) {
        order.push(index);
        for (child, group) in groups.iter().enumerate().skip(index + 1) {
            if group.parent.is_some_and(|(parent, _)| parent == index) {
                visit(groups, child, order);
            }
        }
    }
    let mut order = Vec::with_capacity(groups.len());
    if !groups.is_empty() {
        visit(groups, 0, &mut order);
    }
    order
}

/// Every part group's bones at rest, by name, in `CPartGroup::FindBoneType`'s search order: a
/// group's own bones, then each child group's search in turn (`ObjectSkeleton::named_bones` at the
/// rest pose). A bone is its model-space transform in its group, scale stripped as
/// `CPartArray::m_boneMatrices` carry none, taken through the group's matrix: the connection
/// point's offset in the parent's bone, then the parent's matrix (`CPartGroup::SetParentOffset`,
/// `CPartGroup::RefreshMatrices`). Empty when the visual draws no groups.
pub fn rest_named_bones(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Vec<(u32, Ac2Transform)>> {
    let Some(groups) = resolve_drawn_groups(store, visual, None)? else {
        return Ok(Vec::new());
    };
    let mut setups: Vec<CachedResource<Ac2Setup>> = Vec::with_capacity(groups.len());
    for group in &groups {
        let Some(setup) = crate::ac2_visual_particles::load::<Ac2Setup>(store, group.setup)? else {
            return Ok(Vec::new());
        };
        setups.push(setup);
    }
    // Parents come before their children (`drawn_groups` walks down the tree).
    let mut matrices: Vec<Ac2Transform> = Vec::with_capacity(groups.len());
    let mut bones: Vec<Vec<Ac2Transform>> = Vec::with_capacity(groups.len());
    for (index, setup) in setups.iter().enumerate() {
        bones.push(setup.world_pose(&default_locals(setup)));
        let matrix = match groups[index].parent {
            None => Ac2Transform::IDENTITY,
            Some((parent, connection)) => {
                let (Some(parent_setup), Some(parent_matrix)) =
                    (setups.get(parent), matrices.get(parent).copied())
                else {
                    return Ok(Vec::new());
                };
                match parent_setup
                    .connection_point(connection)
                    .filter(|point| point.bone < parent_setup.bones.len())
                {
                    Some(point) => {
                        parent_matrix.compose(&bones[parent][point.bone].compose(&point.transform))
                    }
                    None => parent_matrix,
                }
            }
        };
        matrices.push(matrix);
    }
    let mut found = Vec::new();
    if !groups.is_empty() {
        collect_named(&groups, &setups, &bones, &matrices, 0, &mut found);
    }
    Ok(found)
}

/// `CPartGroup::FindBoneType`'s walk: the group's own bones, then each child group's walk, in
/// group order.
fn collect_named(
    groups: &[DrawnGroup],
    setups: &[CachedResource<Ac2Setup>],
    bones: &[Vec<Ac2Transform>],
    matrices: &[Ac2Transform],
    group: usize,
    found: &mut Vec<(u32, Ac2Transform)>,
) {
    for (index, bone) in setups[group].bones.iter().enumerate() {
        found.push((bone.name, matrices[group].compose(&bones[group][index])));
    }
    for child in group + 1..groups.len() {
        if groups[child]
            .parent
            .is_some_and(|(parent, _)| parent == group)
        {
            collect_named(groups, setups, bones, matrices, child, found);
        }
    }
}

/// Each bone's transform in placement frame 0 (`SkeletonGroup::new`), scale stripped; identity
/// where the frame lacks the bone.
fn default_locals(setup: &Ac2Setup) -> Vec<Ac2Transform> {
    let frame = setup.placements.iter().find(|frame| frame.kind == 0);
    (0..setup.bones.len())
        .map(|index| {
            let local = frame
                .and_then(|frame| frame.bones.get(index))
                .copied()
                .unwrap_or(Ac2Transform::IDENTITY);
            Ac2Transform {
                scale: Vec3::ONE,
                ..local
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ac2_core::DataId;

    use super::{holding_search_order, DrawnGroup};

    fn group(parent: Option<usize>) -> DrawnGroup {
        DrawnGroup {
            key: 0,
            setup: DataId::new(0x0400_0001),
            parent: parent.map(|parent| (parent, 0)),
            atoms: Default::default(),
        }
    }

    /// Root 0 with children 1 and 2; 3 hangs from 1. Retail searches 0, then 1 and its child 3,
    /// then 2.
    #[test]
    fn the_holding_search_is_own_list_then_each_child_depth_first() {
        let groups = [group(None), group(Some(0)), group(Some(0)), group(Some(1))];
        assert_eq!(holding_search_order(&groups), vec![0, 1, 3, 2]);
        assert!(holding_search_order(&[]).is_empty());
    }
}
