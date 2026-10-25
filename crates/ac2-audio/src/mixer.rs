//! The software mixer behind the speaker output: what DirectSound did with the buffers retail
//! filled. Each voice plays its PCM at its own rate (`SetFrequency`, linear interpolation to
//! the output rate), attenuated (`SetVolume`, ramped over the audio path's duration), once or
//! looping, placed in 3D when the sound is (`IDirectSound3DBuffer`: inverse-distance rolloff past
//! the minimum distance, a constant-power pan by the direction; DirectSound's own software pan
//! curve and doppler are not reproduced), summed into the output frames.
//!
//! A music segment is a voice too ([`crate::music_voice`]): its notes come in as commands and its
//! synthesizer renders them, from the queue's clock ([`SinkCommand::MusicClock`]), into the same
//! sum. Everything here runs on the output's thread (the audio callback).

use std::sync::Arc;

use crate::data::Pcm;
use crate::music::dls::Collection;
use crate::music_voice::MusicVoice;
use crate::sink::{amplitude, Boundary, SinkCommand, Spatial};
use crate::synth::Source;

/// A voice's gain: the amplitude of its attenuation, ramped to a new target over frames
/// (`SetVolume`'s ramp; a ramp of 0 frames is at once).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Level {
    gain: f32,
    target: f32,
    /// The gain's change per output frame while a ramp runs, and the frames left.
    ramp: (f32, u32),
}

impl Level {
    /// A gain at `amplitude`, with no ramp.
    pub(crate) fn new(amplitude: f32) -> Self {
        Self {
            gain: amplitude,
            target: amplitude,
            ramp: (0.0, 0),
        }
    }

    /// Ramps to `target` over `frames` (at once when 0).
    pub(crate) fn set(&mut self, target: f32, frames: u32) {
        self.target = target;
        if frames == 0 {
            self.gain = target;
            self.ramp = (0.0, 0);
        } else {
            self.ramp = ((target - self.gain) / frames as f32, frames);
        }
    }

    /// The gain of the next frame; the ramp steps after it.
    pub(crate) fn next(&mut self) -> f32 {
        let gain = self.gain;
        if self.ramp.1 > 0 {
            self.gain += self.ramp.0;
            self.ramp.1 -= 1;
            if self.ramp.1 == 0 {
                self.gain = self.target;
            }
        }
        gain
    }
}

struct Voice {
    id: u32,
    pcm: Arc<Pcm>,
    looping: bool,
    /// The read position in source frames.
    position: f64,
    /// Source frames per output frame.
    step: f64,
    level: Level,
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
    /// The music segments' synthesizers, by sound handle.
    music: Vec<MusicVoice>,
    /// The queue's time at the first frame of the next render.
    music_clock: f64,
    /// The General MIDI set the synthesizers' GM channels use.
    gm: Option<Arc<Collection>>,
    /// The music voices' output, before it is summed.
    scratch: Vec<f32>,
}

impl Mixer {
    /// A mixer writing `rate` frames per second.
    pub fn new(rate: u32) -> Self {
        Self {
            rate: rate.max(1),
            voices: Vec::new(),
            music: Vec::new(),
            music_clock: 0.0,
            gm: None,
            scratch: Vec::new(),
        }
    }

    /// Wave voices still sounding.
    pub fn voices(&self) -> usize {
        self.voices.len()
    }

    /// Music voices (segments) the mixer holds: their notes are still sounding or they can
    /// still take notes.
    pub fn music_voices(&self) -> usize {
        self.music.len()
    }

    /// The notes sounding on the music voice `voice` (`None` without one).
    pub fn music_sounding(&self, voice: u32) -> Option<usize> {
        self.music
            .iter()
            .find(|m| m.id == voice)
            .map(|m| m.sounding())
    }

    /// The channel bindings of the music voice `voice`.
    pub fn music_bindings(&self, voice: u32) -> Option<Vec<(u32, Source)>> {
        self.music
            .iter()
            .find(|m| m.id == voice)
            .map(|m| m.bindings())
    }

    /// The music voice `voice` ends: no more notes, and it goes once its notes release.
    fn end_music(&mut self, voice: u32) {
        if let Some(music) = self.music.iter_mut().find(|m| m.id == voice) {
            music.end();
        }
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
                let step = self.step(frequency);
                self.voices.push(Voice {
                    id: voice,
                    pcm,
                    looping,
                    position: 0.0,
                    step,
                    level: Level::new(amplitude(millibels)),
                    spatial: spatial.as_ref().map(spatial_gains),
                });
            }
            SinkCommand::Volume {
                voice,
                millibels,
                ramp_ms,
            } => {
                let frames = (u64::from(ramp_ms) * u64::from(self.rate) / 1000) as u32;
                let target = amplitude(millibels);
                if let Some(v) = self.voices.iter_mut().find(|v| v.id == voice) {
                    v.level.set(target, frames);
                } else if let Some(m) = self.music.iter_mut().find(|m| m.id == voice) {
                    m.level.set(target, frames);
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
                } else if let Some(m) = self.music.iter_mut().find(|m| m.id == voice) {
                    m.set_spatial(&spatial);
                }
            }
            SinkCommand::Stop { voice } => self.voices.retain(|v| v.id != voice),
            SinkCommand::StartMusic {
                voice,
                millibels,
                spatial,
                sources,
                band,
                ..
            } => {
                let index = match self.music.iter().position(|m| m.id == voice) {
                    Some(index) => index,
                    None => {
                        self.music
                            .push(MusicVoice::new(voice, self.rate, self.gm.clone()));
                        self.music.len() - 1
                    }
                };
                self.music[index].start(&sources, &band, amplitude(millibels), spatial.as_ref());
            }
            // A stop at a boundary is the queue's to play out; only an immediate one ends the
            // voice here.
            SinkCommand::StopMusic { voice, boundary } => {
                if boundary == Boundary::Immediate {
                    self.end_music(voice);
                }
            }
            SinkCommand::MusicEnded { voice } => self.end_music(voice),
            SinkCommand::MusicBands { voice, sources } => {
                if let Some(music) = self.music.iter_mut().find(|m| m.id == voice) {
                    music.set_band_sources(&sources);
                }
            }
            SinkCommand::Note(event) => {
                if let Some(music) = self.music.iter_mut().find(|m| m.id == event.segment) {
                    music.note(&event);
                }
            }
            SinkCommand::MusicClock { seconds } => self.music_clock = seconds,
            SinkCommand::GeneralMidi { set } => {
                for music in &mut self.music {
                    music.set_gm(set.clone());
                }
                self.gm = set;
            }
            SinkCommand::Groove { .. } => {}
        }
    }

    /// Fills `out` (interleaved, `channels` per frame) with the voices' sum.
    pub fn render(&mut self, out: &mut [f32], channels: usize) {
        out.fill(0.0);
        let channels = channels.max(1);
        let frames_out = out.len() / channels;
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
                let gain = f64::from(voice.level.next());
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
            }
        }
        self.voices
            .retain(|v| v.looping || v.position < v.pcm.frames() as f64);
        for music in &mut self.music {
            music.render_into(out, channels, self.music_clock, &mut self.scratch);
        }
        self.music.retain(|m| !m.finished());
        self.music_clock += frames_out as f64 / f64::from(self.rate);
        for sample in out.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}
