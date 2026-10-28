//! `gm3DItemsUI`'s glue: the HUD's messages go to [`ac2_ui::Items3dUi`], the selection render's
//! answer for a drop's target search comes from the view's pick ([`ClientView::pick`], the
//! object under the pointer), and its requests become the gameplay commands or the stack dialog.
//! `ClientView::frameworks` calls this before the other frameworks take the messages.

use std::cell::RefCell;

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

impl HudView {
    /// The window pixel a target search waits at (`gm3DItemsUI::StartFind3DTarget`), if one does.
    pub(crate) fn world_pick_point(&self) -> Option<(i32, i32)> {
        self.items_3d.as_ref()?.wants_pick()?;
        Some(self.pointer())
    }

    /// One frame of `gm3DItemsUI`: the search the last frame started finishes with `picked` (the
    /// view's answer for [`Self::world_pick_point`]), then the frame's messages are heard.
    /// Returns the gameplay commands the drops ask for.
    pub(crate) fn items_3d_frame(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        picked: Option<Option<u64>>,
    ) -> Vec<Command> {
        let mut commands = Vec::new();
        let Some(strings) = self.strings.clone() else {
            return commands;
        };
        let Some(items) = self.items_3d.as_mut() else {
            return commands;
        };
        let Some((tree, _)) = self.hud.as_mut() else {
            return commands;
        };
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        };
        let requests = items.frame_pass(tree, &player, picked, &self.player_messages);
        for request in requests {
            crate::ui_inventory::item_request(
                request,
                tree,
                self.stack.as_mut(),
                &player,
                &mut commands,
            );
        }
        commands
    }
}
