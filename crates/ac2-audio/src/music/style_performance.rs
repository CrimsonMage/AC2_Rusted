//! The performance of a segment that has a style reference: it plays the style's patterns one
//! measure at a time, each measure's pattern chosen by the groove level in effect when the
//! measure starts. Nothing here synthesises sound; the events go to an [`EventSink`], like
//! [`super::performance::MusicPerformance`].
//!
//! Retail's groove: `gmRadarUI::UpdateGroove` sets the level from the objects in range and calls
//! `SoundManager::SetGrooveLevel`, which keeps `m_nGrooveLevel` and passes the master level to
//! the device; DirectMusic receives it as `GUID_PerfMasterGrooveLevel` with a one-byte size, so
//! the level is an `i8`. [`StylePerformance::set_groove_level`] is that input here. A change
//! applies at the next measure, when the next pattern is chosen.
//!
//! Choosing: the groove patterns that cover the level are candidates. Several may cover it. I
//! understand DirectMusic to choose among them at random; that is an assumption, not checked
//! against retail. Here a xorshift generator seeded by the caller picks one (the sequence is not
//! retail's). When none covers the level, the measure is silent.
//! The style's patterns are looped across the segment's length: a pattern measure repeats modulo
//! its length. Each note is pitched against the segment's chord track in effect at its time
//! ([`super::harmony::Harmony::from_segment`]; C major without one). Segment loop points, and the
//! command and marker tracks, are not used yet.

use std::sync::Arc;

use super::clock::{Boundary, MusicClock};
use super::harmony::Harmony;
use super::performance::{EventSink, MusicEvent, TimedEvent};
use super::segment::Segment;
use super::style::Style;
use super::style_play::{covers, pattern_measure_notes, StyleNoteSpan};

/// A pattern chosen for a measure: the measure's index, the pattern's index in the style, and
/// the groove level it was chosen at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternChoice {
    pub measure: usize,
    pub pattern: usize,
    pub groove: i8,
}

/// A note event not yet sent, at a music tick from the segment's start. Note-offs sort ahead of
/// note-ons at the same tick.
#[derive(Debug, Clone, Copy)]
struct Pending {
    tick: i32,
    on: bool,
    event: MusicEvent,
}

/// A segment's style playing in real time.
#[derive(Debug, Clone)]
pub struct StylePerformance {
    style: Arc<Style>,
    clock: MusicClock,
    /// The segment's chords, which pitch the notes.
    harmony: Harmony,
    /// The segment's length in ticks.
    length: i32,
    /// The groove level the next measure is chosen at.
    groove: i8,
    /// The generator state (xorshift32, never zero).
    rng: u32,
    /// The next measure to play: its index and start tick.
    next_measure: usize,
    next_tick: i32,
    /// Events of the chosen measures, sorted; `cursor` marks the first not yet sent.
    pending: Vec<Pending>,
    cursor: usize,
    choices: Vec<PatternChoice>,
    /// Notes sent as note-ons and not yet released, as (channel, note).
    sounding: Vec<(u32, u8)>,
    /// The time played so far, in seconds from the segment's start.
    position: f64,
    /// The segment's length in seconds.
    stop: f64,
    finished: bool,
}

impl StylePerformance {
    /// A performance of `segment`'s style (the style it references, resolved by the caller) at
    /// `groove`. `seed` seeds the choice among patterns that cover the level.
    pub fn new(segment: &Segment, style: Arc<Style>, groove: i8, seed: u32) -> Self {
        let clock = MusicClock::from_segment(segment);
        let length = segment.length.max(0);
        Self {
            style,
            stop: clock.seconds_at(length),
            clock,
            harmony: Harmony::from_segment(segment),
            length,
            groove,
            rng: seed.max(1),
            next_measure: 0,
            next_tick: 0,
            pending: Vec::new(),
            cursor: 0,
            choices: Vec::new(),
            sounding: Vec::new(),
            position: 0.0,
            finished: length == 0,
        }
    }

    /// Ends the performance now (`IDirectMusicPerformance8::StopEx` at the current time): the
    /// notes sounding are released here, and the rest of the segment is not sent.
    pub fn stop(&mut self, sink: &mut impl EventSink) {
        if self.finished {
            return;
        }
        for (channel, note) in std::mem::take(&mut self.sounding) {
            sink.event(&TimedEvent {
                seconds: self.position,
                event: MusicEvent::NoteOff { channel, note },
            });
        }
        self.pending.clear();
        self.cursor = 0;
        self.finished = true;
    }

