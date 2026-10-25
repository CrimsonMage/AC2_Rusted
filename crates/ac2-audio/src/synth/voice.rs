//! One sounding region: its wave read at the note's pitch, looped while the note is held, shaped
//! by the volume envelope and scaled by velocity and the region's attenuation, and placed by its
//! pan.
//!
//! Pitch: the wave plays at `(note - unity) + fine / 100` semitones (DLS `wsmp`: the unity note and
//! the signed fine tune in cents), times the wave's rate over the output rate, times the connection
//! blocks' cents (static offsets, the LFO, vibrato and EG2 routes; see [`super::modulation`]) and
//! the channel's pitch bend. Samples are read with linear interpolation in f64.
//!
//! Filter: the note's low-pass (see [`super::filter`]) runs on the sample after the gain. Its
//! cutoff is the block's absolute cents plus the cutoff routes, and its coefficients are recomputed
//! at each block's start.
//!
//! Gain: velocity over 127 (linear), times the `wsmp` attenuation in 1/655360 dB, times the
//! connection blocks' attenuation in dB (static and LFO routes), times the volume envelope (dB
//! shaped) and the channel's volume and expression. A forward loop (`WLOOP_TYPE_FORWARD`) wraps
//! while the note is held; after note-off the voice plays on through the sample's tail, under the
//! release.
//!
//! Pan: the channel's pan position and the voice's (the static pan and its routes) add, clamped to
//! -1..1. The left gain is `min(1, 1 - x)` and the right `min(1, 1 + x)`: unity at centre, silent
//! on the far side at the edges, linear between (the port's law; the DLS pan law was not read).

use std::sync::Arc;

use super::envelope::Envelope;
use super::filter::Filter;
use super::lfo::Lfo;
use super::modulation::{Control, Dest, NoteModulation, Route, Source};
use crate::music::dls::{DlsWave, Region, WaveSample};

/// `WLOOP_TYPE_FORWARD`.
const LOOP_FORWARD: u32 = 0;
/// `wsmp` unity note when neither the region nor the wave gives one.
const DEFAULT_UNITY: u16 = 60;
/// The pan position in 0.1-percent units of a full pan (`±500`).
const PAN_FULL: f64 = 500.0;

/// A channel's live values for one frame: the gain it scales a voice by (volume times
/// expression), its pitch bend's frequency factor, its pan position, the mod wheel and the
/// channel pressure (both 0 to 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Mix {
    pub gain: f64,
    pub pitch: f64,
    pub pan: f64,
    pub mod_wheel: f64,
    pub pressure: f64,
}

impl Mix {
    /// A channel with no controller moved.
    pub const UNITY: Mix = Mix {
        gain: 1.0,
        pitch: 1.0,
        pan: 0.0,
        mod_wheel: 0.0,
        pressure: 0.0,
    };
}

/// What the routes add to a voice in one frame.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Offsets {
    cents: f64,
    db: f64,
    pan: f64,
    cutoff: f64,
}

/// This frame's modulator values: the LFO, the EG2 level and the vibrato LFO.
#[derive(Debug, Clone, Copy)]
struct Modulators {
    lfo: f64,
    eg2: f64,
    vibrato: f64,
}

/// A note sounding from one region.
pub(super) struct Voice {
    pub channel: u32,
    pub note: u8,
    /// The wave's first channel, interleaved with `stride` channels per frame.
    sample: Arc<[f32]>,
    stride: usize,
    frames: usize,
    /// The read position in source frames.
    position: f64,
    /// Source frames per output frame, at the note's pitch.
    step: f64,
    gain: f64,
    /// `(start, end)` of the forward loop in source frames.
    loop_range: Option<(f64, f64)>,
    held: bool,
    /// The key is up but the sustain pedal holds the note (see [`Voice::key_off_under_pedal`]).
    pedal: bool,
    envelope: Envelope,
    /// The pitch envelope (EG2), read by routes.
    eg2: Envelope,
    lfo: Option<Lfo>,
    /// The vibrato LFO (`CONN_SRC_VIBRATO`).
    vibrato: Option<Lfo>,
    /// The low-pass filter, when the note's blocks set a cutoff.
    filter: Option<Filter>,
    /// The static cents, dB of attenuation and pan position (the note's connection blocks).
    pitch_cents: f64,
    attenuation_db: f64,
    pan: f64,
    routes: Vec<Route>,
    done: bool,
}

