//! The portal space drawn headless on the real DATs (`AC2_DAT_DIR`) against the Legions 7160
//! server double: the town block is drawn, then the player is teleported to the 0299 interior
//! and the frame in the portal space's tunnel phase is drawn without the world (the fader and
//! the HUD only). Writes `target/ac2-snapshots/portal_town.png` and `portal_tunnel.png`; skipped
//! without a GPU adapter.

#![cfg(feature = "window")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_render::portal::Phase;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use legions_server_double::{addr, double, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;
const SIZE: (u32, u32) = (800, 600);
const INTERIOR_CELL: u32 = 0x0299_0109;
const INTERIOR_ORIGIN: [f32; 3] = [59.0, 240.0, -44.5];

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

fn mean_luminance(rgba: &[u8]) -> f32 {
    rgba.chunks_exact(4)
        .map(|px| u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2]))
        .sum::<u32>() as f32
        / (rgba.len() / 4 * 3) as f32
}

#[test]
fn the_tunnel_phase_draws_no_world_only_the_fader_and_the_hud() {
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
    // a cache-only VM, so the teleport's IFuns run.
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
    server.borrow_mut().send_init_player();
    for _ in 0..128 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    // The login's portal space (the placement's enter, then `LoginFinished`'s leave) fades out
    // before play; a teleport started during that fade would be ignored.
    let mut login = client.frame();
    let mut login_frames = 0;
    while login.portal.in_portal_space {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        login = client.frame();
        login_frames += 1;
        assert!(
            login_frames < 2000,
            "the login's portal space did not run out"
        );
    }

    // The town, drawn with its world.
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let mut view = ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE);
    let town = client.frame();
    view.update(&gpu, &town, Duration::ZERO);
    view.settle(&gpu, &town, aspect);
    assert!(view.prepare(&gpu, &town, aspect, 0.0));
    view.render(&gpu, &target.color_view, &target.depth_view, true);
    let town_rgba = target.read_rgba(&gpu.device, &gpu.queue);
    write_png("portal_town.png", &town_rgba);
    let town_mean = mean_luminance(&town_rgba);

    // The teleport: the position with a newer teleport stamp; the client runs to the tunnel.
    server
        .borrow_mut()
        .teleport_player(INTERIOR_CELL, INTERIOR_ORIGIN, 5, 6);
    let mut tunnel = client.frame();
    for _ in 0..(3 * 64) {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        tunnel = client.frame();
        if tunnel.portal.phase == Phase::Active {
            break;
        }
    }
    assert_eq!(tunnel.portal.phase, Phase::Active, "the tunnel phase");
    assert!(!tunnel.portal.world_drawn());
    assert!(view.prepare(&gpu, &tunnel, aspect, 0.0));
    view.render(&gpu, &target.color_view, &target.depth_view, true);
    let tunnel_rgba = target.read_rgba(&gpu.device, &gpu.queue);
    write_png("portal_tunnel.png", &tunnel_rgba);
    let tunnel_mean = mean_luminance(&tunnel_rgba);
    eprintln!("town mean {town_mean:.1}, tunnel mean {tunnel_mean:.1}");
    assert!(
        tunnel_mean < town_mean * 0.8,
        "the world is not drawn in the tunnel ({tunnel_mean} vs {town_mean})"
    );
}
