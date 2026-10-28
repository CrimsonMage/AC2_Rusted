//! A physics object's story manager (`StoryManager`, `portal/shared/src/physobj/story`): the
//! server's `PhysicsStory` played on the attacker. The attacker's behaviour runs on the body
//! (`DoStory`, with a context of its own), and the story's hooks (damage, missiles) fire as the
//! behaviour's animation reaches its story hooks (`HandleAnimHook`, anim hook type `0x40000002`),
//! the rest when the behaviour is done (`HandleBehaviorDone` -> `FinishStory`). A story without a
//! behaviour fires every hook at once.
//!
//! The manager only orders the hooks; what a hook does (`DoStoryHook`: the effects, the target's
//! reaction, the weenie's damage) is the owner's.

use ac2_net::protocol::{BehaviorParams, PhysicsStory, StoryHook};

/// `WeenieHook::mHookType` of a story hook in an animation (`StoryManager::HandleAnimHook`).
pub const STORY_ANIM_HOOK: u32 = 0x4000_0002;

/// `StoryManager`: the current story, the next of its hooks, the animation hooks reached and the
/// behaviour context.
#[derive(Debug, Default)]
pub struct StoryManager {
    /// `m_current_story`, its hooks in `multiset` order (by `m_animHookNumber`, ties in arrival
    /// order). `None` once cleaned up.
    current: Option<PhysicsStory>,
    /// `m_curr_story_hook`: the index of the next hook to fire.
    next_hook: usize,
    /// `m_curr_animation_hook`: the story hooks the animation has reached.
    animation_hook: u32,
    /// `m_currBehaviorContextID`.
    context: u32,
}

/// `BehaviorParams::IsValid`: a behaviour id was packed (`mBehaviorID != INVALID`).
fn valid(behavior: &Option<BehaviorParams>) -> Option<&BehaviorParams> {
    behavior
        .as_ref()
        .filter(|params| params.bitfield & 1 != 0 && params.behavior != 0)
}

/// `PhysicsStory::IsNull`: no hooks and no valid attacker behaviour.
fn is_null(story: &PhysicsStory) -> bool {
    story.hooks.is_empty() && valid(&story.attacker_behavior).is_none()
}

impl StoryManager {
    /// The current story, while one runs.
    pub fn current(&self) -> Option<&PhysicsStory> {
        self.current.as_ref()
    }

    /// `StoryManager::DoStory`: any running story finishes (its remaining hooks fire first), then
    /// `story` starts. With a valid attacker behaviour, `run` plays it with the next context
    /// (`CPhysicsObj::DoBehavior`); when that fails the story is dropped. Returns the hooks due
    /// now, in order, with the story each belongs to.
    pub fn do_story(
        &mut self,
        story: PhysicsStory,
        run: impl FnOnce(&BehaviorParams, u32) -> bool,
    ) -> Vec<(PhysicsStory, StoryHook)> {
        let mut due = Vec::new();
        if self.current.as_ref().is_some_and(|s| !is_null(s)) {
            due.extend(self.finish());
        }
        let mut story = story;
        // The multiset's order: by animation hook, equal ones as they were inserted.
        story.hooks.sort_by_key(|hook| hook.anim_hook.unwrap_or(0));
        let behavior = valid(&story.attacker_behavior).cloned();
        self.current = Some(story);
        self.next_hook = 0;
        self.animation_hook = 0;
        let Some(behavior) = behavior else {
            due.extend(self.finish());
            return due;
        };
        self.context = self.context.wrapping_add(1);
        if self.context == 0 {
            self.context = 1;
        }
        if !run(&behavior, self.context) {
            self.cleanup();
            return due;
        }
        due.extend(self.current_hooks());
        due
    }

    /// `StoryManager::HandleAnimHook`: a story hook in the animation moves the story on.
    pub fn handle_anim_hook(&mut self, hook_type: u32) -> Vec<(PhysicsStory, StoryHook)> {
        if hook_type != STORY_ANIM_HOOK || self.current.as_ref().is_none_or(is_null) {
            return Vec::new();
        }
        self.animation_hook += 1;
        self.current_hooks()
    }

    /// `StoryManager::HandleBehaviorDone`: the story's behaviour ended; done, its remaining hooks
    /// fire (`FinishStory`); failed, the story is dropped (`CleanupStory`).
    pub fn handle_behavior_done(
        &mut self,
        context: u32,
        error: u32,
    ) -> Vec<(PhysicsStory, StoryHook)> {
        if context != self.context || self.current.is_none() {
            return Vec::new();
        }
        if error == 0 {
            self.finish()
        } else {
            self.cleanup();
            Vec::new()
        }
    }

    /// `StoryManager::DoCurrentStoryHooks`: the hooks whose animation hook has been reached.
    fn current_hooks(&mut self) -> Vec<(PhysicsStory, StoryHook)> {
        let mut due = Vec::new();
        let Some(story) = self.current.as_ref().filter(|s| !is_null(s)) else {
            return due;
        };
        while let Some(hook) = story.hooks.get(self.next_hook) {
            if self.animation_hook < hook.anim_hook.unwrap_or(0) {
                break;
            }
            due.push((story.clone(), *hook));
            self.next_hook += 1;
        }
        due
    }

    /// `StoryManager::FinishStory`: every remaining hook fires, one animation hook at a time,
    /// then the story is cleaned up.
    fn finish(&mut self) -> Vec<(PhysicsStory, StoryHook)> {
        let mut due = Vec::new();
        while let Some(story) = self.current.as_ref().filter(|s| !is_null(s)) {
            if self.next_hook >= story.hooks.len() {
                break;
            }
            self.animation_hook += 1;
            due.extend(self.current_hooks());
        }
        self.cleanup();
        due
    }

    /// `StoryManager::CleanupStory`: the story is let go.
    fn cleanup(&mut self) {
        self.current = None;
        self.next_hook = 0;
        self.animation_hook = 0;
    }
}

#[cfg(test)]
#[path = "story_tests.rs"]
mod tests;
