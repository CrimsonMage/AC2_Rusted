//! Server objects in the world's object lists (`CObjCell::AddObject`, `CPhysicsObj::change_cell`):
//! a mover stops at a dynamic cylinder, passes an ethereal one, follows an object as it moves
//! between cells, and is not blocked by its own entry.

use std::sync::Arc;

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::cells::CellSource;
use crate::frame::Frame;
use crate::landdefs::{lcoord_to_gid, SIDE_VERTEX_COUNT};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::objects::{ObjectShape, StaticObject};
use crate::physics_obj::BodyShape;
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::terrain::Terrain;
use crate::test_support::{desc, walking_values};
use crate::vector::{Quaternion, Sphere, Vector};

const BLOCK: u32 = 0x7F7F_FFFF;

fn cell_at(x: f32, y: f32) -> u32 {
    lcoord_to_gid(0x7F * 8 + (x / 20.0) as i32, 0x7F * 8 + (y / 20.0) as i32)
}

fn position(x: f32, y: f32, z: f32) -> Position {
    Position::new(
        cell_at(x, y),
        Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY),
    )
}

fn flat() -> Terrain {
    let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK, &heights, |_, _| false);
    terrain
}

fn pillar() -> Arc<ObjectShape> {
    Arc::new(ObjectShape::new(
        Vec::new(),
        vec![CylSphere {
            low: Vector::ZERO,
            height: 3.0,
            radius: 0.5,
        }],
        Vec::new(),
    ))
}

/// A dynamic mobile object of the pillar shape at `(80.2, 84)`.
fn creature(terrain: &mut Terrain, x: f32, y: f32) -> u32 {
    let at = position(x, y, 0.0);
    terrain.insert_object(StaticObject::dynamic(
        at,
        Vector::new(1.0, 1.0, 1.0),
        pillar(),
        true,
    ))
}

fn walker(x: f32, y: f32) -> PhysicsBody {
    let mut air = walking_values();
    air.forward_acceleration = 0.0;
    air.backward_acceleration = 0.0;
    air.strafe_acceleration = 0.0;
    let shape = BodyShape {
        spheres: vec![
            Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4),
            Sphere::new(Vector::new(0.0, 0.0, 1.2), 0.4),
        ],
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let desc = desc(&[(ELEMENT_GROUND, walking_values()), (ELEMENT_AIR, air)]);
    PhysicsBody::new(position(x, y, 0.0), shape, Some(desc))
}

/// Settles the body, then walks it north for `seconds`, returning every origin on the way.
fn walk_north(body: &mut PhysicsBody, world: &Terrain, seconds: f32) -> Vec<[f32; 3]> {
    for _ in 0..5 {
        body.step_quantum(world);
    }
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        heading: Some(0.0),
        turn: 0.0,
    });
    let quanta = (seconds / PHYSICS_QUANTUM).round() as usize;
    (0..quanta)
        .map(|_| {
            body.step_quantum(world);
            body.movement_state().origin
        })
        .collect()
}

#[test]
fn a_mover_stops_at_a_dynamic_cylinder() {
    let mut world = flat();
    creature(&mut world, 80.2, 84.0);
    let mut body = walker(80.0, 80.0);
    let path = walk_north(&mut body, &world, 3.0);
    for p in &path {
        let (dx, dy) = (p[0] - 80.2, p[1] - 84.0);
        assert!((dx * dx + dy * dy).sqrt() >= 0.9 - 0.02, "{p:?}");
    }
    assert!(path.last().expect("steps")[1] > 82.5, "{path:?}");
}

#[test]
fn a_mover_passes_an_ethereal_dynamic_object() {
    let mut world = flat();
    let id = creature(&mut world, 80.2, 84.0);
    world.set_object_ethereal(id, true);
    let mut body = walker(80.0, 80.0);
    let path = walk_north(&mut body, &world, 3.0);
    let last = *path.last().expect("steps");
    assert!(last[1] > 86.0, "passed the ethereal object: {last:?}");
}

