//! The UI's sounds on the real DATs (`AC2_DAT_DIR`): pressing a gameplay toolbar button runs
//! its `normal_pressed` state media, whose `MD_Data_Sound` the client plays through
//! `SoundManager::Play`; the sound loads on the loader thread and its wave reaches a recording
//! (headless) sink on the next `UseTime`.

use std::sync::Arc;
use std::time::Duration;

use ac2_audio::{RecordingSink, SinkCommand};
use ac2_ui::{UI_POINTER_X, UI_POINTER_Y};

/// The left mouse button's UI action.
const LEFT: u32 = 0x4000_0004;

#[test]
fn a_pressed_button_sound_reaches_the_sink() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = ac2_ui::build_gameplay_hud(&store, 800, 600).unwrap();
    let tree = &mut hud.tree;
    tree.global_loop(0.0);
    tree.take_sounds();

    // A toolbar button along the bottom edge (800 x 600).
    tree.start_input(UI_POINTER_X, 210.0);
    tree.start_input(UI_POINTER_Y, 580.0);
    tree.start_input(LEFT, 1.0);
    tree.global_loop(0.1);
    let sounds = tree.take_sounds();
    assert!(!sounds.is_empty(), "the pressed state plays a sound");
    tree.stop_input(LEFT);
    eprintln!("button sounds {sounds:08X?}");
    assert!(
        sounds.iter().all(|&did| did >> 24 == 0x2A),
        "SoundInfo DIDs"
    );

    let sink = RecordingSink::new();
    let mut audio = ac2_client::sound_manager(Arc::clone(&store), Box::new(sink.clone()));
    ac2_client::play_ui_sounds(&mut audio, &sounds);
    assert!(sink.commands().is_empty(), "the frame only queues the play");
    assert!(audio.wait_for_loads(Duration::from_secs(30)));
    audio.use_time(0.1);
    let started: Vec<_> = sink
        .commands()
        .into_iter()
        .filter_map(|command| match command {
            SinkCommand::Start {
                pcm,
                looping,
                millibels,
                frequency,
                ..
            } => Some((pcm, looping, millibels, frequency)),
            _ => None,
        })
        .collect();
    assert_eq!(started.len(), sounds.len(), "every button sound plays");
    let (pcm, looping, millibels, frequency) = &started[0];
    assert!(!looping, "a click is a one-shot");
    assert_eq!(*millibels, 0, "full volume on the main channel");
    assert_eq!(*frequency, pcm.rate, "at the wave's own rate");
    assert!(pcm.frames() > 0 && pcm.samples.iter().any(|&s| s != 0.0));
}

/// Walking plays the human's step sounds: the walk animation's effect triggers raise FX
/// 0x40000002 / 0x40000003, whose sound nodes `FXManager::DoSoundFX` plays with the player as
/// emitter, heard from the camera behind it.
#[test]
fn walking_plays_step_sounds_from_the_player() {
    use ac2_weenie::{ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_FORWARD};
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let world = ac2_client::DatWorld::new(Arc::clone(&store)).unwrap();
    let spawn = legions_server_double::Spawn {
        cell: 0x0299_0109,
        origin: [59.060577, 240.199, -44.606205],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = legions_server_double::double(spawn);
    let config = ac2_weenie::OnlineConfig {
        server: legions_server_double::addr(legions_server_double::SERVER),
        account: "local".into(),
        credential: ac2_net::Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: ac2_weenie::CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    };
    let session = ac2_weenie::OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let mut client = ac2_client::Client::new(session, Arc::new(world));
    let sink = RecordingSink::new();
    client.attach_audio(ac2_client::sound_manager(
        Arc::clone(&store),
        Box::new(sink.clone()),
    ));
    client.set_load_budget(Duration::from_secs(30));
    let idle = ac2_client::FrameInput::default();
    let mut frames = 0;
    while !client.frame().in_world {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += 1.0 / 64.0;
        frames += 1;
        assert!(frames < 600);
    }
    let walk = ac2_client::FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..ac2_client::FrameInput::default()
    };
    for _ in 0..(3 * 64) {
        client.use_time(&walk).unwrap();
        // The step sounds load on the sound loader thread; take them as they finish.
        client
            .audio_mut()
            .unwrap()
            .wait_for_loads(Duration::from_secs(30));
        server.borrow_mut().elapsed += 1.0 / 64.0;
    }
    let placed: Vec<_> = sink
        .commands()
        .into_iter()
        .filter_map(|command| match command {
            SinkCommand::Start { spatial, .. } => Some(spatial),
            _ => None,
        })
        .collect();
    eprintln!("{} sounds started: {placed:?}", placed.len());
    assert!(placed.len() >= 2, "the steps sound");
    let spatial = placed
        .iter()
        .flatten()
        .next()
        .expect("the step sounds are 3D");
    let distance = spatial.position.iter().map(|v| v * v).sum::<f32>().sqrt();
    assert!(
        distance > 0.5 && distance < 20.0,
        "heard from the camera behind the player ({spatial:?})"
    );
}
