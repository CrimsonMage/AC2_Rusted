use ac2_core::CellId;

/// AC2 `CLandBlockData::CachePackValues`: per-polygon diagonal from global polygon coordinates
/// (`polygons_per_side * block + polygon`).
pub(crate) fn ac2_sw_to_ne_cut(global_x: i32, global_y: i32) -> bool {
    let seed = global_y
        .wrapping_mul(global_x)
        .wrapping_mul(global_x)
        .wrapping_mul(0x3955_7FCF)
        .wrapping_sub(0x4A65_06DB)
        .wrapping_mul(global_y)
        .wrapping_sub(global_x.wrapping_mul(0x6B3C_5F11))
        .wrapping_sub(0x51DD_9315);
    (seed as u32 as f64) * 2.328_306_4e-10 >= 0.5
}

/// Diagonal of one terrain polygon of a block whose grid has `polygons_per_side` polygons.
pub fn terrain_polygon_cut(
    block_cell_id: CellId,
    polygon_x: usize,
    polygon_y: usize,
    polygons_per_side: usize,
) -> bool {
    let per_side = polygons_per_side as i32;
    let block_x = ((block_cell_id.raw() >> 24) & 0xFF) as i32;
    let block_y = ((block_cell_id.raw() >> 16) & 0xFF) as i32;
    ac2_sw_to_ne_cut(
        block_x * per_side + polygon_x as i32,
        block_y * per_side + polygon_y as i32,
    )
}
