//! `gmGamePlayUI`'s story slot (`_storyUI`) and its portal-scene flag (`_playingPortalScene`): the
//! `gmStoryUI` a play sequence or a portal scene starts (`RecvNotice_PlaySequence`,
//! `RecvNotice_PortalScene` through `CheckPortalScene`), its teardown (`RecvNotice_FinishPlaySequence`,
//! `EndSession`) and the notices that go with it. The story's own sequencing is in [`StoryPlayback`].
//!
//! A play sequence can be skipped and a portal scene cannot. While a story holds the slot, a
//! second play sequence is dropped, and a portal scene that is entered reuses nothing. Leaving a
//! portal scene, or a story finishing, takes the story out of the slot and reports the scenes
//! played (`gmCWInterface::HandlePlayScenesDone`, which the caller hands to the player's script).
//!
//! Not made here: the scene list itself. `gmSceneInfo` and `gmKeyframe` are script natives whose
//! efuns are not ported, so the caller builds the [`StoryScene`]s. The story fields, the sound
//! playback and the mouse look are the caller's, from the [`StoryEvent`]s.

use crate::story::{StoryFrame, StoryPlayback, StoryShow, StoryStep};
use ac2_weenie::{Keyframe, SceneInfo};

/// One `gmSceneInfo` of a play sequence's or a portal scene's list.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryScene {
    /// `gmSceneInfo::id`, reported back by `HandlePlayScenesDone`.
    pub id: u32,
    /// `gmSceneInfo::_bIsPlayable`: only a playable scene's keyframes play.
    pub playable: bool,
    /// The scene's keyframes, in order.
    pub frames: Vec<StoryFrame>,
}

/// The portal-space call the gameplay UI makes on its `SmartBox` (`EnterPortalSpace` or
/// `ExitPortalSpace`, both with `bInstantly` 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalSpaceCall {
    Enter,
    Exit,
}

/// What the story slot asks of its caller.
#[derive(Debug, Clone, PartialEq)]
pub enum StoryEvent {
    /// `gmStoryUI::gmStoryUI`: the background and the story fields come up, and the background's
    /// fade starts (its media reports `token_play` when done).
    Begin,
    /// `CM_UI::SendNotice_UpdateCanMouseLook`: mouse look is off while a story holds the slot
    /// (the story's constructor sends 0, its destructor 1).
    MouseLook(bool),
    /// A keyframe shows: its image, music, voice and subtitle, for its duration.
    Show(StoryShow),
    /// The story's music and voice stop (`gmStoryUI::Finish` on a story that had shown).
    StopAudio,
    /// `gmStoryUI::~gmStoryUI`: the story's fields leave the screen (its roots are removed).
    Hide,
    /// `gmCWInterface::HandlePlayScenesDone`: the scenes' ids go back to the player's script.
    PlayScenesDone(Vec<u32>),
}

/// The gameplay UI's story state.
#[derive(Debug, Default)]
pub struct GameplayStory {
    /// `_storyUI`: the story holding the slot.
    story: Option<StoryPlayback>,
    /// `_playingPortalScene`: set by the first portal scene entered and kept for the gameplay
    /// UI's life (only its constructor clears it).
    playing_portal_scene: bool,
    /// `_weenieReady`: the last portal-space or portal-scene notice was a leave; with the client
    /// ready, the portal space exits.
    weenie_ready: bool,
    /// `_clientReady`: the player is placed in the world since the last teleport started
    /// (`RecvNotice_PlayerReadyToEnterWorld` sets it, `RecvNotice_PlayerTeleportStart` clears it).
    client_ready: bool,
    /// The ids of the scenes the slot's story was made from, in list order.
    scene_ids: Vec<u32>,
    /// Whether the slot's story has shown a keyframe, so it has music or voice to stop.
    shown: bool,
}

/// The ids of `scenes` in list order.
fn ids(scenes: &[StoryScene]) -> Vec<u32> {
    scenes.iter().map(|scene| scene.id).collect()
}

impl GameplayStory {
    /// A gameplay UI with no story and no portal scene.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a story holds the slot. `gmGamePlayUI::Show` keeps the gameplay UI hidden while one
    /// does.
    pub fn is_active(&self) -> bool {
        self.story.is_some()
    }

