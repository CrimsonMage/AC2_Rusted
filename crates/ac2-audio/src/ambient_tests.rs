use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::*;

const WAIT: Duration = Duration::from_secs(5);
const FOREST: u32 = 0x8100_0010;
const TOWN: u32 = 0x8100_0011;
const FOREST_MUSIC: u32 = 0x2C00_0001;
const TOWN_MUSIC: u32 = 0x2C00_0002;
const ALWAYS_MUSIC: u32 = 0x2C00_0003;

struct Musics;

impl SoundSource for Musics {
    fn load(&self, did: u32) -> Option<LoadedSound> {
        Some(LoadedSound::Music {
            info: MusicInfo {
                did,
                // 4: self-exclusive, as the Legions ambient music is.
                bit_field: 0x44,
                priority: 1.0,
                min_distance: 1.0,
                kind: 0x8000_0001,
                segment: 0x2400_0000 | (did & 0xFFFF),
            },
            segment: None,
            sources: Arc::from(Vec::new()),
            bands: Arc::from(Vec::new()),
            cues: Arc::default(),
        })
    }
}

fn contin(bit_field: u32, sound: u32) -> ContinSoundInfo {
    ContinSoundInfo {
        bit_field,
        sound,
        volume: 1.0,
        min_pitch: 1.0,
        max_pitch: 1.0,
        min_height: 0.0,
        max_height: 0.0,
    }
}

fn data() -> Arc<AmbientData> {
    let info = |sound_type, env, contin: Vec<ContinSoundInfo>| AmbientSoundInfo {
        sound_type,
        env,
        intermit: Vec::new(),
        contin,
    };
    Arc::new(AmbientData::new(
        SoundDesc {
            sounds: vec![
                // Music while dominant (6), and music that plays whenever heard (2 | 8).
                info(
                    FOREST,
                    0x2900_0001,
                    vec![contin(6, FOREST_MUSIC), contin(10, ALWAYS_MUSIC)],
                ),
                info(TOWN, 0x2900_0002, vec![contin(6, TOWN_MUSIC)]),
            ],
        },
        HashMap::new(),
    ))
}

fn started(sink: &RecordingSink) -> Vec<u32> {
    sink.commands()
        .into_iter()
        .filter_map(|c| match c {
            SinkCommand::StartMusic { music, .. } => Some(music),
            _ => None,
        })
        .collect()
}

/// Weighs the cells around (`ChangeCellStart` / `AddCellSound` / `ChangeCellEnd`), then lets
/// the manager load and start what was asked for.
fn update(
    ambient: &mut AmbientSound,
    manager: &mut SoundManager,
    now: f64,
    points: &[(u32, [f32; 3])],
) {
    ambient.change_cell_start();
    for &(sound_type, offset) in points {
        ambient.add_cell_sound(sound_type, offset, true, true);
    }
    ambient.change_cell_end(now, manager);
    manager.wait_for_loads(WAIT);
    manager.use_time(now);
}

#[test]
fn the_densest_environment_is_dominant_and_plays_its_music() {
    let sink = RecordingSink::new();
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(Musics));
    let mut ambient = AmbientSound::default();
    ambient.set_data(Some(data()));
    // Three forest points close by, one town point farther off.
    let forest = [
        (FOREST, [5.0, 0.0, 0.0]),
        (FOREST, [0.0, 6.0, 0.0]),
        (FOREST, [-4.0, 0.0, 0.0]),
    ];
    let mut points = forest.to_vec();
    points.push((TOWN, [30.0, 0.0, 0.0]));
    update(&mut ambient, &mut manager, 1.0, &points);
    assert_eq!(ambient.sound_types()[0], FOREST);
    assert_eq!(ambient.environment(), Some(0x2900_0001));
    let mut playing = started(&sink);
    playing.sort();
    assert_eq!(
        playing,
        vec![FOREST_MUSIC, ALWAYS_MUSIC],
        "the town waits to be dominant"
    );

    // Walk into town: it takes over; the forest's dominant-only music stops at its boundary,
    // the always-on music keeps playing (bit 8).
    let town = [
        (TOWN, [2.0, 0.0, 0.0]),
        (TOWN, [0.0, 3.0, 0.0]),
        (TOWN, [0.0, -3.0, 0.0]),
    ];
    let mut points = town.to_vec();
    points.push((FOREST, [40.0, 0.0, 0.0]));
    update(&mut ambient, &mut manager, 2.0, &points);
    assert_eq!(ambient.environment(), Some(0x2900_0002));
    assert!(started(&sink).contains(&TOWN_MUSIC));
    let stops: Vec<_> = sink
        .commands()
        .into_iter()
        .filter(|c| matches!(c, SinkCommand::StopMusic { .. }))
        .collect();
    assert_eq!(stops.len(), 1, "{stops:?}");
}

