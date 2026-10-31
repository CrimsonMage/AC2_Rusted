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

/// A side of a block a mover crosses, with what `handle_move_restriction` must answer.
#[derive(Debug, Clone, Copy)]
struct Crossing {
    name: &'static str,
    /// Block step across the edge.
    step: (i32, i32),
    /// Unit direction of the 2 m move.
    dir: Vector,
    /// Where the mover starts in its own block (1 m inside the edge, mid-block along it).
    start: Vector,
    /// Where it ends in the neighbouring block.
    finish: Vector,
    /// The collision normal expected: `curr_pos` lies on this side of the target cell's centre.
    normal: Vector,
}

const CROSSINGS: [Crossing; 4] = [
    Crossing {
        name: "east",
        step: (1, 0),
        dir: Vector::new(1.0, 0.0, 0.0),
        start: Vector::new(159.0, 70.0, 0.0),
        finish: Vector::new(1.0, 70.0, 0.0),
        normal: Vector::new(-1.0, 0.0, 0.0),
    },
    Crossing {
        name: "west",
        step: (-1, 0),
        dir: Vector::new(-1.0, 0.0, 0.0),
        start: Vector::new(1.0, 70.0, 0.0),
        finish: Vector::new(159.0, 70.0, 0.0),
        normal: Vector::new(1.0, 0.0, 0.0),
    },
    Crossing {
        name: "north",
        step: (0, 1),
        dir: Vector::new(0.0, 1.0, 0.0),
        start: Vector::new(70.0, 159.0, 0.0),
        finish: Vector::new(70.0, 1.0, 0.0),
        normal: Vector::new(0.0, -1.0, 0.0),
    },
    Crossing {
        name: "south",
        step: (0, -1),
        dir: Vector::new(0.0, -1.0, 0.0),
        start: Vector::new(70.0, 1.0, 0.0),
        finish: Vector::new(70.0, 159.0, 0.0),
        normal: Vector::new(0.0, 1.0, 0.0),
    },
];

impl Crossing {
    fn neighbour(&self, (x, y): (u32, u32)) -> Option<(u32, u32)> {
        let (nx, ny) = (x as i32 + self.step.0, y as i32 + self.step.1);
        ((0..256).contains(&nx) && (0..256).contains(&ny)).then_some((nx as u32, ny as u32))
    }
}

/// A passable block whose neighbour across `crossing` has passability `neighbour_passable`.
fn passable_block_beside(
    world: &Ac2WorldSource,
    crossing: &Crossing,
    neighbour_passable: bool,
) -> Option<((u32, u32), (u32, u32))> {
    (0..256u32)
        .flat_map(|x| (0..256u32).map(move |y| (x, y)))
        .filter(|&(x, y)| passable(world, x, y) == Some(true))
        .find_map(|block| {
            let next = crossing.neighbour(block)?;
            (passable(world, next.0, next.1) == Some(neighbour_passable)).then_some((block, next))
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

/// A mover on `block`'s edge, moving 2 m across `crossing`. Returns the transition after one
/// pass and where the check position started.
fn cross_edge(
    terrain: &Terrain,
    crossing: &Crossing,
    block: (u32, u32),
    next: (u32, u32),
) -> (Transition, TransitionState, Position) {
    let mut from = position(block.0, block.1, crossing.start);
    let mut end = position(next.0, next.1, crossing.finish);
    for pos in [&mut from, &mut end] {
        let ground = -terrain.height_above_terrain(pos).unwrap_or(0.0);
        pos.frame.origin.z = ground;
    }
    let (transition, state) = insert_pass(terrain, from, end);
    (transition, state, end)
}

/// Loads both blocks of a crossing found by `passable_block_beside`, or `None` (noted) without
/// game data or such a pair.
fn crossing_setup(
    crossing: &Crossing,
    neighbour_passable: bool,
) -> Option<(Arc<Ac2WorldSource>, Terrain, (u32, u32), (u32, u32))> {
    let world = open_world()?;
    let Some((block, next)) = passable_block_beside(&world, crossing, neighbour_passable) else {
        eprintln!(
            "{}: no passable block has a neighbour with passable={neighbour_passable}",
            crossing.name
        );
        return None;
    };
    let mut terrain = Terrain::new();
    for (bx, by) in [block, next] {
        load_terrain_block(&world, &mut terrain, block_id(bx, by)).expect("landblock decodes");
    }
    Some((world, terrain, block, next))
}

#[test]
fn a_move_into_an_impassable_block_is_turned_back_at_each_edge() {
    for crossing in &CROSSINGS {
        let Some((_world, terrain, block, next)) = crossing_setup(crossing, false) else {
            continue;
        };
        let target = terrain
            .land_cell(position(next.0, next.1, crossing.finish).cell_id)
            .expect("the impassable block's cell is resident");
        assert!(
            !target.passable,
            "{}: insert_block carries the block's passability",
            crossing.name
        );
        let (t, state, end) = cross_edge(&terrain, crossing, block, next);
        eprintln!(
            "{}: block {block:?} -> impassable {next:?}: {state:?}",
            crossing.name
        );
        assert_eq!(state, TransitionState::Adjusted, "{}", crossing.name);
        assert!(t.collision_info.collision_normal_valid, "{}", crossing.name);
        assert_eq!(
            t.collision_info.collision_normal, crossing.normal,
            "{}",
            crossing.name
        );
        // The 2 m path component along the move is negated: the check position ends back
        // 2 m behind where it was, i.e. where the mover started; the other axis is unchanged.
        let pushed = t.sphere_path.check_pos.frame.origin;
        let expected = (
            end.frame.origin.x - crossing.dir.x * 2.0,
            end.frame.origin.y - crossing.dir.y * 2.0,
        );
        assert!(
            (pushed.x - expected.0).abs() < 1e-3 && (pushed.y - expected.1).abs() < 1e-3,
            "{}: {pushed:?} vs {expected:?}",
            crossing.name
        );
    }
}

#[test]
fn the_same_move_into_a_passable_neighbour_is_not_restricted() {
    for crossing in &CROSSINGS {
        let Some((_world, terrain, block, next)) = crossing_setup(crossing, true) else {
            continue;
        };
        let (t, state, end) = cross_edge(&terrain, crossing, block, next);
        eprintln!(
            "{}: block {block:?} -> passable {next:?}: {state:?}",
            crossing.name
        );
        assert!(
            !t.collision_info.collision_normal_valid,
            "{}",
            crossing.name
        );
        assert_ne!(state, TransitionState::Adjusted, "{}", crossing.name);
        assert_eq!(
            t.sphere_path.check_pos.frame.origin.x, end.frame.origin.x,
            "{}",
            crossing.name
        );
        assert_eq!(
            t.sphere_path.check_pos.frame.origin.y, end.frame.origin.y,
            "{}",
            crossing.name
        );
    }
}
