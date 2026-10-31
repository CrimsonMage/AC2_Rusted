//! Offline AC2 world viewer: draws the landscape straight from the DATs to test client crates
//! without a server. It never connects anywhere.
//!
//! ```text
//! world-viewer [--dat-dir DIR] [--block 7F7F] [--radius 2] [--fly]
//! world-viewer ... --snapshot out.png [--model N] [--walk SECONDS] [--speed N] [--warm-up SECONDS] [--at X,Y] [--fly]
//! world-viewer ... --snapshot out.png --map-view   (the walk camera's map view, `CameraManager::SetMapView`)
//! world-viewer ... --bench N [--gpu-times] [--snapshot out.png] [--model N] [--walk SECONDS] [--at X,Y] [--fly]
//! world-viewer ... --snapshot out.png --crowd N   (or --bench N --crowd N)
//! world-viewer ... [--model-detail 0|1|2] [--scenery-distance 0|1|2] [--object-shadows 0|1|2|3]
//! [--light-coronas 0|1] [--static-shadows 0|1] [--cloud-shadows 0|1] [--water-reflections 0|1|2]
//! [--shore-effects 0|1|2] [--flash R,G,B,A] [--grass-copies N] [--size WIDTHxHEIGHT]
//! world-viewer ... --snapshot out.png --fly --look YAW,PITCH [--time SECONDS] [--height Z]
//! world-viewer ... [--hud-style modern|classic]
//! world-viewer ... --snapshot out.png --hud [--hud-panel NAME ...] [--hud-window NAME ...] [--hud-pointer X,Y]
//! tools/world-viewer/snapshot_all.sh OUT_DIR   (one PNG per toolbar panel and per window)
//! ```
//! `--hud-window NAME` (repeatable, implies `--hud`; see `windows.rs`) opens a window that
//! normally appears only during play, with sample content: confirm, message, wait, resurrect,
//! loot, stack, trade, shop, catalog, consignment, new-sale, ingredient, craft, book, help,
//! guide, tracker, pets, fellowship, fellow-mini, chat-settings, edit-layout, targeting. Each is
//! raised as the game raises it (the notice the server would send, fed to the HUD's frameworks;
//! the double's chest and craft registry; a drag or a press on the real UI). A snapshot's HUD
//! is the modern style unless `--hud-style classic` says otherwise. Help pages need
//! `AC2_CONTENT_DIR` or a `Content` folder beside the DATs.
//! `--hud-style` builds the HUD in that style (default modern; classic is the retail DAT HUD); in the window
//! F7 switches between the two and rebuilds the HUD. `--hud` logs the snapshot's HUD on to the
//! in-process server double (the window's session) so its vitals, shortcuts and inventory are
//! filled, and leaves out the control panel; `--hud-panel NAME` (Inventory, Skill, Quest, Social,
//! Examination, Navigation, Option; repeatable, implies `--hud`) opens that toolbar panel, and
//! `--hud-pointer X,Y` puts the pointer there (window pixels; a hover look, implies `--hud`).
//! `--fly --at X,Y` starts the fly camera 12 m over that point of the block, looking north.
//! `--look` turns the `--fly` snapshot camera to YAW degrees clockwise from north and PITCH
//! degrees above the horizon (the sky: `--look 0,60`). `--height Z` puts the `--fly` camera at
//! that height in the block instead (inside a dungeon's cells).
//! `--model-detail` and `--scenery-distance` set the headless scene's `ModelDetail` and
//! `SceneryDrawDistance` (object degrades and distances); both default to the preference default,
//! medium. `--object-shadows` sets its `DynamicObjectShadows` level (default 2, medium; 0 none).
//! `--light-coronas 0` turns off the sky's lens flares (`LightCoronas`; default on).
//! `--static-shadows` and `--cloud-shadows` set `LandscapeStaticObjectShadows` (default on) and
//! `LandscapeCloudShadows` (default off; drawn only with the static shadows).
//! `--water-reflections` sets `WaterReflections` (default 2: reflected and rippled), and
//! `--shore-effects` `LandscapeShoreEffects` (default 2: breaking water on the shores).
//! `--flash` draws a camera effect's screen flash of that colour over the snapshot (0..1 each, as
//! `CameraFxManager`'s sum: `--flash 0,0,0,1` is a full fade to black).
//! `--size` sets the headless frame's size (default 1280x720). `--grass-copies N` draws each near grass frill N times (1: retail's frills only; default 4,
//! draw distance 2's).
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
//! V first person, Home resets the camera; the numpad drives the camera as retail's key map
//! does (8/2 tilt, 4/6 turn, 5 reset, +/- zoom, 0 shoulder, . first person). Fly (Tab): drag with the left button to look, WASD,
//! Space/C up and down, Shift fast. `[` and `]` step the time of day, F1 hides the panel.
//! M opens the egui world map; G opens the client's quest guide (its In-Game Wiki) and T the
//! Tracker's window, both on the HUD as in the client, and a HUD text field takes the typing.
//! The HUD has a session (`session.rs`): the client logged on to the Legions server double in
//! process, so the toolbar's panels work with a starter inventory, worn gear, skills and options.
//! The HUD takes the window's input through the client's key maps (`hud_input.rs`): the toolbar's
//! keys, Enter for the chat, typing in its fields and the wheel over a panel work as in the client.

