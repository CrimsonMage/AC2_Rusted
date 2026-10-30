use std::sync::Arc;

use super::{is_entity_desc, Remote, Remotes};

#[test]
fn only_entity_desc_dids_are_read_and_each_object_is_asked_once() {
    assert!(is_entity_desc(0x4700_0012));
    assert!(!is_entity_desc(0x1F00_0001));
    let mut remotes = Remotes::<()>::default();
    remotes.record_entity(7, 0x4700_0012);
    assert_eq!(remotes.take_entity_requests(), vec![(7, 0x4700_0012)]);
    assert!(remotes.take_entity_requests().is_empty());
}

#[test]
fn a_mask_that_arrives_is_kept_for_the_body_made_later() {
    let mut remotes = Remotes::<()>::default();
    remotes.entity_mask_loaded(7, 1 << 4);
    assert_eq!(remotes.entity_mask(7), Some(1 << 4));
}

/// A remote in the world with a body and a shape, as `try_create` and `sync_shadow` leave it.
fn in_world_remote(bits: Option<u64>) -> Remote {
    let shape = Arc::new(ac2_physics::ObjectShape::new(
        Vec::new(),
        vec![ac2_physics::CylSphere {
            low: ac2_physics::Vector::ZERO,
            height: 3.0,
            radius: 0.5,
        }],
        Vec::new(),
    ));
    let body_shape = ac2_physics::BodyShape {
        spheres: Vec::new(),
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let mut body = ac2_physics::PhysicsBody::new(Default::default(), body_shape, None);
    if let Some(bits) = bits {
        body.apply_blocking_bits(bits);
    }
    Remote {
        visual: None,
        body: Some(body),
        failed: false,
        stored: None,
        in_world: true,
        lost_cell: None,
        active: true,
        player_distance: 0.0,
        object: Some((shape, ac2_physics::Vector::new(1.0, 1.0, 1.0))),
        shadow: None,
        listed_cell: None,
        ethereal: false,
        alpha_moves: None,
        fx_follow: None,
    }
}

#[test]
fn a_remote_entry_follows_its_body_blocking_bits_when_they_change() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(None));
    let mut terrain = ac2_physics::Terrain::new();
    remotes.sync_shadows(&mut terrain);
    let id = remotes.get(7).and_then(|r| r.shadow).expect("listed");
    assert_eq!(terrain.object_blocking(id), Some(0));

    // The EntityDesc's bits arrive after the body is made: the next sync carries them.
    remotes.entity_mask_loaded(7, 1 << 4);
    remotes.sync_shadows(&mut terrain);
    assert_eq!(terrain.object_blocking(id), Some(1 << 4));
}

#[test]
fn a_body_made_with_bits_lists_them_from_its_first_entry() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(Some(1 << 9)));
    let mut terrain = ac2_physics::Terrain::new();
    remotes.sync_shadows(&mut terrain);
    let id = remotes.get(7).and_then(|r| r.shadow).expect("listed");
    assert_eq!(terrain.object_blocking(id), Some(1 << 9));
}

#[test]
fn an_object_in_the_world_is_listed_in_its_cell_and_leaves_with_it() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(None));
    let mut terrain = ac2_physics::Terrain::new();
    remotes.sync_shadows(&mut terrain);
    let cell = remotes.get(7).and_then(|r| r.listed_cell).expect("listed");
    assert_eq!(remotes.membership().inq_objects(cell, false), vec![7]);
    assert_eq!(remotes.membership().dynamic_objects(cell).len(), 1);
    // Leaving the world takes it out of the list.
    remotes.removed(7, &mut terrain);
    assert!(remotes.membership().inq_objects(cell, false).is_empty());
    assert!(remotes.membership().dynamic_objects(cell).is_empty());
}

#[test]
fn a_stuck_body_leaves_the_world_without_being_queued() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(None));
    remotes.leave_world_for_parent(7);
    assert!(!remotes.get(7).unwrap().in_world);
    // Out of the world already: it leaves its lost cell instead; nothing panics.
    remotes.leave_world_for_parent(7);
    remotes.leave_world_for_parent(99);
}

