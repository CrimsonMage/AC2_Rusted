//! Retail `RenderTexture::DropUnwantedLevels`: how many of a `RenderTexture`'s source levels, largest
//! first, are left out by the Environment Texture Detail choice (`Render::m_Prefs.TextureDetail`),
//! and `RenderTexture::ShouldDropHighDetail`: the high-detail level, which lives in `highres.dat`,
//! is dropped at the top detail only while that file is not mounted.

use std::sync::atomic::Ordering;

use ac2_core::DataId;
use ac2_dat::DatFileKind;

use super::ResourceStore;

/// The highest `TextureDetail` choice (quality level 5's), which keeps every source level.
pub const MAX_TEXTURE_DETAIL: u8 = 4;

impl ResourceStore {
    /// Records the Environment Texture Detail choice (0 to 4; larger values are 4) for textures
    /// resolved from now on. Clones of this store share it. A texture keeps the levels it was
    /// built with, as retail's `DropUnwantedLevels` ran once per texture (`m_bDropLevelsCalled`), so the
    /// owner sets this before a world's textures are prepared.
    pub fn set_texture_detail(&self, detail: u8) {
        self.texture_detail
            .store(detail.min(MAX_TEXTURE_DETAIL), Ordering::Relaxed);
    }

    /// The Environment Texture Detail choice textures are resolved at.
    pub fn texture_detail(&self) -> u8 {
        self.texture_detail.load(Ordering::Relaxed)
    }

    /// `RenderTexture::DropUnwantedLevels`: of `count` source levels, largest first, how many leading
    /// levels are left out: 4, 3, 2 or 1 at detail 0 to 3, and at detail 4 one while
    /// `ShouldDropHighDetail` (no `highres.dat` mounted) else none; never the last level.
    pub fn dropped_texture_levels(&self, count: usize) -> usize {
        let drop = match self.texture_detail() {
            0 => 4,
            1 => 3,
            2 => 2,
            3 => 1,
            _ if self.dataset.file_meta(DatFileKind::HighRes).is_none() => 1,
            _ => 0,
        };
        if count > 1 {
            drop.min(count - 1)
        } else {
            0
        }
    }

    /// Whether the mounted `highres.dat` holds a SurfaceTexture's high-detail level,
    /// whatever the current detail choice.
    pub fn holds_high_detail_level(&self, did: DataId) -> bool {
        self.dataset.contains(DatFileKind::HighRes, did.raw())
    }

    /// The DAT that holds a SurfaceTexture source level: HighRes when the mounted overlay has the
    /// record, otherwise `fallback`.
    pub fn source_level_file_kind(&self, did: DataId, fallback: DatFileKind) -> DatFileKind {
        if self.dataset.contains(DatFileKind::HighRes, did.raw()) {
            DatFileKind::HighRes
        } else {
            fallback
        }
    }
}
