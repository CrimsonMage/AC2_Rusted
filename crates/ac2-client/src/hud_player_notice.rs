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
            panels: state,
            dialogs,
            dirty,
            item_selection,
            shortcuts,
            toolbar,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let panels = ac2_ui::NoticePanels {
            tree,
            now: state.now,
            dirty,
            item_selection,
            vitals: &mut state.vitals,
            effects: &mut state.effects,
            vitae: &mut state.vitae,
            net: &mut state.net,
            pk: &mut state.pk,
            stack: &mut state.stack,
            external_container: &mut state.external_container,
            secure_trade: &mut state.secure_trade,
            dialogs,
            shop: &mut state.shop,
            catalog: &mut state.catalog,
            consignment: &mut state.consignment,
            fellowship: &mut state.fellowship,
            mini_fellow: &mut state.mini_fellow,
            pets: &mut state.pets,
            friends: &mut state.friends,
            quests: &mut state.quests,
            allegiance: &mut state.allegiance,
            paper_doll: &mut state.paper_doll,
            backpack: &mut state.backpack,
            examination: &mut state.examination,
            skills: &mut state.skills,
            shortcuts,
            money: &mut state.money,
            toolbar,
            damage: &mut state.damage,
        };
        let requests =
            ac2_ui::notice_pass(panels, player, events, frame.server_time, reload_shortcuts);
        PassOut {
            commands: requests.into_iter().map(Command::Friends).collect(),
            ..PassOut::default()
        }
    }
}
