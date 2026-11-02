//! `SmartBox::UpdateCamera`'s collision against the start dungeon of the DATs (they run only
//! when `AC2_DAT_DIR` points at them). Expectations are invariants, nothing recorded from data.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_physics::{
    camera_ethereal_mask, CellSource, Frame, LosParameters, Position, Quaternion, RayIntersection,
    Terrain, Vector, WeenieTypes,
};
use ac2_world::Ac2WorldSource;

use super::*;
use crate::camera::CameraManager;
use crate::targeting::NoObjects;

const PLAYER: u64 = 0x5000_0001;
const BLOCK: u32 = 0x0299_0000;

struct Dungeon {
    terrain: Terrain,
    collision: CameraCollision,
}

fn dungeon() -> Option<Dungeon> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    ac2_db::install_ac2_property_types(&store).expect("property types");
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    assert!(crate::load_dat_block(&world, &mut terrain, BLOCK).expect("block loads"));
    let did = camera_object_did(&store).expect("enum reads");
    assert_eq!(did, Some(DataId::new(0x0400_058F)));
    let shape = ac2_physics::load_body_shape(&store, did.unwrap())
        .expect("shape reads")
        .expect("camera shape");
    assert_eq!(shape.spheres.len(), 1);
    assert_eq!(shape.spheres[0].radius, 0.25);
    let mut collision = CameraCollision::new();
    collision.set_object(shape);
    Some(Dungeon { terrain, collision })
}

fn subject(cell: u32, origin: Vector) -> CameraSubject {
    CameraSubject {
        id: PLAYER,
        position: Position::new(
            cell,
            Frame::new(origin, Quaternion::new(1.0, 0.0, 0.0, 0.0)),
        ),
        height: 1.8,
        radius: 0.4,
        scale_z: 1.0,
        local_velocity: Vector::new(0.0, 0.0, 0.0),
        element: 0,
        no_draw: false,
    }
}

/// The straight line from `from` to `to` meets nothing the camera cannot pass.
fn line_is_clear(terrain: &Terrain, from: &Position, to: &Position) -> bool {
    let mut los = LosParameters::new();
    los.set_from_begin_and_end_position(from, to);
    los.intersect = RayIntersection::PhysicsRep;
    los.set_los_params(&WeenieTypes::default(), camera_ethereal_mask());
    terrain.los_check(&mut los) && los.trace_result() == 0
}

fn at_wall(spot: (u32, Vector)) {
    let Some(mut world) = dungeon() else {
        return;
    };
    let (cell, origin) = spot;
    assert!(
        world.terrain.env_cell(cell).is_some(),
        "{cell:08x} resident"
    );
    let player = subject(cell, origin);
    let mut camera = CameraManager::new();
    camera.set_player(&player, 0.75);
    let head = {
        let offset = camera.get_target_offset(Some(&player));
        Vector::new(
            origin.x + offset.x,
            origin.y + offset.y,
            origin.z + offset.z,
        )
    };
    let (mut pulled, mut free, mut tried) = (0, 0, 0);
    for step in 0..24 {
        let angle = f64::from(step) * std::f64::consts::TAU / 24.0;
        for wanted in [1.0_f64, 3.0, 6.0] {
            let desired = Position::new(
                cell,
                Frame::new(
                    Vector::new(
                        (f64::from(head.x) + angle.cos() * wanted) as f32,
                        (f64::from(head.y) + angle.sin() * wanted) as f32,
                        head.z + 0.3,
                    ),
                    Quaternion::new(1.0, 0.0, 0.0, 0.0),
                ),
            );
            let current = player.position;
            let got = world.collision.trace(
                &mut camera,
                &player,
                &desired,
                &current,
                0.1,
                &world.terrain,
            );
            let Some(got) = got else { continue };
            tried += 1;
            // In a resident interior cell, never farther than wanted, with a clear line from
            // the head to it.
            assert!(world.terrain.env_cell(got.cell_id).is_some(), "{got:?}");
            let head_position =
                Position::new(cell, Frame::new(head, Quaternion::new(1.0, 0.0, 0.0, 0.0)));
            let reach = head_position.offset_to(&got);
            let length = f64::from(reach.x)
                .hypot(f64::from(reach.y))
                .hypot(f64::from(reach.z));
            let limit = (wanted * wanted + 0.09).sqrt();
            assert!(
                length <= limit + 1e-3,
                "{length} > {limit} at {step}/{wanted}"
            );
            if length < limit - 0.05 {
                pulled += 1;
            } else {
                free += 1;
            }
            if length < limit - 0.05 {
                // What the pull-in was needed for: the wanted place was behind something.
                assert!(!line_is_clear(&world.terrain, &head_position, &desired));
            }
            assert!(line_is_clear(&world.terrain, &head_position, &got));
        }
    }
    eprintln!("{cell:08x}: {tried} traced, {pulled} pulled in, {free} reached the wanted place");
    assert!(tried > 0);
    assert!(pulled > 0, "a wall pulled nothing in");
    assert!(free > 0, "nothing reached its wanted place");
}

