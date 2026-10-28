//! `CPartArray` / `CPartGroup` on hand-made Setups: bones, connection points, searches, tree edits
//! and the ray test.

use std::sync::Arc;

use ac2_anim::{Ac2Bone, Ac2DegradeInfo, Ac2Location, Ac2PlacementFrame, Ac2Setup, Ac2Transform};
use ac2_assets::Ac2SetupPhysics;
use ac2_core::DataId;
use ac2_math::{Mat4, Quat, Vec3};

use crate::los::Ray;
use crate::part_array::{ray_intersects_tri_list, PartArray, PartMesh, PartSetup};
use crate::part_group::PartGroupTree;
use crate::vector::{Sphere, Vector};

fn at(offset: Vec3) -> Ac2Transform {
    Ac2Transform {
        offset,
        ..Ac2Transform::IDENTITY
    }
}

fn physics() -> Ac2SetupPhysics {
    Ac2SetupPhysics {
        cylspheres: Arc::from([ac2_assets::Ac2CylSphere {
            low_point: Vec3::new(0.0, 0.0, 0.1),
            radius: 0.4,
            height: 1.8,
        }]),
        spheres: Arc::from([ac2_math::Sphere {
            origin: Vec3::new(0.0, 0.0, 0.5),
            radius: 0.5,
        }]),
        height: 1.9,
        radius: 0.45,
        step_up_height: 0.4,
        step_down_height: 0.6,
        allow_free_heading: false,
        bounding_sphere: ac2_math::Sphere {
            origin: Vec3::new(0.0, 0.0, 1.0),
            radius: 1.5,
        },
        selection_sphere: ac2_math::Sphere {
            origin: Vec3::new(0.0, 0.0, 0.9),
            radius: 1.0,
        },
        bounds_min: Vec3::new(-1.0, -1.0, 0.0),
        bounds_max: Vec3::new(1.0, 1.0, 2.0),
    }
}

/// Two bones: bone 0 at the origin, bone 1 two metres up it; a holding location on bone 1 and a
/// connection point on bone 1.
fn setup(did: u32, parts: Vec<Option<PartMesh>>) -> Arc<PartSetup> {
    let frame = Arc::from([at(Vec3::ZERO), at(Vec3::new(0.0, 0.0, 2.0))]);
    let setup = Ac2Setup {
        meshes: parts.iter().map(|_| DataId::new(0x0600_0001)).collect(),
        bones: Arc::from([
            Ac2Bone {
                parent: None,
                name: 11,
            },
            Ac2Bone {
                parent: Some(0),
                name: 22,
            },
        ]),
        placements: Arc::from([
            Ac2PlacementFrame {
                kind: 0,
                bones: Arc::clone(&frame),
            },
            Ac2PlacementFrame {
                kind: 5,
                bones: Arc::from([at(Vec3::ZERO), at(Vec3::new(0.0, 0.0, 3.0))]),
            },
        ]),
        holding_points: Arc::from([Ac2Location {
            id: 7,
            bone: 1,
            transform: at(Vec3::new(0.5, 0.0, 0.0)),
        }]),
        connection_points: Arc::from([Ac2Location {
            id: 9,
            bone: 1,
            transform: at(Vec3::new(0.0, 1.0, 0.0)),
        }]),
        degrade: Ac2DegradeInfo::default(),
        bounds_min: Vec3::ZERO,
        bounds_max: Vec3::ONE,
    };
    Arc::new(PartSetup {
        did: DataId::new(did),
        setup: Arc::new(setup),
        physics: Arc::new(physics()),
        parts,
    })
}

fn mesh(has_bsp: bool, triangles: Vec<[Vec3; 3]>) -> Option<PartMesh> {
    Some(PartMesh {
        mesh_type: 0x42,
        has_physics_bsp: has_bsp,
        triangles: Arc::from(triangles),
        ..Default::default()
    })
}

/// A triangle facing -y in the plane y = 0, x and z within one metre of the origin.
fn wall() -> Vec<[Vec3; 3]> {
    vec![[
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ]]
}

fn ray(from: Vector, towards: Vector, length: f32) -> Ray {
    Ray {
        pt: from,
        dir: towards,
        length,
    }
}

