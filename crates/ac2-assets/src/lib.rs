#![forbid(unsafe_code)]

//! Asheron's Call 2 render assets: textures, materials, meshes, visual descriptions, FX and
//! particle descriptions, decoded on the CPU into backend-neutral products.

mod ac2_appearance;
mod ac2_cell_surface;
#[cfg(test)]
mod ac2_cell_surface_tests;
mod ac2_entity;
mod ac2_fx;
mod ac2_material;
mod ac2_material_instance;
mod ac2_mesh;
mod ac2_particle_desc;
#[cfg(test)]
mod ac2_particle_tests;
mod ac2_physics_mesh;
#[cfg(test)]
mod ac2_real_data_tests;
mod ac2_static_model;
mod ac2_surface_pixels;
mod ac2_terrain;
mod ac2_texture;
mod ac2_visual_desc;
mod ac2_visual_particles;
mod cull;
mod font;
#[cfg(test)]
mod locomotion_tests;
mod mesh;
mod static_model;
mod static_surface;
mod texture;
mod texture_asset;
mod texture_attribution;
mod texture_dxt;

pub use cull::DecodedSurfaceCullMode;
pub use font::{BitmapFontAsset, BitmapFontGlyph};
pub use mesh::DecodedMeshVertex;

pub use static_model::DecodedStaticModel;
pub(crate) use static_model::DecodedStaticSurfaceBatch;
pub(crate) use static_surface::DecodedConstructedSurfaceDelay;
pub use static_surface::{DecodedStaticSurfaceMaterial, DecodedSurfaceBlendMode};
pub use texture::{
    DecodedTextureAsset, DecodedTextureLevel, DecodedTextureShape, DecodedTextureSurface,
    ResolvedSurfaceIdentity, TextureSourceKind,
};

pub use texture_asset::{
    texture_asset_from_decoded_source, CompressedTextureLevel, RgbaTextureLevel, TextureAsset,
    TexturePayload, TextureShape, TextureWrapMode,
};
pub use texture_attribution::{
    GeneratedTextureOrigin, TextureConversionRoute, TextureSourceAttribution,
};
pub(crate) use texture_dxt::BlockCompressedTexture;
pub use texture_dxt::{decode_block_compressed_texture, BlockCompressedTextureFormat};

pub use ac2_appearance::{
    ac2_appearance_material_atoms, ac2_dungeon_appearance_table, load_ac2_material_with_atoms,
    Ac2MaterialAtom,
};
pub use ac2_cell_surface::decode_ac2_cell_surfaces;
pub use ac2_entity::{read_ac2_entity_desc, Ac2EntityDesc};
pub use ac2_material::{
    Ac2LayerLighting, Ac2LayerModifier, Ac2LayerStage, Ac2MaterialLayer, Ac2RenderMaterial,
    Ac2Waveform,
};
pub use ac2_material_instance::load_ac2_material;
pub use ac2_mesh::Ac2Mesh;
pub use ac2_particle_desc::{
    Ac2ParticleEmitterDesc, Ac2ParticleKeyframe, Ac2ParticleSystemDesc, Ac2WaveformVec3,
};
pub use ac2_physics_mesh::{Ac2PhysicsBspNode, Ac2PhysicsMesh, Ac2PhysicsPolygon, Ac2SetupPhysics};
pub(crate) use ac2_static_model::resolve_ac2_visual_setup;
pub use ac2_static_model::{
    decode_ac2_posed_model, decode_ac2_static_model, resolve_ac2_visual_meshes,
    resolve_ac2_visual_parts, Ac2Pose,
};
pub use ac2_terrain::{ac2_mipmapped_clamped_texture, compose_ac2_vertex_blended_terrain_texture};
pub use ac2_texture::{load_ac2_texture_asset, resolve_ac2_material_texture_chain};
pub use ac2_visual_particles::{resolve_ac2_visual_particles, Ac2VisualParticles};
