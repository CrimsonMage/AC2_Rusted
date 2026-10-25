//! The General MIDI set, from a file the user supplies. Nothing GM is shipped or committed: a
//! band instrument with no DLS collection plays from this file only, and stays silent without
//! one. The path comes from the `AC2_GM_DLS` environment variable (or the caller's own
//! configuration). [`load_gm_file`] reads the file: call it on a worker thread, never on the
//! frame thread.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

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

/// A GM file loading on its own worker thread, until its result is taken.
#[derive(Default)]
pub struct GmLoad {
    receiver: Option<Receiver<Result<Arc<Collection>, GmLoadError>>>,
}

impl GmLoad {
    /// Starts loading `path` on a worker (the caller's thread never reads the file). A failure to
    /// start the thread is logged, and nothing loads.
    pub fn spawn(&mut self, path: PathBuf) {
        let (sender, receiver) = channel();
        let spawned = std::thread::Builder::new()
            .name("ac2-audio-gm".into())
            .spawn(move || {
                let _ = sender.send(load_gm_file(&path));
            });
        match spawned {
            Ok(_) => self.receiver = Some(receiver),
            Err(error) => log::warn!("no General MIDI set: {error}"),
        }
    }

    /// Whether a load is under way (its result not yet taken).
    pub fn is_pending(&self) -> bool {
        self.receiver.is_some()
    }

    /// The result, once the worker has finished (`None` while it loads or when none is loading).
    pub fn poll(&mut self) -> Option<Result<Arc<Collection>, GmLoadError>> {
        let arrived = match self.receiver.as_ref().map(Receiver::try_recv) {
            None => return None,
            Some(Ok(result)) => Some(result),
            Some(Err(TryRecvError::Empty)) => None,
            Some(Err(TryRecvError::Disconnected)) => {
                self.receiver = None;
                return None;
            }
        };
        if arrived.is_some() {
            self.receiver = None;
        }
        arrived
    }

    /// Waits up to `timeout` for the result (headless runs and tests). `None` on a timeout, which
    /// leaves the load pending.
    pub fn wait(&mut self, timeout: Duration) -> Option<Result<Arc<Collection>, GmLoadError>> {
        let receiver = self.receiver.take()?;
        match receiver.recv_timeout(timeout) {
            Ok(result) => Some(result),
            Err(RecvTimeoutError::Timeout) => {
                self.receiver = Some(receiver);
                None
            }
            Err(RecvTimeoutError::Disconnected) => None,
        }
    }
}
