//! The headless client on the real DATs (`AC2_DAT_DIR`) against the Legions 7160 server double:
//! spawn at the human start (an interior cell), walk forward into the room's wall, log off.

use std::sync::Arc;

use ac2_client::{Client, DatWorld, FrameInput};
use ac2_net::Credential;
use ac2_weenie::{
    ActiveAction, CharacterChoice, OnlineConfig, OnlineSession, ACTION_CLASS_MOVEMENT,
    ACTION_FORWARD,
};
use legions_server_double::{addr, double, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;

#[test]
fn the_client_walks_into_a_wall_of_the_start_room() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let dataset = ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(root)).unwrap();
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let world = DatWorld::new(store).unwrap();
    let spawn = Spawn {
        cell: 0x0299_0109,
        origin: [59.060577, 240.199, -44.606205],
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
    let session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let mut client = Client::new(session, Arc::new(world));
    client.set_load_budget(std::time::Duration::from_secs(30));
    let idle = FrameInput::default();
    let mut frames = 0;
    while !client.frame().in_world {
        client
            .settle_loads(std::time::Duration::from_secs(30))
            .unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        frames += 1;
        assert!(frames < 600);
    }
    let walk = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    let mut track = Vec::new();
    for _ in 0..(10 * 64) {
        client.use_time(&walk).unwrap();
        server.borrow_mut().elapsed += STEP;
        track.push(client.frame().player.unwrap());
    }
    // The human body animates with its behaviour table: its Setup parts are posed.
    assert!(!client.frame().player.unwrap().pose.is_empty());
    // The camera follows behind the player.
    let camera = client.frame().camera.expect("camera");
    assert!(camera.origin.iter().all(|v| v.is_finite()), "{camera:?}");
    let first = &track[0];
    let last = track.last().unwrap();
    let second_before = &track[track.len() - 64];
    // It walked, and the last second it made no headway: a wall holds it (pressing into it,
    // the body only shuffles along the face).
    assert!((last.origin[1] - first.origin[1]).abs() > 1.0, "{last:?}");
    let moved: f32 = (0..3)
        .map(|i| (last.origin[i] - second_before.origin[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    assert!(moved < 0.25, "{moved}");
    assert!(
        (last.origin[1] - second_before.origin[1]).abs() < 0.1,
        "{last:?}"
    );
    assert!(
        last.cell & 0xFFFF >= 0x100,
        "still indoors: {:08X}",
        last.cell
    );
    assert!(client
        .session_mut()
        .log_off(std::time::Duration::from_secs(3)));
}
