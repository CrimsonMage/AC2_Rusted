//! One segment playing on the queue's clock: the segment's sequence performance and, when it
//! names a style, its style performance. Each performance counts from its own start, so the
//! events are stamped with the playback's start to put them on the queue's timeline.
//!
//! The band: a segment with a band track plays its notes on the band of its track, which changes
//! at the track's items (the schedule); a segment with none plays on the performance's band, which
//! the queue sets ([`super::MusicThread`]).

use std::collections::VecDeque;
use std::sync::Arc;

use crate::music::band::Band;
use crate::music::clock::MusicClock;
use crate::music::library::LoadedMusic;
use crate::music::performance::{assign, EventSink, MusicEvent, MusicPerformance, TimedEvent};
use crate::music::script::{BoundCall, BoundSegment, BoundTrigger};
use crate::music::style_performance::{PatternChoice, StylePerformance};
use crate::music::{MusicObject, WaveSegment};
use crate::synth::PerformanceBand;

/// Hands events to a buffer, moved by the playback's start (the queue's clock) and stamped with
/// the segment that sent them.
struct Stamp<'a> {
    base: f64,
    owner: u32,
    out: &'a mut Vec<TimedEvent>,
    /// The band a note with no instrument of its own takes its instrument from (the segment's
    /// band, or the performance's). `None` keeps the events as they come.
    band: Option<&'a Band>,
}

impl EventSink for Stamp<'_> {
    fn event(&mut self, timed: &TimedEvent) {
        let mut event = timed.event;
        if let (
            Some(band),
            MusicEvent::NoteOn {
                channel,
                instrument: None,
                note,
                velocity,
            },
        ) = (self.band, event)
        {
            // A style note on a channel its style's band does not assign takes the performance's
            // instrument for the channel (`assign`), as the channel's band sets it.
            let (instrument, note) = assign(Some(band), channel, note);
            event = MusicEvent::NoteOn {
                channel,
                instrument,
                note,
                velocity,
            };
        }
        self.out.push(TimedEvent {
            seconds: self.base + timed.seconds,
            event,
            segment: self.owner,
        });
    }
}

/// How far short of a band item a span stops (seconds): the item applies before the notes at its
/// instant, which the span before it does not send.
const BAND_LEAD: f64 = 1e-7;

/// A seconds tolerance (the queue's `EPS`).
const EPS: f64 = 1e-9;

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
    /// Where the pass ends, in seconds from its start: its length, or the end of the pass in
    /// progress when a stop cleared the loop of a repeating wave ([`Self::end_pass_after_stop`]).
    cut: f64,
    /// Seconds played so far.
    elapsed: f64,
    /// Whether it has reached its end or been stopped.
    ended: bool,
    /// The sequence's performance (none for a wave segment, which has no notes).
    sequence: Option<MusicPerformance>,
    /// The band the notes play on: the segment's own, or the performance's (see [`Stamp`]).
    band: Option<Arc<Band>>,
    style: Option<StylePerformance>,
    /// The pass's length, for a wave that repeats within one playback (a stop ends the pass in
    /// progress at its end, see [`Self::end_pass_after_stop`]).
    pass: f64,
    /// The cues not yet reached, in time order.
    cues: VecDeque<Cue>,
}

/// A cue a playback reaches: a script call, or a segment trigger that starts a secondary.
#[derive(Debug, Clone)]
pub(super) enum Cue {
    Call(BoundCall),
    Trigger(BoundTrigger),
}

impl Cue {
    fn seconds(&self) -> f64 {
        match self {
            Cue::Call(call) => call.seconds,
            Cue::Trigger(trigger) => trigger.seconds,
        }
    }
}

impl Playback {
    /// The playback of `music` (a segment of sound `owner`) starting at `start`, at `groove`.
    /// `bands` are the segment's band track's items as (seconds from its start, band), in the
    /// order the track gives them (empty for a segment with no band track). The first starts the
    /// notes' band; the rest are the schedule. Without a band track the notes play on
    /// `performance`. `seed` seeds the style's pattern choice. A wave segment plays its wave for
    /// its time on the performance's default clock (120 quarter notes a minute, 4/4). `None` when
    /// the object is neither a segment nor a wave.
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
        let segment = match &music.segment {
            MusicObject::Segment(segment) => segment,
            MusicObject::Wave(wave) => return Some(Self::wave(id, owner, start, *wave)),
            _ => return None,
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
        let band = first.as_ref().map(|p| Arc::clone(&p.band));
        let sequence = Some(MusicPerformance::new(
            segment,
            first.map(|p| Arc::clone(&p.band)),
            false,
        ));
        let style = music
            .style_of(segment)
            .map(|style| StylePerformance::new(segment, style, groove, seed));
        let length = clock.seconds_at(segment.length.max(0));
        Some(Self {
            id,
            owner,
            own_band,
            schedule,
            start,
            length,
            cut: length,
            elapsed: 0.0,
            ended: false,
            sequence,
            style,
            band,
            pass: length,
            cues: VecDeque::new(),
        })
    }

