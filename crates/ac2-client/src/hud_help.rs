//! The help window's frame side: `KeyStone::OpenHelp` opens a topic's page, the help buttons of
//! the panels open theirs, and the key stone key toggles the window.
//!
//! Retail: a panel's `ListenToMessage` calls `KeyStone::OpenHelp(ID_Help_<topic>, table)` when its
//! help button is released (`gmSkillUI`, `gmOptionUI`, ...). `gmToolbarUI::ListenToMessage` turns
//! the key stone key (`0x400000BB`) into `ShowKeyStone(!shown)`: the default page (token 0) when
//! it opens, `KeyStone::CloseHelp` when it closes. The book and the social panel return their
//! help requests from their own messages ([`crate::hud_book`], [`crate::hud_player_message`]).

use ac2_ui::{
    help_button_topic, tokenize, UiMessage, GLOBAL_KEY_PRESS, KEYSTONE_PANEL_KEY,
    MSG_BUTTON_RELEASE_ON,
};

use crate::hud_view::HudView;

impl HudView {
    /// `KeyStone::OpenHelp(token)`: the topic's page in the help window (token 0 is the default
    /// page). The window reads the pages it already holds; no file is read here.
    pub(crate) fn open_help(&mut self, token: u32) {
        let (Some(help), Some((tree, _))) = (self.help.as_mut(), self.hud.as_mut()) else {
            return;
        };
        help.open(tree, self.help_pages.as_ref(), token);
        self.dirty = true;
    }

    /// The help frameworks' messages: the key stone key, the help window's own messages (close,
    /// slider, links), and the panels' help buttons.
    pub(crate) fn help_pass(&mut self, messages: &[UiMessage]) {
        for message in messages {
            if message.message == GLOBAL_KEY_PRESS && message.data as u32 == KEYSTONE_PANEL_KEY {
                self.toggle_keystone();
                continue;
            }
            if let (Some(help), Some((tree, _))) = (self.help.as_mut(), self.hud.as_mut()) {
                if help.message(tree, self.help_pages.as_ref(), message) {
                    self.dirty = true;
                    continue;
                }
            }
            if let Some(token) = self.help_button_token(message) {
                self.open_help(token);
            }
        }
    }

    /// `gmToolbarUI::ShowKeyStone(!shown)`: the window opens on its default page or closes.
    fn toggle_keystone(&mut self) {
        let shown = self.help.as_ref().is_some_and(|help| help.is_shown());
        if shown {
            if let (Some(help), Some((tree, _))) = (self.help.as_mut(), self.hud.as_mut()) {
                help.close(tree);
                self.dirty = true;
            }
        } else {
            self.open_help(0);
        }
    }

    /// The topic a panel's help button opens, when the message is that button's release.
    fn help_button_token(&self, message: &UiMessage) -> Option<u32> {
        if message.message != MSG_BUTTON_RELEASE_ON {
            return None;
        }
        let topic = help_button_topic(message.element_id)?;
        Some(tokenize(topic.id))
    }
}
