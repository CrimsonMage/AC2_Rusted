use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ac2_core::{DataId, QualifiedDataId};
use ac2_db::ResourcePackedDecode;

use crate::*;

const WAIT: Duration = Duration::from_secs(5);

/// Sounds made up in the test: a second of silence-free PCM per `SoundInfo`.
struct Fixture(HashMap<u32, LoadedSound>);

impl SoundSource for Fixture {
    fn load(&self, did: u32) -> Option<LoadedSound> {
        self.0.get(&did).cloned()
    }
}

fn sound(did: u32, bit_field: u32, priority: f32, rate: u32, seconds: f32) -> LoadedSound {
    let frames = (rate as f32 * seconds) as usize;
    LoadedSound::Sound {
        info: SoundInfo {
            did,
            bit_field,
            rand_freq_top: -1.0,
            rand_freq_bot: -1.0,
            priority,
            min_distance: 1.0,
            wave: 0x0C00_0000 | (did & 0xFFFF),
        },
        pcm: Arc::new(Pcm {
            rate,
            channels: 1,
            samples: vec![0.25; frames].into(),
        }),
        length: seconds,
    }
}

fn manager(sounds: Vec<LoadedSound>) -> (SoundManager, RecordingSink) {
    let map = sounds
        .into_iter()
        .map(|s| match &s {
            LoadedSound::Sound { info, .. } => (info.did, s),
            LoadedSound::Music(m) => (m.did, s),
        })
        .collect();
    let sink = RecordingSink::new();
    let manager = SoundManager::new(Box::new(sink.clone()), Arc::new(Fixture(map)));
    (manager, sink)
}

fn starts(sink: &RecordingSink) -> Vec<(u32, i32, u32, bool)> {
    sink.commands()
        .into_iter()
        .filter_map(|c| match c {
            SinkCommand::Start {
                voice,
                millibels,
                frequency,
                looping,
                ..
            } => Some((voice, millibels, frequency, looping)),
            _ => None,
        })
        .collect()
}

#[test]
fn attenuation_follows_direct_sound() {
    assert_eq!(millibels(1.0, 1.0), 0);
    assert_eq!(millibels(0.5, 1.0), -1250);
    assert_eq!(millibels(1.0, 0.5), -1250);
    assert_eq!(millibels(1.0, 2.0), 0, "the volume is clamped to 1");
    assert_eq!(millibels(0.0, 1.0), SILENT);
    assert_eq!(millibels(1.0, 0.0), SILENT);
    assert_eq!(amplitude(0), 1.0);
    assert!((amplitude(-2000) - 0.1).abs() < 1e-6);
    assert_eq!(amplitude(SILENT), 0.0);
}

#[test]
fn records_decode() {
    let qdid = |did| QualifiedDataId::new(DataId::new(did), 0);
    let mut info = vec![];
    for word in [0x2A00_0001u32, 0x14] {
        info.extend(word.to_le_bytes());
    }
    for float in [-1.0f32, -1.0, 0.1, 3.0] {
        info.extend(float.to_le_bytes());
    }
    info.extend(0x0C00_0002u32.to_le_bytes());
    let decoded = SoundInfo::decode_packed(qdid(0x2A00_0001), 0, &info).unwrap();
    assert_eq!(decoded.wave, 0x0C00_0002);
    assert!(decoded.test(flags::THREE_D) && decoded.test(flags::NO_VIRTUAL));
    assert!(SoundInfo::decode_packed(qdid(0x2A00_0002), 0, &info).is_err());

    // A 16-bit mono PCM wave at 8000 Hz, four frames.
    let mut wave = vec![];
    for word in [0x0C00_0002u32, 18, 8] {
        wave.extend(word.to_le_bytes());
    }
    for half in [1u16, 1] {
        wave.extend(half.to_le_bytes());
    }
    for word in [8000u32, 16000] {
        wave.extend(word.to_le_bytes());
    }
    for half in [2u16, 16, 0] {
        wave.extend(half.to_le_bytes());
    }
    for sample in [0i16, 16384, -16384, 32767] {
        wave.extend(sample.to_le_bytes());
    }
    let decoded = Wave::decode_packed(qdid(0x0C00_0002), 0, &wave).unwrap();
    assert_eq!(decoded.pcm.rate, 8000);
    assert_eq!(&decoded.pcm.samples[..3], &[0.0, 0.5, -0.5]);
    assert_eq!(decoded.time(), 4.0 / 8000.0);
}

#[test]
fn a_sound_loads_then_plays_and_ends() {
    let did = 0x2A00_0010;
    let (mut manager, sink) = manager(vec![sound(did, 0, 1.0, 22050, 1.0)]);
    let handle = manager.play(&PlaySoundData::new(did)).unwrap();
    assert!(handle != 0);
    assert!(sink.commands().is_empty(), "nothing plays before it loads");
    assert!(!manager.has_sound_finished(handle));
    assert!(manager.wait_for_loads(WAIT));
    manager.use_time(0.1);
    assert_eq!(starts(&sink), vec![(handle, 0, 22050, false)]);
    // Resident now: a second play needs no load.
    manager.use_time(0.5);
    let again = manager.play(&PlaySoundData::new(did)).unwrap();
    manager.use_time(0.5);
    assert_eq!(starts(&sink).len(), 2);
    // The first ends after its length and half a second.
    manager.use_time(1.6);
    assert!(sink
        .commands()
        .contains(&SinkCommand::Stop { voice: handle }));
    manager.use_time(1.7);
    assert!(manager.has_sound_finished(handle));
    assert!(!manager.has_sound_finished(again));
}