mod bench;
mod crowd;
mod fly;
mod frame;
mod gui;
mod hud_input;
mod panel;
mod races;
mod session;
mod walk;
mod window;
mod windows;

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
    /// Snapshot the walk camera in its map view (`CameraManager::SetMapView`).
    map_view: bool,
    model: usize,
    walk_seconds: f32,
    speed: u32,
    warm_up: u32,
    /// The HUD's style (`--hud-style`).
    hud_style: ac2_ui::HudStyle,
    /// The snapshot's HUD with the server double's session (`--hud`).
    hud: bool,
    /// The toolbar panels the snapshot opens (`--hud-panel`).
    hud_panels: Vec<ac2_ui::ToolbarChild>,
    /// The windows the snapshot opens (`--hud-window`).
    hud_windows: Vec<windows::HudWindow>,
    /// The pointer over the snapshot's HUD (`--hud-pointer`, window pixels).
    hud_pointer: Option<(f32, f32)>,
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
    /// The `--fly` camera's height in the block, over the ground's.
    height: Option<f32>,
    /// The headless scene's `LandscapeStaticObjectShadows` and `LandscapeCloudShadows`.
    static_shadows: bool,
    cloud_shadows: bool,
    /// The headless scene's `WaterReflections` and `LandscapeShoreEffects` levels.
    water_reflections: u8,
    shore_effects: u8,
    /// A screen flash to draw over the snapshot (`--flash`).
    flash: [f32; 4],
    /// Copies of each near grass frill (`FrillSettings::copies`).
    grass_copies: u32,
    /// The headless frame's size in pixels.
    size: (u32, u32),
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
        map_view: false,
        model: 0,
        walk_seconds: 4.0,
        speed: 1,
        warm_up: 3,
        hud_style: ac2_ui::HudStyle::Modern,
        hud: false,
        hud_panels: Vec::new(),
        hud_windows: Vec::new(),
        hud_pointer: None,
        at: None,
        time: None,
        crowd: 0,
        look: None,
        degrade: ac2_render::object::DegradeOptions::default(),
        object_shadows: ac2_render::prefs::SceneOptions::default().object_shadows,
        light_coronas: ac2_render::prefs::SceneOptions::default().light_coronas,
        height: None,
        static_shadows: ac2_render::prefs::SceneOptions::default().static_shadows,
        cloud_shadows: ac2_render::prefs::SceneOptions::default().cloud_shadows,
        water_reflections: ac2_render::prefs::SceneOptions::default().water_reflections,
        shore_effects: ac2_render::prefs::SceneOptions::default().shore_effects,
        flash: [0.0; 4],
        grass_copies: ac2_render::prefs::SceneOptions::default().frill.copies,
        size: (1280, 720),
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
            "--size" => {
                let text = value()?;
                let (w, h) = text.split_once('x').ok_or("--size: WIDTHxHEIGHT")?;
                args.size = (
                    w.parse().map_err(|e| format!("--size: {e}"))?,
                    h.parse().map_err(|e| format!("--size: {e}"))?,
                );
            }
            "--grass-copies" => {
                args.grass_copies = value()?
                    .parse()
                    .map_err(|e| format!("--grass-copies: {e}"))?;
            }
            "--bench" => args.bench = value()?.parse().map_err(|e| format!("--bench: {e}"))?,
            "--gpu-times" => args.gpu_times = true,
            "--fly" => args.fly = true,
            "--map-view" => args.map_view = true,
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
            "--static-shadows" => {
                args.static_shadows = value()?
                    .parse::<u8>()
                    .map_err(|e| format!("--static-shadows: {e}"))?
                    != 0;
            }
            "--cloud-shadows" => {
                args.cloud_shadows = value()?
                    .parse::<u8>()
                    .map_err(|e| format!("--cloud-shadows: {e}"))?
                    != 0;
            }
            "--shore-effects" => {
                args.shore_effects = value()?
                    .parse()
                    .map_err(|e| format!("--shore-effects: {e}"))?;
            }
            "--water-reflections" => {
                args.water_reflections = value()?
                    .parse()
                    .map_err(|e| format!("--water-reflections: {e}"))?;
            }
            "--flash" => {
                let parts: Vec<f32> = value()?
                    .split(',')
                    .map(|v| v.parse().map_err(|e| format!("--flash: {e}")))
                    .collect::<Result<_, _>>()?;
                args.flash = parts
                    .try_into()
                    .map_err(|_| "--flash takes R,G,B,A".to_owned())?;
            }
            "--height" => {
                args.height = Some(value()?.parse().map_err(|e| format!("--height: {e}"))?);
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
            "--hud-style" => {
                let word = value()?;
                args.hud_style = ac2_ui::HudStyle::parse(&word)
                    .ok_or(format!("--hud-style: modern or classic, not {word}"))?;
            }
            "--hud" => args.hud = true,
            "--hud-panel" => {
                let name = value()?;
                let child = ac2_ui::ToolbarChild::from_name(&name)
                    .ok_or(format!("--hud-panel: no panel named {name}"))?;
                args.hud_panels.push(child);
                args.hud = true;
            }
            "--hud-window" => {
                let name = value()?;
                let window = windows::HudWindow::parse(&name).ok_or_else(|| {
                    let names: Vec<&str> = windows::ALL.iter().map(|(n, _)| *n).collect();
                    format!(
                        "--hud-window: no window named {name} ({})",
                        names.join(", ")
                    )
                })?;
                args.hud_windows.push(window);
                args.hud = true;
            }
            "--hud-pointer" => {
                let text = value()?;
                let (x, y) = text.split_once(',').ok_or("--hud-pointer: X,Y")?;
                let parse = |v: &str| {
                    v.trim()
                        .parse::<f32>()
                        .map_err(|e| format!("--hud-pointer: {e}"))
                };
                args.hud_pointer = Some((parse(x)?, parse(y)?));
                args.hud = true;
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
    // The help window's pages: the Content folder beside the DATs, else `AC2_CONTENT_DIR`.
    ac2_client::set_content_dir(ac2_client::resolve_content_dir(
        None,
        &args.dat_dir,
        std::env::var_os(ac2_client::CONTENT_DIR_ENV).as_deref(),
    ));
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
            dat_dir: args.dat_dir.clone(),
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
            hud_style: args.hud_style,
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
    let (width, height) = args.size;
    // Only `--gpu-times` asks the device for timestamp queries.
    let gpu = if args.gpu_times {
        Gpu::headless_timed()
    } else {
        Gpu::headless()
    }
    .expect("GPU adapter");
    // As the client: DXT textures stay BC blocks when the GPU samples them.
    ac2_render::set_gpu_block_compression(gpu.block_compression() && !args.rgba_textures);
    let mut view = ClientView::with_hud_style(
        Arc::clone(&world),
        &gpu,
        COLOR_FORMAT,
        args.radius,
        (width, height),
        args.hud_style,
    );
    view.set_scene_options(ac2_render::prefs::SceneOptions {
        degrade: args.degrade,
        object_shadows: args.object_shadows,
        light_coronas: args.light_coronas,
        static_shadows: args.static_shadows,
        cloud_shadows: args.cloud_shadows,
        water_reflections: args.water_reflections,
        shore_effects: args.shore_effects,
        frill: ac2_render::frill::FrillSettings {
            copies: args.grass_copies,
            ..Default::default()
        },
        ..Default::default()
    });
    let model = models.get(args.model).filter(|_| !args.fly);
    let mut fly = match (args.fly, args.at) {
        (true, Some(at)) => ground_camera(&world, args.block, at),
        _ => start_camera(&world, args.block),
    };
    if let Some(height) = args.height {
        fly.position.z = height;
    }
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
        if args.map_view {
            walk.camera_action(walk::actions::MAP_VIEW, true);
            walk_for(&mut view, &mut walk, &gpu, 2.0, model.visual);
        }
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
    if let Some(camera) = frame.camera.as_mut() {
        camera.flash = args.flash;
    }
    view.update(&gpu, &frame, std::time::Duration::from_millis(4));
    view.settle(&gpu, &frame, width as f32 / height as f32);
    if args.hud {
        hud_session(&mut view, &gpu, &store, &world, &mut frame, args, (x, y));
        view.settle(&gpu, &frame, width as f32 / height as f32);
    }
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
    if !args.hud {
        control_panel(
            &gpu,
            &target,
            models,
            mode,
            args,
            (x, y),
            &walk,
            time_of_day,
        );
    }
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

/// The snapshot's HUD logged on to the server double (`--hud`): the session's frames until the
/// HUD has its panels, then the `--hud-panel` panels opened.
fn hud_session(
    view: &mut ClientView,
    gpu: &Gpu,
    store: &Arc<ResourceStore>,
    world: &Arc<Ac2WorldSource>,
    frame: &mut ac2_client::ClientFrame,
    args: &Args,
    (x, y): (f32, f32),
) {
    const STEP: f64 = 1.0 / 30.0;
    let landing = walk::Landing {
        block: args.block,
        x,
        y,
    };
    let mut session = match session::Session::start(store, world, landing) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("world-viewer: --hud: no session: {error}");
            return;
        }
    };
    for _ in 0..90 {
        session.frame(view, frame, STEP);
        view.update(gpu, frame, std::time::Duration::from_millis(4));
    }
    for &child in &args.hud_panels {
        view.show_panel(child, true);
    }
    for &window in &args.hud_windows {
        let mut driver = windows::Driver {
            session: &mut session,
            view,
            gpu,
            frame,
        };
        if !driver.open(window) {
            eprintln!("world-viewer: --hud-window: {window:?} did not come up");
        }
    }
    if let Some((x, y)) = args.hud_pointer {
        let pointer = [
            (ac2_ui::UI_POINTER_X, x, true),
            (ac2_ui::UI_POINTER_Y, y, true),
        ];
        view.ui_input(&pointer, ac2_ui::Modifiers::default());
    }
    for _ in 0..30 {
        session.frame(view, frame, STEP);
        view.update(gpu, frame, std::time::Duration::from_millis(4));
    }
}

/// The viewer's control panel over the snapshot, as the window draws it.
#[allow(clippy::too_many_arguments)]
fn control_panel(
    gpu: &Gpu,
    target: &FrameTarget,
    models: &[races::PlayerModel],
    mode: panel::Mode,
    args: &Args,
    (x, y): (f32, f32),
    walk: &walk::Walk,
    time_of_day: f64,
) {
    let (width, height) = args.size;
    let mut gui = gui::Gui::headless(gpu, COLOR_FORMAT);
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
    gui.headless_frame(gpu, &target.color_view, [width, height], |ui| {
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
