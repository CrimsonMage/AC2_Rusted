//! Offline AC2 world viewer: draws the landscape straight from the DATs to test client crates
//! without a server. It never connects anywhere.
//!
//! ```text
//! world-viewer [--dat-dir DIR] [--block 7F7F] [--radius 2] [--fly]
//! world-viewer ... --snapshot out.png [--model N] [--walk SECONDS] [--speed N] [--warm-up SECONDS] [--at X,Y] [--fly]
//! world-viewer ... --bench N [--gpu-times] [--snapshot out.png] [--model N] [--walk SECONDS] [--at X,Y] [--fly]
//! world-viewer ... --snapshot out.png --crowd N   (or --bench N --crowd N)
//! world-viewer ... [--model-detail 0|1|2] [--scenery-distance 0|1|2] [--object-shadows 0|1|2|3]
//! [--light-coronas 0|1]
//! world-viewer ... --snapshot out.png --fly --look YAW,PITCH [--time SECONDS]
//! ```
//! `--fly --at X,Y` starts the fly camera 12 m over that point of the block, looking north.
//! `--look` turns the `--fly` snapshot camera to YAW degrees clockwise from north and PITCH
//! degrees above the horizon (the sky: `--look 0,60`).
//! `--model-detail` and `--scenery-distance` set the headless scene's `ModelDetail` and
//! `SceneryDrawDistance` (object degrades and distances); both default to the preference default,
//! medium. `--object-shadows` sets its `DynamicObjectShadows` level (default 2, medium; 0 none).
//! `--light-coronas 0` turns off the sky's lens flares (`LightCoronas`; default on).
//! `--snapshot` renders one headless frame: the player model N of the race table walks forward
//! for SECONDS (default 4) and is seen from the player camera (`--fly`: the terrain alone).
//! `--bench N` runs the same headless scene (no snapshot file needed) and then times N more
//! frames of update, prepare and render, each waiting for the GPU, and prints one `bench` line
//! (frame p50, p95 and max, the per-step p50s, peak RSS, the typed cache's bytes and entries, and
//! the texture bytes uploaded). `--rgba-textures` decodes DXT textures to RGBA8 even where the
//! GPU samples BC blocks, to compare the two. `--crowd N` adds N idle race and sex models on a
//! grid around the player to both (a busy scene). `--gpu-times` adds each render pass's GPU p50
//! and the frame's to the bench line (timestamp queries; off by default because asking the device
//! for them raises the NVIDIA driver's resident memory by about 130 MB, which would skew the peak
//! RSS). The line always ends with the allocator's VRAM bytes (0 on Metal and GL).
//! The player walks the world with collision (`ac2-physics`) under the player camera
//! (`ac2-smartbox`); an egui panel swaps the race and sex, teleports and speeds the player up.
//! Walk: W/S, A/D turn, Q/E strafe, Space jump, hold the right button to look, wheel to zoom,
//! V first person, Home resets the camera. Fly (Tab): drag with the left button to look, WASD,
//! Space/C up and down, Shift fast. `[` and `]` step the time of day, F1 hides the panel.

mod bench;
mod crowd;
mod fly;
mod frame;
mod gui;
mod panel;
mod races;
mod walk;
mod window;

use std::path::PathBuf;
use std::sync::Arc;

use ac2_client::view::ClientView;
use ac2_core::CellId;
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_math::Vec3;
use ac2_render::scene::DAY_SECONDS;
use ac2_render::SceneLighting;
use ac2_world::Ac2WorldSource;

use fly::FlyCamera;

