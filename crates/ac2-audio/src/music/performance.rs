//! The performance of a segment's sequence tracks: its note-ons and note-offs in physical time,
//! from the segment's [`MusicClock`], with the band's instrument per performance channel. Nothing
//! here synthesises sound; the events go to an [`EventSink`].
//!
//! A sequence item (`DMUS_IO_SEQ_ITEM`) is a note at `time + offset` ticks lasting `duration`
//! ticks. Only note-on items (status 0x90) are played for now; other channel messages and the
//! curve items are not emitted yet.

use super::band::Band;
use super::clock::MusicClock;
use super::segment::{Segment, TrackData};

/// One note event on a performance channel (`pchannel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicEvent {
    NoteOn {
        channel: u32,
        /// The patch the band assigns to the channel (none without a band).
        instrument: Option<u32>,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u32,
        note: u8,
    },
}

/// A note event at a physical time from the segment's start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedEvent {
    pub seconds: f64,
    pub event: MusicEvent,
}

/// Receives the events a performance emits.
pub trait EventSink {
    fn event(&mut self, event: &TimedEvent);
}

/// Collects the events (the test sink).
impl EventSink for Vec<TimedEvent> {
    fn event(&mut self, event: &TimedEvent) {
        self.push(*event);
    }
}

/// The note events of `segment`'s sequence tracks, in time order (a note-off before a note-on
/// at the same tick). `band` maps each channel to its instrument and transposition.
pub fn note_events(segment: &Segment, clock: &MusicClock, band: Option<&Band>) -> Vec<TimedEvent> {
    // (tick, is_on, event): note-offs sort ahead of note-ons at the same tick.
    let mut ticked: Vec<(i32, bool, MusicEvent)> = Vec::new();
    for track in &segment.tracks {
        let TrackData::Sequence { events, .. } = &track.data else {
            continue;
        };
        for item in events.iter().filter(|e| e.status & 0xF0 == 0x90) {
            let on = (item.time + i32::from(item.offset)).max(0);
            let off = on + item.duration.max(0);
            let assignment =
                band.and_then(|b| b.instruments.iter().find(|i| i.pchannel == item.pchannel));
            let note = assignment.map_or(i32::from(item.byte1), |i| {
                i32::from(item.byte1) + i32::from(i.transpose)
            });
            let note = note.clamp(0, 127) as u8;
            ticked.push((
                on,
                true,
                MusicEvent::NoteOn {
                    channel: item.pchannel,
                    instrument: assignment.map(|i| i.effective_patch()),
                    note,
                    velocity: item.byte2,
                },
            ));
            ticked.push((
                off,
                false,
                MusicEvent::NoteOff {
                    channel: item.pchannel,
                    note,
                },
            ));
        }
    }
    ticked.sort_by_key(|(tick, on, _)| (*tick, *on));
    ticked
        .into_iter()
        .map(|(tick, _, event)| TimedEvent {
            seconds: clock.seconds_at(tick),
            event,
        })
        .collect()
}

/// A segment playing in real time: [`advance`](Self::advance) by seconds and the events due in
/// that span go to the sink, stamped with the elapsed time since the start. It ends at the
/// segment's length (sounding notes are released there), or wraps from its loop end to its loop
/// start when looping.
#[derive(Debug, Clone)]
pub struct MusicPerformance {
    events: Vec<TimedEvent>,
    /// The segment's length in seconds.
    stop: f64,
    /// `(loop start, loop end)` in seconds, when the segment loops.
    looping: Option<(f64, f64)>,
    /// The position in the segment, in seconds.
    position: f64,
    /// Elapsed time at the segment's start for the current pass: elapsed = base + position.
    base: f64,
    cursor: usize,
    /// Notes started and not yet released, as (channel, note).
    sounding: Vec<(u32, u8)>,
    finished: bool,
}

impl MusicPerformance {
    /// A performance of `segment` with `band`'s instruments. `looping` wraps from the loop end to
    /// the loop start (a segment with no loop region plays once either way).
    pub fn new(segment: &Segment, band: Option<&Band>, looping: bool) -> Self {
        let clock = MusicClock::from_segment(segment);
        let events = note_events(segment, &clock, band);
        let looping = (looping && segment.loop_end > segment.loop_start).then(|| {
            (
                clock.seconds_at(segment.loop_start),
                clock.seconds_at(segment.loop_end),
            )
        });
        Self {
            events,
            stop: clock.seconds_at(segment.length),
            looping: looping.filter(|(start, end)| end > start),
            position: 0.0,
            base: 0.0,
            cursor: 0,
            sounding: Vec::new(),
            finished: false,
        }
    }

