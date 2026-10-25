//! The software mixer behind the speaker output: what DirectSound did with the buffers retail
//! filled. Each voice plays its PCM at its own rate (`SetFrequency`, linear interpolation to
//! the output rate), attenuated (`SetVolume`, ramped over the audio path's duration), once or
//! looping, placed in 3D when the sound is (`IDirectSound3DBuffer`: inverse-distance rolloff past
//! the minimum distance, a constant-power pan by the direction; DirectSound's own software pan
//! curve and doppler are not reproduced), summed into the output frames.

use std::sync::Arc;

use crate::data::Pcm;
use crate::sink::{amplitude, SinkCommand, Spatial};

struct Voice {
    id: u32,
    pcm: Arc<Pcm>,
    looping: bool,
    /// The read position in source frames.
    position: f64,
    /// Source frames per output frame.
    step: f64,
    gain: f32,
    target: f32,
    /// The gain's change per output frame while a ramp runs, and the frames left.
    ramp: (f32, u32),
    /// The left and right gains of a 3D voice, which is heard as mono.
    spatial: Option<[f32; 2]>,
}

/// A head-relative 3D buffer's left and right gains (rolloff factor 1, maximum distance far
/// beyond any world distance).
pub fn spatial_gains(spatial: &Spatial) -> [f32; 2] {
    let [x, y, z] = spatial.position.map(f64::from);
    let distance = (x * x + y * y + z * z).sqrt();
    let min = f64::from(spatial.min_distance).max(1e-3);
    let rolloff = if distance <= min { 1.0 } else { min / distance };
    let pan = if distance > 1e-6 {
        (x / distance).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let angle = (pan + 1.0) * std::f64::consts::FRAC_PI_4;
    let side = rolloff * std::f64::consts::SQRT_2;
    [(side * angle.cos()) as f32, (side * angle.sin()) as f32]
}

/// Mixes the voices into interleaved output frames.
pub struct Mixer {
    rate: u32,
    voices: Vec<Voice>,
}

impl Mixer {
    /// A mixer writing `rate` frames per second.
    pub fn new(rate: u32) -> Self {
        Self {
            rate: rate.max(1),
            voices: Vec::new(),
        }
    }

    /// Voices still sounding.
    pub fn voices(&self) -> usize {
        self.voices.len()
    }

    fn step(&self, frequency: u32) -> f64 {
        f64::from(frequency) / f64::from(self.rate)
    }

    pub fn apply(&mut self, command: SinkCommand) {
        match command {
            SinkCommand::Start {
                voice,
                pcm,
                looping,
                millibels,
                frequency,
                spatial,
            } => {
                self.voices.retain(|v| v.id != voice);
                let gain = amplitude(millibels);
                let step = self.step(frequency);
                self.voices.push(Voice {
                    id: voice,
                    pcm,
                    looping,
                    position: 0.0,
                    step,
                    gain,
                    target: gain,
                    ramp: (0.0, 0),
                    spatial: spatial.as_ref().map(spatial_gains),
                });
            }
            SinkCommand::Volume {
                voice,
                millibels,
                ramp_ms,
            } => {
                let frames = (u64::from(ramp_ms) * u64::from(self.rate) / 1000) as u32;
                if let Some(v) = self.voices.iter_mut().find(|v| v.id == voice) {
                    v.target = amplitude(millibels);
                    if frames == 0 {
                        v.gain = v.target;
                        v.ramp = (0.0, 0);
                    } else {
                        v.ramp = ((v.target - v.gain) / frames as f32, frames);
                    }
                }
            }
            SinkCommand::Frequency { voice, frequency } => {
                let step = self.step(frequency);
                if let Some(v) = self.voices.iter_mut().find(|v| v.id == voice) {
                    v.step = step;
                }
            }
            SinkCommand::Spatial { voice, spatial } => {
                if let Some(v) = self.voices.iter_mut().find(|v| v.id == voice) {
                    v.spatial = Some(spatial_gains(&spatial));
                }
            }
            SinkCommand::Stop { voice } => self.voices.retain(|v| v.id != voice),
        }
    }

    /// Fills `out` (interleaved, `channels` per frame) with the voices' sum.
    pub fn render(&mut self, out: &mut [f32], channels: usize) {
        out.fill(0.0);
        let channels = channels.max(1);
        for voice in &mut self.voices {
            let source_channels = usize::from(voice.pcm.channels.max(1));
            let frames = voice.pcm.frames();
            if frames == 0 {
                voice.looping = false;
                voice.position = 0.0;
                continue;
            }
            let samples = &voice.pcm.samples;
            let sample = |frame: usize, channel: usize| -> f64 {
                f64::from(samples[frame * source_channels + channel.min(source_channels - 1)])
            };
            for frame in out.chunks_exact_mut(channels) {
                if voice.position >= frames as f64 {
                    if !voice.looping {
                        break;
                    }
                    voice.position %= frames as f64;
                }
                let at = voice.position.floor();
                let index = at as usize;
                let next = if index + 1 < frames {
                    index + 1
                } else if voice.looping {
                    0
                } else {
                    index
                };
                let t = voice.position - at;
                let gain = f64::from(voice.gain);
                let mut left = (sample(index, 0) * (1.0 - t) + sample(next, 0) * t) * gain;
                let mut right = (sample(index, 1) * (1.0 - t) + sample(next, 1) * t) * gain;
                if let Some([l, r]) = voice.spatial {
                    let mono = (left + right) * 0.5;
                    (left, right) = (mono * f64::from(l), mono * f64::from(r));
                }
                if channels == 1 {
                    frame[0] += ((left + right) * 0.5) as f32;
                } else {
                    frame[0] += left as f32;
                    frame[1] += right as f32;
                }
                voice.position += voice.step;
                if voice.ramp.1 > 0 {
                    voice.gain += voice.ramp.0;
                    voice.ramp.1 -= 1;
                    if voice.ramp.1 == 0 {
                        voice.gain = voice.target;
                    }
                }
            }
        }
        self.voices
            .retain(|v| v.looping || v.position < v.pcm.frames() as f64);
        for sample in out.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}
