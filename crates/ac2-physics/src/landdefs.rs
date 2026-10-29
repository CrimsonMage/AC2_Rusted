//! Retail `LandDefs` cell addressing for Asheron's Call 2 (`LandDefs::init_misc`,
//! `set_bitlengths`, `init_size` with the AC2 `LandscapeDefs`): 255 x 255 landblocks of 8 x 8
//! land cells of 20 m, two terrain polygons per land cell side.

use crate::fpu::{ld, st, F_EPSILON};
use crate::vector::Vector;

/// `LandDefs::square_length`.
pub const SQUARE_LENGTH: f32 = 20.0;
/// `LandDefs::block_length`.
pub const BLOCK_LENGTH: f32 = 160.0;
/// `LandDefs::lblock_side` (land cells per block side) and `lblock_shift`/`lblock_mask`.
pub const LBLOCK_SIDE: i32 = 8;
const LBLOCK_SHIFT: u32 = 3;
const LBLOCK_MASK: u32 = 7;
/// `LandDefs::vertex_per_cell`: terrain polygons per land cell side.
pub(crate) const VERTEX_PER_CELL: usize = 2;
/// `LandDefs::polys_per_landcell` (two triangles per terrain polygon).
pub(crate) const POLYS_PER_LANDCELL: usize = VERTEX_PER_CELL * VERTEX_PER_CELL * 2;
/// `LandDefs::side_vertex_count`.
pub(crate) const SIDE_VERTEX_COUNT: usize = 17;
/// `LandDefs::land_width`/`land_length` in land cells.
pub const LAND_WIDTH: i32 = 255 * LBLOCK_SIDE;
const FIRST_LCELL_ID: u32 = 1;
const LAST_LCELL_ID: u32 = 64;
const FIRST_ENVCELL_ID: u32 = 0x100;
const LAST_ENVCELL_ID: u32 = 0xFFFD;
const BLOCK_CELL_ID: u32 = 0xFFFF;
const CELLID_MASK: u32 = 0xFFFF;

/// Whether a cell id names a land cell (`(id & cellid_mask) < first_envcell_id`).
pub fn is_outside(cell_id: u32) -> bool {
    (cell_id & CELLID_MASK) < FIRST_ENVCELL_ID
}

/// `LandDefs::blockid_to_lcoord`: the land-cell coordinates of a block's south-west corner.
/// Retail leaves the outputs unwritten for id 0; that case maps to `(0, 0)` here.
pub fn blockid_to_lcoord(block_id: u32) -> Option<(i32, i32)> {
    if block_id == 0 {
        return None;
    }
    let x = (((block_id >> 16) & 0xFF00) >> 8) << LBLOCK_SHIFT;
    let y = ((block_id >> 16) & 0xFF) << LBLOCK_SHIFT;
    let (x, y) = (x as i32, y as i32);
    ((0..LAND_WIDTH).contains(&x) && (0..LAND_WIDTH).contains(&y)).then_some((x, y))
}

/// `LandDefs::gid_to_lcoord` for a land cell.
pub fn gid_to_lcoord(cell_id: u32) -> Option<(i32, i32)> {
    let local = cell_id & CELLID_MASK;
    if !(FIRST_LCELL_ID..=LAST_LCELL_ID).contains(&local) || (cell_id >> 16) == 0 {
        return None;
    }
    let block_x = (((cell_id >> 16) & 0xFF00) >> 8) << LBLOCK_SHIFT;
    let block_y = ((cell_id >> 16) & 0xFF) << LBLOCK_SHIFT;
    let x = (block_x + ((local - FIRST_LCELL_ID) >> LBLOCK_SHIFT)) as i32;
    let y = (block_y + ((local - FIRST_LCELL_ID) & LBLOCK_MASK)) as i32;
    ((0..LAND_WIDTH).contains(&x) && (0..LAND_WIDTH).contains(&y)).then_some((x, y))
}

/// `LandDefs::gid_to_game_map_coords`: a land cell's map coordinates, -100..100 east and north
/// of the map's middle (`(coord - half) / half * 100` in the FPU, stored as floats); `None`
/// inside.
pub fn gid_to_game_map_coords(cell_id: u32) -> Option<(f32, f32)> {
    let (x, y) = gid_to_lcoord(cell_id)?;
    let half = f64::from(LAND_WIDTH) * 0.5;
    let map = |v: i32| ((f64::from(v) - half) / half * 100.0) as f32;
    Some((map(x), map(y)))
}

/// `LandDefs::game_map_coords_to_gid`: the land cell at map coordinates (east, north percent of
/// the half map); 0 off the map.
pub fn game_map_coords_to_gid(x: f32, y: f32) -> u32 {
    let w = f64::from(LAND_WIDTH);
    let to_cell = |v: f32| (ld(v) * 0.01 * w * 0.5 + w * 0.5 + 0.5).floor() as i64 as i32;
    lcoord_to_gid(to_cell(x), to_cell(y))
}