    /// The events, in time order (before any playback).
    pub fn events(&self) -> &[TimedEvent] {
        &self.events
    }

    /// The time played so far, in seconds.
    pub fn elapsed(&self) -> f64 {
        self.base + self.position
    }

    /// Whether a non-looping segment has reached its end.
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Plays `seconds` of real time, sending the events that fall in it to `sink`.
    pub fn advance(&mut self, seconds: f64, sink: &mut impl EventSink) {
        if self.finished || seconds <= 0.0 {
            return;
        }
        let mut target = self.position + seconds;
        loop {
            let region_end = self.looping.map_or(self.stop, |(_, end)| end);
            let reach = target.min(region_end);
            while let Some(&event) = self.events.get(self.cursor) {
                if event.seconds > reach {
                    break;
                }
                self.cursor += 1;
                self.play(event, sink);
            }
            if target < region_end {
                self.position = target;
                return;
            }
            self.release_all(self.base + region_end, sink);
            match self.looping {
                Some((start, end)) => {
                    target = start + (target - end);
                    self.position = start;
                    self.base += end - start;
                    self.cursor = self.events.partition_point(|e| e.seconds < start);
                }
                None => {
                    self.position = self.stop;
                    self.finished = true;
                    return;
                }
            }
        }
    }

    /// Ends the performance now (`IDirectMusicPerformance8::StopEx` at the current time): the
    /// notes sounding are released here, and nothing more is sent.
    pub fn stop(&mut self, sink: &mut impl EventSink) {
        if self.finished {
            return;
        }
        self.release_all(self.base + self.position, sink);
        self.finished = true;
    }

    /// Sends one event; a note-off only goes out for a note that is sounding (a note that began
    /// before a loop wrap is not released by its own later note-off).
    fn play(&mut self, event: TimedEvent, sink: &mut impl EventSink) {
        let at = TimedEvent {
            seconds: self.base + event.seconds,
            event: event.event,
        };
        match event.event {
            MusicEvent::NoteOn { channel, note, .. } => {
                self.sounding.push((channel, note));
                sink.event(&at);
            }
            MusicEvent::NoteOff { channel, note } => {
                if let Some(i) = self.sounding.iter().position(|k| *k == (channel, note)) {
                    self.sounding.swap_remove(i);
                    sink.event(&at);
                }
            }
        }
    }

