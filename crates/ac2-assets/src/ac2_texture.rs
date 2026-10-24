use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::ac2_material::AC2_TEXTURE_CHAIN_PREFIX;
use crate::ac2_surface_pixels::{decode_surface_pixels, Ac2SurfacePixels};
use crate::texture_dxt::retain;
use crate::{
    load_ac2_material, BlockCompressedTextureFormat, CompressedTextureLevel, DecodedTextureAsset,
    DecodedTextureLevel, DecodedTextureShape, DecodedTextureSurface, RgbaTextureLevel,
    TextureAsset, TextureSourceKind, TextureWrapMode,
};
use ac2_core::PixelFormatID;

const AC2_TEXTURE_PREFIX: u32 = 0x4100_0000;
/// `RenderTexture::m_SourceLevels` never exceeds a full 2^12 mip chain in the Legions DATs.
const MAX_TEXTURE_LEVELS: usize = 32;

/// AC2 `RenderTexture` (`0x40`): `TextureType` then the `RenderSurface` DID of each source level.
#[derive(Debug)]
struct Ac2TextureChain {
    levels: Vec<DataId>,
}

impl ResourcePackedDecode for Ac2TextureChain {
    const TYPE_NAME: &'static str = "AC2 RenderTexture";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        Ok(Self {
            levels: decode_chain(qdid.data_id(), bytes)?,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.levels.len() * 4
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ac2TextureLevel {
    did: DataId,
    width: u32,
    height: u32,
    pixels: Ac2SurfacePixels,
}

/// `RenderSurface` (`0x41`) records are decoded once into GPU-ready pixels; the typed cache
/// retains that `Arc`, which the composed `TextureAsset` shares instead of copying.
impl ResourcePackedDecode for Ac2TextureLevel {
    const TYPE_NAME: &'static str = "AC2 RenderSurface";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode_level(qdid.data_id(), bytes)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(self.pixels.bytes().len())
    }
}

/// Resolves an AC2 `RenderMaterial` or `MaterialInstance` to the `RenderTexture` its first
/// textured stage samples (after any instance modifiers are applied).
pub fn resolve_ac2_material_texture_chain(
    store: &ResourceStore,
    material_did: DataId,
) -> ResourceResult<Option<DataId>> {
    Ok(load_ac2_material(store, material_did)?
        .and_then(|material| material.base_texture())
        .filter(|did| did.raw() & 0xFF00_0000 == AC2_TEXTURE_CHAIN_PREFIX))
}

/// Decodes an AC2 `RenderTexture` and its `RenderSurface` levels into a texture asset.
/// HighRes is preferred per level and Portal supplies the remaining mip chain.
pub fn load_ac2_texture_asset(
    store: &ResourceStore,
    chain_did: DataId,
) -> ResourceResult<Option<TextureAsset>> {
    let Some(levels) = load_ac2_texture_chain(store, chain_did)? else {
        return Ok(None);
    };
    let levels = mip_chain(&levels);
    let first = &levels[0].1;
    let (width, height) = (first.width, first.height);
    if levels.len() == 1 && (width > 1 || height > 1) {
        return Ok(generated_mip_chain(first));
    }
    Ok(match first.pixels {
        Ac2SurfacePixels::BlockCompressed(format, _) => TextureAsset::bc_2d(
            width,
            height,
            format,
            levels
                .iter()
                .map(|(_, level)| CompressedTextureLevel {
                    width: level.width,
                    height: level.height,
                    faces: Arc::from([level.pixels.bytes().clone()]),
                })
                .collect(),
            TextureWrapMode::Repeat,
        ),
        Ac2SurfacePixels::Rgba(_) => TextureAsset::rgba_2d(
            width,
            height,
            levels
                .iter()
                .map(|(_, level)| RgbaTextureLevel {
                    width: level.width,
                    height: level.height,
                    faces: Arc::from([level.pixels.bytes().clone()]),
                })
                .collect(),
            TextureWrapMode::Repeat,
        ),
    })
}

/// A chain authored with one source level (44 Legions `RenderTexture`s) gets a box-filtered RGBA
/// mip chain to 1 x 1, as the device generates sublevels for a single-level texture; one level
/// would alias and shimmer under minification.
fn generated_mip_chain(level: &Ac2TextureLevel) -> Option<TextureAsset> {
    let rgba: Arc<[u8]> = match &level.pixels {
        Ac2SurfacePixels::Rgba(rgba) => Arc::clone(rgba),
        Ac2SurfacePixels::BlockCompressed(format, blocks) => {
            Arc::from(crate::decode_block_compressed_texture(
                *format,
                level.width as usize,
                level.height as usize,
                blocks,
            )?)
        }
    };
    let mut asset = crate::ac2_mipmapped_clamped_texture(level.width, level.height, rgba)?;
    asset.wrap_mode = TextureWrapMode::Repeat;
    Some(asset)
}

