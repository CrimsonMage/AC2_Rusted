//! The world map's imagery (a modern addition: retail's map is painted art). The world drawn from
//! its own terrain: every landblock's per-vertex far-landscape colour
//! (`SurfaceInfo::far_vertex_color`, the colour `LScapeTerrainMesh::BuildFarTerrainMesh` gives the
//! distant terrain), lit by the vertex normal from the north-west, with the block's water plane
//! (`CLandBlockMesh::InqWaterPlane`) over the vertices below it and roads where the vertex has a
//! road surface. Two products: the whole world at one pixel per land cell, and one landblock at
//! `DETAIL_PX` pixels per polygon for close zoom.

use std::sync::Arc;

use ac2_core::CellId;

use crate::{Ac2WorldSource, LandBlockDesc};

/// Land cells per side of the world and of a block; terrain polygons per block side.
pub const WORLD_CELLS: usize = 2040;
pub const BLOCKS: u32 = 255;
const POLYS: usize = 16;
/// Pixels per terrain polygon in a detail tile (8 per land cell).
pub const DETAIL_PX: usize = 4;
/// A detail tile's side in pixels.
pub const DETAIL_SIDE: usize = POLYS * DETAIL_PX;

pub const OCEAN: [u8; 3] = [38, 78, 128];
const WATER: [u8; 3] = [44, 88, 138];
const ROAD: [f32; 3] = [0.62, 0.53, 0.40];
/// The light the shading assumes: from the north-west, 45 degrees up.
const LIGHT: [f32; 3] = [-0.5, 0.5, std::f32::consts::FRAC_1_SQRT_2];

/// The colour of one vertex sample: the far colour (linear) to display levels, shaded.
fn shade(colour: [f32; 3], normal: [f32; 3]) -> [u8; 3] {
    let lit = normal[0] * LIGHT[0] + normal[1] * LIGHT[1] + normal[2] * LIGHT[2];
    let k = (0.45 + 0.75 * lit.max(0.0)).min(1.3);
    colour.map(|v| (v.max(0.0).powf(1.0 / 2.2) * k * 255.0).clamp(0.0, 255.0) as u8)
}

fn block_cell(bx: u32, by: u32) -> CellId {
    CellId::new((bx << 24) | (by << 16) | 1)
}

/// The vertex samples of one block, x-major (`vertex_side()` squared).
struct Samples {
    side: usize,
    height: Vec<f32>,
    colour: Vec<[f32; 3]>,
    normal: Vec<[f32; 3]>,
    road: Vec<f32>,
    water: Option<f32>,
}

fn samples(world: &Ac2WorldSource, bx: u32, by: u32) -> Option<Samples> {
    let cell = block_cell(bx, by);
    let lb: Arc<LandBlockDesc> = world.load_landblock(cell).ok()??;
    let side = lb.vertex_side();
    let mut s = Samples {
        side,
        height: lb.height_grid(),
        colour: Vec::with_capacity(side * side),
        normal: Vec::with_capacity(side * side),
        road: Vec::with_capacity(side * side),
        water: world.water_surface_height(cell).ok().flatten(),
    };
    for x in 0..side {
        for y in 0..side {
            let c = lb
                .surface_info(x, y)
                .map_or([1.0; 4], |i| i.far_vertex_color);
            s.colour.push([c[0], c[1], c[2]]);
            s.normal.push(lb.normal(x, y).unwrap_or([0.0, 0.0, 1.0]));
            let road = lb
                .road_info(x, y)
                .is_some_and(|r| r.base_material.is_some());
            s.road.push(if road { 1.0 } else { 0.0 });
        }
    }
    Some(s)
}

/// One column of blocks at one pixel per land cell: `8` wide, `WORLD_CELLS` tall, RGBA, north up.
pub fn block_column(world: &Ac2WorldSource, bx: u32) -> Vec<u8> {
    let mut column = vec![255u8; 8 * WORLD_CELLS * 4];
    for by in 0..BLOCKS {
        let block = samples(world, bx, by);
        for cy in 0..8 {
            for cx in 0..8 {
                let rgb = block.as_ref().map_or(OCEAN, |s| cell_colour(s, cx, cy));
                let row = WORLD_CELLS - 1 - (by as usize * 8 + cy);
                let i = (row * 8 + cx) * 4;
                column[i..i + 3].copy_from_slice(&rgb);
            }
        }
    }
    column
}

