//! A music segment's voice in the mixer: its synthesizer, which takes the notes the music queue
//! sends for the segment ([`crate::sink::SinkCommand::Note`]), with the segment's band bound to
//! its channels. Gained by the music channel's attenuation and placed like a 3D wave. Ended, it
//! takes no more notes and goes once its notes have released.

use std::sync::Arc;

use crate::mixer::{spatial_gains, Level};
use crate::music::dls::Collection;
use crate::music::performance::{EventSink, TimedEvent};
use crate::sink::Spatial;
use crate::synth::{ChannelSources, Source, Synth};

pub(crate) struct MusicVoice {
    pub(crate) id: u32,
    synth: Synth,
    pub(crate) level: Level,
    /// The left and right gains of a 3D segment, which is heard as mono.
    spatial: Option<[f32; 2]>,
    ended: bool,
}

/// Mono to a frame: stereo as is, a mono output as the average of the two gains.
fn place(frame: &mut [f32], left: f64, right: f64) {
    if frame.len() == 1 {
        frame[0] += ((left + right) * 0.5) as f32;
    } else {
        frame[0] += left as f32;
        frame[1] += right as f32;
    }
}

impl MusicVoice {
    pub(crate) fn new(id: u32, rate: u32, gm: Option<Arc<Collection>>) -> Self {
        let mut synth = Synth::new(rate);
        synth.set_gm(gm);
        Self {
            id,
            synth,
            level: Level::new(0.0),
            spatial: None,
            ended: false,
        }
    }

    /// The segment starts (or starts again): its style's channels bind to `sources` and its band
    /// to `band`, at `amplitude`.
    pub(crate) fn start(
        &mut self,
        sources: &ChannelSources,
        band: &ChannelSources,
        amplitude: f32,
        spatial: Option<&Spatial>,
    ) {
        self.synth.set_sources(sources);
        self.synth.set_band_sources(band);
        self.level = Level::new(amplitude);
        self.spatial = spatial.map(spatial_gains);
        self.ended = false;
    }

    /// The band changed: the channels bind to `band`.
    pub(crate) fn set_band_sources(&mut self, band: &ChannelSources) {
        self.synth.set_band_sources(band);
    }

    pub(crate) fn set_gm(&mut self, gm: Option<Arc<Collection>>) {
        self.synth.set_gm(gm);
    }

    pub(crate) fn set_spatial(&mut self, spatial: &Spatial) {
        self.spatial = Some(spatial_gains(spatial));
    }

    /// A note of the segment (dropped once it has ended).
    pub(crate) fn note(&mut self, event: &TimedEvent) {
        if !self.ended {
            self.synth.event(event);
        }
    }

    /// The segment ends: its notes release, and no more come in.
    pub(crate) fn end(&mut self) {
        if !self.ended {
            self.ended = true;
            self.synth.stop();
        }
    }

    /// Ended with every note released: the mixer drops it.
    pub(crate) fn finished(&self) -> bool {
        self.ended && self.synth.voices() == 0
    }

    /// The notes sounding.
    pub(crate) fn sounding(&self) -> usize {
        self.synth.voices()
    }

    pub(crate) fn bindings(&self) -> Vec<(u32, Source)> {
        self.synth.bindings()
    }

    /// Adds the segment's frames, from the queue's time `clock`, to `out` (interleaved,
    /// `channels` per frame). `scratch` holds the synthesizer's output between calls.
    pub(crate) fn render_into(
        &mut self,
        out: &mut [f32],
        channels: usize,
        clock: f64,
        scratch: &mut Vec<f32>,
    ) {
        scratch.resize(out.len(), 0.0);
        self.synth.render(scratch, channels, clock);
        for (frame, source) in out
            .chunks_exact_mut(channels)
            .zip(scratch.chunks_exact(channels))
        {
            // The synthesizer's left and right (a mono output is already the average).
            let level = f64::from(self.level.next());
            let left = f64::from(source[0]) * level;
            let right = f64::from(source[channels.min(2) - 1]) * level;
            match self.spatial {
                Some([gl, gr]) => place(frame, left * f64::from(gl), right * f64::from(gr)),
                None => place(frame, left, right),
            }
        }
    }
}
