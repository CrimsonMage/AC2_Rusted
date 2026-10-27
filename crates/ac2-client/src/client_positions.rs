//! The player's window positions, kept on this machine per character (the window exception in
//! `AGENTS.md`). Every HUD window's place is written when a drag ends, to one file per character
//! in the player's AC2 directory (`ac2_prefs::paths::config_dir`), and read back on the HUD's
//! login reload, over the server's saved locations (`client_locations`). The format is
//! `ac2_ui::window_positions`.

use std::path::{Path, PathBuf};

use ac2_prefs::paths;
use ac2_ui::WindowPosition;

use crate::client::Client;

/// The file of `character`'s positions in `dir`.
pub fn positions_file(dir: &Path, character: u64) -> PathBuf {
    dir.join(format!("WindowPositions-{character:016x}.txt"))
}

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
        let path = positions_file(&self.positions_dir(), character);
        let bytes = paths::read_or_empty(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if bytes.is_empty() {
            return Ok(Vec::new());
        }
        let text = String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        ac2_ui::parse_window_positions(&text)
    }

    /// Writes `character`'s window positions (the windows' places when a drag ended). A failure is
    /// reported on stderr; the window stays where the player left it for this session.
    pub(crate) fn save_local_window_positions(&self, character: u64, positions: &[WindowPosition]) {
        let path = positions_file(&self.positions_dir(), character);
        let text = ac2_ui::window_positions_text(positions);
        if let Err(error) = paths::write_atomic(&path, text.as_bytes()) {
            eprintln!("ac2: window positions {}: {error}", path.display());
        }
    }
}
