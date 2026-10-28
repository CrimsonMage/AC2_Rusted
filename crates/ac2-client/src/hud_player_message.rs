//! The HUD frameworks' message pass in the client: `ac2_ui::message_pass` runs the frame's
//! messages over the panels in the frameworks' order, and the requests it returns become gameplay
//! commands ([`hud_request_commands`]). The client options page is answered over the client here.

use std::sync::Arc;

use ac2_ui::{DialogFactory, LayoutDesc, MessagePanels, OptionsMessages, UiMessage, UiTree};

use crate::frame::ClientFrame;
use crate::hud_friends::friend_target;
use crate::hud_options::OptionsPages;
use crate::hud_player::{name_looks, PassOut};
use crate::hud_request::hud_request_commands;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

/// `gmClientUI`'s page over the player's client: the page's widgets and its messages need the
/// client's preferences.
struct ClientOptions<'a, 'b> {
    pages: &'a mut OptionsPages,
    player: &'a ClientPlayer<'b>,
}

impl OptionsMessages for ClientOptions<'_, '_> {
    fn make(&mut self, tree: &mut UiTree, toolbar: Option<&Arc<LayoutDesc>>) {
        self.pages.make(&self.player.client.borrow(), tree, toolbar);
    }

    fn message(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, message: &UiMessage) {
        self.pages
            .message(&mut self.player.client.borrow_mut(), tree, dialogs, message);
    }
}

impl HudView {
    /// The quests' global loop, then the frame's messages to the panels in the frameworks' order
    /// (`ListenToMessage`), as the gameplay commands the panels asked for.
    pub(crate) fn message_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        messages: &[UiMessage],
    ) -> PassOut {
        let HudView {
            hud,
            allegiance,
            assistance,
            backpack,
            catalog,
            character,
            chat_options,
            consignment,
            dialogs,
            effects,
            external_container,
            fellowship,
            friends,
            game_dialogs,
            items_3d,
            mini_fellow,
            money,
            names,
            now,
            new_sale,
            options,
            overview,
            paper_doll,
            pets,
            quests,
            radar,
            secure_trade,
            shift_held,
            shop,
            shortcuts,
            skills,
            social,
            stack,
            toolbar,
            toolbar_layout,
            vitae,
            vitals,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let looks = || name_looks(frame);
        let mut most = |item| {
            player
                .client
                .borrow_mut()
                .max_purchase_quantity(item)
                .unwrap_or(0)
        };
        let mut charge = || {
            player
                .client
                .borrow_mut()
                .consignment_charge()
                .unwrap_or(0.0)
        };
        let mut page = ClientOptions {
            pages: options,
            player,
        };
        let panels = MessagePanels {
            tree,
            now: *now,
            server_time: frame.server_time,
            shift_held: *shift_held,
            selection: frame.selection.as_ref().map(|s| s.id),
            friend_target: friend_target(frame.selection.as_ref()),
            name_looks: &looks,
            max_purchase: &mut most,
            consignment_charge: &mut charge,
            options: &mut page,
            toolbar_layout: toolbar_layout.as_ref(),
            game_dialogs,
            dialogs,
            toolbar,
            shortcuts,
            skills,
            overview,
            fellowship,
            mini_fellow,
            pets,
            friends,
            social,
            assistance,
            quests,
            vitals,
            vitae,
            radar,
            names,
            effects,
            backpack,
            money,
            items_3d,
            external_container,
            stack,
            secure_trade,
            shop,
            catalog,
            new_sale,
            consignment,
            paper_doll,
            allegiance,
            character,
            chat_options,
        };
        let requests = ac2_ui::message_pass(panels, player, messages);
        let mut out = PassOut::default();
        hud_request_commands(requests, &mut out);
        out
    }
}
