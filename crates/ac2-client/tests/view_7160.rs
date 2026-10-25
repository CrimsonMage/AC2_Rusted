//! The drawn client on the real DATs (`AC2_DAT_DIR`) against the Legions 7160 server double:
//! log in outdoors in the town block CB4D, walk forward with W through the retail controls
//! (key maps, action interpreter), and render the frame headless with the player's character
//! posed under its appearance. Writes `target/ac2-snapshots/client_cb4d_*.png`; skipped without
//! a GPU adapter.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{frame_input, Client, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_ui::{Controls, InputEvent};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use legions_server_double::{addr, double, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;
/// The smallest display retail ran at; the HUD layouts are authored for 1024 x 768.
const SIZE: (u32, u32) = (800, 600);
/// DirectInput's `DIK_W`.
const DIK_W: u32 = 0x11;

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
fn the_client_walks_with_w_and_draws_its_character() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let tables_dataset = Arc::clone(&dataset);
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    // Outdoor cell 0x25 (the block centre) of CB4D, above the highest ground: the body falls
    // to the terrain.
    let block = ac2_core::CellId::new(0xCB4D_0001);
    let top = world
        .load_landblock(block)
        .unwrap()
        .unwrap()
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    let spawn = Spawn {
        cell: 0xCB4D_0025,
        origin: [96.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = double(spawn);
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
    // As the binary does: the string tables, the chat command line and the player's scripts on
    // a cache-only VM.
    let script_data = ac2_script::data::DatScriptData::new(tables_dataset, (*store).clone());
    session.set_string_tables(ac2_client::string_table_ids(&script_data));
    session.set_chat_commands(ac2_client::chat_commands(&script_data));
    let dat_world = dat_world.with_script_data(script_data.clone());
    let mut client = Client::new(session, Arc::new(dat_world));
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
    // The server's INIT_PLAYER runs the player's script (`CPlayer`).
    server.borrow_mut().send_init_player();
    // Settle onto the ground.
    for _ in 0..128 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    let start = client.frame().player.unwrap();

    let mut controls = Controls::load(&store).unwrap().expect("controls");
    let mut now = 0.0;
    controls.event(
        InputEvent::Key {
            scan: DIK_W,
            pressed: true,
        },
        0,
        now,
    );
    for _ in 0..(2 * 64) {
        client.use_time(&frame_input(controls.frame())).unwrap();
        server.borrow_mut().elapsed += STEP;
        now += STEP;
    }
    controls.event(
        InputEvent::Key {
            scan: DIK_W,
            pressed: false,
        },
        (now * 1000.0) as u32,
        now,
    );
    client.use_time(&frame_input(controls.frame())).unwrap();
    let end = client.frame().player.unwrap();
    let walked: f32 = (0..2)
        .map(|i| (end.origin[i] - start.origin[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    eprintln!(
        "walked {walked:.2} m from {:?} to {:?}",
        start.origin, end.origin
    );
    assert!(walked > 2.0, "W walks the player forward");

    // Mouse look: the middle button turns it on (through the camera UI), the mouse turns the
    // camera, and walking steers the body along the camera.
    controls.event(
        InputEvent::Button {
            button: 2,
            pressed: true,
        },
        (now * 1000.0) as u32,
        now,
    );
    controls.event(
        InputEvent::Button {
            button: 2,
            pressed: false,
        },
        (now * 1000.0) as u32 + 50,
        now + 0.05,
    );
    client.use_time(&frame_input(controls.frame())).unwrap();
    let state = client.camera().movement_state();
    assert!(state.mouse_look, "the middle button turns mouse look on");
    controls.input.set_mouse_look(true);
    let before = state.direction;
    for _ in 0..30 {
        controls.event(
            InputEvent::Motion { dx: 6, dy: 0 },
            (now * 1000.0) as u32,
            now,
        );
        client.use_time(&frame_input(controls.frame())).unwrap();
        server.borrow_mut().elapsed += STEP;
        now += STEP;
    }
    let after = client.camera().movement_state().direction;
    let turned = (before[0] * after[1] - before[1] * after[0])
        .atan2(before[0] * after[0] + before[1] * after[1])
        .to_degrees();
    eprintln!("mouse look turned the camera {turned:.1} degrees");
    assert!(turned.abs() > 60.0, "{turned}");
    controls.event(
        InputEvent::Key {
            scan: DIK_W,
            pressed: true,
        },
        (now * 1000.0) as u32,
        now,
    );
    // The body turns to the camera's heading as it walks; take the last half second.
    for _ in 0..96 {
        client.use_time(&frame_input(controls.frame())).unwrap();
        server.borrow_mut().elapsed += STEP;
        now += STEP;
    }
    let facing = client.frame().player.unwrap();
    for _ in 0..32 {
        client.use_time(&frame_input(controls.frame())).unwrap();
        server.borrow_mut().elapsed += STEP;
        now += STEP;
    }
    let steered = client.frame().player.unwrap();
    let heading =
        (steered.origin[1] - facing.origin[1]).atan2(steered.origin[0] - facing.origin[0]);
    let camera_heading = after[1].atan2(after[0]);
    let off = (heading - camera_heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    eprintln!(
        "walking {:.0} deg, camera {:.0} deg",
        heading.to_degrees(),
        camera_heading.to_degrees()
    );
    assert!(
        off.abs().to_degrees() < 20.0,
        "mouse look steers the walk ({off})"
    );

    let frame = client.frame();
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let mut view = ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE);
    view.update(&gpu, &frame, Duration::ZERO);
    view.settle(&gpu, &frame, aspect);
    assert_eq!(
        view.loaded_objects(&frame),
        1,
        "the player's character model loads"
    );
    assert_eq!(
        view.hud_layers(),
        16,
        "every HUD root (and the reticle's objects, the shortcut bar and the first chat window) is drawn"
    );
    assert!(view.prepare(&gpu, &frame, aspect, 0.0));
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| view.draw(pass));
    let rgba = target.read_rgba(&gpu.device, &gpu.queue);
    write_png("client_cb4d_walk.png", &rgba);
    let mean = rgba
        .chunks_exact(4)
        .map(|px| u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2]))
        .sum::<u32>() as f32
        / (rgba.len() / 4 * 3) as f32;
    eprintln!("mean {mean:.1}");
    assert!(mean > 20.0, "the world is drawn ({mean})");

    // Clicking the character selects it (`gmSelectionUI` over the selection render): its middle,
    // a metre above its feet, projected to the screen.
    let player = frame.player.as_ref().expect("player");
    let camera = view.camera(&frame).expect("camera");
    let body = view.world_point(player.cell, player.origin) + ac2_math::Vec3::new(0.0, 0.0, 1.0);
    let clip = camera.view_projection(aspect) * body.extend(1.0);
    let (x, y) = (clip.x / clip.w, clip.y / clip.w);
    let pixel = (
        ((x + 1.0) * 0.5 * SIZE.0 as f32) as i32,
        ((1.0 - y) * 0.5 * SIZE.1 as f32) as i32,
    );
    assert_eq!(
        view.pick(&frame, SIZE, pixel),
        Some(player.id),
        "at {pixel:?}"
    );
    assert_eq!(view.pick(&frame, SIZE, (5, 5)), None, "the sky's corner");

    // Selecting it frames it in the targeting reticle (`gm3DTextUI`), its name above.
    let shown_before = view.hud_shown_layers();
    let select = FrameInput {
        commands: vec![ac2_client::Command::Select(Some(player.id))],
        ..FrameInput::default()
    };
    client.use_time(&select).unwrap();
    server.borrow_mut().elapsed += STEP;
    let frame = client.frame();
    let selection = frame.selection.as_ref().expect("the character is selected");
    assert!(view.prepare(&gpu, &frame, aspect, 0.0));
    let shown = view.hud_shown_layers();
    // `UpdateReticleInternal` shows the corners only with a name to show above them: the
    // double's character is named, so its five layers show.
    assert_eq!(selection.name.as_deref(), Some("Explorer"));
    assert_eq!(
        shown,
        shown_before + 5,
        "the reticle follows the name ({:?})",
        selection.name
    );
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| view.draw(pass));
    write_png(
        "client_cb4d_selected.png",
        &target.read_rgba(&gpu.device, &gpu.queue),
    );

    // Enter opens the first chat window's entry line, typing fills it and Enter sends it
    // (`ChatWindow::HandleEnterKey` / `ProcessCommand`).
    let enter = |view: &mut ClientView| {
        let modifiers = ac2_ui::Modifiers::default();
        view.ui_input(
            &[(0x4000_0018, 1.0, true), (0x4000_0018, 0.0, false)],
            modifiers,
        );
    };
    assert!(!view.text_mode());
    enter(&mut view);
    assert!(view.text_mode(), "the entry line takes typing");
    let typed: Vec<(u16, u32)> = "well met".encode_utf16().map(|c| (c, 0)).collect();
    view.text_input(&[], &typed, ac2_ui::Modifiers::default());
    enter(&mut view);
    assert!(!view.text_mode());
    assert_eq!(
        view.take_chat_requests(),
        vec![ac2_ui::ChatRequest::Line("well met".into())]
    );
    // The toolbar panels start hidden; the skills key shows its panel.
    assert!(!view.panel_shown(ac2_ui::ToolbarChild::Skill));
    view.ui_input(
        &[(0x4100_0006, 1.0, true), (0x4100_0006, 0.0, false)],
        ac2_ui::Modifiers::default(),
    );
    assert!(view.panel_shown(ac2_ui::ToolbarChild::Skill));
    // A line said through the server comes back as a string-table line, rendered off the frame
    // thread, and reaches the log of a window whose filter shows every text type.
    let mut settings = ac2_ui::ChatSettings::default();
    settings.filters.insert(1, u32::MAX);
    view.reload_chat_settings(&settings);
    let say = FrameInput {
        commands: vec![ac2_client::Command::ChatLine("Well met".into())],
        ..FrameInput::default()
    };
    let mut events = client.use_time(&say).unwrap();
    for _ in 0..4 {
        server.borrow_mut().elapsed += STEP;
        events.extend(client.use_time(&FrameInput::default()).unwrap());
    }
    for event in &events {
        if let ac2_weenie::SessionEvent::Notice(ac2_weenie::GameplayNotice::DisplayStringInfo {
            kind,
            text,
        }) = event
        {
            view.chat_display_info(*kind, text);
        }
    }
    let frame = client.frame();
    let mut log = String::new();
    for _ in 0..200 {
        view.update(&gpu, &frame, Duration::ZERO);
        log = view.chat_log().unwrap_or_default();
        if log.contains("Well met") {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    eprintln!("chat log: {log:?}");
    assert!(log.contains("You say, \"Well met\""), "{log:?}");
    // The frameworks that ask the player load once the string tables are in: the shortcut bar
    // pads to its slots, and the skills panel follows an examination request for the race's
    // melee panel (the request goes through the server and comes back as a notice).
    let mut ready = false;
    for _ in 0..300 {
        view.update(&gpu, &frame, Duration::ZERO);
        let notices = client.take_script_notices();
        let commands = view.frameworks(&mut client, &frame, &[], &notices);
        assert!(commands.is_empty(), "{commands:?}");
        if view.shortcut_slots() > 0 {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(ready, "the UI string tables load");
    assert_eq!(view.shortcut_slots(), 100);
    // `InqRootPanelFromRace` answers 0 for the double's player (it carries no race), so the
    // human melee root panel (`HumanRootMeleePanel_SkillPanel`) is asked for directly.
    let race_root = client.root_panel_from_race(ac2_ui::ROOT_MELEE_PANEL);
    eprintln!("InqRootPanelFromRace(melee): {race_root:?}");
    let root = 0x8000_000C;
    let examine = FrameInput {
        commands: vec![ac2_client::Command::Examine(
            ac2_net::protocol::ExaminationRequest {
                kind: ac2_ui::EXAMINE_PANEL,
                data_enum: root,
                ..Default::default()
            },
        )],
        ..FrameInput::default()
    };
    let mut events = client.use_time(&examine).unwrap();
    for _ in 0..4 {
        server.borrow_mut().elapsed += STEP;
        events.extend(client.use_time(&FrameInput::default()).unwrap());
    }
    let notices = client.take_script_notices();
    view.frameworks(&mut client, &frame, &events, &notices);
    assert_eq!(view.skills_current(), Some((root, 0)), "{events:?}");
    // Examining the selected character (`gmSelectionUI` right click): the panel opens, holds the
    // request open, and lays out the server's profile.
    {
        use ac2_net::protocol::{
            ExaminationDataNode, ExaminationProfile, ExaminationRequest, ExaminationValue,
            StringInfo,
        };
        let node = |order, value| ExaminationDataNode {
            order,
            appearance: 0,
            value,
        };
        let name = StringInfo {
            literal: Some("Explorer".into()),
            ..StringInfo::default()
        };
        server.borrow_mut().script_world.profiles.insert(
            player.id,
            ExaminationProfile {
                request: ExaminationRequest::object(player.id),
                nodes: vec![
                    node(1, ExaminationValue::Text(name)),
                    node(2, ExaminationValue::Break),
                    node(3, ExaminationValue::Int(42)),
                ],
            },
        );
    }
    let examine = FrameInput {
        commands: vec![ac2_client::Command::Examine(
            ac2_net::protocol::ExaminationRequest::object(player.id),
        )],
        ..FrameInput::default()
    };
    let mut events = client.use_time(&examine).unwrap();
    view.open_examination();
    let mut texts = Vec::new();
    for _ in 0..20 {
        let notices = client.take_script_notices();
        let commands = view.frameworks(&mut client, &frame, &events, &notices);
        let input = FrameInput {
            commands,
            ..FrameInput::default()
        };
        server.borrow_mut().elapsed += STEP;
        events = client.use_time(&input).unwrap();
        texts = view.examination_texts();
        if texts.len() > 1 {
            break;
        }
    }
    eprintln!("examination: {texts:?}");
    assert!(view.panel_shown(ac2_ui::ToolbarChild::Examination));
    assert!(texts.iter().any(|t| t == "Explorer"), "{texts:?}");
    assert!(texts.iter().any(|t| t.contains("42")), "{texts:?}");
    // Creating a fellowship (`gmFellowshipUI::HandleCreateButton`): the double answers with the
    // player leading and Birch; the page lists both, the leader first.
    let create = FrameInput {
        commands: vec![ac2_client::Command::Social(
            ac2_weenie::SocialRequest::FellowshipCreate {
                name: "Wanderers".into(),
                social: 1,
                looting: 1,
            },
        )],
        ..FrameInput::default()
    };
    let mut events = client.use_time(&create).unwrap();
    let mut rows = Vec::new();
    for _ in 0..20 {
        let notices = client.take_script_notices();
        let commands = view.frameworks(&mut client, &frame, &events, &notices);
        let input = FrameInput {
            commands,
            ..FrameInput::default()
        };
        server.borrow_mut().elapsed += STEP;
        events = client.use_time(&input).unwrap();
        rows = view.fellowship_rows();
        if rows.len() > 1 {
            break;
        }
    }
    eprintln!("fellowship: {rows:?}");
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![player.id, legions_server_double::FELLOW],
        "{rows:?}"
    );
    assert_eq!(rows[1].1, "Birch");
    // A displayed line reaches the log on the next update; the window shows it.
    view.chat_display(0x4_0000, "Explorer says, \"well met\"\n");
    // Images set since the build (icons) load on a worker; give them a moment.
    for _ in 0..20 {
        view.update(&gpu, &frame, Duration::ZERO);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(view.prepare(&gpu, &frame, aspect, 0.0));
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| view.draw(pass));
    write_png(
        "client_cb4d_chat.png",
        &target.read_rgba(&gpu.device, &gpu.queue),
    );
}
