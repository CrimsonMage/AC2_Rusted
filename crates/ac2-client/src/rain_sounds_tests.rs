//! The rain and thunder sounds on the real DATs (`AC2_DAT_DIR`): which DIDs each amount queues,
//! and the loop's start, volume and stop.

use std::sync::Arc;
use std::time::Duration;

use ac2_audio::{RecordingSink, SinkCommand};
use ac2_core::Random;

use super::{rain_sound_ids, RainSounds};

#[test]
fn the_rain_loop_follows_the_amount_and_thunder_joins_above_point_four() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let ids = rain_sound_ids(&store);
    assert!(
        ids.rain_loop != 0 && ids.thunder.iter().all(|&did| did != 0),
        "the DAT names the rain sounds: {ids:?}"
    );

    let sink = RecordingSink::new();
    let mut manager = crate::sound_manager(Arc::clone(&store), Box::new(sink.clone()));
    let mut rain = RainSounds::default();
    rain.set_ids(ids);
    let mut random = Random::default();

    // Dry: nothing plays.
    assert!(rain
        .update(0.0, true, 1.0, &mut random, &mut manager)
        .is_empty());

    // Rain at 0.5 starts the loop; above 0.4 the first thunder comes at once (its timer is 0).
    let queued = rain.update(0.5, true, 10.0, &mut random, &mut manager);
    assert_eq!(
        queued.last(),
        Some(&ids.rain_loop),
        "the loop starts: {queued:?}"
    );
    assert_eq!(queued.len(), 2, "one thunder and the loop: {queued:?}");
    assert!(ids.thunder.contains(&queued[0]), "the first is a thunder");

    // The sounds reach the sink once loaded: the thunder and the loop start.
    assert!(manager.wait_for_loads(Duration::from_secs(30)));
    manager.use_time(0.1);
    let starts = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Start { .. }))
        .count();
    assert_eq!(starts, 2, "the thunder and the loop start");

    // The same amount later: no new loop, and the thunder timer (15 to 60 s) has not run out.
    assert!(rain
        .update(0.5, true, 11.0, &mut random, &mut manager)
        .is_empty());

    // A heavier amount only moves the loop's volume; a lighter one still under 0.4 queues nothing.
    assert!(rain
        .update(0.8, true, 12.0, &mut random, &mut manager)
        .is_empty());
    assert!(rain
        .update(0.2, true, 13.0, &mut random, &mut manager)
        .is_empty());
    manager.use_time(0.1);

    // Indoors (the camera cannot see outside) the amount is 0: the loop stops.
    assert!(rain
        .update(0.9, false, 14.0, &mut random, &mut manager)
        .is_empty());
    manager.use_time(0.1);
    let stops = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Stop { .. }))
        .count();
    assert!(stops >= 1, "the loop stops when the amount is 0");

    // Back in the rain, a thunder comes once its timer (set at 10 s, at most 70 s) has run out.
    let queued = rain.update(0.9, true, 100.0, &mut random, &mut manager);
    assert!(queued.iter().any(|did| ids.thunder.contains(did)));
    assert_eq!(queued.last(), Some(&ids.rain_loop));
}

#[test]
fn no_sound_ids_queue_nothing() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let sink = RecordingSink::new();
    let mut manager = crate::sound_manager(Arc::clone(&store), Box::new(sink));
    let mut silent = RainSounds::default();
    let mut random = Random::default();
    assert!(silent
        .update(0.9, true, 10.0, &mut random, &mut manager)
        .is_empty());
}
