//! One segment playing on the queue's clock: the segment's sequence performance and, when it
//! names a style, its style performance. Each performance counts from its own start, so the
//! events are stamped with the playback's start to put them on the queue's timeline.

use std::sync::Arc;

use crate::music::band::Band;
use crate::music::clock::MusicClock;
use crate::music::library::LoadedMusic;
use crate::music::performance::{EventSink, MusicPerformance, TimedEvent};
use crate::music::segment::{Segment, TrackData};
use crate::music::style::Style;
use crate::music::style_performance::{PatternChoice, StylePerformance};
use crate::music::MusicObject;

/// Hands events to a buffer, moved by the playback's start (the queue's clock).
struct Stamp<'a> {
    base: f64,
    out: &'a mut Vec<TimedEvent>,
}

impl EventSink for Stamp<'_> {
    fn event(&mut self, event: &TimedEvent) {
        self.out.push(TimedEvent {
            seconds: self.base + event.seconds,
            event: event.event,
        });
    }
}

/// A running segment (`IDirectMusicSegmentState`): its performances and where it is in time.
pub(super) struct Playback {
    id: u32,
    /// The queue's time at which it started.
    start: f64,
    /// The segment's length in seconds.
    length: f64,
    /// Seconds played so far.
    elapsed: f64,
    /// Whether it has reached its end or been stopped.
    ended: bool,
    sequence: MusicPerformance,
    style: Option<StylePerformance>,
}

/// The segment's band: the first of its band tracks (`DMUS_IO_BAND_ITEM`, by logical time).
fn first_band(segment: &Segment) -> Option<&Band> {
    segment.tracks.iter().find_map(|track| match &track.data {
        TrackData::Bands(bands) => bands.first().map(|(_, band)| band),
        _ => None,
    })
}

/// The style the segment references first (its styles track), when the DATs have it.
fn referenced_style(music: &LoadedMusic, segment: &Segment) -> Option<Arc<Style>> {
    let reference = segment.tracks.iter().find_map(|track| match &track.data {
        TrackData::Styles(styles) => styles.first().map(|(_, reference)| reference),
        _ => None,
    })?;
    match music.resolve(reference)? {
        MusicObject::Style(style) => Some(Arc::clone(style)),
        _ => None,
    }
}

impl Playback {
    /// The playback of `music` (a segment) starting at `start`, at `groove`. `seed` seeds the
    /// style's pattern choice. `None` when the object is not a segment.
    pub(super) fn new(
        id: u32,
        start: f64,
        music: &LoadedMusic,
        groove: i8,
        seed: u32,
    ) -> Option<Self> {
        let MusicObject::Segment(segment) = &music.segment else {
            return None;
        };
        let clock = MusicClock::from_segment(segment);
        let sequence = MusicPerformance::new(segment, first_band(segment), false);
        let style = referenced_style(music, segment)
            .map(|style| StylePerformance::new(segment, style, groove, seed));
        Some(Self {
            id,
            start,
            length: clock.seconds_at(segment.length.max(0)),
            elapsed: 0.0,
            ended: false,
            sequence,
            style,
        })
    }

    pub(super) fn id(&self) -> u32 {
        self.id
    }

    pub(super) fn length(&self) -> f64 {
        self.length
    }

    pub(super) fn ended(&self) -> bool {
        self.ended
    }

    /// The seconds left before the segment's end.
    pub(super) fn remaining(&self) -> f64 {
        (self.length - self.elapsed).max(0.0)
    }

    /// Plays `seconds` more, sending the events due to `out`.
    pub(super) fn advance(&mut self, seconds: f64, out: &mut Vec<TimedEvent>) {
        let mut stamp = Stamp {
            base: self.start,
            out,
        };
        self.sequence.advance(seconds, &mut stamp);
        if let Some(style) = self.style.as_mut() {
            style.advance(seconds, &mut stamp);
        }
        self.elapsed += seconds;
    }

    /// Marks the segment ended at its length (its performances have sent everything).
    pub(super) fn finish(&mut self) {
        self.elapsed = self.length;
        self.ended = true;
    }

    /// Stops the segment now, releasing the notes sounding (`StopEx` at the current time).
    pub(super) fn stop(&mut self, out: &mut Vec<TimedEvent>) {
        let mut stamp = Stamp {
            base: self.start,
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
