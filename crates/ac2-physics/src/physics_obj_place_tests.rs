//! Placement on synthetic terrain (no game data): scatter, `SetPlacementPosition`,
//! `StorePosition`, the height-above-walkable probe and the shadow-list queries.

use ac2_core::Random;

use crate::frame::Frame;
use crate::objects_tests::{bsp_box, flat, position, walker, world_with};
use crate::physics_obj_place::{
    SetPositionError, SetPositionStruct, PLACEMENT_SPF, RANDOMSCATTER_SPF, SCATTER_SPF,
};
use crate::position::Position;
use crate::vector::{Quaternion, Vector};

/// A 3 m tall, 1 m radius cylinder standing at (x, y, z) in the physics representation.
fn pillar(x: f32, y: f32, z: f32, id: Option<u64>) -> crate::objects::StaticObject {
    let shape = std::sync::Arc::new(crate::objects::ObjectShape::new(
        Vec::new(),
        vec![crate::shapes::cylsphere::CylSphere {
            low: Vector::ZERO,
            height: 3.0,
            radius: 1.0,
        }],
        Vec::new(),
    ));
    let object =
        crate::objects::StaticObject::new(position(x, y, z), Vector::new(1.0, 1.0, 1.0), shape);
    match id {
        Some(id) => object.with_instance_id(id),
        None => object,
    }
}

/// A box 2 m wide and 2 m tall at (80, 84): the walker's spheres cut its north face at y = 85.2.
fn blocked_world() -> crate::terrain::Terrain {
    world_with(bsp_box(
        position(80.0, 84.0, 0.0),
        Vector::new(-1.0, -1.0, 0.0),
        Vector::new(1.0, 1.0, 2.0),
    ))
}

#[test]
fn a_scatter_placement_tries_nearby_spots_when_the_exact_one_is_blocked() {
    let world = blocked_world();
    let mut body = walker(70.0, 70.0, 0.0);
    let mut random = Random::default();
    let mut sps = SetPositionStruct::new(position(80.0, 85.2, 0.0), 0);
    assert_ne!(
        body.set_position_struct(&sps, &world, &mut random),
        SetPositionError::Ok
    );
    sps.flags = SCATTER_SPF;
    sps.set_scatter_radius(6.0, -1.0);
    sps.set_num_tries(30);
    assert_eq!(sps.y_radius, 6.0);
    assert_eq!(
        body.set_position_struct(&sps, &world, &mut random),
        SetPositionError::Ok
    );
    let p = body.position().frame.origin;
    assert!(
        (p.x - 80.0).abs() <= 6.0 && (p.y - 85.2).abs() <= 6.0,
        "{p:?}"
    );
    assert!((p.x, p.y) != (80.0, 85.2), "{p:?}");
}

#[test]
fn a_random_scatter_never_places_at_the_exact_position() {
    let world = flat();
    let mut body = walker(70.0, 70.0, 0.0);
    let mut random = Random::default();
    let mut sps = SetPositionStruct::new(position(80.0, 84.0, 0.0), RANDOMSCATTER_SPF);
    sps.set_scatter_radius(4.0, 2.0);
    sps.set_num_tries(1);
    assert_eq!(
        body.set_position_struct(&sps, &world, &mut random),
        SetPositionError::Ok
    );
    let p = body.position().frame.origin;
    assert!(
        (p.x - 80.0).abs() <= 4.0 && (p.y - 84.0).abs() <= 2.0,
        "{p:?}"
    );
    assert!(p.x != 80.0 && p.y != 84.0, "{p:?}");
    assert_eq!(p.z, 0.0);
}

#[test]
fn a_scatter_with_no_tries_is_a_general_failure() {
    let world = flat();
    let mut body = walker(70.0, 70.0, 0.0);
    let before = body.position();
    let sps = SetPositionStruct::new(position(80.0, 84.0, 0.0), RANDOMSCATTER_SPF);
    assert_eq!(
        body.set_position_struct(&sps, &world, &mut Random::default()),
        SetPositionError::GeneralFailure
    );
    assert_eq!(body.position(), before);
}

