//! The General MIDI set, from a file the user supplies. Nothing GM is shipped or committed: a
//! band instrument with no DLS collection plays from this file only, and stays silent without
//! one. The path comes from the `AC2_GM_DLS` environment variable (or the caller's own
//! configuration). [`load_gm_file`] reads the file: call it on a worker thread, never on the
//! frame thread.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::music::dls::{read_collection, Collection};
use crate::music::riff::MusicFormatError;

/// The environment variable that names the user's `gm.dls`.
pub const GM_ENV: &str = "AC2_GM_DLS";

/// The GM file the environment names, when it is set.
pub fn gm_path_from_env() -> Option<PathBuf> {
    std::env::var_os(GM_ENV)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Why a GM file did not load.
#[derive(Debug)]
pub enum GmLoadError {
    Io(std::io::Error),
    Format(MusicFormatError),
}

impl std::fmt::Display for GmLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GmLoadError::Io(e) => write!(f, "reading the GM file: {e}"),
            GmLoadError::Format(e) => write!(f, "the GM file: {e}"),
        }
    }
}

impl std::error::Error for GmLoadError {}

/// Reads a DLS collection (`RIFF 'DLS '`) from `path`, to be given to
/// [`Synth::set_gm`](super::Synth::set_gm).
pub fn load_gm_file(path: &Path) -> Result<Arc<Collection>, GmLoadError> {
    let bytes = std::fs::read(path).map_err(GmLoadError::Io)?;
    read_collection(&bytes)
        .map(Arc::new)
        .map_err(GmLoadError::Format)
}
