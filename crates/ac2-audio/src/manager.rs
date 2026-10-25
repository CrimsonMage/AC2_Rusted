//! `SoundManager`: the frame thread's side of sound. `Play` queues a sound (loading it on the
//! loader thread first when it is not resident), `UseTime` runs the playing sounds, orders them
//! by priority and keeps the budgeted ones within `m_nMaxSecondarySounds`, and the volume
//! settings follow the preferences. It never reads a file.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::Duration;

use crate::data::{flags, sound_db_type, LoadedSound, SoundSource};
use crate::loader::SoundLoader;
use crate::play_data::{Channel, PlaySoundData, SoundType};
use crate::playing::{DeviceState, Emitter, PlayingSound};
use crate::sink::SoundSink;

/// `ID_Sound_NumPlayingSounds`' default (`SoundManager::SoundManager`).
const MAX_SECONDARY_SOUNDS: u32 = 32;
/// Past this priority a sound is not given a voice (`AllocateSounds`).
const PRIORITY_LIMIT: f64 = 250.0;

/// The sound preferences (`SoundManager::InitPrefs`) and what changing each does
/// (`HandleChangeInternal`, `HandleApply`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundPreference {
    /// `ID_Sound_FXVolume` (`0x40000040`): the main channel, `0..1`.
    SoundVolume(f32),
    /// `ID_Sound_MusicVolume` (`0x40000041`).
    MusicVolume(f32),
    /// `ID_Sound_Player_Music` (`0x4000003D`).
    PlayerMusic(f32),
    /// `ID_Sound_AmbientVolume` (`0x40000056`).
    AmbientVolume(f32),
    /// `ID_Sound_NumPlayingSounds` (`0x40000058`, `8..64`).
    NumPlayingSounds(f32),
    /// `ID_Sound_DisableSound` (`0x40000033`).
    DisableSound(bool),
    /// `ID_Sound_DisableMusic` (`0x40000034`): stops the music playing.
    DisableMusic(bool),
}

/// `SoundManager::SoundInstanceInfo`: a resident sound and when it was last played.
struct Instance {
    sound: LoadedSound,
    last_used: f64,
}

/// `SoundManager::SoundRequest`: a play waiting for its sound to load.
struct SoundRequest {
    context: u32,
    data: PlaySoundData,
    handle: u32,
    cancel: bool,
}

/// `SoundManager` over its device (`ISoundDevice`, here the sink).
pub struct SoundManager {
    /// `m_pDevice`.
    sink: Box<dyn SoundSink>,
    loader: SoundLoader,
    /// `m_mapLoadedSounds` with `m_aLoadedSoundsLRU`'s order.
    loaded: HashMap<u32, Instance>,
    lru: Vec<u32>,
    /// `m_mapGetRequests`.
    requests: Vec<SoundRequest>,
    next_context: u32,
    /// `m_aPlayingSounds` (`m_mapAllPlayingSounds` is its handles).
    playing: Vec<PlayingSound>,
    /// `m_selfExclusiveSet`: sounds that play once at a time.
    self_exclusive: BTreeSet<u32>,
    disabled: bool,
    music_disabled: bool,
    non_looping: u32,
    max_secondary: u32,
    next_handle: u32,
    /// `m_fLastMusicVolume`.
    last_music_volume: f32,
    /// The device's channel volumes and state, and `Timer::local_time`.
    device: DeviceState,
}

impl SoundManager {
    /// `SoundManager::SoundManager` and `Initialize`: the device is `sink`, sounds load from
    /// `source` on a loader thread, and the preferences start at their defaults (every volume
    /// full, sound and music on).
    pub fn new(sink: Box<dyn SoundSink>, source: Arc<dyn SoundSource>) -> Self {
        Self {
            sink,
            loader: SoundLoader::new(source),
            loaded: HashMap::new(),
            lru: Vec::new(),
            requests: Vec::new(),
            next_context: 1,
            playing: Vec::new(),
            self_exclusive: BTreeSet::new(),
            disabled: false,
            music_disabled: false,
            non_looping: 0,
            max_secondary: MAX_SECONDARY_SOUNDS,
            next_handle: 1,
            last_music_volume: 1.0,
            device: DeviceState {
                enabled: true,
                volumes: [1.0; 4],
                now: 0.0,
            },
        }
    }

    /// `SoundManager::Play`: the sound's handle, `Some(0)` when a self-exclusive copy of it is
    /// already playing, `None` when it cannot play. A sound not yet resident is loaded first
    /// and plays from a later [`SoundManager::use_time`].
    pub fn play(&mut self, data: &PlaySoundData) -> Option<u32> {
        self.play_internal(data, 0, None)
    }

