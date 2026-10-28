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
