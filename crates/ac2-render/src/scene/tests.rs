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

/// The meadow (block `0x408E`, retail's grass reference at 12.1N 49.2W): each frill of retail's
/// ring is drawn `FrillSettings::copies` times, the copies standing on the ground near it.
#[test]
fn the_near_grass_is_drawn_in_jittered_copies_on_the_ground() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let size = world.block_size();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0x40, 0x8E));
    scene.wait();
    // The middle of a 20 m landscape cell, so retail's 40 m ring is centred on it.
    let centre = Vec3::new(64.5 * size + 10.0, 142.5 * size + 10.0, 0.0);
    let camera = Camera {
        eye: centre + Vec3::Z * 200.0,
        target: centre + Vec3::new(0.0, 1.0, 200.0),
        fov_y: 60_f32.to_radians(),
        near: 0.5,
        far: 4000.0,
    };
    let mut near_frills = |copies: u32| {
        let mut options = SceneOptions::default();
        options.frill.copies = copies;
        scene.set_options(options);
        let frame = scene.frame(&camera, 1.0);
        frame
            .placements
            .into_iter()
            .filter(|p| p.frill_normal.is_some())
            .filter(|p| {
                p.world
                    .w_axis
                    .truncate()
                    .truncate()
                    .distance(centre.truncate())
                    < 35.0
            })
            .collect::<Vec<_>>()
    };
    let retail = near_frills(1);
    let dense = near_frills(8);
    eprintln!(
        "near frills: retail {}, dense {}",
        retail.len(),
        dense.len()
    );
    assert!(retail.len() >= 20, "the meadow has grass");
    assert!(dense.len() >= retail.len() * 7, "copies fill in the meadow");
    // Every copy stands within its spread of an authored frill, at about that frill's height.
    for copy in &dense {
        let at = copy.world.w_axis.truncate();
        assert!(
            retail.iter().any(|r| {
                let origin = r.world.w_axis.truncate();
                (at.x - origin.x).abs() <= 1.0 + 1e-3
                    && (at.y - origin.y).abs() <= 1.0 + 1e-3
                    && (at.z - origin.z).abs() < 2.0
            }) || at.truncate().distance(centre.truncate()) > 33.5,
            "copy at {at} is near no frill"
        );
    }
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

#[test]
fn an_fx_light_over_the_ground_pools_its_light_there_for_one_frame() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let size = world.block_size();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0xCB, 0x4D));
    let _ = scene.wait();
    let materials = scene
        .light_decal_materials
        .expect("the light pool materials load with the scene");
    assert!(
        materials.iter().all(|m| m.raw() >> 24 == 0x31),
        "{materials:?}"
    );
    let (x, y) = (203.5 * size, 77.5 * size);
    let ground = scene
        .heights_at(x, y, size)
        .and_then(|heights| heights.height_at(x, y))
        .expect("the centre block is resident");
    // `CPartArray::CreateLightSource`: a point light (range 5) 2 m over the ground.
    scene.start_object_light(
        9,
        ac2_assets::Ac2FxLight {
            offset: Vec3::ZERO,
            source: ac2_assets::Ac2LightSource::default(),
            lifespan: 0.0,
            fade_time: 0.0,
        },
    );
    scene.place_objects(&[super::ObjectPlacement {
        emitter: 9,
        cell: 0xCB4D_0001,
        frame: ac2_math::Frame::new(Vec3::new(x, y, ground + 2.0), ac2_math::Quat::IDENTITY),
        scale: Vec3::ONE,
        bones: Vec::new(),
        named_bones: Vec::new(),
        parents: std::sync::Arc::from(Vec::new()),
    }]);
    let camera = Camera {
        eye: Vec3::new(x, y - 10.0, ground + 5.0),
        target: Vec3::new(x, y, ground),
        fov_y: 1.0,
        near: 0.5,
        far: 1000.0,
    };
    let pooled = |frame: &super::SceneFrame| {
        frame
            .particles
            .iter()
            .any(|batch| materials.contains(&batch.material))
    };
    assert!(pooled(&scene.frame(&camera, 1.5)), "the light pools");
    // Retail makes the pool anew each frame (draw once); without the light there is none.
    scene.remove_object_lights(9);
    assert!(!pooled(&scene.frame(&camera, 1.5)));
}

#[test]
fn the_rain_and_water_systems_load_with_the_scene() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let mut scene = WorldScene::new(Arc::clone(&world), 1);
    scene.set_center((0xCB, 0x4D));
    let _ = scene.wait();
    // `WorldRender::Init`: the rain vapour (splash), ripples and water bubbles systems
    // (`RenderSkeletonEffects` discharges into all three).
    assert!(scene.rain_splash.is_some());
    assert!(scene.rain_ripples.is_some());
    assert!(scene.water_bubbles.is_some());
}
