//! One sounding region: its wave read at the note's pitch, looped while the note is held, shaped
//! by the volume envelope and scaled by velocity and the region's attenuation.
//!
//! Pitch: the wave plays at `(note - unity) + fine / 100` semitones (DLS `wsmp`: the unity note
//! and the signed fine tune in cents), times the wave's rate over the output rate. Samples are
//! read with linear interpolation in f64. Gain: velocity over 127 (linear), times the `wsmp`
//! attenuation in 1/655360 dB. A forward loop (`WLOOP_TYPE_FORWARD`) wraps while the note is
//! held; after note-off the voice plays on through the sample's tail, under the release.

use std::sync::Arc;

use super::envelope::{Envelope, EnvelopeParams};
use crate::music::dls::{Articulation, DlsWave, Region, WaveSample};

/// `WLOOP_TYPE_FORWARD`.
const LOOP_FORWARD: u32 = 0;
/// `wsmp` unity note when neither the region nor the wave gives one.
const DEFAULT_UNITY: u16 = 60;

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
    /// Source frames per output frame.
    step: f64,
    gain: f64,
    /// `(start, end)` of the forward loop in source frames.
    loop_range: Option<(f64, f64)>,
    held: bool,
    envelope: Envelope,
    done: bool,
}

impl Voice {
    /// The voice for `region` playing `wave` at `note` with `velocity` (1 to 127), at an output
    /// rate of `rate`. `instrument` is the instrument's articulation (the region's comes first).
    /// Returns `None` for a wave with no samples.
    pub fn start(
        channel: u32,
        note: u8,
        velocity: u8,
        region: &Region,
        wave: &DlsWave,
        instrument: Option<&Articulation>,
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
        let params = EnvelopeParams::from_articulations(
            region.articulation.iter().chain(instrument.iter().copied()),
        );
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
            envelope: Envelope::new(params),
            done: false,
        })
    }

    /// Note-off: the loop stops and the envelope releases.
    pub fn release(&mut self) {
        self.held = false;
        self.envelope.release();
    }

    pub fn finished(&self) -> bool {
        self.done
    }

    fn looping(&self) -> bool {
        self.held && self.loop_range.is_some()
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

    /// The voice's output for one frame of `dt` seconds.
    pub fn next(&mut self, dt: f64) -> f64 {
        if self.done {
            return 0.0;
        }
        let level = self.envelope.tick(dt);
        let out = self.read() * self.gain * level;
        self.position += self.step;
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
