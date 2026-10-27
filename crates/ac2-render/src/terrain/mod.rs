//! The landscape (`LScapeTerrainMesh`): blended terrain layers and the per-block near mesh, and
//! the far mesh past the near ring.

mod far;
mod heights;
mod layers;
mod mesh;
mod shadow;

pub use far::{
    far_radius_for_draw_distance, FarTerrain, FarVertex, FarVertices, FIRST_FAR_CHUNK, NEAR_RADIUS,
};
pub use heights::TerrainHeights;
pub use layers::{terrain_layers, AlphaRotation, Corners, TerrainLayer, TerrainLayers};
pub use mesh::{build_terrain_mesh, TerrainBatch, TerrainMesh, TerrainVertex};
pub use shadow::{self_shadow_factor, shadow_light_bit, vertex_shadow_alpha};

#[cfg(test)]
mod real_data_tests;
