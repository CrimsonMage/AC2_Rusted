//! The music clock of a DirectMusic segment: music time in ticks (`DMUS_PPQ` = 768 per quarter
//! note) to physical seconds through the tempo track, and the meter's grid, beat and measure
//! boundaries from the time-signature track. Transitions (`DMUS_SEGF_GRID`, `_BEAT`, `_MEASURE`)
//! start at the next boundary of their unit.
//!
//! Checked against the DATs: every segment's length is a whole number of measures
//! (`length % measure_ticks == 0` for all 630), and the DATs' tempo and meter tracks hold one item
//! at time 0 each (80 BPM mostly, 6/8 mostly). Tempo is read as quarter notes per minute, the
//! MIDI convention; that reading is not confirmed against retail output.

use super::segment::{Segment, TimeSignature, TrackData};

/// `DMUS_PPQ`: ticks per quarter note.
pub const PPQ: i32 = 768;
/// The tempo before the segment's first tempo item, and with no tempo track (DirectMusic's
/// default, quarter notes per minute).
pub const DEFAULT_TEMPO: f64 = 120.0;

/// The unit a transition waits for (`DMUS_SEGF_GRID` / `_BEAT` / `_MEASURE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    Grid,
    Beat,
    Measure,
}

/// A segment's tempo and meter over music time.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicClock {
    /// `(tick, quarter notes per minute)`, sorted by tick.
    tempo: Vec<(i32, f64)>,
    /// `(tick, signature)`, sorted by tick; the first range starts at 0.
    meter: Vec<(i32, TimeSignature)>,
}

impl Default for MusicClock {
    /// No tempo track and 4/4 from the start.
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

impl MusicClock {
    /// A clock from the tempo and time-signature items, in any order.
    pub fn new(mut tempo: Vec<(i32, f64)>, mut meter: Vec<(i32, TimeSignature)>) -> Self {
        tempo.sort_by_key(|(t, _)| *t);
        meter.sort_by_key(|(t, _)| *t);
        let mut clock = Self { tempo, meter };
        if clock.meter.first().is_none_or(|(t, _)| *t > 0) {
            clock.meter.insert(0, (0, TimeSignature::default()));
        }
        clock
    }

    /// The first tempo and time-signature tracks of `segment` (a segment without one keeps the
    /// default).
    pub fn from_segment(segment: &Segment) -> Self {
        let mut tempo = None;
        let mut meter = None;
        for track in &segment.tracks {
            match &track.data {
                TrackData::Tempo(items) if tempo.is_none() => tempo = Some(items.clone()),
                TrackData::TimeSig(items) if meter.is_none() => meter = Some(items.clone()),
                _ => {}
            }
        }
        Self::new(tempo.unwrap_or_default(), meter.unwrap_or_default())
    }

    /// The tempo in effect at `tick`: the last item at or before it.
    pub fn tempo_at(&self, tick: i32) -> f64 {
        self.tempo
            .iter()
            .rev()
            .find(|(t, _)| *t <= tick)
            .map_or(DEFAULT_TEMPO, |(_, bpm)| *bpm)
    }

    /// The physical time from the segment's start to `tick`, in seconds: each tempo range's
    /// ticks at its `bpm * PPQ / 60` ticks a second, summed in f64.
    pub fn seconds_at(&self, tick: i32) -> f64 {
        let tick = tick.max(0);
        let ranges = self.tempo_ranges();
        let mut seconds = 0.0;
        for (i, &(from, bpm)) in ranges.iter().enumerate() {
            let from = from.max(0).min(tick);
            let to = ranges.get(i + 1).map_or(tick, |r| r.0.min(tick)).max(from);
            seconds += f64::from(to - from) * 60.0 / (bpm * f64::from(PPQ));
        }
        seconds
    }

    /// The music time (in ticks, fractional) `seconds` from the segment's start reaches: the
    /// inverse of [`Self::seconds_at`].
    pub fn tick_at(&self, seconds: f64) -> f64 {
        let mut remaining = seconds.max(0.0);
        let ranges = self.tempo_ranges();
        for (i, &(from, bpm)) in ranges.iter().enumerate() {
            let from = f64::from(from.max(0));
            let to = ranges.get(i + 1).map_or(f64::INFINITY, |r| f64::from(r.0));
            let ticks_per_second = bpm * f64::from(PPQ) / 60.0;
            let span = (to - from) / ticks_per_second;
            if remaining <= span {
                return from + remaining * ticks_per_second;
            }
            remaining -= span;
        }
        // Unreachable: the last range runs to infinity.
        f64::from(i32::MAX)
    }

    /// `(start tick, bpm)` for each range: the default from 0 unless the first item is at 0.
    fn tempo_ranges(&self) -> Vec<(i32, f64)> {
        let mut ranges = Vec::with_capacity(self.tempo.len() + 1);
        if self.tempo.first().is_none_or(|(t, _)| *t > 0) {
            ranges.push((0, DEFAULT_TEMPO));
        }
        ranges.extend(self.tempo.iter().copied());
        ranges
    }

    /// The time signature in effect at `tick`.
    pub fn signature_at(&self, tick: i32) -> TimeSignature {
        self.meter
            .iter()
            .rev()
            .find(|(t, _)| *t <= tick)
            .map_or_else(TimeSignature::default, |(_, sig)| *sig)
    }

