//! `LandDefs` landblock id arithmetic: block ids (`0xXXYYFFFF`), block-part words, landblock
//! coordinates and the bilinear corner interpolation of a landblock.

use crate::cell::WORLD_CELLS_PER_SIDE;
use crate::cell_query::retail_block_lcoord;
use crate::land_block_size;

/// `LandDefs::block_cell_id`: the low word of a landblock's own id.
const BLOCK_CELL_ID: u32 = 0xFFFF;
/// `LandDefs::blockid_mask`.
const BLOCKID_MASK: u32 = 0xFFFF_0000;
/// `LandDefs::block_part_shift`, `max_block_shift` and the x / y masks of a block part.
const BLOCK_PART_SHIFT: u32 = 16;
const MAX_BLOCK_SHIFT: u32 = 8;
const BLOCKX_MASK: u32 = 0xFF00;
const BLOCKY_MASK: u32 = 0x00FF;
/// `LandDefs::lblock_shift`: land cells per landblock side, as a shift.
const LBLOCK_SHIFT: u32 = 3;
/// `LandDefs::num_block_width` / `num_block_length` of the AC2 world.
const NUM_BLOCKS: i32 = WORLD_CELLS_PER_SIDE >> LBLOCK_SHIFT;

/// `LandDefs::inbound_valid_blockid`: a landblock's own id (`block_cell_id` in the low word) that
/// lies inside the world.
pub fn inbound_valid_blockid(block_id: u32) -> bool {
    let part = block_id >> BLOCK_PART_SHIFT;
    let x = (((part & BLOCKX_MASK) >> MAX_BLOCK_SHIFT) << LBLOCK_SHIFT) as i32;
    let y = ((part & BLOCKY_MASK) << LBLOCK_SHIFT) as i32;
    block_id == ((BLOCKID_MASK & block_id) | BLOCK_CELL_ID)
        && (0..WORLD_CELLS_PER_SIDE).contains(&x)
        && (0..WORLD_CELLS_PER_SIDE).contains(&y)
}

/// `LandDefs::blockpart_to_blockcoord`: the landblock x and y of a block-part word (the high word
/// of a cell id), `None` outside the world.
pub fn blockpart_to_blockcoord(block_part: u32) -> Option<(i32, i32)> {
    if block_part & BLOCKID_MASK != 0 {
        return None;
    }
    if !inbound_valid_blockid((block_part << BLOCK_PART_SHIFT) | BLOCK_CELL_ID) {
        return None;
    }
    let x = ((BLOCKX_MASK & block_part) >> MAX_BLOCK_SHIFT) as i32;
    let y = (BLOCKY_MASK & block_part) as i32;
    (x < NUM_BLOCKS && y < NUM_BLOCKS).then_some((x, y))
}

/// `LandDefs::get_block_gid`: the id of the landblock holding land-cell coordinates `(x, y)`,
/// `None` outside the world.
pub fn get_block_gid(x: i32, y: i32) -> Option<u32> {
    if !(0..WORLD_CELLS_PER_SIDE).contains(&x) || !(0..WORLD_CELLS_PER_SIDE).contains(&y) {
        return None;
    }
    Some(
        (((x >> LBLOCK_SHIFT) << MAX_BLOCK_SHIFT | y >> LBLOCK_SHIFT) as u32) << BLOCK_PART_SHIFT
            | BLOCK_CELL_ID,
    )
}

/// `LandDefs::GetBlockID(block_id, block_x_offset, block_y_offset)`: the landblock that many
/// blocks away, `None` off the world or for an id that is no landblock.
pub fn block_id_at_offset(block_id: u32, block_x_offset: i32, block_y_offset: i32) -> Option<u32> {
    let (x, y) = retail_block_lcoord(block_id)?;
    let side = 1 << LBLOCK_SHIFT;
    get_block_gid(x + side * block_x_offset, y + side * block_y_offset)
}