/// Every block column, drawn by `threads` workers and handed to `each` on the calling thread as
/// it finishes (in any order); `progress` counts finished columns.
pub fn world_columns(
    world: &Ac2WorldSource,
    threads: usize,
    progress: &std::sync::atomic::AtomicU32,
    mut each: impl FnMut(u32, Vec<u8>),
) {
    use std::sync::atomic::Ordering;
    let next = std::sync::atomic::AtomicU32::new(0);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            let tx = tx.clone();
            let next = &next;
            scope.spawn(move || loop {
                let bx = next.fetch_add(1, Ordering::Relaxed);
                if bx >= BLOCKS || tx.send((bx, block_column(world, bx))).is_err() {
                    break;
                }
            });
        }
        drop(tx);
        for (bx, column) in rx {
            progress.fetch_add(1, Ordering::Relaxed);
            each(bx, column);
        }
    });
}

/// The whole world at one pixel per land cell (`WORLD_CELLS` square, RGBA, north up).
pub fn world_raster(world: &Ac2WorldSource, threads: usize) -> Vec<u8> {
    let mut out = vec![255u8; WORLD_CELLS * WORLD_CELLS * 4];
    let progress = std::sync::atomic::AtomicU32::new(0);
    world_columns(world, threads, &progress, |bx, column| {
        for row in 0..WORLD_CELLS {
            let dst = (row * WORLD_CELLS + bx as usize * 8) * 4;
            out[dst..dst + 32].copy_from_slice(&column[row * 32..row * 32 + 32]);
        }
    });
    out
}

/// A land cell's colour: its middle vertex.
fn cell_colour(s: &Samples, cx: usize, cy: usize) -> [u8; 3] {
    let i = (cx * 2 + 1) * s.side + cy * 2 + 1;
    if s.water.is_some_and(|w| s.height[i] < w) {
        return WATER;
    }
    let rgb = shade(s.colour[i], s.normal[i]);
    if s.road[i] > 0.0 {
        let r = shade(ROAD, s.normal[i]);
        return [0, 1, 2].map(|k| ((u16::from(rgb[k]) + u16::from(r[k]) * 3) / 4) as u8);
    }
    rgb
}

/// One block at `DETAIL_PX` pixels per terrain polygon (RGBA, north up), the vertex samples
/// blended bilinearly; `None` for a block the DATs do not hold.
pub fn detail_tile(world: &Ac2WorldSource, bx: u32, by: u32) -> Option<Vec<u8>> {
    let s = samples(world, bx, by)?;
    let mut out = vec![255u8; DETAIL_SIDE * DETAIL_SIDE * 4];
    let last = (s.side - 1) as f32;
    for py in 0..DETAIL_SIDE {
        // Vertex y runs north; the image's rows run south.
        let fy = (last - (py as f32 + 0.5) / DETAIL_PX as f32).clamp(0.0, last);
        for px in 0..DETAIL_SIDE {
            let fx = ((px as f32 + 0.5) / DETAIL_PX as f32).clamp(0.0, last);
            let (x0, y0) = ((fx as usize).min(s.side - 2), (fy as usize).min(s.side - 2));
            let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
            let w = [
                (1.0 - tx) * (1.0 - ty),
                tx * (1.0 - ty),
                (1.0 - tx) * ty,
                tx * ty,
            ];
            let idx = [
                x0 * s.side + y0,
                (x0 + 1) * s.side + y0,
                x0 * s.side + y0 + 1,
                (x0 + 1) * s.side + y0 + 1,
            ];
            let mix3 = |v: &[[f32; 3]]| -> [f32; 3] {
                let mut o = [0.0; 3];
                for (k, &i) in idx.iter().enumerate() {
                    for (c, oc) in o.iter_mut().enumerate() {
                        *oc += v[i][c] * w[k];
                    }
                }
                o
            };
            let mix1 = |v: &[f32]| idx.iter().zip(w).map(|(&i, w)| v[i] * w).sum::<f32>();
            let height = mix1(&s.height);
            let rgb = if s.water.is_some_and(|wl| height < wl) {
                WATER
            } else {
                let n = mix3(&s.normal);
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                let n = n.map(|v| v / len);
                let road = mix1(&s.road);
                let c = mix3(&s.colour);
                let c = [0, 1, 2].map(|k| c[k] * (1.0 - road) + ROAD[k] * road);
                shade(c, n)
            };
            let i = (py * DETAIL_SIDE + px) * 4;
            out[i..i + 3].copy_from_slice(&rgb);
        }
    }
    Some(out)
}

/// Halves an RGBA image (box filter); odd edges keep their last row and column.
pub fn half(src: &[u8], w: usize, h: usize) -> (Vec<u8>, usize, usize) {
    let (nw, nh) = (w.div_ceil(2), h.div_ceil(2));
    let mut out = vec![255u8; nw * nh * 4];
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..3 {
                let mut sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                    sum += u32::from(src[(sy * w + sx) * 4 + c]);
                }
                out[(y * nw + x) * 4 + c] = (sum / 4) as u8;
            }
        }
    }
    (out, nw, nh)
}
