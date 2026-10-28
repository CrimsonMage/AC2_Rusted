//! The part-group tree on the real human male body (only with `AC2_DAT_DIR`). Every expectation
//! is an invariant of the data or a cross-check against the render side's separate reading of
//! the same Setups.

use std::sync::Arc;

use ac2_assets::{holding_search_order, resolve_drawn_groups, rest_named_bones, Ac2SetupPhysics};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};

use crate::los::Ray;
use crate::part_load::{load_part_group_tree, load_part_setup};
use crate::vector::Vector;

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

#[test]
fn the_human_setup_has_its_spheres_and_bounds() {
    let Some(store) = store() else {
        return;
    };
    let groups = resolve_drawn_groups(&store, DataId::new(HUMAN_MALE_VISUAL), None)
        .unwrap()
        .expect("the human draws groups");
    let setup = load_part_setup(&store, groups[0].setup)
        .unwrap()
        .expect("the root Setup loads");
    let raw = store
        .load_cached_packed::<Ac2SetupPhysics>(TypedResourceRef::new(
            DatFileKind::Portal,
            groups[0].setup,
            0,
        ))
        .unwrap()
        .unwrap();
    assert_eq!(setup.physics.as_ref(), &*raw);
    let (b, s) = (raw.bounding_sphere, raw.selection_sphere);
    assert!(
        b.radius >= 0.0 && s.radius > 0.0,
        "bounding {b:?} selection {s:?} height {} box {:?} {:?}",
        raw.height,
        raw.bounds_min,
        raw.bounds_max
    );
    assert!(raw.height > 1.0 && raw.height < 4.0);
    // The selection sphere stands on the body: its centre is within the Setup's height.
    assert!(
        s.origin.z >= -0.5 && s.origin.z <= raw.height + 0.5,
        "{s:?}"
    );
    // The box has volume.
    assert!(raw.bounds_min.x < raw.bounds_max.x && raw.bounds_min.z < raw.bounds_max.z);
}

#[test]
fn the_tree_holds_every_group_and_the_render_sides_holding_search_order() {
    let Some(store) = store() else {
        return;
    };
    let visual = DataId::new(HUMAN_MALE_VISUAL);
    let groups = resolve_drawn_groups(&store, visual, None).unwrap().unwrap();
    let tree = load_part_group_tree(&store, visual)
        .unwrap()
        .expect("the human's tree loads");
    assert_eq!(tree.indices().len(), groups.len());
    // The holding list in `CPartGroup::GetHoldingLocation`'s order is the one the client builds
    // from `ac2_assets::holding_search_order`.
    let order = holding_search_order(&groups);
    let mut expected = Vec::new();
    for group in order {
        let setup = load_part_setup(&store, groups[group].setup)
            .unwrap()
            .unwrap();
        for location in setup.setup.holding_points.iter() {
            expected.push((group, location.id, location.bone));
        }
    }
    let found: Vec<_> = tree
        .holdings()
        .iter()
        .map(|h| (h.group, h.location.id, h.location.bone))
        .collect();
    assert_eq!(found, expected);
    assert!(!found.is_empty(), "a human holds things");
    // `find_holding_point` and the tree's own search agree.
    for &(_, id, _) in &found {
        let by_list = crate::find_holding_point(&tree.holdings(), id).unwrap();
        let by_tree = tree.get_holding_location(0, id).unwrap();
        assert_eq!(
            (by_list.group, by_list.bone),
            (by_tree.group, by_tree.location.bone)
        );
    }
}

#[test]
fn the_trees_bones_match_the_render_sides_rest_bones() {
    let Some(store) = store() else {
        return;
    };
    let visual = DataId::new(HUMAN_MALE_VISUAL);
    let tree = load_part_group_tree(&store, visual).unwrap().unwrap();
    let rest = rest_named_bones(&store, visual).unwrap();
    assert!(!rest.is_empty());
    // `CPartGroup::FindBoneType` takes the first bone of the name in search order, and so does
    // the tree: the group's matrix on the bone's matrix is the render side's transform.
    let mut checked = 0;
    for (name, transform) in rest.iter().take(80) {
        let first = rest.iter().position(|(n, _)| n == name).unwrap();
        if rest[first].1 != *transform {
            continue;
        }
        let Some(index) = tree
            .indices()
            .into_iter()
            .find(|&g| tree.group(g).unwrap().array.find_bone_type(*name).is_some())
        else {
            continue;
        };
        let group = tree.group(index).unwrap();
        let bone = group.array.find_matrix_for_bone_type(*name).unwrap();
        let ours = *group.matrix() * bone;
        let theirs = transform.to_mat4();
        let close = ours
            .to_cols_array()
            .iter()
            .zip(theirs.to_cols_array())
            .all(|(a, b)| (a - b).abs() < 2e-3);
        assert!(close, "bone {name:#x}: {ours:?} vs {theirs:?}");
        checked += 1;
    }
    assert!(checked > 5, "only {checked} bones compared");
}

#[test]
fn a_ray_through_the_humans_chest_hits_a_part_and_one_over_its_head_does_not() {
    let Some(store) = store() else {
        return;
    };
    let tree = load_part_group_tree(&store, DataId::new(HUMAN_MALE_VISUAL))
        .unwrap()
        .unwrap();
    // The selection sphere is centred on the body (the human's bounding sphere is empty).
    let sphere = tree.group(0).unwrap().array.selection_sphere().unwrap();
    let mid = Vector::new(sphere.center.x, sphere.center.y, sphere.center.z);
    let mut hit = None;
    // One of the four horizontal directions crosses the body at its middle height.
    for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        let from = Vector::new(mid.x - dx * 6.0, mid.y - dy * 6.0, mid.z);
        let ray = Ray {
            pt: from,
            dir: Vector::new(dx, dy, 0.0),
            length: 12.0,
        };
        hit = hit.or(tree.trace_ray(0, &ray));
    }
    let (delta, _kind) = hit.expect("a ray across the body hits one of its meshes");
    assert!(delta > 0.0 && delta < 1.0, "{delta}");
    let over = Ray {
        pt: Vector::new(mid.x - 6.0, mid.y, mid.z + 50.0),
        dir: Vector::new(1.0, 0.0, 0.0),
        length: 12.0,
    };
    assert!(tree.trace_ray(0, &over).is_none());
}
