//! Building a part-group tree from the DATs (`CPartGroup::InitVisualDesc` over a visual's total
//! description). The Setups are decoded by `ac2-assets` and `ac2-anim`; nothing is re-decoded
//! here. Run on a loader thread: the frame thread never reads files.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::Ac2Setup;
use ac2_assets::{resolve_drawn_groups, Ac2Mesh, Ac2PhysicsMesh, Ac2SetupPhysics};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};

use crate::bsp::BspTree;
use crate::part_array::{PartMesh, PartSetup};
use crate::part_group::PartGroupTree;

fn typed(did: DataId) -> TypedResourceRef {
    TypedResourceRef::new(DatFileKind::Portal, did, 0)
}

/// One `CPhysicsPart`'s mesh: its type, whether it has a physics BSP and its first degrade's
/// triangles.
fn load_part_mesh(store: &ResourceStore, did: DataId) -> ResourceResult<Option<PartMesh>> {
    let Some(mesh) = store.load_cached_packed::<Ac2Mesh>(typed(did))? else {
        return Ok(None);
    };
    let has_bsp = store
        .load_cached_packed::<Ac2PhysicsMesh>(typed(did))?
        .as_deref()
        .and_then(BspTree::from_mesh)
        .is_some();
    let degrade = mesh.highest_detail_degrade();
    let first = mesh
        .fragments
        .iter()
        .map(|fragment| fragment.degrade)
        .min()
        .unwrap_or(degrade);
    let mut triangles = Vec::new();
    for fragment in mesh.fragments.iter().filter(|f| f.degrade == first) {
        for corners in fragment.indices.chunks_exact(3) {
            let at = |i: u32| mesh.vertices.get(i as usize).map(|v| v.position);
            if let (Some(a), Some(b), Some(c)) = (at(corners[0]), at(corners[1]), at(corners[2])) {
                triangles.push([a, b, c]);
            }
        }
    }
    Ok(Some(PartMesh {
        mesh_type: mesh.mesh_type,
        has_physics_bsp: has_bsp,
        has_matrix_weights: mesh.skin.is_some(),
        triangles: Arc::from(triangles),
    }))
}

/// A `CSetup` as a part array reads it: the bones and locations, the collision primitives and one
/// part per listed mesh (`CPartArray::InitParts` makes none for mesh 0).
pub fn load_part_setup(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<Arc<PartSetup>>> {
    let Some(setup) = store.load_cached_packed::<Ac2Setup>(typed(did))? else {
        return Ok(None);
    };
    let Some(physics) = store.load_cached_packed::<Ac2SetupPhysics>(typed(did))? else {
        return Ok(None);
    };
    let mut parts = Vec::with_capacity(setup.meshes.len());
    for &mesh in setup.meshes.iter() {
        if mesh.raw() == 0 {
            parts.push(None);
        } else {
            parts.push(load_part_mesh(store, mesh)?);
        }
    }
    Ok(Some(Arc::new(PartSetup {
        did,
        setup: Arc::new((*setup).clone()),
        physics: Arc::new((*physics).clone()),
        parts,
    })))
}

/// The part-group tree of a `VisualDesc` (or bare `CSetup`): the groups `CPartGroup::Init`
/// makes, each with its Setup, hung from its parent's connection point, at placement frame 0 with
/// the matrices refreshed. `None` when the visual draws no groups or a Setup does not load.
pub fn load_part_group_tree(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Option<PartGroupTree>> {
    let Some(groups) = resolve_drawn_groups(store, visual, None)? else {
        return Ok(None);
    };
    let mut cache: HashMap<u32, Option<Arc<PartSetup>>> = HashMap::new();
    let mut tree = PartGroupTree::new();
    for (index, group) in groups.iter().enumerate() {
        let setup = match cache.get(&group.setup.raw()) {
            Some(setup) => setup.clone(),
            None => {
                let setup = load_part_setup(store, group.setup)?;
                cache.insert(group.setup.raw(), setup.clone());
                setup
            }
        };
        let Some(setup) = setup else {
            return Ok(None);
        };
        let slot = if index == 0 { 0 } else { tree.new_group() };
        if !tree.init_setup(slot, setup) {
            return Ok(None);
        }
        if let Some(group_mut) = tree.group_mut(slot) {
            group_mut.key = group.key;
        }
        if let Some((parent, connection)) = group.parent {
            if !tree.add_child(parent, slot, connection) {
                return Ok(None);
            }
        }
    }
    tree.cache_has_physics_bsp(0);
    tree.check_refresh_matrices();
    Ok(Some(tree))
}