/// The texture chain as the engine's authored render-texture asset, for static models: DXT
/// levels stay native blocks, uncompressed and JFIF levels are RGBA.
pub(crate) fn load_ac2_decoded_texture(
    store: &ResourceStore,
    chain_did: DataId,
) -> ResourceResult<Option<DecodedTextureAsset>> {
    let Some(levels) = load_ac2_texture_chain(store, chain_did)? else {
        return Ok(None);
    };
    let levels = mip_chain(&levels)
        .into_iter()
        .map(|(kind, level)| {
            let (rgba, compressed, pixel_format) = match &level.pixels {
                Ac2SurfacePixels::BlockCompressed(format, blocks) => {
                    let source = match format {
                        BlockCompressedTextureFormat::Bc1Rgba => PixelFormatID::PFID_DXT1,
                        BlockCompressedTextureFormat::Bc2Rgba => PixelFormatID::PFID_DXT3,
                        BlockCompressedTextureFormat::Bc3Rgba => PixelFormatID::PFID_DXT5,
                    };
                    let retained =
                        retain(source, level.width as usize, level.height as usize, blocks);
                    (Arc::from([]), retained, source)
                }
                Ac2SurfacePixels::Rgba(rgba) => {
                    (Arc::clone(rgba), None, PixelFormatID::PFID_A8R8G8B8)
                }
            };
            DecodedTextureLevel {
                source_level_did: level.did,
                surfaces: Arc::from([DecodedTextureSurface {
                    authored_face_index: 0,
                    render_surface_file_kind: kind,
                    render_surface_did: level.did,
                    width: level.width,
                    height: level.height,
                    pixel_format,
                    rgba,
                    clip_map_transform: false,
                    compressed,
                    resolved_identity: None,
                }]),
            }
        })
        .collect();
    Ok(Some(DecodedTextureAsset {
        source_kind: TextureSourceKind::AuthoredRenderTexture,
        texture_did: chain_did,
        shape: DecodedTextureShape::Texture2d,
        raw_texture_type: 2,
        diagnostic_fallback: false,
        levels,
    }))
}

/// The largest level and each following level that halves it in the same pixel kind.
/// Fraction of near-transparent texels (alpha < 64) above which a texture's alpha is authored
/// cut-out coverage (leaf and frond silhouettes cover a third or more) rather than a gloss or skin
/// mask (well under one percent on the human body).
const CUTOUT_COVERAGE: f32 = 0.10;
/// Smallest level side sampled for the coverage estimate.
const COVERAGE_LEVEL_SIDE: u32 = 16;

/// Whether the texture chain's alpha channel carries cut-out coverage, estimated from its
/// smallest level of at least 16 texels on a side.
pub(crate) fn ac2_texture_has_cutout_alpha(
    store: &ResourceStore,
    chain_did: DataId,
) -> ResourceResult<bool> {
    let Some(levels) = load_ac2_texture_chain(store, chain_did)? else {
        return Ok(false);
    };
    let levels = mip_chain(&levels);
    let Some((_, level)) = levels
        .iter()
        .rev()
        .find(|(_, level)| level.width.min(level.height) >= COVERAGE_LEVEL_SIDE)
        .or_else(|| levels.first())
    else {
        return Ok(false);
    };
    let rgba = match &level.pixels {
        Ac2SurfacePixels::BlockCompressed(BlockCompressedTextureFormat::Bc1Rgba, _) => {
            return Ok(false)
        }
        Ac2SurfacePixels::BlockCompressed(format, blocks) => {
            match crate::decode_block_compressed_texture(
                *format,
                level.width as usize,
                level.height as usize,
                blocks,
            ) {
                Some(rgba) => Arc::from(rgba),
                None => return Ok(false),
            }
        }
        Ac2SurfacePixels::Rgba(rgba) => Arc::clone(rgba),
    };
    let texels = rgba.len() / 4;
    let clear = rgba.chunks_exact(4).filter(|pixel| pixel[3] < 64).count();
    Ok(texels > 0 && clear as f32 / texels as f32 >= CUTOUT_COVERAGE)
}

fn mip_chain(levels: &[(DatFileKind, Ac2TextureLevel)]) -> Vec<(DatFileKind, Ac2TextureLevel)> {
    let first = &levels[0].1;
    let mut expected = (first.width, first.height);
    levels
        .iter()
        .take_while(|(_, level)| {
            let accepted =
                level.pixels.same_kind(&first.pixels) && (level.width, level.height) == expected;
            expected = ((expected.0 / 2).max(1), (expected.1 / 2).max(1));
            accepted
        })
        .cloned()
        .collect()
}

