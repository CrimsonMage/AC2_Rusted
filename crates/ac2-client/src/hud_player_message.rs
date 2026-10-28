//! The HUD frameworks' message pass in the client: `ac2_ui::message_pass` runs the frame's
//! messages over the panels in the frameworks' order, and the requests it returns become gameplay
//! commands ([`hud_request_commands`]). The client options page is answered over the client here.

use ac2_render::prefs::SceneOptions;
use ac2_ui::{MessageInputs, OptionsHost, OptionsPrefs, Shortcut, ShortcutCalls, UiMessage};

use crate::frame::ClientFrame;
use crate::hud_friends::friend_target;
use crate::hud_player::{name_looks, PassOut};
use crate::hud_request::hud_request_commands;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

/// `gmClientUI`'s side over the player's client: the page's preferences and the live applies
/// need the client, which `ac2_ui::OptionsPages` reaches through [`OptionsHost`].
pub(crate) struct ClientOptions<'a, 'b> {
    pub(crate) player: &'a ClientPlayer<'b>,
}

impl OptionsHost for ClientOptions<'_, '_> {
    fn with_prefs(&mut self, f: &mut dyn FnMut(&mut dyn OptionsPrefs)) -> Option<()> {
        let mut client = self.player.client.borrow_mut();
        let prefs = client.prefs_mut()?;
        f(prefs.file_mut());
        Some(())
    }

    fn apply_live(&mut self) -> Option<SceneOptions> {
        self.player.client.borrow_mut().apply_live_preferences()
    }
}

/// The shortcut bar's calls on the player's client; a failed call is reported and dropped.
pub(crate) struct ClientShortcuts<'a, 'b> {
    pub(crate) player: &'a ClientPlayer<'b>,
}

impl ShortcutCalls for ClientShortcuts<'_, '_> {
    fn use_shortcut(&mut self, index: usize) {
        let result = self.player.client.borrow_mut().use_shortcut(index as u32);
        if let Err(error) = result.map(drop) {
            eprintln!("ac2: shortcut: {error}");
        }
    }

    fn set_shortcut(&mut self, index: usize, shortcut: &Shortcut) {
        let info = ac2_ui::shortcut_to_info(shortcut);
        let result = self
            .player
            .client
            .borrow_mut()
            .set_shortcut(index as u32, info);
        if let Err(error) = result.map(drop) {
            eprintln!("ac2: shortcut: {error}");
        }
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
        let HudView { hud, panels, .. } = self;
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
        let mut host = ClientOptions { player };
        let inputs = MessageInputs {
            server_time: frame.server_time,
            selection: frame.selection.as_ref().map(|s| s.id),
            friend_target: friend_target(frame.selection.as_ref()),
            name_looks: &looks,
            max_purchase: &mut most,
            consignment_charge: &mut charge,
            options_host: &mut host,
        };
        let requests = ac2_ui::message_pass(panels, tree, inputs, player, messages);
        let mut out = PassOut::default();
        hud_request_commands(requests, &mut out);
        out
    }
}
