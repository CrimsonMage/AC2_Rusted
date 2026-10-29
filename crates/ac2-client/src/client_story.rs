//! The story hooks of a physics story (`StoryManager::DoStoryHook`, `CM_Physics`): a damage hook
//! reaches the player's script as `gmCWInterface::DoUIDamage`, the IFun `0xB400C0`, with the
//! story's attacker and skill and the hook's target, attack result and health changes. The
//! script raises the chat line and the floating damage text (`Do3DDamage`).

use ac2_anim::Ac2AnimEvent;
use ac2_net::protocol::{ObjectAction, PhysicsStory, StoryHook};
use ac2_physics::StoryStep;
use ac2_script::host::interface_calls::InterfaceCalls;
use ac2_script::host::IFunArg;
use ac2_script::native::{FxAndBehavior, NativeObject};
use ac2_ui::{story_scenes, StoryEvent};
use ac2_weenie::{GameplayNotice, SceneInfo, SessionEvent, NO_SELECTION};

use crate::client::Client;
use crate::client_story_missiles::{FIRE_MISSILE_HOOK, PARENT_MISSILES_HOOK};
use crate::scripts_queries::ifun;

/// `StoryHookData`'s type for damage (the hook bitfield's low four bits).
const DAMAGE_HOOK: u32 = 1;

impl Client {
    /// The story slot's frame step. The keyframes are in once they are made (nothing is prefetched
    /// here). The background's fade is the HUD's: its media reports `token_play` when it ends
    /// ([`Client::story_fade_done`]). Without a background to draw, there is no fade to wait for,
    /// and the story starts with the frame.
    pub(crate) fn story_frame(&mut self) -> Vec<SessionEvent> {
        let now = self.session.local_time();
        let mut events = self.story.prefetch_done(now);
        if !self.story_fades_on_hud {
            events.extend(self.story.fade_done(now));
        }
        events.extend(self.story.tick(now));
        let mut out = Vec::new();
        self.story_events(events, &[], &mut out);
        out
    }

