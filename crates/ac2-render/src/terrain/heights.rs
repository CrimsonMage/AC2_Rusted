//! The near terrain's surface of one landblock for height queries on the frame thread:
//! `CLandCell::find_terrain_plane` (the polygon under a point and its `m_SWtoNEcut` triangle)
//! then `Plane::set_height`, over the same vertex heights the near mesh draws.

use ac2_core::CellId;
use ac2_world::{terrain_polygon_cut, LandBlockDesc};

use crate::decal::{GroundTriangle, GroundVertex};

use super::shadow::{self_shadow_factor, vertex_shadow_alpha};

/// One landblock's terrain heights and polygon diagonals, and its water surface.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainHeights {
    origin: [f32; 2],
    spacing: f32,
    side: usize,
    heights: Vec<f32>,
    /// `CLandBlockData::GetColor` per vertex: the shadow-light bits each vertex is lit by.
    colours: Vec<u32>,
    /// `CLandBlockData::m_vNormals` per vertex, as `build_terrain_mesh` reads them.
    normals: Vec<[f32; 3]>,
    /// Per polygon (`px * polygons + py`): split along SW-NE.
    cuts: Vec<bool>,
    /// The block's water surface height, when it has water.
    pub water: Option<f32>,
}

impl TerrainHeights {
    pub fn new(
        block_cell_id: CellId,
        landblock: &LandBlockDesc,
        block_size: f32,
        water: Option<f32>,
    ) -> Self {
        let side = landblock.vertex_side();
        let polygons = side - 1;
        let cuts = (0..polygons)
            .flat_map(|px| {
                (0..polygons).map(move |py| terrain_polygon_cut(block_cell_id, px, py, polygons))
            })
            .collect();
        let colours = (0..side)
            .flat_map(|x| (0..side).map(move |y| (x, y)))
            .map(|(x, y)| landblock.vertex_colour(x, y).unwrap_or(0))
            .collect();
        let normals = (0..side)
            .flat_map(|x| (0..side).map(move |y| (x, y)))
            .map(|(x, y)| landblock.normal(x, y).unwrap_or([0.0, 0.0, 1.0]))
            .collect();
        Self {
            origin: [
                ((block_cell_id.raw() >> 24) & 0xFF) as f32 * block_size,
                ((block_cell_id.raw() >> 16) & 0xFF) as f32 * block_size,
            ],
            spacing: block_size / polygons as f32,
            side,
            heights: landblock.height_grid(),
            colours,
            normals,
            cuts,
            water,
        }
    }

    /// Position of world `(x, y)` in vertex-grid units from the block's corner; `None` off the
    /// block.
    fn grid_at(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let polygons = self.side - 1;
        let gx = (x - self.origin[0]) / self.spacing;
        let gy = (y - self.origin[1]) / self.spacing;
        if !(0.0..=polygons as f32).contains(&gx) || !(0.0..=polygons as f32).contains(&gy) {
            return None;
        }
        Some((gx, gy))
    }

    /// The side of one polygon in metres (`LandDefs::pixel_length`).
    pub fn polygon_size(&self) -> f32 {
        self.spacing
    }

    /// The landscape self-shadow factor under world `(x, y)` (`LScapeTerrainMesh::GetShadowColor`
    /// self term): the polygon's four corner vertices, alpha 255 where lit by `shadow_bit`
    /// (`UpdateNearTerrain`). Off the block, or with no shadow bit, nothing is shadowed (1.0).
    pub fn self_shadow(&self, x: f32, y: f32, shadow_bit: Option<u32>) -> f32 {
        let (Some(bit), Some((gx, gy))) = (shadow_bit, self.grid_at(x, y)) else {
            return 1.0;
        };
        let polygons = self.side - 1;
        let px = (gx.floor() as usize).min(polygons - 1);
        let py = (gy.floor() as usize).min(polygons - 1);
        let alpha = |dx: usize, dy: usize| {
            vertex_shadow_alpha(self.colours[(px + dx) * self.side + py + dy], bit)
        };
        self_shadow_factor([alpha(0, 0), alpha(1, 0), alpha(0, 1), alpha(1, 1)])
    }

