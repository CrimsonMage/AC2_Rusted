//! Conversions between cell-local positions and global positions.
//!
//! A position is stored relative to the landblock named by the high word of its cell id, for
//! land cells and env cells alike (`Position::frame` against `objcell_id`). AC axes are z-up;
//! render axes are y-up with the AC y axis on render z.

use ac2_core::CellId;
use ac2_math::Vec3;

use crate::{classify_cell, land_block_size, CellKind};

/// Metric origin of the landblock named by `cell_id`'s high word, or `None` for an invalid cell.
fn block_origin(cell_id: CellId) -> Option<[f32; 2]> {
    (classify_cell(cell_id) != CellKind::Invalid).then(|| {
        let raw = cell_id.raw();
        [
            f32::from(((raw >> 24) & 0xFF) as u8) * land_block_size(),
            f32::from(((raw >> 16) & 0xFF) as u8) * land_block_size(),
        ]
    })
}

/// Integer landblock delta from `from` to `to`, in metres.
fn block_delta(from: CellId, to: CellId) -> [f32; 2] {
    let x = |cell: CellId| ((cell.raw() >> 24) & 0xFF) as i32;
    let y = |cell: CellId| ((cell.raw() >> 16) & 0xFF) as i32;
    [
        (x(to) - x(from)) as f32 * land_block_size(),
        (y(to) - y(from)) as f32 * land_block_size(),
    ]
}

/// `Position::get_offset`: the AC-axis offset from one cell-local position to another, taken
/// from the integer landblock delta first so distant positions never cancel in `f32`.
pub fn retail_position_offset(
    from_cell: CellId,
    from_origin: [f32; 3],
    to_cell: CellId,
    to_origin: [f32; 3],
) -> Option<[f32; 3]> {
    if classify_cell(from_cell) == CellKind::Invalid || classify_cell(to_cell) == CellKind::Invalid
    {
        return None;
    }
    let [block_x, block_y] = block_delta(from_cell, to_cell);
    Some([
        (f64::from(block_x) + f64::from(to_origin[0]) - f64::from(from_origin[0])) as f32,
        (f64::from(block_y) + f64::from(to_origin[1]) - f64::from(from_origin[1])) as f32,
        (f64::from(to_origin[2]) - f64::from(from_origin[2])) as f32,
    ])
}

/// Converts a cell-local AC position into the global render basis.
pub fn world_render_position_from_cell_local(
    cell_id: CellId,
    local_origin_ac: Vec3,
) -> Option<Vec3> {
    let [x, y] = block_origin(cell_id)?;
    Some(Vec3::new(
        x + local_origin_ac.x,
        local_origin_ac.z,
        y + local_origin_ac.y,
    ))
}

/// Converts a cell-local AC position into a render position relative to another cell's
/// landblock origin, using the integer block delta so nearby positions stay exact.
pub fn cell_local_render_position_relative_to_landblock(
    source_cell_id: CellId,
    local_origin_ac: Vec3,
    render_origin_cell_id: CellId,
) -> Option<Vec3> {
    if classify_cell(source_cell_id) == CellKind::Invalid
        || classify_cell(render_origin_cell_id) == CellKind::Invalid
        || !local_origin_ac.is_finite()
    {
        return None;
    }
    let [block_x, block_y] = block_delta(render_origin_cell_id, source_cell_id);
    Some(Vec3::new(
        block_x + local_origin_ac.x,
        local_origin_ac.z,
        block_y + local_origin_ac.y,
    ))
}

/// Localizes a global AC-axis origin against the landblock encoded by `cell_id`.
pub fn cell_local_origin_from_world_ac_position(
    cell_id: CellId,
    world_origin_ac: Vec3,
) -> Option<Vec3> {
    if !world_origin_ac.is_finite() {
        return None;
    }
    let [x, y] = block_origin(cell_id)?;
    Some(Vec3::new(
        world_origin_ac.x - x,
        world_origin_ac.y - y,
        world_origin_ac.z,
    ))
}

/// Inverse of [`world_render_position_from_cell_local`]. The selected cell is an input, so an
/// env cell identity is never derived from coordinates.
pub fn cell_local_origin_from_world_render_position(
    cell_id: CellId,
    world_origin_render: Vec3,
) -> Option<Vec3> {
    if !world_origin_render.is_finite() {
        return None;
    }
    let [x, y] = block_origin(cell_id)?;
    Some(Vec3::new(
        world_origin_render.x - x,
        world_origin_render.z - y,
        world_origin_render.y,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_local_and_world_render_positions_round_trip_for_land_and_env_cells() {
        for cell in [CellId::new(0x7D64_0014), CellId::new(0x7D64_0151)] {
            let local = Vec3::new(59.84, 83.408, 13.0686);
            let world = world_render_position_from_cell_local(cell, local).unwrap();
            let round_trip = cell_local_origin_from_world_render_position(cell, world).unwrap();
            assert!((round_trip - local).abs().max_element() < 0.001);
        }
    }

    #[test]
    fn position_offset_crosses_blocks_without_global_f32_cancellation() {
        let block = land_block_size();
        let from = CellId::new(0xF0F0_0001);
        let to = CellId::new(0xF1F0_0001);
        let near_edge = block - 0.000_1;
        let expected = (f64::from(block) + f64::from(0.000_1_f32) - f64::from(near_edge)) as f32;
        assert_eq!(
            retail_position_offset(from, [near_edge, 7.0, 3.0], to, [0.000_1, 7.0, 3.0]),
            Some([expected, 0.0, 0.0])
        );
        assert_ne!(expected, 0.0);
    }

    #[test]
    fn cell_relative_render_position_uses_the_integer_block_delta_first() {
        let block = land_block_size();
        let source = CellId::new(0x7D64_0015);
        let same_block_env = CellId::new(0x7D64_012B);
        let neighboring_block = CellId::new(0x7C65_0001);
        let local = Vec3::new(49.0, 98.0, 12.0);
        assert_eq!(
            cell_local_render_position_relative_to_landblock(source, local, same_block_env),
            Some(Vec3::new(49.0, 12.0, 98.0))
        );
        assert_eq!(
            cell_local_render_position_relative_to_landblock(source, local, neighboring_block),
            Some(Vec3::new(block + 49.0, 12.0, 98.0 - block))
        );
    }

    #[test]
    fn global_ac_origin_localizes_without_a_basis_swap() {
        let cell = CellId::new(0x7D64_0015);
        let block = land_block_size();
        let global = Vec3::new(0x7D as f32 * block + 49.0, 0x64 as f32 * block + 98.0, 12.0);
        let local = cell_local_origin_from_world_ac_position(cell, global).unwrap();
        assert!((local - Vec3::new(49.0, 98.0, 12.0)).abs().max_element() < 0.01);
    }
}
