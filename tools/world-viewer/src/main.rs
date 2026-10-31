//! Offline AC2 world viewer: draws the landscape straight from the DATs to test client crates
//! without a server. It never connects anywhere.
//!
//! ```text
//! world-viewer [--dat-dir DIR] [--block 7F7F] [--radius 2] [--fly]
//! world-viewer ... --snapshot out.png [--model N] [--walk SECONDS] [--speed N] [--warm-up SECONDS] [--at X,Y] [--fly]
//! world-viewer ... --bench N [--gpu-times] [--snapshot out.png] [--model N] [--walk SECONDS] [--at X,Y] [--fly]
//! world-viewer ... --snapshot out.png --crowd N   (or --bench N --crowd N)
//! world-viewer ... --ace-weenies DIR [--ac1-dat-dir DIR] [--ac1-heartbeat SECONDS] [--ac1-lineup]
//! world-viewer ... --ac1-weapons DIR [--ac1-weapon N] [--ac1-weapon-placement P] [--ac1-weapon-scale S] [--orbit SECONDS]
//! world-viewer ... --ac1-armor DIR [--ac2-armor TABLE,...]
//! world-viewer ... [--model-detail 0|1|2] [--scenery-distance 0|1|2] [--object-shadows 0|1|2|3]
//! [--light-coronas 0|1] [--static-shadows 0|1] [--cloud-shadows 0|1] [--water-reflections 0|1|2]
//! [--shore-effects 0|1|2] [--flash R,G,B,A] [--grass-copies N] [--size WIDTHxHEIGHT]
//! world-viewer ... --snapshot out.png --fly --look YAW,PITCH [--time SECONDS] [--height Z]
//! ```
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
//! `--ace-weenies DIR` stands every ACE weenie patch file (`*.sql`) in DIR, built from the AC1
//! DATs (`--ac1-dat-dir`, else `AC1_DAT_DIR`), in a ring around where the player first stands,
//! each idling through its motion table's default cycle and its weenie's HeartBeat emote actions
//! (`ac1.rs`); `--ac1-heartbeat` rolls those every SECONDS instead of the weenie's interval, and
//! `--ac1-lineup` stands them in a row 6 m north of the player (or a `--fly` camera) facing it.
//! `--ac1-weapons DIR` puts the ACE weapons of DIR, built from the AC1 DATs, in the player's
//! right hand (`ac1_held.rs`): weapon N first (default 0), in the AC1 placement frame P (default
//! 1, `RightHandCombat`), at its weenie's scale times S (default 0.75, `ac1_held::HELD_SCALE`);
//! PageUp and PageDown step through them, past the session's AC2 sword.
//! `--hud-panel NAME` logs the snapshot's HUD on to the session (as the window's) and opens that
//! toolbar panel (`inventory`, `skill`, `quest`, ...); `--equip N` then equips AC1 weapon N in
//! the primary hand as the paper doll does (the session's server wields it).
//! `--ac1-armor DIR` dresses the player in the ACE armour of DIR (`ac1_worn.rs`): each part its
//! ClothingBase gives the race's AC1 body hangs from the AC2 bone that part is paired with.
//! `--ac2-armor TABLE,...` adds AC2 armour appearance tables (hex DIDs: the Lugian's
//! `av_lgm_malok_breastplate.apr` is 0x2000002D) over it, the Lugian's AC1 body included.
//! `--ac1-motion MOVE` (`ready`, `run`, `back`, `strafe`, `turn`) poses the snapshot's player
//! with that movement's AC1 cycle, retargeted (`ac1_motion.rs`; the window's panel toggle), at
//! `--ac1-motion-time` seconds (default 0.3).
//! `--orbit SECONDS` turns the snapshot's player camera about the player that long after the walk
//! (the camera's rotate-left action), to see the player from the side; `--zoom SECONDS` then
//! zooms the camera in that long. `--no-hud` draws the snapshot's world alone (no HUD, no panel).
//! `--ac2-action MODE[:BEHAVIOR]` puts the snapshot's player in an AC2 mode and plays a behaviour
//! there (`ac2_action.rs`), `--ac2-action-time` seconds before the frame (default 0.5);
//! `--ac2-actions` lists the DAT's mode and behaviour names.
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

