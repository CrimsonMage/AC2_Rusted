//! `gmStoryUI` on the frame: the story slot's events draw the background and the story fields
//! (`gmStoryUI::Play` shows a keyframe, `~gmStoryUI` hides them), and the skip key reaches the
//! slot (`gmStoryUI::ListenToMessage` 0xffff0003 with `0x4000001E`). The story's music and voice
//! play in the client (`Client::play_story_sounds`); the HUD only draws them.

use ac2_net::protocol::StringInfo;
use ac2_ui::{StoryEvent, GLOBAL_KEY_PRESS, SKIP_STORY_MESSAGE};

use crate::client::Client;
use crate::hud_view::HudView;
use crate::ui_sources::UiStringTables;

/// A subtitle's text: its `StringInfo` rendered on the loaded string tables, as the other
/// `StringInfo` texts are (`ClientPlayer::render_info`). Before the tables load, a literal is
/// its own text and a table entry has none.
fn subtitle_text(strings: Option<&UiStringTables>, info: &StringInfo) -> String {
    match strings {
        Some(strings) => {
            let info = crate::chat_text::render_info(info);
            String::from_utf16_lossy(&ac2_db::get_string(&strings.tables, &info, false))
        }
        None => info.literal.clone().unwrap_or_default(),
    }
}

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
        let strings = self.strings.clone();
        let (Some(story), Some((tree, _))) = (self.story.as_mut(), self.hud.as_mut()) else {
            return;
        };
        for event in events {
            match event {
                StoryEvent::Show(show) => {
                    story.show(tree, &show, &|info| subtitle_text(strings.as_deref(), info));
                }
                StoryEvent::Hide => story.hide(tree),
                StoryEvent::StopAudio
                | StoryEvent::MouseLook(_)
                | StoryEvent::PlayScenesDone(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Without the string tables a literal subtitle is its own text, and a table entry has none.
    #[test]
    fn a_subtitle_without_tables_is_its_literal() {
        assert_eq!(subtitle_text(None, &StringInfo::literal("Hello")), "Hello");
        let table = StringInfo {
            string_id: 0x1234,
            table_id: 0x2500_0014,
            literal: None,
            variables: Vec::new(),
        };
        assert_eq!(subtitle_text(None, &table), "");
    }
}
