//! `wMusicValue` decoding on hand-built values: FIXED, the chord tones, scale steps, accidentals,
//! octaves, the play-mode resolution, and the MIDI range over every value.

use super::harmony::{ChordState, DEFAULT_SUBCHORD};
use super::music_value::{effective_play_mode, midi_note, MusicValue};
use super::segment::SubChord;

const FIXED: u8 = 0x00;
const FIXED_TO_KEY: u8 = 0x01;
const FIXED_TO_CHORD: u8 = 0x02;
const MELODIC: u8 = 0x06;
const PEDALPOINT: u8 = 0x05;
const NORMAL_CHORD: u8 = 0x0E;

/// C major over key C: the default chord.
fn c_major() -> ChordState {
    ChordState {
        key: 0,
        subchord: DEFAULT_SUBCHORD,
    }
}

/// C major over key `key`, its chord root at `root` (semitones above the key).
fn triad(key: i32, root: u8) -> ChordState {
    ChordState {
        key,
        subchord: SubChord {
            chord_root: root,
            ..DEFAULT_SUBCHORD
        },
    }
}

/// A `wMusicValue` from its nibbles: octave, chord tone, scale offset and accidental.
fn mv(octave: i32, tone: u16, scale: u16, accidental: i32) -> u16 {
    (((octave & 0xF) as u16) << 12)
        | ((tone & 0xF) << 8)
        | ((scale & 0xF) << 4)
        | ((accidental & 0xF) as u16)
}

#[test]
fn fixed_passes_the_midi_value() {
    assert_eq!(midi_note(67, FIXED, &c_major()), 67);
    // Only the low seven bits are a MIDI note; the chord does not move it.
    assert_eq!(midi_note(0x1234, FIXED, &c_major()), 0x34);
    assert_eq!(midi_note(0x0043, FIXED, &triad(0, 7)), 67);
}

#[test]
fn normal_chord_maps_chord_tones_to_the_chord() {
    // C major at the middle C root: tones 0, 1, 2 are C, E, G; tone 3 wraps up an octave.
    let state = c_major();
    let notes: Vec<u8> = (0..6)
        .map(|tone| midi_note(mv(0, tone, 0, 0), NORMAL_CHORD, &state))
        .collect();
    assert_eq!(notes, vec![60, 64, 67, 72, 76, 79]);
}

#[test]
fn the_chord_root_sets_the_anchor() {
    // G major (root 7): tones 0, 1, 2 are G, B, D.
    let state = triad(0, 7);
    assert_eq!(midi_note(mv(0, 0, 0, 0), NORMAL_CHORD, &state), 67);
    assert_eq!(midi_note(mv(0, 1, 0, 0), NORMAL_CHORD, &state), 71);
    assert_eq!(midi_note(mv(0, 2, 0, 0), NORMAL_CHORD, &state), 74);
}

#[test]
fn a_scale_offset_steps_along_the_scale() {
    // From C (tone 0): one step is D, two E, three F; from G (tone 2), one step is A.
    let state = c_major();
    assert_eq!(midi_note(mv(0, 0, 1, 0), NORMAL_CHORD, &state), 62);
    assert_eq!(midi_note(mv(0, 0, 2, 0), NORMAL_CHORD, &state), 64);
    assert_eq!(midi_note(mv(0, 0, 3, 0), NORMAL_CHORD, &state), 65);
    assert_eq!(midi_note(mv(0, 2, 1, 0), NORMAL_CHORD, &state), 69);
    // Seven steps from C is the C an octave up.
    assert_eq!(midi_note(mv(0, 0, 7, 0), NORMAL_CHORD, &state), 72);
}

#[test]
fn a_scale_step_from_a_note_outside_the_scale_goes_to_the_next_scale_note() {
    // C sharp (a chord root of 1) is not in C major: its first step up is D, then E.
    let state = triad(0, 1);
    assert_eq!(midi_note(mv(0, 0, 0, 0), NORMAL_CHORD, &state), 61);
    assert_eq!(midi_note(mv(0, 0, 1, 0), NORMAL_CHORD, &state), 62);
    assert_eq!(midi_note(mv(0, 0, 2, 0), NORMAL_CHORD, &state), 64);
}

