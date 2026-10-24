//! Offline AC2 world viewer: draws the landscape straight from the DATs to test client crates
//! without a server. It never connects anywhere.
//!
//! ```text
//! world-viewer [--dat-dir DIR] [--block 7F7F] [--radius 2] [--snapshot out.png]
//! ```
//! Window controls: drag with the left button to look, WASD to move, Space/C up and down, Shift
//! fast, `[` and `]` step the time of day.

mod fly;
mod interiors;
mod scene;
mod window;

use std::path::PathBuf;
use std::sync::Arc;

use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_math::Vec3;
use ac2_world::Ac2WorldSource;

use fly::FlyCamera;
use scene::Scene;

struct Args {
    dat_dir: PathBuf,
    block: (u8, u8),
    radius: i32,
    snapshot: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        dat_dir: std::env::var_os("AC2_DAT_DIR")
            .map(PathBuf::from)
            .unwrap_or_default(),
        block: (0x7F, 0x7F),
        radius: 2,
        snapshot: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--dat-dir" => args.dat_dir = value()?.into(),
            "--block" => {
                let raw = u16::from_str_radix(value()?.trim_start_matches("0x"), 16)
                    .map_err(|e| format!("--block: {e}"))?;
                args.block = ((raw >> 8) as u8, raw as u8);
            }
            "--radius" => args.radius = value()?.parse().map_err(|e| format!("--radius: {e}"))?,
            "--snapshot" => args.snapshot = Some(value()?.into()),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if args.dat_dir.as_os_str().is_empty() {
        return Err("set AC2_DAT_DIR or pass --dat-dir".into());
    }
    Ok(args)
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("world-viewer: {message}");
            std::process::exit(2);
        }
    };
    let dataset = DatDataset::open(DatOpenOptions::new(&args.dat_dir)).expect("AC2 DATs mount");
    let world = Ac2WorldSource::open(Arc::new(ResourceStore::new(dataset)))
        .expect("AC2 landscape vocabulary decodes")
        .expect("the DAT directory holds an AC2 dataset");
    match &args.snapshot {
        Some(path) => snapshot(world, &args, path),
        None => window::run(world, args.block, args.radius),
    }
}

/// The opening view of a block: from its south-west, above its highest vertex.
fn start_camera(scene: &Scene, block: (u8, u8)) -> FlyCamera {
    let size = scene.world.block_size();
    let top = scene.block_top(block);
    let centre = Vec3::new(
        (f32::from(block.0) + 0.5) * size,
        (f32::from(block.1) + 0.5) * size,
        top,
    );
    FlyCamera::looking_at(centre + Vec3::new(-260.0, -260.0, 160.0), centre)
}

fn snapshot(world: Arc<Ac2WorldSource>, args: &Args, path: &PathBuf) {
    let (width, height) = (1280, 720);
    let gpu = Gpu::headless().expect("GPU adapter");
    let mut scene = Scene::new(world, &gpu, COLOR_FORMAT, args.radius);
    scene.set_daytime();
    scene.update_residency(&gpu, args.block);
    let camera = start_camera(&scene, args.block).camera();
    scene.prepare(&gpu, &camera, width as f32 / height as f32, 0.0);
    let target = FrameTarget::new(&gpu.device, width, height);
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| scene.draw(pass));
    let rgba = target.read_rgba(&gpu.device, &gpu.queue);
    let file = std::fs::File::create(path).expect("snapshot file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&rgba))
        .expect("snapshot PNG writes");
    println!("wrote {}", path.display());
}
