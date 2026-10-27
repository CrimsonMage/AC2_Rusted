//! The story hooks of a physics story (`StoryManager::DoStoryHook`, `CM_Physics`): a damage hook
//! reaches the player's script as `gmCWInterface::DoUIDamage`, the IFun `0xB400C0`, with the
//! story's attacker and skill and the hook's target, attack result and health changes. The
//! script raises the chat line and the floating damage text (`Do3DDamage`).

use ac2_net::protocol::{ObjectAction, PhysicsStory};
use ac2_script::host::IFunArg;
use ac2_ui::{StoryEvent, StoryFrame, StoryScene};
use ac2_weenie::{GameplayNotice, Keyframe, SceneInfo, SessionEvent};

use crate::client::Client;
use crate::scripts_queries::ifun;

/// `StoryHookData`'s type for damage (the hook bitfield's low four bits).
const DAMAGE_HOOK: u32 = 1;

impl Client {
    /// The story slot's frame step: the background is never prefetched or faded here (drawing is a
    /// known gap), so the story is ready as soon as it is made, and a playing story moves on.
    pub(crate) fn story_frame(&mut self) -> Vec<SessionEvent> {
        let now = self.session.local_time();
        let mut events = self.story.prefetch_done(now);
        events.extend(self.story.fade_done(now));
        events.extend(self.story.tick(now));
        let mut out = Vec::new();
        self.story_events(events, &[], &mut out);
        out
    }

    /// `CM_Quest::SendNotice_PlaySequence` and `CM_UI::SendNotice_PortalScene`: the scenes go to
    /// the story slot, whose events are acted on here. The IFuns' output is returned.
    pub(crate) fn apply_story_notices(&mut self, events: &[SessionEvent]) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        for event in events {
            let SessionEvent::Notice(notice) = event else {
                continue;
            };
            let (scenes, story) = match notice {
                GameplayNotice::PlaySequence(scenes) => {
                    (scenes, self.story.play_sequence(&story_scenes(scenes)))
                }
                GameplayNotice::PortalScene { enter, scenes } => (
                    scenes,
                    self.story.portal_scene(*enter, &story_scenes(scenes)),
                ),
                _ => continue,
            };
            // The scenes the slot's story holds are the ones it was made from; a scene list the
            // slot dropped (a play sequence while a story holds it) is not kept.
            if story
                .iter()
                .any(|e| matches!(e, StoryEvent::MouseLook(false)))
            {
                self.story_scenes = scenes.clone();
            }
            self.story_events(story, scenes, &mut out);
        }
        out
    }

    /// Acts on the story slot's events. `notice` is the scene list of the notice that produced
    /// them, which `PlayScenesDone` may name when the slot held no story (a portal scene's leave).
    pub(crate) fn story_events(
        &mut self,
        events: Vec<StoryEvent>,
        notice: &[SceneInfo],
        out: &mut Vec<SessionEvent>,
    ) {
        for event in events {
            match event {
                // `CM_UI::SendNotice_UpdateCanMouseLook`: the camera stops mouse look at once.
                StoryEvent::MouseLook(allowed) => {
                    self.mouse_look_allowed = allowed;
                    if !allowed {
                        let subject = self.camera_subject(false);
                        self.camera.set_mouse_look(false, subject.as_ref());
                    }
                }
                // The HUD draws the keyframes and hides the fields (`hud_story.rs`); the story's
                // music and voice are not played yet (`gmStoryUI::Play`).
                event @ (StoryEvent::Show(_) | StoryEvent::StopAudio | StoryEvent::Hide) => {
                    self.story_queue.push(event);
                }
                StoryEvent::PlayScenesDone(ids) => {
                    let played = ids
                        .iter()
                        .filter_map(|id| {
                            self.story_scenes
                                .iter()
                                .chain(notice)
                                .find(|s| s.id == *id)
                                .cloned()
                        })
                        .collect();
                    self.story_scenes.clear();
                    // `gmCWInterface::HandlePlayScenesDone`: the played list to the player's script.
                    // A call that fails leaves the script's handler out; the problem is in the scripts.
                    let _ = self.ifun_into(
                        crate::scripts_queries::ifun::PLAY_SCENES_DONE,
                        &[IFunArg::Vm(
                            ac2_script::native::ntype::GM_SCENE_INFO_LIST,
                            ac2_script::native::NativeObject::SceneList(played),
                        )],
                        out,
                    );
                }
            }
        }
    }

    /// The skip key reaches the story slot (`gmStoryUI::ListenToMessage` 0xffff0003): a skippable
    /// story finishes. Its session events join the next frame.
    pub(crate) fn story_key(&mut self, data: u32) {
        let events = self.story.key(data);
        self.story_step(events);
    }

    /// `gmGamePlayUI::EndSession` on a disconnect: the slot's story is deleted and its events join
    /// the next frame.
    pub(crate) fn story_session_end(&mut self) {
        let events = self.story.end_session();
        self.story_step(events);
    }

    /// The Story page's press on a playable scene (`gmStoryQuestsUI::ListenToMessage`): the scene
    /// plays in the story slot as a play sequence of its own (`CM_Quest::SendNotice_PlaySequence`
    /// with one scene). The slot's events join the next frame.
    pub(crate) fn play_story_scene(&mut self, scene: SceneInfo) {
        let scenes = vec![scene];
        let events = self.story.play_sequence(&story_scenes(&scenes));
        if events
            .iter()
            .any(|e| matches!(e, StoryEvent::MouseLook(false)))
        {
            self.story_scenes = scenes.clone();
        }
        let mut out = Vec::new();
        self.story_events(events, &scenes, &mut out);
        self.story_outbox.extend(out);
    }

    /// Acts on the events a call outside the frame made, into [`Client::story_outbox`].
    fn story_step(&mut self, events: Vec<StoryEvent>) {
        let mut out = Vec::new();
        self.story_events(events, &[], &mut out);
        self.story_outbox.extend(out);
    }

    /// The story slot's drawn events since the last call (`Show`, `Hide`, `StopAudio`).
    pub(crate) fn take_story_events(&mut self) -> Vec<StoryEvent> {
        std::mem::take(&mut self.story_queue)
    }

    /// Runs the damage hooks of the physics stories among `events`, in order.
    pub(crate) fn apply_stories(&mut self, events: &[SessionEvent]) {
        for event in events {
            if let SessionEvent::ObjectAction {
                object,
                action: ObjectAction::DoStory(story),
            } = event
            {
                self.run_story(*object, story);
            }
        }
    }

    /// `StoryManager::DoStoryHook` for each damage hook: `gmCWInterface::DoUIDamage` with the
    /// story's `object` as the context (the story's physics object).
    fn run_story(&mut self, object: u64, story: &PhysicsStory) {
        for hook in story.hooks.iter().filter(|h| h.kind() == DAMAGE_HOOK) {
            let Some(target) = hook.target else {
                continue;
            };
            let args = [
                IFunArg::Long(object),
                IFunArg::Long(target),
                IFunArg::Int(story.skill.unwrap_or(0)),
                IFunArg::Int(hook.attack_result.unwrap_or(0)),
                IFunArg::Int(hook.target_health_change.unwrap_or(0) as u32),
                IFunArg::Int(hook.attacker_health_change.unwrap_or(0) as u32),
                IFunArg::Int(hook.target_pk_damage_change.unwrap_or(0) as u32),
                IFunArg::Int(hook.attacker_pk_damage_change.unwrap_or(0) as u32),
            ];
            // A call that fails leaves the hook's text out; the problem is in the scripts' list.
            let _ = self.call_ifun(ifun::DO_UI_DAMAGE, &args);
        }
    }
}

