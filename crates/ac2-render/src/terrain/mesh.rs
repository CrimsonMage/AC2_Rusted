//! The near landscape mesh of one landblock (`LScapeTerrainMesh::UpdateNearTerrain`): four
//! vertices per terrain polygon (slots SW, SE, NW, NE, as `LScapeTerrainMesh::SetVertex` writes
//! them), two triangles split along the polygon's `CLandBlockData` diagonal, and polygons grouped
//! by their layer set the way the near mesh groups index lists per `LScapeMaterialInfo`.

use std::collections::BTreeMap;

use ac2_core::CellId;
use ac2_world::{terrain_polygon_cut, Ac2SurfaceInfo, LandBlockDesc};

use super::layers::{terrain_layers, TerrainLayers};

/// One near-mesh vertex in world AC axes (x east, y north, z up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    /// `CLandBlockData::m_vNormals`.
    pub normal: [f32; 3],
    /// The corner's terrain `SurfaceInfo::vertex_color`, which `UpdateNearTerrain` stores as the
    /// vertex specular colour (diffuse is white until `IlluminateVertices` lights it).
    pub surface_color: [f32; 4],
    /// Block-local vertex coordinates (0..=16); `LScapeMaterialInfo::SetUVs` scales these by each
    /// layer's tiling.
    pub grid: [f32; 2],
    /// Polygon slot 0 SW, 1 SE, 2 NW, 3 NE, which selects the alpha-mask UV.
    pub corner: u32,
}

/// Triangles of one layer set.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainBatch {
    pub layers: TerrainLayers,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,
    pub batches: Vec<TerrainBatch>,
}

/// Builds the near mesh of the landblock `block_cell_id` whose `block_size` is in metres. Hole
/// polygons (`CLandBlockMesh::ConstructPolygons`) and polygons without a material are skipped.
pub fn build_terrain_mesh(
    block_cell_id: CellId,
    landblock: &LandBlockDesc,
    block_size: f32,
) -> TerrainMesh {
    let side = landblock.vertex_side();
    let polygons = side - 1;
    let heights = landblock.height_grid();
    let spacing = block_size / polygons as f32;
    let origin = [
        ((block_cell_id.raw() >> 24) & 0xFF) as f32 * block_size,
        ((block_cell_id.raw() >> 16) & 0xFF) as f32 * block_size,
    ];
    let height = |x: usize, y: usize| heights[x * side + y];
    let normal = |x: usize, y: usize| landblock.normal(x, y).unwrap_or([0.0, 0.0, 1.0]);
    let default = Ac2SurfaceInfo::default();
    let surface = |x: usize, y: usize| landblock.surface_info(x, y).unwrap_or(&default);
    let road = |x: usize, y: usize| landblock.road_info(x, y).unwrap_or(&default);

    let mut mesh = TerrainMesh::default();
    let mut batches = BTreeMap::<TerrainLayers, Vec<u32>>::new();
    for px in 0..polygons {
        for py in 0..polygons {
            if landblock.ac2_polygon_is_hole(px, py, 1) {
                continue;
            }
            let slots = [(px, py), (px + 1, py), (px, py + 1), (px + 1, py + 1)];
            let Some(layers) = terrain_layers(
                slots.map(|(x, y)| surface(x, y)),
                slots.map(|(x, y)| road(x, y)),
            ) else {
                continue;
            };
            let first = mesh.vertices.len() as u32;
            for (corner, (x, y)) in slots.into_iter().enumerate() {
                mesh.vertices.push(TerrainVertex {
                    position: [
                        origin[0] + x as f32 * spacing,
                        origin[1] + y as f32 * spacing,
                        height(x, y),
                    ],
                    normal: normal(x, y),
                    surface_color: surface(x, y).vertex_color,
                    grid: [x as f32, y as f32],
                    corner: corner as u32,
                });
            }
            let [sw, se, nw, ne] = [first, first + 1, first + 2, first + 3];
            // `m_SWtoNEcut`: split along SW-NE, else along SE-NW; counter-clockwise from above.
            let triangles = if terrain_polygon_cut(block_cell_id, px, py, polygons) {
                [nw, sw, ne, sw, se, ne]
            } else {
                [sw, se, nw, se, ne, nw]
            };
            batches.entry(layers).or_default().extend(triangles);
        }
    }
    mesh.batches = batches
        .into_iter()
        .map(|(layers, indices)| TerrainBatch { layers, indices })
        .collect();
    mesh
}