#[test]
fn the_array_reads_its_setup() {
    let mut array = PartArray::new();
    assert!(array.setup().is_none() && array.num_cylsphere() == 0);
    assert!(array.selection_sphere().is_none());
    assert_eq!(array.bounding_sphere(), Sphere::new(Vector::ZERO, 0.0));
    assert!(array.set_setup(setup(0x0400_0001, vec![None, None])));
    assert_eq!(array.num_bones(), 2);
    assert_eq!(array.num_cylsphere(), 1);
    assert_eq!(array.cylspheres()[0].radius, 0.4);
    assert_eq!(array.num_sphere(), 1);
    assert_eq!((array.radius(), array.height()), (0.45, 1.9));
    assert_eq!(array.selection_sphere().unwrap().radius, 1.0);
    assert_eq!(array.bounding_sphere().radius, 1.5);
    let (min, max) = array.physics_bounding_box().unwrap();
    assert_eq!((min.x, max.z), (-1.0, 2.0));
    // Bone 1 rides bone 0 two metres up.
    assert_eq!(
        array.bone_matrix(1).unwrap().w_axis.truncate(),
        Vec3::new(0.0, 0.0, 2.0)
    );
}

#[test]
fn a_placement_frame_moves_the_bones_and_an_unknown_one_falls_back_to_zero() {
    let mut array = PartArray::new();
    array.set_setup(setup(0x0400_0001, vec![]));
    assert!(array.set_placement_frame_id(5));
    array.update_bones();
    assert_eq!(array.placement_id(), 5);
    assert_eq!(array.bone_matrix(1).unwrap().w_axis.z, 3.0);
    // `SetPlacementFrameID` of a frame the Setup lacks leaves id 0 and frame 0's bones.
    assert!(array.set_placement_frame_id(99));
    array.update_bones();
    assert_eq!(array.placement_id(), 0);
    assert_eq!(array.bone_matrix(1).unwrap().w_axis.z, 2.0);
}

#[test]
fn setting_the_same_setup_keeps_the_placement() {
    let mut array = PartArray::new();
    let s = setup(0x0400_0001, vec![]);
    array.set_setup(s.clone());
    array.set_placement_frame_id(5);
    assert!(array.set_setup(s));
    assert_eq!(
        array.placement_id(),
        5,
        "CPartArray::SetSetupID returns early"
    );
    array.set_setup(setup(0x0400_0002, vec![]));
    assert_eq!(array.placement_id(), 0, "a different Setup resets it");
}

#[test]
fn a_bounding_object_replaces_the_spheres() {
    let mut array = PartArray::new();
    assert!(!array.set_bounding_object(&[Sphere::new(Vector::ZERO, 1.0)]));
    array.set_setup(setup(0x0400_0001, vec![]));
    let own = Sphere::new(Vector::new(0.0, 0.0, 3.0), 2.0);
    assert!(array.set_bounding_object(&[own, own]));
    assert_eq!(array.num_sphere(), 2);
    assert_eq!(array.spheres()[0], own);
}

#[test]
fn holding_locations_and_bone_types() {
    let mut array = PartArray::new();
    array.set_setup(setup(0x0400_0001, vec![]));
    assert!(array.has_holding_location(7) && !array.has_holding_location(8));
    let (location, matrix) = array.get_holding_location(7).unwrap();
    assert_eq!(location.bone, 1);
    assert_eq!(matrix.w_axis.z, 2.0);
    assert_eq!(array.find_bone_type(22), Some(1));
    assert_eq!(array.find_matrix_for_bone_type(11).unwrap(), Mat4::IDENTITY);
    assert!(array.find_matrix_for_bone_type(33).is_none());
}

#[test]
fn the_nearest_bone_and_the_offset_from_the_empty_array() {
    let mut array = PartArray::new();
    assert!(array
        .calculate_bone_index_and_offset(&frame(Vec3::ZERO))
        .is_none());
    array.set_setup(setup(0x0400_0001, vec![]));
    let (index, offset) = array
        .calculate_bone_index_and_offset(&frame(Vec3::new(0.0, 0.0, 1.8)))
        .unwrap();
    assert_eq!(index, 1);
    // `CPartArray::CalculateBoneIndexAndOffset` ends by zeroing the offset's origin.
    assert_eq!(offset.origin, Vec3::ZERO);
    let (index, _) = array
        .calculate_bone_index_and_offset(&frame(Vec3::new(0.0, 0.0, 0.2)))
        .unwrap();
    assert_eq!(index, 0);
}