/// `LandDefs::lcoord_to_gid`: 0 off the map.
pub fn lcoord_to_gid(x: i32, y: i32) -> u32 {
    if !(0..LAND_WIDTH).contains(&x) || !(0..LAND_WIDTH).contains(&y) {
        return 0;
    }
    let (x, y) = (x as u32, y as u32);
    let block = ((x >> LBLOCK_SHIFT) << 8) | (y >> LBLOCK_SHIFT);
    ((y & LBLOCK_MASK) + FIRST_LCELL_ID + ((x & LBLOCK_MASK) << LBLOCK_SHIFT)) | (block << 16)
}

/// `LandDefs::pixel_length`: metres between terrain vertices.
const PIXEL_LENGTH: f32 = SQUARE_LENGTH / VERTEX_PER_CELL as f32;

/// Which way the nearest vertex lies from the point (`DirectionEnum` `*_OF_VIEWER`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelDirection {
    SouthWest,
    SouthEast,
    NorthWest,
    NorthEast,
}

/// `PixelInfo`: the terrain vertex nearest a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelInfo {
    /// Vertex coordinates within the block (`m_lx`, `m_ly`).
    pub local_x: u32,
    pub local_y: u32,
    /// Vertex coordinates on the whole map (`m_gx`, `m_gy`).
    pub global_x: u32,
    pub global_y: u32,
    pub direction: PixelDirection,
    /// The vertex in the block frame (`m_loc`, z 0).
    pub location: Vector,
    /// The land cell holding the vertex (`m_cid`; 0 off the map).
    pub cell_id: u32,
}

/// `LandDefs::GetNearestPixel`: the vertex nearest `loc` (in block `block_id`'s frame). A
/// coordinate at or under half a pixel past a vertex rounds down to it, else up.
pub fn get_nearest_pixel(block_id: u32, loc: Vector) -> Option<PixelInfo> {
    let (cell_x, cell_y) = blockid_to_lcoord(block_id)?;
    let per_cell = VERTEX_PER_CELL as u32;
    let base_x = per_cell.wrapping_mul(cell_x as u32);
    let base_y = per_cell.wrapping_mul(cell_y as u32);
    let half = st(ld(PIXEL_LENGTH) * 0.5);
    let floor = |v: f32| (ld(v) / ld(PIXEL_LENGTH)).floor() as i32 as u32;
    let (lx, ly) = (floor(loc.x), floor(loc.y));
    let past = |v: f32, n: u32| ld(v) - ld(n as i32 as f32) * ld(PIXEL_LENGTH) > ld(half);
    let east = past(loc.x, lx);
    let north = past(loc.y, ly);
    let (lx, ly) = (
        lx.wrapping_add(u32::from(east)),
        ly.wrapping_add(u32::from(north)),
    );
    let direction = match (east, north) {
        (false, false) => PixelDirection::SouthWest,
        (false, true) => PixelDirection::NorthWest,
        (true, false) => PixelDirection::SouthEast,
        (true, true) => PixelDirection::NorthEast,
    };
    let global_x = base_x.wrapping_add(lx);
    let global_y = base_y.wrapping_add(ly);
    let coord = |v: u32| st(ld(v as i32 as f32) * ld(PIXEL_LENGTH));
    let to_cell = |v: u32| (f64::from(v as i32) / f64::from(per_cell)) as i32;
    Some(PixelInfo {
        local_x: lx,
        local_y: ly,
        global_x,
        global_y,
        direction,
        location: Vector::new(coord(lx), coord(ly), 0.0),
        cell_id: lcoord_to_gid(to_cell(global_x), to_cell(global_y)),
    })
}

/// `LandDefs::get_block_offset`: the offset from block `from`'s origin to block `to`'s, zero
/// for the same block.
pub fn get_block_offset(from: u32, to: u32) -> Vector {
    if (from == 0 && to == 0) || (from != 0 && to != 0 && from >> 16 == to >> 16) {
        return Vector::ZERO;
    }
    let (fx, fy) = blockid_to_lcoord(from).unwrap_or((0, 0));
    let (tx, ty) = blockid_to_lcoord(to).unwrap_or((0, 0));
    Vector::new(
        st(f64::from(tx - fx) * ld(SQUARE_LENGTH)),
        st(f64::from(ty - fy) * ld(SQUARE_LENGTH)),
        0.0,
    )
}

/// `LandDefs::get_outside_lcoord`.
fn get_outside_lcoord(cell_id: u32, local: Vector) -> Option<(i32, i32)> {
    let id = cell_id & CELLID_MASK;
    let valid = (FIRST_LCELL_ID..=LAST_LCELL_ID).contains(&id)
        || (FIRST_ENVCELL_ID..=LAST_ENVCELL_ID).contains(&id)
        || id == BLOCK_CELL_ID;
    if !valid {
        return None;
    }
    let (bx, by) = blockid_to_lcoord(cell_id)?;
    let x = bx + (ld(local.x) / ld(SQUARE_LENGTH)).floor() as i32;
    let y = by + (ld(local.y) / ld(SQUARE_LENGTH)).floor() as i32;
    ((0..LAND_WIDTH).contains(&x) && (0..LAND_WIDTH).contains(&y)).then_some((x, y))
}

