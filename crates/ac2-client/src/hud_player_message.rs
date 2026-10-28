//! The client's sides of the frame's option and shortcut calls: `ac2_ui::Hud::frame` takes them
//! as [`ac2_ui::FrameHosts`] (the options page's `OptionsHost`, the bar's `ShortcutCalls`), and
//! they answer over the player's client here.

use ac2_render::prefs::SceneOptions;
use ac2_ui::{OptionsHost, OptionsPrefs, Shortcut, ShortcutCalls};

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