#[test]
fn far_points_add_nothing_and_no_data_disables() {
    let mut ambient = AmbientSound::default();
    assert!(!ambient.add_cell_sound(FOREST, [1.0, 0.0, 0.0], true, true));
    assert!(
        !ambient.enabled(),
        "no ambient sound information, disabling"
    );
    let mut ambient = AmbientSound::default();
    ambient.set_data(Some(data()));
    assert!(!ambient.add_cell_sound(FOREST, [101.0, 0.0, 0.0], true, true));
    assert!(ambient.add_cell_sound(FOREST, [99.0, 0.0, 0.0], true, true));
}

#[test]
fn silent_environments_time_out() {
    let sink = RecordingSink::new();
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(Musics));
    let mut ambient = AmbientSound::default();
    ambient.set_data(Some(data()));
    update(
        &mut ambient,
        &mut manager,
        1.0,
        &[(FOREST, [1.0, 0.0, 0.0])],
    );
    // Nothing heard: the forest falls below its minimum density and goes.
    update(&mut ambient, &mut manager, 2.0, &[]);
    ambient.use_time(3.0, &mut manager);
    assert!(ambient.sound_types().is_empty());
}

#[test]
fn the_region_sound_desc_decodes() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let data = AmbientData::load(&store, 0x1000_0000)
        .unwrap()
        .expect("CSoundDesc");
    assert_eq!(data.desc.sounds.len(), 62);
    for info in &data.desc.sounds {
        assert_eq!(info.env >> 24, 0x29, "an EnvInfo");
        for contin in &info.contin {
            // Every Legions ambient sound is dominant-only music.
            assert!(contin.test(2) && contin.test(4), "{contin:?}");
            assert_eq!(contin.sound >> 24, 0x2C);
            assert!(!data.is_3d(contin.sound));
        }
    }
}

const CHIMES: u32 = 0x2C00_0004;

/// A quarter-second sound for the intermittent chimes. Retail's `IsPlaying` holds a fire back
/// while the last sound's voice is still in the sound manager's list; a voice (its length is the
/// wave's time plus half a second) is released within the cadence, so no wait is held back.
struct Chimes;

impl SoundSource for Chimes {
    fn load(&self, did: u32) -> Option<LoadedSound> {
        Some(LoadedSound::Sound {
            info: SoundInfo {
                did,
                bit_field: 0,
                rand_freq_top: -1.0,
                rand_freq_bot: -1.0,
                priority: 1.0,
                min_distance: 1.0,
                wave: 0,
            },
            pcm: Arc::new(Pcm {
                rate: 8000,
                channels: 1,
                samples: vec![0.0; 2000].into(),
            }),
            length: 0.25,
        })
    }
}

fn intermit(sound: u32, min_time: f32, max_time: f32) -> IntermitSoundInfo {
    IntermitSoundInfo {
        bit_field: 2,
        sound,
        volume: 1.0,
        probability: 1.0,
        min_pitch: 1.0,
        max_pitch: 1.0,
        min_height: 0.0,
        max_height: 0.0,
        min_time,
        max_time,
    }
}

/// `FOREST` with one intermittent sound whose wait is 2 to 4 seconds (its probability 1).
fn chimes() -> Arc<AmbientData> {
    Arc::new(AmbientData::new(
        SoundDesc {
            sounds: vec![AmbientSoundInfo {
                sound_type: FOREST,
                env: 0x2900_0001,
                intermit: vec![intermit(CHIMES, 2.0, 4.0)],
                contin: Vec::new(),
            }],
        },
        HashMap::new(),
    ))
}

/// The sounds started so far.
fn starts(sink: &RecordingSink) -> usize {
    sink.commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Start { .. }))
        .count()
}

/// The times (seconds) the intermittent sound starts over `seconds`, with the cells weighed
/// once at 1 s and `AmbientSound::use_time` every half second after.
fn chime_times(seconds: f64) -> Vec<f64> {
    let sink = RecordingSink::new();
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(Chimes));
    let mut ambient = AmbientSound::default();
    ambient.set_data(Some(chimes()));
    update(
        &mut ambient,
        &mut manager,
        1.0,
        &[(FOREST, [5.0, 0.0, 0.0])],
    );
    let mut times = Vec::new();
    let mut fired = 0;
    for step in 1..=(seconds / 0.5) as usize {
        let now = 1.0 + step as f64 * 0.5;
        ambient.use_time(now, &mut manager);
        manager.wait_for_loads(WAIT);
        manager.use_time(now);
        while fired < starts(&sink) {
            times.push(now);
            fired += 1;
        }
    }
    times
}

#[test]
fn the_intermittent_sound_fires_on_its_timer_within_its_wait_and_repeats_exactly() {
    let times = chime_times(60.0);
    assert!(times.len() >= 10, "it fires again and again: {times:?}");
    // Each wait is `min_time` to `max_time` seconds, read at the half-second cadence.
    for pair in times.windows(2) {
        let gap = pair[1] - pair[0];
        assert!((2.0..=4.5 + 1e-9).contains(&gap), "gap {gap} in {times:?}");
    }
    // Retail's `Random::rand` stream is fixed (its state is never reseeded), so the same
    // weighing gives the same schedule.
    assert_eq!(times, chime_times(60.0));
}
