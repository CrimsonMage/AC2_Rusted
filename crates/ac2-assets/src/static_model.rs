use std::sync::Arc;

use ac2_core::DataId;
use ac2_math::BBox;

use crate::cull::DecodedSurfaceCullMode;
use crate::mesh::DecodedMeshVertex;
use crate::static_surface::{DecodedConstructedSurfaceDelay, DecodedStaticSurfaceMaterial};
use crate::texture::DecodedTextureAsset;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedStaticSurfaceBatch {
    /// Owning child GfxObj position inside a Setup root; zero for a direct GfxObj decode.
    pub mesh_ordinal: usize,
    /// Native `GfxObj::surfaces` position. Consumers retain this through deferred alpha lanes
    /// instead of re-sorting subsets by data id.
    pub subset_ordinal: usize,
    pub surface_did: DataId,
    pub cull_mode: DecodedSurfaceCullMode,
    pub material: DecodedStaticSurfaceMaterial,
    /// Delay lane baked from the geometry GfxObj's original surface array by `ConstructMesh`.
    pub constructed_delay: DecodedConstructedSurfaceDelay,
    /// Native ConstructMesh subset addressing bit accumulated from its polygons.
    pub stippled: bool,
    pub texture: Arc<DecodedTextureAsset>,
    pub vertices: Arc<[DecodedMeshVertex]>,
    pub indices: Arc<[u32]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedStaticModel {
    pub model_id: DataId,
    pub bounds: BBox,
    pub surface_batches: Arc<[DecodedStaticSurfaceBatch]>,
}
