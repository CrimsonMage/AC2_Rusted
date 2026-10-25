//! One playing sound (`DirectXPlayingSound`): how it was asked to play
//! (`DirectXSoundInstance::InitDXPSDFromSoundInfo`, `DirectXPlayingSound::DXInitialize`), its
//! lifetime, and the voice calls it makes on the sink.

use std::sync::Arc;

use crate::data::{flags, MusicInfo, Pcm, SoundInfo};
use crate::music_thread::SegmentKind;
use crate::play_data::{Channel, LoopingMode, PlaySoundData, SoundType, Track};
use crate::sink::{millibels, Boundary, SinkCommand, SoundSink, Spatial};
use crate::synth::ChannelSources;

/// `DirectXPlayingSound::LifetimeState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifetime {
    KeepAlive,
    Playing,
    WaitingForEnd,
    Fading,
    ReadyToDie,
}

/// What a playing sound reads from its device each call: whether it is enabled, the channel
/// volumes (`DirectXSoundDevice::m_fSoundVolume` and the rest) and `Timer::local_time`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeviceState {
    pub enabled: bool,
    pub volumes: [f32; 4],
    pub now: f64,
}

/// An emitter as `DirectXPlayingSound::UpdatePosition` sees it (`ISoundObj`): its offset from
/// the listener and its velocity, both already in the listener's frame
/// (`Position::globaltolocalvec` of `PositionOffset::GetOffset`), and whether it is muted
/// (`CPhysicsObj::ShouldMuteSound`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Emitter {
    /// Listener-local, AC2 axes (x right, y ahead, z up), metres.
    pub offset: [f32; 3],
    pub velocity: [f32; 3],
    pub muted: bool,
}

impl DeviceState {
    fn channel_volume(&self, channel: Channel) -> f32 {
        self.volumes[channel as usize]
    }
}

/// What a voice plays: a wave's samples, or a `MusicInfo`'s DirectMusic segment with its
/// start and stop boundaries, whether it is the primary segment and whether it controls.
#[derive(Debug, Clone)]
pub(crate) enum VoiceSource {
    Wave(Arc<Pcm>),
    Music {
        info: MusicInfo,
        start: Boundary,
        stop: Boundary,
        primary: bool,
        control: bool,
        /// The segment's style's bands, bound to the voice's synthesizer.
        sources: ChannelSources,
        /// The band its notes play on (its own band, or the performance's): searched first.
        band: ChannelSources,
    },
}

/// `DirectXSoundInstance::InitDXPSDFromMusicInfo`'s start boundary from the `MusicInfo` bits.
fn start_boundary(info: &MusicInfo) -> Boundary {
    if info.test(0x20) {
        Boundary::End
    } else if info.test(0x80) {
        Boundary::Grid
    } else if info.test(0x40) {
        Boundary::Beat
    } else if info.test(0x200) {
        Boundary::Marker
    } else if info.test(0x100) {
        Boundary::Immediate
    } else {
        Boundary::Measure
    }
}

/// The stop boundary (`m_stopTiming`).
fn stop_boundary(info: &MusicInfo) -> Boundary {
    if info.test(0x800) {
        Boundary::End
    } else if info.test(0x2000) {
        Boundary::Grid
    } else if info.test(0x1000) {
        Boundary::Beat
    } else if info.test(0x8000) {
        Boundary::Marker
    } else if info.test(0x4000) {
        Boundary::Measure
    } else {
        Boundary::Immediate
    }
}

/// `DirectXPlayingSound`.
#[derive(Debug, Clone)]
pub(crate) struct PlayingSound {
    /// `m_soundHandle`, also the sink's voice.
    pub handle: u32,
    /// The instance's `GetDataID`.
    pub sound: u32,
    pub emitter: Option<u64>,
    pub kind: SoundType,
    pub channel: Channel,
    pub looping: bool,
    source: VoiceSource,
    /// `m_fCurrentVolume`.
    pub volume: f32,
    /// `m_fFrequencyModulate`.
    pub frequency: f32,
    /// `m_fBaseFrequency`: the buffer's rate once it has one.
    base_frequency: f32,
    base_priority: f32,
    /// `m_fCurrentPriority`: lower plays first.
    pub priority: f32,
    /// `m_fSoundLength`: the wave's time plus half a second.
    length: f32,
    start_time: f64,
    fade_time: f64,
    fade_length: f32,
    ready_after_fade: bool,
    pub lifetime: Lifetime,
    has_played: bool,
    /// The sink holds the voice (retail's `m_pAudioPath`).
    has_voice: bool,
    disabled: bool,
    muted: bool,
    in_world: bool,
    /// The `SoundInfo`'s `3D` flag: `Update3DInfo` places the buffer (else `DS3DMODE_DISABLE`).
    three_d: bool,
    /// `m_fMinDistance`.
    min_distance: f32,
    /// `m_vLocalOffset` / `m_vLocalVelocity`, DirectSound axes.
    offset: [f32; 3],
    velocity: [f32; 3],
}