    /// `SoundManager::PlayInternal`.
    fn play_internal(
        &mut self,
        data: &PlaySoundData,
        final_handle: u32,
        fetched: Option<LoadedSound>,
    ) -> Option<u32> {
        if self.self_exclusive.contains(&data.sound) {
            return Some(0);
        }
        if data.sound == 0
            || (self.music_disabled && matches!(data.channel, Channel::Music | Channel::Player))
        {
            return None;
        }
        let handle = if final_handle == 0 {
            let handle = self.next_handle;
            self.next_handle = self.next_handle.wrapping_add(1).max(1);
            handle
        } else {
            final_handle
        };
        let sound = match self.loaded.get_mut(&data.sound) {
            Some(instance) => {
                instance.last_used = self.device.now;
                instance.sound.clone()
            }
            None => {
                sound_db_type(data.sound)?;
                let Some(sound) = fetched else {
                    // `DBCache::GetIfInMemory` missed: an asynchronous get.
                    let context = self.next_context;
                    self.next_context = self.next_context.wrapping_add(1).max(1);
                    self.requests.push(SoundRequest {
                        context,
                        data: *data,
                        handle,
                        cancel: false,
                    });
                    self.loader.request(context, data.sound);
                    return Some(handle);
                };
                if !self.check_budget(data.kind, &sound) {
                    return None;
                }
                // `ISoundDevice::CreateInstance` for a `MusicInfo` needs DirectMusic
                // segments, which are not played yet.
                if matches!(sound, LoadedSound::Music(_)) {
                    return None;
                }
                self.add_sound_instance(data.sound, sound.clone());
                return self.start_playing(data, handle, &sound);
            }
        };
        if !self.check_budget(data.kind, &sound) {
            return None;
        }
        self.start_playing(data, handle, &sound)
    }

    /// `ISoundInstance::CreatePlayingSound`, then the manager keeps it (and marks a
    /// self-exclusive sound playing).
    fn start_playing(
        &mut self,
        data: &PlaySoundData,
        handle: u32,
        sound: &LoadedSound,
    ) -> Option<u32> {
        let LoadedSound::Sound { info, pcm, length } = sound else {
            return None;
        };
        let playing = PlayingSound::new(
            handle,
            data,
            info,
            Arc::clone(pcm),
            *length,
            self.device.now,
        );
        if playing.is_self_exclusive() {
            self.self_exclusive.insert(playing.sound);
        }
        self.playing.push(playing);
        Some(handle)
    }

    /// `SoundManager::CheckSoundBudget`: a one-shot budgeted sound waits while the non-looping
    /// sounds fill the budget.
    fn check_budget(&self, kind: SoundType, sound: &LoadedSound) -> bool {
        let LoadedSound::Sound { info, .. } = sound else {
            return true;
        };
        let budgeted = matches!(
            kind,
            SoundType::ForceNonExclusiveBudgeted
                | SoundType::ForceSelfExclusiveBudgeted
                | SoundType::Inherit
        );
        info.test(flags::LOOP) || !budgeted || self.non_looping < self.max_secondary
    }

    /// `SoundManager::AddSoundInstance`: past four times the budget, the least recently played
    /// sound is dropped.
    fn add_sound_instance(&mut self, did: u32, sound: LoadedSound) {
        if self.lru.len() as u32 > self.max_secondary * 4 {
            let loaded = &self.loaded;
            let time = |did: &u32| loaded.get(did).map_or(f64::MIN, |i| i.last_used);
            self.lru.sort_by(|a, b| time(b).total_cmp(&time(a)));
            if let Some(oldest) = self.lru.pop() {
                self.loaded.remove(&oldest);
            }
        }
        self.loaded.insert(
            did,
            Instance {
                sound,
                last_used: self.device.now,
            },
        );
        self.lru.push(did);
    }

    /// `SoundManager::OnContextFinished`: a loaded sound plays unless its request was cancelled.
    fn on_context_finished(&mut self, context: u32, sound: Option<LoadedSound>) {
        let Some(at) = self.requests.iter().position(|r| r.context == context) else {
            return;
        };
        let request = self.requests.remove(at);
        if let Some(sound) = sound {
            if !request.cancel {
                self.play_internal(&request.data, request.handle, Some(sound));
            }
        }
    }

    /// The loads finished since the last frame (`DBCache::UseTime` running the callbacks).
    fn take_loads(&mut self) {
        while let Some((context, sound)) = self.loader.try_next() {
            self.on_context_finished(context, sound);
        }
    }

    /// Blocks until every pending load has finished (headless runs and tests); whether they
    /// all did within `timeout` each.
    pub fn wait_for_loads(&mut self, timeout: Duration) -> bool {
        while !self.requests.is_empty() {
            let Some((context, sound)) = self.loader.wait(timeout) else {
                return false;
            };
            self.on_context_finished(context, sound);
        }
        true
    }

