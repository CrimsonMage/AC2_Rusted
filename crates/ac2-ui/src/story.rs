//! `gmStoryUI`'s sequencer: the story slides a play sequence or a portal scene shows
//! (`gmGamePlayUI::RecvNotice_PlaySequence` and `CheckPortalScene` make it; its frames are the
//! scene's keyframes).
//!
//! The story starts when both its prefetch and its background fade are done
//! (`gmStoryUI::CheckStart`, reached from `OnContextFinished` and from the `token_play` message on
//! the background element). `gmStoryUI::Play` then shows the next keyframe: its image (the two
//! story fields cross-fade when the image changes), its music and voice (the previous ones stop),
//! its subtitle, and its duration (5 seconds unless the keyframe defines one). When the frames run
//! out, or the player skips a story that can be disabled (`ListenToMessage` 0x4000001E), the story
//! finishes and `CM_UI::SendNotice_FinishPlaySequence` carries the played scenes back.
//!
//! This module is the sequencing only: it returns what each step shows and leaves the drawing,
//! the sound playback and the notices to the caller. The story fields, the prefetch and the
//! gameplay UI's `_storyUI` slot are not made here.

use ac2_net::protocol::StringInfo;

/// `gmStoryUI::Play`'s default frame duration in seconds (the double 5.0 when a keyframe has no
/// defined duration).
pub const DEFAULT_FRAME_SECONDS: f64 = 5.0;

/// `gmStoryUI::ListenToMessage` (0xffff0003): the key that skips a story which can be disabled.
pub const SKIP_STORY_MESSAGE: u32 = 0x4000_001E;

/// One keyframe as the sequencer reads it (`gmKeyframe`'s tracks, subtitle and duration). The
/// subtitle is the keyframe's `StringInfo` (a literal, or a string-table entry the HUD renders).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StoryFrame {
    /// `m_didVideoFragment`, when the keyframe has a video track (an image).
    pub image: Option<u32>,
    /// `m_didAudioFragment`, when the keyframe has an audio track (the music).
    pub music: Option<u32>,
    /// `m_didVoiceFragment`, when the keyframe has a voice track.
    pub voice: Option<u32>,
    /// `gmKeyframe::GetSubtitle`, when the keyframe has one.
    pub subtitle: Option<StringInfo>,
    /// `m_ttDuration`, when the keyframe defines its duration.
    pub duration: Option<f64>,
}

/// The story field showing the image (`gmStoryUI::_lastState`: `Frame1` or `Frame2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryField {
    First,
    Second,
}

/// What `gmStoryUI::Play` shows for one keyframe.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryShow {
    /// The index of the keyframe shown (`_currentFrame` before it advanced).
    pub index: usize,
    /// The field the image goes into (`_lastState`'s field).
    pub field: StoryField,
    /// The image now in that field (`_lastImage`, when not 0).
    pub image: Option<u32>,
    /// Whether the image changed, so the two fields were reset (`bVar5`).
    pub image_changed: bool,
    /// The music started for this keyframe (its audio track), replacing the last music.
    pub music: Option<u32>,
    /// The voice started for this keyframe (its voice track), replacing the last voice.
    pub voice: Option<u32>,
    /// The subtitle in the text field (`_lastSubtitle`, kept when the keyframe has none), as its
    /// `StringInfo`: the view renders it.
    pub subtitle: Option<StringInfo>,
    /// How long the keyframe shows, in seconds.
    pub seconds: f64,
}

/// The result of one step of the story.
#[derive(Debug, Clone, PartialEq)]
pub enum StoryStep {
    /// A keyframe is shown until its end time.
    Show(StoryShow),
    /// The story finished (`gmStoryUI::Finish`): `CM_UI::SendNotice_FinishPlaySequence` goes out
    /// with the scenes played.
    Finished,
}

