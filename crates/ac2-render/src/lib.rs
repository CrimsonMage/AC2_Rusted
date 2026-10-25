#![forbid(unsafe_code)]

//! Asheron's Call 2 scene rendering, backend-neutral: it turns `ac2-world` and `ac2-assets`
//! products into meshes, layer sets and images that a GPU backend (`ac2-gpu`) draws. Modelled on
//! the retail `smartbox/worldrender` pipeline (`LScapeTerrainMesh`, `LScapeMaterialInfo`).

pub mod frill;
pub mod frustum;
mod image;
pub mod object;
pub mod scene;
pub mod sky;
pub mod terrain;
mod view;

pub use image::{
    load_image, load_material_image, load_texture_image, Image, ImageLevel, TextureKey,
};
pub use view::{Camera, SceneLighting};
