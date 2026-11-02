//! The DAT release the process reads. Retail built each client for one release; a few packed
//! layouts differ between the Legions DVD (7160) data and the 7230 and end-of-retail (client
//! build 0930, `eor2017`) data, so the codecs for those layouts follow the release of the
//! mounted DATs (`ac2_dat` sets it when a DAT set is opened). A process that mounts two sets
//! (`dat-diff`) reads with the last one's layouts; the client mounts one.

use std::sync::atomic::{AtomicU8, Ordering};

/// A DAT release whose packed layouts the client reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRelease {
    /// The Legions DVD's data (portal iteration 4862) and the releases before 7230.
    Legions,
    /// The 7230 release (portal iteration 5064).
    Release7230,
    /// The end-of-retail data of client build 0930 (2013-10-01; portal iteration 137, the
    /// relaunch's restarted numbering).
    ///
    /// Checked against the 0930 executable, every layout the switch covers reads as 7230's:
    /// `SkillUINode::StreamPack` (0930 0x005D5780) packs the perk word last, at +0xA0, and
    /// `SkillUINode::Clear` (0x005D5850) zeroes it, as 7230's does; `gmCharacterUI::Setup`
    /// (0x004703F0) makes the same 24 options (the classic examination option's strings are
    /// in the executable). It differs in the logon: `ClientNet::SendLoginRequest` (0x004D3B70)
    /// packs the data version `"1802"`, otherwise as Legions' (`CLogonHeader::CreateFromData`
    /// 0x00612430, `NetAuthenticator::StreamPack` 0x004DA1B0 and `SetToAuthType` 0x004DA140
    /// compare equal to Legions' 0x005F0E90, 0x004B95C0 and 0x004B9550).
    Release0930,
}

impl DataRelease {
    /// The first portal iteration of the 7230 release.
    pub const RELEASE_7230_ITERATION: i32 = 5064;

    /// The release of a portal DAT whose latest iteration is `iteration`. The original retail
    /// line's iterations before 7230 are Legions; 5064 and later are 7230; the 2013 relaunch's
    /// DATs restarted their iterations (137), so anything below 1000 is the end-of-retail set.
    pub fn from_portal_iteration(iteration: i32) -> Self {
        if iteration < 1000 {
            Self::Release0930
        } else if iteration < Self::RELEASE_7230_ITERATION {
            Self::Legions
        } else {
            Self::Release7230
        }
    }

    /// Whether the packed layouts are the 7230 ones (7230 and 0930), not the DVD's.
    pub fn reads_7230_layouts(self) -> bool {
        self != Self::Legions
    }
}

static RELEASE: AtomicU8 = AtomicU8::new(1);

/// The release the process reads; 7230 until a DAT set says otherwise.
pub fn data_release() -> DataRelease {
    match RELEASE.load(Ordering::Relaxed) {
        0 => DataRelease::Legions,
        1 => DataRelease::Release7230,
        _ => DataRelease::Release0930,
    }
}

/// Sets the release the process reads (the mounted DAT set's).
pub fn set_data_release(release: DataRelease) {
    RELEASE.store(
        match release {
            DataRelease::Legions => 0,
            DataRelease::Release7230 => 1,
            DataRelease::Release0930 => 2,
        },
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
            DataRelease::Release0930
        );
        assert!(DataRelease::Release0930.reads_7230_layouts());
        assert!(DataRelease::Release7230.reads_7230_layouts());
        assert!(!DataRelease::Legions.reads_7230_layouts());
    }
}
