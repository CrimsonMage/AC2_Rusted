use super::*;
use crate::{DecodedTextureSurface, TextureSourceKind};

/// Converts a decoded AC2 texture into a backend-neutral asset with its mip tail from `base`.
pub fn texture_asset_from_decoded_source(
    texture: &DecodedTextureAsset,
    base: u8,
    wrap: TextureWrapMode,
) -> Option<TextureAsset> {
    let mut asset = texture_asset_from_render_texture(
        texture,
        RenderTextureMipPolicy {
            environment_texture_detail: 0,
            high_detail_available: true,
        },
        wrap,
    )?;
    if texture.source_kind == TextureSourceKind::Generated {
        asset.source_attribution = TextureSourceAttribution::generated_rgba();
    }
    let mut asset = asset.mip_tail(base);
    asset.high_detail_level = decoded_high_detail_level(texture);
    Some(asset)
}

/// `Some(drawn)` when a source texture has a high-resolution DAT level: whether that level
/// supplied these texels.
fn decoded_high_detail_level(texture: &DecodedTextureAsset) -> Option<bool> {
    let surfaces = || {
        texture
            .levels
            .iter()
            .flat_map(|level| level.surfaces.iter())
    };
    surfaces()
        .any(|surface| {
            surface
                .resolved_identity
                .as_ref()
                .is_some_and(|identity| identity.high_detail_alternative)
        })
        .then(|| {
            surfaces().any(|surface| {
                surface.render_surface_file_kind == ac2_dat::DatFileKind::HighRes
            })
        })
}

impl TextureAsset {
    /// `Some(drawn)` when the texture has a high-resolution DAT level.
    pub const fn high_detail_level(&self) -> Option<bool> {
        self.high_detail_level
    }
}

pub(super) fn rgba_face(surface: &DecodedTextureSurface) -> Option<Arc<[u8]>> {
    if !surface.rgba.is_empty() {
        Some(Arc::clone(&surface.rgba))
    } else {
        surface
            .compressed
            .as_ref()?
            .decode_rgba(surface.width as usize, surface.height as usize)
            .map(Arc::from)
    }
}
