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
//! A segment's notes play on its own band's instruments, or, when it has no band, on the
//! performance's band: the band of the band segment that started last ([`PerformanceBand`]). The
//! band's bound collections are the channels' sources ([`segment_sources`]).
//!
//! Retail's DirectMusic synthesizer is not reproduced, so this is the port's own engine. Modelled:
//! the DLS volume envelope (EG1, dB-shaped), the pitch envelope (EG2), the LFO (triangle, with its
//! frequency and start delay), the DLS2 vibrato LFO (`CONN_SRC_VIBRATO`, with its own frequency and
//! delay), the connection blocks' pitch, attenuation, pan and filter destinations (static, from key
//! and velocity, or moving with the LFO, vibrato and EG2; the units and the constants are in
//! [`modulation`]), the DLS2 resonant low-pass ([`filter`]: cutoff and Q, coefficients per block),
//! and the channel controllers the segments send: volume (CC7), expression (CC11), pan (CC10), the
//! mod wheel (CC1) and channel pressure (0xD0), which scale a route only when the block names them,
//! the sustain pedal (CC64), and the pitch bend with its RPN 0 range. The output is stereo (a mono
//! output takes the average).
//!
//! Not modelled: the DirectMusic and DLS2 synthesizer's other sources, the velocity and key curves
//! of retail, layered regions (all of them sound), and a polyphony cap of [`MAX_VOICES`] (the
//! oldest voice drops). The `wsmp` attenuation unit, the connection units and the envelope and
//! LFO shapes follow the port's reading, which the DLS Level 1 text did not confirm (see each
//! module). Controller gains are linear and the pan law is linear, both the port's choice. Each
//! segment has its own synthesizer, so channel numbers of two segments do not share state (the
//! queue's gap 2 still applies to the sound handles).
//!
//! Corpus coverage (the 61 collections; counts of connection blocks as the parser reads them).
//! art1 (43 collections): pan (`CONN_DST_PAN`) fixed in 24 blocks, EG1 attack 15, decay 6 (and one
//! key-number route), release 102, sustain 9, EG2 times fixed in 4, LFO frequency 2, LFO start delay
//! 2, LFO to pitch 3 (2 of them under the mod wheel), and the filter cutoff (`0x0500`) fixed at the
//! open maximum (`0x7FFFFFFF`) in all 121 blocks, which the filter bypasses. Not in the corpus at
//! all in art1: EG2 to pitch, the attenuation destination, velocity to attenuation, and the fixed
//! pitch. art2 (DLS2, 34 collections): pan 18, LFO frequency 62, LFO start delay 5, the filter cutoff
//! in 119 blocks (74 at the open maximum, 25 fixed cutoffs of 822 Hz to 8 kHz in 11 collections, 19
//! velocity routes of -2400 cents in 5 collections, one LFO route of 50 cents), the filter Q (`0x0501`)
//! in 2 (30 and 128 cB), the vibrato frequency (`0x0114`) in 56 blocks in 11 collections, the vibrato
//! start delay (`0x0115`) in 4, and the vibrato to pitch (`CONN_SRC_VIBRATO`, source `0x0009`) in 117
//! blocks in 12 collections: 116 at 50 cents (61 under the mod wheel, 55 under channel pressure
//! (`0x0008`)) and one at 25 cents with no control. A controlled route is silent at a zero control,
//! and the corpus's own sequences and curves never move the mod wheel or the pressure (style curves
//! are not emitted), so only the uncontrolled block sounds on the corpus's own data. The art1 and
//! art2 blocks of a note are read art2 first (it overrides art1 for a fixed destination), then the
//! region's before the instrument's.
//!
//! The performance band is the queue's ([`crate::music_thread`]): the band of the band segment that
//! started last, or whose band track reached an item last. A band-less segment already playing
//! maps its later notes by the new band and its voice takes the band's sources (the queue's
//! `BandChange`, sent as `SinkCommand::MusicBands`); a segment with a band track of its own keeps
//! its own band. The notes map by the band at their own instant; the voice's sources change at the
//! block that carries the change, so a note in that block may sound on the sources before it.

mod channel;
mod envelope;
mod filter;
mod gm;
mod lfo;
mod modulation;
mod voice;

#[cfg(test)]
mod control_tests;
#[cfg(test)]
mod dat_tests;
#[cfg(test)]
mod filter_dat_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
pub(crate) mod tests;

use std::sync::Arc;

use channel::ChannelState;
use modulation::resolve as resolve_modulation;
use voice::{Mix, Voice};

