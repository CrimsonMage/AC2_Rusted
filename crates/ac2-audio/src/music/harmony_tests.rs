//! The harmony of a segment: its chord track in effect at each tick, the subchord by level, the
//! scale fallback, and the key from `crdh`.

use super::harmony::{Harmony, DEFAULT_SCALE, DEFAULT_SUBCHORD};
use super::riff::Guid;
use super::segment::{Chord, Segment, SubChord, Track, TrackData};

/// A subchord with `root` and the default triad and scale.
fn subchord(root: u8) -> SubChord {
    SubChord {
        chord_root: root,
        ..DEFAULT_SUBCHORD
    }
}

/// A segment with one chord track of `crdh` `scale` and chords at the given times.
fn segment_with_chords(scale: u32, chords: Vec<(i32, Vec<SubChord>)>) -> Segment {
    let chords = chords
        .into_iter()
        .map(|(time, subchords)| Chord {
            name: String::new(),
            time,
            measure: 0,
            beat: 0,
            flags: 0,
            subchords,
        })
        .collect();
    Segment {
        guid: None,
        name: None,
        repeats: 0,
        length: 0,
        play_start: 0,
        loop_start: 0,
        loop_end: 0,
        resolution: 0,
        flags: 0,
        tracks: vec![Track {
            class: Guid([0; 16]),
            position: 0,
            group: 0,
            config: 0,
            priority: 0,
            data: TrackData::Chords { scale, chords },
        }],
    }
}

#[test]
fn no_chord_track_is_c_major_everywhere() {
    let segment = Segment {
        tracks: Vec::new(),
        ..segment_with_chords(0, Vec::new())
    };
    let harmony = Harmony::from_segment(&segment);
    assert_eq!(harmony, Harmony::default());
    for tick in [-100, 0, 6144, 1_000_000] {
        let state = harmony.state_at(tick, 0);
        assert_eq!(state.key, 0);
        assert_eq!(state.subchord.chord_root, 0);
        assert_eq!(state.subchord.chord_pattern, 0x91);
        assert_eq!(state.subchord.scale_pattern, DEFAULT_SCALE);
    }
}

#[test]
fn a_chord_applies_from_its_time_and_the_first_applies_before_it() {
    // C at 0, then G at measure 2 (6144 ticks): later ticks take G, and the G starts at its time.
    let harmony = Harmony::new(
        0,
        0xAB5AB5,
        vec![(6144, vec![subchord(7)]), (0, vec![subchord(0)])],
    );
    assert_eq!(harmony.state_at(-5, 0).subchord.chord_root, 0);
    assert_eq!(harmony.state_at(6143, 0).subchord.chord_root, 0);
    assert_eq!(harmony.state_at(6144, 0).subchord.chord_root, 7);
    assert_eq!(harmony.state_at(12_000, 0).subchord.chord_root, 7);
}

#[test]
fn the_subchord_is_the_one_whose_levels_hold_the_part_level() {
    let harmony = Harmony::new(
        0,
        0xAB5AB5,
        vec![(
            0,
            vec![
                SubChord {
                    levels: 0x1,
                    ..subchord(0)
                },
                SubChord {
                    levels: 0x2,
                    ..subchord(4)
                },
            ],
        )],
    );
    assert_eq!(harmony.state_at(0, 0).subchord.chord_root, 0);
    assert_eq!(harmony.state_at(0, 1).subchord.chord_root, 4);
    // No subchord holds level 2: the first one plays.
    assert_eq!(harmony.state_at(0, 2).subchord.chord_root, 0);
}

#[test]
fn a_subchord_without_a_scale_takes_the_track_scale() {
    let harmony = Harmony::new(
        0,
        0x00_0AB6,
        vec![(
            0,
            vec![SubChord {
                scale_pattern: 0,
                ..subchord(0)
            }],
        )],
    );
    assert_eq!(harmony.state_at(0, 0).subchord.scale_pattern, 0xAB6);
    // With no scale in the track either, the major scale.
    let bare = Harmony::new(
        0,
        0,
        vec![(
            0,
            vec![SubChord {
                scale_pattern: 0,
                ..subchord(0)
            }],
        )],
    );
    assert_eq!(bare.state_at(0, 0).subchord.scale_pattern, DEFAULT_SCALE);
}

#[test]
fn the_key_is_the_top_byte_of_crdh() {
    let segment = segment_with_chords(0x02AB_5AB5, vec![(0, vec![subchord(0)])]);
    let harmony = Harmony::from_segment(&segment);
    assert_eq!(harmony.state_at(0, 0).key, 2);
    assert_eq!(harmony.state_at(0, 0).subchord.scale_pattern, 0xAB5);
    // A key past the octave wraps.
    assert_eq!(Harmony::new(14, 0, Vec::new()).state_at(0, 0).key, 2);
}

#[test]
fn a_segment_chord_track_gives_its_chords_in_time() {
    let segment = segment_with_chords(
        0x00AB_5AB5,
        vec![(0, vec![subchord(0)]), (3072, vec![subchord(9)])],
    );
    let harmony = Harmony::from_segment(&segment);
    assert_eq!(harmony.state_at(0, 0).subchord.chord_root, 0);
    assert_eq!(harmony.state_at(3072, 0).subchord.chord_root, 9);
}