    /// `gmGamePlayUI::RecvNotice_PlaySequence`: a skippable story starts in the empty slot. A
    /// play sequence that arrives while a story holds the slot is dropped.
    pub fn play_sequence(&mut self, scenes: &[StoryScene]) -> Vec<StoryEvent> {
        if self.story.is_some() {
            return Vec::new();
        }
        self.create(scenes, true)
    }

    /// `gmGamePlayUI::RecvNotice_PortalScene(enter, scenes)`: entering sets
    /// `_playingPortalScene` (leaving does not clear it), `_weenieReady` becomes `!enter`, then
    /// `CheckPortalScene` decides (see [`GameplayStory::check_portal_scene`]).
    pub fn portal_scene(&mut self, enter: bool, scenes: &[StoryScene]) -> Vec<StoryEvent> {
        if enter {
            self.playing_portal_scene = true;
        }
        self.weenie_ready = !enter;
        self.check_portal_scene(scenes)
    }

    /// `_weenieReady`: the server's last portal-scene notice let the player out.
    pub fn weenie_ready(&self) -> bool {
        self.weenie_ready
    }

    /// The gate every portal-space notice runs (`gmGamePlayUI::RecvNotice_PortalSpace` and the
    /// others): `SmartBox::ExitPortalSpace` once the weenie and the client are both ready,
    /// otherwise `SmartBox::EnterPortalSpace`.
    fn portal_gate(&self) -> PortalSpaceCall {
        if self.weenie_ready && self.client_ready {
            PortalSpaceCall::Exit
        } else {
            PortalSpaceCall::Enter
        }
    }

    /// `gmGamePlayUI::RecvNotice_PlayerTeleportStart`: `_clientReady` clears, then unless a portal
    /// scene plays the gate runs (the player is not placed yet, so it enters).
    pub fn player_teleport_start(&mut self) -> Option<PortalSpaceCall> {
        self.client_ready = false;
        (!self.playing_portal_scene).then(|| self.portal_gate())
    }

    /// `gmGamePlayUI::RecvNotice_PlayerReadyToEnterWorld`: `_clientReady` is set. Outside a portal
    /// scene the gate runs; inside one, the scene ends: `_playingPortalScene` clears and
    /// `CheckPortalScene` runs with no scenes, whose events come back.
    pub fn player_ready_to_enter_world(&mut self) -> (Option<PortalSpaceCall>, Vec<StoryEvent>) {
        self.client_ready = true;
        if !self.playing_portal_scene {
            return (Some(self.portal_gate()), Vec::new());
        }
        self.playing_portal_scene = false;
        (None, self.check_portal_scene(&[]))
    }

    /// `gmGamePlayUI::RecvNotice_PortalSpace(enter)`: `_weenieReady` becomes `!enter`, then the
    /// gate runs.
    pub fn portal_space(&mut self, enter: bool) -> Option<PortalSpaceCall> {
        self.weenie_ready = !enter;
        Some(self.portal_gate())
    }

    /// `gmGamePlayUI::CheckPortalScene`: outside a portal scene the slot's story is torn down and
    /// the given scenes are reported played. Inside one, a non-skippable story takes the slot
    /// unless one holds it already.
    fn check_portal_scene(&mut self, scenes: &[StoryScene]) -> Vec<StoryEvent> {
        if !self.playing_portal_scene {
            let mut events = self.teardown();
            events.push(StoryEvent::PlayScenesDone(ids(scenes)));
            return events;
        }
        if self.story.is_some() {
            return Vec::new();
        }
        self.create(scenes, false)
    }

    /// The background's `token_play` message, or the prefetch being in: the story starts once both
    /// are done (`gmStoryUI::CheckStart`).
    pub fn prefetch_done(&mut self, now: f64) -> Vec<StoryEvent> {
        self.with_story(|story| story.prefetch_done(now))
    }

    /// The background's fade is done (`gmStoryUI::ListenToMessage` on `token_play`).
    pub fn fade_done(&mut self, now: f64) -> Vec<StoryEvent> {
        self.with_story(|story| story.fade_done(now))
    }

