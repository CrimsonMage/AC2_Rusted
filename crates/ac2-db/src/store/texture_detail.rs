//! Retail `Render::ShouldDropHighDetail`: the high-detail source level of a two-level
//! SurfaceTexture lives in `highres.dat` and is used only while that file is mounted and
//! Environment Texture Detail is at its maximum.

use std::sync::atomic::Ordering;

use ac2_core::DataId;
use ac2_dat::DatFileKind;

use super::ResourceStore;

impl ResourceStore {
    /// Records the Environment Texture Detail choice for textures resolved from now on. Clones of
    /// this store share it. Resolved texture identities keep the level they were built from, so
    /// the owner sets this before a world's textures are prepared.
    pub fn set_high_detail_textures(&self, enabled: bool) {
        self.high_detail_textures.store(enabled, Ordering::Relaxed);
    }

    /// Retail `!Render::ShouldDropHighDetail()`.
    pub fn uses_high_detail_textures(&self) -> bool {
        self.high_detail_textures.load(Ordering::Relaxed)
            && self.dataset.file_meta(DatFileKind::HighRes).is_some()
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
