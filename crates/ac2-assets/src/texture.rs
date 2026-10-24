use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_core::PixelFormatID;
use ac2_core::DataId;

use crate::BlockCompressedTexture;

/// Source-state identity of one resolved surface image (not a hash of decoded pixels).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedSurfaceIdentity {
    pub authored_surface_did: DataId,
    pub requested_texture_did: DataId,
    pub render_surface_file_kind: DatFileKind,
    pub render_surface_did: DataId,
    pub palette_did: DataId,
    pub subpalettes: Arc<[[u32; 3]]>,
    pub clip_map_transform: bool,
    /// The source texture also has a high-resolution DAT level.
    pub high_detail_alternative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTextureSurface {
    pub authored_face_index: usize,
    pub render_surface_file_kind: DatFileKind,
    pub render_surface_did: DataId,
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormatID,
    pub rgba: Arc<[u8]>,
    /// A clip-map alpha operation changed texels before upload.
    pub clip_map_transform: bool,
    /// Present only for static surfaces that can remain as native DXT/BC blocks through upload.
    pub compressed: Option<BlockCompressedTexture>,
    /// Source state that selected these texels.
    pub resolved_identity: Option<ResolvedSurfaceIdentity>,
}

impl DecodedTextureSurface {
    /// Clip-map alpha testing uses the lower palettized reference for these formats.
    pub fn uses_palette_alpha_test_reference(&self) -> bool {
        matches!(
            self.pixel_format,
            PixelFormatID::PFID_P8 | PixelFormatID::PFID_INDEX16
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTextureLevel {
    pub source_level_did: DataId,
    pub surfaces: Arc<[DecodedTextureSurface]>,
}

/// Source conversion family selected by typed resource resolution, not appearance identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureSourceKind {
    DirectImgTex,
    AuthoredRenderTexture,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTextureAsset {
    pub source_kind: TextureSourceKind,
    pub texture_did: DataId,
    pub shape: DecodedTextureShape,
    /// Retained strictly for diagnostics of unknown texture types.
    pub raw_texture_type: u8,
    pub diagnostic_fallback: bool,
    pub levels: Arc<[DecodedTextureLevel]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedTextureShape {
    Texture2d,
    Cube,
}

impl DecodedTextureAsset {
    pub const fn is_diagnostic_fallback(&self) -> bool {
        self.diagnostic_fallback
    }
}