impl PlayingSound {
    /// `DirectXSoundInstance::CreatePlayingSound` for a `SoundInfo`
    /// (`InitDXPSDFromSoundInfo`, then `DXInitialize`).
    pub fn new(
        handle: u32,
        data: &PlaySoundData,
        info: &SoundInfo,
        pcm: Arc<Pcm>,
        wave_length: f32,
        now: f64,
    ) -> Self {
        let looping = match data.looping {
            LoopingMode::Inherit => info.test(flags::LOOP),
            LoopingMode::ForceOn => true,
            LoopingMode::ForceOff => false,
        };
        let kind = match data.kind {
            SoundType::Inherit if info.test(flags::EXCLUSIVE) => {
                SoundType::ForceSelfExclusiveBudgeted
            }
            SoundType::Inherit => SoundType::ForceNonExclusiveBudgeted,
            kind => kind,
        };
        Self {
            handle,
            sound: info.did,
            emitter: data.emitter,
            kind,
            channel: data.channel,
            looping,
            source: VoiceSource::Wave(pcm),
            volume: data.volume,
            frequency: data.frequency,
            base_frequency: -1.0,
            base_priority: info.priority,
            priority: 1.0,
            length: (f64::from(wave_length) + 0.5) as f32,
            start_time: now,
            fade_time: 0.0,
            fade_length: 0.0,
            ready_after_fade: false,
            lifetime: Lifetime::KeepAlive,
            has_played: false,
            has_voice: false,
            disabled: false,
            muted: false,
            in_world: true,
            three_d: info.test(flags::THREE_D),
            min_distance: info.min_distance,
            offset: [0.0; 3],
            velocity: [0.0; 3],
        }
    }

    /// `DirectXSoundInstance::CreatePlayingSound` for a `MusicInfo`
    /// (`InitDXPSDFromMusicInfo`, then `DXInitialize`): no length (it plays until stopped),
    /// placed in 3D when its `0x10` bit asks (`AudioPath_3DMusic`).
    pub fn from_music(
        handle: u32,
        data: &PlaySoundData,
        info: &MusicInfo,
        sources: ChannelSources,
        band: ChannelSources,
        now: f64,
    ) -> Self {
        let exclusive = info.test(flags::EXCLUSIVE);
        let looping = match data.looping {
            LoopingMode::Inherit => !exclusive && info.test(flags::LOOP),
            LoopingMode::ForceOn => true,
            LoopingMode::ForceOff => false,
        };
        let kind = match data.kind {
            SoundType::Inherit if exclusive => SoundType::ForceNonExclusive,
            SoundType::Inherit if info.test(flags::THREE_D) => SoundType::ForceSelfExclusive,
            SoundType::Inherit => SoundType::ForceNonExclusive,
            kind => kind,
        };
        let track = match data.track {
            Track::Inherit if exclusive => Track::ForcePrimary,
            Track::Inherit => Track::ForceSecondary,
            track => track,
        };
        let mut sound = Self::new(
            handle,
            data,
            &SoundInfo {
                did: info.did,
                bit_field: 0,
                rand_freq_top: -1.0,
                rand_freq_bot: -1.0,
                priority: info.priority,
                min_distance: info.min_distance,
                wave: info.segment,
            },
            Arc::new(Pcm {
                rate: 0,
                channels: 1,
                samples: Arc::from([]),
            }),
            -1.0,
            now,
        );
        sound.kind = kind;
        sound.looping = looping;
        sound.three_d = info.test(flags::NO_VIRTUAL);
        sound.source = VoiceSource::Music {
            info: *info,
            start: start_boundary(info),
            stop: stop_boundary(info),
            primary: track == Track::ForcePrimary,
            control: track == Track::ForceSecondary && info.test(0x400),
            sources,
            band,
        };
        sound
    }

    /// `Update3DInfo`'s buffer parameters.
    fn spatial(&self) -> Option<Spatial> {
        self.three_d.then_some(Spatial {
            position: self.offset,
            velocity: self.velocity,
            min_distance: self.min_distance,
        })
    }

