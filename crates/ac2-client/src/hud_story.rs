//! `gmStoryUI` on the frame: the story slot's events draw the background and the story fields
//! (`gmStoryUI::Play` shows a keyframe, `~gmStoryUI` hides them), and the skip key reaches the
//! slot (`gmStoryUI::ListenToMessage` 0xffff0003 with `0x4000001E`). The story's music and voice
//! play in the client (`Client::play_story_sounds`); the HUD only draws them.

use ac2_ui::{StoryEvent, GLOBAL_KEY_PRESS, SKIP_STORY_MESSAGE};

use crate::client::Client;
use crate::hud_view::HudView;

impl HudView {
    /// The skip key first, then the events the slot made (the key's own teardown included), so a
    /// story skipped this frame hides this frame.
    pub(crate) fn story_frameworks(&mut self, client: &mut Client) {
        for message in &self.player_messages {
            if message.message == GLOBAL_KEY_PRESS && message.data as u32 == SKIP_STORY_MESSAGE {
                client.story_key(SKIP_STORY_MESSAGE);
            }
        }
        let events = client.take_story_events();
        let (Some(story), Some((tree, _))) = (self.story.as_mut(), self.hud.as_mut()) else {
            return;
        };
        for event in events {
            match event {
                StoryEvent::Show(show) => story.show(tree, &show),
                StoryEvent::Hide => story.hide(tree),
                StoryEvent::StopAudio
                | StoryEvent::MouseLook(_)
                | StoryEvent::PlayScenesDone(_) => {}
            }
        }
    }
}