    /// The ground triangles of the polygon under world `(x, y)`, split as the near mesh splits it
    /// (`build_terrain_mesh`'s `m_SWtoNEcut`), with each corner's world position and normal. This
    /// is the quad `LScapeTerrainMesh::GetPixelGeometry` gives a decal. `None` off the block.
    pub fn polygon_triangles(&self, x: f32, y: f32) -> Option<[GroundTriangle; 2]> {
        let polygons = self.side - 1;
        let (gx, gy) = self.grid_at(x, y)?;
        let px = (gx.floor() as usize).min(polygons - 1);
        let py = (gy.floor() as usize).min(polygons - 1);
        let corner = |dx: usize, dy: usize| {
            let (vx, vy) = (px + dx, py + dy);
            GroundVertex {
                position: [
                    self.origin[0] + vx as f32 * self.spacing,
                    self.origin[1] + vy as f32 * self.spacing,
                    self.heights[vx * self.side + vy],
                ],
                normal: self.normals[vx * self.side + vy],
            }
        };
        let (sw, se, nw, ne) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
        Some(if self.cuts[px * polygons + py] {
            [[nw, sw, ne], [sw, se, ne]]
        } else {
            [[sw, se, nw], [se, ne, nw]]
        })
    }

    /// The terrain height at world `(x, y)`, on the triangle of its polygon; `None` off the
    /// block.
    pub fn height_at(&self, x: f32, y: f32) -> Option<f32> {
        let polygons = self.side - 1;
        let (gx, gy) = self.grid_at(x, y)?;
        let px = (gx.floor() as usize).min(polygons - 1);
        let py = (gy.floor() as usize).min(polygons - 1);
        let (u, v) = (gx - px as f32, gy - py as f32);
        let h = |dx: usize, dy: usize| self.heights[(px + dx) * self.side + py + dy];
        let (sw, se, nw, ne) = (h(0, 0), h(1, 0), h(0, 1), h(1, 1));
        Some(if self.cuts[px * polygons + py] {
            // Triangles (NW, SW, NE) and (SW, SE, NE).
            if v >= u {
                sw + u * (ne - nw) + v * (nw - sw)
            } else {
                sw + u * (se - sw) + v * (ne - se)
            }
        } else if u + v <= 1.0 {
            // Triangles (SW, SE, NW) and (SE, NE, NW).
            sw + u * (se - sw) + v * (nw - sw)
        } else {
            ne + (1.0 - u) * (nw - ne) + (1.0 - v) * (se - ne)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TerrainHeights;

    fn field(cut: bool) -> TerrainHeights {
        // One polygon: SW 0, SE 10, NW 20, NE 40 (heights[x * side + y]).
        TerrainHeights {
            origin: [0.0, 0.0],
            spacing: 10.0,
            side: 2,
            heights: vec![0.0, 20.0, 10.0, 40.0],
            colours: vec![0; 4],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            cuts: vec![cut],
            water: None,
        }
    }

    #[test]
    fn corners_and_both_diagonals_interpolate_on_their_triangles() {
        for cut in [false, true] {
            let f = field(cut);
            assert_eq!(f.height_at(0.0, 0.0), Some(0.0));
            assert_eq!(f.height_at(10.0, 0.0), Some(10.0));
            assert_eq!(f.height_at(0.0, 10.0), Some(20.0));
            assert_eq!(f.height_at(10.0, 10.0), Some(40.0));
        }
        // SE-NW diagonal: the centre is the mean of SE and NW; SW-NE: the mean of SW and NE.
        assert_eq!(field(false).height_at(5.0, 5.0), Some(15.0));
        assert_eq!(field(true).height_at(5.0, 5.0), Some(20.0));
        assert_eq!(field(true).height_at(11.0, 5.0), None);
    }

    #[test]
    fn self_shadow_counts_the_polygon_corners_lit_by_the_shadow_bit() {
        // Vertices (x * side + y): SW lit by bit 0, SE and NW lit by bit 1 only, NE lit by bit 0.
        let mut f = field(false);
        f.colours = vec![1, 2, 2, 1];
        // Bit 0 lights SW and NE: half the corners, so the factor is 0.5.
        assert_eq!(f.self_shadow(5.0, 5.0, Some(0)), 0.5);
        // Bit 1 lights SE and NW: also half.
        assert_eq!(f.self_shadow(5.0, 5.0, Some(1)), 0.5);
        // Bit 2 lights no corner: black.
        assert_eq!(f.self_shadow(5.0, 5.0, Some(2)), 0.0);
        // No shadow bit, or off the block: nothing is shadowed.
        assert_eq!(f.self_shadow(5.0, 5.0, None), 1.0);
        assert_eq!(f.self_shadow(11.0, 5.0, Some(0)), 1.0);
    }
}
