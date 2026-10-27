//! One client per user: `Turbine::Client::IsAlreadyRunning` (Legions 0x006A0A60) creates the
//! named semaphore "Turbine Client" and, when it already exists (`ERROR_ALREADY_EXISTS`), shows
//! the error dialog and stops the start-up. Here an exclusive lock on `Turbine Client.lock` in
//! the AC2 configuration directory does the same job; the lock is released when the process
//! exits, so a crashed client never blocks the next start.

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;

/// The lock's file name in the configuration directory.
const LOCK_FILE: &str = "Turbine Client.lock";

/// The message the start-up stops with when another client holds the lock.
pub(crate) const ALREADY_RUNNING: &str = "the Asheron's Call 2 client is already running";

/// Claims the single-client lock. The returned file holds it; drop it to release it.
pub(crate) fn claim() -> Result<File, String> {
    claim_at(&ac2_prefs::paths::config_file(LOCK_FILE))
}

fn claim_at(path: &std::path::Path) -> Result<File, String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("lock file: {e}"))?;
    }
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)
        .map_err(|e| format!("lock file: {e}"))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err(ALREADY_RUNNING.into()),
        Err(std::fs::TryLockError::Error(e)) if e.kind() == ErrorKind::Unsupported => Ok(file),
        Err(std::fs::TryLockError::Error(e)) => Err(format!("lock file: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_claim_is_refused_until_the_first_is_dropped() {
        let dir = std::env::temp_dir().join(format!("ac2-instance-{}", std::process::id()));
        let path = dir.join(LOCK_FILE);
        let first = claim_at(&path).unwrap();
        assert_eq!(claim_at(&path).err().as_deref(), Some(ALREADY_RUNNING));
        drop(first);
        assert!(claim_at(&path).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }
}
