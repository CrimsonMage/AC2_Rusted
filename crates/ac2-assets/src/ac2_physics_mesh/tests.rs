use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_db::{ResourcePackedDecode, ResourceStore, TypedResourceRef};

use super::{vertex_stride, Ac2PhysicsMesh};

#[test]
fn vertex_strides_match_the_render_decoder() {
    assert_eq!(vertex_stride(0x0000_02D2), 48);
    assert_eq!(vertex_stride(0x3000_02D2), 72);
    assert_eq!(vertex_stride(0x3004_02D2), 92);
    assert_eq!(vertex_stride(0x3003_02D2), 87);
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_f32s(bytes: &mut Vec<u8>, values: &[f32]) {
    values
        .iter()
        .for_each(|value| bytes.extend_from_slice(&value.to_le_bytes()));
}

/// A one-triangle mesh with no render polygons and a two-leaf physics tree.
fn triangle_record(did: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_u32(&mut bytes, did);
    push_u32(&mut bytes, 2); // mesh type
    push_u32(&mut bytes, 0); // materials
    push_u32(&mut bytes, 0x0000_0002); // position-only vertex format (12 bytes)
    push_u32(&mut bytes, 3);
    push_f32s(&mut bytes, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    push_f32s(&mut bytes, &[0.0; 10]); // bounding box and sphere
    push_u32(&mut bytes, 0); // degrades
    push_u32(&mut bytes, 0); // render polygons
    push_u32(&mut bytes, 1); // physics polygons
    bytes.extend_from_slice(&[7, 0, 3, 0, 0, 0, 1, 0, 2, 0, 0, 0]); // id 7, 3 points, aligned
    push_u32(&mut bytes, 1); // has physics BSP
    push_u32(&mut bytes, 0x4250_2D4E); // 'BP-N'
    push_f32s(&mut bytes, &[0.0, 0.0, 1.0, 0.0]);
    for (solid, slots) in [(1_u32, 1_u32), (0, 0)] {
        push_u32(&mut bytes, 0x4C45_4146);
        push_u32(&mut bytes, solid);
        push_f32s(&mut bytes, &[0.3, 0.3, 0.0, 1.0]);
        push_u32(&mut bytes, slots);
        if slots == 1 {
            bytes.extend_from_slice(&[0, 0, 0, 0]); // slot 0, aligned
        }
    }
    push_f32s(&mut bytes, &[0.3, 0.3, 0.0, 1.0]); // node sphere
    bytes
}

#[test]
fn physics_polygons_and_tree_follow_the_render_data() {
    let qdid = QualifiedDataId::new(DataId::new(0x0600_0001), 0);
    let mesh =
        Ac2PhysicsMesh::decode_packed(qdid, 0, &triangle_record(0x0600_0001)).expect("decodes");
    assert_eq!(mesh.positions().len(), 3);
    assert_eq!(mesh.polygons().len(), 1);
    assert_eq!(mesh.polygons()[0].polygon_id(), 7);
    assert_eq!(mesh.polygons()[0].vertex_ids(), &[0, 1, 2]);
    let root = mesh.bsp().expect("physics tree");
    assert!(!root.is_leaf());
    let pos = root.pos_node().expect("positive child");
    assert!(pos.is_leaf());
    assert_eq!(pos.solid(), 1);
    assert_eq!(pos.polygon_slots(), &[0]);
    assert!(root
        .neg_node()
        .expect("negative child")
        .polygon_slots()
        .is_empty());

    let mut trailing = triangle_record(0x0600_0001);
    trailing.extend_from_slice(&[0; 4]);
    assert!(Ac2PhysicsMesh::decode_packed(qdid, 0, &trailing).is_err());
}

/// Every Legions `CDataMesh` parses through its physics tree to the exact record length.
#[test]
fn every_legions_mesh_record_decodes_its_physics_data() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let ids = dataset
        .ids_in_range(DatFileKind::Portal, 0x0600_0000, 0x06FF_FFFF)
        .collect::<Vec<_>>();
    let mut with_tree = 0;
    for did in &ids {
        let mesh = store
            .load_cached_packed::<Ac2PhysicsMesh>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(*did),
                0,
            ))
            .unwrap_or_else(|error| panic!("{did:08X}: {error}"))
            .expect("listed record exists");
        with_tree += usize::from(mesh.bsp().is_some());
    }
    assert_eq!(ids.len(), 4501);
    assert_eq!(with_tree, 1377);
}

/// Every Legions `CSetup` parses through its collision primitives to its fixed tail.
#[test]
fn every_legions_setup_decodes_its_collision_primitives() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let ids = dataset
        .ids_in_range(DatFileKind::Portal, 0x0400_0000, 0x04FF_FFFF)
        .collect::<Vec<_>>();
    let mut with_cylinders = 0;
    for did in &ids {
        let setup = store
            .load_cached_packed::<super::Ac2SetupPhysics>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(*did),
                0,
            ))
            .unwrap_or_else(|error| panic!("{did:08X}: {error}"))
            .expect("listed record exists");
        with_cylinders += usize::from(!setup.cylspheres.is_empty());
    }
    let tree = store
        .load_cached_packed::<super::Ac2SetupPhysics>(TypedResourceRef::new(
            DatFileKind::Portal,
            DataId::new(0x0400_03DE),
            0,
        ))
        .expect("decodes")
        .expect("exists");
    assert_eq!(tree.cylspheres.len(), 1);
    assert!((tree.cylspheres[0].radius - 6.17).abs() < 0.01);
    assert!(with_cylinders > 0 && !ids.is_empty());
}

/// The human male body Setup: its two body spheres and the physics words
/// `CPartArray::GetHeight`/`GetRadius`/`GetStepUpHeight`/`GetStepDownHeight` read.
#[test]
fn the_human_body_setup_carries_its_spheres_and_step_heights() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let human = store
        .load_cached_packed::<super::Ac2SetupPhysics>(TypedResourceRef::new(
            DatFileKind::Portal,
            DataId::new(0x0400_0029),
            0,
        ))
        .expect("decodes")
        .expect("exists");
    assert!(human.cylspheres.is_empty());
    assert_eq!(human.spheres.len(), 2);
    assert!((human.spheres[0].origin.z - 0.55).abs() < 1.0e-5);
    assert!((human.spheres[1].origin.z - 1.5).abs() < 1.0e-5);
    assert!(human
        .spheres
        .iter()
        .all(|sphere| (sphere.radius - 0.555).abs() < 1.0e-5));
    assert!((human.height - 2.06).abs() < 1.0e-5);
    assert!((human.radius - 0.7849).abs() < 1.0e-3);
    assert!((human.step_up_height - 0.9).abs() < 1.0e-5);
    assert!((human.step_down_height - 1.1).abs() < 1.0e-5);
    assert!(human.allow_free_heading);
}