#[test]
fn exclusive_sounds_play_once_at_a_time() {
    let did = 0x2A00_0011;
    let (mut manager, sink) = manager(vec![sound(did, flags::EXCLUSIVE, 1.0, 11025, 2.0)]);
    manager.play(&PlaySoundData::new(did)).unwrap();
    manager.wait_for_loads(WAIT);
    manager.use_time(1.0);
    assert_eq!(manager.play(&PlaySoundData::new(did)), Some(0));
    manager.use_time(1.1);
    assert_eq!(starts(&sink).len(), 1);
}

#[test]
fn the_budget_limits_one_shots() {
    let sounds: Vec<_> = (0..10)
        .map(|i| sound(0x2A00_0100 + i, 0, 1.0, 22050, 1.0))
        .collect();
    let (mut manager, sink) = manager(sounds);
    manager.set_preference(SoundPreference::NumPlayingSounds(8.0));
    for i in 0..10 {
        manager.play(&PlaySoundData::new(0x2A00_0100 + i)).unwrap();
    }
    manager.wait_for_loads(WAIT);
    manager.use_time(1.0);
    assert_eq!(starts(&sink).len(), 8);
    // A full budget refuses more one-shots (`CheckSoundBudget`).
    assert_eq!(manager.play(&PlaySoundData::new(0x2A00_0100)), None);
}

#[test]
fn volumes_and_switches_follow_the_preferences() {
    let did = 0x2A00_0012;
    let (mut manager, sink) = manager(vec![sound(did, flags::LOOP, 1.0, 22050, 1.0)]);
    let handle = manager.play(&PlaySoundData::new(did)).unwrap();
    manager.wait_for_loads(WAIT);
    manager.use_time(1.0);
    assert_eq!(starts(&sink), vec![(handle, 0, 22050, true)]);
    manager.set_preference(SoundPreference::SoundVolume(0.5));
    assert!(sink.commands().contains(&SinkCommand::Volume {
        voice: handle,
        millibels: -1250,
        ramp_ms: 0,
    }));
    assert!(!manager.set_channel_volume(Channel::Main, 1.5));
    // Music off refuses the music channels.
    manager.set_preference(SoundPreference::DisableMusic(true));
    let mut music = PlaySoundData::new(did);
    music.channel = Channel::Music;
    assert_eq!(manager.play(&music), None);
    // Sound off stops the voice and keeps the loop for when it comes back.
    manager.set_preference(SoundPreference::DisableSound(true));
    assert_eq!(
        sink.commands().last(),
        Some(&SinkCommand::Stop { voice: handle })
    );
    manager.set_preference(SoundPreference::DisableSound(false));
    assert_eq!(starts(&sink).len(), 2);
}

#[test]
fn an_emitters_sounds_stop_with_it() {
    let did = 0x2A00_0013;
    let (mut manager, sink) = manager(vec![sound(did, flags::LOOP, 1.0, 22050, 1.0)]);
    let mut data = PlaySoundData::new(did);
    data.emitter = Some(7);
    let handle = manager.play(&data).unwrap();
    manager.wait_for_loads(WAIT);
    manager.use_time(1.0);
    assert_eq!(manager.stop_sounds_for_emitter(7), 1);
    assert!(sink
        .commands()
        .contains(&SinkCommand::Stop { voice: handle }));
    manager.use_time(1.1);
    assert!(manager.has_sound_finished(handle));
}

#[test]
fn the_mixer_sums_resampled_voices() {
    let mut mixer = Mixer::new(100);
    mixer.apply(SinkCommand::Start {
        voice: 1,
        pcm: Arc::new(Pcm {
            rate: 50,
            channels: 1,
            samples: vec![0.5; 4].into(),
        }),
        looping: false,
        millibels: 0,
        frequency: 50,
        spatial: None,
    });
    let mut out = vec![0.0; 2 * 10];
    mixer.render(&mut out, 2);
    // Four source frames at half the output rate fill eight output frames.
    assert!(out[..16].iter().all(|&s| (s - 0.5).abs() < 1e-6), "{out:?}");
    assert!(out[16..].iter().all(|&s| s == 0.0));
    assert_eq!(mixer.voices(), 0);
}

