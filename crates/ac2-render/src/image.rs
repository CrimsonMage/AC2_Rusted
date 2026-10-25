//! Backend-neutral images: RGBA8 mip chains decoded from AC2 `RenderTexture`s, ready for upload.

use std::sync::Arc;

use ac2_assets::{
    decode_block_compressed_texture, load_ac2_texture_asset, resolve_ac2_material_texture_chain,
    TexturePayload,
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

/// An RGBA8 mip chain, largest level first, each level half the previous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub levels: Vec<ImageLevel>,
}

impl Image {
    pub fn width(&self) -> u32 {
        self.levels[0].width
    }

    pub fn height(&self) -> u32 {
        self.levels[0].height
    }
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

/// An AC2 `RenderTexture` (`0x40......`) as RGBA8 levels; block-compressed levels are decoded.
pub fn load_texture_image(store: &ResourceStore, texture: DataId) -> ResourceResult<Option<Image>> {
    let Some(asset) = load_ac2_texture_asset(store, texture)? else {
        return Ok(None);
    };
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
    Ok(complete_chain(levels).map(|levels| Image { levels }))
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