fn load_ac2_texture_chain(
    store: &ResourceStore,
    chain_did: DataId,
) -> ResourceResult<Option<Vec<(DatFileKind, Ac2TextureLevel)>>> {
    if chain_did.raw() & 0xFF00_0000 != AC2_TEXTURE_CHAIN_PREFIX {
        return Ok(None);
    }
    let Some(chain) = store.load_cached_packed::<Ac2TextureChain>(TypedResourceRef::new(
        DatFileKind::Portal,
        chain_did,
        0,
    ))?
    else {
        return Ok(None);
    };
    let mut levels = Vec::with_capacity(chain.levels.len());
    for did in &chain.levels {
        // HighRes is optional; an unmounted or missing high-resolution record falls back to Portal.
        let level = match load_level(store, DatFileKind::HighRes, *did) {
            Ok(Some(level)) => Some((DatFileKind::HighRes, level)),
            Ok(None) | Err(_) => load_level(store, DatFileKind::Portal, *did)?
                .map(|level| (DatFileKind::Portal, level)),
        };
        if let Some((kind, level)) = level {
            levels.push((kind, (*level).clone()));
        }
    }
    if levels.is_empty() {
        Ok(None)
    } else {
        levels
            .sort_by_key(|(_, level)| std::cmp::Reverse(level.width.saturating_mul(level.height)));
        Ok(Some(levels))
    }
}

/// `RenderTexture::StreamPack`: DID, `m_SourceTextureType`, then the counted level DIDs.
fn decode_chain(qdid: DataId, bytes: &[u8]) -> ResourceResult<Vec<DataId>> {
    let mut cursor = ArchiveCursor::new(bytes);
    ensure_embedded_did(qdid, cursor.read_u32()?)?;
    let _texture_type = cursor.read_u32()?;
    let count = cursor.read_u32_count()?;
    if count == 0 || count > MAX_TEXTURE_LEVELS || cursor.remaining_bytes().len() != count * 4 {
        return Err(malformed(qdid, "invalid AC2 texture-chain length"));
    }
    let mut dids = Vec::with_capacity(count);
    for _ in 0..count {
        let did = DataId::new(cursor.read_u32()?);
        if did.raw() & 0xFF00_0000 != AC2_TEXTURE_PREFIX {
            return Err(malformed(qdid, "texture chain contained a non-surface DID"));
        }
        dids.push(did);
    }
    Ok(dids)
}

/// `RenderSurface::StreamPack`: DID, data category, width, height, `PixelFormat`, image size and
/// the image bytes.
fn decode_level(qdid: DataId, bytes: &[u8]) -> ResourceResult<Ac2TextureLevel> {
    let mut cursor = ArchiveCursor::new(bytes);
    ensure_embedded_did(qdid, cursor.read_u32()?)?;
    let _category = cursor.read_u32()?;
    let width = cursor.read_u32()?;
    let height = cursor.read_u32()?;
    let format = cursor.read_u32()?;
    let byte_len = cursor.read_u32_count()?;
    if cursor.remaining_bytes().len() != byte_len {
        return Err(malformed(qdid, "invalid AC2 surface payload length"));
    }
    let (width, height, pixels) =
        decode_surface_pixels(format, width, height, cursor.remaining_bytes())
            .map_err(|context| malformed(qdid, context))?;
    Ok(Ac2TextureLevel {
        did: qdid,
        width,
        height,
        pixels,
    })
}

fn load_level(
    store: &ResourceStore,
    kind: DatFileKind,
    did: DataId,
) -> ResourceResult<Option<CachedResource<Ac2TextureLevel>>> {
    store.load_cached_packed::<Ac2TextureLevel>(TypedResourceRef::new(kind, did, 0))
}

fn ensure_embedded_did(expected: DataId, actual: u32) -> ResourceResult<()> {
    if expected.raw() == actual {
        Ok(())
    } else {
        Err(malformed(
            expected,
            "embedded DID did not match DAT identity",
        ))
    }
}

fn malformed(did: DataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid: QualifiedDataId::new(did, 0),
        type_name: "AC2 texture",
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_chain, decode_level};
    use ac2_core::DataId;

    #[test]
    fn decodes_ac2_texture_chain() {
        let did = DataId::new(0x4000_0257);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&did.raw().to_le_bytes());
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&0x4100_1338_u32.to_le_bytes());
        bytes.extend_from_slice(&0x4100_1339_u32.to_le_bytes());
        assert_eq!(decode_chain(did, &bytes).unwrap().len(), 2);
    }

    #[test]
    fn decodes_ac2_dxt1_header_and_blocks() {
        let did = DataId::new(0x4100_1338);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&did.raw().to_le_bytes());
        bytes.extend_from_slice(&0x8000_000D_u32.to_le_bytes());
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(b"DXT1");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(&[0_u8; 8]);
        let level = decode_level(did, &bytes).unwrap();
        assert_eq!(
            (level.width, level.height, level.pixels.bytes().len()),
            (4, 4, 8)
        );
    }
}
