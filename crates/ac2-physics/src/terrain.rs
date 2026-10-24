//! Resident terrain for collision: the land cells of loaded landblocks with their terrain
//! polygons (`CLandBlockMesh::ConstructVertices` + `ConstructPolygons`).

use std::collections::HashMap;

use ac2_core::CellId;
use ac2_world::{terrain_polygon_cut, LandBlockDesc};

use crate::cells::{CellSource, LandCell, TerrainPolygon};
use crate::fpu::{ld, st};
use crate::landdefs::{
    adjust_to_outside, blockid_to_lcoord, lcoord_to_gid, BLOCK_LENGTH, LBLOCK_SIDE,
    POLYS_PER_LANDCELL, SIDE_VERTEX_COUNT, VERTEX_PER_CELL,
};
use crate::position::Position;
use crate::vector::Vector;

/// Terrain polygons per block side.
const SIDE_POLYGON_COUNT: usize = SIDE_VERTEX_COUNT - 1;

/// The resident landblocks' land cells, keyed by cell id.
#[derive(Debug, Clone, Default)]
pub struct Terrain {
    cells: HashMap<u32, LandCell>,
    /// `CLandBlockData::m_waterPlane` heights of the blocks that have water.
    water: HashMap<u32, f32>,
}

impl Terrain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes a landblock resident from its `CLandBlockData`.
    pub fn insert_block(&mut self, block_cell_id: CellId, desc: &LandBlockDesc) {
        let heights = desc.height_grid();
        if heights.len() != SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT {
            return;
        }
        self.insert_heights(block_cell_id.raw(), &heights, |x, y| {
            desc.ac2_polygon_is_hole(x, y, 1)
        });
    }

    /// Makes a landblock resident from a 17 x 17 x-major vertex height grid (metres) and a
    /// per-polygon hole test.
    pub fn insert_heights(
        &mut self,
        block_cell_id: u32,
        heights: &[f32],
        is_hole: impl Fn(usize, usize) -> bool,
    ) {
        let Some((block_x, block_y)) = blockid_to_lcoord(block_cell_id) else {
            return;
        };
        let block = CellId::new(block_cell_id & 0xFFFF_0000);
        // `ConstructVertices`: `poly_size = block_length / side_polygon_count`.
        let spacing = st(ld(BLOCK_LENGTH) / SIDE_POLYGON_COUNT as f64);
        let vertex = |x: usize, y: usize| {
            Vector::new(
                st(x as f64 * ld(spacing)),
                st(y as f64 * ld(spacing)),
                heights[x * SIDE_VERTEX_COUNT + y],
            )
        };
        for cell_x in 0..LBLOCK_SIDE as usize {
            for cell_y in 0..LBLOCK_SIDE as usize {
                let cell_id = lcoord_to_gid(block_x + cell_x as i32, block_y + cell_y as i32);
                let mut polygons = [None; POLYS_PER_LANDCELL];
                for ix in 0..VERTEX_PER_CELL {
                    for iy in 0..VERTEX_PER_CELL {
                        let (px, py) =
                            (cell_x * VERTEX_PER_CELL + ix, cell_y * VERTEX_PER_CELL + iy);
                        let slot = (VERTEX_PER_CELL * ix + iy) * 2;
                        if is_hole(px, py) {
                            continue;
                        }
                        let triangles = if terrain_polygon_cut(block, px, py, SIDE_POLYGON_COUNT) {
                            [
                                [vertex(px, py), vertex(px + 1, py), vertex(px + 1, py + 1)],
                                [vertex(px, py), vertex(px + 1, py + 1), vertex(px, py + 1)],
                            ]
                        } else {
                            [
                                [vertex(px, py), vertex(px + 1, py), vertex(px, py + 1)],
                                [
                                    vertex(px + 1, py + 1),
                                    vertex(px, py + 1),
                                    vertex(px + 1, py),
                                ],
                            ]
                        };
                        polygons[slot] = Some(TerrainPolygon::new(triangles[0]));
                        polygons[slot + 1] = Some(TerrainPolygon::new(triangles[1]));
                    }
                }
                self.cells.insert(cell_id, LandCell { cell_id, polygons });
            }
        }
    }

    /// Drops a landblock's cells.
    pub fn remove_block(&mut self, block_cell_id: u32) {
        let block = block_cell_id & 0xFFFF_0000;
        self.cells.retain(|id, _| id & 0xFFFF_0000 != block);
        self.water.remove(&block);
    }

    /// Sets a block's water surface height (`CLandBlockData::m_bHasWater`/`m_waterPlane`), or
    /// removes its water.
    pub fn set_block_water(&mut self, block_cell_id: u32, height: Option<f32>) {
        let block = block_cell_id & 0xFFFF_0000;
        match height {
            Some(height) => self.water.insert(block, height),
            None => self.water.remove(&block),
        };
    }

    /// `CLandCell::InqWaterPlane` (`CLandBlockMesh::InqWaterPlane`): the cell's block water
    /// plane, `z = height`.
    pub(crate) fn water_plane(&self, cell_id: u32) -> Option<crate::vector::Plane> {
        self.land_cell(cell_id)?;
        let height = *self.water.get(&(cell_id & 0xFFFF_0000))?;
        Some(crate::vector::Plane::new(
            Vector::new(0.0, 0.0, 1.0),
            -height,
        ))
    }

    /// The vertical distance from `position`'s origin down to the terrain polygon under it
    /// (negative when the origin is below it), as `CPhysicsObj::GetHeightAboveWalkable`'s
    /// downward `LOSCheck` finds it over open terrain. `None` off resident terrain.
    pub(crate) fn height_above_terrain(&self, position: &Position) -> Option<f32> {
        let mut cell_id = position.cell_id;
        let mut point = position.frame.origin;
        if !adjust_to_outside(&mut cell_id, &mut point) {
            return None;
        }
        let polygon = self.land_cell(cell_id)?.find_terrain_poly(point)?;
        let n = polygon.plane.n;
        if n.z == 0.0 {
            return None;
        }
        let ground =
            -((ld(n.x) * ld(point.x) + ld(n.y) * ld(point.y)) + ld(polygon.plane.d)) / ld(n.z);
        Some(st(ld(point.z) - ground))
    }
}

