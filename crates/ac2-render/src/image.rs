//! Backend-neutral images: mip chains of AC2 `RenderTexture`s, ready for upload. DXT levels stay
//! native blocks when the GPU samples BC formats ([`set_gpu_block_compression`]); otherwise, and
//! for every other format, the levels are RGBA8.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use ac2_assets::BlockCompressedTextureFormat;
use ac2_assets::{
    decode_block_compressed_texture, load_ac2_texture_asset, resolve_ac2_material_texture_chain,
    CompressedTextureLevel, TexturePayload,
};
use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

/// One RGBA8 level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageLevel {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

/// One level of DXT blocks (`ceil(w / 4) * ceil(h / 4)` blocks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLevel {
    pub width: u32,
    pub height: u32,
    pub blocks: Arc<[u8]>,
}

/// A DXT mip chain kept as blocks for a BC-capable GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockImage {
    pub format: BlockCompressedTextureFormat,
    pub levels: Vec<BlockLevel>,
}

/// A mip chain, largest level first, each level half the previous: RGBA8 `levels`, or (with
/// `levels` empty) the DXT blocks in `blocks`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub levels: Vec<ImageLevel>,
    pub blocks: Option<BlockImage>,
}

impl Image {
    /// An RGBA8 chain.
    pub fn rgba(levels: Vec<ImageLevel>) -> Self {
        Self {
            levels,
            blocks: None,
        }
    }

    pub fn width(&self) -> u32 {
        match &self.blocks {
            Some(blocks) => blocks.levels[0].width,
            None => self.levels[0].width,
        }
    }

    pub fn height(&self) -> u32 {
        match &self.blocks {
            Some(blocks) => blocks.levels[0].height,
            None => self.levels[0].height,
        }
    }
}

/// Whether DXT textures load as native blocks: set once from the device's
/// `TEXTURE_COMPRESSION_BC` before the scene loads anything. Off, they decode to RGBA8 on the
/// loader (the frame thread never decodes either way).
static GPU_BLOCK_COMPRESSION: AtomicBool = AtomicBool::new(false);

pub fn set_gpu_block_compression(enabled: bool) {
    GPU_BLOCK_COMPRESSION.store(enabled, Ordering::Release);
}

pub fn gpu_block_compression() -> bool {
    GPU_BLOCK_COMPRESSION.load(Ordering::Acquire)
}

/// Bytes in one 4 x 4 block.
pub fn block_bytes(format: BlockCompressedTextureFormat) -> usize {
    match format {
        BlockCompressedTextureFormat::Bc1Rgba => 8,
        BlockCompressedTextureFormat::Bc2Rgba | BlockCompressedTextureFormat::Bc3Rgba => 16,
    }
}

/// Bytes of a `width` x `height` level of `format` blocks.
fn block_level_bytes(format: BlockCompressedTextureFormat, width: u32, height: u32) -> usize {
    width.div_ceil(4) as usize * height.div_ceil(4) as usize * block_bytes(format)
}

/// What a texture is for: a `RenderMaterial`'s base texture, an alpha mask `RenderTexture`, or a
/// layer stage's `RenderTexture`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureKey {
    Material(DataId),
    Mask(DataId),
    Texture(DataId),
}

/// The image a texture key names: a material's base texture, or a `RenderTexture` itself.
pub fn load_image(store: &ResourceStore, key: TextureKey) -> ResourceResult<Option<Image>> {
    match key {
        TextureKey::Material(did) => load_material_image(store, did),
        TextureKey::Mask(did) | TextureKey::Texture(did) => load_texture_image(store, did),
    }
}

/// The base texture of a `RenderMaterial`'s first layer (`LScapeMaterialInfo::GetCombineMaterials`
/// takes `layers[0].stages[0].texture`).
pub fn load_material_image(
    store: &ResourceStore,
    material: DataId,
) -> ResourceResult<Option<Image>> {
    match resolve_ac2_material_texture_chain(store, material)? {
        Some(chain) => load_texture_image(store, chain),
        None => Ok(None),
    }
}

/// An AC2 `RenderTexture` (`0x40......`): DXT levels as blocks when the GPU takes them and the
/// chain is block-aligned, otherwise RGBA8 levels (block-compressed levels decoded).
pub fn load_texture_image(store: &ResourceStore, texture: DataId) -> ResourceResult<Option<Image>> {
    let Some(asset) = load_ac2_texture_asset(store, texture)? else {
        return Ok(None);
    };
    if let TexturePayload::BlockCompressed(levels) = asset.payload() {
        if gpu_block_compression() {
            if let Some(blocks) = block_chain(levels.format, &levels.levels) {
                return Ok(Some(Image {
                    levels: Vec::new(),
                    blocks: Some(blocks),
                }));
            }
        }
    }
    let levels: Vec<ImageLevel> = match asset.payload() {
        TexturePayload::Rgba(levels) => levels
            .levels
            .iter()
            .filter_map(|level| {
                Some(ImageLevel {
                    width: level.width,
                    height: level.height,
                    rgba: Arc::clone(level.faces.first()?),
                })
            })
            .collect(),
        TexturePayload::BlockCompressed(levels) => levels
            .levels
            .iter()
            .filter_map(|level| {
                let rgba = decode_block_compressed_texture(
                    levels.format,
                    level.width as usize,
                    level.height as usize,
                    level.faces.first()?,
                )?;
                Some(ImageLevel {
                    width: level.width,
                    height: level.height,
                    rgba: Arc::from(rgba),
                })
            })
            .collect(),
    };
    Ok(complete_chain(levels).map(Image::rgba))
}

/// The DXT levels that halve cleanly from a block-aligned first level (wgpu takes BC textures
/// only at sizes that are whole blocks; smaller mips are padded to a block), or `None` when the
/// first level is not block-aligned.
fn block_chain(
    format: BlockCompressedTextureFormat,
    source: &[CompressedTextureLevel],
) -> Option<BlockImage> {
    let first = source.first()?;
    if first.width % 4 != 0 || first.height % 4 != 0 {
        return None;
    }
    let (mut w, mut h) = (first.width, first.height);
    let mut levels = Vec::with_capacity(source.len());
    for level in source {
        let Some(blocks) = level.faces.first() else {
            break;
        };
        if level.width != w || level.height != h || blocks.len() != block_level_bytes(format, w, h)
        {
            break;
        }
        levels.push(BlockLevel {
            width: w,
            height: h,
            blocks: Arc::clone(blocks),
        });
        if w == 1 && h == 1 {
            break;
        }
        (w, h) = ((w / 2).max(1), (h / 2).max(1));
    }
    Some(BlockImage { format, levels })
}

/// Keeps the levels that halve cleanly from the first, so every GPU mip level is present.
fn complete_chain(levels: Vec<ImageLevel>) -> Option<Vec<ImageLevel>> {
    let first = levels.first()?;
    let (mut w, mut h) = (first.width, first.height);
    let mut kept = Vec::with_capacity(levels.len());
    for level in levels {
        if level.width != w || level.height != h || level.rgba.len() != (w * h * 4) as usize {
            break;
        }
        kept.push(level);
        if w == 1 && h == 1 {
            break;
        }
        (w, h) = ((w / 2).max(1), (h / 2).max(1));
    }
    Some(kept)
}