/// `CPhysicsObj::SetScale` on a listed body (a script's `CWRepository::SetScale`): the next sync
/// lists the object again at the Setup scale times the body's own.
#[test]
fn a_rescaled_body_is_listed_again_at_its_new_scale() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(None));
    let mut terrain = ac2_physics::Terrain::new();
    remotes.sync_shadows(&mut terrain);
    let id = remotes.get(7).and_then(|r| r.shadow).expect("listed");
    let one = ac2_physics::Vector::new(1.0, 1.0, 1.0);
    assert_eq!(terrain.object_scale(id), Some(one));
    remotes.sync_shadows(&mut terrain);
    assert_eq!(remotes.get(7).and_then(|r| r.shadow), Some(id), "unchanged");
    remotes.body_mut(7).unwrap().set_uniform_scale(2.0);
    remotes.sync_shadows(&mut terrain);
    let again = remotes.get(7).and_then(|r| r.shadow).expect("listed again");
    let two = ac2_physics::Vector::new(2.0, 2.0, 2.0);
    assert_eq!(terrain.object_scale(again), Some(two));
}

#[test]
fn the_player_distance_sets_physics_range_each_time_it_is_checked() {
    // `CPhysicsObj::UpdateMovement`: within 96 the object is in range, beyond it out.
    let at = |x: f32| {
        let mut frame = ac2_physics::Frame::IDENTITY;
        frame.origin = ac2_physics::Vector::new(x, 0.0, 0.0);
        ac2_physics::Position::new(0xA9B4_0001, frame)
    };
    let mut remote = in_world_remote(None);
    remote.update_physics_range(&at(0.0), &at(95.0));
    assert!(remote.active);
    assert_eq!(remote.player_distance, 95.0);
    remote.update_physics_range(&at(0.0), &at(97.0));
    assert!(!remote.active);
    remote.update_physics_range(&at(50.0), &at(97.0));
    assert!(remote.active);
}

#[test]
fn a_temporary_object_out_of_physics_range_is_destroyed_and_others_stay() {
    // `CPhysicsObj::HandleOutOfPhysicsRange`: `VerifyTemporary` ids are destroyed.
    let mut remotes = Remotes::<()>::default();
    let temporary = crate::temporary_ids::TEMPORARY_BASE + 1;
    remotes.insert(temporary, in_world_remote(None));
    remotes.insert(7, in_world_remote(None));
    let mut terrain = ac2_physics::Terrain::new();
    let mut random = ac2_core::Random::default();
    let body = remotes.get(7).and_then(|r| r.body()).unwrap();
    let mut view = body.object_view();
    view.position.frame.origin.x += 500.0;
    remotes.step_quantum(&mut terrain, 0.0, &mut random, &[(1, view)]);
    assert!(remotes.get(temporary).is_none());
    assert!(remotes.get(7).is_some_and(|r| !r.active));

/// `CPhysicsObj::InqCellObjects` / `InqBlockObjects`: the objects of the object's own cell, and of
/// its whole landblock.
#[test]
fn an_object_lists_the_objects_of_its_cell_and_block() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(None));
    remotes.insert(8, in_world_remote(None));
    let mut terrain = ac2_physics::Terrain::new();
    remotes.sync_shadows(&mut terrain);
    assert_eq!(remotes.inq_cell_objects(7, false), vec![7, 8]);
    assert_eq!(remotes.inq_block_objects(8, true), vec![7, 8]);
    // An object in no cell lists nothing.
    remotes.insert(9, in_world_remote(None));
    assert!(remotes.inq_cell_objects(9, false).is_empty());
    assert!(remotes.inq_block_objects(99, false).is_empty());
}

/// `CPhysicsObj::SyncPositionSimple` ends in `SetPlacementPosition`, placed or not.
#[test]
fn a_synced_position_becomes_the_previous_position() {
    let mut remotes = Remotes::<()>::default();
    remotes.insert(7, in_world_remote(None));
    let mut terrain = ac2_physics::Terrain::new();
    let mut position = ac2_physics::Position {
        cell_id: 0x7F7F_0001,
        ..Default::default()
    };
    position.frame.origin = ac2_physics::Vector::new(5.0, 6.0, 7.0);
    let records = |_: u64| None;
    remotes.sync_position_simple(7, position, &records, &mut terrain);
    let body = remotes.get(7).unwrap().body.as_ref().unwrap();
    assert_eq!(body.previous_position(), position);
}