fn frame(origin: Vec3) -> ac2_math::Frame {
    ac2_math::Frame::new(origin, Quat::IDENTITY)
}

#[test]
fn skinned_parts_stand_at_the_group_matrix_alone() {
    let skinned = PartMesh {
        has_matrix_weights: true,
        ..mesh(false, vec![]).unwrap()
    };
    let mut array = PartArray::new();
    array.set_setup(setup(0x0400_0001, vec![None, Some(skinned)]));
    assert_eq!(
        array.state() & crate::part_array::HAS_MATRIX_WEIGHTS_STATE,
        0x4000_0000
    );
    let parent = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
    array.refresh_matrices(&parent);
    assert_eq!(
        array.part_matrix(1).unwrap().w_axis.truncate(),
        Vec3::new(5.0, 0.0, 0.0)
    );
    // Without weights the part also rides its bone (z = 2).
    array.set_setup(setup(0x0400_0002, vec![None, mesh(false, vec![])]));
    array.refresh_matrices(&parent);
    assert_eq!(
        array.part_matrix(1).unwrap().w_axis.truncate(),
        Vec3::new(5.0, 0.0, 2.0)
    );
}

#[test]
fn the_physics_bsp_bit_follows_the_parts() {
    let mut array = PartArray::new();
    array.set_setup(setup(
        0x0400_0001,
        vec![mesh(false, vec![]), mesh(true, vec![])],
    ));
    assert!(array.cache_has_physics_bsp());
    assert_eq!(
        array.state() & crate::part_array::HAS_PHYSICS_BSP_STATE,
        0x1000
    );
    array.set_setup(setup(0x0400_0002, vec![mesh(false, vec![])]));
    assert!(!array.cache_has_physics_bsp());
    assert_eq!(array.state() & 0x1000, 0);
}

/// A root with a child hanging from its connection point 9 (bone 1, one metre along y).
fn tree() -> (PartGroupTree, usize) {
    let mut tree = PartGroupTree::new();
    assert!(tree.init_setup(0, setup(0x0400_0001, vec![])));
    let child = tree.new_group();
    assert!(!tree.add_child(0, child, 9), "a child needs a Setup first");
    assert!(tree.init_setup(child, setup(0x0400_0002, vec![])));
    tree.group_mut(child).unwrap().key = 77;
    assert!(tree.add_child(0, child, 9));
    (tree, child)
}

#[test]
fn a_child_hangs_from_its_parents_connection_point() {
    let (mut tree, child) = tree();
    assert!(!tree.add_child(0, child, 9), "already has a parent");
    assert_eq!(tree.group(child).unwrap().parent(), Some(0));
    assert_eq!(tree.group(child).unwrap().connection_point(), 9);
    tree.check_refresh_matrices();
    // bone 1 (z = 2) then the point's offset (y = 1).
    assert_eq!(
        tree.group(child).unwrap().matrix().w_axis.truncate(),
        Vec3::new(0.0, 1.0, 2.0)
    );
    // A different placement frame moves the parent's bone and with it the child.
    assert!(tree.set_placement_frame_id(0, 5));
    tree.check_refresh_matrices();
    assert_eq!(tree.group(child).unwrap().matrix().w_axis.z, 3.0);
}

#[test]
fn a_missing_connection_point_refuses_the_child() {
    let mut tree = PartGroupTree::new();
    tree.init_setup(0, setup(0x0400_0001, vec![]));
    let child = tree.new_group();
    tree.init_setup(child, setup(0x0400_0002, vec![]));
    assert!(!tree.add_child(0, child, 1234));
    assert!(tree.group(child).unwrap().parent().is_none());
}