/// The story's view of a list of `gmSceneInfo`s (the playable scenes' keyframes, in order).
fn story_scenes(scenes: &[SceneInfo]) -> Vec<StoryScene> {
    scenes
        .iter()
        .map(|scene| StoryScene {
            id: scene.id,
            playable: scene.playable,
            frames: scene.keyframes.iter().map(story_frame).collect(),
        })
        .collect()
}

/// One keyframe's tracks, subtitle and duration as the sequencer reads them. A subtitle that is
/// a string-table entry (no literal) is left out: the table is not rendered here yet.
fn story_frame(keyframe: &Keyframe) -> StoryFrame {
    StoryFrame {
        image: keyframe.has_video().then_some(keyframe.video),
        music: keyframe.has_audio().then_some(keyframe.audio),
        voice: keyframe.has_voice().then_some(keyframe.voice),
        subtitle: keyframe
            .has_subtitle()
            .then(|| keyframe.subtitle.literal.clone())
            .flatten(),
        duration: keyframe.has_duration().then_some(keyframe.duration),
    }
}

#[cfg(feature = "test-support")]
impl Client {
    /// Feeds session events to the story slot as a frame does (the notices, then the frame's
    /// step), returning the IFuns' output. The test harness raises the scene notices this way.
    pub fn feed_story_notices(&mut self, events: &[SessionEvent]) -> Vec<SessionEvent> {
        let mut out = self.apply_story_notices(events);
        out.extend(self.story_frame());
        out
    }
}
