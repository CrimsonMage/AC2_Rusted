//! Asheron's Call 2 audio (retail `portal/client/src/sound`): the DAT sound records, decoded on
//! a loader thread; `SoundManager`, which the frame thread asks to play sounds and runs each
//! frame; the music queue (`DXMusicMessageThread`), which plays the DirectMusic segments of the
//! music voices and sends their events to an event sink; and the output the voices go to (the
//! speakers, or a null or recording sink).
//!
//! A music segment's notes are synthesised ([`synth`]) inside the same mixer, so the speakers play
//! music and sound alike. The frame thread never reads or decodes a file: a sound not yet resident
//! is loaded on the loader thread and plays from a later [`SoundManager::use_time`].

mod ambient;
mod ambient_data;
mod ambient_driver;
mod ambient_objects;
mod data;
mod headless;
mod loader;
mod manager;
mod mixer;
pub mod music;
mod music_thread;
mod music_voice;
#[cfg(feature = "output")]
mod output;
mod play_data;
mod playing;
mod portal;
pub mod prefs;
mod rain;
mod screen_sounds;
mod sink;
mod space;
pub mod synth;

#[cfg(test)]
mod ambient_tests;
#[cfg(test)]
mod music_output_tests;
#[cfg(test)]
mod music_wave_tests;
#[cfg(test)]
mod tests;

pub use ambient::{is_ambient_emitter, AmbientSound, AMBIENT_EMITTERS};
pub use ambient_data::{
    AmbientData, AmbientSoundInfo, ContinSoundInfo, IntermitSoundInfo, SoundDesc,
};
pub use ambient_driver::{is_outside, AmbientDriver, AmbientPoint, AmbientRequest};
pub use data::{
    flags, sound_db_type, DatSounds, LoadedSound, MusicInfo, Pcm, SoundInfo, SoundSource, Wave,
    DB_TYPE_MUSIC_INFO, DB_TYPE_SOUND_INFO, DB_TYPE_WAVE,
};
pub use headless::MixingSink;
pub use manager::{SoundManager, SoundPreference};
pub use mixer::{spatial_gains, Mixer};
pub use music_thread::{
    MusicEventKind, MusicHandler, MusicThread, SegmentEvent, SegmentInfo, SegmentKind,
};
#[cfg(feature = "output")]
pub use output::SpeakerSink;
pub use play_data::{Channel, LoopingMode, PlaySoundData, SoundType, Track};
pub use playing::Emitter;
pub use portal::{PortalCue, PortalSoundIds, PortalSounds};
pub use rain::{RainSoundIds, RainSounds};
pub use screen_sounds::ScreenSounds;
pub use sink::{
    amplitude, millibels, Boundary, NullSink, RecordingSink, SinkCommand, SoundSink, Spatial,
    SILENT,
};
pub use space::{emitter, EnvCellPlace, SoundSpace};
