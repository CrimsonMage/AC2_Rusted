//! The player's window positions, kept on this machine per character (the window exception in
//! `AGENTS.md`). The file IO and its format are `ac2_ui::window_positions`; this is the client's
//! wiring: the directory (the player's AC2 directory unless a test names another), the reads at
//! login over the server's saved locations (`client_locations`), and the writes when a drag ends.
//! The radar's coordinates box is kept here too: shown unless the player hid it in this client
//! (`AGENTS.md`).

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

    /// `character`'s saved window positions for the HUD's `style`; none when the player has not
    /// moved a window yet.
    pub fn local_window_positions(
        &self,
        character: u64,
        style: ac2_ui::HudStyle,
    ) -> Result<Vec<WindowPosition>, String> {
        ac2_ui::read_window_positions(&self.positions_dir(), character, style)
    }

    /// Writes `character`'s window positions (the windows' places when a drag ended). A failure is
    /// reported on stderr; the window stays where the player left it for this session.
    pub(crate) fn save_local_window_positions(
        &self,
        character: u64,
        style: ac2_ui::HudStyle,
        positions: &[WindowPosition],
    ) {
        let dir = self.positions_dir();
        if let Err(error) = ac2_ui::write_window_positions(&dir, character, style, positions) {
            let path = ac2_ui::window_positions_file(&dir, character, style);
            eprintln!("ac2: window positions {}: {error}", path.display());
        }
    }

    /// Whether the radar shows its coordinates box (`gmRadarUI::ReloadOptions`' argument). Retail
    /// passed the profile's `PROFILE_SHOW_RADAR_COORDS` bit; here the box is also open while the
    /// player has not hidden it in this client, so a new character sees its coordinates. A change
    /// of the bit seen here (the radar's button or the options page) records the choice locally.
    pub fn radar_coords_shown(&mut self) -> bool {
        let profile = self.show_radar_coords();
        let Some(character) = self.session().player() else {
            return profile;
        };
        let hidden = match self.radar_coords {
            Some((who, last, hidden)) if who == character => {
                if last == profile {
                    hidden
                } else {
                    self.save_radar_coords_hidden(character, !profile);
                    !profile
                }
            }
            _ => ac2_ui::read_radar_coords_hidden(&self.positions_dir(), character),
        };
        self.radar_coords = Some((character, profile, hidden));
        profile || !hidden
    }

    /// The radar's coordinates button (`gmRadarUI::ListenToMessage`) set the box to `show`: kept
    /// locally even when the profile's bit does not change (it is off on a new character).
    pub(crate) fn record_radar_coords_choice(&mut self, show: bool) {
        let Some(character) = self.session().player() else {
            return;
        };
        self.save_radar_coords_hidden(character, !show);
        let profile = match self.radar_coords {
            Some((who, last, _)) if who == character => last,
            _ => self.show_radar_coords(),
        };
        self.radar_coords = Some((character, profile, !show));
    }

    fn save_radar_coords_hidden(&self, character: u64, hidden: bool) {
        let dir = self.positions_dir();
        if let Err(error) = ac2_ui::write_radar_coords_hidden(&dir, character, hidden) {
            let path = ac2_ui::radar_coords_hidden_file(&dir, character);
            eprintln!("ac2: radar coordinates {}: {error}", path.display());
        }
    }
}
