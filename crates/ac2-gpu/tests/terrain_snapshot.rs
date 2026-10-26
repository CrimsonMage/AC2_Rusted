//! Headless near-landscape snapshots against real AC2 data. Runs when `AC2_DAT_DIR` is set and a
//! GPU adapter exists; writes PNGs to `target/ac2-snapshots/` for inspection.

use std::path::PathBuf;
use std::sync::Arc;

use ac2_core::CellId;
use ac2_db::ResourceStore;
use ac2_gpu::{FrameTarget, Gpu, TerrainRenderer, TextureKey, HDR_FORMAT};
use ac2_math::Vec3;
use ac2_render::terrain::build_terrain_mesh;
use ac2_render::{load_material_image, load_texture_image, Camera, SceneLighting};
use ac2_world::Ac2WorldSource;

const SIZE: (u32, u32) = (640, 400);

fn snapshot_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
    dir
}

fn write_png(name: &str, rgba: &[u8]) {
    let file = std::fs::File::create(snapshot_dir().join(name)).expect("png file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

/// Renders the blocks within `radius` of `center` and returns the RGBA frame.
fn render(
    world: &Ac2WorldSource,
    gpu: &Gpu,
    center: (u8, u8),
    radius: i32,
    camera: Camera,
) -> Vec<u8> {
    let store = world.resource_store().clone();
    let mut renderer = TerrainRenderer::new(gpu, HDR_FORMAT);
    let mut load = |key: TextureKey| match key {
        TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
        TextureKey::Mask(did) | TextureKey::Texture(did) => {
            load_texture_image(&store, did).ok().flatten()
        }
    };
    let mut blocks = Vec::new();
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let (x, y) = (i32::from(center.0) + dx, i32::from(center.1) + dy);
            let id = CellId::new(((x as u32) << 24) | ((y as u32) << 16) | 1);
            let Some(landblock) = world.load_landblock(id).unwrap() else {
                continue;
            };
            let mesh = build_terrain_mesh(id, &landblock, world.block_size());
            blocks.push(renderer.upload_block(gpu, &mesh, world.block_size(), &mut load));
        }
    }
    // Daytime: the brightest fog of one `GameTime` day (7347 s); the directional light's pitch
    // sweeps over the zenith by night as well.
    let lighting = world
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
        .unwrap_or_default();
    eprintln!("lighting {lighting:?}");
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    renderer.set_frame(
        &gpu.queue,
        &camera,
        &lighting,
        SIZE.0 as f32 / SIZE.1 as f32,
    );
    target.render_scene(gpu, lighting.fog_color, |pass| renderer.draw(pass, &blocks));
    target.read_rgba(&gpu.device, &gpu.queue)
}

/// Distinct colours and mean brightness of a frame.
fn stats(rgba: &[u8]) -> (usize, f32) {
    let mut colors = std::collections::HashSet::new();
    let mut sum = 0_u64;
    for px in rgba.chunks_exact(4) {
        colors.insert([px[0] >> 2, px[1] >> 2, px[2] >> 2]);
        sum += u64::from(px[0]) + u64::from(px[1]) + u64::from(px[2]);
    }
    (colors.len(), sum as f32 / (rgba.len() / 4 * 3) as f32)
}

#[test]
fn near_landscape_renders_blended_terrain() {
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
    let block = world.block_size();
    let middle = |x: u8, y: u8| {
        Vec3::new(
            (f32::from(x) + 0.5) * block,
            (f32::from(y) + 0.5) * block,
            0.0,
        )
    };
    let height_at = |x: u8, y: u8| {
        let id = CellId::new((u32::from(x) << 24) | (u32::from(y) << 16) | 1);
        let grid = world.load_landblock(id).unwrap().unwrap().height_grid();
        grid.iter().copied().fold(f32::MIN, f32::max)
    };

    let upland = middle(0x7F, 0x7F) + Vec3::Z * height_at(0x7F, 0x7F);
    let oblique = render(
        &world,
        &gpu,
        (0x7F, 0x7F),
        2,
        Camera {
            eye: upland + Vec3::new(-260.0, -260.0, 160.0),
            target: upland,
            fov_y: 60_f32.to_radians(),
            near: 1.0,
            far: 4000.0,
        },
    );
    write_png("terrain_7f7f_oblique.png", &oblique);

    let town = middle(0xCB, 0x4D) + Vec3::Z * height_at(0xCB, 0x4D);
    let top = render(
        &world,
        &gpu,
        (0xCB, 0x4D),
        1,
        Camera {
            eye: town + Vec3::new(0.0, -1.0, 420.0),
            target: town,
            fov_y: 60_f32.to_radians(),
            near: 1.0,
            far: 4000.0,
        },
    );
    write_png("terrain_cb4d_top.png", &top);
    for (name, frame) in [("oblique", &oblique), ("top", &top)] {
        let (colors, mean) = stats(frame);
        eprintln!("{name}: {colors} colours, mean {mean:.1}");
        assert!((60.0..200.0).contains(&mean), "{name}: lit band ({mean})");
        // Blown highlights: pixels with every channel at 255.
        let blown = frame
            .chunks_exact(4)
            .filter(|px| px[..3] == [255; 3])
            .count();
        assert!(
            blown * 100 < frame.len() / 4,
            "{name}: {blown} blown pixels"
        );
    }

    for (name, frame) in [("oblique", &oblique), ("top", &top)] {
        let (colors, mean) = stats(frame);
        eprintln!("{name}: {colors} colours, mean {mean:.1}");
        assert!(
            colors > 200,
            "{name}: textured terrain, not flat colour ({colors})"
        );
        assert!(
            (20.0..235.0).contains(&mean),
            "{name}: lit, not black or white ({mean})"
        );
    }
}