    /// Releases every note still sounding at `seconds` (elapsed time: the segment's end or loop
    /// end).
    fn release_all(&mut self, seconds: f64, sink: &mut impl EventSink) {
        for (channel, note) in std::mem::take(&mut self.sounding) {
            sink.event(&TimedEvent {
                seconds,
                event: MusicEvent::NoteOff { channel, note },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::band::BandInstrument;
    use super::super::riff::Guid;
    use super::super::segment::{SeqItem, Track};
    use super::*;

    const QUARTER: i32 = 768;

    fn note(time: i32, duration: i32, pchannel: u32, note: u8) -> SeqItem {
        SeqItem {
            time,
            duration,
            pchannel,
            offset: 0,
            status: 0x90,
            byte1: note,
            byte2: 100,
        }
    }

    fn track(class: u8, data: TrackData) -> Track {
        Track {
            class: Guid([class; 16]),
            position: 0,
            group: 0,
            config: 0,
            priority: 0,
            data,
        }
    }

    /// A 120 BPM segment of `length` ticks with `notes` on one sequence track.
    fn segment(length: i32, notes: Vec<SeqItem>) -> Segment {
        Segment {
            guid: None,
            name: None,
            repeats: 0,
            length,
            play_start: 0,
            loop_start: 0,
            loop_end: 0,
            resolution: 0,
            flags: 0,
            tracks: vec![
                track(1, TrackData::Tempo(vec![(0, 120.0)])),
                track(
                    2,
                    TrackData::Sequence {
                        events: notes,
                        curves: Vec::new(),
                    },
                ),
            ],
        }
    }

    fn on(channel: u32, note: u8) -> MusicEvent {
        MusicEvent::NoteOn {
            channel,
            instrument: None,
            note,
            velocity: 100,
        }
    }

    fn off(channel: u32, note: u8) -> MusicEvent {
        MusicEvent::NoteOff { channel, note }
    }

    #[test]
    fn note_events_are_timed_by_the_clock_and_ordered() {
        let seg = segment(
            2 * QUARTER,
            vec![note(0, QUARTER, 0, 60), note(QUARTER, QUARTER, 1, 64)],
        );
        let clock = MusicClock::from_segment(&seg);
        let events = note_events(&seg, &clock, None);
        let got: Vec<(f64, MusicEvent)> = events.iter().map(|e| (e.seconds, e.event)).collect();
        // The note-off at 0.5 s comes before the note-on at the same instant.
        assert_eq!(
            got,
            vec![
                (0.0, on(0, 60)),
                (0.5, off(0, 60)),
                (0.5, on(1, 64)),
                (1.0, off(1, 64)),
            ]
        );
    }

    #[test]
    fn the_offset_moves_a_note_and_the_band_assigns_instrument_and_transpose() {
        let mut shifted = note(0, QUARTER, 1, 60);
        shifted.offset = 384;
        let seg = segment(2 * QUARTER, vec![shifted, note(0, QUARTER, 0, 60)]);
        let band = Band {
            instruments: vec![BandInstrument {
                patch: 5,
                assign_patch: 0,
                note_ranges: [0; 4],
                pchannel: 1,
                flags: 0,
                pan: 0,
                volume: 0,
                transpose: 12,
                channel_priority: 0,
                pitch_bend_range: 0,
                collection: None,
            }],
            ..Band::default()
        };
        let clock = MusicClock::from_segment(&seg);
        let events = note_events(&seg, &clock, Some(&band));
        assert_eq!(events[0].event, on(0, 60));
        // Channel 1 starts half a quarter late (384 ticks) and is transposed up an octave.
        let late = events
            .iter()
            .find(|e| matches!(e.event, MusicEvent::NoteOn { channel: 1, .. }))
            .unwrap();
        assert_eq!(late.seconds, 0.25);
        assert_eq!(
            late.event,
            MusicEvent::NoteOn {
                channel: 1,
                instrument: Some(5),
                note: 72,
                velocity: 100,
            }
        );
    }

    #[test]
    fn advancing_sends_what_falls_in_each_span_and_ends_at_the_length() {
        // One second long; a quarter note, then a note-off past the end.
        let seg = segment(2 * QUARTER, vec![note(0, 4 * QUARTER, 0, 60)]);
        let mut perf = MusicPerformance::new(&seg, None, false);
        let mut sink: Vec<TimedEvent> = Vec::new();
        perf.advance(0.1, &mut sink);
        assert_eq!(sink.len(), 1);
        perf.advance(0.4, &mut sink);
        assert_eq!(sink.len(), 1);
        perf.advance(1.0, &mut sink);
        // The note is released at the segment's end (1 s), not at its own note-off (2 s).
        assert_eq!(sink.len(), 2);
        assert_eq!(sink[1].seconds, 1.0);
        assert_eq!(sink[1].event, off(0, 60));
        assert!(perf.finished());
        perf.advance(1.0, &mut sink);
        assert_eq!(sink.len(), 2);
    }

    #[test]
    fn a_looping_segment_wraps_to_its_loop_start() {
        // Half a second, looping over all of it; a note of an eighth at each start.
        let mut seg = segment(QUARTER, vec![note(0, QUARTER / 2, 0, 60)]);
        seg.loop_end = QUARTER;
        let mut perf = MusicPerformance::new(&seg, None, true);
        let mut sink: Vec<TimedEvent> = Vec::new();
        perf.advance(1.2, &mut sink);
        let times: Vec<f64> = sink.iter().map(|e| e.seconds).collect();
        assert_eq!(times, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert!(!perf.finished());
        assert!((perf.elapsed() - 1.2).abs() < 1e-12);
    }

    #[test]
    fn a_looping_note_that_started_before_the_wrap_is_released_there() {
        // A note from 0.25 s lasting 1 s crosses the 0.5 s loop end: it is released at the wrap,
        // and its own note-off (in the first pass, past the loop) never goes out.
        let mut seg = segment(QUARTER, vec![note(QUARTER / 2, 2 * QUARTER, 0, 60)]);
        seg.loop_end = QUARTER;
        let mut perf = MusicPerformance::new(&seg, None, true);
        let mut sink: Vec<TimedEvent> = Vec::new();
        perf.advance(0.6, &mut sink);
        let got: Vec<(f64, MusicEvent)> = sink.iter().map(|e| (e.seconds, e.event)).collect();
        assert_eq!(got, vec![(0.25, on(0, 60)), (0.5, off(0, 60))]);
    }
}
