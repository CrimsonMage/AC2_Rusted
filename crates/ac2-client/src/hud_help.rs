//! The help window's frame side: `KeyStone::OpenHelp` opens a topic's page, the help buttons of
//! the panels open theirs, and the key stone key toggles the window.
//!
//! Retail: a panel's `ListenToMessage` calls `KeyStone::OpenHelp(ID_Help_<topic>, table)` when its
//! help button is released (`gmSkillUI`, `gmOptionUI`, ...). `gmToolbarUI::ListenToMessage` turns
//! the key stone key (`0x400000BB`) into `ShowKeyStone(!shown)`: the default page (token 0) when
//! it opens, `KeyStone::CloseHelp` when it closes. The book and the social panel return their
//! help requests from their own messages ([`crate::hud_book`], [`crate::hud_player_message`]).
//! The frame's pass is [`ac2_ui::help_pass`].

use ac2_ui::UiMessage;

use crate::hud_view::HudView;

impl HudView {
    /// `KeyStone::OpenHelp(token)`: the topic's page in the help window (token 0 is the default
    /// page). The window reads the pages it already holds; no file is read here.
    pub(crate) fn open_help(&mut self, token: u32) {
        let (Some(help), Some((tree, _))) = (self.panels.help.as_mut(), self.hud.as_mut()) else {
            return;
        };
        help.open(tree, self.panels.help_pages.as_ref(), token);
        self.dirty = true;
    }

    /// The help frameworks' messages (`ac2_ui::help_pass`): the key stone key, the help window's
    /// own messages, and the panels' help buttons.
    pub(crate) fn help_pass(&mut self, messages: &[UiMessage]) {
        let (Some(help), Some((tree, _))) = (self.panels.help.as_mut(), self.hud.as_mut()) else {
            return;
        };
        if ac2_ui::help_pass(help, tree, self.panels.help_pages.as_ref(), messages) {
            self.dirty = true;
        }
    }
}
