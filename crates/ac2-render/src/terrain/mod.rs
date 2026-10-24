//! The landscape (`LScapeTerrainMesh`): blended terrain layers and the per-block near mesh, and
//! the far mesh past the near ring.

mod far;
mod layers;
mod mesh;

pub use far::{
    far_radius_for_draw_distance, FarTerrain, FarVertex, FarVertices, FIRST_FAR_CHUNK, NEAR_RADIUS,
};
pub use layers::{terrain_layers, AlphaRotation, Corners, TerrainLayer, TerrainLayers};
pub use mesh::{build_terrain_mesh, TerrainBatch, TerrainMesh, TerrainVertex};

#[cfg(test)]
mod real_data_tests;
