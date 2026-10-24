//! The near landscape (`LScapeTerrainMesh`): blended terrain layers and the per-block mesh.

mod layers;
mod mesh;

pub use layers::{terrain_layers, AlphaRotation, Corners, TerrainLayer, TerrainLayers};
pub use mesh::{build_terrain_mesh, TerrainBatch, TerrainMesh, TerrainVertex};

#[cfg(test)]
mod real_data_tests;