#[test]
fn an_accidental_shifts_by_semitones() {
    let state = c_major();
    assert_eq!(midi_note(mv(0, 0, 0, 1), NORMAL_CHORD, &state), 61);
    assert_eq!(midi_note(mv(0, 0, 0, -1), NORMAL_CHORD, &state), 59);
    assert_eq!(midi_note(mv(0, 0, 0, 7), NORMAL_CHORD, &state), 67);
    assert_eq!(midi_note(mv(0, 0, 0, -8), NORMAL_CHORD, &state), 52);
}

#[test]
fn an_octave_adds_twelve_per_step() {
    let state = c_major();
    assert_eq!(midi_note(mv(1, 0, 0, 0), NORMAL_CHORD, &state), 72);
    assert_eq!(midi_note(mv(2, 0, 0, 0), NORMAL_CHORD, &state), 84);
    // The octave is signed: -1 is an octave down.
    assert_eq!(midi_note(mv(-1, 0, 0, 0), NORMAL_CHORD, &state), 48);
    // Out of the MIDI range it clamps.
    assert_eq!(midi_note(mv(7, 0, 0, 0), NORMAL_CHORD, &state), 127);
    assert_eq!(midi_note(mv(-8, 0, 0, 0), NORMAL_CHORD, &state), 0);
}

#[test]
fn melodic_ignores_the_chord_tone() {
    // MELODIC is the chord root with scale steps: tone 2 gives the root, not the G.
    let state = c_major();
    assert_eq!(midi_note(mv(0, 2, 0, 0), MELODIC, &state), 60);
    assert_eq!(midi_note(mv(0, 2, 1, 0), MELODIC, &state), 62);
}

#[test]
fn fixed_to_chord_adds_the_scale_offset_as_semitones() {
    // Without SCALE_INTERVALS the offset is semitones (an assumption, see the module note).
    let state = c_major();
    assert_eq!(midi_note(mv(0, 2, 3, 0), FIXED_TO_CHORD, &state), 63);
}

#[test]
fn pedal_point_steps_from_the_key_root() {
    // Key D (2): the root is D at 62; one scale step in D major is E, at 64. The chord tone is
    // ignored.
    let state = ChordState {
        key: 2,
        subchord: DEFAULT_SUBCHORD,
    };
    assert_eq!(midi_note(mv(0, 3, 1, 0), PEDALPOINT, &state), 64);
}

#[test]
fn fixed_to_key_is_the_key_root_plus_the_accidental() {
    let state = ChordState {
        key: 2,
        subchord: DEFAULT_SUBCHORD,
    };
    assert_eq!(midi_note(mv(0, 0, 0, 0), FIXED_TO_KEY, &state), 62);
    assert_eq!(midi_note(mv(0, 0, 0, 1), FIXED_TO_KEY, &state), 63);
}

#[test]
fn a_note_takes_its_own_play_mode_unless_it_is_none() {
    assert_eq!(effective_play_mode(16, 14), NORMAL_CHORD);
    assert_eq!(effective_play_mode(6, 14), MELODIC);
    // Zero is FIXED, not "inherit": an explicit FIXED note stays FIXED under a chord part.
    assert_eq!(effective_play_mode(0, 14), FIXED);
    // NONE on both levels is FIXED.
    assert_eq!(effective_play_mode(16, 16), FIXED);
    assert_eq!(effective_play_mode(16, 0), FIXED);
}

#[test]
fn the_nibbles_are_read_high_to_low_and_signed() {
    let value = MusicValue::decode(0xF2C8);
    assert_eq!(value.octave, -1);
    assert_eq!(value.chord_tone, 2);
    assert_eq!(value.scale, 12);
    assert_eq!(value.accidental, -8);
    assert_eq!(MusicValue::decode(0x0007).accidental, 7);
}

#[test]
fn every_value_under_every_mode_is_a_midi_note() {
    // No value and mode can leave the MIDI range or panic (the clamp and the bounded steps).
    let states = [c_major(), triad(5, 13), triad(11, 14)];
    for state in &states {
        for mode in 0..16u8 {
            for value in 0..=u16::MAX {
                let note = midi_note(value, mode, state);
                assert!(note <= 127, "{value:#06x} mode {mode}: {note}");
            }
        }
    }
}
