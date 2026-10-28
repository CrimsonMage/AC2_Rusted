//! The HUD frameworks' notice pass (`RecvNotice_*`): `ac2_ui::notice_pass` takes the frame's
//! notices into the HUD's panels; it returns the friends requests the page raised, which the
//! client turns into its commands.

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
            hud, panels, dirty, ..
        } = self;
        let Some((tree, _)) = hud else {
            return PassOut::default();
        };
        let requests = ac2_ui::notice_pass(
            panels,
            tree,
            dirty,
            player,
            events,
            frame.server_time,
            reload_shortcuts,
        );
        PassOut {
            commands: requests.into_iter().map(Command::Friends).collect(),
            ..PassOut::default()
        }
    }
}
