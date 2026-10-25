//! A style note's `wMusicValue` decoded to a MIDI note under its play mode, against the chord in
//! effect ([`ChordState`]). Pure: the caller picks the chord ([`super::harmony`]) and the play
//! mode ([`effective_play_mode`]).
//!
//! The DirectMusic rules here, with what is documented and what is assumed:
//!
//! - Documented: DirectMusic's format and play-mode rules, as stated for this work (no DirectMusic
//!   source is in the workspace, so these are not re-read from a document here). `wMusicValue`
//!   holds four nibbles from the
//!   high end, octave, chord tone, scale offset and accidental; the accidental is signed (-8 to
//!   7). `bPlayModeFlags` bits are KEY_ROOT 0x01, CHORD_ROOT 0x02, SCALE_INTERVALS 0x04,
//!   CHORD_INTERVALS 0x08 and NONE 0x10 (a note's NONE takes the part's mode). FIXED (0) passes
//!   `wMusicValue` as the MIDI note. NORMALCHORD (0x0E) is the chord tone of the chord, then the
//!   scale offset in scale steps. MELODIC (0x06) is the chord root and scale steps.
//! - Confirmed on the DATs: the chord pattern bits are semitones above the chord root (0x91 is
//!   a major triad), the scale pattern is 12 bits (0xAB5 major), and the play mode a note takes
//!   is its part's (every note in the DATs is 16; the parts are 0, 6 or 14). Every part
//!   reference is 28 bytes, the `DMUS_IO_PARTREF` layout without a play-mode field, so the part
//!   is the only other source.
//! - Assumed, not checked against retail: the octave nibble is signed (-8 to 7) and the scale
//!   offset unsigned (0 to 15); the chord and key are anchored at middle C (60) plus their
//!   roots, a root of 12 or more being an octave up; chord tones index the chord's sorted tones
//!   and wrap by octave; a scale step counts the scale's pitches above the note; without
//!   SCALE_INTERVALS the scale offset is semitones; with both KEY_ROOT and CHORD_ROOT the chord
//!   root is the anchor; a part whose mode is NONE is FIXED; `dwInversionPoints` and the
//!   inversion ID are not applied.
//!
//! The result is clamped to the MIDI range `0..=127`.

use super::harmony::ChordState;

/// `DMUS_PLAYMODE_NONE`: the note takes its part's play mode.
pub const PLAY_NONE: u8 = 0x10;
/// `DMUS_PLAYMODE_KEY_ROOT`.
pub const PLAY_KEY_ROOT: u8 = 0x01;
/// `DMUS_PLAYMODE_CHORD_ROOT`.
pub const PLAY_CHORD_ROOT: u8 = 0x02;
/// `DMUS_PLAYMODE_SCALE_INTERVALS`.
pub const PLAY_SCALE_INTERVALS: u8 = 0x04;
/// `DMUS_PLAYMODE_CHORD_INTERVALS`.
pub const PLAY_CHORD_INTERVALS: u8 = 0x08;

/// The reference pitch of a root at offset 0 (middle C).
const REFERENCE_NOTE: i32 = 60;

/// The play mode a note plays in: its own when it names one, else its part's, else FIXED.
/// A note's 0 is FIXED (explicit); NONE (0x10) defers to the part.
pub fn effective_play_mode(note_mode: u8, part_mode: u8) -> u8 {
    let named = |mode: u8| (mode != PLAY_NONE).then_some(mode & 0x0F);
    named(note_mode).or(named(part_mode)).unwrap_or(0)
}

/// The four fields of a `wMusicValue`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicValue {
    pub octave: i32,
    pub chord_tone: u32,
    pub scale: u32,
    pub accidental: i32,
}

impl MusicValue {
    pub fn decode(music_value: u16) -> Self {
        Self {
            octave: signed_nibble(music_value >> 12),
            chord_tone: u32::from((music_value >> 8) & 0xF),
            scale: u32::from((music_value >> 4) & 0xF),
            accidental: signed_nibble(music_value),
        }
    }
}

/// The low nibble of `n` as a two's-complement value, `-8..=7`.
fn signed_nibble(n: u16) -> i32 {
    let nibble = i32::from(n & 0xF);
    if nibble >= 8 {
        nibble - 16
    } else {
        nibble
    }
}

/// The MIDI note of a style note with `music_value` under `play_mode` (already the effective
/// mode, see [`effective_play_mode`]) against `state`.
pub fn midi_note(music_value: u16, play_mode: u8, state: &ChordState) -> u8 {
    let mode = play_mode & 0x0F;
    if mode == 0 {
        return (music_value & 0x7F) as u8;
    }
    let value = MusicValue::decode(music_value);
    let subchord = &state.subchord;
    let chord_based = mode & (PLAY_CHORD_ROOT | PLAY_CHORD_INTERVALS) != 0;
    let anchor = REFERENCE_NOTE
        + state.key
        + if chord_based {
            i32::from(subchord.chord_root)
        } else {
            0
        };
    let mut note = if mode & PLAY_CHORD_INTERVALS != 0 {
        chord_tone(anchor, subchord.chord_pattern, value.chord_tone)
    } else {
        anchor
    };
    if mode & PLAY_SCALE_INTERVALS != 0 {
        let scale = Scale {
            base: state.key + i32::from(subchord.scale_root),
            pattern: subchord.scale_pattern,
        };
        note = scale.steps_up(note, value.scale);
    } else {
        note += value.scale as i32;
    }
    (note + value.accidental + 12 * value.octave).clamp(0, 127) as u8
}

/// The chord tone `index` of a chord (the set bits of `pattern`, in order, from `root`), wrapping
/// up an octave per run of tones. A pattern with no tones gives the root.
fn chord_tone(root: i32, pattern: u32, index: u32) -> i32 {
    let tones: Vec<i32> = (0..32).filter(|&bit| (pattern >> bit) & 1 == 1).collect();
    let Ok(count) = u32::try_from(tones.len()) else {
        return root;
    };
    if count == 0 {
        return root;
    }
    root + tones[(index % count) as usize] + 12 * (index / count) as i32
}

/// A scale: its pattern (12 bits, semitones above `base`), with `base` in semitones above C.
struct Scale {
    base: i32,
    pattern: u32,
}

impl Scale {
    fn contains(&self, pitch: i32) -> bool {
        let pitch_class = (pitch - self.base).rem_euclid(12) as u32;
        (self.pattern >> pitch_class) & 1 == 1
    }

    /// `steps` scale pitches above `note` (each step is the next pitch in the scale). A scale
    /// with no tones leaves the note as it is.
    fn steps_up(&self, note: i32, steps: u32) -> i32 {
        if self.pattern & 0xFFF == 0 {
            return note;
        }
        let mut pitch = note;
        for _ in 0..steps {
            pitch += 1;
            while !self.contains(pitch) {
                pitch += 1;
            }
        }
        pitch
    }
}
