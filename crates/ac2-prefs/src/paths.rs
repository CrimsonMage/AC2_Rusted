//! Where the preferences file lives. Retail's default is `Preferences.ini` in the working
//! directory (`PSUtils::full_path` is `_fullpath`), which is beside the game when it starts. The
//! port keeps the file in the platform's user configuration directory instead, so nothing is
//! written into the game's (or the DAT's) directory:
//!
//! - Windows: `%APPDATA%\AC2\Preferences.ini`
//! - macOS: `~/Library/Application Support/AC2/Preferences.ini`
//! - other: `$XDG_CONFIG_HOME/AC2/Preferences.ini`, or `~/.config/AC2/Preferences.ini`.
//!
//! The `--prefs` argument names another file: a relative one is taken against the working
//! directory, as retail's `full_path` does.

use std::io;
use std::path::{Path, PathBuf};

/// The file name retail uses (`Preferences.ini`).
pub const FILE_NAME: &str = "Preferences.ini";

/// The default preferences file in the user's configuration directory.
pub fn default_file() -> PathBuf {
    user_config_dir().join("AC2").join(FILE_NAME)
}

fn user_config_dir() -> PathBuf {
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata);
        }
    }
    if cfg!(target_os = "macos") {
        return home().join("Library").join("Application Support");
    }
    if !cfg!(windows) {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            let xdg = PathBuf::from(xdg);
            if xdg.is_absolute() {
                return xdg;
            }
        }
        return home().join(".config");
    }
    home()
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// `PSUtils::full_path`: a relative file name against the working directory.
pub fn resolve(name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_or_else(|_| path.to_path_buf(), |dir| dir.join(path))
    }
}

/// The file's bytes; an empty image when there is no file yet (retail reads nothing and falls
/// back to the defaults).
pub fn read_or_empty(path: &Path) -> io::Result<Vec<u8>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Writes `bytes` to `path` (creating its directory), through a temporary file renamed over it.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temp = path.with_extension("ini.tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path)
}
