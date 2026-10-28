//! Where the help pages are read from: the DVD's `Content` folder.
//!
//! Retail: `KeyStone::Create` (Legions 0x004B22A0) hands `KeystoneCreate` the process's working
//! directory (`_getcwd`), and the help plugin (`plugins\AC2HelpPlugin.dll`, started by
//! `KeyStone::Init`) reads the `content/<page>.ksml` names from there. The install runs with the
//! Content folder beside its DATs. Here the folder is taken, in order, from `--content-dir`, then
//! the `Content` folder beside the DAT directory (matched without regard to case, `Content` on
//! the DVD, `content` elsewhere), then `AC2_CONTENT_DIR`. The first one that is a directory wins.
//! With none, the help window has no pages.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The environment variable that names the content directory when no option does.
pub const CONTENT_DIR_ENV: &str = "AC2_CONTENT_DIR";

static CONTENT_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Records the content directory the command line resolved. Only the first call counts.
pub fn set_content_dir(dir: Option<PathBuf>) {
    let _ = CONTENT_DIR.set(dir);
}

/// The content directory the HUD reads its help pages from: the one [`set_content_dir`]
/// recorded, else `AC2_CONTENT_DIR` (for a program that never resolved one).
pub fn content_dir() -> Option<PathBuf> {
    match CONTENT_DIR.get() {
        Some(dir) => dir.clone(),
        None => env_content_dir(std::env::var_os(CONTENT_DIR_ENV).as_deref()),
    }
}

/// The content directory for a client started with `explicit` (`--content-dir`), the DATs in
/// `dat_dir`, and `env` (`AC2_CONTENT_DIR`). `None` when no candidate is an existing directory.
pub fn resolve_content_dir(
    explicit: Option<&Path>,
    dat_dir: &Path,
    env: Option<&OsStr>,
) -> Option<PathBuf> {
    explicit
        .filter(|dir| dir.is_dir())
        .map(Path::to_path_buf)
        .or_else(|| content_beside(dat_dir))
        .or_else(|| env_content_dir(env))
}

/// The `AC2_CONTENT_DIR` value, when it names a directory.
fn env_content_dir(value: Option<&OsStr>) -> Option<PathBuf> {
    value
        .filter(|text| !text.is_empty())
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
}

/// The folder named `Content` (in any case) directly inside `dat_dir`. When several match, the
/// first by name.
pub fn content_beside(dat_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dat_dir).ok()?;
    let mut matches: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case("content"))
        })
        .collect();
    matches.sort();
    matches.into_iter().next()
}

#[cfg(test)]
#[path = "help_content_tests.rs"]
mod tests;