#[test]
fn a_moved_object_is_listed_in_its_new_cells_only() {
    let mut world = flat();
    let id = creature(&mut world, 80.2, 84.0);
    let before = cell_at(80.2, 84.0);
    let away = cell_at(150.2, 84.0);
    assert!(world.land_cell(before).expect("cell").objects.contains(&id));
    world.move_object(id, position(150.2, 84.0, 0.0));
    assert!(!world.land_cell(before).expect("cell").objects.contains(&id));
    assert!(world.land_cell(away).expect("cell").objects.contains(&id));

    // Out of the path, the mover walks past where the object was.
    let mut body = walker(80.0, 80.0);
    let path = walk_north(&mut body, &world, 3.0);
    assert!(path.last().expect("steps")[1] > 82.5, "{path:?}");

    // Back into the path, it stops the mover again.
    world.move_object(id, position(80.2, 84.0, 0.0));
    assert!(world.land_cell(before).expect("cell").objects.contains(&id));
    let mut body = walker(80.0, 80.0);
    let path = walk_north(&mut body, &world, 3.0);
    for p in &path {
        let (dx, dy) = (p[0] - 80.2, p[1] - 84.0);
        assert!((dx * dx + dy * dy).sqrt() >= 0.9 - 0.02, "{p:?}");
    }
}

#[test]
fn a_mover_is_not_blocked_by_its_own_entry() {
    let mut world = flat();
    let id = creature(&mut world, 80.0, 80.0);
    let mut body = walker(80.0, 80.0);
    body.set_world_id(Some(id));
    let path = walk_north(&mut body, &world, 3.0);
    assert!(path.last().expect("steps")[1] > 82.5, "{path:?}");
}

#[test]
fn removed_ids_are_reused_and_removed_objects_stop_blocking() {
    let mut world = flat();
    let id = creature(&mut world, 80.2, 84.0);
    world.remove_object(id);
    assert!(world
        .land_cell(cell_at(80.2, 84.0))
        .expect("cell")
        .objects
        .is_empty());
    let mut body = walker(80.0, 80.0);
    let path = walk_north(&mut body, &world, 3.0);
    assert!(path.last().expect("steps")[1] > 82.5, "{path:?}");
    assert_eq!(creature(&mut world, 120.2, 84.0), id);
}

/// A dynamic mobile pillar at `(80.2, 84)` whose blocking mask is `bits` (`CPhysicsObj::
/// m_blocking_mask`, the bits its body took from its `EntityDesc`).
fn creature_with_blocking(terrain: &mut Terrain, bits: u64) -> u32 {
    let at = position(80.2, 84.0, 0.0);
    terrain.insert_object(
        StaticObject::dynamic(at, Vector::new(1.0, 1.0, 1.0), pillar(), true)
            .with_blocking_mask(bits),
    )
}

#[test]
fn a_dynamic_entry_whose_blocking_bit_the_mover_shares_is_passed_through() {
    let mut world = flat();
    creature_with_blocking(&mut world, 1 << 4);
    let mut body = walker(80.0, 80.0);
    // The walker's movement physics set shares bit 4 with the entry's blocking mask.
    body.collision_mut().update_movement(4, true, |_| true);
    let path = walk_north(&mut body, &world, 3.0);
    let last = *path.last().expect("steps");
    assert!(last[1] > 86.0, "passed the shared-bit object: {last:?}");
}

#[test]
fn a_dynamic_entry_whose_blocking_bit_the_mover_lacks_still_blocks() {
    let mut world = flat();
    creature_with_blocking(&mut world, 1 << 4);
    let mut body = walker(80.0, 80.0);
    // Bit 5 is shared with nothing in the entry's mask, so the entry blocks as a solid object.
    body.collision_mut().update_movement(5, true, |_| true);
    let path = walk_north(&mut body, &world, 3.0);
    for p in &path {
        let (dx, dy) = (p[0] - 80.2, p[1] - 84.0);
        assert!((dx * dx + dy * dy).sqrt() >= 0.9 - 0.02, "{p:?}");
    }
    assert!(path.last().expect("steps")[1] > 82.5, "{path:?}");
}