    /// The per-frame step (`gmStoryUI::ListenToMessage` 0xffff0005). Only a playing story moves on,
    /// so a story that has not started shows nothing (the caller gates it, as the start does).
    pub fn tick(&mut self, now: f64) -> Vec<StoryEvent> {
        match self.story.as_ref() {
            Some(story) if story.is_playing() => self.with_story(|story| story.tick(now)),
            _ => Vec::new(),
        }
    }

    /// A key reaches the story (`gmStoryUI::ListenToMessage` 0xffff0003): the skip key finishes a
    /// skippable story.
    pub fn key(&mut self, data: u32) -> Vec<StoryEvent> {
        self.with_story(|story| story.key(data))
    }

    /// Runs `step` on the slot's story and turns its step into events. Without a story, nothing.
    fn with_story(
        &mut self,
        step: impl FnOnce(&mut StoryPlayback) -> Option<StoryStep>,
    ) -> Vec<StoryEvent> {
        let Some(story) = self.story.as_mut() else {
            return Vec::new();
        };
        match step(story) {
            None => Vec::new(),
            Some(StoryStep::Show(show)) => {
                self.shown = true;
                vec![StoryEvent::Show(show)]
            }
            Some(StoryStep::Finished) => self.finish_story(),
        }
    }

    /// `gmStoryUI::Play`'s finish, through `CM_UI::SendNotice_FinishPlaySequence` and
    /// `gmGamePlayUI::RecvNotice_FinishPlaySequence`: the story leaves the slot and its scenes are
    /// reported played.
    fn finish_story(&mut self) -> Vec<StoryEvent> {
        let scenes = std::mem::take(&mut self.scene_ids);
        let mut events = self.teardown();
        events.push(StoryEvent::PlayScenesDone(scenes));
        events
    }

    /// `gmStoryUI::gmStoryUI`: the slot's story from the scenes' playable keyframes, mouse look
    /// off. The story starts later, once its prefetch and fade are in.
    fn create(&mut self, scenes: &[StoryScene], can_be_disabled: bool) -> Vec<StoryEvent> {
        let frames = scenes
            .iter()
            .filter(|scene| scene.playable)
            .flat_map(|scene| scene.frames.iter().cloned())
            .collect();
        self.story = Some(StoryPlayback::new(frames, can_be_disabled));
        self.scene_ids = ids(scenes);
        self.shown = false;
        vec![StoryEvent::MouseLook(false), StoryEvent::Begin]
    }

    /// `gmGamePlayUI::EndSession`: the slot's story is deleted (the session's end, not a scene's).
    pub fn end_session(&mut self) -> Vec<StoryEvent> {
        self.teardown()
    }

    /// `gmStoryUI::~gmStoryUI`: the story leaves the slot, mouse look comes back and any music or
    /// voice stops. Empty without a story.
    fn teardown(&mut self) -> Vec<StoryEvent> {
        let Some(mut story) = self.story.take() else {
            return Vec::new();
        };
        story.finish();
        let mut events = vec![StoryEvent::MouseLook(true), StoryEvent::Hide];
        if std::mem::take(&mut self.shown) {
            events.push(StoryEvent::StopAudio);
        }
        self.scene_ids.clear();
        events
    }
}

/// The story's view of a list of `gmSceneInfo`s (the playable scenes' keyframes, in order).
pub fn story_scenes(scenes: &[SceneInfo]) -> Vec<StoryScene> {
    scenes
        .iter()
        .map(|scene| StoryScene {
            id: scene.id,
            playable: scene.playable,
            frames: scene.keyframes.iter().map(story_frame).collect(),
        })
        .collect()
}

/// One keyframe's tracks, subtitle and duration as the sequencer reads them. The subtitle is its
/// `StringInfo`, rendered on the HUD (a string-table entry or a literal).
pub fn story_frame(keyframe: &Keyframe) -> StoryFrame {
    StoryFrame {
        image: keyframe.has_video().then_some(keyframe.video),
        music: keyframe.has_audio().then_some(keyframe.audio),
        voice: keyframe.has_voice().then_some(keyframe.voice),
        subtitle: keyframe.has_subtitle().then(|| keyframe.subtitle.clone()),
        duration: keyframe.has_duration().then_some(keyframe.duration),
    }
}