/// `LandDefs::within_block`: whether landblock-local `(x, y)` lies in `[0, block_length)`.
pub fn within_block(x: f32, y: f32) -> bool {
    0.0 <= x && 0.0 <= y && x < land_block_size() && y < land_block_size()
}

/// `LandDefs::Interpolate`: the bilinear blend of a landblock's corner values at landblock-local
/// `(x, y)`, clamped to the block. `se_val` is the corner at +x, `nw_val` at +y and `ne_val` at
/// both; `px` (the origin corner) is the south-west value.
pub fn interpolate(x: f32, y: f32, px: f32, se_val: f32, nw_val: f32, ne_val: f32) -> f32 {
    let inverse = 1.0 / f64::from(land_block_size());
    let fx = f64::from((inverse * f64::from(x)) as f32).clamp(0.0, 1.0);
    let fy = f64::from((inverse * f64::from(y)) as f32).clamp(0.0, 1.0);
    let (px, se, nw, ne) = (
        f64::from(px),
        f64::from(se_val),
        f64::from(nw_val),
        f64::from(ne_val),
    );
    ((fy * ne + (1.0 - fy) * se) * fx + (1.0 - fx) * (fy * nw + px * (1.0 - fy))) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_query::retail_lcoord_to_gid as lcoord_to_gid;

    #[test]
    fn block_ids_round_trip_through_block_parts() {
        let id = get_block_gid(8 * 0xA9 + 3, 8 * 0xB2 + 5).unwrap();
        assert_eq!(id, 0xA9B2_FFFF);
        assert!(inbound_valid_blockid(id));
        assert_eq!(blockpart_to_blockcoord(id >> 16), Some((0xA9, 0xB2)));
        // The land cell ids of that block share its block part.
        assert_eq!(lcoord_to_gid(8 * 0xA9, 8 * 0xB2).unwrap() >> 16, 0xA9B2);
    }

    #[test]
    fn invalid_blocks_are_rejected() {
        assert!(!inbound_valid_blockid(0xA9B2_0001));
        assert!(!inbound_valid_blockid(0xFFFF_FFFF)); // block 255 is outside the 255 x 255 world
        assert_eq!(get_block_gid(-1, 0), None);
        assert_eq!(get_block_gid(0, WORLD_CELLS_PER_SIDE), None);
        assert_eq!(blockpart_to_blockcoord(0x1_0000), None);
        assert_eq!(blockpart_to_blockcoord(0xFF00), None);
    }

    #[test]
    fn neighbour_blocks_step_by_block() {
        assert_eq!(block_id_at_offset(0xA9B2_FFFF, 1, -1), Some(0xAAB1_FFFF));
        assert_eq!(block_id_at_offset(0x0000_FFFF, -1, 0), None);
        // blockid_to_lcoord reads only the block part, so a cell id names its block too.
        assert_eq!(block_id_at_offset(0xA9B2_0001, 0, 0), Some(0xA9B2_FFFF));
    }

    #[test]
    fn within_block_is_half_open() {
        let side = land_block_size();
        assert!(within_block(0.0, side - 0.01));
        assert!(!within_block(side, 0.0));
        assert!(!within_block(-0.01, 5.0));
    }

    #[test]
    fn interpolate_blends_corners_and_clamps() {
        let side = land_block_size();
        assert_eq!(interpolate(0.0, 0.0, 1.0, 2.0, 3.0, 4.0), 1.0);
        assert_eq!(interpolate(side, 0.0, 1.0, 2.0, 3.0, 4.0), 2.0);
        assert_eq!(interpolate(0.0, side, 1.0, 2.0, 3.0, 4.0), 3.0);
        assert_eq!(interpolate(side, side, 1.0, 2.0, 3.0, 4.0), 4.0);
        assert_eq!(interpolate(side * 3.0, -5.0, 1.0, 2.0, 3.0, 4.0), 2.0);
        assert!((interpolate(side / 2.0, side / 2.0, 1.0, 2.0, 3.0, 4.0) - 2.5).abs() < 1e-6);
    }
}