/// `gmStoryUI`'s playback state.
#[derive(Debug, Clone)]
pub struct StoryPlayback {
    frames: Vec<StoryFrame>,
    /// `_currentFrame`: the next keyframe to show.
    current: usize,
    /// `_playing`: set by `CheckStart`, cleared by `Finish`.
    playing: bool,
    /// `_prefetchDone` (`OnContextFinished`).
    prefetch_done: bool,
    /// `_fadeDone` (the background's `token_play` message).
    fade_done: bool,
    /// `_endTime`, in the same seconds as the `now` the caller passes.
    end_time: f64,
    /// `_canBeDisabled`: the player may skip the story (the portal scene's is not skippable).
    can_be_disabled: bool,
    /// `_lastImage`: the last keyframe's video DID (0 before any).
    last_image: u32,
    /// `_lastState`.
    field: StoryField,
    /// `_lastSubtitle`: kept from the last keyframe that had one.
    last_subtitle: Option<StringInfo>,
}

impl StoryPlayback {
    /// `gmStoryUI::gmStoryUI(parent, scenes, bCanBeDisabled)`: the keyframes, not yet started,
    /// with `_lastState` at `Frame1`.
    pub fn new(frames: Vec<StoryFrame>, can_be_disabled: bool) -> Self {
        Self {
            frames,
            current: 0,
            playing: false,
            prefetch_done: false,
            fade_done: false,
            end_time: 0.0,
            can_be_disabled,
            last_image: 0,
            field: StoryField::First,
            last_subtitle: None,
        }
    }

    /// `_playing`: the story has started and not finished.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// `OnContextFinished`: the keyframes' prefetch is in. Starts the story if the fade is done.
    pub fn prefetch_done(&mut self, now: f64) -> Option<StoryStep> {
        self.prefetch_done = true;
        self.check_start(now)
    }

    /// The background's `token_play` message (`ListenToMessage`): the fade is done. Starts the
    /// story if the prefetch is in.
    pub fn fade_done(&mut self, now: f64) -> Option<StoryStep> {
        self.fade_done = true;
        self.check_start(now)
    }

    /// `gmStoryUI::CheckStart`: starts the story once the prefetch and the fade are both done,
    /// then plays the first keyframe. Does nothing before that, or once started.
    fn check_start(&mut self, now: f64) -> Option<StoryStep> {
        if !(self.prefetch_done && self.fade_done) || self.playing {
            return None;
        }
        self.playing = true;
        Some(self.play(now))
    }

    /// The per-frame message (`ListenToMessage` 0xffff0005): when the shown keyframe's end time
    /// has come, the next one plays.
    pub fn tick(&mut self, now: f64) -> Option<StoryStep> {
        (self.end_time <= now).then(|| self.play(now))
    }

    /// `ListenToMessage` (0xffff0003) with `data`: the skip key (0x4000001E) finishes a story
    /// that can be disabled now. Any other key, or a story that cannot be skipped, does nothing.
    pub fn key(&mut self, data: u32) -> Option<StoryStep> {
        if data != SKIP_STORY_MESSAGE || !self.can_be_disabled {
            return None;
        }
        self.finish();
        Some(StoryStep::Finished)
    }

    /// `gmStoryUI::Play`: shows the next keyframe and sets the end time, or finishes the story
    /// when no keyframe is left.
    fn play(&mut self, now: f64) -> StoryStep {
        let Some(frame) = self.frames.get(self.current).cloned() else {
            self.finish();
            return StoryStep::Finished;
        };
        let mut image_changed = false;
        if let Some(image) = frame.image {
            image_changed = image != self.last_image;
            self.last_image = image;
            if image_changed {
                self.field = match self.field {
                    StoryField::First => StoryField::Second,
                    StoryField::Second => StoryField::First,
                };
            }
        }
        if let Some(subtitle) = frame.subtitle.clone() {
            self.last_subtitle = Some(subtitle);
        }
        let seconds = frame.duration.unwrap_or(DEFAULT_FRAME_SECONDS);
        let show = StoryShow {
            index: self.current,
            field: self.field,
            image: (self.last_image != 0).then_some(self.last_image),
            image_changed,
            music: frame.music,
            voice: frame.voice,
            subtitle: self.last_subtitle.clone(),
            seconds,
        };
        self.current += 1;
        self.end_time = now + seconds;
        StoryStep::Show(show)
    }

    /// `gmStoryUI::Finish`: the story stops (and its music and voice with it, on the caller's
    /// side). Returns whether it was playing.
    pub fn finish(&mut self) -> bool {
        std::mem::replace(&mut self.playing, false)
    }
}
