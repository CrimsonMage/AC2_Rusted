//! Headless snapshot of a town block's terrain with its `CLandBlockInfo` entities and scenery,
//! drawn through the retail material path. Runs when `AC2_DAT_DIR` is set and a GPU adapter
//! exists; writes PNGs to `target/ac2-snapshots/`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ac2_core::{CellId, DataId, Random};
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_gpu::{
    DrawGroup, FrameTarget, Gpu, GpuObjectModel, ObjectInstance, ObjectRenderer, ScreenQuad,
    TerrainRenderer, TextureKey, COLOR_FORMAT,
};
use ac2_math::{Mat4, Vec3};
use ac2_render::frill::{frill_normal, select_frills, FrillCandidate, FrillSettings};
use ac2_render::object::{load_object_model, load_water_model, ObjectEnvironment, RenderSettings};
use ac2_render::sky::sky_frame;
use ac2_render::terrain::{
    build_terrain_mesh, far_radius_for_draw_distance, FarTerrain, FIRST_FAR_CHUNK, NEAR_RADIUS,
};
use ac2_render::{load_material_image, load_texture_image, Camera, SceneLighting};
use ac2_world::Ac2WorldSource;

const SIZE: (u32, u32) = (800, 500);

fn write_png(name: &str, rgba: &[u8]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
    let file = std::fs::File::create(dir.join(name)).expect("png file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

fn block_id(x: u8, y: u8) -> CellId {
    CellId::new((u32::from(x) << 24) | (u32::from(y) << 16) | 1)
}

/// The server time and lighting of the brightest fog in one `GameTime` day (7347 s).
fn daylight(world: &Ac2WorldSource) -> (f64, SceneLighting) {
    world
        .region()
        .and_then(|region| {
            (0..48)
                .filter_map(|step| {
                    let time = f64::from(step) * 7347.0 / 48.0;
                    Some((
                        time,
                        SceneLighting::from_sky(&region.sample_sky_environment(time)?),
                    ))
                })
                .max_by(|a, b| {
                    let bright = |l: &SceneLighting| l.fog_color.iter().sum::<f32>();
                    bright(&a.1).total_cmp(&bright(&b.1))
                })
        })
        .unwrap_or_default()
}

/// What one area render produced.
struct AreaStats {
    drawn_models: usize,
    water_blocks: usize,
    means: Vec<f32>,
}

/// Renders the 3 x 3 blocks around `center` (terrain, `CLandBlockInfo` entities and scenery, near
/// water, sky) and the far landscape past them from each `(name, eye offset, target offset)`
/// around the centre block's highest vertex.
fn render_area(center: (u8, u8), views: &[(&str, Vec3, Vec3)]) -> Option<AreaStats> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return None;
    };
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .unwrap();
    let store = world.resource_store().clone();
    let mut load = |key: TextureKey| match key {
        TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
        TextureKey::Mask(did) | TextureKey::Texture(did) => {
            load_texture_image(&store, did).ok().flatten()
        }
    };
    let mut terrain = TerrainRenderer::new(&gpu, COLOR_FORMAT);
    let mut objects = ObjectRenderer::new(&gpu, COLOR_FORMAT);
    // `AC2_HARDWARE_CLASS=1` renders the fixed-function fallback layers for comparison.
    let settings = RenderSettings {
        hardware_class: std::env::var("AC2_HARDWARE_CLASS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2),
        ..RenderSettings::default()
    };
    let mut blocks = Vec::new();
    let mut models: HashMap<DataId, Option<GpuObjectModel>> = HashMap::new();
    let mut placements = Vec::new();
    let mut frills = Vec::new();
    let mut waters = Vec::new();
    for dx in -1..=1_i32 {
        for dy in -1..=1_i32 {
            let (x, y) = (
                (i32::from(center.0) + dx) as u8,
                (i32::from(center.1) + dy) as u8,
            );
            let id = block_id(x, y);
            let Some(landblock) = world.load_landblock(id).unwrap() else {
                continue;
            };
            let mesh = build_terrain_mesh(id, &landblock, world.block_size());
            blocks.push(terrain.upload_block(&gpu, &mesh, world.block_size(), &mut load));
            if let Some(water) = world.block_water(id, &landblock) {
                if let Some(model) =
                    load_water_model(&store, id, &water, world.block_size(), &settings).unwrap()
                {
                    waters.push(objects.upload(&gpu, &model, &mut load));
                }
            }
            for marker in world.entity_markers(id).unwrap().iter() {
                let visual = marker.visual();
                if !models.contains_key(&visual) {
                    let model = load_object_model(&store, visual, &settings)
                        .unwrap()
                        .map(|model| objects.upload(&gpu, &model, &mut load));
                    models.insert(visual, model);
                }
                let world_matrix =
                    marker.world_frame().to_mat4() * Mat4::from_scale(marker.scale());
                if marker.frill() {
                    frills.push((visual, world_matrix));
                } else {
                    placements.push((visual, world_matrix));
                }
            }
        }
    }
    // The far mesh at the default draw distance, its diagonals from a fresh `Random` stream.
    let far_layout = FarTerrain::new(
        NEAR_RADIUS,
        far_radius_for_draw_distance(2),
        16,
        world.block_size(),
        &mut Random::default(),
    );
    let far_desc = world.far_terrain();
    let center_block = (i32::from(center.0), i32::from(center.1));
    let far_vertices = far_layout.vertices(&world, center_block, &far_desc);
    let far = terrain.upload_far(
        &gpu,
        &far_layout,
        &far_vertices,
        far_desc.texture,
        &mut load,
    );
    let far_water = far_layout
        .water_model(&store, &far_desc, &far_vertices, &settings)
        .unwrap()
        .map(|model| objects.upload(&gpu, &model, &mut load));
    // `AC2_SKY_TIME` (server seconds) overrides the brightest daylight.
    let (server_time, lighting) = match std::env::var("AC2_SKY_TIME")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        Some(time) => (
            time,
            world
                .region()
                .and_then(|region| region.sample_sky_environment(time))
                .map(|sample| SceneLighting::from_sky(&sample))
                .unwrap_or_default(),
        ),
        None => daylight(&world),
    };
    let sky_objects = world
        .region()
        .and_then(|region| region.sky_objects(server_time))
        .unwrap_or_default();
    for object in &sky_objects {
        if let Some(gfx) = object.gfx {
            if !models.contains_key(&gfx) {
                let model = load_object_model(&store, gfx, &settings)
                    .unwrap()
                    .map(|model| objects.upload(&gpu, &model, &mut load));
                models.insert(gfx, model);
            }
        }
    }
    let instances: Vec<ObjectInstance<'_>> = placements
        .iter()
        .filter_map(|(visual, world)| {
            Some(ObjectInstance::new(models.get(visual)?.as_ref()?, *world))
        })
        .chain(
            waters
                .iter()
                .chain(&far_water)
                .map(|model| ObjectInstance::new(model, Mat4::IDENTITY)),
        )
        .collect();
    let drawn_models = models.values().filter(|m| m.is_some()).count();
    eprintln!(
        "{:02X}{:02X}: {} placements, {} of {} visuals drawable, {} water blocks",
        center.0,
        center.1,
        placements.len(),
        drawn_models,
        models.len(),
        waters.len()
    );

    let block = world.block_size();
    let grid = world
        .load_landblock(block_id(center.0, center.1))
        .unwrap()
        .unwrap()
        .height_grid();
    let ground = grid.iter().copied().fold(f32::MIN, f32::max);
    let focus = Vec3::new(
        (f32::from(center.0) + 0.5) * block,
        (f32::from(center.1) + 0.5) * block,
        ground,
    );
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let quad = ScreenQuad::new(&gpu, COLOR_FORMAT);
    let mut means = Vec::new();
    for (name, offset, look) in views {
        let camera = Camera {
            eye: focus + *offset,
            target: focus + *look,
            fov_y: 60_f32.to_radians(),
            near: 0.5,
            far: 4000.0,
        };
        let sky = sky_frame(
            &sky_objects,
            camera.eye,
            false,
            lighting.fog_color,
            lighting.fog_sky,
        );
        let sky_instances = |placements: &[ac2_render::sky::SkyPlacement]| {
            placements
                .iter()
                .filter_map(|p| {
                    Some(ObjectInstance {
                        tint: p.tint,
                        ..ObjectInstance::new(models.get(&p.gfx)?.as_ref()?, p.world)
                    })
                })
                .collect::<Vec<_>>()
        };
        let (before, after) = (sky_instances(&sky.before), sky_instances(&sky.after));
        // `FrillRender::RefreshFrillMesh` around the camera.
        let candidates: Vec<_> = frills
            .iter()
            .filter_map(|(visual, world)| {
                let (indices, vertices) = models.get(visual)?.as_ref()?.first_list();
                Some(FrillCandidate {
                    world: *world,
                    indices,
                    vertices,
                })
            })
            .collect();
        let drawable: Vec<_> = frills
            .iter()
            .filter(|(visual, _)| matches!(models.get(visual), Some(Some(_))))
            .collect();
        let chosen = select_frills(&candidates, camera.eye, &FrillSettings::default());
        let world_instances: Vec<ObjectInstance<'_>> = instances
            .iter()
            .copied()
            .chain(chosen.iter().map(|&index| {
                let (visual, world) = drawable[index];
                ObjectInstance {
                    frill_normal: Some(frill_normal(world)),
                    ..ObjectInstance::new(models[visual].as_ref().unwrap(), *world)
                }
            }))
            .collect();
        if std::env::var_os("AC2_DEBUG_SKY").is_some() {
            eprintln!(
                "time {server_time} fog_sky {} fog {:?} before {:?} after {:?}",
                lighting.fog_sky,
                lighting.fog_color,
                sky.before
                    .iter()
                    .map(|p| (p.gfx, p.tint))
                    .collect::<Vec<_>>(),
                sky.after
                    .iter()
                    .map(|p| (p.gfx, p.tint))
                    .collect::<Vec<_>>()
            );
        }
        terrain.set_frame(&gpu.queue, &camera, &lighting, aspect);
        objects.prepare(
            &gpu,
            &camera,
            &ObjectEnvironment::outside(&lighting),
            aspect,
            0.0,
            &[
                DrawGroup {
                    instances: &before,
                    ordered: true,
                    environment: None,
                },
                DrawGroup {
                    instances: &world_instances,
                    ordered: false,
                    environment: None,
                },
                DrawGroup {
                    instances: &after,
                    ordered: true,
                    environment: None,
                },
            ],
        );
        if let Some(color) = sky.fog_quad {
            quad.set_color(&gpu.queue, color);
        }
        // `SceneTool::backgroundColor` is `RGBAColor_Null`; the sky draws over black.
        target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| {
            objects.draw(pass, 0, &before);
            if sky.fog_quad.is_some() {
                quad.draw(pass);
            }
            terrain.draw(pass, &blocks);
            terrain.draw_far(pass, &far, FIRST_FAR_CHUNK);
            objects.draw(pass, 1, &world_instances);
            objects.draw(pass, 2, &after);
        });
        let rgba = target.read_rgba(&gpu.device, &gpu.queue);
        write_png(name, &rgba);
        let mean = rgba
            .chunks_exact(4)
            .map(|px| u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2]))
            .sum::<u32>() as f32
            / (rgba.len() / 4 * 3) as f32;
        eprintln!("{name}: mean {mean:.1}");
        assert!((15.0..240.0).contains(&mean), "{name}: lit frame ({mean})");
        means.push(mean);
    }
    Some(AreaStats {
        drawn_models,
        water_blocks: waters.len(),
        means,
    })
}

