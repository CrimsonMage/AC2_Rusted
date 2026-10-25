//! Asheron's Call 2 audio (retail `portal/client/src/sound`): the DAT sound records, decoded on
//! a loader thread; `SoundManager`, which the frame thread asks to play sounds and runs each
//! frame; and the output the voices go to (the speakers, or a null or recording sink).
//!
//! The frame thread never reads or decodes a file: a sound not yet resident is loaded on the
//! loader thread and plays from a later [`SoundManager::use_time`].

mod data;
mod loader;
mod manager;
mod mixer;
#[cfg(feature = "output")]
mod output;
mod play_data;
mod playing;
mod sink;

#[cfg(test)]
mod tests;

pub use data::{
    flags, sound_db_type, DatSounds, LoadedSound, MusicInfo, Pcm, SoundInfo, SoundSource, Wave,
    DB_TYPE_MUSIC_INFO, DB_TYPE_SOUND_INFO, DB_TYPE_WAVE,
};
pub use manager::{SoundManager, SoundPreference};
pub use mixer::{spatial_gains, Mixer};
#[cfg(feature = "output")]
pub use output::SpeakerSink;
pub use play_data::{Channel, LoopingMode, PlaySoundData, SoundType, Track};
pub use playing::Emitter;
pub use sink::{
    amplitude, millibels, NullSink, RecordingSink, SinkCommand, SoundSink, Spatial, SILENT,
};
