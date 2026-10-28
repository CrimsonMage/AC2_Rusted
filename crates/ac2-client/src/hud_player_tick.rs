//! The HUD frameworks' tick pass in the client: `ac2_ui::tick_pass` runs the panels' ticks, the
//! dialogs' outcomes and the calls the panels queued, and the requests it returns become gameplay
//! commands ([`hud_request_commands`]).

use ac2_ui::TickInputs;
use ac2_weenie::SessionEvent;

use crate::frame::ClientFrame;
use crate::hud_player::PassOut;
use crate::hud_player_message::{ClientOptions, ClientShortcuts};
use crate::hud_request::hud_request_commands;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

impl HudView {
    /// The panels' ticks and global loops after the messages, the dialogs' outcomes and the calls the
    /// panels queued (`gmSecureTradeUI`, `gmPetUI`, `gmShopUI`, `gmSkillTreeUI`, `gmShortcutUI`).
    pub(crate) fn tick_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        events: &[SessionEvent],
    ) -> PassOut {
        let HudView { hud, panels, .. } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let mut host = ClientOptions { player };
        let mut calls = ClientShortcuts { player };
        let inputs = TickInputs {
            server_time: frame.server_time,
            selection: frame.selection.as_ref().map(|s| s.id),
            options_host: &mut host,
            shortcut_calls: &mut calls,
        };
        let requests = ac2_ui::tick_pass(panels, tree, inputs, player, events);
        let mut out = PassOut::default();
        hud_request_commands(requests, &mut out);
        out
    }
}
