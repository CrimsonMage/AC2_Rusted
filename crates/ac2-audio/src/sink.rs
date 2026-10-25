//! Where the sound manager's voices go: the DirectSound buffer calls retail's
//! `DirectXPlayingSound` makes, as commands. The speaker backend mixes them
//! ([`crate::output`]); tests use a [`NullSink`] or a [`RecordingSink`].

use std::sync::{Arc, Mutex};

use crate::data::Pcm;

/// The quietest attenuation, silence (`DSBVOLUME_MIN`, hundredths of a decibel).
pub const SILENT: i32 = -9600;

/// One call on a voice (a playing sound's buffer).
#[derive(Debug, Clone, PartialEq)]
pub enum SinkCommand {
    /// `DXMusicMessageThread::Play` of the sound's segment.
    Start {
        voice: u32,
        pcm: Arc<Pcm>,
        looping: bool,
        /// `IDirectSoundBuffer::SetVolume`'s attenuation, hundredths of a decibel.
        millibels: i32,
        /// `IDirectSoundBuffer::SetFrequency`, samples per second.
        frequency: u32,
    },
    /// `IDirectMusicAudioPath::SetVolume`: the attenuation, reached over `ramp_ms`.
    Volume {
        voice: u32,
        millibels: i32,
        ramp_ms: u32,
    },
    /// `IDirectSoundBuffer::SetFrequency`.
    Frequency { voice: u32, frequency: u32 },
    /// `DXMusicMessageThread::Stop`.
    Stop { voice: u32 },
}

/// An output for the voices.
pub trait SoundSink: Send {
    fn send(&mut self, command: SinkCommand);
}

/// Discards everything.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullSink;

impl SoundSink for NullSink {
    fn send(&mut self, _command: SinkCommand) {}
}

/// Keeps every command, for tests (headless).
#[derive(Debug, Default, Clone)]
pub struct RecordingSink {
    log: Arc<Mutex<Vec<SinkCommand>>>,
}

impl RecordingSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// The commands so far (a clone of the sink shares the log).
    pub fn commands(&self) -> Vec<SinkCommand> {
        self.log.lock().map(|log| log.clone()).unwrap_or_default()
    }
}

impl SoundSink for RecordingSink {
    fn send(&mut self, command: SinkCommand) {
        if let Ok(mut log) = self.log.lock() {
            log.push(command);
        }
    }
}

/// `DirectXPlayingSound::SetVolumeInternal`'s attenuation for a sound at `volume` (clamped to
/// `0..1`) on a channel at `channel_volume`: `-2500 + 2500 * channel * volume` hundredths of a
/// decibel (`_ftol2` truncates), or silence when either is zero.
pub fn millibels(channel_volume: f32, volume: f32) -> i32 {
    let volume = volume.clamp(0.0, 1.0);
    if volume > 0.0 && channel_volume > 0.0 {
        let scaled = f64::from(channel_volume) * f64::from(volume) * -2500.0;
        -2500 - scaled.trunc() as i32
    } else {
        SILENT
    }
}

/// The amplitude DirectSound applies for an attenuation (`10^(mB / 2000)`).
pub fn amplitude(millibels: i32) -> f32 {
    if millibels <= SILENT {
        0.0
    } else {
        10f64.powf(f64::from(millibels) / 2000.0) as f32
    }
}
