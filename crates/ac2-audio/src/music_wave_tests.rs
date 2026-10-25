//! A `MusicInfo` whose object is an `EncodedWave` (retail's `DirectXMusicLoader::LoadFromObject`
//! for `0x61`): the wave plays on the music channel with the `MusicInfo`'s terms. On the DATs
//! (`AC2_DAT_DIR`); nothing from the DATs is stored, and the mix goes to a [`MixingSink`].

use std::sync::Arc;
use std::time::Duration;

use ac2_db::ResourceStore;

use crate::data::{DatSounds, LoadedSound, SoundSource};
use crate::play_data::{Channel, LoopingMode, PlaySoundData, Track};
use crate::{MixingSink, SoundManager, SoundPreference};

const RATE: u32 = 44_100;
const WAIT: Duration = Duration::from_secs(30);
/// A `MusicInfo` whose segment is an `EncodedWave` (`0x61`, the local DAT's `EncodedWave`).
const ENCODED_WAVE_MUSIC: u32 = 0x2C00_0072;

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

fn manager(store: &Arc<ResourceStore>) -> (SoundManager, MixingSink) {
    let sink = MixingSink::new(RATE);
    let source = Arc::new(DatSounds::new(Arc::clone(store)));
    (SoundManager::new(Box::new(sink.clone()), source), sink)
}

/// `seconds` of the mix, stereo, rendered in 10 ms steps (the frame thread's `use_time`, then the
/// callback's render).
fn run(manager: &mut SoundManager, sink: &MixingSink, seconds: f64) -> Vec<f32> {
    let blocks = (seconds / 0.01).round() as usize;
    let mut out = Vec::with_capacity(blocks * 441 * 2);
    for block in 0..blocks {
        manager.use_time(block as f64 * 0.01);
        out.extend(sink.render(441, 2));
    }
    out
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |p, s| p.max(s.abs()))
}

#[test]
fn an_encoded_wave_music_info_plays_its_wave_on_the_music_channel() {
    let Some(store) = store() else {
        return;
    };
    let (mut manager, sink) = manager(&store);
    let data = PlaySoundData {
        channel: Channel::Music,
        ..PlaySoundData::new(ENCODED_WAVE_MUSIC)
    };
    let handle = manager.play(&data).expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT), "the wave loads");
    let out = run(&mut manager, &sink, 0.5);
    assert!(out.iter().all(|s| s.is_finite()), "the mix is finite");
    assert!(peak(&out) > 0.0001, "the wave is heard");
    assert!(!manager.has_sound_finished(handle), "it is still playing");
    assert_eq!(sink.with_mixer(|m| m.voices()), Some(1), "one wave voice");
}

#[test]
fn stopping_an_encoded_wave_music_info_ends_its_voice() {
    let Some(store) = store() else {
        return;
    };
    let (mut manager, sink) = manager(&store);
    let data = PlaySoundData {
        channel: Channel::Music,
        ..PlaySoundData::new(ENCODED_WAVE_MUSIC)
    };
    let handle = manager.play(&data).expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    assert!(peak(&run(&mut manager, &sink, 0.5)) > 0.0001);
    manager.stop_context(handle, 0.0);
    let tail = run(&mut manager, &sink, 0.2);
    assert!(tail.iter().all(|s| s.is_finite()));
    assert_eq!(peak(&tail), 0.0, "silent once stopped");
    assert!(manager.has_sound_finished(handle));
}

#[test]
fn music_volume_zero_silences_an_encoded_wave_music_info() {
    let Some(store) = store() else {
        return;
    };
    let (mut manager, sink) = manager(&store);
    manager.set_preference(SoundPreference::MusicVolume(0.0));
    let data = PlaySoundData {
        channel: Channel::Music,
        ..PlaySoundData::new(ENCODED_WAVE_MUSIC)
    };
    manager.play(&data).expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    assert_eq!(peak(&run(&mut manager, &sink, 1.0)), 0.0, "no music volume");
}

/// A primary segment that does not loop (the `MusicInfo` is an `EncodedWave` on the queue's
/// primary track), so its pass ends and the queue takes over.
fn primary() -> PlaySoundData {
    PlaySoundData {
        channel: Channel::Music,
        track: Track::ForcePrimary,
        looping: LoopingMode::ForceOff,
        ..PlaySoundData::new(ENCODED_WAVE_MUSIC)
    }
}

/// The wave's time, from the loader (`DBWave::GetTime`), read on the test's own thread.
fn wave_seconds(store: &Arc<ResourceStore>) -> f64 {
    match DatSounds::new(Arc::clone(store)).load(ENCODED_WAVE_MUSIC) {
        Some(LoadedSound::MusicWave { length, .. }) => f64::from(length),
        _ => panic!("the wave loads"),
    }
}