#[test]
fn the_dats_sounds_decode() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let sounds = DatSounds::new(Arc::new(ac2_db::ResourceStore::new(dataset)));
    // A toolbar button's click: a `SoundInfo` naming a PCM wave.
    let Some(LoadedSound::Sound { info, pcm, length }) = sounds.load(0x2A00_020F) else {
        panic!("the click sound");
    };
    assert_eq!(info.wave >> 24, 0x0C);
    assert!(pcm.rate >= 8000 && pcm.frames() > 0 && length > 0.0);
    assert!((f64::from(length) - pcm.frames() as f64 / f64::from(pcm.rate)).abs() < 1e-3);
    assert!(matches!(
        sounds.load(0x2C00_004A),
        Some(LoadedSound::Music(MusicInfo { segment, .. })) if segment >> 24 == 0x24
    ));
    assert_eq!(
        sounds.load(0x0C00_0000),
        None,
        "a wave is not a sound to play"
    );
}

fn spatials(sink: &RecordingSink) -> Vec<(u32, Option<Spatial>)> {
    sink.commands()
        .into_iter()
        .filter_map(|c| match c {
            SinkCommand::Start { voice, spatial, .. } => Some((voice, spatial)),
            SinkCommand::Spatial { voice, spatial } => Some((voice, Some(spatial))),
            _ => None,
        })
        .collect()
}

fn at(offset: [f32; 3]) -> Option<Emitter> {
    Some(Emitter {
        offset,
        velocity: [0.0; 3],
        muted: false,
    })
}

#[test]
fn a_3d_sound_follows_its_emitter() {
    let did = 0x2A00_0020;
    let mut looped = sound(did, flags::LOOP | flags::THREE_D, 1.0, 22050, 1.0);
    if let LoadedSound::Sound { info, .. } = &mut looped {
        info.min_distance = 3.0;
    }
    let (mut manager, sink) = manager(vec![looped]);
    let mut data = PlaySoundData::new(did);
    data.emitter = Some(9);
    let handle = manager.play(&data).unwrap();
    manager.wait_for_loads(WAIT);
    // Ten metres ahead of the listener (AC2 y), then two to its right (x) and one up (z).
    manager.use_time_with(0.1, &|id| (id == 9).then(|| at([0.0, 10.0, 0.0])).flatten());
    manager.use_time_with(0.2, &|id| (id == 9).then(|| at([2.0, 0.0, 1.0])).flatten());
    let spatial = |position| Spatial {
        position,
        velocity: [0.0; 3],
        min_distance: 3.0,
    };
    assert_eq!(
        spatials(&sink),
        vec![
            (handle, Some(spatial([0.0, 0.0, 10.0]))),
            (handle, Some(spatial([2.0, 1.0, 0.0]))),
        ],
        "DirectSound axes: y up, z ahead"
    );
    // A muted emitter stops its loop, which plays again once it is heard.
    manager.use_time_with(0.3, &|_| {
        Some(Emitter {
            muted: true,
            ..Emitter::default()
        })
    });
    assert_eq!(
        sink.commands().last(),
        Some(&SinkCommand::Stop { voice: handle })
    );
    manager.use_time_with(0.4, &|_| at([0.0, 1.0, 0.0]));
    assert_eq!(starts(&sink).len(), 2);
}

#[test]
fn distance_sets_the_priority_and_far_sounds_wait() {
    let did = 0x2A00_0021;
    let (mut manager, sink) = manager(vec![sound(
        did,
        flags::LOOP | flags::THREE_D,
        1.0,
        22050,
        1.0,
    )]);
    let mut data = PlaySoundData::new(did);
    data.emitter = Some(4);
    manager.play(&data).unwrap();
    manager.wait_for_loads(WAIT);
    // `(2 - priority) * distance` of 300 is past `AllocateSounds`' 250.
    manager.use_time_with(0.1, &|_| at([0.0, 300.0, 0.0]));
    assert!(starts(&sink).is_empty(), "too far to be given a voice");
    manager.use_time_with(0.2, &|_| at([0.0, 20.0, 0.0]));
    assert_eq!(starts(&sink).len(), 1);
}

#[test]
fn a_2d_sound_with_an_emitter_is_not_placed() {
    let did = 0x2A00_0022;
    let (mut manager, sink) = manager(vec![sound(did, 0, 1.0, 22050, 1.0)]);
    let mut data = PlaySoundData::new(did);
    data.emitter = Some(1);
    manager.play(&data).unwrap();
    manager.wait_for_loads(WAIT);
    manager.use_time_with(0.1, &|_| at([5.0, 5.0, 0.0]));
    assert_eq!(spatials(&sink), vec![(1, None)]);
}

#[test]
fn spatial_gains_roll_off_and_pan() {
    let gains = |position| {
        spatial_gains(&Spatial {
            position,
            velocity: [0.0; 3],
            min_distance: 2.0,
        })
    };
    let [l, r] = gains([0.0, 0.0, 1.0]);
    assert!(
        (l - 1.0).abs() < 1e-6 && (r - 1.0).abs() < 1e-6,
        "ahead, inside"
    );
    let [l, r] = gains([0.0, 0.0, 4.0]);
    assert!(
        (l - 0.5).abs() < 1e-6 && (r - 0.5).abs() < 1e-6,
        "twice the minimum"
    );
    let [l, r] = gains([3.0, 0.0, 0.0]);
    assert!(l.abs() < 1e-6 && r > 0.9, "to the right");
}
