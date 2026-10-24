//! Headless snapshot of a town block's terrain with its `CLandBlockInfo` entities and scenery,
//! drawn through the retail material path. Runs when `AC2_DAT_DIR` is set and a GPU adapter
//! exists; writes PNGs to `target/ac2-snapshots/`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_gpu::{
    FrameTarget, Gpu, GpuObjectModel, ObjectInstance, ObjectRenderer, TerrainRenderer, TextureKey,
    COLOR_FORMAT,
};
use ac2_math::{Mat4, Vec3};
use ac2_render::object::{load_object_model, RenderSettings};
use ac2_render::terrain::build_terrain_mesh;
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

/// The brightest-fog lighting of one `GameTime` day (7347 s).
fn daylight(world: &Ac2WorldSource) -> SceneLighting {
    world
        .region()
        .and_then(|region| {
            (0..48)
                .filter_map(|step| region.sample_sky_environment(f64::from(step) * 7347.0 / 48.0))
                .map(|sample| SceneLighting::from_sky(&sample))
                .max_by(|a, b| {
                    let bright = |l: &SceneLighting| l.fog_color.iter().sum::<f32>();
                    bright(a).total_cmp(&bright(b))
                })
        })
        .unwrap_or_default()
}

#[test]
fn town_block_renders_its_objects() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
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
    let center = (0xCB_u8, 0x4D_u8);
    let mut blocks = Vec::new();
    let mut models: HashMap<DataId, Option<GpuObjectModel>> = HashMap::new();
    let mut placements = Vec::new();
    let mut pass_counts = HashMap::<bool, usize>::new();
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
            for marker in world.entity_markers(id).unwrap().iter() {
                let visual = marker.visual();
                if !models.contains_key(&visual) {
                    let model = load_object_model(&store, visual, &settings).unwrap();
                    for batch in model
                        .iter()
                        .flat_map(|m| m.parts.iter())
                        .flat_map(|p| p.batches.iter())
                    {
                        for pass in batch.material.passes([1.0; 4], 0.0) {
                            if pass.pixel_program.is_some()
                                && std::env::var_os("AC2_DEBUG_PS").is_some()
                            {
                                eprintln!(
                                    "PS pass {visual:?} prog {:?} stages {:?}",
                                    pass.program,
                                    pass.stages
                                        .iter()
                                        .map(|s| (s.texture, s.tex_coord))
                                        .collect::<Vec<_>>()
                                );
                            }
                            *pass_counts.entry(pass.pixel_program.is_some()).or_insert(0) += 1;
                        }
                    }
                    let model = model.map(|model| objects.upload(&gpu, &model, &mut load));
                    models.insert(visual, model);
                }
                let world_matrix =
                    marker.world_frame().to_mat4() * Mat4::from_scale(marker.scale());
                placements.push((visual, world_matrix));
            }
        }
    }
    let instances: Vec<ObjectInstance<'_>> = placements
        .iter()
        .filter_map(|(visual, world)| {
            Some(ObjectInstance {
                model: models.get(visual)?.as_ref()?,
                world: *world,
            })
        })
        .collect();
    let drawn_models = models.values().filter(|m| m.is_some()).count();
    eprintln!("layer passes by pixel shader: {pass_counts:?}");
    eprintln!(
        "{} placements, {} instances, {} of {} visuals drawable",
        placements.len(),
        instances.len(),
        drawn_models,
        models.len()
    );
    assert!(drawn_models > 10, "town visuals resolve to drawable models");

    let lighting = daylight(&world);
    let block = world.block_size();
    let grid = world
        .load_landblock(block_id(center.0, center.1))
        .unwrap()
        .unwrap()
        .height_grid();
    let ground = grid.iter().copied().fold(f32::MIN, f32::max);
    let town = Vec3::new(
        (f32::from(center.0) + 0.5) * block,
        (f32::from(center.1) + 0.5) * block,
        ground,
    );
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    for (name, eye) in [
        (
            "objects_cb4d_oblique.png",
            town + Vec3::new(-150.0, -150.0, 90.0),
        ),
        (
            "objects_cb4d_street.png",
            town + Vec3::new(-60.0, -20.0, 12.0),
        ),
        (
            "objects_cb4d_close.png",
            town + Vec3::new(-75.0, -12.0, 4.0),
        ),
    ] {
        let camera = Camera {
            eye,
            target: town,
            fov_y: 60_f32.to_radians(),
            near: 0.5,
            far: 4000.0,
        };
        terrain.set_frame(&gpu.queue, &camera, &lighting, aspect);
        objects.prepare(&gpu, &camera, &lighting, aspect, 0.0, &instances);
        target.render(&gpu.device, &gpu.queue, lighting.fog_color, |pass| {
            terrain.draw(pass, &blocks);
            objects.draw(pass, &instances);
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
    }
}
