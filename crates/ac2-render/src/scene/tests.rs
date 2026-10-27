//! The shared scene against real data (`AC2_DAT_DIR`): loads arrive from the loader thread and a
//! frame keeps only the far chunks the camera can see.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_world::Ac2WorldSource;

use super::{Loaded, WorldScene};
use crate::prefs::SceneOptions;
use crate::Camera;

#[test]
fn a_town_block_loads_off_thread_and_culls_far_chunks_behind_the_camera() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let size = world.block_size();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0xCB, 0x4D));
    let update = scene.wait();
    let blocks = update
        .loaded
        .iter()
        .filter(|l| matches!(l, Loaded::Block(_)))
        .count();
    assert_eq!(blocks, 9, "the 3 x 3 ring");
    assert!(update.loaded.iter().any(|l| matches!(l, Loaded::Far(_))));
    let centre = Vec3::new(203.5 * size, 77.5 * size, 60.0);
    let camera = |target: Vec3| Camera {
        eye: centre,
        target: centre + target,
        fov_y: 60_f32.to_radians(),
        near: 0.5,
        far: 4000.0,
    };
    let north = scene.frame(&camera(Vec3::Y), 1.0);
    let south = scene.frame(&camera(-Vec3::Y), 1.0);
    assert!(!north.placements.is_empty(), "the town's objects lay out");
    let total = scene.far_bounds.len() - crate::terrain::FIRST_FAR_CHUNK;
    eprintln!(
        "far chunks: {} north, {} south of {total}",
        north.far_chunks.len(),
        south.far_chunks.len()
    );
    assert!(north.far_chunks.len() < total && south.far_chunks.len() < total);
    assert!(north
        .far_chunks
        .iter()
        .all(|c| !south.far_chunks.contains(c) || {
            // Chunks straddling the east-west line can be seen both ways.
            let (min, max) = scene.far_bounds[*c];
            min.y <= centre.y && max.y >= centre.y
        }));
}

#[test]
fn a_draw_distance_change_lays_the_far_ring_out_again_on_the_loader_thread() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0xCB, 0x4D));
    scene.wait();
    let before = scene.far_bounds.len();
    assert!(before > 0, "the default far ring loads");

    // The nearest draw distance has no far ring: the old chunks go at once and the new layout
    // loads with the next wait.
    scene.set_options(SceneOptions {
        far_radius: 0,
        ..SceneOptions::default()
    });
    assert!(
        scene.far_bounds.is_empty(),
        "no far chunk while the layout loads"
    );
    let update = scene.wait();
    assert!(update.loaded.iter().any(|l| matches!(l, Loaded::Layout)));
    assert_ne!(
        scene.far_bounds.len(),
        before,
        "the far ring is the new one"
    );

    // Setting the same radius again asks for nothing.
    let now = scene.far_bounds.len();
    scene.set_options(SceneOptions {
        far_radius: 0,
        ..SceneOptions::default()
    });
    assert_eq!(scene.far_bounds.len(), now);
    assert!(scene.wait().loaded.is_empty());
}
