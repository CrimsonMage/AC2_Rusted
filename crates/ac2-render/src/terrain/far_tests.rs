use std::collections::HashSet;

use ac2_core::Random;

use super::{far_radius_for_draw_distance, lod_step, FarBlock, FarTerrain, FIRST_FAR_CHUNK};

const POLYGONS: u32 = 16;
const BLOCK: f32 = 160.0;

fn layout(near: i32, far: i32) -> FarTerrain {
    FarTerrain::new(near, far, POLYGONS, BLOCK, &mut Random::default())
}

/// Centre-relative position of vertex `index`, found in its block.
fn position(terrain: &FarTerrain, index: u32) -> (f32, f32) {
    let block: &FarBlock = terrain
        .chunks
        .iter()
        .flat_map(|chunk| chunk.blocks.iter())
        .find(|b| (b.base..b.base + b.side * b.side).contains(&index))
        .expect("index lies in a block");
    let local = index - block.base;
    let (x, y) = (local % block.side, local / block.side);
    (
        x as f32 * block.quad_length + block.x as f32 * BLOCK,
        y as f32 * block.quad_length + block.y as f32 * BLOCK,
    )
}

#[test]
fn ring_steps_double_every_two_blocks() {
    let steps: Vec<u32> = (1..=8).map(lod_step).collect();
    assert_eq!(steps, [1, 2, 2, 4, 4, 8, 8, 16]);
    assert_eq!(far_radius_for_draw_distance(0), 0);
    assert_eq!(far_radius_for_draw_distance(2), 4);
    assert_eq!(far_radius_for_draw_distance(4), 8);
}

#[test]
fn far_chunks_tile_the_ring_once() {
    let terrain = layout(1, 4);
    assert_eq!(terrain.chunks.len(), 6);
    assert_eq!(terrain.chunks[0].blocks.len(), 1);
    assert_eq!(terrain.chunks[1].blocks.len(), 9);
    let mut seen = HashSet::new();
    for chunk in &terrain.chunks[FIRST_FAR_CHUNK..] {
        for block in &chunk.blocks {
            assert!(block.x.abs().max(block.y.abs()) > 1);
            assert!(seen.insert((block.x, block.y)), "{block:?} twice");
        }
    }
    assert_eq!(seen.len(), 11 * 11 - 9);
    // Without a far radius only the centre and near chunks exist.
    assert_eq!(layout(1, 0).chunks.len(), 2);
}

#[test]
fn indices_stay_in_their_vertex_ranges() {
    let terrain = layout(2, 4);
    assert!(terrain.indices().iter().all(|i| *i < terrain.vertex_count));
    assert!(terrain
        .water_indices
        .iter()
        .all(|i| *i < terrain.water_vertex_count));
    assert_eq!(terrain.chunk_ranges().len(), terrain.chunks.len());
    assert_eq!(terrain.indices().len() % 3, 0);
}

#[test]
fn diagonals_follow_the_random_stream() {
    let mut random = Random::default();
    let terrain = layout(1, 2);
    // Chunk 0's first quad takes the generator's first roll.
    let roll = random.roll_dice(0, 1);
    let side = terrain.chunks[0].blocks[0].side;
    let (a, b) = (0, side);
    let expected = if roll == 0 {
        [b, a, b + 1, a, a + 1, b + 1]
    } else {
        [b, a, a + 1, b, a + 1, b + 1]
    };
    assert_eq!(terrain.indices()[..6], expected);
    assert_eq!(
        layout(1, 2),
        terrain,
        "the same stream rebuilds the same mesh"
    );
}

#[test]
fn crack_fill_meets_the_fine_midpoints() {
    let terrain = layout(1, 6);
    let mut fills = 0;
    for (chunk, range) in terrain.chunks.iter().zip(terrain.chunk_ranges()) {
        let quads: u32 = chunk.blocks.iter().map(|b| b.quads * b.quads * 6).sum();
        let edge = &terrain.indices()[(range.start + quads) as usize..range.end as usize];
        for triangle in edge.chunks(3) {
            let [c0, c1, f] = [0, 1, 2].map(|k| position(&terrain, triangle[k]));
            // The fine vertex is the midpoint of the coarse segment on the shared border.
            assert!(
                (f.0 - (c0.0 + c1.0) * 0.5).abs() < 1e-3,
                "{c0:?} {c1:?} {f:?}"
            );
            assert!(
                (f.1 - (c0.1 + c1.1) * 0.5).abs() < 1e-3,
                "{c0:?} {c1:?} {f:?}"
            );
            fills += 1;
        }
    }
    assert!(fills > 0);
}