use crate::music::band::Band;
use crate::music::clock::MusicClock;
use crate::music::dls::Collection;
use crate::music::library::LoadedMusic;
use crate::music::performance::{EventSink, MusicEvent, TimedEvent};
use crate::music::segment::TrackData;
use crate::music::MusicObject;

pub use gm::{gm_path_from_env, load_gm_file, GmLoad, GmLoadError, GM_ENV};

/// The most voices sounding at once; a new note past it drops the oldest.
pub const MAX_VOICES: usize = 64;

/// Where a channel's patches come from.
#[derive(Debug, Clone, PartialEq)]
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

/// A segment's channel bindings, shared with the sound that plays it (`(performance channel,
/// source)` for each of its band's instruments).
pub type ChannelSources = Arc<[(u32, Source)]>;

/// The channel bindings of a loaded segment's style: the bands the style references (the style's
/// own band and each pattern's band), which the style's notes take their instruments from. The
/// segment's own band track is not here (see [`performance_bands`]).
pub fn style_sources(music: &LoadedMusic) -> Vec<(u32, Source)> {
    let MusicObject::Segment(segment) = &music.segment else {
        return Vec::new();
    };
    let Some(style) = music.style_of(segment) else {
        return Vec::new();
    };
    let mut bands: Vec<&Band> = style.bands.iter().collect();
    bands.extend(style.patterns.iter().filter_map(|p| p.band.as_ref()));
    bands
        .into_iter()
        .flat_map(|band| band_sources(band, music))
        .collect()
}

/// A segment's own band with its channels' sources, resolved on the loader thread. It is the
/// performance's band while it plays: a segment with no band of its own plays its notes on the
/// band of the band segment that started last (DirectMusic's performance band, which band tracks
/// set).
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceBand {
    pub band: Arc<Band>,
    pub sources: ChannelSources,
}

/// The bands of a segment's band track, each with the seconds from the segment's start its item
/// reaches, in the track's order (empty with no band track). The first is the band the segment
/// starts on.
pub fn performance_bands(music: &LoadedMusic) -> Vec<(f64, Arc<PerformanceBand>)> {
    let MusicObject::Segment(segment) = &music.segment else {
        return Vec::new();
    };
    let clock = MusicClock::from_segment(segment);
    let Some(items) = segment.tracks.iter().find_map(|t| match &t.data {
        TrackData::Bands(items) => Some(items),
        _ => None,
    }) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|(time, band)| {
            (
                clock.seconds_at(*time),
                Arc::new(PerformanceBand {
                    band: Arc::new(band.clone()),
                    sources: band_sources(band, music).into(),
                }),
            )
        })
        .collect()
}

/// The first band of a segment's band track (see [`performance_bands`]), if it has one.
pub fn performance_band(music: &LoadedMusic) -> Option<PerformanceBand> {
    performance_bands(music)
        .into_iter()
        .next()
        .map(|(_, band)| PerformanceBand::clone(&band))
}