mod ac1;
mod ac1_fx;
mod ac1_held;
mod ac1_motion;
mod ac1_worn;
mod ac2_action;
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
    /// ACE weenie patch files to stand in the world, and the AC1 DATs they are built from.
    ace_weenies: Option<PathBuf>,
    ac1_dat_dir: Option<PathBuf>,
    ac1_heartbeat: Option<f64>,
    ac1_lineup: bool,
    /// ACE weapon patch files for the player's right hand, the one in hand first, and the AC1
    /// placement frame it takes there.
    ac1_weapons: Option<PathBuf>,
    /// `--ac1-armor DIR`: ACE armour weenies the player wears.
    ac1_armor: Option<PathBuf>,
    /// `--ac2-armor TABLE,...`: AC2 armour appearance tables the player wears.
    ac2_armor: Vec<u32>,
    ac1_weapon: usize,
    ac1_weapon_placement: Option<u32>,
    /// `--ac1-weapon-scale S`: the held weapons' scale over their weenies' (default 0.75).
    ac1_weapon_scale: Option<f32>,
    /// `--ac1-motion MOVE`: the snapshot's player takes the AC1 motion of that movement
    /// (`ready`, `run`, `back`, `strafe` or `turn`), `--ac1-motion-time` seconds into it.
    ac1_motion: Option<walk::WalkInput>,
    ac1_motion_time: f64,
    /// `--orbit SECONDS`: the player camera turns about the player that long after the walk.
    orbit: f32,
    /// `--zoom SECONDS`: the player camera zooms in that long after the orbit.
    zoom: f32,
    /// `--no-hud`: the snapshot draws the world without the HUD and the panel.
    no_hud: bool,
    /// `--ac2-action MODE[:BEHAVIOR]` and the seconds it plays before the frame.
    ac2_action: Option<String>,
    ac2_action_time: f32,
    /// `--ac2-actions`: list the mode and behaviour names and exit.
    list_ac2_actions: bool,
    /// `--hud-panel NAME`: the snapshot logs on to the HUD's session and opens that toolbar panel.
    hud_panel: Option<ac2_ui::ToolbarChild>,
    /// `--equip N`: with `--hud-panel`, the session equips AC1 weapon N in the primary hand.
    equip: Option<usize>,
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
        height: None,
        static_shadows: ac2_render::prefs::SceneOptions::default().static_shadows,
        cloud_shadows: ac2_render::prefs::SceneOptions::default().cloud_shadows,
        water_reflections: ac2_render::prefs::SceneOptions::default().water_reflections,
        shore_effects: ac2_render::prefs::SceneOptions::default().shore_effects,
        flash: [0.0; 4],
        grass_copies: ac2_render::prefs::SceneOptions::default().frill.copies,
        size: (1280, 720),
        ace_weenies: None,
        ac1_dat_dir: std::env::var_os(ac1_assets::AC1_DAT_DIR_ENV).map(PathBuf::from),
        ac1_heartbeat: None,
        ac1_lineup: false,
        ac1_weapons: None,
        ac1_armor: None,
        ac2_armor: Vec::new(),
        ac1_weapon: 0,
        ac1_weapon_placement: None,
        ac1_weapon_scale: None,
        ac1_motion: None,
        ac1_motion_time: 0.3,
        orbit: 0.0,
        zoom: 0.0,
        no_hud: false,
        ac2_action: None,
        ac2_action_time: 0.5,
        list_ac2_actions: false,
        hud_panel: None,
        equip: None,
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
            "--ace-weenies" => args.ace_weenies = Some(value()?.into()),
            "--ac1-dat-dir" => args.ac1_dat_dir = Some(value()?.into()),
            "--ac1-lineup" => args.ac1_lineup = true,
            "--hud-panel" => {
                let name = value()?;
                args.hud_panel = Some(
                    ac2_ui::ToolbarChild::ALL
                        .into_iter()
                        .find(|child| format!("{child:?}").eq_ignore_ascii_case(&name))
                        .ok_or(format!("--hud-panel: no toolbar panel {name}"))?,
                );
            }
            "--equip" => args.equip = Some(value()?.parse().map_err(|e| format!("--equip: {e}"))?),
            "--orbit" => args.orbit = value()?.parse().map_err(|e| format!("--orbit: {e}"))?,
            "--zoom" => args.zoom = value()?.parse().map_err(|e| format!("--zoom: {e}"))?,
            "--no-hud" => args.no_hud = true,
            "--ac2-action" => args.ac2_action = Some(value()?),
            "--ac2-actions" => args.list_ac2_actions = true,
            "--ac2-action-time" => {
                args.ac2_action_time = value()?
                    .parse()
                    .map_err(|e| format!("--ac2-action-time: {e}"))?;
            }
            "--ac1-weapons" => args.ac1_weapons = Some(value()?.into()),
            "--ac1-armor" => args.ac1_armor = Some(value()?.into()),
            "--ac2-armor" => {
                args.ac2_armor = value()?
                    .split(',')
                    .map(|table| u32::from_str_radix(table.trim().trim_start_matches("0x"), 16))
                    .collect::<Result<_, _>>()
                    .map_err(|e| format!("--ac2-armor: {e}"))?;
            }
            "--ac1-weapon" => {
                args.ac1_weapon = value()?.parse().map_err(|e| format!("--ac1-weapon: {e}"))?;
            }
            "--ac1-motion" => {
                let mut input = walk::WalkInput::default();
                match value()?.as_str() {
                    "ready" => {}
                    "run" => input.forward = 1.0,
                    "back" => input.forward = -1.0,
                    "strafe" => input.strafe = 1.0,
                    "turn" => input.turn = -1.0,
                    other => return Err(format!("--ac1-motion: no movement {other}")),
                }
                args.ac1_motion = Some(input);
            }
            "--ac1-motion-time" => {
                args.ac1_motion_time = value()?
                    .parse()
                    .map_err(|e| format!("--ac1-motion-time: {e}"))?;
            }
            "--ac1-weapon-scale" => {
                args.ac1_weapon_scale = Some(
                    value()?
                        .parse()
                        .map_err(|e| format!("--ac1-weapon-scale: {e}"))?,
                );
            }
            "--ac1-weapon-placement" => {
                args.ac1_weapon_placement = Some(
                    value()?
                        .parse()
                        .map_err(|e| format!("--ac1-weapon-placement: {e}"))?,
                );
            }
            "--ac1-heartbeat" => {
                args.ac1_heartbeat = Some(
                    value()?
                        .parse()
                        .map_err(|e| format!("--ac1-heartbeat: {e}"))?,
                );
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
    let ac1_wanted = args.ace_weenies.is_some()
        || args.ac1_weapons.is_some()
        || args.ac1_armor.is_some()
        || !args.ac2_armor.is_empty();
    if ac1_wanted && args.ac1_dat_dir.is_none() {
        return Err(
            "--ace-weenies, --ac1-weapons, --ac1-armor and --ac2-armor need --ac1-dat-dir or AC1_DAT_DIR".into(),
        );
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
    if args.list_ac2_actions {
        ac2_action::list(&store);
        return;
    }
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("AC2 landscape vocabulary decodes")
        .expect("the DAT directory holds an AC2 dataset");
    let (models, models_error) = match races::load(&store, &args.dat_dir) {
        Ok(models) => (models, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    let wanted = args.ace_weenies.is_some()
        || args.ac1_weapons.is_some()
        || args.ac1_armor.is_some()
        || !args.ac2_armor.is_empty();
    let ac1 = match (&args.ac1_dat_dir, wanted) {
        (Some(dats), true) => match ac1::Ac1Showcase::load(
            dats,
            args.ace_weenies.as_deref(),
            args.ac1_weapons.as_deref(),
            args.ac1_armor.as_deref(),
            &args.ac2_armor,
            &models
                .iter()
                .map(|m| (m.visual, m.race, m.sex))
                .collect::<Vec<_>>(),
            args.ac1_weapon_scale,
            Arc::clone(&store),
        ) {
            Ok(mut showcase) => {
                showcase.heartbeat = args.ac1_heartbeat;
                showcase.lineup = args.ac1_lineup;
                let weapons = &mut showcase.weapons;
                weapons.current = weapons
                    .current
                    .map(|_| args.ac1_weapon.min(weapons.weapons.len() - 1));
                if let Some(placement) = args.ac1_weapon_placement {
                    weapons.placement = placement;
                }
                if let Some(scale) = args.ac1_weapon_scale {
                    weapons.scale = scale;
                }
                Some(showcase)
            }
            Err(error) => {
                eprintln!("world-viewer: {error}");
                std::process::exit(2);
            }
        },
        _ => None,
    };
    if args.snapshot.is_some() || args.bench > 0 {
        snapshot(world, store, &models, ac1, &args);
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
            ac1,
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
    mut ac1: Option<ac1::Ac1Showcase>,
    args: &Args,
) {
    let (width, height) = args.size;
    let ac2_action = args.ac2_action.as_deref().map(|spec| {
        ac2_action::Ac2Action::parse(&store, spec).unwrap_or_else(|e| {
            eprintln!("world-viewer: --ac2-action: {e}");
            std::process::exit(2);
        })
    });
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
    if let Some(showcase) = &ac1 {
        view.set_foreign_models(showcase.models());
    }
    let model = models.get(args.model).filter(|_| !args.fly);
    let mut fly = match (args.fly, args.at) {
        (true, Some(at)) => ground_camera(&world, args.block, at),
        _ => start_camera(&world, args.block),
    };
    // `ground_camera` stands 12 m over the ground; the AC1 ring stands on that ground.
    let fly_ground = fly.position.z - 12.0;
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
        let walked = walk_for(&mut view, &mut walk, &gpu, args.walk_seconds, model.visual);
        let mut now = walked;
        if args.orbit > 0.0 {
            now = hold_camera(&mut walk, walk::actions::ROTATE_LEFT, args.orbit, now);
        }
        if args.zoom > 0.0 {
            now = hold_camera(&mut walk, walk::actions::ZOOM_IN, args.zoom, now);
        }
        if let Some(action) = ac2_action {
            let started = walk.act(action.mode, action.behavior);
            println!(
                "ac2 action: {action:x?} {}",
                if started { "started" } else { "refused" }
            );
            stand(&mut walk, args.ac2_action_time, now);
        }
        if let Some(message) = &walk.message {
            println!("walk: {message}");
        }
        if let Some(status) = walk.status() {
            println!("walk: {status:?}");
        }
    }
    let mut frame = frame::client_frame(mode, &walk, &fly, model.map(|m| m.visual), time_of_day);
    if let Some(panel) = args.hud_panel {
        let weapons = ac1.as_mut().map(|showcase| &mut showcase.weapons);
        hud_session(
            &mut view, &gpu, &store, &world, args, &mut frame, panel, weapons,
        );
    }
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
    if let Some(showcase) = ac1.as_mut() {
        if frame.player.is_none() {
            // `--fly`: the ring stands 12 m ahead of the camera, at the ground under it.
            let size = world.block_size();
            // The line-up stands 6 m north of its anchor itself.
            let reach = if showcase.lineup { 0.0 } else { 12.0 };
            let ahead = fly.position + fly.forward() * reach;
            let (bx, by) = ((ahead.x / size).floor(), (ahead.y / size).floor());
            let cell = ((bx as u32) << 24) | ((by as u32) << 16) | 1;
            let origin = [ahead.x - bx * size, ahead.y - by * size, fly_ground];
            showcase.set_anchor(cell, origin);
        }
        let creatures = showcase.objects(frame.player.as_ref(), 0.0, |cell, origin| {
            walk.ground_height(cell, origin)
        });
        frame.objects.extend(creatures);
        showcase.weapons.hold(&mut frame);
        if let (Some(input), Some(model), Some(player)) =
            (args.ac1_motion, model, frame.player.as_mut())
        {
            // The cycle starts at 0; the snapshot shows it `--ac1-motion-time` seconds in.
            showcase
                .motion
                .apply(&mut player.clone(), model.race, model.sex, &input, 0.0);
            let applied =
                showcase
                    .motion
                    .apply(player, model.race, model.sex, &input, args.ac1_motion_time);
            println!(
                "ac1 motion: {}",
                if applied { "retargeted" } else { "no AC1 body" }
            );
        }
        // After the motion: the dressed body is drawn under a visual of its own.
        if let Some(model) = model {
            showcase.wear(&mut frame, model.race, model.sex);
        }
    }
    if let Some(camera) = frame.camera.as_mut() {
        camera.flash = args.flash;
    }
    view.update(&gpu, &frame, std::time::Duration::from_millis(4));
    view.settle(&gpu, &frame, width as f32 / height as f32);
    // The scene's particle systems run on the wall clock: let them emit before the frame is
    // taken.
    for step in 0..args.warm_up * 20 {
        // The AC1 particles run on: their emitters ride last frame's placements.
        if let Some(showcase) = ac1.as_mut() {
            showcase.fx.frame(
                &mut view,
                &mut frame,
                f64::from(step) * 0.05,
                world.block_size(),
            );
            view.update(&gpu, &frame, std::time::Duration::from_millis(4));
        }
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
    if args.no_hud {
        view.render_world(&gpu, &target.color_view, &target.depth_view);
    } else {
        view.render(&gpu, &target.color_view, &target.depth_view, true);
    }
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
        if args.no_hud {
            return;
        }
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

/// `--hud-panel`: logs `view`'s HUD on to the session as the window does, waits for the HUD to
/// build, opens `panel` and merges the session's parts into `frame` (its player id, what the
/// player holds, the inventory).
fn hud_session(
    view: &mut ClientView,
    gpu: &Gpu,
    store: &Arc<ResourceStore>,
    world: &Arc<Ac2WorldSource>,
    args: &Args,
    frame: &mut ac2_client::ClientFrame,
    panel: ac2_ui::ToolbarChild,
    weapons: Option<&mut ac1_held::Ac1Weapons>,
) {
    let (x, y) = args.at.unwrap_or_else(|| {
        let half = world.block_size() / 2.0;
        (half, half)
    });
    let landing = walk::Landing {
        block: args.block,
        x,
        y,
    };
    let mut session = match session::Session::start(store, world, landing) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("world-viewer: no HUD session: {error}");
            return;
        }
    };
    // The AC1 weapons go in the pack; the hand shows what the inventory wields.
    let equip = match weapons {
        Some(weapons) => {
            session.give_items(&weapons.items());
            weapons.current = None;
            args.equip.filter(|i| *i < weapons.weapons.len())
        }
        None => None,
    };
    let step = |view: &mut ClientView, session: &mut session::Session| {
        let mut merged = frame.clone();
        session.frame(view, &mut merged, 1.0 / 64.0);
        view.update(gpu, &merged, std::time::Duration::from_millis(4));
        merged
    };
    let started = std::time::Instant::now();
    while view.shortcut_slots() == 0 && started.elapsed() < std::time::Duration::from_secs(60) {
        step(view, &mut session);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    view.show_panel(panel, true);
    if let Some(index) = equip {
        // As the paper doll's primary hand does (`gmCWInterface::EquipItem`).
        session.command(ac2_client::Command::EquipItem {
            item: ac1_held::Ac1Weapons::item_id(index),
            location: legions_server_double::PRIMARY_HAND,
        });
    }
    let mut merged = frame.clone();
    for _ in 0..32 {
        merged = step(view, &mut session);
    }
    *frame = merged;
}

/// Holds the camera action `action` for `seconds` of simulated 60 Hz time from `now` about the
/// standing player (`--orbit` turns it to the player's side, `--zoom` closes in). Returns the
/// simulated time reached.
fn hold_camera(walk: &mut walk::Walk, action: u32, seconds: f32, mut now: f64) -> f64 {
    const STEP: f32 = 1.0 / 60.0;
    walk.camera_action(action, true);
    let mut turned = 0.0;
    while turned < seconds {
        now += f64::from(STEP);
        turned += STEP;
        walk.update(STEP, now, walk::WalkInput::default());
    }
    walk.camera_action(action, false);
    now
}

/// Lets the standing player animate for `seconds` of simulated 60 Hz time from `now`.
fn stand(walk: &mut walk::Walk, seconds: f32, mut now: f64) -> f64 {
    const STEP: f32 = 1.0 / 60.0;
    let mut stood = 0.0;
    while stood < seconds {
        now += f64::from(STEP);
        stood += STEP;
        walk.update(STEP, now, walk::WalkInput::default());
    }
    now
}

/// Runs the player in simulated 60 Hz time: until it is in the world (the cells load on their
/// own threads), then `seconds` of walking forward. Returns the simulated time reached.
fn walk_for(
    view: &mut ClientView,
    walk: &mut walk::Walk,
    gpu: &Gpu,
    seconds: f32,
    visual: ac2_core::DataId,
) -> f64 {
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
                return now;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    now
}
