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

#[test]
fn a_frill_takes_the_self_shadow_of_the_terrain_cell_under_it() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let size = world.block_size();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0xCB, 0x4D));
    scene.wait();
    let centre = Vec3::new(203.5 * size, 77.5 * size, 60.0);
    let camera = Camera {
        eye: centre,
        target: centre + Vec3::Y,
        fov_y: 60_f32.to_radians(),
        near: 0.5,
        far: 4000.0,
    };
    let frame = scene.frame(&camera, 1.0);
    let shades: Vec<f32> = frame
        .placements
        .iter()
        .filter(|p| p.frill_normal.is_some())
        .map(|p| p.frill_shade)
        .collect();
    eprintln!("frill shades: {shades:?}");
    // Each frill's factor is the mean of its terrain cell's four corner alphas: a quarter step.
    assert!(!shades.is_empty(), "the town has grass frills");
    assert!(shades
        .iter()
        .all(|s| [0.0, 0.25, 0.5, 0.75, 1.0].contains(s)));
    // Scenery is never shaded.
    assert!(frame
        .placements
        .iter()
        .filter(|p| p.frill_normal.is_none())
        .all(|p| p.frill_shade == 1.0));
}

/// A dungeon block (`0x02BB`) loads every cell, and a camera inside one sees the cells its portals
/// open on, not the whole block.
#[test]
fn a_dungeon_camera_sees_the_cells_its_portals_open_on() {
    use crate::frustum::Frustum;
    use crate::interior_cull::InteriorCull;
    use crate::object::RenderSettings;

    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = Ac2WorldSource::open(Arc::clone(&store)).unwrap().unwrap();
    let block = ac2_core::CellId::new(0x02BB_0100);
    let interiors = super::load_block_interiors(&world, &store, block, &RenderSettings::default())
        .expect("the dungeon has cells");
    let cull = &interiors.cull;
    assert!(cull.cells.len() > 10, "{} cells", cull.cells.len());
    assert_eq!(interiors.cells.len(), interiors.cell_ids.len());
    assert_eq!(interiors.objects.len(), interiors.object_cells.len());
    let mut most = 0;
    for id in [0x02BB_0100_u32, 0x02BB_0101, 0x02BB_0102] {
        let Some(cell) = cull.cells.get(&id) else {
            continue;
        };
        let centre = (cell.bbox.min + cell.bbox.max) * 0.5;
        assert!(cull.cell_at(centre).is_some(), "{id:08X}");
        for target in [Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y] {
            let camera = Camera {
                eye: centre,
                target: centre + target,
                fov_y: 70_f32.to_radians(),
                near: 0.1,
                far: 1000.0,
            };
            let frustum = Frustum::from_camera(&camera, 1.5);
            let mut walk = InteriorCull::new(&cull.cells, &cull.groups, centre);
            walk.cull_cells(id, &frustum, cull.culling_plane);
            let seen = walk.finish().visible;
            assert!(seen.contains(&id), "the start cell draws");
            assert!(seen.len() < cull.cells.len(), "not the whole block");
            most = most.max(seen.len());
        }
    }
    assert!(most > 1, "a portal in view opens another cell");
}
