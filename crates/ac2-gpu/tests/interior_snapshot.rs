//! Headless snapshot of a dungeon's env cells with their baked static lighting, drawn under the
//! landblock's inside environment. Runs when `AC2_DAT_DIR` is set and a GPU adapter exists;
//! writes PNGs to `target/ac2-snapshots/`.

use std::path::PathBuf;
use std::sync::Arc;

use ac2_core::CellId;
use ac2_db::ResourceStore;
use ac2_gpu::{
    DrawGroup, FrameTarget, Gpu, ObjectInstance, ObjectRenderer, TextureKey, COLOR_FORMAT,
};
use ac2_math::{Mat4, Vec3};
use ac2_render::object::{
    load_cell_model, object_point_lights, CellLighting, ObjectEnvironment, RenderSettings,
    SceneLight,
};
use ac2_render::{load_material_image, load_texture_image, Camera};
use ac2_world::Ac2WorldSource;

const SIZE: (u32, u32) = (800, 500);
/// `CLandBlockInfo::m_InteriorFogType` linear.
const FOG_LINEAR: u32 = 0x4000_0002;

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

#[test]
fn dungeon_cells_render_with_baked_light() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
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
    let block = CellId::new(0x005D_0001);
    let interior = world
        .landblock_interior(block)
        .unwrap()
        .expect("dungeon block info");
    let lights = world.light_info(block).unwrap();
    let ambient = interior.ambient.unwrap_or([0.0; 3]);
    let lighting = CellLighting {
        light_info: lights.as_deref(),
        interior_ambient: ambient,
    };
    // `AC2_HARDWARE_CLASS=1` draws the fixed-function layers lit by the baked diffuse.
    let settings = RenderSettings {
        hardware_class: std::env::var("AC2_HARDWARE_CLASS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2),
        ..RenderSettings::default()
    };
    let mut objects = ObjectRenderer::new(&gpu, COLOR_FORMAT);
    let mut models = Vec::new();
    let mut cell_lights = Vec::new();
    let block_origin = Vec3::new(
        ((block.raw() >> 24) & 0xFF) as f32 * world.block_size(),
        ((block.raw() >> 16) & 0xFF) as f32 * world.block_size(),
        0.0,
    );
    let scene_lights: Vec<SceneLight> = lights
        .iter()
        .flat_map(|info| info.lights.iter())
        .map(|light| SceneLight {
            light: light.clone(),
            origin: block_origin + light.origin,
            indoor: light.cell.raw() & 0xFFFF >= 0x100,
        })
        .collect();
    let mut first_frame = None;
    for id in interior.cell_ids(block) {
        let Some(cell) = world.env_cell(id).unwrap() else {
            continue;
        };
        let frame = world.env_cell_world_frame(&cell);
        first_frame.get_or_insert(frame);
        if let Some(model) = load_cell_model(&store, &cell, frame, lighting, &settings).unwrap() {
            // `CEnvCell::DrawCellMesh`: lights cached at the cell's position for 1.5 x its
            // bounding radius.
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            for v in model.parts[0].vertices.iter() {
                lo = lo.min(Vec3::from_array(v.position));
                hi = hi.max(Vec3::from_array(v.position));
            }
            let radius = (hi - lo).length() * 0.5 * 1.5;
            cell_lights.push(object_point_lights(
                &scene_lights,
                frame.origin,
                radius,
                true,
            ));
            models.push(objects.upload(&gpu, &model, &mut load));
        }
    }
    eprintln!(
        "{} cells, {} drawable, ambient {ambient:?}, {} static lights, fog {:?}",
        interior.cell_count,
        models.len(),
        lights.as_ref().map_or(0, |l| l.lights.len()),
        interior.fog_type
    );
    assert!(
        !models.is_empty(),
        "dungeon cells resolve to drawable models"
    );
    let instances: Vec<_> = models
        .iter()
        .zip(&cell_lights)
        .map(|(model, lights)| ObjectInstance {
            point_lights: lights,
            ..ObjectInstance::new(model, Mat4::IDENTITY)
        })
        .collect();
    let fog = (interior.fog_type == Some(FOG_LINEAR)).then(|| {
        (
            interior.fog_color.unwrap_or([0.0; 3]),
            (
                interior.fog_near.unwrap_or(0.0),
                interior.fog_far.unwrap_or(1000.0),
            ),
        )
    });
    let environment = ObjectEnvironment::inside(ambient, fog);
    let start = first_frame.unwrap().origin + Vec3::new(0.0, 0.0, 6.0);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    for (name, look) in [
        ("dungeon_005d_north.png", Vec3::new(0.0, 30.0, -1.0)),
        ("dungeon_005d_east.png", Vec3::new(30.0, 0.0, -1.0)),
    ] {
        let camera = Camera {
            eye: start,
            target: start + look,
            fov_y: 70_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        };
        objects.prepare(
            &gpu,
            &camera,
            &environment,
            aspect,
            0.0,
            &[DrawGroup {
                instances: &instances,
                ordered: false,
                environment: None,
            }],
        );
        target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| {
            objects.draw(pass, 0, &instances)
        });
        let rgba = target.read_rgba(&gpu.device, &gpu.queue);
        write_png(name, &rgba);
        let mean = rgba
            .chunks_exact(4)
            .map(|px| u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2]))
            .sum::<u32>() as f32
            / (rgba.len() / 4 * 3) as f32;
        eprintln!("{name}: mean {mean:.1}");
        assert!(mean > 5.0, "{name}: the cell is visible ({mean})");
    }
}
