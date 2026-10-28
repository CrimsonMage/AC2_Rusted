//! Physics material lookups on the resident terrain: `CLandBlock::GetPhysicsMaterial` (the
//! vertex nearest the body), `CLandBlock::GetWaterPhysicsMaterial` and `CEnvCell::GetPhysicsMaterialDID`,
//! on flat synthetic blocks.

use crate::frame::Frame;
use crate::landdefs::{lcoord_to_gid, SIDE_VERTEX_COUNT};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Quaternion, Vector};

const BLOCK: u32 = 0x7F7F_0000;

fn flat_terrain() -> Terrain {
    let mut terrain = Terrain::new();
    let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
    terrain.insert_heights(BLOCK, &heights, |_, _| false);
    terrain
}

/// A body at block-local (`x`, `y`) on the land cell there.
fn at(x: f32, y: f32) -> Position {
    let cell = lcoord_to_gid(0x7F * 8 + (x / 20.0) as i32, 0x7F * 8 + (y / 20.0) as i32);
    Position::new(
        cell,
        Frame::new(Vector::new(x, y, 0.0), Quaternion::IDENTITY),
    )
}

#[test]
fn land_material_is_the_vertex_nearest_the_body() {
    let mut terrain = flat_terrain();
    let mut grid = vec![Some(1); SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
    grid[8 * SIDE_VERTEX_COUNT + 8] = Some(9);
    terrain.set_block_materials(BLOCK, grid);
    assert_eq!(terrain.land_material(&at(81.0, 79.0)), Some(9));
    assert_eq!(terrain.land_material(&at(60.0, 60.0)), Some(1));
}

#[test]
fn a_block_without_a_material_grid_names_no_landscape_material() {
    let terrain = flat_terrain();
    assert_eq!(terrain.land_material(&at(81.0, 79.0)), None);
}

#[test]
fn water_material_belongs_to_a_block_that_has_water() {
    let mut terrain = flat_terrain();
    terrain.set_block_water_material(BLOCK, Some(55));
    assert_eq!(terrain.water_material(BLOCK | 0x0042), None);
    terrain.set_block_water(BLOCK, Some(1.0));
    assert_eq!(terrain.water_material(BLOCK | 0x0042), Some(55));
    terrain.set_block_water(BLOCK, None);
    assert_eq!(terrain.water_material(BLOCK | 0x0042), None);
}
