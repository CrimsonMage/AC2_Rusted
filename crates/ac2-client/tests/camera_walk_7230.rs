//! The chase camera follows a walking player without jumps: walking across cells (in the start
//! dungeon, 7230 DATs, and outdoors over a landblock edge), the camera's step each frame stays
//! with the player's. Skipped without `AC2_DAT_DIR`.

#![cfg(feature = "window")]

mod common;

use std::sync::Arc;
use std::time::Duration;

use ac2_client::{Client, DatWorld, FrameInput};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use common::script_rules::{new_host, ScriptRules};
use legions_server_double::{addr, double_with_rules, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;

type Server = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn log_in(spawn: Spawn) -> Option<(Client, Server, Arc<ac2_db::ResourceStore>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let tables_dataset = Arc::clone(&dataset);
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
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
    let local_dir = std::env::temp_dir().join(format!("ac2-camwalk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&local_dir);
    client.set_window_positions_dir(Some(local_dir));
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
    let mut spins = 0;
    while (client.frame().portal.in_portal_space || spins < 256) && spins < 4000 {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        spins += 1;
    }
    Some((client, server, store))
}

/// A cell-local origin in world metres (`LandDefs` block grid, 192 m blocks).
fn world(cell: u32, origin: [f32; 3]) -> [f64; 3] {
    [
        f64::from(cell >> 24) * 192.0 + f64::from(origin[0]),
        f64::from((cell >> 16) & 0xff) * 192.0 + f64::from(origin[1]),
        f64::from(origin[2]),
    ]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Walks forward for `frames` frames (turning a little now and then with `turn`) and returns the
/// frames where the camera moved over a metre against the player: a snap, not the walls' pull-in
/// easing in and out (a camera grazing a sloped ceiling moves some 0.3 m).
fn walk(
    client: &mut Client,
    server: &Server,
    store: &ac2_db::ResourceStore,
    frames: usize,
    turn: bool,
) -> usize {
    use ac2_ui::{Controls, InputEvent};
    let mut controls = Controls::load(store).unwrap().expect("controls");
    controls.event(
        InputEvent::Key {
            scan: 0x11,
            pressed: true,
        },
        0,
        0.0,
    );
    let mut last: Option<([f64; 3], [f64; 3], u32, u32)> = None;
    let (mut jumps, mut seen) = (0, 0);
    for step in 0..frames {
        let mut input = ac2_client::frame_input(controls.frame());
        if turn && (step / 96) % 2 == 1 {
            input.mouse_look = Some((3, 0));
        }
        client.use_time(&input).unwrap();
        server.borrow_mut().elapsed += STEP;
        let frame = client.frame();
        let (Some(player), Some(camera)) = (frame.player.as_ref(), frame.camera) else {
            continue;
        };
        let p = world(player.cell, player.origin);
        let c = world(camera.cell, camera.origin);
        if let Some((lp, lc, lpc, lcc)) = last {
            seen += 1;
            // The camera's place relative to the player: smooth unless a wall pulls it in.
            let rel = [c[0] - p[0], c[1] - p[1], c[2] - p[2]];
            let last_rel = [lc[0] - lp[0], lc[1] - lp[1], lc[2] - lp[2]];
            let (ps, cs) = (dist(p, lp), dist(rel, last_rel));
            if step > 64 && cs > 1.0 {
                jumps += 1;
                eprintln!(
                    "frame {step}: camera moved {cs:.3} m against the player, player {ps:.3} m; player cell {lpc:08x}->{:08x}, camera cell {lcc:08x}->{:08x}, camera {:?} -> {:?}, distance {:.2}",
                    player.cell, camera.cell, lc, c, dist(p, c)
                );
            }
        }
        last = Some((p, c, player.cell, camera.cell));
    }
    eprintln!("{jumps} jumps in {seen} frames");
    jumps
}

#[test]
fn the_camera_follows_a_player_walking_through_the_dungeon() {
    let spawn = Spawn {
        cell: 0x0299_011B,
        origin: [49.525, 230.409, -43.616],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let Some((mut client, server, store)) = log_in(spawn) else {
        return;
    };
    let look = FrameInput {
        mouse_look_state: Some(true),
        ..FrameInput::default()
    };
    client.use_time(&look).unwrap();
    let jumps = walk(&mut client, &server, &store, 6 * 64, false);
    assert_eq!(jumps, 0, "the camera jumped while the player walked");
}

#[test]
fn the_camera_follows_a_player_walking_outdoors() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let dat_world = DatWorld::new(Arc::new(ac2_db::ResourceStore::new(dataset))).unwrap();
    let top = dat_world
        .world()
        .load_landblock(ac2_core::CellId::new(0xCB4D_0001))
        .unwrap()
        .unwrap()
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    let spawn = Spawn {
        cell: 0xCB4D_0028,
        origin: [96.0, 170.0, top + 1.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let Some((mut client, server, store)) = log_in(spawn) else {
        return;
    };
    let jumps = walk(&mut client, &server, &store, 10 * 64, false);
    assert_eq!(jumps, 0, "the camera jumped while the player walked");
}
