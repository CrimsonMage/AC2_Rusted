use crate::DecodedTextureSurface;

/// The first render-codec route that selected the final texture payload.
///
/// This is read-only attribution, not texture-quality or residency policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureConversionRoute {
    GeneratedRgba,
    DirectNativeCompressed,
    DirectRgba,
    DirectScaledRgba,
    RenderTextureNativeCompressed,
    RenderTextureClipMapRgba,
    RenderTextureRgba,
    RenderTextureMixedRgba,
}

/// Provenance for generated RGBA pixels.
///
/// Retail `TexMerge` produces a custom image rather than an authored image texture. Keep that
/// fact separate from the conversion route so observation can distinguish its transient terrain
/// products from UI and other generated pixels without retaining a history of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneratedTextureOrigin {
    Other,
    TexMergeComposed,
    TexMergeReused,
}

/// POD source facts retained with a backend-neutral texture asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureSourceAttribution {
    source_pixel_format: u32,
    route: TextureConversionRoute,
    decoded_source_bytes: u64,
    generated_origin: GeneratedTextureOrigin,
    land_texture_scale: u8,
}

impl TextureSourceAttribution {
    pub const fn generated_rgba() -> Self {
        Self {
            source_pixel_format: 0,
            route: TextureConversionRoute::GeneratedRgba,
            decoded_source_bytes: 0,
            generated_origin: GeneratedTextureOrigin::Other,
            land_texture_scale: 0,
        }
    }

    /// Attributes a custom RGBA terrain image to retail's `TexMerge` path.
    pub const fn texmerge_rgba(
        generated_origin: GeneratedTextureOrigin,
        land_texture_scale: u8,
    ) -> Self {
        Self {
            source_pixel_format: 0,
            route: TextureConversionRoute::GeneratedRgba,
            decoded_source_bytes: 0,
            generated_origin,
            land_texture_scale,
        }
    }

    pub const fn source_pixel_format(self) -> u32 {
        self.source_pixel_format
    }

    pub const fn route(self) -> TextureConversionRoute {
        self.route
    }

    pub const fn decoded_source_bytes(self) -> u64 {
        self.decoded_source_bytes
    }

    pub const fn generated_origin(self) -> GeneratedTextureOrigin {
        self.generated_origin
    }

    /// Retail `ImgTex::fLandTextureScale` used while composing a TexMerge image.
    /// Zero is also the neutral value for non-TexMerge assets.
    pub const fn land_texture_scale(self) -> u8 {
        self.land_texture_scale
    }

    pub fn direct(surface: &DecodedTextureSurface, route: TextureConversionRoute) -> Self {
        Self {
            source_pixel_format: surface.pixel_format.0,
            route,
            decoded_source_bytes: source_bytes(surface),
            generated_origin: GeneratedTextureOrigin::Other,
            land_texture_scale: 0,
        }
    }

    pub fn render_texture<'a>(
        surfaces: impl Iterator<Item = &'a DecodedTextureSurface>,
        route: TextureConversionRoute,
    ) -> Self {
        let mut source_pixel_format = None;
        let mut mixed = false;
        let mut decoded_source_bytes = 0_u64;
        for surface in surfaces {
            let format = surface.pixel_format.0;
            if source_pixel_format.is_some_and(|first| first != format) {
                mixed = true;
            } else if source_pixel_format.is_none() {
                source_pixel_format = Some(format);
            }
            decoded_source_bytes = decoded_source_bytes.saturating_add(source_bytes(surface));
        }
        Self {
            source_pixel_format: if mixed {
                u32::MAX
            } else {
                source_pixel_format.unwrap_or(0)
            },
            route,
            decoded_source_bytes,
            generated_origin: GeneratedTextureOrigin::Other,
            land_texture_scale: 0,
        }
    }
}

fn source_bytes(surface: &DecodedTextureSurface) -> u64 {
    surface.compressed.as_ref().map_or_else(
        || surface.rgba.len() as u64,
        |compressed| compressed.blocks().len() as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::{GeneratedTextureOrigin, TextureConversionRoute, TextureSourceAttribution};

    #[test]
    fn texmerge_provenance_keeps_reuse_and_land_scale_without_source_pixels() {
        let attribution =
            TextureSourceAttribution::texmerge_rgba(GeneratedTextureOrigin::TexMergeReused, 1);

        assert_eq!(attribution.route(), TextureConversionRoute::GeneratedRgba);
        assert_eq!(
            attribution.generated_origin(),
            GeneratedTextureOrigin::TexMergeReused
        );
        assert_eq!(attribution.land_texture_scale(), 1);
        assert_eq!(attribution.decoded_source_bytes(), 0);
    }
}
