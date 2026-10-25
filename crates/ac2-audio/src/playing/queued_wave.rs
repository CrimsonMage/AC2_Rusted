//! A wave the music queue plays: an `EncodedWave` `MusicInfo` is a segment with a wave track
//! (`DirectXMusicLoader::LoadFromObject`), so its voice starts and ends with the queue's notices
//! rather than when the sound plays.

use std::sync::Arc;

use super::{DeviceState, Lifetime, PlayingSound, VoiceSource};
use crate::sink::{millibels, Boundary, SinkCommand, SoundSink};

/// A `MusicInfo`'s wave played as a segment of the music queue (`DirectXMusicLoader::LoadFromObject`
/// on an `EncodedWave`): its track's kind and stop timing, and whether the queue has started its
/// pass (the wave's voice starts and ends with the queue's notices).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct QueuedWave {
    pub(super) primary: bool,
    pub(super) stop: Boundary,
    pub(super) started: bool,
}

impl PlayingSound {
    /// Whether this is a wave the music queue plays (its voice follows the queue's segment).
    pub(super) fn is_queued_wave(&self) -> bool {
        self.queued_wave.is_some()
    }

    /// The wave's voice on the sink: its PCM from the start (a restart replaces it), at the
    /// sound's volume on the music channel.
    pub(super) fn send_queued_wave(&mut self, sink: &mut dyn SoundSink, device: &DeviceState) {
        let VoiceSource::Wave(pcm) = &self.source else {
            return;
        };
        let pcm = Arc::clone(pcm);
        if !self.has_voice {
            self.base_frequency = pcm.rate as f32;
        }
        let millibels = millibels(device.channel_volume(self.channel), self.volume);
        sink.send(SinkCommand::Start {
            voice: self.handle,
            pcm,
            looping: self.looping,
            millibels,
            frequency: if self.frequency < 0.0 { 0 } else { self.rate() },
            spatial: self.spatial(),
        });
        self.has_voice = true;
        self.lifetime = Lifetime::Playing;
    }

    /// `DXMusicMessageThread::SegmentStart` for this wave: its pass starts, and its voice with it
    /// (a repeated pass restarts the voice).
    pub(crate) fn on_segment_started(&mut self, sink: &mut dyn SoundSink, device: &DeviceState) {
        let Some(queued) = self.queued_wave.as_mut() else {
            return;
        };
        queued.started = true;
        if self.in_world
            && device.enabled
            && !self.muted
            && !self.disabled
            && matches!(self.lifetime, Lifetime::Playing | Lifetime::KeepAlive)
        {
            self.send_queued_wave(sink, device);
        }
    }
}
