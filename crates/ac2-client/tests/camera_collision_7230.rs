//! `SmartBox::UpdateCamera`'s collision through the client: with the player standing in the start
//! dungeon (cell 0x0299011B, 7230 DATs) and the mouse-look camera turned all the way round, the
//! camera never leaves the player's side of the walls. With `AC2_SHOT_DIR` set it renders the
//! most pulled-in frame with and without the pull-in. Skipped without `AC2_DAT_DIR`.

#![cfg(feature = "window")]

mod common;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use common::script_rules::{new_host, ScriptRules};
use legions_server_double::{addr, double_with_rules, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;
const SIZE: (u32, u32) = (1280, 720);
const CELL: u32 = 0x0299_011B;
const ORIGIN: [f32; 3] = [49.525, 230.409, -43.616];

#[allow(dead_code)]
fn write_png(dir: &std::path::Path, name: &str, rgba: &[u8]) {
    std::fs::create_dir_all(dir).expect("snapshot directory");
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
fn the_turning_camera_stays_inside_the_dungeon() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let out = std::env::var_os("AC2_SHOT_DIR").map(PathBuf::from);
    let tables_dataset = Arc::clone(&dataset);
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    let spawn = Spawn {
        cell: CELL,
        origin: ORIGIN,
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = double_with_rules(spawn, Box::new(ScriptRules(new_host(&store))));
    let config = OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    };
    let mut session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let script_data = ac2_script::data::DatScriptData::new(tables_dataset, (*store).clone());
    session.set_string_tables(ac2_client::string_table_ids(&script_data));
    session.set_chat_commands(ac2_client::chat_commands(&script_data));
    let dat_world = dat_world.with_script_data(script_data.clone());
    let mut client = Client::new(session, Arc::new(dat_world));
    let local_dir = std::env::temp_dir().join(format!("ac2-dungeon-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&local_dir);
    client.set_window_positions_dir(Some(local_dir.clone()));
    let wlib = ac2_script::load_wlib(&store, ac2_core::DataId::new(ac2_script::LEGIONS_WLIB_DID))
        .unwrap()
        .expect("the script library");
    let program = Arc::new(ac2_script::vm::Program::from_stream(&wlib.stream));
    script_data.preload_tables();
    script_data.preload_states();
    script_data.preload_skill_states(&program);
    script_data.set_cache_only(true);
    client.attach_scripts(
        ac2_script::host::ScriptHost::new(program, Box::new(script_data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    client.set_load_budget(Duration::from_secs(30));
    let idle = FrameInput::default();
    let mut frames = 0;
    while !client.frame().in_world {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        frames += 1;
        assert!(frames < 600);
    }
    server.borrow_mut().send_init_player();
    // The AC2e training-world placements (content/training-world.tsv): a lifestone and a
    // Shelter Guard in this block.
    server.borrow_mut().spawn_creature(
        0x8140_0000_0003_0001,
        0x1f00_002d,
        0x0299_0109,
        [55.060_577, 240.199, -44.606_205],
    );
    server.borrow_mut().spawn_creature(
        0x8140_0000_0003_0040,
        0x1f00_0d74,
        0x0299_0109,
        [52.159_115, 242.034_38, -44.450_88],
    );
    for _ in 0..128 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    let mut spins = 0;
    while client.frame().portal.in_portal_space && spins < 4000 {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        spins += 1;
    }
    let frame = client.frame();
    assert_eq!(frame.player.as_ref().unwrap().cell, CELL);
    // The same cells, to trace against.
    let mut terrain = ac2_physics::Terrain::new();
    assert!(ac2_smartbox::load_dat_block(&world, &mut terrain, 0x0299_0000).unwrap());
    let look = FrameInput {
        mouse_look_state: Some(true),
        ..FrameInput::default()
    };
    client.use_time(&look).unwrap();
    server.borrow_mut().elapsed += STEP;
    let turn = FrameInput {
        mouse_look: Some((12, 0)),
        ..FrameInput::default()
    };
    let (mut pulled, mut frames_seen, mut worst) = (0, 0, (0.0_f64, None));
    // The camera never stands in the player it follows: its ray starts inside the player's upper
    // sphere (0.551 m from its centre, radius 0.555), and the player does not block it.
    let mut shortest = f64::MAX;
    for _ in 0..240 {
        client.use_time(&turn).unwrap();
        server.borrow_mut().elapsed += STEP;
        let frame = client.frame();
        let player = frame.player.as_ref().unwrap();
        let camera = frame.camera.expect("the camera");
        assert!(
            terrain_has_env_cell(&terrain, camera.cell),
            "camera cell {:08x}",
            camera.cell
        );
        let offset = client.camera().target_offset;
        let head = ac2_physics::Position::new(
            player.cell,
            ac2_physics::Frame::new(
                ac2_physics::Vector::new(
                    player.origin[0] + offset.x,
                    player.origin[1] + offset.y,
                    player.origin[2] + offset.z,
                ),
                ac2_physics::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ),
        );
        let eye = ac2_physics::Position::new(
            camera.cell,
            ac2_physics::Frame::new(
                ac2_physics::Vector::new(camera.origin[0], camera.origin[1], camera.origin[2]),
                ac2_physics::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ),
        );
        let reach = head.offset_to(&eye);
        let length = f64::from(reach.x)
            .hypot(f64::from(reach.y))
            .hypot(f64::from(reach.z));
        let wanted = f64::from(client.camera().camera_distance);
        shortest = shortest.min(length);
        assert!(length <= wanted + 0.6, "{length} past the wanted {wanted}");
        // Nothing the camera cannot pass lies between the player's head and the camera.
        let mut los = ac2_physics::LosParameters::new();
        los.set_from_begin_and_end_position(&head, &eye);
        los.set_los_params(
            &ac2_physics::WeenieTypes::default(),
            ac2_physics::camera_ethereal_mask(),
        );
        assert!(terrain.los_check(&mut los) && los.trace_result() == 0);
        frames_seen += 1;
        if length < wanted - 0.3 {
            pulled += 1;
            if wanted - length > worst.0 {
                worst = (wanted - length, Some((frame.clone(), wanted, length)));
            }
        }
    }
    eprintln!(
        "{pulled} of {frames_seen} frames pulled in (worst by {:.2} m), nearest {shortest:.3} m",
        worst.0
    );
    assert!(
        shortest > 0.6,
        "the camera came {shortest:.3} m from the head: it stood in the player"
    );
    assert!(
        pulled > 0,
        "the room's walls never pulled the turning camera in"
    );

    let (Some(out), Some((frame, wanted, length))) = (out, worst.1) else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping the renders");
        return;
    };
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let settings =
        ac2_client::ClientPrefs::load(local_dir.join("prefs.cfg"), Arc::clone(&store)).unwrap();
    let mut prefs = settings;
    prefs.apply_startup_quality(3);
    let mut view = ClientView::new(Arc::clone(&world), &gpu, COLOR_FORMAT, 1, SIZE);
    view.set_scene_options(prefs.scene_options());
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    for (name, extend) in [
        ("after_pulled_in", false),
        ("before_through_the_wall", true),
    ] {
        let mut frame = frame.clone();
        if extend {
            // Where the camera would stand without the pull-in: the same line, the full distance.
            let camera = frame.camera.as_mut().unwrap();
            let scale = (wanted / length) as f32;
            let p = frame.player.as_ref().unwrap();
            let offset = client.camera().target_offset;
            let h = [
                p.origin[0] + offset.x,
                p.origin[1] + offset.y,
                p.origin[2] + offset.z,
            ];
            for (axis, from) in camera.origin.iter_mut().zip(h) {
                *axis = from + (*axis - from) * scale;
            }
        }
        view.update(&gpu, &frame, Duration::ZERO);
        view.settle(&gpu, &frame, aspect);
        assert!(view.prepare(&gpu, &frame, aspect, 0.0));
        for step in 0..90 {
            if step % 10 == 0 {
                view.settle(&gpu, &frame, aspect);
            }
            view.update(&gpu, &frame, Duration::from_millis(16));
            view.prepare(&gpu, &frame, aspect, 0.016);
            view.render(&gpu, &target.color_view, &target.depth_view, true);
        }
        let rgba = target.read_rgba(&gpu.device, &gpu.queue);
        write_png(&out, &format!("{name}.png"), &rgba);
    }
}

fn terrain_has_env_cell(terrain: &ac2_physics::Terrain, cell: u32) -> bool {
    use ac2_physics::CellSource;
    terrain.env_cell(cell).is_some()
}