    /// The playback of a wave segment: no notes, its pass is the wave's time (forever when it
    /// loops), on the default clock (no tempo or meter track).
    fn wave(id: u32, owner: u32, start: f64, wave: WaveSegment) -> Self {
        let length = if wave.looping {
            f64::INFINITY
        } else {
            wave.seconds.max(0.0)
        };
        Self {
            id,
            owner,
            own_band: false,
            schedule: VecDeque::new(),
            start,
            length,
            cut: length,
            elapsed: 0.0,
            ended: false,
            sequence: None,
            style: None,
            band: None,
            pass: wave.seconds.max(0.0),
            cues: VecDeque::new(),
        }
    }

    /// Schedules the segment's cues: its script calls (`DMUS_IO_SCRIPTTRACK`) and its triggers,
    /// each at its time.
    pub(super) fn schedule_cues(&mut self, cues: &BoundSegment) {
        let mut all: Vec<Cue> = cues.calls.iter().cloned().map(Cue::Call).collect();
        all.extend(cues.triggers.iter().cloned().map(Cue::Trigger));
        all.sort_by(|a, b| a.seconds().total_cmp(&b.seconds()));
        self.cues = all.into();
    }

    /// The seconds until the next cue (infinite with none left), stopping [`BAND_LEAD`] short of it
    /// as the band items do.
    pub(super) fn next_cue_in(&self) -> f64 {
        self.cues.front().map_or(f64::INFINITY, |cue| {
            (cue.seconds() - self.elapsed - BAND_LEAD).max(0.0)
        })
    }

    /// The cues reached by now, in order, removed from the schedule.
    pub(super) fn take_due_cues(&mut self) -> Vec<Cue> {
        let mut due = Vec::new();
        while self
            .cues
            .front()
            .is_some_and(|cue| cue.seconds() <= self.elapsed + 2.0 * BAND_LEAD)
        {
            if let Some(cue) = self.cues.pop_front() {
                due.push(cue);
            }
        }
        due
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
    pub(super) fn set_band(&mut self, band: Option<&Arc<Band>>) {
        self.band = band.map(Arc::clone);
        if let Some(sequence) = self.sequence.as_mut() {
            sequence.set_band(band.map(Arc::clone));
        }
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

    /// A stop of the current primary (`DXMusicMessageThread::Stop` on `m_pCurrentPrimary`): the
    /// queue clears the loop and nothing else, so the pass plays out to its end whatever the
    /// boundary the stop names. A pass that repeats within one playback (a wave) ends at the end
    /// of the pass it is in; a segment's pass already ends at its length.
    pub(super) fn end_pass_after_stop(&mut self) {
        if self.length.is_finite() || self.pass <= EPS {
            return;
        }
        let passes = (self.elapsed / self.pass).floor() + 1.0;
        self.cut = self.cut.min(passes * self.pass);
    }

    /// Plays `seconds` more, sending the events due to `out`. The sequence's notes come mapped by
    /// its band already; a style's notes take their instrument from the playback's band.
    pub(super) fn advance(&mut self, seconds: f64, out: &mut Vec<TimedEvent>) {
        if let Some(sequence) = self.sequence.as_mut() {
            let mut stamp = Stamp {
                base: self.start,
                owner: self.owner,
                out: &mut *out,
                band: None,
            };
            sequence.advance(seconds, &mut stamp);
        }
        if let Some(style) = self.style.as_mut() {
            let mut stamp = Stamp {
                base: self.start,
                owner: self.owner,
                out,
                band: self.band.as_deref(),
            };
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
            band: None,
        };
        if let Some(sequence) = self.sequence.as_mut() {
            sequence.stop(&mut stamp);
        }
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