    /// `SoundManager::UseTime` at `Timer::local_time` `now`: finished loads play, ended
    /// sounds go (`FlushSounds`), the playing sounds run and are ordered by priority, and an
    /// all-exclusive sound silences the rest; otherwise the budget is applied
    /// (`AllocateSounds`).
    pub fn use_time(&mut self, now: f64) {
        self.use_time_with(now, &|_| None);
    }

    /// [`SoundManager::use_time`] with the listener's view of the emitters
    /// (`DirectXSoundDevice::UpdateListener`): `emitters` gives each emitter's offset and
    /// velocity in the listener's frame and whether it is muted; one it does not know keeps
    /// its last place.
    pub fn use_time_with(&mut self, now: f64, emitters: &dyn Fn(u64) -> Option<Emitter>) {
        self.device.now = now;
        self.take_loads();
        self.flush_sounds(true);
        let device = self.device;
        for playing in &mut self.playing {
            let muted = playing
                .emitter
                .and_then(emitters)
                .is_some_and(|emitter| emitter.muted);
            playing.use_time(&mut *self.sink, &device, muted);
        }
        for playing in &mut self.playing {
            let emitter = playing.emitter.and_then(emitters);
            playing.update_position(&mut *self.sink, emitter.as_ref());
        }
        if self.playing.is_empty() {
            return;
        }
        // `_PlayingSoundSorter`: the lowest priority value first.
        self.playing
            .sort_by(|a, b| a.priority.total_cmp(&b.priority));
        if !self.playing[0].is_all_exclusive() {
            self.set_channel_volume(Channel::Music, self.last_music_volume);
            self.allocate_sounds(false);
            self.allocate_sounds(true);
            return;
        }
        for playing in &mut self.playing[1..] {
            playing.enable(&mut *self.sink, &device, false);
        }
        self.set_channel_volume(Channel::Music, 0.0);
        let first = &mut self.playing[0];
        if !first.is_playing() {
            first.play(&mut *self.sink, &device);
        }
        first.enable(&mut *self.sink, &device, true);
    }

    /// `SoundManager::AllocateSounds`: the sounds in the world, under the priority limit and
    /// within the budget play; the rest stop (one-shots) or wait (loops).
    fn allocate_sounds(&mut self, allocate: bool) {
        let device = self.device;
        let (mut budgeted, mut non_looping) = (0u32, 0u32);
        for playing in &mut self.playing {
            let over = !playing.in_world()
                || f64::from(playing.priority) >= PRIORITY_LIMIT
                || (playing.is_budgeted() && self.max_secondary <= budgeted);
            if over {
                if !allocate {
                    if playing.looping {
                        playing.enable(&mut *self.sink, &device, false);
                    } else {
                        playing.stop(&mut *self.sink, &device, 0.0, false);
                    }
                }
                continue;
            }
            if allocate {
                if !playing.is_playing() {
                    playing.play(&mut *self.sink, &device);
                }
                playing.enable(&mut *self.sink, &device, true);
            }
            if playing.is_budgeted() {
                budgeted += 1;
                if !playing.looping {
                    non_looping += 1;
                }
            }
        }
        self.non_looping = non_looping;
    }

    /// `SoundManager::FlushSounds`: drops the sounds ready to die (or all of them, stopped).
    fn flush_sounds(&mut self, expired_only: bool) -> bool {
        let device = self.device;
        let mut flushed = false;
        let mut index = 0;
        while index < self.playing.len() {
            if expired_only && !self.playing[index].ready_to_die() {
                index += 1;
                continue;
            }
            let mut playing = self.playing.swap_remove(index);
            if playing.is_self_exclusive() {
                self.self_exclusive.remove(&playing.sound);
            }
            if !expired_only {
                playing.stop(&mut *self.sink, &device, 0.0, true);
            }
            flushed = true;
        }
        flushed
    }

    /// `SoundManager::StopSounds`: stops the playing sounds `bits` names (1 music, 2 3D music,
    /// 4 sounds, 8 3D sounds); how many.
    pub fn stop_sounds(&mut self, bits: u32) -> u32 {
        let device = self.device;
        let mut stopped = 0;
        for playing in &mut self.playing {
            let Some(Instance { sound, .. }) = self.loaded.get(&playing.sound) else {
                continue;
            };
            let stop = match sound {
                LoadedSound::Sound { info, .. } if bits & 0xC != 0 => {
                    if info.test(flags::THREE_D) {
                        bits & 8 != 0
                    } else {
                        bits & 4 != 0
                    }
                }
                LoadedSound::Music(music) if bits & 3 != 0 => {
                    if music.test(0x10) {
                        bits & 2 != 0
                    } else {
                        bits & 1 != 0
                    }
                }
                _ => false,
            };
            if stop {
                playing.stop(&mut *self.sink, &device, 0.0, false);
                stopped += 1;
            }
        }
        stopped
    }

