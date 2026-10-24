use std::sync::Arc;

mod direct;
mod mip_tail;
use direct::rgba_face;
pub use direct::texture_asset_from_decoded_source;

use crate::BlockCompressedTextureFormat;

use crate::{
    DecodedTextureAsset, DecodedTextureShape, TextureConversionRoute, TextureSourceAttribution,
};

/// Backend-neutral texture topology.  Cubes use the retail authored face ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureShape {
    Texture2d,
    Cube,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureWrapMode {
    Clamp,
    Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderTextureMipPolicy {
    pub environment_texture_detail: u8,
    pub high_detail_available: bool,
}

impl Default for RenderTextureMipPolicy {
    fn default() -> Self {
        Self {
            environment_texture_detail: 0,
            high_detail_available: true,
        }
    }
}

impl RenderTextureMipPolicy {
    pub fn leading_levels_to_drop(self, level_count: usize) -> usize {
        let requested = match self.environment_texture_detail.min(4) {
            0 if self.high_detail_available => 0,
            0 => 1,
            detail => detail as usize,
        };
        requested.min(level_count.saturating_sub(1))
    }
}

#[derive(Debug, Clone)]
pub struct TextureAsset {
    pub shape: TextureShape,
    pub width: u32,
    pub height: u32,
    payload: TexturePayload,
    source_base_mip: u8,
    // Flattened mip-major, face-major SHA-256 identities. Clones share these;
    // payload access is immutable so a digest can never outlive its content.
    content_fingerprints: Arc<[[u8; 32]]>,
    pub wrap_mode: TextureWrapMode,
    source_attribution: TextureSourceAttribution,
    decoded_levels: Option<Arc<[crate::DecodedTextureLevel]>>,
    high_detail_level: Option<bool>,
    water_coverage: Option<Arc<[u8]>>,
    water_palette: Option<Arc<[u8]>>,
    ice_coverage: Option<Arc<[u8]>>,
    vegetation_coverage: Option<Arc<[u8]>>,
}

// Decode-lifetime sharing is incidental; equality describes the original render product.
impl PartialEq for TextureAsset {
    fn eq(&self, other: &Self) -> bool {
        self.source_base_mip == other.source_base_mip
            && self.shape == other.shape
            && self.width == other.width
            && self.height == other.height
            && self.payload == other.payload
            && self.wrap_mode == other.wrap_mode
            && self.source_attribution == other.source_attribution
            && self.water_coverage == other.water_coverage
            && self.water_palette == other.water_palette
            && self.ice_coverage == other.ice_coverage
            && self.vegetation_coverage == other.vegetation_coverage
    }
}
impl Eq for TextureAsset {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexturePayload {
    Rgba(RgbaTextureLevels),
    BlockCompressed(CompressedTextureLevels),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaTextureLevels {
    pub levels: Arc<[RgbaTextureLevel]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaTextureLevel {
    pub width: u32,
    pub height: u32,
    pub faces: Arc<[Arc<[u8]>]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedTextureLevels {
    pub format: BlockCompressedTextureFormat,
    pub levels: Arc<[CompressedTextureLevel]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedTextureLevel {
    pub width: u32,
    pub height: u32,
    pub faces: Arc<[Arc<[u8]>]>,
}

impl TextureAsset {
    pub fn payload(&self) -> &TexturePayload {
        &self.payload
    }

    /// How many clones of this asset exist (they share one digest allocation); a cache that
    /// holds the only one can drop it.
    pub fn clone_count(&self) -> usize {
        Arc::strong_count(&self.content_fingerprints)
    }

    /// Content digests in mip-major, face-major order, prepared exactly once.
    /// No GPU state or allocation-address identity crosses the codec boundary.
    pub fn content_fingerprints(&self) -> &[[u8; 32]] {
        &self.content_fingerprints
    }

    pub fn vegetation_coverage(&self) -> Option<&Arc<[u8]>> {
        self.vegetation_coverage.as_ref()
    }
    pub fn with_vegetation_coverage(mut self, mask: Option<Arc<[u8]>>) -> Self {
        self.vegetation_coverage = mask.filter(|m| m.len() == 128 * 128);
        self
    }
    /// Separate TexMerge authored-water coverage, never the source texture's alpha channel.
    pub fn water_coverage(&self) -> Option<&Arc<[u8]>> {
        self.water_coverage.as_ref()
    }
    pub fn with_water_coverage(mut self, mask: Option<Arc<[u8]>>) -> Self {
        self.water_coverage = mask.filter(|m| m.len() == 128 * 128);
        self
    }
    pub fn water_palette(&self) -> Option<&Arc<[u8]>> {
        self.water_palette.as_ref()
    }
    pub fn with_water_palette(mut self, palette: Option<Arc<[u8]>>) -> Self {
        self.water_palette = palette.filter(|p| p.len() == 8 * 8 * 4);
        self
    }
    pub fn ice_coverage(&self) -> Option<&Arc<[u8]>> {
        self.ice_coverage.as_ref()
    }
    pub fn with_ice_coverage(mut self, mask: Option<Arc<[u8]>>) -> Self {
        self.ice_coverage = mask.filter(|m| m.len() == 128 * 128);
        self
    }
    pub fn rgba_2d(
        width: u32,
        height: u32,
        levels: Arc<[RgbaTextureLevel]>,
        wrap_mode: TextureWrapMode,
    ) -> Option<Self> {
        Self::new(
            TextureShape::Texture2d,
            width,
            height,
            TexturePayload::Rgba(RgbaTextureLevels { levels }),
            wrap_mode,
        )
    }
    pub fn rgba_cube(
        width: u32,
        height: u32,
        levels: Arc<[RgbaTextureLevel]>,
        wrap_mode: TextureWrapMode,
    ) -> Option<Self> {
        Self::new(
            TextureShape::Cube,
            width,
            height,
            TexturePayload::Rgba(RgbaTextureLevels { levels }),
            wrap_mode,
        )
    }
    pub fn bc_2d(
        width: u32,
        height: u32,
        format: BlockCompressedTextureFormat,
        levels: Arc<[CompressedTextureLevel]>,
        wrap_mode: TextureWrapMode,
    ) -> Option<Self> {
        Self::new(
            TextureShape::Texture2d,
            width,
            height,
            TexturePayload::BlockCompressed(CompressedTextureLevels { format, levels }),
            wrap_mode,
        )
    }
    pub fn bc_cube(
        width: u32,
        height: u32,
        format: BlockCompressedTextureFormat,
        levels: Arc<[CompressedTextureLevel]>,
        wrap_mode: TextureWrapMode,
    ) -> Option<Self> {
        Self::new(
            TextureShape::Cube,
            width,
            height,
            TexturePayload::BlockCompressed(CompressedTextureLevels { format, levels }),
            wrap_mode,
        )
    }

    fn new(
        shape: TextureShape,
        width: u32,
        height: u32,
        payload: TexturePayload,
        wrap_mode: TextureWrapMode,
    ) -> Option<Self> {
        if !validate_payload(shape, width, height, &payload) {
            return None;
        }
        Some(Self {
            shape,
            width,
            height,
            source_base_mip: 0,
            content_fingerprints: prepare_content_fingerprints(&payload),
            payload,
            wrap_mode,
            source_attribution: TextureSourceAttribution::generated_rgba(),
            decoded_levels: None,
            high_detail_level: None,
            water_coverage: None,
            water_palette: None,
            ice_coverage: None,
            vegetation_coverage: None,
        })
    }

    pub fn clamped(width: u32, height: u32, rgba: Arc<[u8]>) -> Self {
        Self::try_clamped(width, height, rgba).expect("valid caller RGBA texture")
    }

    /// Builds a single-level clamped RGBA texture without assuming that an external
    /// raster source satisfied the texture contract.
    pub fn try_clamped(width: u32, height: u32, rgba: Arc<[u8]>) -> Option<Self> {
        Self::rgba_2d(
            width,
            height,
            Arc::from([RgbaTextureLevel {
                width,
                height,
                faces: Arc::from([rgba]),
            }]),
            TextureWrapMode::Clamp,
        )
    }
    pub fn repeating(width: u32, height: u32, rgba: Arc<[u8]>) -> Self {
        Self::try_repeating(width, height, rgba).expect("valid caller RGBA texture")
    }

    /// Builds a single-level repeating RGBA texture from already-decoded
    /// pixels. Callers that consume external decoded media must handle `None`
    /// instead of assuming the width/height/payload contract holds.
    pub fn try_repeating(width: u32, height: u32, rgba: Arc<[u8]>) -> Option<Self> {
        Self::rgba_2d(
            width,
            height,
            Arc::from([RgbaTextureLevel {
                width,
                height,
                faces: Arc::from([rgba]),
            }]),
            TextureWrapMode::Repeat,
        )
    }
    pub fn level_count(&self) -> usize {
        match &self.payload {
            TexturePayload::Rgba(levels) => levels.levels.len(),
            TexturePayload::BlockCompressed(levels) => levels.levels.len(),
        }
    }
    pub fn payload_byte_len(&self) -> usize {
        match &self.payload {
            TexturePayload::Rgba(levels) => levels
                .levels
                .iter()
                .flat_map(|level| level.faces.iter())
                .map(|face| face.len())
                .sum(),
            TexturePayload::BlockCompressed(levels) => levels
                .levels
                .iter()
                .flat_map(|level| level.faces.iter())
                .map(|face| face.len())
                .sum(),
        }
    }
    pub const fn source_attribution(&self) -> TextureSourceAttribution {
        self.source_attribution
    }
    pub fn with_source_attribution(mut self, source_attribution: TextureSourceAttribution) -> Self {
        self.source_attribution = source_attribution;
        self
    }
    pub fn base_rgba(&self) -> Option<&Arc<[u8]>> {
        match &self.payload {
            TexturePayload::Rgba(levels) => levels.levels.first()?.faces.first(),
            TexturePayload::BlockCompressed(_) => None,
        }
    }
}

fn prepare_content_fingerprints(payload: &TexturePayload) -> Arc<[[u8; 32]]> {
    use sha2::{Digest, Sha256};
    match payload {
        TexturePayload::Rgba(levels) => levels
            .levels
            .iter()
            .flat_map(|level| level.faces.iter())
            .map(|face| Sha256::digest(face.as_ref()).into())
            .collect(),
        TexturePayload::BlockCompressed(levels) => levels
            .levels
            .iter()
            .flat_map(|level| level.faces.iter())
            .map(|face| Sha256::digest(face.as_ref()).into())
            .collect(),
    }
}

fn validate_payload(
    shape: TextureShape,
    width: u32,
    height: u32,
    payload: &TexturePayload,
) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let faces = if shape == TextureShape::Cube { 6 } else { 1 };
    match payload {
        TexturePayload::Rgba(payload) => {
            validate_levels(width, height, faces, &payload.levels, |level| {
                level.faces.iter().all(|face| {
                    usize::try_from(level.width)
                        .ok()
                        .and_then(|w| {
                            usize::try_from(level.height)
                                .ok()
                                .and_then(|h| w.checked_mul(h)?.checked_mul(4))
                        })
                        .is_some_and(|len| face.len() == len)
                })
            })
        }
        TexturePayload::BlockCompressed(payload) => {
            validate_levels(width, height, faces, &payload.levels, |level| {
                let bytes = block_bytes(payload.format);
                level.faces.iter().all(|face| {
                    usize::try_from(level.width.div_ceil(4))
                        .ok()
                        .and_then(|w| {
                            usize::try_from(level.height.div_ceil(4))
                                .ok()
                                .and_then(|h| w.checked_mul(h)?.checked_mul(bytes))
                        })
                        .is_some_and(|len| face.len() == len)
                })
            })
        }
    }
}

trait LevelDimensions {
    fn dimensions(&self) -> (u32, u32);
    fn face_len(&self) -> usize;
}
impl LevelDimensions for RgbaTextureLevel {
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn face_len(&self) -> usize {
        self.faces.len()
    }
}
impl LevelDimensions for CompressedTextureLevel {
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn face_len(&self) -> usize {
        self.faces.len()
    }
}
fn validate_levels<T: LevelDimensions>(
    width: u32,
    height: u32,
    faces: usize,
    levels: &[T],
    valid_faces: impl Fn(&T) -> bool,
) -> bool {
    let Some(first) = levels.first() else {
        return false;
    };
    if first.dimensions() != (width, height) {
        return false;
    }
    let mut previous = (width, height);
    levels.iter().all(|level| {
        let expected = previous;
        let dimensions = level.dimensions();
        previous = ((expected.0 / 2).max(1), (expected.1 / 2).max(1));
        dimensions == expected && level.face_len() == faces && valid_faces(level)
    })
}

fn block_bytes(format: BlockCompressedTextureFormat) -> usize {
    match format {
        BlockCompressedTextureFormat::Bc1Rgba => 8,
        BlockCompressedTextureFormat::Bc2Rgba | BlockCompressedTextureFormat::Bc3Rgba => 16,
    }
}

pub fn texture_asset_from_render_texture(
    texture: &DecodedTextureAsset,
    mip_policy: RenderTextureMipPolicy,
    wrap_mode: TextureWrapMode,
) -> Option<TextureAsset> {
    let mut asset = convert_render_texture(texture, mip_policy, wrap_mode)?;
    asset.source_base_mip = mip_policy.leading_levels_to_drop(texture.levels.len()) as u8;
    if mip_policy.leading_levels_to_drop(texture.levels.len()) == 0
        && shares_decoded_pixels(&asset, texture)
    {
        asset.decoded_levels = Some(Arc::clone(&texture.levels));
    }
    Some(asset)
}

fn shares_decoded_pixels(asset: &TextureAsset, decoded: &DecodedTextureAsset) -> bool {
    match &asset.payload {
        TexturePayload::Rgba(levels) => {
            levels
                .levels
                .iter()
                .zip(decoded.levels.iter())
                .all(|(a, b)| {
                    a.faces
                        .iter()
                        .zip(b.surfaces.iter())
                        .all(|(a, b)| b.compressed.is_none() && Arc::ptr_eq(a, &b.rgba))
                })
        }
        TexturePayload::BlockCompressed(levels) => levels
            .levels
            .iter()
            .zip(decoded.levels.iter())
            .all(|(a, b)| {
                a.faces.iter().zip(b.surfaces.iter()).all(|(a, b)| {
                    b.rgba.is_empty()
                        && b.compressed
                            .as_ref()
                            .is_some_and(|b| Arc::ptr_eq(a, b.blocks_ref()))
                })
            }),
    }
}

fn convert_render_texture(
    texture: &DecodedTextureAsset,
    mip_policy: RenderTextureMipPolicy,
    wrap_mode: TextureWrapMode,
) -> Option<TextureAsset> {
    let shape = match texture.shape {
        DecodedTextureShape::Texture2d => TextureShape::Texture2d,
        DecodedTextureShape::Cube => TextureShape::Cube,
    };
    let selected = texture
        .levels
        .get(mip_policy.leading_levels_to_drop(texture.levels.len())..)?;
    let face_count = if shape == TextureShape::Cube { 6 } else { 1 };
    if selected.is_empty()
        || selected.iter().any(|level| {
            level.surfaces.len() != face_count
                || level
                    .surfaces
                    .iter()
                    .enumerate()
                    .any(|(face, surface)| surface.authored_face_index != face)
        })
    {
        return None;
    }
    if let Some(asset) = native_bc_asset(shape, selected, wrap_mode) {
        return Some(
            asset.with_source_attribution(TextureSourceAttribution::render_texture(
                selected.iter().flat_map(|level| level.surfaces.iter()),
                TextureConversionRoute::RenderTextureNativeCompressed,
            )),
        );
    }
    let levels: Vec<_> = selected
        .iter()
        .map(|level| {
            let faces: Option<Vec<_>> = level.surfaces.iter().map(rgba_face).collect();
            faces.map(|faces| RgbaTextureLevel {
                width: level.surfaces[0].width,
                height: level.surfaces[0].height,
                faces: Arc::from(faces),
            })
        })
        .collect::<Option<_>>()?;
    let route = if selected
        .iter()
        .flat_map(|level| level.surfaces.iter())
        .any(|surface| surface.clip_map_transform)
    {
        TextureConversionRoute::RenderTextureClipMapRgba
    } else if selected
        .iter()
        .flat_map(|level| level.surfaces.iter())
        .any(|surface| surface.compressed.is_some())
    {
        TextureConversionRoute::RenderTextureMixedRgba
    } else {
        TextureConversionRoute::RenderTextureRgba
    };
    let source = TextureSourceAttribution::render_texture(
        selected.iter().flat_map(|level| level.surfaces.iter()),
        route,
    );
    match shape {
        TextureShape::Texture2d => TextureAsset::rgba_2d(
            levels[0].width,
            levels[0].height,
            Arc::from(levels),
            wrap_mode,
        )
        .map(|asset| asset.with_source_attribution(source)),
        TextureShape::Cube => TextureAsset::rgba_cube(
            levels[0].width,
            levels[0].height,
            Arc::from(levels),
            wrap_mode,
        )
        .map(|asset| asset.with_source_attribution(source)),
    }
}

fn native_bc_asset(
    shape: TextureShape,
    levels: &[crate::DecodedTextureLevel],
    wrap: TextureWrapMode,
) -> Option<TextureAsset> {
    let first = levels.first()?.surfaces.first()?.compressed.as_ref()?;
    let format = first.format();
    let levels: Vec<_> = levels
        .iter()
        .map(|level| {
            let faces: Option<Vec<_>> = level
                .surfaces
                .iter()
                .map(|surface| {
                    let compressed = surface.compressed.as_ref()?;
                    (compressed.format() == format).then(|| compressed.blocks_arc())
                })
                .collect();
            faces.map(|faces| CompressedTextureLevel {
                width: level.surfaces[0].width,
                height: level.surfaces[0].height,
                faces: Arc::from(faces),
            })
        })
        .collect::<Option<_>>()?;
    match shape {
        TextureShape::Texture2d => TextureAsset::bc_2d(
            levels[0].width,
            levels[0].height,
            format,
            Arc::from(levels),
            wrap,
        ),
        TextureShape::Cube => TextureAsset::bc_cube(
            levels[0].width,
            levels[0].height,
            format,
            Arc::from(levels),
            wrap,
        ),
    }
}

#[cfg(test)]
#[path = "texture_asset_tests.rs"]
mod tests;
