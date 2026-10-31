//! `CLandCell::find_env_collisions` / `handle_move_restriction` on real landblocks: the
//! per-block passability the world source reads (`CLandBlockData::m_bPassable`) reaches the land
//! cells through the loader, and a transition into an impassable block is turned back. Runs only
//! when `AC2_DAT_DIR` points at the DATs.

use std::sync::Arc;

use ac2_core::CellId;
use ac2_db::ResourceStore;
use ac2_world::Ac2WorldSource;

use crate::cells::CellSource;
use crate::collision_types::CollisionParameters;
use crate::frame::Frame;
use crate::landdefs::lcoord_to_gid;
use crate::loader::load_terrain_block;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::transition::{Transition, TransitionObject, TransitionState};
use crate::vector::{Quaternion, Sphere, Vector};

fn open_world() -> Option<Arc<Ac2WorldSource>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    Some(
        Ac2WorldSource::open(store)
            .expect("world source opens")
            .expect("AC2 landscape"),
    )
}

fn block_id(x: u32, y: u32) -> CellId {
    CellId::new(x << 24 | y << 16 | 0xFFFF)
}

/// Whether block (`x`, `y`) exists and, per the world source, is (im)passable.
fn passable(world: &Ac2WorldSource, x: u32, y: u32) -> Option<bool> {
    let block = block_id(x, y);
    if !world.has_landblock(block) {
        return None;
    }
    Some(world.load_landblock(block).ok()??.is_passable())
}

/// A passable block and its eastern neighbour whose passability is `east`.
fn passable_block_before(world: &Ac2WorldSource, east: bool) -> Option<(u32, u32)> {
    (0..255u32)
        .flat_map(|x| (0..256u32).map(move |y| (x, y)))
        .find(|&(x, y)| {
            passable(world, x, y) == Some(true) && passable(world, x + 1, y) == Some(east)
        })
}

/// The south-west cell of block (`x`, `y`) with a position `local` metres into it.
fn position(x: u32, y: u32, local: Vector) -> Position {
    let cell = lcoord_to_gid(
        (x * 8) as i32 + (local.x / 20.0) as i32,
        (y * 8) as i32 + (local.y / 20.0) as i32,
    );
    Position::new(cell, Frame::new(local, Quaternion::IDENTITY))
}

/// Runs one `CTransition::transitional_insert` pass for a small sphere from `from` to `end`.
fn insert_pass(terrain: &Terrain, from: Position, end: Position) -> (Transition, TransitionState) {
    let spheres = [Sphere::new(Vector::new(0.0, 0.0, 1.0), 0.1)];
    let collision = CollisionParameters::default();
    let object = TransitionObject {
        state: 0,
        spheres: &spheres,
        scale: 1.0,
        step_up_height: 0.0,
        step_down_height: 0.0,
        affected_by_gravity: false,
        contact: None,
        sliding_normal: None,
        frames_stationary_fall: 0,
        containment_radius: 0.1,
        collision: &collision,
        identity: None,
        instance_id: 0x99,
        blocking: Default::default(),
        missile_proximity: None,
    };
    let mut transition = Transition::new(&object, Some(from.cell_id), Some(from), end);
    transition.sphere_path.set_check_pos(end, Some(end.cell_id));
    let state = transition.transitional_insert(1, terrain);
    (transition, state)
}

/// A mover on the western block's eastern edge, moving 2 m east. Returns the transition after
/// one pass and where the check position started.
fn cross_east_edge(
    terrain: &Terrain,
    (x, y): (u32, u32),
) -> (Transition, TransitionState, Position) {
    // 1 m inside the western block, ending 1 m inside the eastern one; mid-block in y.
    let mut from = position(x, y, Vector::new(159.0, 70.0, 0.0));
    let mut end = position(x + 1, y, Vector::new(1.0, 70.0, 0.0));
    for pos in [&mut from, &mut end] {
        let ground = -terrain.height_above_terrain(pos).unwrap_or(0.0);
        pos.frame.origin.z = ground;
    }
    let (transition, state) = insert_pass(terrain, from, end);
    (transition, state, end)
}

#[test]
fn a_move_into_an_impassable_block_is_turned_back_at_the_edge() {
    let Some(world) = open_world() else {
        return;
    };
    let Some(edge) = passable_block_before(&world, false) else {
        eprintln!("no passable block borders an impassable one to its east");
        return;
    };
    let mut terrain = Terrain::new();
    for (bx, by) in [edge, (edge.0 + 1, edge.1)] {
        load_terrain_block(&world, &mut terrain, block_id(bx, by)).expect("landblock decodes");
    }
    let target = terrain
        .land_cell(position(edge.0 + 1, edge.1, Vector::new(1.0, 70.0, 0.0)).cell_id)
        .expect("the impassable block's cell is resident");
    assert!(
        !target.passable,
        "insert_block carries the block's passability"
    );
    let (t, state, end) = cross_east_edge(&terrain, edge);
    eprintln!("block {edge:?} -> east impassable: {state:?}");
    assert_eq!(state, TransitionState::Adjusted);
    assert!(t.collision_info.collision_normal_valid);
    assert_eq!(
        t.collision_info.collision_normal,
        Vector::new(-1.0, 0.0, 0.0)
    );
    // The 2 m eastward path component is flipped: the check position is back at the edge side
    // of the impassable block (x = 1 - 2 in its frame), i.e. where the mover started.
    let pushed = t.sphere_path.check_pos.frame.origin;
    assert!(
        (pushed.x - (end.frame.origin.x - 2.0)).abs() < 1e-3,
        "{pushed:?}"
    );
    assert!((pushed.y - end.frame.origin.y).abs() < 1e-3);
}

#[test]
fn the_same_move_into_a_passable_neighbour_is_not_restricted() {
    let Some(world) = open_world() else {
        return;
    };
    let Some(edge) = passable_block_before(&world, true) else {
        eprintln!("no two passable blocks border each other");
        return;
    };
    let mut terrain = Terrain::new();
    for (bx, by) in [edge, (edge.0 + 1, edge.1)] {
        load_terrain_block(&world, &mut terrain, block_id(bx, by)).expect("landblock decodes");
    }
    let (t, state, end) = cross_east_edge(&terrain, edge);
    eprintln!("block {edge:?} -> east passable: {state:?}");
    assert!(!t.collision_info.collision_normal_valid);
    assert_ne!(state, TransitionState::Adjusted);
    assert_eq!(t.sphere_path.check_pos.frame.origin.x, end.frame.origin.x);
}
