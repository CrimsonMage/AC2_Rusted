//! Retail `LandDefs` cell addressing for Asheron's Call 2 (`LandDefs::init_misc`,
//! `set_bitlengths`, `init_size` with the AC2 `LandscapeDefs`): 255 x 255 landblocks of 8 x 8
//! land cells of 20 m, two terrain polygons per land cell side.

use crate::fpu::{ld, st, F_EPSILON};
use crate::vector::Vector;

/// `LandDefs::square_length`.
pub(crate) const SQUARE_LENGTH: f32 = 20.0;
/// `LandDefs::block_length`.
pub(crate) const BLOCK_LENGTH: f32 = 160.0;
/// `LandDefs::lblock_side` (land cells per block side) and `lblock_shift`/`lblock_mask`.
pub(crate) const LBLOCK_SIDE: i32 = 8;
const LBLOCK_SHIFT: u32 = 3;
const LBLOCK_MASK: u32 = 7;
/// `LandDefs::vertex_per_cell`: terrain polygons per land cell side.
pub(crate) const VERTEX_PER_CELL: usize = 2;
/// `LandDefs::polys_per_landcell` (two triangles per terrain polygon).
pub(crate) const POLYS_PER_LANDCELL: usize = VERTEX_PER_CELL * VERTEX_PER_CELL * 2;
/// `LandDefs::side_vertex_count`.
pub(crate) const SIDE_VERTEX_COUNT: usize = 17;
/// `LandDefs::land_width`/`land_length` in land cells.
pub(crate) const LAND_WIDTH: i32 = 255 * LBLOCK_SIDE;
const FIRST_LCELL_ID: u32 = 1;
const LAST_LCELL_ID: u32 = 64;
const FIRST_ENVCELL_ID: u32 = 0x100;
const LAST_ENVCELL_ID: u32 = 0xFFFD;
const BLOCK_CELL_ID: u32 = 0xFFFF;
const CELLID_MASK: u32 = 0xFFFF;

/// Whether a cell id names a land cell (`(id & cellid_mask) < first_envcell_id`).
pub(crate) fn is_outside(cell_id: u32) -> bool {
    (cell_id & CELLID_MASK) < FIRST_ENVCELL_ID
}

/// `LandDefs::blockid_to_lcoord`: the land-cell coordinates of a block's south-west corner.
/// Retail leaves the outputs unwritten for id 0; that case maps to `(0, 0)` here.
pub(crate) fn blockid_to_lcoord(block_id: u32) -> Option<(i32, i32)> {
    if block_id == 0 {
        return None;
    }
    let x = (((block_id >> 16) & 0xFF00) >> 8) << LBLOCK_SHIFT;
    let y = ((block_id >> 16) & 0xFF) << LBLOCK_SHIFT;
    let (x, y) = (x as i32, y as i32);
    ((0..LAND_WIDTH).contains(&x) && (0..LAND_WIDTH).contains(&y)).then_some((x, y))
}

/// `LandDefs::gid_to_lcoord` for a land cell.
pub(crate) fn gid_to_lcoord(cell_id: u32) -> Option<(i32, i32)> {
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

/// `LandDefs::lcoord_to_gid`: 0 off the map.
pub(crate) fn lcoord_to_gid(x: i32, y: i32) -> u32 {
    if !(0..LAND_WIDTH).contains(&x) || !(0..LAND_WIDTH).contains(&y) {
        return 0;
    }
    let (x, y) = (x as u32, y as u32);
    let block = ((x >> LBLOCK_SHIFT) << 8) | (y >> LBLOCK_SHIFT);
    ((y & LBLOCK_MASK) + FIRST_LCELL_ID + ((x & LBLOCK_MASK) << LBLOCK_SHIFT)) | (block << 16)
}

/// `LandDefs::get_block_offset`: the offset from block `from`'s origin to block `to`'s, zero
/// for the same block.
pub(crate) fn get_block_offset(from: u32, to: u32) -> Vector {
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
pub(crate) fn adjust_to_outside(cell_id: &mut u32, point: &mut Vector) -> bool {
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
