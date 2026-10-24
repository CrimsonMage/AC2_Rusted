//! The resident world for collision: the land cells of loaded landblocks with their terrain
//! polygons (`CLandBlockMesh::ConstructVertices` + `ConstructPolygons`) and the static objects
//! shadowing them (`CLandBlock::CreateStaticObjects`, `CPhysicsObj::add_shadows_to_cells`).

use std::collections::HashMap;

use ac2_core::CellId;
use ac2_world::{terrain_polygon_cut, LandBlockDesc};

use crate::cells::{CellSource, LandCell, TerrainPolygon};
use crate::envcell::EnvCell;
use crate::find_cells::shadow_cells;
use crate::fpu::{ld, st};
use crate::landdefs::{
    adjust_to_outside, blockid_to_lcoord, is_outside, lcoord_to_gid, BLOCK_LENGTH, LBLOCK_SIDE,
    POLYS_PER_LANDCELL, SIDE_VERTEX_COUNT, VERTEX_PER_CELL,
};
use crate::objects::StaticObject;
use crate::position::Position;
use crate::vector::Vector;

/// Terrain polygons per block side.
const SIDE_POLYGON_COUNT: usize = SIDE_VERTEX_COUNT - 1;

/// The resident landblocks' land cells and interior cells, keyed by cell id, and their static
/// objects.
#[derive(Debug, Clone, Default)]
pub struct Terrain {
    cells: HashMap<u32, LandCell>,
    env_cells: HashMap<u32, EnvCell>,
    /// `CLandBlockData::m_waterPlane` heights of the blocks that have water.
    water: HashMap<u32, f32>,
    /// Static objects by id (a removed object leaves `None`) and the land cells their shadow box
    /// covers.
    objects: Vec<Option<(StaticObject, Vec<u32>)>>,
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
                let objects = self.shadowing(cell_id);
                self.cells.insert(
                    cell_id,
                    LandCell {
                        cell_id,
                        polygons,
                        objects,
                        touching: Vec::new(),
                    },
                );
            }
        }
    }

    /// Drops a landblock's cells (land and interior) and the static objects it placed.
    pub fn remove_block(&mut self, block_cell_id: u32) {
        let block = block_cell_id & 0xFFFF_0000;
        self.cells.retain(|id, _| id & 0xFFFF_0000 != block);
        self.env_cells.retain(|id, _| id & 0xFFFF_0000 != block);
        self.water.remove(&block);
        for id in 0..self.objects.len() as u32 {
            let owned = self.objects[id as usize]
                .as_ref()
                .is_some_and(|(object, _)| object.position().cell_id & 0xFFFF_0000 == block);
            if owned {
                self.remove_object(id);
            }
        }
    }

    /// Adds a static object and enters it in the shadow lists of the resident cells its shadow
    /// box touches (`CPhysicsObj::calc_cross_cells` -> `add_shadows_to_cells`); cells made
    /// resident later pick it up too. Returns its id.
    pub fn insert_object(&mut self, object: StaticObject) -> u32 {
        let id = self.objects.len() as u32;
        let cells = shadow_cells(&object.position(), object.shadow_box(), self);
        for cell_id in &cells {
            if let Some(cell) = self.cells.get_mut(cell_id) {
                cell.objects.push(id);
            }
            if let Some(cell) = self.env_cells.get_mut(cell_id) {
                cell.objects.push(id);
            }
        }
        self.objects.push(Some((object, cells)));
        id
    }

    /// Makes an interior cell resident (objects already shadowing it join its list).
    pub fn insert_env_cell(&mut self, mut cell: EnvCell) {
        cell.objects = self.shadowing(cell.cell_id);
        self.env_cells.insert(cell.cell_id, cell);
    }

    /// `CLandBlock::InitCellConnections`: every resident interior cell of the block's outside stab
    /// list that reaches the outside becomes a touching cell of the block's land cells under its
    /// physics tree's bounding sphere.
    pub fn connect_block(&mut self, block_cell_id: u32, outside_stab_list: &[u32]) {
        let block = block_cell_id & 0xFFFF_0000;
        let mut links = Vec::new();
        for &cell_id in outside_stab_list {
            let Some(cell) = self.env_cells.get(&cell_id) else {
                continue;
            };
            let Some(sphere) = cell
                .structure
                .physics
                .as_ref()
                .and_then(|tree| tree.bounding_sphere())
            else {
                continue;
            };
            if !cell.touches_outside() {
                continue;
            }
            let mut position = cell.position;
            if !adjust_to_outside(&mut position.cell_id, &mut position.frame.origin) {
                continue;
            }
            let local = crate::find_cells::sphere_bounds(&[sphere]);
            for land in shadow_cells(&position, local, self) {
                if is_outside(land) && land & 0xFFFF_0000 == block && self.cells.contains_key(&land)
                {
                    links.push((land, cell_id));
                }
            }
        }
        for (land, cell_id) in links {
            if let Some(cell) = self.cells.get_mut(&land) {
                cell.touching.push(cell_id);
            }
        }
    }

    /// Makes the landblocks of `other` resident, as if they had been loaded into this world: a
    /// block prepared apart (on a loader thread) joins here. Its land and interior cells pick up
    /// the resident objects already shadowing them, then its static objects are entered in the
    /// shadow lists of every cell now resident (`CPhysicsObj::add_shadows_to_cells`), in their
    /// original order.
    pub fn absorb(&mut self, other: Terrain) {
        let Terrain {
            cells,
            env_cells,
            water,
            objects,
        } = other;
        let mut cells: Vec<_> = cells.into_iter().collect();
        cells.sort_by_key(|(id, _)| *id);
        for (id, mut cell) in cells {
            cell.objects = self.shadowing(id);
            self.cells.insert(id, cell);
        }
        let mut env_cells: Vec<_> = env_cells.into_iter().collect();
        env_cells.sort_by_key(|(id, _)| *id);
        for (id, mut cell) in env_cells {
            cell.objects = self.shadowing(id);
            self.env_cells.insert(id, cell);
        }
        self.water.extend(water);
        for (object, _) in objects.into_iter().flatten() {
            self.insert_object(object);
        }
    }

    /// Removes a static object from the world and its cells' shadow lists.
    pub fn remove_object(&mut self, id: u32) {
        let Some(Some((_, cells))) = self.objects.get_mut(id as usize).map(Option::take) else {
            return;
        };
        for cell_id in cells {
            if let Some(cell) = self.cells.get_mut(&cell_id) {
                cell.objects.retain(|&other| other != id);
            }
        }
    }

    /// The static objects whose shadow box covers `cell_id`, in insertion order.
    fn shadowing(&self, cell_id: u32) -> Vec<u32> {
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry
                    .as_ref()
                    .is_some_and(|(_, cells)| cells.contains(&cell_id))
            })
            .map(|(id, _)| id as u32)
            .collect()
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
        if !is_outside(position.cell_id) {
            // Interiors: the downward `LOSCheck` would end on the cell's floor, which is not
            // ported; an interior body counts as on its floor.
            return None;
        }
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

    fn env_cell(&self, cell_id: u32) -> Option<&EnvCell> {
        self.env_cells.get(&cell_id)
    }

    fn object(&self, id: u32) -> Option<&StaticObject> {
        self.objects
            .get(id as usize)?
            .as_ref()
            .map(|(object, _)| object)
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
    fn an_absorbed_block_matches_one_loaded_in_place() {
        use crate::objects::{ObjectShape, StaticObject};
        use crate::shapes::cylsphere::CylSphere;
        use std::sync::Arc;
        let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
        let pillar = Arc::new(ObjectShape::new(
            Vec::new(),
            vec![CylSphere {
                low: Vector::ZERO,
                height: 4.0,
                radius: 1.0,
            }],
            Vec::new(),
        ));
        // A pillar on the east edge of block 7F7F shadows cells of 807F too.
        let object = || {
            StaticObject::new(
                Position::new(
                    0x7F7F_0039,
                    Frame::new(Vector::new(159.5, 10.0, 0.0), Quaternion::IDENTITY),
                ),
                Vector::new(1.0, 1.0, 1.0),
                Arc::clone(&pillar),
            )
        };
        let mut in_place = Terrain::new();
        in_place.insert_heights(0x7F7F_FFFF, &heights, |_, _| false);
        in_place.insert_object(object());
        in_place.insert_heights(0x807F_FFFF, &heights, |_, _| false);

        let mut absorbed = Terrain::new();
        absorbed.insert_heights(0x807F_FFFF, &heights, |_, _| false);
        let mut prepared = Terrain::new();
        prepared.insert_heights(0x7F7F_FFFF, &heights, |_, _| false);
        prepared.insert_object(object());
        absorbed.absorb(prepared);

        for cell in [0x7F7F_0039, 0x807F_0001] {
            let a = in_place.land_cell(cell).expect("cell").objects.clone();
            let b = absorbed.land_cell(cell).expect("cell").objects.clone();
            assert_eq!(a, b, "cell {cell:08x}");
            assert_eq!(a, vec![0], "cell {cell:08x}");
        }
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
