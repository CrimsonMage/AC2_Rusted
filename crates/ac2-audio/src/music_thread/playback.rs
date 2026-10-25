//! One segment playing on the queue's clock: the segment's sequence performance and, when it
//! names a style, its style performance. Each performance counts from its own start, so the
//! events are stamped with the playback's start to put them on the queue's timeline.
//!
//! The band: a segment with a band track plays its notes on the band of its track, which changes
//! at the track's items (the schedule); a segment with none plays on the performance's band, which
//! the queue sets ([`super::MusicThread`]).

use std::collections::VecDeque;
use std::sync::Arc;

use crate::music::clock::{Boundary, MusicClock};
use crate::music::library::LoadedMusic;
use crate::music::performance::{EventSink, MusicPerformance, TimedEvent};
use crate::music::style_performance::{PatternChoice, StylePerformance};
use crate::music::MusicObject;
use crate::synth::PerformanceBand;

/// Hands events to a buffer, moved by the playback's start (the queue's clock) and stamped with
/// the segment that sent them.
struct Stamp<'a> {
    base: f64,
    owner: u32,
    out: &'a mut Vec<TimedEvent>,
}

impl EventSink for Stamp<'_> {
    fn event(&mut self, event: &TimedEvent) {
        self.out.push(TimedEvent {
            seconds: self.base + event.seconds,
            event: event.event,
            segment: self.owner,
        });
    }
}

/// How far short of a band item a span stops (seconds): the item applies before the notes at its
/// instant, which the span before it does not send.
const BAND_LEAD: f64 = 1e-7;

/// A running segment (`IDirectMusicSegmentState`): its performances and where it is in time.
pub(super) struct Playback {
    id: u32,
    /// The sound handle of the segment (its events' `segment`).
    owner: u32,
    /// Whether the segment has a band track (its notes play on its own band, not the
    /// performance's).
    own_band: bool,
    /// The band items not yet reached, as (seconds from the start, band), in time order.
    schedule: VecDeque<(f64, Arc<PerformanceBand>)>,
    /// The queue's time at which it started.
    start: f64,
    /// The segment's length in seconds.
    length: f64,
    /// The music clock (its boundaries are the play-out boundaries of a stop).
    clock: MusicClock,
    /// Where the pass ends, in seconds from its start: its length, or earlier when a stop at a
    /// boundary cut it short ([`Self::play_out_to`]).
    cut: f64,
    /// Seconds played so far.
    elapsed: f64,
    /// Whether it has reached its end or been stopped.
    ended: bool,
    sequence: MusicPerformance,
    style: Option<StylePerformance>,
}

impl Playback {
    /// The playback of `music` (a segment of sound `owner`) starting at `start`, at `groove`.
    /// `bands` are the segment's band track's items as (seconds from its start, band), in the
    /// order the track gives them (empty for a segment with no band track). The first starts the
    /// notes' band; the rest are the schedule. Without a band track the notes play on
    /// `performance`. `seed` seeds the style's pattern choice. `None` when the object is not a
    /// segment.
    pub(super) fn new(
        id: u32,
        owner: u32,
        start: f64,
        music: &LoadedMusic,
        groove: i8,
        seed: u32,
        bands: &[(f64, Arc<PerformanceBand>)],
        performance: Option<&Arc<PerformanceBand>>,
    ) -> Option<Self> {
        let MusicObject::Segment(segment) = &music.segment else {
            return None;
        };
        let clock = MusicClock::from_segment(segment);
        let mut schedule: Vec<(f64, Arc<PerformanceBand>)> = bands.to_vec();
        schedule.sort_by(|a, b| a.0.total_cmp(&b.0));
        let own_band = !schedule.is_empty();
        let first = schedule
            .first()
            .map(|(_, band)| Arc::clone(band))
            .or_else(|| performance.map(Arc::clone));
        let schedule: VecDeque<_> = schedule.into_iter().skip(1).collect();
        let sequence = MusicPerformance::new(segment, first.map(|p| Arc::clone(&p.band)), false);
        let style = music
            .style_of(segment)
            .map(|style| StylePerformance::new(segment, style, groove, seed));
        Some(Self {
            id,
            owner,
            own_band,
            schedule,
            start,
            length: clock.seconds_at(segment.length.max(0)),
            cut: clock.seconds_at(segment.length.max(0)),
            clock,
            elapsed: 0.0,
            ended: false,
            sequence,
            style,
        })
    }

