//! `gmStoryUI`'s sequencer (`story.rs`): the start gate, the frame order and durations, the image
//! cross-fade, the audio and subtitle carry-over, the finish and the skip rule.

use ac2_net::protocol::StringInfo;

use crate::story::{
    StoryField, StoryFrame, StoryPlayback, StoryStep, DEFAULT_FRAME_SECONDS, SKIP_STORY_MESSAGE,
};

fn image(did: u32) -> StoryFrame {
    StoryFrame {
        image: Some(did),
        ..StoryFrame::default()
    }
}

fn shown(step: Option<StoryStep>) -> crate::story::StoryShow {
    match step {
        Some(StoryStep::Show(show)) => show,
        other => panic!("expected a keyframe, got {other:?}"),
    }
}

#[test]
fn the_story_waits_for_both_the_prefetch_and_the_fade() {
    let mut story = StoryPlayback::new(vec![image(1)], true);
    assert!(story.prefetch_done(0.0).is_none());
    assert!(!story.is_playing());
    let show = shown(story.fade_done(2.0));
    assert_eq!(show.index, 0);
    assert!(story.is_playing());
    // The reverse order starts it the same way, and never twice.
    let mut other = StoryPlayback::new(vec![image(1)], true);
    assert!(other.fade_done(0.0).is_none());
    assert!(other.prefetch_done(0.0).is_some());
    assert!(other.fade_done(0.0).is_none());
}

#[test]
fn keyframes_play_in_order_for_their_durations_then_the_story_finishes() {
    let frames = vec![
        StoryFrame {
            duration: Some(2.0),
            ..image(10)
        },
        image(11),
    ];
    let mut story = StoryPlayback::new(frames, true);
    story.prefetch_done(0.0);
    let first = shown(story.fade_done(1.0));
    assert_eq!(first.seconds, 2.0);
    // The first keyframe runs to 3.0 (its end is 1.0 + 2.0).
    assert!(story.tick(2.9).is_none());
    let second = shown(story.tick(3.0));
    assert_eq!(second.index, 1);
    assert_eq!(second.seconds, DEFAULT_FRAME_SECONDS);
    // The second keyframe ends at 8.0: the next tick finishes the story.
    assert_eq!(story.tick(8.0), Some(StoryStep::Finished));
    assert!(!story.is_playing());
}

#[test]
fn a_story_without_frames_finishes_as_soon_as_it_starts() {
    let mut story = StoryPlayback::new(Vec::new(), false);
    story.prefetch_done(0.0);
    assert_eq!(story.fade_done(0.0), Some(StoryStep::Finished));
    assert!(!story.is_playing());
}

#[test]
fn an_image_change_flips_the_field_and_the_same_image_does_not() {
    let mut story = StoryPlayback::new(vec![image(5), image(5), image(6)], true);
    story.prefetch_done(0.0);
    let a = shown(story.fade_done(0.0));
    assert!(a.image_changed);
    assert_eq!(a.field, StoryField::Second);
    assert_eq!(a.image, Some(5));
    let b = shown(story.tick(a.seconds));
    assert!(!b.image_changed);
    assert_eq!(b.field, StoryField::Second);
    let c = shown(story.tick(a.seconds * 2.0));
    assert!(c.image_changed);
    assert_eq!(c.field, StoryField::First);
    assert_eq!(c.image, Some(6));
}

#[test]
fn audio_replaces_only_on_frames_that_have_it_and_subtitles_carry_over() {
    let frames = vec![
        StoryFrame {
            music: Some(77),
            voice: Some(78),
            subtitle: Some(StringInfo::literal("Hello")),
            ..StoryFrame::default()
        },
        StoryFrame {
            duration: Some(1.0),
            ..StoryFrame::default()
        },
    ];
    let mut story = StoryPlayback::new(frames, true);
    story.prefetch_done(0.0);
    let first = shown(story.fade_done(0.0));
    assert_eq!(first.music, Some(77));
    assert_eq!(first.voice, Some(78));
    assert_eq!(first.subtitle, Some(StringInfo::literal("Hello")));
    assert_eq!(first.image, None, "no image before any video track");
    let second = shown(story.tick(first.seconds));
    assert_eq!(second.music, None);
    assert_eq!(second.voice, None);
    assert_eq!(second.subtitle, Some(StringInfo::literal("Hello")));
}

#[test]
fn only_a_skippable_story_finishes_on_the_skip_message() {
    let mut skippable = StoryPlayback::new(vec![image(1)], true);
    skippable.prefetch_done(0.0);
    skippable.fade_done(0.0);
    assert_eq!(skippable.key(0x4000_0001), None, "another key does nothing");
    assert_eq!(skippable.key(SKIP_STORY_MESSAGE), Some(StoryStep::Finished));
    assert!(!skippable.is_playing());

    let mut fixed = StoryPlayback::new(vec![image(1)], false);
    fixed.prefetch_done(0.0);
    fixed.fade_done(0.0);
    assert_eq!(fixed.key(SKIP_STORY_MESSAGE), None);
    assert!(fixed.is_playing());
}

#[test]
fn finish_reports_whether_the_story_was_playing() {
    let mut story = StoryPlayback::new(vec![image(1)], true);
    assert!(!story.finish());
    story.prefetch_done(0.0);
    story.fade_done(0.0);
    assert!(story.finish());
    assert!(!story.finish());
}
