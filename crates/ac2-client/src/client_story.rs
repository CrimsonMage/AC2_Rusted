//! The story hooks of a physics story (`StoryManager::DoStoryHook`, `CM_Physics`): a damage hook
//! reaches the player's script as `gmCWInterface::DoUIDamage`, the IFun `0xB400C0`, with the
//! story's attacker and skill and the hook's target, attack result and health changes. The
//! script raises the chat line and the floating damage text (`Do3DDamage`).

use ac2_net::protocol::{ObjectAction, PhysicsStory};
use ac2_script::host::IFunArg;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::scripts_queries::ifun;

/// `StoryHookData`'s type for damage (the hook bitfield's low four bits).
const DAMAGE_HOOK: u32 = 1;

impl Client {
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
