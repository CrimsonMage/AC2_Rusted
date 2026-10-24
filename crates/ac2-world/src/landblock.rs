//! One Asheron's Call 2 landblock's terrain: the per-vertex samples of its `CLandBlockData`.

use std::sync::Arc;

use ac2_core::QualifiedDataId;

use crate::ac2_world::Ac2TerrainSamples;
use crate::Ac2SurfaceInfo;
use crate::LAND_BLOCK_CELLS_PER_SIDE;

#[derive(Debug, Clone, PartialEq)]
pub struct LandBlockDesc {
    qdid: QualifiedDataId,
    samples: Arc<Ac2TerrainSamples>,
}

impl LandBlockDesc {
    pub(crate) fn new(qdid: QualifiedDataId, samples: Arc<Ac2TerrainSamples>) -> Self {
        Self { qdid, samples }
    }

    pub fn qdid(&self) -> QualifiedDataId {
        self.qdid
    }

    /// Terrain vertices per block side (17: two per land cell).
    pub fn vertex_side(&self) -> usize {
        self.samples.vertex_side()
    }

    /// Terrain vertices per land-cell side.
    pub fn vertices_per_cell(&self) -> usize {
        (self.vertex_side() - 1) / LAND_BLOCK_CELLS_PER_SIDE
    }

    /// Unit normal of vertex (`x`, `y`) (`CLandBlockData::m_vNormals`, AC axes, z up).
    pub fn normal(&self, x: usize, y: usize) -> Option<[f32; 3]> {
        let side = self.vertex_side();
        (x < side && y < side)
            .then(|| self.samples.normal(x * side + y))
            .flatten()
    }

    /// Terrain `SurfaceInfo` of vertex (`x`, `y`) (`CLandBlockData::m_vertexSurfaceInfo`).
    pub fn surface_info(&self, x: usize, y: usize) -> Option<&Ac2SurfaceInfo> {
        let side = self.vertex_side();
        (x < side && y < side)
            .then(|| self.samples.surface(x * side + y))
            .flatten()
    }

    /// Road `SurfaceInfo` of vertex (`x`, `y`) (`CLandBlockData::m_vertexRoadInfo`); its base
    /// material is empty where no road runs.
    pub fn road_info(&self, x: usize, y: usize) -> Option<&Ac2SurfaceInfo> {
        let side = self.vertex_side();
        (x < side && y < side)
            .then(|| self.samples.road(x * side + y))
            .flatten()
    }

    /// Whether the terrain polygon of `polygon_size` vertex steps at vertex (`vertex_x`,
    /// `vertex_y`) is a landscape hole: any of its vertices has the hole terrain type, which the
    /// original removes from both the render and the physics terrain.
    pub fn ac2_polygon_is_hole(
        &self,
        vertex_x: usize,
        vertex_y: usize,
        polygon_size: usize,
    ) -> bool {
        let side = self.vertex_side();
        let size = polygon_size.max(1);
        (vertex_x..=(vertex_x + size).min(side - 1)).any(|x| {
            (vertex_y..=(vertex_y + size).min(side - 1)).any(|y| self.samples.is_hole(x * side + y))
        })
    }

    /// Height at a vertex of the block grid.
    fn grid_height(&self, x: usize, y: usize) -> f32 {
        self.samples
            .height(x * self.vertex_side() + y)
            .unwrap_or(0.0)
    }

    /// Mean height of a land cell's four corner vertices.
    pub fn average_cell_height(&self, cell_x: usize, cell_y: usize) -> Option<f32> {
        if cell_x >= LAND_BLOCK_CELLS_PER_SIDE || cell_y >= LAND_BLOCK_CELLS_PER_SIDE {
            return None;
        }
        let step = self.vertices_per_cell();
        let corner = |x: usize, y: usize| self.grid_height(x * step, y * step);
        Some(
            (corner(cell_x, cell_y)
                + corner(cell_x + 1, cell_y)
                + corner(cell_x, cell_y + 1)
                + corner(cell_x + 1, cell_y + 1))
                * 0.25,
        )
    }

    /// Heights of the block's vertex grid (`vertex_side()` squared, x-major).
    pub fn height_grid(&self) -> Vec<f32> {
        let side = self.vertex_side();
        (0..side)
            .flat_map(|x| (0..side).map(move |y| (x, y)))
            .map(|(x, y)| self.grid_height(x, y))
            .collect()
    }
}
