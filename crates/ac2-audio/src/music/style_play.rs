//! Style playback's pure part: which patterns a groove level covers, and the notes a pattern's
//! parts play in one of its measures. Times are music ticks from the measure's start; the
//! performance ([`super::style_performance`]) turns them into physical time.
//!
//! What is implemented, and what is not (the decompile has no DirectMusic, so these follow the
//! DirectMusic file format and its documented behaviour):
//!
//! - Patterns: only groove patterns (`wEmbellishment` 0) are chosen by groove. Intro, fill,
//!   break and end patterns are triggered by commands, which are not played yet.
//! - Parts: a pattern's part references (`pref`) give the part and its performance channel. A
//!   part measure is taken modulo the part's measure count, as a pattern measure is modulo the
//!   pattern's.
//! - Variations: the variation played is the lowest-numbered one the part allows
//!   (`dwVariationChoices` nonzero). DirectMusic chooses among the allowed variations for the
//!   chord; that choice is not implemented.
//! - Pitch: a note's `wMusicValue` is decoded under its play mode ([`super::music_value`]:
//!   the note's, else the part's, where 16 defers to the part) against the chord in effect at
//!   the note's time ([`super::harmony`]: the segment's chord track, with its subchord at the
//!   part reference's level; a segment with no chord track is C major). The DATs' 21,568 style
//!   notes are all play mode 16; their parts are FIXED (MIDI notes, the tabla groove), MELODIC
//!   or NORMALCHORD, so the chord matters only in the chord-relative parts. The assumptions in
//!   the decoding are listed in [`super::music_value`].
//! - Band: the pattern's band, else the style's first band, transposes and assigns the patch per
//!   channel, as a sequence track does ([`super::performance::note_events`]).
//! - Timing: each note's start is its grid (`mtGridStart`, in grids from the part's start)
//!   times the grid length, plus its time offset. Randomised ranges (`time_range`,
//!   `duration_range`, `velocity_range`) are ignored.

use super::band::{Band, BandInstrument};
use super::clock::MusicClock;
use super::harmony::Harmony;
use super::music_value::{effective_play_mode, midi_note};
use super::style::{Part, PartRef, Pattern, Style, StyleNote};

/// `wEmbellishment` of a groove pattern (`DMUS_EMBELLISHT_NORMAL`).
const GROOVE: u16 = 0;
/// `DMUS_IO_STYLENOTE` variation bits and `dwVariationChoices` are 32 wide.
const VARIATIONS: usize = 32;

/// Whether `pattern` is a groove pattern whose groove range (`bottom..=top`) contains `level`.
pub fn covers(pattern: &Pattern, level: i8) -> bool {
    let level = i32::from(level);
    pattern.embellishment == GROOVE
        && i32::from(pattern.groove_bottom) <= level
        && level <= i32::from(pattern.groove_top)
}

/// A note of a pattern's measure: its start `tick` from the measure's start, its duration in
/// ticks, and what plays it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleNoteSpan {
    pub tick: i32,
    pub duration: i32,
    pub channel: u32,
    pub instrument: Option<u32>,
    pub note: u8,
    pub velocity: u8,
}

/// The notes `pattern` plays in its `measure`-th measure (measures repeat with the pattern's
/// length), in the order of its part references. `harmony` is the segment's chord track and
/// `measure_start` the measure's start in music ticks from the segment's start: each note is
/// decoded against the chord in effect at its own time.
pub fn pattern_measure_notes(
    style: &Style,
    pattern: &Pattern,
    band: Option<&Band>,
    measure: usize,
    harmony: &Harmony,
    measure_start: i32,
) -> Vec<StyleNoteSpan> {
    let measure = measure % usize::from(pattern.measures.max(1));
    let mut out = Vec::new();
    for pref in &pattern.parts {
        if let Some(part) = style.part(&pref.part) {
            part_measure_notes(part, pref, band, measure, harmony, measure_start, &mut out);
        }
    }
    out
}

/// One part's notes in its measure `measure` (taken modulo the part's length), in the variation
/// the part allows (see the module note).
fn part_measure_notes(
    part: &Part,
    pref: &PartRef,
    band: Option<&Band>,
    measure: usize,
    harmony: &Harmony,
    measure_start: i32,
    out: &mut Vec<StyleNoteSpan>,
) {
    let Some(variation) = (0..VARIATIONS).find(|&v| part.variation_choices[v] != 0) else {
        return;
    };
    let part_measure = measure % usize::from(part.measures.max(1));
    let grids_per_measure = i32::from(part.time_sig.beats_per_measure.max(1))
        * i32::from(part.time_sig.grids_per_beat.max(1));
    let first_grid = part_measure as i32 * grids_per_measure;
    let notes = PartNotes {
        part,
        pref,
        assignment: band.and_then(|b| b.instruments.iter().find(|i| i.pchannel == pref.pchannel)),
        harmony,
        measure_start,
        first_grid,
        grid_ticks: MusicClock::grid_ticks(&part.time_sig),
    };
    for note in part
        .notes
        .iter()
        .filter(|n| n.grid_start >= first_grid && n.grid_start < first_grid + grids_per_measure)
    {
        if note.variation & (1u32 << variation) == 0 {
            continue;
        }
        out.push(notes.span(note));
    }
}

/// What a part's notes in one measure are decoded from.
struct PartNotes<'a> {
    part: &'a Part,
    pref: &'a PartRef,
    assignment: Option<&'a BandInstrument>,
    harmony: &'a Harmony,
    /// The measure's start in music ticks from the segment's start.
    measure_start: i32,
    /// The measure's first grid, in grids from the part's start.
    first_grid: i32,
    grid_ticks: i32,
}

impl PartNotes<'_> {
    fn span(&self, note: &StyleNote) -> StyleNoteSpan {
        let tick =
            (note.grid_start - self.first_grid) * self.grid_ticks + i32::from(note.time_offset);
        let play_mode = effective_play_mode(note.play_mode, self.part.play_mode);
        let state = self
            .harmony
            .state_at(self.measure_start + tick, self.pref.subchord_level);
        let pitch = i32::from(midi_note(note.music_value, play_mode, &state));
        let pitch = self
            .assignment
            .map_or(pitch, |i| (pitch + i32::from(i.transpose)).clamp(0, 127));
        StyleNoteSpan {
            tick,
            duration: note.duration.max(1),
            channel: self.pref.pchannel,
            instrument: self.assignment.map(|i| i.effective_patch()),
            note: pitch as u8,
            velocity: note.velocity,
        }
    }
}
