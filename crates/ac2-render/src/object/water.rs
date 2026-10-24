//! The near water of one landblock as `LScapeTerrainMesh::UpdateNearWater` builds it: a 4 x 4 grid
//! of quads a quarter block wide (`LScapeTerrainMesh::Init`: 5 side vertices, 4 side polygons,
//! `m_fNearWaterSize = block_length * 0.25`), four unshared vertices per quad with UVs 0..1 and
//! an up normal at the water height, indexed (2, 0, 3), (0, 1, 3). `BuildNearWaterMesh` draws it
//! with the block's above-water material (`DrawWaterMesh`, tint white, high detail).

use std::sync::Arc;

use ac2_assets::load_ac2_material;
use ac2_core::CellId;
use ac2_db::{ResourceResult, ResourceStore};
use ac2_math::Mat4;
use ac2_world::Ac2BlockWater;

use super::model::{ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectVertex};
use super::pass::{layer_drawn, RenderSettings};

const SIDE_POLYGONS: u32 = 4;

/// The block's near water mesh with its drawn material layers, in world space. `None` when the
/// material has no layer this device draws.
pub fn load_water_model(
    store: &ResourceStore,
    block_cell_id: CellId,
    water: &Ac2BlockWater,
    block_size: f32,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    let Some(material) = load_ac2_material(store, water.above_material)? else {
        return Ok(None);
    };
    let layers: Vec<_> = material
        .layers
        .into_iter()
        .filter(|layer| layer_drawn(layer, settings))
        .collect();
    if layers.is_empty() {
        return Ok(None);
    }
    let origin_x = ((block_cell_id.raw() >> 24) & 0xFF) as f32 * block_size;
    let origin_y = ((block_cell_id.raw() >> 16) & 0xFF) as f32 * block_size;
    let size = block_size * 0.25;
    let mut vertices = Vec::with_capacity((SIDE_POLYGONS * SIDE_POLYGONS * 4) as usize);
    let mut indices = Vec::with_capacity((SIDE_POLYGONS * SIDE_POLYGONS * 6) as usize);
    for i in 0..SIDE_POLYGONS {
        for j in 0..SIDE_POLYGONS {
            let base = vertices.len() as u32;
            // Corner k of quad (i, j) sits at vertex (i + dx, j + dy) with UV (dx, dy).
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                vertices.push(ObjectVertex {
                    position: [
                        (i + dx) as f32 * size + origin_x,
                        (j + dy) as f32 * size + origin_y,
                        water.height,
                    ],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0; 4],
                    uvs: [[dx as f32, dy as f32], [0.0; 2], [0.0; 2], [0.0; 2]],
                    tangents: [[0.0; 3]; 2],
                });
            }
            indices.extend([base + 2, base, base + 3, base, base + 1, base + 3]);
        }
    }
    Ok(Some(ObjectModel {
        visual: water.above_material,
        parts: vec![ObjectPart {
            mesh: water.above_material,
            transform: Mat4::IDENTITY,
            vertices: Arc::from(vertices),
            batches: vec![ObjectBatch {
                indices: Arc::from(indices),
                material: Arc::new(ObjectMaterial::new(layers)),
            }],
        }],
    }))
}
