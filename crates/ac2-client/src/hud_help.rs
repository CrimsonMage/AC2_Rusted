//! The help window's frame side: `KeyStone::OpenHelp` opens a topic's page, the help buttons of
//! the panels open theirs, and the key stone key toggles the window.
//!
//! Retail: a panel's `ListenToMessage` calls `KeyStone::OpenHelp(ID_Help_<topic>, table)` when its
//! help button is released (`gmSkillUI`, `gmOptionUI`, ...). `gmToolbarUI::ListenToMessage` turns
//! the key stone key (`0x400000BB`) into `ShowKeyStone(!shown)`: the default page (token 0) when
//! it opens, `KeyStone::CloseHelp` when it closes. The book and the social panel return their
//! help requests from their own messages ([`crate::hud_player_message`]).
//! The frame's pass is [`ac2_ui::help_pass`].

use crate::hud_view::HudView;

impl HudView {
    /// The window pixel a `gm3DItemsUI` target search waits at (`StartFind3DTarget`), if one does.
    pub(crate) fn world_pick_point(&self) -> Option<(i32, i32)> {
        self.panels.items_3d.as_ref()?.wants_pick()?;
        Some(self.pointer())
    }

    /// The gameplay panels' show state for a story: hidden at its start (the state they had is
    /// kept), restored at its end.
    pub(crate) fn set_story_gameplay_shown(&mut self, shown: bool) {
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