    /// `GetIsSelfExclusive`.
    pub fn is_self_exclusive(&self) -> bool {
        matches!(
            self.kind,
            SoundType::ForceSelfExclusive | SoundType::ForceSelfExclusiveBudgeted
        )
    }

    /// `GetIsAllExclusive`.
    pub fn is_all_exclusive(&self) -> bool {
        self.kind == SoundType::ForceAllExclusive
    }

    /// `GetIsBudgeted`.
    pub fn is_budgeted(&self) -> bool {
        self.kind == SoundType::ForceNonExclusiveBudgeted
            || (self.kind == SoundType::ForceSelfExclusiveBudgeted && !self.muted)
    }

    /// `GetIsPlaying`.
    pub fn is_playing(&self) -> bool {
        self.has_played
            && matches!(
                self.lifetime,
                Lifetime::Playing | Lifetime::Fading | Lifetime::KeepAlive
            )
    }

    /// `GetInWorld`.
    pub fn in_world(&self) -> bool {
        self.in_world
    }

    /// `GetReadyToDie`.
    pub fn ready_to_die(&self) -> bool {
        self.lifetime == Lifetime::ReadyToDie
    }

    /// `SetVolumeInternal`: clamps, skips a change under `0.0002` unless `force`, and sets the
    /// voice's attenuation (over `ramp` seconds) when it has one.
    fn set_volume_internal(
        &mut self,
        sink: &mut dyn SoundSink,
        device: &DeviceState,
        volume: f32,
        ramp: f32,
        force: bool,
    ) {
        let volume = volume.clamp(0.0, 1.0);
        if !self.has_voice {
            self.volume = volume;
            return;
        }
        if !force && (f64::from(volume) - f64::from(self.volume)).abs() < 0.0002 {
            return;
        }
        let millibels = millibels(device.channel_volume(self.channel), volume);
        let ramp_ms = (f64::from(ramp) * 1000.0).trunc() as u32;
        sink.send(SinkCommand::Volume {
            voice: self.handle,
            millibels,
            ramp_ms,
        });
        self.volume = volume;
    }

    /// `RecalculateVolume`: the channel's volume changed.
    pub fn recalculate_volume(&mut self, sink: &mut dyn SoundSink, device: &DeviceState) {
        self.set_volume_internal(sink, device, self.volume, 0.0, true);
    }

    /// `SetVolume`.
    pub fn set_volume(&mut self, sink: &mut dyn SoundSink, device: &DeviceState, volume: f32) {
        self.set_volume_internal(sink, device, volume, 0.0, false);
    }

    fn rate(&self) -> u32 {
        (f64::from(self.frequency) * f64::from(self.base_frequency))
            .trunc()
            .max(0.0) as u32
    }

    /// `PlayInternal`: (re)starts the voice when the sound is kept alive and neither disabled
    /// nor muted.
    pub fn play(&mut self, sink: &mut dyn SoundSink, device: &DeviceState) -> bool {
        if !self.in_world || self.lifetime != Lifetime::KeepAlive {
            return true;
        }
        if !self.has_played {
            self.start_time = device.now;
            self.has_played = true;
        }
        if !device.enabled {
            if self.length < 0.0 {
                self.lifetime = Lifetime::ReadyToDie;
            }
            return true;
        }
        if self.muted || self.disabled {
            return true;
        }
        let volume = self.volume.clamp(0.0, 1.0);
        self.volume = volume;
        let millibels = millibels(device.channel_volume(self.channel), volume);
        match &self.source {
            VoiceSource::Wave(pcm) => {
                // `GetAudioPathAndBuffers` on the first play: the buffer's own rate is the base.
                if !self.has_voice {
                    self.base_frequency = pcm.rate as f32;
                }
                sink.send(SinkCommand::Start {
                    voice: self.handle,
                    pcm: Arc::clone(pcm),
                    looping: self.looping,
                    millibels,
                    frequency: if self.frequency < 0.0 { 0 } else { self.rate() },
                    spatial: self.spatial(),
                });
            }
            VoiceSource::Music {
                info,
                start,
                primary,
                control,
                sources,
                band,
                ..
            } => sink.send(SinkCommand::StartMusic {
                voice: self.handle,
                music: info.did,
                segment: info.segment,
                looping: self.looping,
                millibels,
                boundary: *start,
                primary: *primary,
                control: *control,
                spatial: self.spatial(),
                sources: Arc::clone(sources),
                band: Arc::clone(band),
            }),
        }
        self.has_voice = true;
        self.lifetime = Lifetime::Playing;
        true
    }