#[test]
fn a_dynamic_entry_carries_the_blocking_bits_it_is_set_to() {
    let mut world = flat();
    let id = creature_with_blocking(&mut world, 0);
    assert_eq!(world.object_blocking(id), Some(0));
    world.set_object_blocking(id, (1 << 4) | (1 << 9));
    assert_eq!(world.object_blocking(id), Some((1 << 4) | (1 << 9)));
    world.move_object(id, position(80.2, 84.0, 0.0));
    assert_eq!(world.object_blocking(id), Some((1 << 4) | (1 << 9)));
}

/// A dynamic mobile pillar at `(80.2, 84)` with the `InstanceID` `id` and blocking mask `bits`.
fn server_creature(terrain: &mut Terrain, id: u64, bits: u64) {
    let at = position(80.2, 84.0, 0.0);
    terrain.insert_object(
        StaticObject::dynamic(at, Vector::new(1.0, 1.0, 1.0), pillar(), true)
            .with_instance_id(id)
            .with_blocking_mask(bits),
    );
}

const CREATURE: u64 = 0x5000_0123;

fn blocked(body: &mut PhysicsBody, world: &Terrain) -> bool {
    // A blocked mover slides around the pillar and never comes within its radius.
    let path = walk_north(body, world, 3.0);
    path.iter().all(|p| {
        let (dx, dy) = (p[0] - 80.2, p[1] - 84.0);
        (dx * dx + dy * dy).sqrt() >= 0.85
    })
}

#[test]
fn an_object_on_the_movers_temporary_ethereal_list_is_passed_through() {
    // `CollisionParameters::CheckIfSolid`: the temporary ethereal list (`AddToTempEtherealList`)
    // makes an object that would block the mover passable.
    let mut world = flat();
    server_creature(&mut world, CREATURE, 0);
    assert!(blocked(&mut walker(80.0, 80.0), &world));
    let mut body = walker(80.0, 80.0);
    assert!(body.add_to_temp_ethereal_list(CREATURE));
    assert!(!blocked(&mut body, &world));
}

#[test]
fn the_solid_and_ethereal_lists_name_the_object_by_its_instance_id() {
    let mut world = flat();
    server_creature(&mut world, CREATURE, 1 << 4);
    // The mover is ethereal to the object's type: it passes, unless the object is listed solid.
    let ethereal_to_type = |body: &mut PhysicsBody| {
        body.collision_mut().update_movement(4, true, |_| true);
    };
    let mut passes = walker(80.0, 80.0);
    ethereal_to_type(&mut passes);
    assert!(!blocked(&mut passes, &world));
    let mut solid = walker(80.0, 80.0);
    ethereal_to_type(&mut solid);
    let mut list = crate::WeenieTypes::empty();
    list.add_to_solid_list(CREATURE);
    let mut info = crate::CollisionParameters::default();
    info.set_weenie_movement_type(&list);
    solid.collision_mut().merge_set_state(&info);
    assert!(blocked(&mut solid, &world));
    // An unrelated id in the list changes nothing (the slot index is not the id).
    let mut other = walker(80.0, 80.0);
    ethereal_to_type(&mut other);
    let mut list = crate::WeenieTypes::empty();
    list.add_to_solid_list(CREATURE + 1);
    let mut info = crate::CollisionParameters::default();
    info.set_weenie_movement_type(&list);
    other.collision_mut().merge_set_state(&info);
    assert!(!blocked(&mut other, &world));
    // Without the shared type the object blocks unless its id is on the ethereal list.
    let mut listed = walker(80.0, 80.0);
    let mut ethereal = crate::WeenieTypes::empty();
    ethereal.add_to_ethereal_list(CREATURE);
    let mut info = crate::CollisionParameters::default();
    info.set_weenie_movement_type(&ethereal);
    listed.collision_mut().merge_set_state(&info);
    assert!(!blocked(&mut listed, &world));
}
