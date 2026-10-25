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