    /// `DXMusicMessageThread::Stop`, then (`OnSegmentEnded`, as the stop is immediate) the
    /// voice is released: a looping sound waiting for its end, or a one-shot no longer kept
    /// alive, is ready to die.
    fn stop_voice(&mut self, sink: &mut dyn SoundSink) {
        self.stop_at(sink, Boundary::Immediate);
    }

    /// `DXMusicMessageThread::Stop` at `boundary` (a music segment's stop timing), then
    /// `OnSegmentEnded`.
    fn stop_at(&mut self, sink: &mut dyn SoundSink, boundary: Boundary) {
        self.send_stop(sink, boundary);
        // A music segment stopped at a boundary plays out: its end comes from the queue (its
        // `SegmentEnd`), so the voice stays until then.
        if boundary == Boundary::Immediate || !matches!(self.source, VoiceSource::Music { .. }) {
            self.on_segment_ended();
        }
    }

    /// The boundary the music queue stops this voice's segment at: its `MusicInfo`'s stop timing,
    /// or immediately for an immediate stop (and for a wave, which has no queue segment).
    pub fn stop_timing(&self, immediate: bool) -> Boundary {
        match (&self.source, immediate) {
            (VoiceSource::Music { stop, .. }, false) => *stop,
            _ => Boundary::Immediate,
        }
    }

    fn send_stop(&self, sink: &mut dyn SoundSink, boundary: Boundary) {
        match self.source {
            VoiceSource::Wave(_) => sink.send(SinkCommand::Stop { voice: self.handle }),
            VoiceSource::Music { .. } => sink.send(SinkCommand::StopMusic {
                voice: self.handle,
                boundary,
            }),
        }
    }

    /// Whether this is a music voice, and its segment's kind: a primary segment when the
    /// `MusicInfo` track says so (`Track_ForcePrimary`). `None` for a wave.
    pub fn segment_kind(&self) -> Option<SegmentKind> {
        match &self.source {
            VoiceSource::Music { primary: true, .. } => Some(SegmentKind::Primary),
            VoiceSource::Music { .. } => Some(SegmentKind::Secondary),
            VoiceSource::Wave(_) => None,
        }
    }

    /// `OnSegmentEnded`, as the music queue calls it back for this voice's segment.
    pub(crate) fn on_segment_ended(&mut self) {
        self.has_voice = false;
        if (self.looping && self.lifetime == Lifetime::WaitingForEnd)
            || (!self.looping && self.lifetime != Lifetime::KeepAlive)
        {
            self.lifetime = Lifetime::ReadyToDie;
        }
    }

    fn release(&mut self, sink: &mut dyn SoundSink) {
        if self.has_voice {
            self.send_stop(sink, Boundary::Immediate);
            self.has_voice = false;
        }
    }

    /// `Stop` (`StopInternal` with `ready_to_die`): fades out over `fade` seconds, or stops now.
    pub fn stop(
        &mut self,
        sink: &mut dyn SoundSink,
        device: &DeviceState,
        fade: f32,
        immediate: bool,
    ) {
        if !immediate && fade > 0.0 {
            self.fade_time = device.now;
            self.fade_length = fade;
            self.lifetime = Lifetime::Fading;
            self.ready_after_fade = true;
            let volume = self.volume;
            self.set_volume_internal(sink, device, 0.0, fade, true);
            // The ramp's target is silence; the sound's own volume is kept for a restart.
            self.volume = volume;
            return;
        }
        if !matches!(self.lifetime, Lifetime::Playing | Lifetime::Fading) {
            self.release(sink);
            self.lifetime = Lifetime::ReadyToDie;
            return;
        }
        self.lifetime = if immediate {
            self.emitter = None;
            Lifetime::ReadyToDie
        } else {
            Lifetime::WaitingForEnd
        };
        let boundary = self.stop_timing(immediate);
        self.stop_at(sink, boundary);
    }

    /// `Enable`: a disabled sound stops but is kept alive to play again.
    pub fn enable(&mut self, sink: &mut dyn SoundSink, device: &DeviceState, enabled: bool) {
        self.disabled = !enabled;
        if self.muted || !self.has_played {
            return;
        }
        if enabled {
            if self.lifetime == Lifetime::KeepAlive {
                self.play(sink, device);
            }
            return;
        }
        match self.lifetime {
            Lifetime::Playing => self.lifetime = Lifetime::KeepAlive,
            Lifetime::Fading if self.ready_after_fade => {
                self.lifetime = Lifetime::ReadyToDie;
                self.emitter = None;
            }
            Lifetime::Fading => self.lifetime = Lifetime::KeepAlive,
            _ => return,
        }
        self.stop_voice(sink);
    }

