//! `PlaySoundData`: what a caller asks `SoundManager::Play` for.

/// `PlaySoundData::LoopingMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum LoopingMode {
    /// The sound's `loop` flag decides.
    #[default]
    Inherit = 0,
    ForceOff = 1,
    ForceOn = 2,
}

/// `PlaySoundData::Type`: exclusivity and whether the sound counts against the budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum SoundType {
    #[default]
    Inherit = 0,
    ForceNonExclusive = 1,
    ForceNonExclusiveBudgeted = 2,
    ForceSelfExclusive = 3,
    ForceSelfExclusiveBudgeted = 4,
    ForceAllExclusive = 5,
}

/// `PlaySoundData::Track`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum Track {
    #[default]
    Inherit = 0,
    ForcePrimary = 1,
    ForceSecondary = 2,
}

/// `PlaySoundData::Channel`: which volume setting a sound follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[repr(u32)]
pub enum Channel {
    /// `SoundVolume` (effects and UI).
    #[default]
    Main = 0,
    /// `MusicVolume`.
    Music = 1,
    /// `PlayerMusic`.
    Player = 2,
    /// `AmbientSoundVolume`.
    Ambient = 3,
}

impl Channel {
    pub const ALL: [Channel; 4] = [Self::Main, Self::Music, Self::Player, Self::Ambient];
}

/// `PlaySoundData`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaySoundData {
    /// `m_didSound`: a `SoundInfo` or `MusicInfo`.
    pub sound: u32,
    /// `m_pEmitter`: the object the sound comes from (`ISoundObj`), none for the UI.
    pub emitter: Option<u64>,
    /// `m_fVolume`.
    pub volume: f32,
    /// `m_fFrequency`: the playback rate's multiplier.
    pub frequency: f32,
    pub looping: LoopingMode,
    pub kind: SoundType,
    pub track: Track,
    pub channel: Channel,
}

impl PlaySoundData {
    /// `PlaySoundData::PlaySoundData` with the sound set: full volume and rate, everything
    /// inherited, the main channel. `MD_Data_Sound::Update` plays the UI's sounds with exactly
    /// this.
    pub fn new(sound: u32) -> Self {
        Self {
            sound,
            emitter: None,
            volume: 1.0,
            frequency: 1.0,
            looping: LoopingMode::Inherit,
            kind: SoundType::Inherit,
            track: Track::Inherit,
            channel: Channel::Main,
        }
    }
}
