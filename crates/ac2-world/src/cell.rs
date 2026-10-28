//! Cell ids: a landblock in the high word, a land cell (`0x0001`-`0x0040`) or an env cell
//! (`0x0100` and up) in the low word.

use ac2_core::CellId;
use ac2_math::{BBox, Vec3};

use crate::cell_query::{retail_adjust_to_outside_cell_id, retail_lcoord_to_gid};
use crate::land_cell_size;

/// Land cells per landblock side.
pub const LAND_BLOCK_CELLS_PER_SIDE: usize = 8;
/// Land cells per world side (255 blocks of 8 cells).
pub(crate) const WORLD_CELLS_PER_SIDE: i32 = 2040;
const BLOCK_PREFIX_MASK: u32 = 0xFFFF_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Invalid,
    Outdoor,
    Interior,
}

pub const fn classify_cell(cell_id: CellId) -> CellKind {
    if !cell_id.is_valid() {
        return CellKind::Invalid;
    }
    if (cell_id.raw() & 0xFFFF) < 0x100 {
        CellKind::Outdoor
    } else {
        CellKind::Interior
    }
}

/// The land cell under a landblock-local AC position (`LandDefs::adjust_to_outside`).
pub fn adjust_to_outside_cell_id(cell_id: CellId, origin: Vec3) -> Option<CellId> {
    retail_adjust_to_outside_cell_id(cell_id.raw(), origin.x, origin.y).map(CellId::new)
}

/// The land cell under a global render-axes position.
pub fn outdoor_cell_id_at_render_position(position: Vec3) -> Option<CellId> {
    if !position.is_finite() || position.x < 0.0 || position.z < 0.0 {
        return None;
    }
    let cell_x = (position.x / land_cell_size()).floor() as i32;
    let cell_y = (position.z / land_cell_size()).floor() as i32;
    retail_lcoord_to_gid(cell_x, cell_y).map(CellId::new)
}

/// `CEnvCell::CullingPlaneCheck`: whether none of the eight corners of `bbox` (`BBox::GetPoints`)
/// lies within `culling_plane_dist` of `camera`; any corner that close fails the check.
pub fn culling_plane_check(camera: Vec3, culling_plane_dist: f32, bbox: BBox) -> bool {
    let limit = culling_plane_dist * culling_plane_dist;
    !bbox
        .points()
        .iter()
        .any(|point| (camera - *point).length_squared() < limit)
}

pub fn same_outdoor_block(left: CellId, right: CellId) -> bool {
    classify_cell(left) == CellKind::Outdoor
        && classify_cell(right) == CellKind::Outdoor
        && (left.raw() & BLOCK_PREFIX_MASK) == (right.raw() & BLOCK_PREFIX_MASK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn culling_plane_check_fails_when_any_corner_is_near_the_camera() {
        let bbox = BBox::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(20.0, 20.0, 20.0));
        assert!(culling_plane_check(Vec3::ZERO, 5.0, bbox));
        // The nearest corner (10, 10, 10) is 17.3 away; a 18 m plane reaches it.
        assert!(!culling_plane_check(Vec3::ZERO, 18.0, bbox));
        // Exactly on the plane is not within it.
        assert!(culling_plane_check(Vec3::new(5.0, 10.0, 10.0), 5.0, bbox));
    }
}