    /// `SoundManager::StopSoundsForEmitter`: stops `emitter`'s sounds at once and cancels its
    /// loading ones; how many.
    pub fn stop_sounds_for_emitter(&mut self, emitter: u64) -> u32 {
        let device = self.device;
        let mut stopped = 0;
        for playing in &mut self.playing {
            if playing.emitter == Some(emitter) {
                playing.stop(&mut *self.sink, &device, 0.0, true);
                stopped += 1;
            }
        }
        for request in &mut self.requests {
            if request.data.emitter == Some(emitter) {
                request.cancel = true;
                stopped += 1;
            }
        }
        stopped
    }

    /// `SoundManager::StopSoundRequest`: a sound still loading will not play.
    pub fn stop_sound_request(&mut self, handle: u32) {
        if let Some(request) = self.requests.iter_mut().find(|r| r.handle == handle) {
            request.cancel = true;
        }
    }

    /// `PlayingSoundContext::Stop`: the playing sound `handle` stops (fading over `fade`
    /// seconds), or its load is cancelled.
    pub fn stop_context(&mut self, handle: u32, fade: f32) {
        if handle == 0 {
            return;
        }
        let device = self.device;
        match self.playing.iter_mut().find(|p| p.handle == handle) {
            Some(playing) => playing.stop(&mut *self.sink, &device, fade, false),
            None => self.stop_sound_request(handle),
        }
    }

    /// `PlayingSoundContext::SetVolume`.
    pub fn set_context_volume(&mut self, handle: u32, volume: f32) {
        let device = self.device;
        if let Some(playing) = self.playing.iter_mut().find(|p| p.handle == handle) {
            playing.set_volume(&mut *self.sink, &device, volume);
        }
    }

    /// `SoundManager::HasSoundFinished`: neither playing nor loading.
    pub fn has_sound_finished(&self, handle: u32) -> bool {
        handle == 0
            || (!self.playing.iter().any(|p| p.handle == handle)
                && !self.requests.iter().any(|r| r.handle == handle))
    }

    /// `DirectXSoundDevice::SetChannelVolume`: `0..1`; the channel's sounds follow.
    pub fn set_channel_volume(&mut self, channel: Channel, volume: f32) -> bool {
        if !(0.0..=1.0).contains(&volume) {
            return false;
        }
        let slot = &mut self.device.volumes[channel as usize];
        // `AreFloatEqual(.., 0.0002)`.
        if (f64::from(*slot) - f64::from(volume)).abs() <= 0.0002 {
            return true;
        }
        *slot = volume;
        let device = self.device;
        for playing in &mut self.playing {
            playing.recalculate_volume(&mut *self.sink, &device);
        }
        true
    }

    /// A channel's volume.
    pub fn channel_volume(&self, channel: Channel) -> f32 {
        self.device.volumes[channel as usize]
    }

    /// `SoundManager::HandleChangeInternal` / `HandleApply` for one preference.
    pub fn set_preference(&mut self, preference: SoundPreference) {
        match preference {
            SoundPreference::SoundVolume(v) => {
                self.set_channel_volume(Channel::Main, v);
            }
            SoundPreference::MusicVolume(v) => {
                self.set_channel_volume(Channel::Music, v);
                self.last_music_volume = v;
            }
            SoundPreference::PlayerMusic(v) => {
                self.set_channel_volume(Channel::Player, v);
            }
            SoundPreference::AmbientVolume(v) => {
                self.set_channel_volume(Channel::Ambient, v);
            }
            SoundPreference::NumPlayingSounds(v) => {
                self.max_secondary = (f64::from(v).trunc() as u32).clamp(8, 64);
            }
            SoundPreference::DisableSound(disabled) => {
                self.disabled = disabled;
                self.enable_device(!disabled);
            }
            SoundPreference::DisableMusic(disabled) => {
                self.music_disabled = disabled;
                if disabled {
                    self.stop_sounds(3);
                }
            }
        }
    }

    /// `DirectXSoundDevice::Enable`: every playing sound hears of it (`OnDeviceStateChanged`).
    fn enable_device(&mut self, enabled: bool) {
        if self.device.enabled == enabled {
            return;
        }
        self.device.enabled = enabled;
        let device = self.device;
        for playing in &mut self.playing {
            playing.on_device_state_changed(&mut *self.sink, &device);
        }
    }

    /// Whether `ID_Sound_DisableSound` is set.
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// The handles playing (or kept alive), in priority order.
    pub fn playing(&self) -> Vec<u32> {
        self.playing.iter().map(|p| p.handle).collect()
    }

    /// `SoundManager::Shutdown`: every sound stopped, the resident sounds and loads dropped.
    pub fn shutdown(&mut self) {
        self.flush_sounds(false);
        self.loaded.clear();
        self.lru.clear();
        self.requests.clear();
    }
}