#[test]
fn groups_are_found_by_key_and_removed_with_their_children() {
    let (mut tree, child) = tree();
    tree.group_mut(0).unwrap().key = 1;
    assert_eq!(tree.get_part_group(0, 1), Some(0));
    assert_eq!(tree.get_part_group(0, 77), Some(child));
    assert_eq!(tree.get_part_group(0, 5), None);
    assert!(!tree.remove_child(child, 0), "not a child of it");
    assert!(tree.remove_child(0, child));
    assert!(tree.group(child).unwrap().parent().is_none());
    assert_eq!(tree.get_part_group(0, 77), None);
    assert!(
        tree.add_child(0, child, 9),
        "a removed child can hang again"
    );
    tree.destroy_children(0);
    assert!(tree.group(child).is_none());
    assert_eq!(tree.indices(), vec![0]);
}

#[test]
fn holding_locations_search_the_group_then_its_children() {
    let (tree, child) = tree();
    let holdings = tree.holdings();
    assert_eq!(holdings.len(), 2);
    assert_eq!((holdings[0].group, holdings[1].group), (0, child));
    // The first group's entry wins, as `CPartGroup::GetHoldingLocation` returns it.
    let found = tree.get_holding_location(0, 7).unwrap();
    assert_eq!(found.group, 0);
    assert_eq!(found.bone_matrix.w_axis.z, 2.0);
    assert!(tree.has_holding_location(0, 7) && !tree.has_holding_location(0, 8));
    let point = crate::find_holding_point(&holdings, 7).unwrap();
    assert_eq!((point.group, point.bone), (0, 1));
    assert!(tree.has_bone_type(0, 22) && !tree.has_bone_type(0, 33));
}