impl CellSource for Terrain {
    fn land_cell(&self, cell_id: u32) -> Option<&LandCell> {
        self.cells.get(&cell_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;
    use crate::vector::Quaternion;

    fn slope_block() -> Terrain {
        // Rises 1 m per 10 m east.
        let heights: Vec<f32> = (0..SIDE_VERTEX_COUNT)
            .flat_map(|x| (0..SIDE_VERTEX_COUNT).map(move |_| x as f32))
            .collect();
        let mut terrain = Terrain::new();
        terrain.insert_heights(0x7F7F_FFFF, &heights, |_, _| false);
        terrain
    }

    #[test]
    fn every_cell_gets_eight_polygons_covering_it() {
        let terrain = slope_block();
        for local in 1..=64 {
            let cell = terrain.land_cell(0x7F7F_0000 | local).expect("cell");
            assert!(cell.polygons.iter().all(Option::is_some));
        }
        let cell = terrain.land_cell(0x7F7F_0001).expect("cell");
        for point in [(1.0, 1.0), (9.0, 1.0), (15.0, 15.0), (19.9, 0.1)] {
            let found = cell.find_terrain_poly(Vector::new(point.0, point.1, 0.0));
            let n = found.expect("covered").plane.n;
            assert!((n.x + 0.099_503_72).abs() < 1e-6, "{n:?}");
        }
    }

    #[test]
    fn height_above_terrain_on_a_slope() {
        let terrain = slope_block();
        let position = Position::new(
            0x7F7F_0001,
            Frame::new(Vector::new(15.0, 5.0, 3.5), Quaternion::IDENTITY),
        );
        let height = terrain.height_above_terrain(&position).expect("terrain");
        assert!((height - 2.0).abs() < 1e-5, "{height}");
    }

    #[test]
    fn holes_leave_their_polygons_out() {
        let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
        let mut terrain = Terrain::new();
        terrain.insert_heights(0x7F7F_FFFF, &heights, |x, y| x == 0 && y == 0);
        let cell = terrain.land_cell(0x7F7F_0001).expect("cell");
        assert!(cell.polygons[0].is_none() && cell.polygons[1].is_none());
        assert!(cell.polygons[2].is_some());
    }
}
