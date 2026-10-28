//! Real-data checks against an Asheron's Call 2 installation; they run only when `AC2_DAT_DIR`
//! points at the DATs. Every expectation is an invariant of the data (no recorded values).

use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_db::ResourceStore;
use ac2_world::{load_ac2_motion_interp_desc, Ac2WorldSource};

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::cells::CellSource;
use crate::collision_report::RecordState;
use crate::fpu::floor_z;
use crate::frame::Frame;
use crate::landdefs::lcoord_to_gid;
use crate::loader::{load_body_shape, load_terrain_block};
use crate::motion_interp::{ELEMENT_GROUND, MODE_DEFAULT};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Quaternion, Vector};

/// `m_midesc_did` of the human `VisualDesc`s and the human male visual.
const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;
const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const BLOCK: u32 = 0x7F7F_0000;

struct Scene {
    terrain: Terrain,
    body: PhysicsBody,
    forward_speed: f32,
}

fn scene(x: f32, y: f32) -> Option<Scene> {
    scene_at(0x7F, 0x7F, x, y)
}

/// A human standing at (`x`, `y`) in block (`block_x`, `block_y`), with its neighbours resident.
fn scene_at(block_x: u32, block_y: u32, x: f32, y: f32) -> Option<Scene> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    for dx in -1..=1_i32 {
        for dy in -1..=1_i32 {
            let block = ((block_x as i32 + dx) as u32) << 24 | ((block_y as i32 + dy) as u32) << 16;
            load_terrain_block(&world, &mut terrain, CellId::new(block | 0xFFFF))
                .expect("landblock decodes");
        }
    }
    let desc = load_ac2_motion_interp_desc(&store, DataId::new(HUMAN_MOTION_INTERP_DESC))
        .expect("motion interp desc decodes")
        .expect("human motion interp desc");
    let desc: Arc<_> = Arc::new((*desc).clone());
    let forward_speed = desc
        .desired_motion_values(ELEMENT_GROUND, MODE_DEFAULT, 0, 0x8000_0001, None)
        .max_forward_velocity;
    let shape = load_body_shape(&store, DataId::new(HUMAN_MALE_VISUAL))
        .expect("body setup decodes")
        .expect("human body setup");
    let cell = lcoord_to_gid(
        block_x as i32 * 8 + (x / 20.0) as i32,
        block_y as i32 * 8 + (y / 20.0) as i32,
    );
    let start = Position::new(
        cell,
        Frame::new(Vector::new(x, y, 0.0), Quaternion::IDENTITY),
    );
    let ground = -terrain
        .height_above_terrain(&start)
        .expect("block 0x7F7F terrain under the start");
    let start = Position::new(
        cell,
        Frame::new(Vector::new(x, y, ground + 1.0), Quaternion::IDENTITY),
    );
    Some(Scene {
        terrain,
        body: PhysicsBody::new(start, shape, Some(desc)),
        forward_speed,
    })
}

fn run(scene: &mut Scene, seconds: f32) {
    for _ in 0..(seconds / PHYSICS_QUANTUM).round() as usize {
        scene.body.step_quantum(&scene.terrain);
    }
}

fn height_above_ground(scene: &Scene) -> f32 {
    scene
        .terrain
        .height_above_terrain(&scene.body.position())
        .expect("on resident terrain")
}

#[test]
fn the_terrain_of_block_7f7f_is_closed_and_its_planes_hold_their_vertices() {
    let Some(scene) = scene(80.0, 80.0) else {
        return;
    };
    for local in 1..=64 {
        let cell = scene
            .terrain
            .land_cell(BLOCK | local)
            .expect("block 0x7F7F is resident");
        for polygon in cell.polygons.iter().flatten() {
            let n = polygon.plane.n;
            assert!(((n.x * n.x + n.y * n.y + n.z * n.z) - 1.0).abs() < 1e-5);
            assert!(n.z > 0.0, "terrain faces up");
            for v in polygon.vertices {
                let d = n.x * v.x + n.y * v.y + n.z * v.z + polygon.plane.d;
                assert!(d.abs() < 1e-3, "{d}");
            }
        }
    }
}