#[test]
fn a_ray_hits_the_nearest_part_and_reports_its_mesh_type() {
    let mut tree = PartGroupTree::new();
    tree.init_setup(0, setup(0x0400_0001, vec![mesh(true, wall()), None]));
    tree.check_refresh_matrices();
    let toward = Vector::new(0.0, 1.0, 0.0);
    let (delta, kind) = tree
        .trace_ray(0, &ray(Vector::new(0.0, -4.0, 0.0), toward, 8.0))
        .expect("the wall is on the ray");
    assert_eq!(kind, 0x42);
    assert!((delta - 0.5).abs() < 1e-5, "{delta}");
    // Past the triangle's edge, and pointing away from it.
    assert!(tree
        .trace_ray(0, &ray(Vector::new(5.0, -4.0, 0.0), toward, 8.0))
        .is_none());
    assert!(tree
        .trace_ray(
            0,
            &ray(
                Vector::new(0.0, -4.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
                8.0
            )
        )
        .is_none());
}

#[test]
fn a_ray_ending_before_the_plane_misses() {
    let tris = wall();
    let toward = Vector::new(0.0, 1.0, 0.0);
    // Both ends strictly in front of the plane: skipped.
    let short = ray(Vector::new(0.0, -4.0, 0.0), toward, 2.0);
    assert!(ray_intersects_tri_list(&short, &tris, true).is_none());
    // Crossing the plane: hit at 4 of 8.
    let long = ray(Vector::new(0.0, -4.0, 0.0), toward, 8.0);
    assert!((ray_intersects_tri_list(&long, &tris, true).unwrap() - 0.5).abs() < 1e-6);
    // Starting behind the plane, only the back-facing flag lets it through.
    let behind = ray(Vector::new(0.0, 4.0, 0.0), Vector::new(0.0, -1.0, 0.0), 8.0);
    assert!(ray_intersects_tri_list(&behind, &tris, true).is_some());
    assert!(ray_intersects_tri_list(&behind, &tris, false).is_none());
    // Parallel to the plane: no determinant.
    let parallel = ray(Vector::new(-3.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0), 8.0);
    assert!(ray_intersects_tri_list(&parallel, &tris, true).is_none());
}

#[test]
fn the_part_matrix_places_the_mesh_for_the_ray() {
    let mut tree = PartGroupTree::new();
    // The wall rides bone 1, two metres up.
    tree.init_setup(0, setup(0x0400_0001, vec![None, mesh(true, wall())]));
    tree.check_refresh_matrices();
    let toward = Vector::new(0.0, 1.0, 0.0);
    assert!(tree
        .trace_ray(0, &ray(Vector::new(0.0, -4.0, 2.0), toward, 8.0))
        .is_some());
    assert!(tree
        .trace_ray(0, &ray(Vector::new(0.0, -4.0, 0.0), toward, 8.0))
        .is_none());
}

#[test]
fn compare_and_modify_follows_a_new_description() {
    use ac2_assets::{Ac2TotalGroup, Ac2TotalVisualDesc};

    let group = |key: u32, setup: u32, cp: u32| Ac2TotalGroup {
        key,
        connection_point: cp,
        setup: Some(DataId::new(setup)),
        ..Default::default()
    };
    let mut desc = Ac2TotalVisualDesc {
        root: Some(1),
        groups: vec![group(1, 0x0400_0001, 0), group(2, 0x0400_0002, 9)],
        children: vec![(1, vec![2])],
        ..Default::default()
    };
    let mut load = |did: DataId| Some(setup(did.raw(), vec![]));
    let mut tree = PartGroupTree::new();
    assert!(tree.init_visual_desc(0, &desc, &mut load));
    assert_eq!(tree.indices().len(), 2);
    assert_eq!(
        tree.get_part_group(0, 2)
            .map(|g| tree.group(g).unwrap().key),
        Some(2)
    );
    // Nothing changed: the same groups stay.
    assert!(tree.compare_and_modify(0, &desc, &mut load));
    assert_eq!(tree.indices().len(), 2);
    // The child group's Setup changes: it is reloaded in place.
    desc.groups[1].setup = Some(DataId::new(0x0400_0003));
    assert!(tree.compare_and_modify(0, &desc, &mut load));
    let child = tree.get_part_group(0, 2).unwrap();
    assert_eq!(
        tree.group(child).unwrap().array.setup().unwrap().did.raw(),
        0x0400_0003
    );
    // A third group joins.
    desc.groups.push(group(3, 0x0400_0004, 9));
    desc.children = vec![(1, vec![2, 3])];
    assert!(tree.compare_and_modify(0, &desc, &mut load));
    assert_eq!(tree.indices().len(), 3);
    // The second goes: `HandleResidualChildren` removes it.
    desc.groups.remove(1);
    desc.children = vec![(1, vec![3])];
    assert!(tree.compare_and_modify(0, &desc, &mut load));
    assert!(tree.get_part_group(0, 2).is_none());
    assert!(tree.get_part_group(0, 3).is_some());
}

fn body() -> crate::PhysicsBody {
    use crate::{Frame, Position, Quaternion};
    let position = Position::new(
        crate::landdefs::lcoord_to_gid(0x7F * 8 + 4, 0x7F * 8 + 4),
        Frame::new(Vector::new(10.0, 10.0, 0.0), Quaternion::IDENTITY),
    );
    crate::PhysicsBody::new(position, crate::test_support::shape(), None)
}

fn body_with_parts(parts: Vec<Option<PartMesh>>) -> crate::PhysicsBody {
    let mut body = body();
    let mut tree = PartGroupTree::new();
    tree.init_setup(0, setup(0x0400_0001, parts));
    tree.check_refresh_matrices();
    body.set_part_group(Arc::new(tree));
    body
}

#[test]
fn a_body_without_a_representation_answers_empty() {
    let mut body = body();
    assert_eq!(body.num_cylsphere(), 0);
    assert!(body.cylspheres().is_empty());
    assert_eq!(body.sorting_sphere(), Sphere::new(Vector::ZERO, 0.0));
    assert!(body.selection_sphere().is_none() && body.physics_bounding_box().is_none());
    assert!(!body.set_bounding_object(&[]) && !body.has_bone_type(11));
    assert!(!body.cache_has_physics_bsp() && body.part_group_with_key(0).is_none());
}

#[test]
fn the_cylsphere_and_bounds_getters_read_the_setup() {
    let body = body_with_parts(vec![]);
    assert_eq!(body.num_cylsphere(), 1);
    assert_eq!(body.cylspheres()[0].height, 1.8);
    assert_eq!(body.sorting_sphere().radius, 1.5);
    assert_eq!(body.physics_bounding_box().unwrap().1.z, 2.0);
    assert!(body.has_bone_type(22) && !body.has_bone_type(23));
    assert_eq!(body.bone_type_matrix(22).unwrap().w_axis.z, 2.0);
    assert_eq!(body.part_group_with_key(0), Some(0));
    assert!(body.has_holding_location_in_parts(7));
    let point = body.holding_point_in_parts(7).unwrap();
    assert_eq!((point.group, point.bone), (0, 1));
    assert_eq!(body.group_holdings().len(), 1);
}

#[test]
fn the_selection_sphere_takes_both_scales() {
    // The shape's scale (the representation's) is 1 in the test shape; the object's own scale
    // multiplies the centre by axis and the radius by z, as `CPhysicsObj::GetSelectionSphere`.
    let mut body = body_with_parts(vec![]);
    let unscaled = body.selection_sphere().unwrap();
    assert_eq!((unscaled.center.z, unscaled.radius), (0.9, 1.0));
    body.set_scale(Vector::new(2.0, 3.0, 4.0));
    let scaled = body.selection_sphere().unwrap();
    assert_eq!((scaled.center.z, scaled.radius), (3.6, 4.0));
    assert_eq!(
        body.sorting_sphere().radius,
        1.5,
        "the sorting sphere is unscaled"
    );
}

#[test]
fn a_bounding_object_changes_the_collision_spheres() {
    let mut body = body_with_parts(vec![]);
    let own = Sphere::new(Vector::new(0.0, 0.0, 1.0), 0.75);
    assert!(body.set_bounding_object(&[own]));
    assert_eq!(body.spheres(), vec![own]);
}

#[test]
fn caching_the_physics_bsp_sets_the_state_bit() {
    let mut body = body_with_parts(vec![mesh(true, vec![])]);
    assert!(!body.has_physics_bsp());
    assert!(body.cache_has_physics_bsp());
    assert!(body.has_physics_bsp());
    let mut plain = body_with_parts(vec![mesh(false, vec![])]);
    assert!(!plain.cache_has_physics_bsp() && !plain.has_physics_bsp());
}

#[test]
fn a_body_traces_rays_through_its_parts() {
    let body = body_with_parts(vec![mesh(true, wall())]);
    let hit = body.trace_ray_parts(&ray(
        Vector::new(0.0, -4.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        8.0,
    ));
    assert_eq!(hit.map(|h| h.1), Some(0x42));
}

#[test]
fn a_missile_sticks_to_the_nearest_bone_and_follows_it() {
    use crate::{ChildGraph, ChildType, Frame, Position, Quaternion};

    let holder = body_with_parts(vec![]);
    let cell = holder.position().cell_id;
    let hit = Position::new(
        cell,
        Frame::new(Vector::new(10.0, 10.0, 1.9), Quaternion::IDENTITY),
    );
    // Bone 1 stands two metres up and the hit is 0.1 m below it.
    let (bone, offset) = holder
        .sticky_attachment(&hit)
        .expect("the holder has bones");
    assert_eq!(bone, 1);
    assert_eq!(offset.origin, Vec3::ZERO);
    let low = Position::new(
        cell,
        Frame::new(Vector::new(10.0, 10.0, 0.3), Quaternion::IDENTITY),
    );
    assert_eq!(holder.sticky_attachment(&low).unwrap().0, 0);
    // A holder without a part tree has nothing to stick to.
    assert!(body().sticky_attachment(&hit).is_none());

    let mut graph = ChildGraph::default();
    assert!(
        !graph.sticky_set_parent(5, 5, bone, offset),
        "not to itself"
    );
    assert!(graph.sticky_set_parent(9, 5, bone, offset));
    assert_eq!(graph.parent_of(9), Some(5));
    let info = graph.info_of(9).unwrap();
    assert_eq!((info.kind, info.location), (ChildType::BoneIndex, 1));
    // The holder places it on the bone (`UpdateChild`), so it follows the bone's matrix.
    let at = crate::parent_drawing_position(
        &holder.position(),
        holder.object_view().scale,
        info,
        holder.bone_transform(info.location).unwrap(),
        None,
    )
    .expect("a bone-index child needs no holding location");
    assert_eq!(at.frame.origin.z, 2.0);
    assert_eq!(at.frame.origin.x, 10.0);
}