/// One block of the frame thread and the callback: `seconds` of the mix, in 10 ms steps.
fn step(manager: &mut SoundManager, sink: &MixingSink, now: &mut f64) {
    manager.use_time(*now);
    sink.render(441, 2);
    *now += 0.01;
}

#[test]
fn a_primary_wave_waits_behind_the_one_playing() {
    let Some(store) = store() else {
        return;
    };
    let (mut manager, sink) = manager(&store);
    let a = manager
        .play(&primary())
        .expect("the first wave is requested");
    assert!(manager.wait_for_loads(WAIT));
    let mut now = 0.0;
    step(&mut manager, &sink, &mut now);
    let b = manager
        .play(&primary())
        .expect("the second wave is requested");
    let mut a_ended_at = None;
    for _ in 0..(120 * 100) {
        step(&mut manager, &sink, &mut now);
        assert!(
            sink.with_mixer(|m| m.voices()).unwrap_or(0) <= 1,
            "one wave voice at a time"
        );
        if manager.music().is_playing(a) {
            assert!(!manager.music().is_playing(b), "B waits while A plays");
        } else if a_ended_at.is_none() {
            a_ended_at = Some(now);
            assert!(
                manager.music().is_playing(b),
                "B takes over when A's pass ends"
            );
            break;
        }
    }
    assert!(a_ended_at.is_some(), "A's pass ends within two minutes");
    assert!(!manager.has_sound_finished(b), "B plays");
}

#[test]
fn a_primary_wave_repeats_on_one_voice_until_stopped() {
    let Some(store) = store() else {
        return;
    };
    let seconds = wave_seconds(&store);
    let (mut manager, sink) = manager(&store);
    let a = manager.play(&primary()).expect("the wave is requested");
    assert!(manager.wait_for_loads(WAIT));
    let mut now = 0.0;
    // Two and a half passes (at most two minutes): the repeat starts each pass on the queue.
    let blocks = ((seconds * 2.5).min(120.0) / 0.01) as usize;
    let mut passes = 0;
    let mut was_sounding = false;
    for _ in 0..blocks {
        step(&mut manager, &sink, &mut now);
        assert!(manager.music().is_playing(a), "the repeat keeps the handle");
        let voices = sink.with_mixer(|m| m.voices()).unwrap_or(0);
        assert!(voices <= 1, "one wave voice");
        if was_sounding && voices == 0 {
            passes += 1;
        }
        was_sounding = voices == 1;
    }
    if seconds * 2.5 <= 120.0 {
        assert!(passes >= 1, "the first pass ends and the repeat starts");
    }
    manager.stop_context(a, 0.0);
    for _ in 0..30 {
        step(&mut manager, &sink, &mut now);
    }
    assert!(manager.has_sound_finished(a), "the stop ends the repeat");
}

#[test]
fn a_faded_primary_wave_plays_out_silently_and_the_next_takes_over() {
    // The fade ramps A's volume to zero; its end is `Stop(handle, 0)`, which plays the current
    // pass out (`DXMusicMessageThread::Stop` only clears the loop), so B takes over at A's end.
    let Some(store) = store() else {
        return;
    };
    let seconds = wave_seconds(&store);
    let (mut manager, sink) = manager(&store);
    let a = manager
        .play(&primary())
        .expect("the first wave is requested");
    assert!(manager.wait_for_loads(WAIT));
    let mut now = 0.0;
    for _ in 0..10 {
        step(&mut manager, &sink, &mut now);
    }
    let b = manager
        .play(&primary())
        .expect("the second wave is requested");
    manager.stop_context(a, 2.0);
    let mut tail = 0.0f32;
    let blocks = ((seconds.min(120.0) + 2.0) / 0.01) as usize;
    for block in 0..blocks {
        manager.use_time(now);
        let out = sink.render(441, 2);
        now += 0.01;
        if (block as f64) * 0.01 > 2.2 && (block as f64) * 0.01 < 2.8 {
            tail = tail.max(peak(&out));
        }
        if block == 150 {
            assert!(
                manager.music().holds(a),
                "still in the queue during the fade"
            );
            assert!(manager.music().is_playing(a), "its pass plays out");
            assert!(!manager.music().is_playing(b), "B waits");
        }
        if !manager.music().is_playing(a) {
            break;
        }
    }
    assert!(tail < 0.0001, "silent once the fade ends: {tail}");
    assert!(
        now >= seconds.min(120.0) * 0.5,
        "A's pass played out (its length {seconds} s)"
    );
    assert!(
        manager.music().is_playing(b),
        "B takes over at A's pass end"
    );
    assert!(manager.has_sound_finished(a), "A is released");
}
