#![forbid(unsafe_code)]

//! Asheron's Call 2 scene rendering, backend-neutral: it turns `ac2-world` and `ac2-assets`
//! products into meshes, layer sets and images that a GPU backend (`ac2-gpu`) draws. Modelled on
//! the retail `smartbox/worldrender` pipeline (`LScapeTerrainMesh`, `LScapeMaterialInfo`).

pub mod frill;
pub mod frustum;
mod image;
pub mod object;
pub mod particles;
pub mod portal;
pub mod prefs;
pub mod rain;
pub mod scene;
pub mod selection;
#[cfg(test)]
mod selection_tests;
pub mod sky;
pub mod streak;
pub mod terrain;
mod view;

pub use image::{
    block_bytes, gpu_block_compression, load_image, load_material_image, load_texture_image,
    set_gpu_block_compression, BlockCompressedTextureFormat, BlockImage, BlockLevel, Image,
    ImageLevel, TextureKey,
};
pub use view::{Camera, SceneLighting};