impl Voice {
    /// The voice for `region` playing `wave` at `note` with `velocity` (1 to 127), at an output
    /// rate of `rate`, under the note's connection blocks `modulation`. Returns `None` for a wave
    /// with no samples.
    pub fn start(
        channel: u32,
        note: u8,
        velocity: u8,
        region: &Region,
        wave: &DlsWave,
        modulation: &NoteModulation,
        rate: u32,
    ) -> Option<Self> {
        let stride = usize::from(wave.pcm.channels.max(1));
        let frames = wave.pcm.samples.len() / stride;
        if frames == 0 || rate == 0 || wave.pcm.rate == 0 {
            return None;
        }
        // The region's `wsmp` overrides the wave's own.
        let sample: Option<&WaveSample> = region.sample.as_ref().or(wave.sample.as_ref());
        let unity = sample.map_or(DEFAULT_UNITY, |s| s.unity_note);
        let fine = sample.map_or(0, |s| s.fine_tune);
        let attenuation = sample.map_or(0, |s| s.attenuation);
        let loop_range = sample.and_then(|s| {
            let l = s
                .loops
                .iter()
                .find(|l| l.kind == LOOP_FORWARD && l.length > 0)?;
            let start = f64::from(l.start);
            let end = (f64::from(l.start) + f64::from(l.length)).min(frames as f64);
            (end > start).then_some((start, end))
        });
        let semitones = f64::from(note) - f64::from(unity) + f64::from(fine) / 100.0;
        let ratio = (semitones / 12.0).exp2();
        let step = f64::from(wave.pcm.rate) / f64::from(rate) * ratio;
        let velocity_gain = f64::from(velocity.min(127)) / 127.0;
        let attenuation_gain = 10f64.powf(f64::from(attenuation) / 655_360.0 / 20.0);
        Some(Self {
            channel,
            note,
            sample: Arc::clone(&wave.pcm.samples),
            stride,
            frames,
            position: 0.0,
            step,
            gain: velocity_gain * attenuation_gain,
            loop_range,
            held: true,
            pedal: false,
            envelope: Envelope::new(modulation.eg1),
            eg2: Envelope::new(modulation.eg2),
            lfo: modulation.lfo.map(|lfo| Lfo::new(lfo.frequency, lfo.delay)),
            vibrato: modulation
                .vibrato
                .map(|lfo| Lfo::new(lfo.frequency, lfo.delay)),
            filter: modulation
                .filter
                .map(|f| Filter::new(f.cutoff_cents, f.q, rate)),
            pitch_cents: modulation.pitch_cents,
            attenuation_db: modulation.attenuation_db,
            pan: modulation.pan,
            routes: modulation.routes.clone(),
            done: false,
        })
    }

    /// Note-off: the loop stops and the envelope releases.
    pub fn release(&mut self) {
        self.held = false;
        self.pedal = false;
        self.envelope.release();
        self.eg2.release();
    }

    /// Note-off with the sustain pedal down: the key is up, but the note keeps sounding (and its
    /// loop plays on) until the pedal goes up.
    pub fn key_off_under_pedal(&mut self) {
        if self.held {
            self.held = false;
            self.pedal = true;
        }
    }

    /// The sustain pedal goes up: a note the pedal held is released.
    pub fn pedal_up(&mut self) {
        if self.pedal {
            self.release();
        }
    }

    pub fn finished(&self) -> bool {
        self.done
    }

    fn looping(&self) -> bool {
        (self.held || self.pedal) && self.loop_range.is_some()
    }

    fn read(&self) -> f64 {
        let frames = self.frames;
        let index = self.position.floor() as usize;
        if index >= frames {
            return 0.0;
        }
        let t = self.position - index as f64;
        let next = if index + 1 < frames {
            index + 1
        } else if let (true, Some((start, _))) = (self.looping(), self.loop_range) {
            start as usize
        } else {
            index
        };
        let a = f64::from(self.sample[index * self.stride]);
        let b = f64::from(self.sample[next * self.stride]);
        a + (b - a) * t
    }

    /// The per-frame values of the routes: what they add to the pitch, attenuation, pan and cutoff.
    fn route_offsets(&self, modulators: Modulators, mix: Mix) -> Offsets {
        let mut offsets = Offsets::default();
        for route in &self.routes {
            let source = match route.source {
                Source::Lfo => modulators.lfo,
                Source::Eg2 => modulators.eg2,
                Source::Vibrato => modulators.vibrato,
            };
            let depth = match route.control {
                Control::None => 1.0,
                Control::ModWheel => mix.mod_wheel,
                Control::ChannelPressure => mix.pressure,
            };
            let value = source * route.scale * depth;
            match route.dest {
                Dest::Pitch => offsets.cents += value,
                Dest::Attenuation => offsets.db += value,
                Dest::Pan => offsets.pan += value / PAN_FULL,
                Dest::Cutoff => offsets.cutoff += value,
            }
        }
        offsets
    }

    /// The voice's output for one frame of `dt` seconds, under the channel's live `mix`: the
    /// left and right samples.
    pub fn next(&mut self, dt: f64, mix: Mix) -> (f64, f64) {
        if self.done {
            return (0.0, 0.0);
        }
        let level = self.envelope.tick(dt);
        let modulators = Modulators {
            lfo: self.lfo.as_mut().map_or(0.0, |lfo| lfo.tick(dt)),
            eg2: self.eg2.tick(dt),
            vibrato: self.vibrato.as_mut().map_or(0.0, |lfo| lfo.tick(dt)),
        };
        let offsets = self.route_offsets(modulators, mix);
        let total_db = self.attenuation_db + offsets.db;
        let gain = self.gain * mix.gain * level * 10f64.powf(-total_db / 20.0);
        let x = (self.pan + offsets.pan + mix.pan).clamp(-1.0, 1.0);
        let (left, right) = ((1.0 - x).min(1.0), (1.0 + x).min(1.0));
        let mut sample = self.read() * gain;
        if let Some(filter) = self.filter.as_mut() {
            sample = filter.process(sample, offsets.cutoff);
        }
        let out = (sample * left, sample * right);
        let bend = (self.pitch_cents + offsets.cents) / 1200.0;
        self.position += self.step * bend.exp2() * mix.pitch;
        if let (true, Some((start, end))) = (self.looping(), self.loop_range) {
            while self.position >= end {
                self.position -= end - start;
            }
        }
        if self.position >= self.frames as f64 && !self.looping() {
            self.done = true;
        }
        if self.envelope.finished() {
            self.done = true;
        }
        out
    }
}