struct Args {
    dat_dir: PathBuf,
    block: (u8, u8),
    radius: i32,
    snapshot: Option<PathBuf>,
    /// Timed frames after the warm-up (0: no bench).
    bench: u32,
    /// Time each render pass on the GPU during the bench.
    gpu_times: bool,
    /// Decode DXT textures to RGBA8 even on a BC-capable GPU (compares the two upload paths).
    rgba_textures: bool,
    fly: bool,
    model: usize,
    walk_seconds: f32,
    speed: u32,
    warm_up: u32,
    /// The start position within the block (default: its centre).
    at: Option<(f32, f32)>,
    /// Server seconds to show (default: the day's brightest daylight); other days bring their
    /// own weather.
    time: Option<f64>,
    /// Idle models placed around the player in the headless scene (0: none).
    crowd: u32,
    /// The fly camera's yaw (clockwise from north) and pitch (above the horizon), in degrees.
    look: Option<(f32, f32)>,
    /// The headless scene's degrade settings (`ModelDetail`, `SceneryDrawDistance`).
    degrade: ac2_render::object::DegradeOptions,
    /// The headless scene's `DynamicObjectShadows` level (sun shadows of bodies and objects).
    object_shadows: ac2_render::prefs::ObjectShadows,
    /// The headless scene's `LightCoronas` (the sky's lens flares).
    light_coronas: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        dat_dir: std::env::var_os("AC2_DAT_DIR")
            .map(PathBuf::from)
            .unwrap_or_default(),
        block: (0x7F, 0x7F),
        radius: 2,
        snapshot: None,
        bench: 0,
        gpu_times: false,
        rgba_textures: false,
        fly: false,
        model: 0,
        walk_seconds: 4.0,
        speed: 1,
        warm_up: 3,
        at: None,
        time: None,
        crowd: 0,
        look: None,
        degrade: ac2_render::object::DegradeOptions::default(),
        object_shadows: ac2_render::prefs::SceneOptions::default().object_shadows,
        light_coronas: ac2_render::prefs::SceneOptions::default().light_coronas,
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
            "--bench" => args.bench = value()?.parse().map_err(|e| format!("--bench: {e}"))?,
            "--gpu-times" => args.gpu_times = true,
            "--fly" => args.fly = true,
            "--rgba-textures" => args.rgba_textures = true,
            "--model" => args.model = value()?.parse().map_err(|e| format!("--model: {e}"))?,
            "--walk" => {
                args.walk_seconds = value()?.parse().map_err(|e| format!("--walk: {e}"))?;
            }
            "--speed" => args.speed = value()?.parse().map_err(|e| format!("--speed: {e}"))?,
            "--at" => {
                let text = value()?;
                let (x, y) = text.split_once(',').ok_or("--at: x,y")?;
                args.at = Some((
                    x.parse().map_err(|e| format!("--at: {e}"))?,
                    y.parse().map_err(|e| format!("--at: {e}"))?,
                ));
            }
            "--time" => args.time = Some(value()?.parse().map_err(|e| format!("--time: {e}"))?),
            "--look" => {
                let text = value()?;
                let (yaw, pitch) = text.split_once(',').ok_or("--look: yaw,pitch")?;
                args.look = Some((
                    yaw.parse().map_err(|e| format!("--look: {e}"))?,
                    pitch.parse().map_err(|e| format!("--look: {e}"))?,
                ));
            }
            "--crowd" => args.crowd = value()?.parse().map_err(|e| format!("--crowd: {e}"))?,
            "--model-detail" => {
                args.degrade.model_detail = value()?
                    .parse()
                    .map_err(|e| format!("--model-detail: {e}"))?;
            }
            "--scenery-distance" => {
                args.degrade.scenery_draw_distance = value()?
                    .parse()
                    .map_err(|e| format!("--scenery-distance: {e}"))?;
            }
            "--object-shadows" => {
                let level: i32 = value()?
                    .parse()
                    .map_err(|e| format!("--object-shadows: {e}"))?;
                args.object_shadows = ac2_render::prefs::ObjectShadows::from_preference(level);
            }
            "--light-coronas" => {
                let on: u8 = value()?
                    .parse()
                    .map_err(|e| format!("--light-coronas: {e}"))?;
                args.light_coronas = on != 0;
            }
            "--warm-up" => {
                args.warm_up = value()?.parse().map_err(|e| format!("--warm-up: {e}"))?;
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if args.dat_dir.as_os_str().is_empty() {
        return Err("set AC2_DAT_DIR or pass --dat-dir".into());
    }
    if args.gpu_times && args.bench == 0 {
        return Err("--gpu-times needs --bench".into());
    }
    if args.crowd > 0 && args.snapshot.is_none() && args.bench == 0 {
        return Err("--crowd needs --snapshot or --bench".into());
    }
    Ok(args)
}

fn main() {
    ac2_db::install_frame_workers();
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("world-viewer: {message}");
            std::process::exit(2);
        }
    };
    let dataset = DatDataset::open(DatOpenOptions::new(&args.dat_dir)).expect("AC2 DATs mount");
    let store = Arc::new(ResourceStore::new(dataset));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("AC2 landscape vocabulary decodes")
        .expect("the DAT directory holds an AC2 dataset");
    let (models, models_error) = match races::load(&store, &args.dat_dir) {
        Ok(models) => (models, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    if args.snapshot.is_some() || args.bench > 0 {
        snapshot(world, store, &models, &args);
    } else {
        let (x, y) = args.at.unwrap_or_else(|| {
            let half = world.block_size() / 2.0;
            (half, half)
        });
        let fly = start_camera(&world, args.block);
        let time_of_day = args.time.unwrap_or_else(|| daytime(&world));
        window::run(window::Setup {
            world,
            store,
            models,
            models_error,
            landing: walk::Landing {
                block: args.block,
                x,
                y,
            },
            radius: args.radius,
            fly_first: args.fly,
            fly,
            time_of_day,
        });
    }
}

/// The opening view of a block: from its south-west, above its highest vertex.
fn start_camera(world: &Ac2WorldSource, block: (u8, u8)) -> FlyCamera {
    let size = world.block_size();
    let id = CellId::new((u32::from(block.0) << 24) | (u32::from(block.1) << 16) | 1);
    let top = world
        .load_landblock(id)
        .ok()
        .flatten()
        .map_or(0.0, |landblock| {
            landblock.height_grid().into_iter().fold(0.0, f32::max)
        });
    let centre = Vec3::new(
        (f32::from(block.0) + 0.5) * size,
        (f32::from(block.1) + 0.5) * size,
        top,
    );
    FlyCamera::looking_at(centre + Vec3::new(-260.0, -260.0, 160.0), centre)
}

/// The fly camera of `--fly --at X,Y`: over that point of the block, `FLY_AT_HEIGHT` metres above
/// its land cell, looking north (turned by `--look`).
fn ground_camera(world: &Ac2WorldSource, block: (u8, u8), (x, y): (f32, f32)) -> FlyCamera {
    const FLY_AT_HEIGHT: f32 = 12.0;
    let size = world.block_size();
    let id = CellId::new((u32::from(block.0) << 24) | (u32::from(block.1) << 16) | 1);
    let cell = size / ac2_world::LAND_BLOCK_CELLS_PER_SIDE as f32;
    let ground = world
        .load_landblock(id)
        .ok()
        .flatten()
        .and_then(|landblock| {
            landblock.average_cell_height((x / cell) as usize, (y / cell) as usize)
        })
        .unwrap_or(0.0);
    let position = Vec3::new(
        f32::from(block.0) * size + x,
        f32::from(block.1) * size + y,
        ground + FLY_AT_HEIGHT,
    );
    FlyCamera::looking_at(position, position + Vec3::Y)
}

/// The brightest daylight of the region's day (`WorldScene::set_daytime`).
fn daytime(world: &Ac2WorldSource) -> f64 {
    let Some(region) = world.region() else {
        return 0.0;
    };
    let brightness = |time: f64| {
        region
            .sample_sky_environment(time)
            .map(|sample| {
                SceneLighting::from_sky(&sample)
                    .fog_color
                    .iter()
                    .sum::<f32>()
            })
            .unwrap_or(0.0)
    };
    (0..48)
        .map(|step| f64::from(step) * DAY_SECONDS / 48.0)
        .max_by(|a, b| brightness(*a).total_cmp(&brightness(*b)))
        .unwrap_or(0.0)
}

/// The headless frame: walks the player, renders it (and writes `--snapshot`'s PNG), and with
/// `--bench` times the frames after it.
fn snapshot(
    world: Arc<Ac2WorldSource>,
    store: Arc<ResourceStore>,
    models: &[races::PlayerModel],
    args: &Args,
) {
    let (width, height) = (1280, 720);
    // Only `--gpu-times` asks the device for timestamp queries.
    let gpu = if args.gpu_times {
        Gpu::headless_timed()
    } else {
        Gpu::headless()
    }
    .expect("GPU adapter");
    // As the client: DXT textures stay BC blocks when the GPU samples them.
    ac2_render::set_gpu_block_compression(gpu.block_compression() && !args.rgba_textures);
    let mut view = ClientView::new(
        Arc::clone(&world),
        &gpu,
        COLOR_FORMAT,
        args.radius,
        (width, height),
    );
    view.set_scene_options(ac2_render::prefs::SceneOptions {
        degrade: args.degrade,
        object_shadows: args.object_shadows,
        light_coronas: args.light_coronas,
        ..Default::default()
    });
    let model = models.get(args.model).filter(|_| !args.fly);
    let mut fly = match (args.fly, args.at) {
        (true, Some(at)) => ground_camera(&world, args.block, at),
        _ => start_camera(&world, args.block),
    };
    if let Some((yaw, pitch)) = args.look {
        fly.yaw = yaw.to_radians();
        fly.pitch = pitch.to_radians();
    }
    let time_of_day = args.time.unwrap_or_else(|| daytime(&world));
    let mut walk = walk::Walk::new(&world, Arc::clone(&store), 0.0);
    let mode = if model.is_some() {
        panel::Mode::Walk
    } else {
        panel::Mode::Fly
    };
    let (x, y) = args.at.unwrap_or_else(|| {
        let half = world.block_size() / 2.0;
        (half, half)
    });
    if let Some(model) = model {
        println!("player: {}", model.label());
        walk.set_model(model.visual);
        walk.teleport(walk::Landing {
            block: args.block,
            x,
            y,
        });
        walk.speed = args.speed;
        walk_for(&mut view, &mut walk, &gpu, args.walk_seconds, model.visual);
        if let Some(message) = &walk.message {
            println!("walk: {message}");
        }
        if let Some(status) = walk.status() {
            println!("walk: {status:?}");
        }
    }
    let mut frame = frame::client_frame(mode, &walk, &fly, model.map(|m| m.visual), time_of_day);
    if args.crowd > 0 {
        let Some(player) = frame.player.as_ref() else {
            eprintln!("world-viewer: --crowd needs the player (drop --fly)");
            std::process::exit(2);
        };
        frame.objects = crowd::objects(models, args.crowd, player, |origin| {
            walk.ground_height(player.cell, origin)
        });
        println!("crowd: {} objects", frame.objects.len());
    }
    view.update(&gpu, &frame, std::time::Duration::from_millis(4));
    view.settle(&gpu, &frame, width as f32 / height as f32);
    // The scene's particle systems run on the wall clock: let them emit before the frame is
    // taken.
    for step in 0..args.warm_up * 20 {
        view.prepare(
            &gpu,
            &frame,
            width as f32 / height as f32,
            f64::from(step) * 0.05,
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    view.prepare(&gpu, &frame, width as f32 / height as f32, 0.0);
    let target = FrameTarget::new(&gpu.device, width, height);
    if args.bench > 0 {
        let report = bench::run(
            &mut view,
            &gpu,
            &store,
            &frame,
            &target,
            width as f32 / height as f32,
            args.bench,
        );
        println!("{}", report.line());
    }
    view.render(&gpu, &target.color_view, &target.depth_view, true);
    // The control panel over it, as the window draws it.
    let mut gui = gui::Gui::headless(&gpu, COLOR_FORMAT);
    let mut panel = panel::Panel::new(
        models.to_vec(),
        mode,
        walk::Landing {
            block: args.block,
            x,
            y,
        },
    );
    panel.model = args.model.min(models.len().saturating_sub(1));
    panel.speed = args.speed;
    let mut day = time_of_day;
    let status = walk.status();
    gui.headless_frame(&gpu, &target.color_view, [width, height], |ui| {
        panel.show(
            ui,
            panel::PanelView {
                status,
                message: walk.message.as_deref(),
                fps: 60.0,
                time_of_day: &mut day,
                particle_count: 0,
            },
        );
    });
    let Some(path) = &args.snapshot else {
        return;
    };
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

/// Runs the player in simulated 60 Hz time: until it is in the world (the cells load on their
/// own threads), then `seconds` of walking forward.
fn walk_for(
    view: &mut ClientView,
    walk: &mut walk::Walk,
    gpu: &Gpu,
    seconds: f32,
    visual: ac2_core::DataId,
) {
    const STEP: f32 = 1.0 / 60.0;
    let started = std::time::Instant::now();
    let mut now = 0.0_f64;
    let mut walked = 0.0;
    let fly = FlyCamera::looking_at(Vec3::ZERO, Vec3::Y);
    while walked < seconds {
        now += f64::from(STEP);
        let input = walk::WalkInput {
            forward: if walk.in_world() { 1.0 } else { 0.0 },
            ..Default::default()
        };
        walk.update(STEP, now, input);
        let frame = frame::client_frame(panel::Mode::Walk, walk, &fly, Some(visual), 0.0);
        if frame.player.is_some() {
            view.update(gpu, &frame, std::time::Duration::from_millis(4));
        }
        if walk.in_world() {
            walked += STEP;
        } else {
            if started.elapsed() > std::time::Duration::from_secs(120) || walk.message.is_some() {
                println!("walk: the player never entered the world");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
