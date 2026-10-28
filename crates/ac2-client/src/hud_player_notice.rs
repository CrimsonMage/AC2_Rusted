//! The HUD frameworks' notice pass (`RecvNotice_*`): the panels' notices live in
//! `ac2_ui::notice_pass`; this file hands it the HUD's panels and the player, and turns the
//! friends requests it raises into commands.

use ac2_weenie::SessionEvent;

use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_player::PassOut;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

impl HudView {
    /// The frame's notices, in the order the frameworks get them (`RecvNotice_*`), then the
    /// shortcut bar's checks (`gmShortcutUI`).
    pub(crate) fn notice_pass(
        &mut self,
        player: &ClientPlayer<'_>,
        frame: &ClientFrame,
        events: &[SessionEvent],
        reload_shortcuts: bool,
    ) -> PassOut {
        let HudView {
            hud,
            allegiance,
            backpack,
            catalog,
            consignment,
            panels: state,
            dialogs,
            dirty,
            effects,
            examination,
            external_container,
            fellowship,
            friends,
            item_selection,
            mini_fellow,
            money,
            net,
            paper_doll,
            pets,
            pk,
            quests,
            secure_trade,
            shop,
            shortcuts,
            skills,
            stack,
            toolbar,
            vitae,
            vitals,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let panels = ac2_ui::NoticePanels {
            tree,
            now: state.now(),
            dirty,
            item_selection,
            vitals,
            effects,
            vitae,
            net,
            pk,
            stack,
            external_container,
            secure_trade,
            dialogs,
            shop,
            catalog,
            consignment,
            fellowship,
            mini_fellow,
            pets,
            friends,
            quests,
            allegiance,
            paper_doll,
            backpack,
            examination,
            skills,
            shortcuts,
            money,
            toolbar,
            damage: state.damage_slot(),
        };
        let requests =
            ac2_ui::notice_pass(panels, player, events, frame.server_time, reload_shortcuts);
        PassOut {
            commands: requests.into_iter().map(Command::Friends).collect(),
            ..PassOut::default()
        }
    }
}