    /// Whether the segment has a band track (see [`Self::new`]).
    pub(super) fn own_band(&self) -> bool {
        self.own_band
    }

    /// The seconds until the next band item is reached (infinite with none left). The span stops
    /// [`BAND_LEAD`] short of the item, so the notes at the item's instant are sent after its band.
    pub(super) fn next_band_in(&self) -> f64 {
        self.schedule.front().map_or(f64::INFINITY, |(at, _)| {
            (at - self.elapsed - BAND_LEAD).max(0.0)
        })
    }

    /// The band items reached by now, in order, removed from the schedule.
    pub(super) fn take_due_bands(&mut self) -> Vec<Arc<PerformanceBand>> {
        let mut due = Vec::new();
        while self
            .schedule
            .front()
            .is_some_and(|(at, _)| *at <= self.elapsed + 2.0 * BAND_LEAD)
        {
            if let Some((_, band)) = self.schedule.pop_front() {
                due.push(band);
            }
        }
        due
    }

    /// The notes from now on are mapped by `band` (a performance band change, or the segment's
    /// own band item).
    pub(super) fn set_band(&mut self, band: Option<&Arc<crate::music::band::Band>>) {
        self.sequence.set_band(band.map(Arc::clone));
    }

    pub(super) fn id(&self) -> u32 {
        self.id
    }

    /// The sound handle of the segment (the voice its notes go to).
    pub(super) fn owner(&self) -> u32 {
        self.owner
    }

    pub(super) fn length(&self) -> f64 {
        self.length
    }

    pub(super) fn ended(&self) -> bool {
        self.ended
    }

    /// The seconds left before the pass ends (its end, or the boundary a stop cut it at).
    pub(super) fn remaining(&self) -> f64 {
        (self.cut - self.elapsed).max(0.0)
    }

    /// A stop at `unit` (a grid, beat or measure of the segment's clock) plays the pass out to
    /// the next such boundary: its pass ends there, and its notes release there. `None` (an end
    /// or marker stop) plays the pass to its end.
    pub(super) fn play_out_to(&mut self, unit: Option<Boundary>) {
        let Some(unit) = unit else {
            return;
        };
        let tick = self.clock.tick_at(self.elapsed).ceil() as i32;
        let next = self.clock.next_boundary(tick, unit);
        let at = self.clock.seconds_at(next);
        self.cut = self.cut.min(at.max(self.elapsed));
    }

    /// Plays `seconds` more, sending the events due to `out`.
    pub(super) fn advance(&mut self, seconds: f64, out: &mut Vec<TimedEvent>) {
        let mut stamp = Stamp {
            base: self.start,
            owner: self.owner,
            out,
        };
        self.sequence.advance(seconds, &mut stamp);
        if let Some(style) = self.style.as_mut() {
            style.advance(seconds, &mut stamp);
        }
        self.elapsed += seconds;
    }

    /// The pass has reached its end: a pass cut short by a stop releases its notes there, and a
    /// pass that ran to its length has sent everything.
    pub(super) fn finish(&mut self, out: &mut Vec<TimedEvent>) {
        if self.cut < self.length {
            self.stop(out);
            return;
        }
        self.elapsed = self.length;
        self.ended = true;
    }

    /// Stops the segment now, releasing the notes sounding (`StopEx` at the current time).
    pub(super) fn stop(&mut self, out: &mut Vec<TimedEvent>) {
        let mut stamp = Stamp {
            base: self.start,
            owner: self.owner,
            out,
        };
        self.sequence.stop(&mut stamp);
        if let Some(style) = self.style.as_mut() {
            style.stop(&mut stamp);
        }
        self.ended = true;
    }

    /// The master groove level (`GUID_PerfMasterGrooveLevel`): the style's next measure uses it.
    pub(super) fn set_groove(&mut self, level: i8) {
        if let Some(style) = self.style.as_mut() {
            style.set_groove_level(level);
        }
    }

    /// The patterns the style has chosen so far.
    pub(super) fn choices(&self) -> Option<&[PatternChoice]> {
        self.style.as_ref().map(|style| style.choices())
    }
}
