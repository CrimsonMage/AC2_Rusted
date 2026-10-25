//! The client's sound: retail's `SoundManager` (`ac2-audio`) over the DATs, with sounds loaded
//! on its loader thread. The UI's media are its first users: every sound an element's state
//! media played (`MD_Data_Sound`) is played this frame.

use std::sync::Arc;

use ac2_audio::{DatSounds, PlaySoundData, SoundManager, SoundSink};
use ac2_db::ResourceStore;

/// The sound manager over the DATs in `store`, its voices going to `sink`.
pub fn sound_manager(store: Arc<ResourceStore>, sink: Box<dyn SoundSink>) -> SoundManager {
    SoundManager::new(sink, Arc::new(DatSounds::new(store)))
}

/// `MD_Data_Sound::Update`: each sound the UI's media played goes to `SoundManager::Play`
/// with `PlaySoundData`'s defaults (no emitter, full volume and rate, everything inherited,
/// the main channel).
pub fn play_ui_sounds(manager: &mut SoundManager, sounds: &[u32]) {
    for &sound in sounds {
        manager.play(&PlaySoundData::new(sound));
    }
}
