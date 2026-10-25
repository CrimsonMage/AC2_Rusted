//! The software synthesizer: the music queue's note events become sound. A [`Synth`] is an
//! [`EventSink`]: note-ons and note-offs (stamped with the segment's timeline in seconds) are
//! queued and applied when [`Synth::render`] reaches them. Each note picks its instrument from
//! the channel's source (the band's DLS collection, or the GM file) by patch, and plays every
//! region covering its key and velocity.
//!
//! No file is read here. Collections come in on the loader thread, inside the segment's
//! [`LoadedMusic`] closure (a band's collection reference resolves there), and the GM file is
//! read by [`load_gm_file`] on a worker. A band instrument with no collection and no GM set
//! stays silent.
//!
//! Retail's DirectMusic synthesizer is not reproduced, so this is the port's own engine. Gaps:
//! the DLS volume envelope is linear per segment (no dB-shaped decay); pitch and pan connection
//! blocks, LFOs, key-scaled envelopes and the velocity transform are not applied (velocity is
//! linear); controller messages, the sustain pedal and channel volume are not sent; layered
//! regions all sound; a polyphony cap of [`MAX_VOICES`] drops the oldest voice; the
//! sustain-level unit and the `wsmp` attenuation unit follow the reader's notes, unconfirmed
//! against the DLS Level 1 text.

mod envelope;
mod gm;
mod voice;

#[cfg(test)]
mod dat_tests;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use voice::Voice;

use crate::music::band::Band;
use crate::music::dls::Collection;
use crate::music::library::LoadedMusic;
use crate::music::performance::{EventSink, MusicEvent, TimedEvent};
use crate::music::MusicObject;

pub use gm::{gm_path_from_env, load_gm_file, GmLoadError, GM_ENV};

/// The most voices sounding at once; a new note past it drops the oldest.
pub const MAX_VOICES: usize = 64;

/// Where a channel's patches come from.
#[derive(Debug, Clone)]
pub enum Source {
    /// A DLS collection the band references.
    Collection(Arc<Collection>),
    /// The General MIDI set (the file the user supplied, if any).
    GeneralMidi,
    /// A reference that did not resolve: silent.
    Missing,
}

/// The sources of a band's instruments, by performance channel, resolved against the segment's
/// closure (a reference to a collection the DATs do not hold is [`Source::Missing`]).
pub fn band_sources(band: &Band, music: &LoadedMusic) -> Vec<(u32, Source)> {
    band.instruments
        .iter()
        .map(|instrument| {
            let source = match &instrument.collection {
                None => Source::GeneralMidi,
                Some(reference) => match music.resolve(reference) {
                    Some(MusicObject::Collection(c)) => Source::Collection(Arc::clone(c)),
                    _ => Source::Missing,
                },
            };
            (instrument.pchannel, source)
        })
        .collect()
}

/// The synthesizer at one output rate, mixing f32 stereo (or mono) frames.
pub struct Synth {
    rate: u32,
    gm: Option<Arc<Collection>>,
    sources: Vec<(u32, Source)>,
    voices: Vec<Voice>,
    /// The events not yet applied, in time order.
    pending: Vec<TimedEvent>,
}

impl Synth {
    /// A synthesizer writing `rate` frames per second, with no GM set and no channels bound.
    pub fn new(rate: u32) -> Self {
        Self {
            rate: rate.max(1),
            gm: None,
            sources: Vec::new(),
            voices: Vec::new(),
            pending: Vec::new(),
        }
    }

    /// Sets the General MIDI set (from [`load_gm_file`]); `None` makes GM channels silent.
    pub fn set_gm(&mut self, gm: Option<Arc<Collection>>) {
        self.gm = gm;
    }

    /// Binds a channel's patches to `source` (the last binding for a channel wins).
    pub fn bind(&mut self, channel: u32, source: Source) {
        self.sources.retain(|(c, _)| *c != channel);
        self.sources.push((channel, source));
    }

    /// Binds every channel of a band to its source (see [`band_sources`]).
    pub fn bind_band(&mut self, band: &Band, music: &LoadedMusic) {
        for (channel, source) in band_sources(band, music) {
            self.bind(channel, source);
        }
    }

    /// Voices sounding.
    pub fn voices(&self) -> usize {
        self.voices.len()
    }

    fn source_of(&self, channel: u32) -> Source {
        self.sources
            .iter()
            .find(|(c, _)| *c == channel)
            .map_or(Source::Missing, |(_, s)| s.clone())
    }

    fn start_note(&mut self, channel: u32, patch: u32, note: u8, velocity: u8) {
        let collection = match self.source_of(channel) {
            Source::Collection(c) => c,
            Source::GeneralMidi => match &self.gm {
                Some(gm) => Arc::clone(gm),
                None => return,
            },
            Source::Missing => return,
        };
        let Some(instrument) = collection.instrument(patch) else {
            return;
        };
        // A note that sounds again ends its earlier voice (a retrigger).
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.channel == channel && v.note == note)
        {
            v.release();
        }
        for region in instrument
            .regions
            .iter()
            .filter(|r| r.covers(note, velocity))
        {
            let wave = collection
                .waves
                .get(region.link.table_index as usize)
                .and_then(|w| w.as_deref());
            let Some(wave) = wave else {
                continue;
            };
            let started = Voice::start(
                channel,
                note,
                velocity,
                region,
                wave,
                instrument.articulation.as_ref(),
                self.rate,
            );
            if let Some(voice) = started {
                if self.voices.len() >= MAX_VOICES {
                    self.voices.remove(0);
                }
                self.voices.push(voice);
            }
        }
    }

    fn release_note(&mut self, channel: u32, note: u8) {
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.channel == channel && v.note == note)
        {
            v.release();
        }
    }

    fn apply(&mut self, event: TimedEvent) {
        match event.event {
            MusicEvent::NoteOn {
                channel,
                instrument,
                note,
                velocity,
            } => {
                if velocity == 0 {
                    self.release_note(channel, note);
                } else if let Some(patch) = instrument {
                    self.start_note(channel, patch, note, velocity);
                }
            }
            MusicEvent::NoteOff { channel, note } => self.release_note(channel, note),
        }
    }

    fn apply_due(&mut self, now: f64) {
        let mut due = 0;
        while due < self.pending.len() && self.pending[due].seconds <= now {
            let event = self.pending[due];
            self.apply(event);
            due += 1;
        }
        self.pending.drain(..due);
    }

    /// Fills `out` (interleaved, `channels` per frame) with the voices' sum for the frames from
    /// timeline time `start`. Events due at a frame's time are applied before that frame.
    pub fn render(&mut self, out: &mut [f32], channels: usize, start: f64) {
        out.fill(0.0);
        let channels = channels.max(1);
        let dt = 1.0 / f64::from(self.rate);
        for (i, frame) in out.chunks_exact_mut(channels).enumerate() {
            self.apply_due(start + i as f64 * dt);
            let mut sum = 0.0f64;
            for voice in &mut self.voices {
                sum += voice.next(dt);
            }
            self.voices.retain(|v| !v.finished());
            let s = sum.clamp(-1.0, 1.0) as f32;
            frame[0] = s;
            if channels > 1 {
                frame[1] = s;
            }
        }
    }
}

impl EventSink for Synth {
    fn event(&mut self, event: &TimedEvent) {
        let at = self.pending.partition_point(|p| p.seconds <= event.seconds);
        self.pending.insert(at, *event);
    }
}