/// The synthesizer at one output rate, mixing f32 stereo (or mono) frames.
pub struct Synth {
    rate: u32,
    gm: Option<Arc<Collection>>,
    /// The style's channel bindings, in the order they were made (see [`Synth::bind`]).
    sources: Vec<(u32, Source)>,
    /// The band's channel bindings (the performance's or the segment's own band), searched before
    /// the style's: they change with the band (see [`Synth::set_band_sources`]).
    band_sources: Vec<(u32, Source)>,
    voices: Vec<Voice>,
    /// The controller state of each channel the segment has sent a controller for.
    controls: Vec<(u32, ChannelState)>,
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
            band_sources: Vec::new(),
            voices: Vec::new(),
            controls: Vec::new(),
            pending: Vec::new(),
        }
    }

    /// Sets the General MIDI set (from [`load_gm_file`]); `None` makes GM channels silent.
    pub fn set_gm(&mut self, gm: Option<Arc<Collection>>) {
        self.gm = gm;
    }

    /// Adds a source for a channel. A note on the channel takes the first source whose
    /// collection holds its patch (so bands that share a channel number stay apart).
    pub fn bind(&mut self, channel: u32, source: Source) {
        self.sources.push((channel, source));
    }

    /// Replaces the style's channel bindings with `sources` (a segment starting again).
    pub fn set_sources(&mut self, sources: &[(u32, Source)]) {
        self.sources = sources.to_vec();
    }

    /// Replaces the band's channel bindings (the band changed; see [`band_sources`]).
    pub fn set_band_sources(&mut self, sources: &[(u32, Source)]) {
        self.band_sources = sources.to_vec();
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

    /// The channel bindings: the band's, then the style's.
    pub fn bindings(&self) -> Vec<(u32, Source)> {
        self.band_sources
            .iter()
            .chain(self.sources.iter())
            .cloned()
            .collect()
    }

    /// The channel's controller state (the default for a channel nothing has controlled).
    pub fn channel(&self, channel: u32) -> ChannelState {
        self.controls
            .iter()
            .find(|(c, _)| *c == channel)
            .map_or_else(ChannelState::default, |(_, state)| state.clone())
    }

    fn channel_mut(&mut self, channel: u32) -> &mut ChannelState {
        let at = match self.controls.iter().position(|(c, _)| *c == channel) {
            Some(at) => at,
            None => {
                self.controls.push((channel, ChannelState::default()));
                self.controls.len() - 1
            }
        };
        &mut self.controls[at].1
    }

    /// The live mix of `channel` (unity for a channel with no controller).
    fn mix_of(&self, channel: u32) -> Mix {
        self.controls
            .iter()
            .find(|(c, _)| *c == channel)
            .map_or(Mix::UNITY, |(_, state)| Mix {
                gain: state.volume_gain() * state.expression_gain(),
                pitch: (state.bend_semitones() / 12.0).exp2(),
                pan: state.pan_position(),
                mod_wheel: state.mod_wheel_fraction(),
                pressure: state.pressure_fraction(),
            })
    }

    /// The segment ends: every note releases (its voice plays out its release), and the events
    /// not yet applied are dropped.
    pub fn stop(&mut self) {
        self.pending.clear();
        for voice in &mut self.voices {
            voice.release();
        }
    }

    /// The collection a note on `channel` with `patch` plays from: the first bound source whose
    /// collection holds the patch (the GM set for a GM source; none for a missing one).
    fn collection_for(&self, channel: u32, patch: u32) -> Option<Arc<Collection>> {
        self.band_sources
            .iter()
            .chain(self.sources.iter())
            .filter(|(c, _)| *c == channel)
            .find_map(|(_, source)| {
                let collection = match source {
                    Source::Collection(c) => Arc::clone(c),
                    Source::GeneralMidi => Arc::clone(self.gm.as_ref()?),
                    Source::Missing => return None,
                };
                collection.instrument(patch).is_some().then_some(collection)
            })
    }

    fn start_note(&mut self, channel: u32, patch: u32, note: u8, velocity: u8) {
        let Some(collection) = self.collection_for(channel, patch) else {
            return;
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
            let modulation = resolve_modulation(
                region
                    .articulation
                    .iter()
                    .chain(instrument.articulation.iter()),
                note,
                velocity,
            );
            let started = Voice::start(
                channel,
                note,
                velocity,
                region,
                wave,
                &modulation,
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

    /// A key up: the notes on `channel` at `note` release, unless the sustain pedal holds them.
    fn release_note(&mut self, channel: u32, note: u8) {
        let pedal = self.channel(channel).sustain;
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.channel == channel && v.note == note)
        {
            if pedal {
                v.key_off_under_pedal();
            } else {
                v.release();
            }
        }
    }

    /// The sustain pedal goes up: the notes it held on `channel` release.
    fn pedal_up(&mut self, channel: u32) {
        for v in self.voices.iter_mut().filter(|v| v.channel == channel) {
            v.pedal_up();
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
            MusicEvent::Control {
                channel,
                controller,
                value,
            } => {
                if self.channel_mut(channel).control(controller, value) {
                    self.pedal_up(channel);
                }
            }
            MusicEvent::PitchBend { channel, value } => self.channel_mut(channel).set_bend(value),
            MusicEvent::ChannelPressure { channel, value } => {
                self.channel_mut(channel).pressure = value;
            }
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
    /// timeline time `start`. Events due at a frame's time are applied before that frame. A
    /// stereo output takes each voice's left and right; a mono output takes their average.
    pub fn render(&mut self, out: &mut [f32], channels: usize, start: f64) {
        out.fill(0.0);
        let channels = channels.max(1);
        let dt = 1.0 / f64::from(self.rate);
        for (i, frame) in out.chunks_exact_mut(channels).enumerate() {
            self.apply_due(start + i as f64 * dt);
            let (mut left, mut right) = (0.0f64, 0.0f64);
            for i in 0..self.voices.len() {
                let mix = self.mix_of(self.voices[i].channel);
                let (l, r) = self.voices[i].next(dt, mix);
                left += l;
                right += r;
            }
            self.voices.retain(|v| !v.finished());
            if channels > 1 {
                frame[0] = left.clamp(-1.0, 1.0) as f32;
                frame[1] = right.clamp(-1.0, 1.0) as f32;
            } else {
                frame[0] = ((left + right) * 0.5).clamp(-1.0, 1.0) as f32;
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