#[test]
fn a_human_settles_onto_the_terrain_of_block_7f7f() {
    let Some(mut scene) = scene(80.0, 80.0) else {
        return;
    };
    run(&mut scene, 3.0);
    let state = scene.body.movement_state();
    assert!(state.contact, "{state:?}");
    let height = height_above_ground(&scene);
    assert!(height.abs() < 0.05, "{height} {state:?}");
    let normal_z = scene
        .terrain
        .land_cell(state.cell_id)
        .and_then(|cell| {
            let origin = Vector::new(state.origin[0], state.origin[1], state.origin[2]);
            cell.find_terrain_poly(origin)
                .map(|polygon| polygon.plane.n.z)
        })
        .expect("ground polygon");
    assert_eq!(state.on_walkable, normal_z >= floor_z(), "{state:?}");
}

#[test]
fn a_human_walks_at_the_motion_interp_forward_speed() {
    let Some(mut scene) = scene(80.0, 80.0) else {
        return;
    };
    run(&mut scene, 3.0);
    assert!(
        scene.forward_speed > 0.0,
        "human ground values carry a forward speed"
    );
    scene.body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    let mut walkable_quanta = 0;
    for _ in 0..50 {
        scene.body.step_quantum(&scene.terrain);
        let state = scene.body.movement_state();
        if state.on_walkable {
            walkable_quanta += 1;
            // On walkable ground the body stays on the terrain.
            assert!(height_above_ground(&scene).abs() < 0.1, "{state:?}");
        }
    }
    assert!(walkable_quanta > 25, "{walkable_quanta}");
    let state = scene.body.movement_state();
    if state.on_walkable {
        let [vx, vy, vz] = state.velocity;
        let speed = (vx * vx + vy * vy + vz * vz).sqrt();
        assert!(
            (speed - scene.forward_speed).abs() < 0.05 * scene.forward_speed,
            "{speed} vs {}",
            scene.forward_speed
        );
    }
}

/// Block 0xCB4D: a body standing on a vertex that names a landscape material reports that
/// material as a landscape record (`CCollisionManager::SetOnLandscape` and `ReportSurfaceProperties`).
#[test]
fn a_body_on_the_landscape_reports_its_vertex_material() {
    const BLOCK_CELL: u32 = 0xCB4D_FFFF;
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let desc = world
        .load_landblock(CellId::new(BLOCK_CELL))
        .expect("landblock decodes")
        .expect("block 0xCB4D");
    let (vx, vy) = (1..16)
        .flat_map(|x| (1..16).map(move |y| (x, y)))
        .find(|&(x, y)| desc.physics_material(x, y).is_some())
        .expect("block 0xCB4D names a landscape material");
    let expected = desc
        .physics_material(vx, vy)
        .map(|did| u64::from(did.raw()))
        .expect("material");
    // A quarter metre off the vertex, so the nearest vertex is (vx, vy).
    let Some(mut scene) = scene_at(0xCB, 0x4D, vx as f32 * 10.0 + 0.5, vy as f32 * 10.0 + 0.5)
    else {
        return;
    };
    let mut surface = Vec::new();
    for _ in 0..(1.0 / PHYSICS_QUANTUM).round() as usize {
        scene.body.step_quantum(&scene.terrain);
        let materials = scene.body.contact_materials(&scene.terrain);
        if let Some(report) = scene.body.report_collisions(&materials, false) {
            surface.extend(report.surface);
        }
    }
    assert!(
        surface
            .iter()
            .any(|record| record.id == expected && record.state == RecordState::Begin),
        "landscape records {surface:?}, expected material {expected}"
    );
}

/// The loader gives each resident block's water its water type's physics material, as
/// `CLandBlock::GetWaterPhysicsMaterial` reads it, and a block without water has none.
#[test]
fn resident_water_carries_its_water_type_material() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    let mut water_blocks = 0;
    for (bx, by) in [(0x7F_u32, 0x7F_u32), (0xCB, 0x4D)] {
        for dx in -2..=2_i32 {
            for dy in -2..=2_i32 {
                let block = ((bx as i32 + dx) as u32) << 24 | ((by as i32 + dy) as u32) << 16;
                let cell = CellId::new(block | 0xFFFF);
                load_terrain_block(&world, &mut terrain, cell).expect("landblock decodes");
                let expected = world
                    .water_physics_material(cell)
                    .expect("water material reads")
                    .map(|did| u64::from(did.raw()));
                assert_eq!(terrain.water_material(block | 1), expected);
                water_blocks += usize::from(expected.is_some());
            }
        }
    }
    assert!(water_blocks > 0, "the sampled blocks hold water");
}
