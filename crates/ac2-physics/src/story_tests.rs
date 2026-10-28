//! `StoryManager`'s ordering of a story's hooks against the decompile: hooks fire as the
//! attacker's animation reaches its story hooks, the rest when the behaviour is done.

use ac2_net::protocol::{BehaviorParams, PhysicsStory, StoryHook};

use super::{StoryManager, StoryStep, STORY_ANIM_HOOK};

fn swing() -> BehaviorParams {
    BehaviorParams {
        bitfield: 1,
        behavior: 0x4100_0089,
        ..BehaviorParams::default()
    }
}

fn hook(target: u64, anim_hook: u32) -> StoryHook {
    StoryHook {
        bitfield: 1 | 0x100,
        target: Some(target),
        anim_hook: Some(anim_hook),
        ..StoryHook::default()
    }
}

fn story(behavior: Option<BehaviorParams>, hooks: Vec<StoryHook>) -> PhysicsStory {
    PhysicsStory {
        bitfield: 2,
        attacker: Some(7),
        attacker_behavior: behavior,
        hooks,
        ..PhysicsStory::default()
    }
}

fn targets(due: &[StoryStep]) -> Vec<u64> {
    due.iter()
        .filter_map(|step| match step {
            StoryStep::Hook(_, hook) => hook.target,
            StoryStep::Cleanup(_) => None,
        })
        .collect()
}

/// The attackers of the stories cleaned up, in order.
fn cleaned(due: &[StoryStep]) -> Vec<u64> {
    due.iter()
        .filter_map(|step| match step {
            StoryStep::Cleanup(story) => story.attacker,
            StoryStep::Hook(..) => None,
        })
        .collect()
}

#[test]
fn a_story_without_a_behaviour_fires_every_hook_at_once() {
    let mut manager = StoryManager::default();
    let due = manager.do_story(story(None, vec![hook(1, 0), hook(2, 3)]), |_, _| {
        panic!("no behaviour runs")
    });
    assert_eq!(targets(&due), [1, 2]);
    assert!(
        matches!(due.last(), Some(StoryStep::Cleanup(_))),
        "cleaned up after its hooks"
    );
    assert!(manager.current().is_none());
}

#[test]
fn hooks_wait_for_the_animation_to_reach_them() {
    let mut manager = StoryManager::default();
    let mut ran = None;
    // Inserted out of order: the multiset sorts them by animation hook.
    let due = manager.do_story(
        story(Some(swing()), vec![hook(3, 2), hook(1, 0), hook(2, 1)]),
        |behavior, context| {
            ran = Some((behavior.behavior, context));
            true
        },
    );
    assert_eq!(ran, Some((0x4100_0089, 1)));
    // Hook 0 is reached at the start.
    assert_eq!(targets(&due), [1]);
    // Other weenie hooks do not move the story.
    assert!(manager.handle_anim_hook(0x4000_0001).is_empty());
    assert_eq!(targets(&manager.handle_anim_hook(STORY_ANIM_HOOK)), [2]);
    // The behaviour ends before the last story hook: the rest fire.
    assert!(
        manager.handle_behavior_done(99, 0).is_empty(),
        "another context"
    );
    assert_eq!(targets(&manager.handle_behavior_done(1, 0)), [3]);
    assert!(manager.current().is_none());
}

#[test]
fn a_failed_behaviour_drops_the_story() {
    let mut manager = StoryManager::default();
    let due = manager.do_story(story(Some(swing()), vec![hook(1, 0)]), |_, _| false);
    assert!(targets(&due).is_empty());
    assert_eq!(
        cleaned(&due),
        [7],
        "CleanupStory undoes what the story set up"
    );
    assert!(manager.current().is_none());
    let mut manager = StoryManager::default();
    manager.do_story(story(Some(swing()), vec![hook(1, 1)]), |_, _| true);
    let due = manager.handle_behavior_done(1, 2);
    assert!(targets(&due).is_empty());
    assert_eq!(cleaned(&due), [7], "an error cleans up");
    assert!(manager.current().is_none());
}

#[test]
fn a_new_story_finishes_the_running_one_first() {
    let mut manager = StoryManager::default();
    manager.do_story(story(Some(swing()), vec![hook(1, 2)]), |_, _| true);
    let mut context = 0;
    let due = manager.do_story(story(Some(swing()), vec![hook(5, 1)]), |_, c| {
        context = c;
        true
    });
    assert_eq!(
        targets(&due),
        [1],
        "the old story's hook fires, the new one waits"
    );
    assert!(
        matches!(&due[..], [StoryStep::Hook(..), StoryStep::Cleanup(_)]),
        "the old story is cleaned up before the new one starts: {due:?}"
    );
    assert_eq!(context, 2, "each behaviour takes the next context");
    assert_eq!(targets(&manager.handle_anim_hook(STORY_ANIM_HOOK)), [5]);
}
