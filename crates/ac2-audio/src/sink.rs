//! Where the sound manager's voices go: the DirectSound buffer calls retail's
//! `DirectXPlayingSound` makes, as commands. The speaker backend mixes them
//! ([`crate::output`]); tests use a [`NullSink`] or a [`RecordingSink`].

use std::sync::{Arc, Mutex};

use crate::data::Pcm;
use crate::music::dls::Collection;
use crate::music::performance::TimedEvent;
use crate::synth::ChannelSources;

/// The quietest attenuation, silence (`DSBVOLUME_MIN`, hundredths of a decibel).
pub const SILENT: i32 = -9600;

/// A 3D voice's `DS3DBUFFER` (`DirectXPlayingSound::Update3DInfo`): head-relative, in
/// DirectSound's axes (x right, y up, z ahead of the listener).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spatial {
    /// `vPosition`: the emitter relative to the listener, metres.
    pub position: [f32; 3],
    /// `vVelocity`.
    pub velocity: [f32; 3],
    /// `flMinDistance`: the sound's `m_minDistance`; it fades with distance beyond it.
    pub min_distance: f32,
}

/// `DirectXPlaySoundData::Timing`: the musical boundary a segment starts or stops on
/// (`DMUS_SEGF_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Boundary {
    /// At the end of the playing primary segment.
    End = 0,
    Grid = 1,
    Beat = 2,
    Marker = 3,
    Measure = 4,
    Immediate = 5,
}

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
        /// The 3D buffer's parameters, or `None` for a sound heard as is (`DS3DMODE_DISABLE`).
        spatial: Option<Spatial>,
    },
    /// `DXMusicMessageThread::Play` of a `MusicInfo`'s segment (`DBMusicObj`) on its audio path.
    /// The voice is the segment's synthesizer: its channels are bound to `sources`.
    StartMusic {
        voice: u32,
        music: u32,
        segment: u32,
        looping: bool,
        millibels: i32,
        /// When it starts (`DMUS_SEGF_*` of `m_startTiming`).
        boundary: Boundary,
        /// The primary segment (`Track_ForcePrimary`), else a secondary one.
        primary: bool,
        /// A secondary control segment (`DMUS_SEGF_CONTROL`).
        control: bool,
        spatial: Option<Spatial>,
        /// The segment's style's bands: each performance channel's patch source.
        sources: ChannelSources,
        /// The band its notes play on (its own band, or the performance's), searched first.
        band: ChannelSources,
    },
    /// The channel sources of the band the voice's segment plays on changed (a band change of
    /// the queue; see [`crate::music_thread::BandChange`]).
    MusicBands { voice: u32, sources: ChannelSources },
    /// `DXMusicMessageThread::Stop` of a segment at `boundary`. Only an immediate stop ends the
    /// voice here; a later boundary is the queue's to play out (its notes end as its events do).
    StopMusic { voice: u32, boundary: Boundary },
    /// The segment `voice` ended in the queue (its sound was flushed): its notes release and the
    /// voice goes once silent.
    MusicEnded { voice: u32 },
    /// A note event of the music queue, for the voice of its `segment`.
    Note(TimedEvent),
    /// The queue's time (`DXMusicMessageThread`'s clock) that the notes after it are timed
    /// against; the mixer advances it by the frames it renders.
    MusicClock { seconds: f64 },
    /// The General MIDI set (`None` makes its channels silent).
    GeneralMidi { set: Option<Arc<Collection>> },
    /// `IDirectMusicPerformance` master groove level (`DirectXSoundDevice::SetGrooveLevel`).
    Groove { level: i32 },
    /// `IDirectSound3DBuffer::SetAllParameters`: the emitter moved.
    Spatial { voice: u32, spatial: Spatial },
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
