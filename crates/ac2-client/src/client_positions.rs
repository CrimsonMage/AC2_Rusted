//! The player's window positions, kept on this machine per character (the window exception in
//! `AGENTS.md`). The file IO and its format are `ac2_ui::window_positions`; this is the client's
//! wiring: the directory (the player's AC2 directory unless a test names another), the reads at
//! login over the server's saved locations (`client_locations`), and the writes when a drag ends.

use std::path::PathBuf;

use ac2_prefs::paths;
use ac2_ui::WindowPosition;

use crate::client::Client;

impl Client {
    /// Keeps the window position files in `dir` instead of the player's AC2 directory (`None`
    /// restores it). The client's test rigs use a temporary directory.
    pub fn set_window_positions_dir(&mut self, dir: Option<PathBuf>) {
        self.window_positions_dir = dir;
    }

    fn positions_dir(&self) -> PathBuf {
        self.window_positions_dir
            .clone()
            .unwrap_or_else(paths::config_dir)
    }

    /// `character`'s saved window positions; none when the player has not moved a window yet.
    pub fn local_window_positions(&self, character: u64) -> Result<Vec<WindowPosition>, String> {
        ac2_ui::read_window_positions(&self.positions_dir(), character)
    }

    /// Writes `character`'s window positions (the windows' places when a drag ended). A failure is
    /// reported on stderr; the window stays where the player left it for this session.
    pub(crate) fn save_local_window_positions(&self, character: u64, positions: &[WindowPosition]) {
        let dir = self.positions_dir();
        if let Err(error) = ac2_ui::write_window_positions(&dir, character, positions) {
            let path = ac2_ui::window_positions_file(&dir, character);
            eprintln!("ac2: window positions {}: {error}", path.display());
        }
    }
}
