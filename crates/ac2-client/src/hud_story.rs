//! `gmStoryUI` on the frame: the story slot's events draw the background and the story fields
//! (`gmStoryUI::Play` shows a keyframe, `~gmStoryUI` hides them), and the skip key reaches the
//! slot (`gmStoryUI::ListenToMessage` 0xffff0003 with `0x4000001E`). The story's music and voice
//! play in the client (`Client::play_story_sounds`); the HUD only draws them.

use ac2_ui::{GLOBAL_KEY_PRESS, SKIP_STORY_MESSAGE};

use crate::client::Client;
use crate::hud_view::HudView;

impl HudView {
    /// The skip key first, then the background's fade (its `token_play` starts the story), then
    /// the events the slot made (the key's own teardown included), so a story skipped this frame
    /// hides this frame.
    pub(crate) fn story_frameworks(&mut self, client: &mut Client) {
        for message in &self.player_messages {
            if message.message == GLOBAL_KEY_PRESS && message.data as u32 == SKIP_STORY_MESSAGE {
                client.story_key(SKIP_STORY_MESSAGE);
            }
        }
        let fades = self
            .panels
            .story
            .as_ref()
            .zip(self.hud.as_ref())
            .is_some_and(|(story, _)| story.fades_by_media());
        client.story_fades_on_hud = fades;
        let fade_done = self.panels.story.as_ref().is_some_and(|story| {
            self.player_messages
                .iter()
                .any(|message| story.is_fade_done(message))
        });
        if fade_done {
            client.story_fade_done();
        }
        let events = client.take_story_events();
        let strings = self.strings.clone();
        let gameplay = match (self.panels.story.as_mut(), self.hud.as_mut()) {
            (Some(story), Some((tree, _))) => story.frame_events(tree, events, strings.as_deref()),
            _ => return,
        };
        // The gameplay panels leave while a story plays and come back as they were after it.
        for shown in gameplay {
            self.set_story_gameplay_shown(shown);
        }
    }

    /// The gameplay panels' show state for a story: hidden at its start (the state they had is
    /// kept), restored at its end.
    fn set_story_gameplay_shown(&mut self, shown: bool) {
        if shown {
            if let Some(before) = self.story_hid_gameplay.take() {
                self.set_gameplay_shown(before);
            }
        } else if self.story_hid_gameplay.is_none() {
            self.story_hid_gameplay = Some(self.gameplay_shown);
            self.set_gameplay_shown(false);
        }
    }
}
