//! The portal space's sounds on the real DATs (`AC2_DAT_DIR`): `StartNow` queues the enter
//! sound and the two tunnel loops, `StopNow` the exit sound and stops the loops.

use std::sync::Arc;
use std::time::Duration;

use ac2_audio::{RecordingSink, SinkCommand};
use ac2_render::portal::PortalSound;

use super::{portal_sound_ids, PortalSounds};

#[test]
fn the_portal_effect_plays_its_enter_loops_and_exit_from_the_dat() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let ids = portal_sound_ids(&store);
    assert!(
        ids.enter != 0 && ids.loop0 != 0 && ids.loop1 != 0 && ids.exit != 0,
        "the DAT names the portal sounds: {ids:?}"
    );

    let sink = RecordingSink::new();
    let mut manager = crate::sound_manager(Arc::clone(&store), Box::new(sink.clone()));
    let mut sounds = PortalSounds::default();
    sounds.set_ids(ids);

    // `StartNow`: the enter sound, then the loops, in that order.
    let queued = sounds.play(&mut manager, &[PortalSound::Start]);
    assert_eq!(queued, vec![ids.enter, ids.loop0, ids.loop1]);
    assert!(manager.wait_for_loads(Duration::from_secs(30)));
    manager.use_time(0.1);
    let voices: Vec<u32> = sink
        .commands()
        .into_iter()
        .filter_map(|command| match command {
            SinkCommand::Start { voice, .. } => Some(voice),
            _ => None,
        })
        .collect();
    assert_eq!(voices.len(), 3, "the enter sound and both loops start");
    let loop_voices = &voices[1..];

    // `StopNow`: the exit sound, and the loops stop (fade 0).
    let queued = sounds.play(&mut manager, &[PortalSound::Stop]);
    assert_eq!(queued, vec![ids.exit]);
    manager.use_time(0.1);
    let stopped: Vec<u32> = sink
        .commands()
        .into_iter()
        .filter_map(|command| match command {
            SinkCommand::Stop { voice } => Some(voice),
            _ => None,
        })
        .collect();
    for voice in loop_voices {
        assert!(stopped.contains(voice), "loop voice {voice} stops");
    }
}

#[test]
fn a_stop_without_a_start_plays_only_the_exit() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let ids = portal_sound_ids(&store);
    let sink = RecordingSink::new();
    let mut manager = crate::sound_manager(Arc::clone(&store), Box::new(sink.clone()));
    let mut sounds = PortalSounds::default();
    sounds.set_ids(ids);
    assert_eq!(
        sounds.play(&mut manager, &[PortalSound::Stop]),
        vec![ids.exit]
    );
    // With no sound ids, nothing is queued.
    let mut silent = PortalSounds::default();
    assert!(silent
        .play(&mut manager, &[PortalSound::Start, PortalSound::Stop])
        .is_empty());
}