    /// Ticks in a beat of `sig` (`DMUS_PPQ * 4 / beat`: a beat is `beat`-notes).
    pub fn beat_ticks(sig: &TimeSignature) -> i32 {
        PPQ * 4 / i32::from(sig.beat.max(1))
    }

    /// Ticks in a grid of `sig` (a beat over `grids_per_beat`).
    pub fn grid_ticks(sig: &TimeSignature) -> i32 {
        (Self::beat_ticks(sig) / i32::from(sig.grids_per_beat.max(1))).max(1)
    }

    /// Ticks in a measure of `sig`.
    pub fn measure_ticks(sig: &TimeSignature) -> i32 {
        Self::beat_ticks(sig) * i32::from(sig.beats_per_measure.max(1))
    }

    /// The first `unit` boundary at or after `tick`. Measures and beats count from the start of
    /// the time signature in effect, and a signature change is a boundary too (the count
    /// restarts there).
    pub fn next_boundary(&self, tick: i32, unit: Boundary) -> i32 {
        for (i, (start, sig)) in self.meter.iter().enumerate() {
            let end = self.meter.get(i + 1).map(|(t, _)| *t);
            let from = tick.max(*start);
            if end.is_some_and(|e| from >= e) {
                continue;
            }
            let len = match unit {
                Boundary::Grid => Self::grid_ticks(sig),
                Boundary::Beat => Self::beat_ticks(sig),
                Boundary::Measure => Self::measure_ticks(sig),
            };
            let steps = (from - start + len - 1) / len;
            let boundary = start + steps * len;
            return match end {
                Some(e) if boundary >= e => e,
                _ => boundary,
            };
        }
        tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(beats: u8, beat: u8, grids: u16) -> TimeSignature {
        TimeSignature {
            beats_per_measure: beats,
            beat,
            grids_per_beat: grids,
        }
    }

    #[test]
    fn seconds_follow_the_tempo_map() {
        // 120 quarter notes a minute: a quarter is half a second.
        let clock = MusicClock::default();
        assert_eq!(clock.seconds_at(0), 0.0);
        assert_eq!(clock.seconds_at(PPQ), 0.5);
        assert_eq!(clock.tempo_at(5), DEFAULT_TEMPO);
        // 60 from tick 768: the next quarter takes a second.
        let clock = MusicClock::new(vec![(768, 60.0), (0, 120.0)], Vec::new());
        assert_eq!(clock.tempo_at(767), 120.0);
        assert_eq!(clock.tempo_at(768), 60.0);
        assert_eq!(clock.seconds_at(768), 0.5);
        assert_eq!(clock.seconds_at(1536), 1.5);
        // A tick inside a range counts only its share.
        assert!((clock.seconds_at(1152) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_first_tempo_item_after_zero_leaves_the_default_before_it() {
        let clock = MusicClock::new(vec![(768, 240.0)], Vec::new());
        // 768 ticks at the default (0.5 s), then one quarter at 240 (0.25 s).
        assert_eq!(clock.seconds_at(1536), 0.75);
    }

    #[test]
    fn beats_grids_and_measures_of_6_8() {
        let clock = MusicClock::new(Vec::new(), vec![(0, sig(6, 8, 4))]);
        assert_eq!(MusicClock::beat_ticks(&sig(6, 8, 4)), 384);
        assert_eq!(MusicClock::grid_ticks(&sig(6, 8, 4)), 96);
        assert_eq!(MusicClock::measure_ticks(&sig(6, 8, 4)), 2304);
        assert_eq!(clock.next_boundary(0, Boundary::Measure), 0);
        assert_eq!(clock.next_boundary(1, Boundary::Measure), 2304);
        assert_eq!(clock.next_boundary(2304, Boundary::Beat), 2304);
        assert_eq!(clock.next_boundary(2305, Boundary::Beat), 2688);
        assert_eq!(clock.next_boundary(100, Boundary::Grid), 192);
        assert_eq!(clock.signature_at(5000), sig(6, 8, 4));
    }

    #[test]
    fn a_signature_change_restarts_the_count() {
        // 4/4 for one measure, then 3/4 from tick 3072.
        let clock = MusicClock::new(Vec::new(), vec![(0, sig(4, 4, 4)), (3072, sig(3, 4, 4))]);
        assert_eq!(clock.next_boundary(2000, Boundary::Measure), 3072);
        assert_eq!(clock.next_boundary(3072, Boundary::Measure), 3072);
        assert_eq!(clock.next_boundary(3073, Boundary::Measure), 5376);
        assert_eq!(clock.next_boundary(3500, Boundary::Beat), 3840);
        assert_eq!(clock.signature_at(3071), sig(4, 4, 4));
        assert_eq!(clock.signature_at(3072), sig(3, 4, 4));
    }

    #[test]
    fn a_meter_that_starts_late_is_4_4_before_it() {
        let clock = MusicClock::new(Vec::new(), vec![(1536, sig(3, 4, 4))]);
        assert_eq!(clock.signature_at(0), TimeSignature::default());
        assert_eq!(clock.next_boundary(100, Boundary::Measure), 1536);
        assert_eq!(clock.next_boundary(1600, Boundary::Measure), 1536 + 2304);
    }
}