#[test]
fn only_the_placement_flag_resets_the_previous_position() {
    let world = flat();
    let mut body = walker(70.0, 70.0, 0.0);
    let start = body.previous_position();
    assert_eq!(
        body.place(position(90.0, 90.0, 0.0), crate::SLIDE_SPF, &world),
        SetPositionError::Ok
    );
    assert_eq!(
        body.previous_position(),
        start,
        "a blip keeps the old position"
    );
    let target = position(100.0, 95.0, 0.0);
    assert_eq!(
        body.place(target, PLACEMENT_SPF, &world),
        SetPositionError::Ok
    );
    assert_eq!(body.previous_position(), target);
    assert_eq!(body.drawing_position(0.5).frame.origin, target.frame.origin);
}

#[test]
fn a_placement_turns_the_body_to_the_placed_heading() {
    let world = flat();
    let mut body = walker(70.0, 70.0, 0.0);
    let mut frame = Frame::new(Vector::new(90.0, 90.0, 0.0), Quaternion::IDENTITY);
    frame.set_heading(crate::heading::Heading::from_degrees(90.0));
    let target = Position::new(position(90.0, 90.0, 0.0).cell_id, frame);
    body.place(target, PLACEMENT_SPF, &world);
    assert!(body.is_turning_to());
}

#[test]
fn a_stored_position_moves_into_the_land_cell_under_it() {
    let mut body = walker(70.0, 70.0, 0.0);
    // 25 m east of the cell's edge: the next land cell over.
    let stored = Position::new(
        position(10.0, 10.0, 0.0).cell_id,
        Frame::new(Vector::new(45.0, 10.0, 3.0), Quaternion::IDENTITY),
    );
    body.store_position(stored);
    let p = body.position();
    assert_eq!(p.cell_id, position(45.0, 10.0, 0.0).cell_id);
    assert_eq!(p.frame.origin, Vector::new(45.0, 10.0, 3.0));
}

#[test]
fn the_height_above_walkable_is_the_drop_to_the_surface_under_the_body() {
    let mut world = flat();
    let mut body = walker(80.0, 80.0, 5.0);
    assert_eq!(body.get_height_above_walkable(&world), (true, 5.0));
    // A box 3 m tall under it is the nearer surface.
    world.insert_object(pillar(80.0, 80.0, 0.0, None));
    assert_eq!(body.get_height_above_walkable(&world), (true, 2.0));
    // Below the terrain, the climb to it is negative.
    let sunk = walker(80.0, 80.0, -4.0);
    assert_eq!(sunk.get_height_above_walkable(&flat()), (true, -4.0));
    // The body's own entry in the world's lists never blocks the probe.
    let mut own = flat();
    body.set_instance_id(77);
    // A pillar the body stands in (z from 4 up to 7) would stop the downward ray at once.
    let id = own.insert_object(pillar(80.0, 80.0, 4.0, Some(77)));
    assert_eq!(id, 0);
    assert_eq!(body.get_height_above_walkable(&own), (true, 5.0));
    // The same pillar under another id blocks it.
    body.set_instance_id(78);
    assert_eq!(body.get_height_above_walkable(&own), (true, 0.0));
}

#[test]
fn the_ground_position_drops_an_airborne_body_onto_the_surface() {
    let world = flat();
    let body = walker(80.0, 80.0, 5.0);
    assert_eq!(body.ground_position(true, &world).frame.origin.z, 0.0);
    assert_eq!(body.ground_position(false, &world).frame.origin.z, 5.0);
}

#[test]
fn a_local_frame_becomes_a_position_in_the_cell_that_holds_it() {
    let world = flat();
    let body = walker(70.0, 70.0, 0.0);
    let local = Frame::new(Vector::new(30.0, 0.0, 1.0), Quaternion::IDENTITY);
    let p = body.local_to_global(&local, &world).expect("resident cell");
    assert_eq!(p.cell_id, position(100.0, 70.0, 0.0).cell_id);
    assert_eq!(p.frame.origin.z, 1.0);
    let mut inside = position(70.0, 70.0, 0.0);
    assert!(!body.find_inside_position(&mut inside, &world));
}

#[test]
fn a_listed_object_reports_its_shadow_cells() {
    let mut world = flat();
    let id = world.insert_object(bsp_box(
        position(80.0, 84.0, 0.0),
        Vector::new(-1.0, -1.0, 0.0),
        Vector::new(1.0, 1.0, 2.0),
    ));
    assert!(world.shadow_object_count(id) >= 1);
    assert!(world.shadow_objects_complete(id));
    assert_eq!(world.shadow_object_count(id + 9), 0);
    assert!(!world.shadow_objects_complete(id + 9));
}