    /// The background's fade is done (`gmStoryUI::ListenToMessage` on `token_play`): the story
    /// starts, or moves on, from now. Its events join the frame.
    pub(crate) fn story_fade_done(&mut self) {
        let now = self.session.local_time();
        let events = self.story.fade_done(now);
        let mut out = Vec::new();
        self.story_events(events, &[], &mut out);
        self.story_outbox.extend(out);
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
                // `CM_UI::SendNotice_UpdateCanMouseLook`: `gmCameraUI` counts it and updates the
                // camera (the frame's owner passes the notice on).
                StoryEvent::MouseLook(allowed) => self.mouse_look_notices.push(allowed),
                // The keyframe's music and voice play (`gmStoryUI::Play`); the HUD draws the
                // keyframe and hides the fields (`ac2_ui::Hud::frame`).
                StoryEvent::Begin => self.story_queue.push(StoryEvent::Begin),
                StoryEvent::Show(show) => {
                    self.play_story_sounds(show.music, show.voice);
                    self.story_queue.push(StoryEvent::Show(show));
                }
                StoryEvent::StopAudio => {
                    self.stop_story_sounds();
                    self.story_queue.push(StoryEvent::StopAudio);
                }
                StoryEvent::Hide => self.story_queue.push(StoryEvent::Hide),
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

    /// `CPhysicsObj::RecvEvt_DoStory`: a physics story goes to its object's story manager
    /// ([`Client::do_story`]); a removed object's manager is dropped.
    pub(crate) fn apply_story(&mut self, event: &SessionEvent) {
        match event {
            SessionEvent::ObjectAction {
                object,
                action: ObjectAction::DoStory(story),
            } => self.do_story(*object, story),
            // The object's story manager goes with it.
            SessionEvent::ObjectRemoved(object) => {
                self.stories.remove_object(*object);
            }
            _ => {}
        }
    }

    /// `StoryManager::DoStory` on `object`: the attacker's behaviour plays on its body with the
    /// story's context, and the hooks already reached run. An object with no body here plays no
    /// behaviour, so its story finishes at once (the hooks still reach the script).
    fn do_story(&mut self, object: u64, story: &PhysicsStory) {
        let mut manager = self.stories.managers.remove(&object).unwrap_or_default();
        let due = match self.story_body(object) {
            Some(body) => manager.do_story(story.clone(), |params, context| {
                let mut params = crate::motion::behavior(params);
                params.context = context;
                // `CPhysicsObj::DoBehavior` returns 1 whether or not the tracker took it (a
                // refusal reaches the story as `HandleBehaviorDone`'s error); only a frozen
                // body cancels the story (`HandleBehaviorDone(context, 0x5d)`).
                let frozen = body.is_animation_frozen();
                body.do_behavior(&params);
                !frozen
            }),
            // No body to play the behaviour on: the story runs as one without a behaviour.
            None => {
                let mut bare = story.clone();
                bare.attacker_behavior = None;
                manager.do_story(bare, |_, _| false)
            }
        };
        if manager.current().is_some() {
            self.stories.managers.insert(object, manager);
        }
        self.story_steps(object, due);
    }

    /// The manager's steps on `object`, in order: each hook reached
    /// ([`Client::do_story_hook`]) and each story cleaned up ([`Client::cleanup_story`]).
    fn story_steps(&mut self, object: u64, steps: Vec<StoryStep>) {
        for step in steps {
            match step {
                StoryStep::Hook(story, hook) => self.do_story_hook(object, &story, &hook),
                StoryStep::Cleanup(story) => self.cleanup_story(object, &story),
            }
        }
    }

    /// `StoryManager::CleanupStory` of a story that was not null: the missiles still pending are
    /// destroyed and the faded weapons fade back ([`Client::cleanup_story_missiles`]); on the
    /// player (`CPhysicsObj::IsThePlayer`) the script hears its special attack ended
    /// (`gmCWInterface::SpecialAttackEnded` with the story's client attack context).
    fn cleanup_story(&mut self, object: u64, story: &PhysicsStory) {
        self.cleanup_story_missiles(object);
        if self.session.player() != Some(object) {
            return;
        }
        let context = story.client_attack_context.unwrap_or(0);
        // A call that fails leaves the script's bookkeeping out; the problem is in the scripts.
        let _ = self.call_ifun(ifun::SPECIAL_ATTACK_ENDED, &[IFunArg::Int(context)]);
    }

    /// The body a story plays on: the player's, or a remote object's.
    pub(crate) fn story_body(&mut self, object: u64) -> Option<&mut ac2_physics::PhysicsBody> {
        if self.player.as_ref().is_some_and(|p| p.id == object) {
            return self
                .player
                .as_mut()
                .and_then(|p| p.body.as_mut())
                .map(|body| &mut body.physics);
        }
        self.remotes.body_mut(object)
    }

    /// The step's animation events of each object (`HandleAnimHook` on a story hook,
    /// `HandleBehaviorDone`) move its story on; the hooks they reach run.
    pub(crate) fn story_animation_events(
        &mut self,
        events: &[(u64, Option<u32>, Vec<Ac2AnimEvent>)],
    ) {
        for (object, _, events) in events {
            let Some(mut manager) = self.stories.managers.remove(object) else {
                continue;
            };
            let mut due = Vec::new();
            for event in events {
                match *event {
                    Ac2AnimEvent::WeenieHook { hook_type, .. } => {
                        due.extend(manager.handle_anim_hook(hook_type));
                    }
                    Ac2AnimEvent::BehaviorDone { context, error } => {
                        due.extend(manager.handle_behavior_done(context, error));
                    }
                    _ => {}
                }
            }
            if manager.current().is_some() {
                self.stories.managers.insert(*object, manager);
            }
            self.story_steps(*object, due);
        }
    }

    /// `StoryManager::DoStoryHook`: the skill's effects and behaviours for the hook, as the
    /// script gives them (`gmCWInterface::GetFXAndBehaviorInformationFromWsl` with the story's
    /// skill and the hook's attack hook number and attack result): the attacker's FX on the
    /// story's `object`, the target's reaction and FX on the hook's target, and the weapon's on
    /// its weapon. Then a damage hook reaches the script as `gmCWInterface::DoUIDamage` with
    /// the story's attacker, and a harmful skill on the player names that attacker in a
    /// `LastAttacker` notice. A missile hook makes and hangs the story's missiles (type 2) or
    /// fires one (type 3), in [`crate::client_story_missiles`].
    fn do_story_hook(&mut self, object: u64, story: &PhysicsStory, hook: &StoryHook) {
        if let Some(info) = self.story_fx_info(story, hook) {
            if info.attacker_fx != 0 {
                self.story_fx(object, info.attacker_fx);
            }
            for (other, behavior, fx) in [
                (hook.target, info.target_behavior, info.target_fx),
                (hook.weapon, info.weapon_behavior, info.weapon_fx),
            ] {
                let Some(other) = other else {
                    continue;
                };
                if let Some(behavior) = behavior {
                    self.story_behavior(other, &behavior);
                }
                if fx != 0 {
                    self.story_fx(other, fx);
                }
            }
        }
        match hook.kind() {
            DAMAGE_HOOK => {}
            PARENT_MISSILES_HOOK => return self.create_and_parent_missiles(object, story),
            FIRE_MISSILE_HOOK => return self.fire_missile(object, story, hook),
            _ => return,
        }
        let Some(target) = hook.target else {
            return;
        };
        // The story's attacker (`PhysicsStory::m_attacker_id`), not the object it plays on: a
        // victim's story names who hit it.
        let attacker = story.attacker.unwrap_or(NO_SELECTION);
        let args = [
            IFunArg::Long(attacker),
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
        // A harmful skill on the player names its attacker: `ECM_Physics::SendNotice_E_LastAttacker`
        // on the player, which `gmSelectionUI` keeps for its key and auto-selects with.
        let skill = story.skill.unwrap_or(0);
        if self.session.player() == Some(target)
            && skill != 0
            && self.is_skill_harmful(skill).unwrap_or(false)
        {
            self.story_outbox
                .push(SessionEvent::Notice(GameplayNotice::LastAttacker {
                    object: target,
                    attacker,
                }));
        }
    }

    /// The `FXAndBehaviorInfo` the script fills for the hook; `None` without a script, or when
    /// the call fails.
    fn story_fx_info(&mut self, story: &PhysicsStory, hook: &StoryHook) -> Option<FxAndBehavior> {
        self.scripts.as_ref()?;
        let info = self
            .fx_and_behavior_info(
                story.skill.unwrap_or(0),
                hook.attack_hook.unwrap_or(0),
                hook.attack_result.unwrap_or(0),
            )
            .ok()??;
        match info {
            NativeObject::FxAndBehavior(info) => Some(*info),
            _ => None,
        }
    }

    /// `CPhysicsObj::DoFX(fx, 1.0)` on `object`'s body.
    fn story_fx(&mut self, object: u64, fx: u32) {
        if let Some(body) = self.story_body(object) {
            body.do_fx(fx, 1.0);
        }
    }

    /// `CPhysicsObj::DoBehavior` on `object`'s body, when the script set a behaviour
    /// (`BehaviorParams::IsValid`).
    fn story_behavior(&mut self, object: u64, params: &ac2_net::protocol::BehaviorParams) {
        let mut packed = params.clone();
        packed.bitfield = ac2_script::native::behavior_pack_bitfield(&packed);
        if packed.bitfield & 1 == 0 || packed.behavior == 0 {
            return;
        }
        if let Some(body) = self.story_body(object) {
            body.do_behavior(&crate::motion::behavior(&packed));
        }
    }
}

#[cfg(feature = "test-support")]
impl Client {
    /// The story missiles alive: each one's id, the holder it hangs from (`None` once fired),
    /// its body's position (`None` until the body is in) and whether its body flies under a
    /// missile manager.
    pub fn story_missiles(&self) -> Vec<(u64, Option<u64>, Option<ac2_physics::Position>, bool)> {
        self.stories
            .missiles()
            .into_iter()
            .map(|id| {
                let body = self.remotes.get(id).and_then(|r| r.body());
                (
                    id,
                    self.stories.missile_parent(id).map(|(holder, _)| holder),
                    body.map(|b| b.position()),
                    body.is_some_and(|b| b.has_missile_manager()),
                )
            })
            .collect()
    }

    /// Whether `object`'s body (the player's or a remote's) is playing `behavior`
    /// (`ModeTracker::IsDoingBehavior`).
    pub fn is_doing_behavior(&mut self, object: u64, behavior: u32) -> bool {
        self.story_body(object)
            .is_some_and(|body| body.is_doing_behavior(behavior))
    }

    /// Feeds session events to the story slot as a frame does (the notices, then the frame's
    /// step), returning the IFuns' output. The test harness raises the scene notices this way.
    pub fn feed_story_notices(&mut self, events: &[SessionEvent]) -> Vec<SessionEvent> {
        let mut out = self.apply_story_notices(events);
        out.extend(self.story_frame());
        out
    }
}