    /// `OnDeviceStateChanged`: a disabled device stops the voice (a playing sound is kept alive
    /// for when it comes back); an enabled one plays what is kept alive.
    pub fn on_device_state_changed(&mut self, sink: &mut dyn SoundSink, device: &DeviceState) {
        if device.enabled {
            self.play(sink, device);
            return;
        }
        if matches!(self.lifetime, Lifetime::Playing | Lifetime::Fading) {
            self.lifetime = Lifetime::KeepAlive;
            self.stop_voice(sink);
        } else {
            self.release(sink);
            self.lifetime = Lifetime::ReadyToDie;
        }
    }

    /// `DirectXPlayingSound::UseTime`: a muted emitter's sound stops (kept alive unless it was
    /// fading out); otherwise a sound kept alive plays again; a one-shot past its length, or a
    /// finished fade, ends.
    pub fn use_time(&mut self, sink: &mut dyn SoundSink, device: &DeviceState, muted: bool) {
        if self.emitter.is_some() && muted {
            self.muted = true;
            match self.lifetime {
                Lifetime::Playing => {
                    self.lifetime = Lifetime::KeepAlive;
                    self.stop_voice(sink);
                }
                Lifetime::Fading => {
                    if self.ready_after_fade {
                        self.lifetime = Lifetime::ReadyToDie;
                        self.emitter = None;
                    } else {
                        self.lifetime = Lifetime::KeepAlive;
                    }
                    self.stop_voice(sink);
                }
                _ => {}
            }
        } else {
            self.muted = false;
            if self.lifetime == Lifetime::KeepAlive && self.has_played {
                self.play(sink, device);
            }
        }
        // Retail compares the times as floats.
        let ends = f64::from(self.length) + f64::from(self.start_time as f32);
        if self.lifetime != Lifetime::ReadyToDie
            && !self.looping
            && self.length > 0.0
            && ends < f64::from(device.now as f32)
        {
            self.end_now(sink, Lifetime::ReadyToDie);
        }
        if self.fade_length > 0.0 && f64::from(self.fade_length) < device.now - self.fade_time {
            let after = if self.ready_after_fade {
                Lifetime::ReadyToDie
            } else {
                Lifetime::KeepAlive
            };
            self.end_now(sink, after);
        }
    }

    fn end_now(&mut self, sink: &mut dyn SoundSink, after: Lifetime) {
        if matches!(self.lifetime, Lifetime::Playing | Lifetime::Fading) {
            self.lifetime = after;
            if after == Lifetime::ReadyToDie {
                self.emitter = None;
            }
            self.stop_voice(sink);
        } else {
            self.release(sink);
            self.lifetime = Lifetime::ReadyToDie;
        }
    }

    /// `UpdatePosition`: an emitter's offset and velocity become the buffer's (y and z swapped
    /// into DirectSound's axes, `Update3DInfo`), and the priority the manager sorts by is
    /// `(2 - base) * distance` (distance 1 without an emitter), `-1` for an all-exclusive sound.
    pub fn update_position(&mut self, sink: &mut dyn SoundSink, emitter: Option<&Emitter>) {
        if matches!(
            self.lifetime,
            Lifetime::WaitingForEnd | Lifetime::ReadyToDie
        ) {
            return;
        }
        let mut distance = 1.0f64;
        if let (Some(_), Some(emitter)) = (self.emitter, emitter) {
            let [x, y, z] = emitter.offset;
            let [vx, vy, vz] = emitter.velocity;
            self.in_world = true;
            distance = (f64::from(z) * f64::from(z)
                + f64::from(y) * f64::from(y)
                + f64::from(x) * f64::from(x))
            .sqrt() as f32 as f64;
            self.offset = [x, z, y];
            self.velocity = [vx, vz, vy];
            if let (true, Some(spatial)) = (self.has_voice, self.spatial()) {
                sink.send(SinkCommand::Spatial {
                    voice: self.handle,
                    spatial,
                });
            }
        }
        self.priority = if self.is_all_exclusive() {
            -1.0
        } else {
            ((2.0 - f64::from(self.base_priority)) * distance) as f32
        };
    }
}
