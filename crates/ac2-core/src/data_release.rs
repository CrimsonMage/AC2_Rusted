//! The DAT release the process reads. Retail built each client for one release; a few packed
//! layouts differ between the Legions DVD (7160) data and the 7230 data this client uses, so
//! the codecs for those layouts follow the release of the mounted DATs (`ac2_dat` sets it when
//! a DAT set is opened). A process that mounts two sets (`dat-diff`) reads with the last one's
//! layouts; the client mounts one.

use std::sync::atomic::{AtomicU8, Ordering};

/// A DAT release whose packed layouts the client reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRelease {
    /// The Legions DVD's data (portal iteration 4862) and the releases before 7230.
    Legions,
    /// The 7230 release (portal iteration 5064) and later.
    Release7230,
}

impl DataRelease {
    /// The first portal iteration of the 7230 release.
    pub const RELEASE_7230_ITERATION: i32 = 5064;

    /// The release of a portal DAT whose latest iteration is `iteration`. The 2013 relaunch's
    /// DATs restarted their iterations (137) and carry the 7230 layouts, so only the original
    /// retail line's iterations before 7230 are Legions.
    pub fn from_portal_iteration(iteration: i32) -> Self {
        if (1000..Self::RELEASE_7230_ITERATION).contains(&iteration) {
            Self::Legions
        } else {
            Self::Release7230
        }
    }
}

static RELEASE: AtomicU8 = AtomicU8::new(1);

/// The release the process reads; 7230 until a DAT set says otherwise.
pub fn data_release() -> DataRelease {
    match RELEASE.load(Ordering::Relaxed) {
        0 => DataRelease::Legions,
        _ => DataRelease::Release7230,
    }
}

/// Sets the release the process reads (the mounted DAT set's).
pub fn set_data_release(release: DataRelease) {
    RELEASE.store(
        u8::from(release == DataRelease::Release7230),
        Ordering::Relaxed,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portal_iteration_picks_the_release() {
        assert_eq!(
            DataRelease::from_portal_iteration(4862),
            DataRelease::Legions
        );
        assert_eq!(
            DataRelease::from_portal_iteration(5064),
            DataRelease::Release7230
        );
        assert_eq!(
            DataRelease::from_portal_iteration(137),
            DataRelease::Release7230
        );
    }
}
