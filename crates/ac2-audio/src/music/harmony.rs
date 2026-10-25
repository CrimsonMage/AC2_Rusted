//! The harmony a style note is decoded against: the chord in effect at a music tick, read from
//! the segment's chord track (`DMUS_IO_CHORD` and its `DMUS_IO_SUBCHORD`s, with the key root and
//! scale of the track's `crdh`).
//!
//! The chord in effect at a tick is the last chord whose `mtTime` is at or before it; before the
//! first chord the first one applies. A subchord is the one whose `dwLevels` holds the part
//! reference's `bSubChordLevel` (bit `L` is level `L`); with none, the first subchord.
//!
//! Checked against the DATs (16 segments with a chord track, 235 chords): every chord's `mtTime`
//! equals its `wMeasure` times the measure's ticks plus its `bBeat` times the beat's ticks;
//! chord patterns have bit `i` set for the chord tone `i` semitones above the root (0x91 major,
//! 0x89 minor, 0x891 major seventh); scale patterns hold a 12-bit scale twice (0xAB5 major is
//! stored as 0xAB5AB5 in `crdh`); the `crdh` key root is in the top byte, zero in every track;
//! and the part references use subchord levels 0 to 2, whose bits (0x1, 0x2, 0x4) all occur in
//! the `dwLevels` masks (0x8 and the all-but-0 masks occur too, and are unused here).
//!
//! Assumptions: a segment with no chord track (or an empty one) plays over [`DEFAULT_SUBCHORD`],
//! a C-major triad in the major scale over key C, the default this work specifies. A subchord whose
//! scale pattern has no 12-bit scale takes the track's scale, else the major scale. Whether
//! `bChordRoot` and `bScaleRoot` are relative to the key root is not checked: every track in the
//! DATs has key C, so the reading only changes the pitch by the key, and the key is zero there.

use super::segment::{Segment, SubChord, TrackData};

/// The fallback chord: C major (root 0, triad `0x91`) in the major scale (`0xAB5`) over key C.
pub const DEFAULT_SUBCHORD: SubChord = SubChord {
    chord_pattern: 0x91,
    scale_pattern: DEFAULT_SCALE,
    inversion_points: 0,
    levels: u32::MAX,
    chord_root: 0,
    scale_root: 0,
};

/// The major scale: bits 0, 2, 4, 5, 7, 9 and 11 (semitones above the scale root).
pub const DEFAULT_SCALE: u32 = 0xAB5;

/// The chord a note is decoded against: the key root (semitones above C, `0..12`) and the
/// subchord in effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChordState {
    pub key: i32,
    pub subchord: SubChord,
}

/// A segment's chords over time, with the key and scale of its chord track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Harmony {
    key: i32,
    /// The track's scale pattern (`crdh`, 24 bits), used when a subchord has no scale.
    scale: u32,
    /// `(mtTime, subchords)` in time order, never empty.
    chords: Vec<(i32, Vec<SubChord>)>,
}

impl Default for Harmony {
    /// C major over the whole segment: the harmony of a segment with no chord track.
    fn default() -> Self {
        Self::new(0, DEFAULT_SCALE, Vec::new())
    }
}

impl Harmony {
    /// A harmony from its key (semitones above C), scale pattern and `(time, subchords)` chords,
    /// in any order. With no chords, the default chord applies from time 0.
    pub fn new(key: i32, scale: u32, mut chords: Vec<(i32, Vec<SubChord>)>) -> Self {
        chords.sort_by_key(|(time, _)| *time);
        if chords.is_empty() {
            chords.push((0, vec![DEFAULT_SUBCHORD]));
        }
        Self {
            key: key.rem_euclid(12),
            scale,
            chords,
        }
    }

    /// The harmony of `segment`'s first chord track; the default when it has none.
    pub fn from_segment(segment: &Segment) -> Self {
        let track = segment.tracks.iter().find_map(|t| match &t.data {
            TrackData::Chords { scale, chords } => Some((*scale, chords)),
            _ => None,
        });
        let Some((crdh, chords)) = track else {
            return Self::default();
        };
        let key = i32::from((crdh >> 24) as u8);
        let chords = chords
            .iter()
            .map(|c| (c.time, c.subchords.clone()))
            .collect();
        Self::new(key, crdh & 0xFF_FFFF, chords)
    }

    /// The chord in effect at `tick` for a part at subchord `level`.
    pub fn state_at(&self, tick: i32, level: u8) -> ChordState {
        let index = self
            .chords
            .partition_point(|(time, _)| *time <= tick)
            .saturating_sub(1);
        let subchords = &self.chords[index].1;
        let mut subchord = subchords
            .iter()
            .find(|s| level < 32 && (s.levels >> level) & 1 == 1)
            .or(subchords.first())
            .copied()
            .unwrap_or(DEFAULT_SUBCHORD);
        if subchord.scale_pattern & 0xFFF == 0 {
            subchord.scale_pattern = if self.scale & 0xFFF != 0 {
                self.scale & 0xFFF
            } else {
                DEFAULT_SCALE
            };
        }
        ChordState {
            key: self.key,
            subchord,
        }
    }
}