#[test]
fn a_camera_behind_a_wall_stands_on_the_players_side_of_it() {
    at_wall((0x0299_011B, Vector::new(49.5, 230.4, -43.8)));
}

#[test]
fn outdoors_the_camera_stays_above_the_terrain() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    ac2_db::install_ac2_property_types(&store).expect("property types");
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    let block = 0x7097_0000;
    assert!(crate::load_dat_block(&world, &mut terrain, block).expect("block loads"));
    let did = camera_object_did(&store).unwrap().unwrap();
    let mut collision = CameraCollision::new();
    collision.set_object(ac2_physics::load_body_shape(&store, did).unwrap().unwrap());
    // Stand on the ground in the middle of the block.
    let cell = block | 0x0024;
    let mut at = Position::new(
        cell,
        Frame::new(
            Vector::new(70.0, 70.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    );
    let ground = terrain
        .terrain_height(&at)
        .expect("terrain under the player");
    at.frame.origin.z = ground;
    let player = subject(cell, at.frame.origin);
    let mut camera = CameraManager::new();
    camera.set_player(&player, 0.75);
    let (mut below, mut checked) = (0, 0);
    for step in 0..24 {
        let angle = f64::from(step) * std::f64::consts::TAU / 24.0;
        for drop in [0.0_f32, -6.0] {
            let mut desired = at;
            desired.frame.origin = Vector::new(
                (70.0 + angle.cos() * 5.0) as f32,
                (70.0 + angle.sin() * 5.0) as f32,
                ground + 1.35 + drop,
            );
            let Some(got) = collision.trace(&mut camera, &player, &desired, &at, 0.1, &terrain)
            else {
                continue;
            };
            checked += 1;
            let height = terrain
                .terrain_height(&got)
                .expect("terrain under the camera");
            if desired.frame.origin.z < height {
                below += 1;
            }
            assert!(
                got.frame.origin.z >= height - 0.01,
                "camera {} under the terrain {height} (wanted {})",
                got.frame.origin.z,
                desired.frame.origin.z
            );
        }
    }
    assert!(
        checked > 0 && below > 0,
        "{checked} traced, {below} wanted below ground"
    );
}

#[test]
fn rotating_the_camera_pulls_it_in_instead_of_through_the_wall() {
    let Some(mut world) = dungeon() else { return };
    let cell = 0x0299_011B;
    let player = subject(cell, Vector::new(49.5, 230.4, -43.8));
    let mut camera = CameraManager::new();
    camera.set_player(&player, 0.75);
    let mut scene = SceneCamera {
        current: Position::new(
            cell,
            Frame::new(
                Vector::new(49.5, 229.0, -42.5),
                Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ),
        ),
        ..SceneCamera::default()
    };
    let mut ends = Vec::new();
    for step in 0..36 {
        let angle = f64::from(step) * std::f64::consts::TAU / 36.0;
        camera.viewer_offset = Vector::new(angle.sin() as f32, angle.cos() as f32, 0.25);
        scene.update_collided(
            &mut camera,
            &player,
            &NoObjects,
            &mut world.collision,
            &world.terrain,
            0.1,
            false,
        );
        ends.push(scene.current);
    }
    let head = Vector::new(49.5, 230.4, -43.8 + 1.35);
    let head_position = Position::new(cell, Frame::new(head, Quaternion::new(1.0, 0.0, 0.0, 0.0)));
    let wanted = f64::from(camera.camera_distance);
    let mut pulled = 0;
    for end in &ends {
        assert!(world.terrain.env_cell(end.cell_id).is_some());
        let reach = head_position.offset_to(end);
        let length = f64::from(reach.x)
            .hypot(f64::from(reach.y))
            .hypot(f64::from(reach.z));
        assert!(length <= wanted + 0.5, "{length} past the wanted {wanted}");
        assert!(line_is_clear(&world.terrain, &head_position, end));
        if length < wanted - 0.2 {
            pulled += 1;
        }
    }
    eprintln!(
        "distance {wanted}: {pulled} of {} rotations pulled in",
        ends.len()
    );
}
