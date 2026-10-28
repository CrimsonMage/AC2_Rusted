//! The sounds the screens play outside the world: the pregame music (`gmIntroUI`,
//! `gmCharacterSelectionUI`, `gmAvatarModificationUI`), the story's music and voice
//! (`gmStoryUI::Play`) and the credits' sound (`gmCreditsUI::UpdateSound`). Each keeps the handle
//! of its playing sound so the next one can replace it.

use crate::manager::SoundManager;
use crate::play_data::{Channel, LoopingMode, PlaySoundData, SoundType};

/// The screens' playing sounds (their `PlayingSoundContext`s).
#[derive(Debug, Default)]
pub struct ScreenSounds {
    /// The screen music's playing sound (`PlayingSoundContext` of the pregame screens).
    pregame: Option<u32>,
    /// `gmStoryUI::_lastMusicCtx` and `_lastVoiceCtx`.
    story_music: Option<u32>,
    story_voice: Option<u32>,
    /// The credits' sound (`gmCreditsUI::UpdateSound`).
    credit: Option<u32>,
}

impl ScreenSounds {
    /// A pregame screen's music: the previous screen's music fades out over two seconds
    /// (`PlayingSoundContext::Stop(ctx, 2.0)`), then `did` plays on `Channel_Main` at full volume,
    /// its handle kept for the next screen.
    pub fn play_pregame(&mut self, manager: &mut SoundManager, did: u32) {
        if let Some(handle) = self.pregame.take() {
            manager.stop_context(handle, 2.0);
        }
        self.pregame = manager.play(&PlaySoundData::new(did));
    }

    /// The screen music's playing sound, if one plays (its handle).
    pub fn pregame_handle(&self) -> Option<u32> {
        self.pregame
    }

    /// `gmCreditsUI::UpdateSound`'s `SoundManager::Play` of `did` on `Channel_Main`, its handle
    /// kept for the next check.
    pub fn play_credit(&mut self, manager: &mut SoundManager, did: u32) {
        self.credit = manager
            .play(&PlaySoundData::new(did))
            .filter(|handle| *handle != 0);
    }

    /// Whether the credits' sound still plays (its handle, and the sound has not finished).
    pub fn credit_busy(&self, manager: &SoundManager) -> bool {
        self.credit
            .is_some_and(|handle| !manager.has_sound_finished(handle))
    }

    /// `gmStoryUI::Play`'s sounds for a keyframe: its music and its voice, each replacing the
    /// story's last one.
    pub fn play_story(
        &mut self,
        manager: &mut SoundManager,
        music: Option<u32>,
        voice: Option<u32>,
    ) {
        if let Some(did) = music {
            replace_story_sound(manager, &mut self.story_music, did, true);
        }
        if let Some(did) = voice {
            replace_story_sound(manager, &mut self.story_voice, did, false);
        }
    }

    /// `gmStoryUI::Finish`: the story's music and voice fade out over two seconds and forget
    /// their handles.
    pub fn stop_story(&mut self, manager: &mut SoundManager) {
        for handle in [self.story_music.take(), self.story_voice.take()]
            .into_iter()
            .flatten()
        {
            manager.stop_context(handle, 2.0);
        }
    }
}

/// One of the story's sounds (`gmStoryUI::Play`): the last one it played stops over two seconds
/// (`PlayingSoundContext::Stop(ctx, 2.0)`), then `did` plays as `Type_ForceAllExclusive` on
/// `Channel_Main`, looping when it is the music (`Looping_ForceOn`).
fn replace_story_sound(manager: &mut SoundManager, last: &mut Option<u32>, did: u32, music: bool) {
    if let Some(handle) = last.take() {
        manager.stop_context(handle, 2.0);
    }
    let mut data = PlaySoundData::new(did);
    data.kind = SoundType::ForceAllExclusive;
    data.channel = Channel::Main;
    if music {
        data.looping = LoopingMode::ForceOn;
    }
    *last = manager.play(&data).filter(|handle| *handle != 0);
}
