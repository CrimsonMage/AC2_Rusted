use ac2_core::CellId;
use ac2_math::Vec3;

use crate::cell::WORLD_CELLS_PER_SIDE;
use crate::{land_cell_size, outdoor_cell_id_at_render_position};

/// Allocation-free retail `CLandCell::add_all_outside_cells` result for one moving sphere.
///
/// Retail admits only the containing land cell and its eight immediate neighbors. Membership
/// uses strict local-coordinate comparisons rather than circle/box intersection, and append order
/// is observable because `CTransition::check_other_cells` stops at the first non-OK result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutdoorCellCandidates {
    cells: [CellId; 9],
    len: u8,
}

impl OutdoorCellCandidates {
    const fn empty() -> Self {
        Self {
            cells: [CellId::INVALID; 9],
            len: 0,
        }
    }

    pub fn as_slice(&self) -> &[CellId] {
        &self.cells[..usize::from(self.len)]
    }

    fn push(&mut self, cell_id: Option<u32>) {
        let Some(cell_id) = cell_id.map(CellId::new) else {
            return;
        };
        if self.as_slice().contains(&cell_id) {
            return;
        }
        self.cells[usize::from(self.len)] = cell_id;
        self.len += 1;
    }
}

impl IntoIterator for OutdoorCellCandidates {
    type Item = CellId;
    type IntoIter = std::iter::Take<std::array::IntoIter<CellId, 9>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter().take(usize::from(self.len))
    }
}

pub fn outdoor_cell_ids_for_moving_sphere(center: Vec3, radius: f32) -> OutdoorCellCandidates {
    if !center.is_finite() || !radius.is_finite() || radius < 0.0 {
        return OutdoorCellCandidates::empty();
    }
    let Some(containing_cell) = outdoor_cell_id_at_render_position(center) else {
        return OutdoorCellCandidates::empty();
    };
    let Some((x, y)) = retail_gid_to_lcoord(containing_cell.raw()) else {
        return OutdoorCellCandidates::empty();
    };
    let local_x = center.x - (center.x / land_cell_size()).floor() * land_cell_size();
    let local_y = center.z - (center.z / land_cell_size()).floor() * land_cell_size();
    let maximum = land_cell_size() - radius;
    let minimum = radius;
    let mut cells = OutdoorCellCandidates::empty();
    cells.push(Some(containing_cell.raw()));
    if local_x > maximum {
        cells.push(retail_lcoord_to_gid(x + 1, y));
        if local_y > maximum {
            cells.push(retail_lcoord_to_gid(x + 1, y + 1));
        }
        if local_y < minimum {
            cells.push(retail_lcoord_to_gid(x + 1, y - 1));
        }
    }
    if local_x < minimum {
        cells.push(retail_lcoord_to_gid(x - 1, y));
        if local_y > maximum {
            cells.push(retail_lcoord_to_gid(x - 1, y + 1));
        }
        if local_y < minimum {
            cells.push(retail_lcoord_to_gid(x - 1, y - 1));
        }
    }
    if local_y > maximum {
        cells.push(retail_lcoord_to_gid(x, y + 1));
    }
    if local_y < minimum {
        cells.push(retail_lcoord_to_gid(x, y - 1));
    }
    cells
}

pub(crate) fn retail_adjust_to_outside_cell_id(cell_id: u32, x: f32, y: f32) -> Option<u32> {
    if !retail_valid_outside_adjust_source(cell_id) {
        return None;
    }

    let x = if x.abs() < 0.0002 { 0.0 } else { x };
    let y = if y.abs() < 0.0002 { 0.0 } else { y };
    let (block_x, block_y) = retail_block_lcoord(cell_id)?;
    let outside_x = block_x + (x / land_cell_size()).floor() as i32;
    let outside_y = block_y + (y / land_cell_size()).floor() as i32;
    retail_lcoord_to_gid(outside_x, outside_y)
}

fn retail_valid_outside_adjust_source(cell_id: u32) -> bool {
    let local = cell_id & 0x0000_FFFF;
    (1..=0x40).contains(&local) || (0x100..=0xFFFD).contains(&local) || local == 0xFFFF
}

pub(crate) fn retail_block_lcoord(block_id: u32) -> Option<(i32, i32)> {
    if block_id == 0 {
        return None;
    }

    let x = ((block_id >> 21) & 0x7F8) as i32;
    let y = (8 * ((block_id >> 16) & 0xFF)) as i32;
    ((0..WORLD_CELLS_PER_SIDE).contains(&x) && (0..WORLD_CELLS_PER_SIDE).contains(&y))
        .then_some((x, y))
}

fn retail_gid_to_lcoord(cell_id: u32) -> Option<(i32, i32)> {
    let (block_x, block_y) = retail_block_lcoord(cell_id)?;
    let local = (cell_id & 0xFFFF).checked_sub(1)?;
    if local >= 64 {
        return None;
    }
    Some((block_x + (local >> 3) as i32, block_y + (local & 7) as i32))
}

pub(crate) fn retail_lcoord_to_gid(x: i32, y: i32) -> Option<u32> {
    if !(0..WORLD_CELLS_PER_SIDE).contains(&x) || !(0..WORLD_CELLS_PER_SIDE).contains(&y) {
        return None;
    }

    Some(((((y >> 3) | (32 * (x & !7))) as u32) << 16) | ((y & 7) + 8 * (x & 7) + 1) as u32)
}

#[cfg(test)]
mod tests {
    use super::outdoor_cell_ids_for_moving_sphere;
    use ac2_math::Vec3;

    #[test]
    fn exact_tangency_keeps_only_the_containing_cell() {
        assert_eq!(
            outdoor_cell_ids_for_moving_sphere(Vec3::new(23.0, 0.0, 23.0), 1.0)
                .as_slice()
                .len(),
            1
        );
    }

    #[test]
    fn positive_corner_keeps_retail_append_order() {
        let cells = outdoor_cell_ids_for_moving_sphere(Vec3::new(19.25, 0.0, 19.25), 1.0);
        let expected = [
            super::retail_lcoord_to_gid(0, 0).unwrap(),
            super::retail_lcoord_to_gid(1, 0).unwrap(),
            super::retail_lcoord_to_gid(1, 1).unwrap(),
            super::retail_lcoord_to_gid(0, 1).unwrap(),
        ]
        .map(ac2_core::CellId::new);
        assert_eq!(cells.as_slice(), expected);
    }
}