/// `LandDefs::adjust_to_outside`: moves a block-relative point into the land cell under it,
/// rebasing the point onto that cell's block. Components within `F_EPSILON` of zero snap to
/// zero first. Fails (cell id 0) off the map or for an invalid id.
pub fn adjust_to_outside(cell_id: &mut u32, point: &mut Vector) -> bool {
    let id = *cell_id & CELLID_MASK;
    let valid = (FIRST_LCELL_ID..=LAST_LCELL_ID).contains(&id)
        || (FIRST_ENVCELL_ID..=LAST_ENVCELL_ID).contains(&id)
        || id == BLOCK_CELL_ID;
    if valid {
        if ld(point.x).abs() < ld(F_EPSILON) {
            point.x = 0.0;
        }
        if ld(point.y).abs() < ld(F_EPSILON) {
            point.y = 0.0;
        }
        if let Some((x, y)) = get_outside_lcoord(*cell_id, *point) {
            *cell_id = lcoord_to_gid(x, y);
            let blocks_x = (ld(point.x) / ld(BLOCK_LENGTH)).floor();
            point.x = st(ld(point.x) - blocks_x * ld(BLOCK_LENGTH));
            let blocks_y = (ld(point.y) / ld(BLOCK_LENGTH)).floor();
            point.y = st(ld(point.y) - blocks_y * ld(BLOCK_LENGTH));
            return true;
        }
    }
    *cell_id = 0;
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn nearest_pixel_rounds_to_the_closest_vertex_with_ties_down() {
        let block = 0x7F7F_FFFF;
        let at = |x: f32, y: f32| get_nearest_pixel(block, Vector::new(x, y, 5.0)).unwrap();
        let p = at(12.0, 3.0);
        assert_eq!((p.local_x, p.local_y), (1, 0));
        assert_eq!(p.direction, PixelDirection::SouthWest);
        assert_eq!(p.location, Vector::new(10.0, 0.0, 0.0));
        // Exactly half a pixel past a vertex still rounds down.
        let p = at(15.0, 15.0);
        assert_eq!((p.local_x, p.local_y), (1, 1));
        let p = at(15.1, 15.1);
        assert_eq!((p.local_x, p.local_y), (2, 2));
        assert_eq!(p.direction, PixelDirection::NorthEast);
        assert_eq!(p.location, Vector::new(20.0, 20.0, 0.0));
        // The map coordinates add the block's corner: block 0x7F7F is land cell 1016 x 1016.
        assert_eq!((p.global_x, p.global_y), (2 * 1016 + 2, 2 * 1016 + 2));
        assert_eq!(p.cell_id, lcoord_to_gid(1017, 1017));
        assert!(get_nearest_pixel(0, Vector::ZERO).is_none());
    }

    use super::*;

    #[test]
    fn land_cell_ids_round_trip() {
        let gid = lcoord_to_gid(0x7F * 8 + 3, 0x7F * 8 + 5);
        assert_eq!(gid, 0x7F7F_001E);
        assert_eq!(gid_to_lcoord(gid), Some((0x7F * 8 + 3, 0x7F * 8 + 5)));
        assert_eq!(lcoord_to_gid(-1, 0), 0);
        assert_eq!(gid_to_lcoord(0x7F7F_0100), None);
    }

    #[test]
    fn block_offsets_are_whole_blocks() {
        assert_eq!(get_block_offset(0x7F7F_0001, 0x7F7F_0040), Vector::ZERO);
        let east = get_block_offset(0x7F7F_0001, 0x807F_0001);
        assert_eq!(east, Vector::new(160.0, 0.0, 0.0));
        let south_west = get_block_offset(0x7F7F_0001, 0x7E7E_0001);
        assert_eq!(south_west, Vector::new(-160.0, -160.0, 0.0));
    }

    #[test]
    fn adjust_to_outside_crosses_into_the_next_block() {
        let mut cell = 0x7F7F_0001;
        let mut point = Vector::new(165.0, 30.0, 4.0);
        assert!(adjust_to_outside(&mut cell, &mut point));
        assert_eq!(cell, lcoord_to_gid(0x80 * 8, 0x7F * 8 + 1));
        assert_eq!(point, Vector::new(5.0, 30.0, 4.0));
        let mut edge = Vector::new(-0.0001, 0.0, 0.0);
        let mut cell = 0x7F7F_0001;
        assert!(adjust_to_outside(&mut cell, &mut edge));
        assert_eq!(cell, 0x7F7F_0001);
    }
}