#[test]
fn town_block_renders_its_objects() {
    let Some(stats) = render_area(
        (0xCB, 0x4D),
        &[
            (
                "objects_cb4d_oblique.png",
                Vec3::new(-150.0, -150.0, 90.0),
                Vec3::ZERO,
            ),
            (
                "objects_cb4d_street.png",
                Vec3::new(-60.0, -20.0, 12.0),
                Vec3::ZERO,
            ),
            (
                "objects_cb4d_close.png",
                Vec3::new(-75.0, -12.0, 4.0),
                Vec3::ZERO,
            ),
            (
                "objects_cb4d_horizon.png",
                Vec3::new(0.0, 0.0, 60.0),
                Vec3::new(900.0, 300.0, 0.0),
            ),
            (
                "objects_cb4d_sky.png",
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(60.0, 60.0, 50.0),
            ),
        ],
    ) else {
        return;
    };
    assert!(
        stats.drawn_models > 10,
        "town visuals resolve to drawable models"
    );
    assert_eq!(stats.means.len(), 5);
}

#[test]
fn lake_block_renders_its_water() {
    let Some(stats) = render_area(
        (0xC0, 0x4E),
        &[(
            "water_c04e_oblique.png",
            Vec3::new(-200.0, -200.0, 120.0),
            Vec3::ZERO,
        )],
    ) else {
        return;
    };
    assert!(stats.water_blocks > 0, "the lake blocks have near water");
}