    /// `SoundManager::SetGrooveLevel` (`gmRadarUI::UpdateGroove`): the level the next measure
    /// chooses its pattern at.
    pub fn set_groove_level(&mut self, level: i8) {
        self.groove = level;
    }

    /// The groove level the next measure chooses at.
    pub fn groove_level(&self) -> i8 {
        self.groove
    }

    /// The patterns chosen so far, in measure order (a measure no pattern covers is absent).
    pub fn choices(&self) -> &[PatternChoice] {
        &self.choices
    }

    /// The time played so far, in seconds.
    pub fn elapsed(&self) -> f64 {
        self.position
    }

    /// Whether the segment has played to its end.
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Plays `seconds` of real time, sending the events that fall in it to `sink`, stamped with
    /// the time since the segment's start. A measure is chosen when playback reaches its start,
    /// so a level set before that applies to it.
    pub fn advance(&mut self, seconds: f64, sink: &mut impl EventSink) {
        if self.finished || seconds <= 0.0 {
            return;
        }
        let target = self.position + seconds;
        while self.next_tick < self.length && self.clock.seconds_at(self.next_tick) <= target {
            self.play_measure();
        }
        let reach = target.min(self.stop);
        while let Some(&p) = self.pending.get(self.cursor) {
            let seconds = self.clock.seconds_at(p.tick);
            if seconds > reach {
                break;
            }
            self.track(p.event);
            sink.event(&TimedEvent {
                seconds,
                event: p.event,
            });
            self.cursor += 1;
        }
        if target >= self.stop {
            self.position = self.stop;
            self.finished = true;
        } else {
            self.position = target;
        }
    }

    /// Chooses the pattern for the next measure at the current groove level and queues its
    /// notes, clipped to the segment's end.
    fn play_measure(&mut self) {
        let measure = self.next_measure;
        let start = self.next_tick;
        let style = Arc::clone(&self.style);
        if let Some(index) = self.choose() {
            self.choices.push(PatternChoice {
                measure,
                pattern: index,
                groove: self.groove,
            });
            let pattern = &style.patterns[index];
            let band = pattern.band.as_ref().or(style.bands.first());
            let notes = pattern_measure_notes(&style, pattern, band, measure, &self.harmony, start);
            for span in notes {
                self.queue(&span, start);
            }
        }
        self.pending.drain(..self.cursor);
        self.cursor = 0;
        self.pending.sort_by_key(|p| (p.tick, p.on));
        self.next_measure += 1;
        self.next_tick = self.clock.next_boundary(start + 1, Boundary::Measure);
    }

    /// The index of the pattern for the groove level, among those covering it (random when
    /// several do); `None` when none covers it.
    fn choose(&mut self) -> Option<usize> {
        let level = self.groove;
        let covering: Vec<usize> = self
            .style
            .patterns
            .iter()
            .enumerate()
            .filter(|(_, p)| covers(p, level))
            .map(|(i, _)| i)
            .collect();
        match covering.len() {
            0 => None,
            1 => Some(covering[0]),
            n => Some(covering[self.next_random() as usize % n]),
        }
    }

    /// Queues a note of the measure that starts at `start`. A note that starts before the
    /// measure (a negative time offset) starts with it; one that starts at or after the segment's
    /// end is dropped, and one that runs past it is released there.
    fn queue(&mut self, span: &StyleNoteSpan, start: i32) {
        let on = (start + span.tick).max(start);
        if on >= self.length {
            return;
        }
        let off = (on + span.duration).min(self.length);
        self.pending.push(Pending {
            tick: on,
            on: true,
            event: MusicEvent::NoteOn {
                channel: span.channel,
                instrument: span.instrument,
                note: span.note,
                velocity: span.velocity,
            },
        });
        self.pending.push(Pending {
            tick: off,
            on: false,
            event: MusicEvent::NoteOff {
                channel: span.channel,
                note: span.note,
            },
        });
    }

    /// Keeps `sounding` in step with the events sent.
    fn track(&mut self, event: MusicEvent) {
        match event {
            MusicEvent::NoteOn { channel, note, .. } => self.sounding.push((channel, note)),
            MusicEvent::NoteOff { channel, note } => {
                if let Some(i) = self.sounding.iter().position(|k| *k == (channel, note)) {
                    self.sounding.swap_remove(i);
                }
            }
        }
    }

    fn next_random(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }
}
