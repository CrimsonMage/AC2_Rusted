//! `gmGamePlayUI`'s story slot (`gameplay_story.rs`): a play sequence starts a skippable story,
//! a portal scene a non-skippable one, the slot holds one story, and the teardown reports the
//! scenes played and restores mouse look.

use crate::gameplay_story::{GameplayStory, StoryEvent, StoryScene};
use crate::story::{StoryFrame, StoryShow, SKIP_STORY_MESSAGE};

/// A scene of `frames` one-second keyframes, each with its own image (`id * 10 + index`).
fn scene(id: u32, playable: bool, frames: u32) -> StoryScene {
    StoryScene {
        id,
        playable,
        frames: (0..frames)
            .map(|index| StoryFrame {
                image: Some(id * 10 + index),
                duration: Some(1.0),
                ..StoryFrame::default()
            })
            .collect(),
    }
}

/// The `Show` events of `events`.
fn shows(events: &[StoryEvent]) -> Vec<StoryShow> {
    events
        .iter()
        .filter_map(|event| match event {
            StoryEvent::Show(show) => Some(show.clone()),
            _ => None,
        })
        .collect()
}

/// A story started by `scenes` (play sequence or portal scene) with its prefetch and fade in.
fn started(game: &mut GameplayStory, scenes: &[StoryScene]) {
    game.play_sequence(scenes);
    game.prefetch_done(0.0);
    game.fade_done(0.0);
}

#[test]
fn a_play_sequence_takes_the_slot_with_mouse_look_off_and_starts_when_ready() {
    let mut game = GameplayStory::new();
    let scenes = [scene(1, true, 2)];
    assert_eq!(
        game.play_sequence(&scenes),
        vec![StoryEvent::MouseLook(false)]
    );
    assert!(game.is_active());
    // The story waits for its prefetch and its fade.
    assert!(game.prefetch_done(0.0).is_empty());
    let events = game.fade_done(0.0);
    let first = shows(&events);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].image, Some(10));
}

#[test]
fn a_second_play_sequence_is_dropped_while_a_story_holds_the_slot() {
    let mut game = GameplayStory::new();
    started(&mut game, &[scene(1, true, 1)]);
    assert!(game.play_sequence(&[scene(2, true, 1)]).is_empty());
    // The first story runs to its end and reports only its own scene.
    let events = game.tick(1.0);
    assert_eq!(events.last(), Some(&StoryEvent::PlayScenesDone(vec![1])));
    assert!(!game.is_active());
}

#[test]
fn the_last_keyframe_ends_the_story_and_reports_the_scenes_played() {
    let mut game = GameplayStory::new();
    game.play_sequence(&[scene(1, true, 1), scene(2, false, 3), scene(3, true, 1)]);
    game.prefetch_done(0.0);
    // The playable scenes' two keyframes make the story; the hidden scene adds none.
    let mut all = game.fade_done(0.0);
    let mut now = 0.0;
    for _ in 0..4 {
        now += 1.0;
        all.extend(game.tick(now));
    }
    let images: Vec<Option<u32>> = shows(&all).iter().map(|s| s.image).collect();
    assert_eq!(images, vec![Some(10), Some(30)]);
    assert!(all.contains(&StoryEvent::MouseLook(true)));
    assert!(all.contains(&StoryEvent::StopAudio));
    // Every scene of the list is reported, hidden ones included.
    assert_eq!(all.last(), Some(&StoryEvent::PlayScenesDone(vec![1, 2, 3])));
    assert!(!game.is_active());
}

#[test]
fn a_skip_key_finishes_only_a_skippable_play_sequence() {
    let mut game = GameplayStory::new();
    started(&mut game, &[scene(1, true, 3)]);
    let events = game.key(SKIP_STORY_MESSAGE);
    assert!(events.contains(&StoryEvent::MouseLook(true)));
    assert_eq!(events.last(), Some(&StoryEvent::PlayScenesDone(vec![1])));
    assert!(!game.is_active());

    // A portal scene cannot be skipped.
    let mut game = GameplayStory::new();
    game.portal_scene(true, &[scene(4, true, 2)]);
    game.prefetch_done(0.0);
    game.fade_done(0.0);
    assert!(game.key(SKIP_STORY_MESSAGE).is_empty());
    assert!(game.is_active());
}

#[test]
fn a_portal_scene_once_entered_keeps_playing_through_the_leave_notice() {
    let mut game = GameplayStory::new();
    let entering = [scene(7, true, 2)];
    assert_eq!(
        game.portal_scene(true, &entering),
        vec![StoryEvent::MouseLook(false)]
    );
    assert!(!game.weenie_ready());
    game.prefetch_done(0.0);
    let shown = shows(&game.fade_done(0.0));
    assert_eq!(shown.len(), 1);
    // `RecvNotice_PortalScene` never clears `_playingPortalScene`; the leave only sets
    // `_weenieReady`, and the story holding the slot plays on.
    assert!(game.portal_scene(false, &[scene(8, true, 1)]).is_empty());
    assert!(game.weenie_ready());
    assert!(game.is_active());
}

#[test]
fn leaving_a_portal_scene_without_a_story_still_reports_its_scenes() {
    let mut game = GameplayStory::new();
    assert_eq!(
        game.portal_scene(false, &[scene(9, true, 1)]),
        vec![StoryEvent::PlayScenesDone(vec![9])]
    );
}

#[test]
fn a_portal_scene_does_not_replace_a_story_that_holds_the_slot() {
    let mut game = GameplayStory::new();
    started(&mut game, &[scene(1, true, 2)]);
    assert!(game.portal_scene(true, &[scene(5, true, 1)]).is_empty());
    // The play sequence is still the story: its keyframes keep coming.
    assert!(game.is_active());
    assert_eq!(shows(&game.tick(1.0)).len(), 1);
}

#[test]
fn a_story_without_playable_frames_finishes_as_soon_as_it_starts() {
    let mut game = GameplayStory::new();
    game.play_sequence(&[scene(3, false, 4)]);
    game.prefetch_done(0.0);
    assert_eq!(
        game.fade_done(0.0),
        vec![
            StoryEvent::MouseLook(true),
            StoryEvent::PlayScenesDone(vec![3]),
        ]
    );
    assert!(!game.is_active());
}

#[test]
fn ticks_do_nothing_before_the_story_starts_or_without_one() {
    let mut game = GameplayStory::new();
    assert!(game.tick(10.0).is_empty());
    game.play_sequence(&[scene(1, true, 1)]);
    assert!(game.tick(10.0).is_empty());
    assert!(game.is_active());
}

#[test]
fn end_session_clears_the_slot_without_reporting_scenes() {
    let mut game = GameplayStory::new();
    started(&mut game, &[scene(1, true, 2)]);
    let events = game.end_session();
    assert_eq!(
        events,
        vec![StoryEvent::MouseLook(true), StoryEvent::StopAudio]
    );
    assert!(!game.is_active());
    assert!(game.end_session().is_empty());
}
