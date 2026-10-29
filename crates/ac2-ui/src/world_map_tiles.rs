//! The world map's terrain at the detail the zoom needs (`ac2_world::map_raster`), as CPU images
//! the navigation panel samples its map from.
//!
//! - The overview: the world at one pixel per land cell, drawn once by `RASTER_THREADS` workers
//!   and cut into a pyramid of `TILE`-pixel tiles at 1, 2, 4 and 8 cells per pixel. Tiles of open
//!   ocean are not kept (the ocean is a flat colour), and the full-size world is never held.
//! - The close view: from `DETAIL_FROM` pixels per land cell, every visible landblock at 8
//!   pixels per cell, drawn on demand by `DETAIL_THREADS` workers that skip blocks scrolled out of
//!   view, kept in a least-recently-used cache of `DETAIL_CACHE` blocks.
//!
//! The workers are fixed counts (as every DAT reader is; `ac2_db::loader_pool` lists them) and
//! read through their own small-cache store. Nothing here touches a file on the frame thread.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use ac2_world::map_raster::{self as raster, BLOCKS, DETAIL_SIDE, OCEAN, WORLD_CELLS};
use ac2_world::Ac2WorldSource;

const TILE: usize = 256;
const LEVELS: usize = 4;
/// The overview's builders (once, at start) and the close view's workers.
pub const RASTER_THREADS: usize = 4;
pub const DETAIL_THREADS: usize = 2;
const DETAIL_CACHE: usize = 400;
const DETAIL_IN_FLIGHT: usize = 32;
/// Pixels per land cell from which landblocks are drawn in detail.
pub const DETAIL_FROM: f32 = 4.0;

/// A finished overview tile: level, column, row, and pixels unless all ocean.
type OverviewTile = (usize, usize, usize, Option<Arc<[u8]>>);

enum Detail {
    Pending,
    Ready(Arc<[u8]>, u64),
    Missing,
}

/// A worker's answer for one block.
enum DetailResult {
    Skipped,
    Missing,
    Drawn(Arc<[u8]>),
}

/// Which part of the land a map view shows: its centre in land cells (x east, y north from the
/// south-west corner) and its pixels per cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapWindow {
    pub centre: (f64, f64),
    pub pixels_per_cell: f32,
}

impl MapWindow {
    /// The land cell under pixel (`px`, `py`) of a `w` x `h` image of this window.
    pub fn cell_at(&self, w: i32, h: i32, px: f64, py: f64) -> (f64, f64) {
        let s = f64::from(self.pixels_per_cell);
        (
            self.centre.0 + (px - f64::from(w) / 2.0) / s,
            self.centre.1 - (py - f64::from(h) / 2.0) / s,
        )
    }

    /// The pixel of land cell (`x`, `y`) in a `w` x `h` image of this window.
    pub fn pixel_of(&self, w: i32, h: i32, x: f64, y: f64) -> (f64, f64) {
        let s = f64::from(self.pixels_per_cell);
        (
            f64::from(w) / 2.0 + (x - self.centre.0) * s,
            f64::from(h) / 2.0 - (y - self.centre.1) * s,
        )
    }
}

pub struct MapTiles {
    overview: Vec<HashMap<(usize, usize), Arc<[u8]>>>,
    /// Overview tiles known to be ocean (not kept).
    ocean: Vec<HashSet<(usize, usize)>>,
    overview_rx: Receiver<OverviewTile>,
    progress: Arc<AtomicU32>,
    detail: HashMap<(u32, u32), Detail>,
    request_tx: Sender<(u32, u32)>,
    result_rx: Receiver<((u32, u32), DetailResult)>,
    visible: Arc<Mutex<HashSet<(u32, u32)>>>,
    frame: u64,
}

impl MapTiles {
    /// Starts the overview builder and the detail workers over `world`.
    pub fn new(world: Arc<Ac2WorldSource>) -> Self {
        let (overview_tx, overview_rx) = channel();
        let progress = Arc::new(AtomicU32::new(0));
        {
            let (world, progress) = (Arc::clone(&world), Arc::clone(&progress));
            let _ = std::thread::Builder::new()
                .name("ac2 map overview".into())
                .spawn(move || build_overview(&world, &progress, &overview_tx));
        }
        let (request_tx, request_rx) = channel::<(u32, u32)>();
        let (result_tx, result_rx) = channel();
        let request_rx = Arc::new(Mutex::new(request_rx));
        let visible = Arc::new(Mutex::new(HashSet::new()));
        for n in 0..DETAIL_THREADS {
            let (world, rx, tx, visible) = (
                Arc::clone(&world),
                Arc::clone(&request_rx),
                result_tx.clone(),
                Arc::clone(&visible),
            );
            let _ = std::thread::Builder::new()
                .name(format!("ac2 map detail {n}"))
                .spawn(move || loop {
                    let Ok(block) = rx.lock().unwrap_or_else(|e| e.into_inner()).recv() else {
                        return;
                    };
                    let wanted = visible
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .contains(&block);
                    let result = if !wanted {
                        DetailResult::Skipped
                    } else {
                        match raster::detail_tile(&world, block.0, block.1) {
                            Some(rgba) => DetailResult::Drawn(rgba.into()),
                            None => DetailResult::Missing,
                        }
                    };
                    if tx.send((block, result)).is_err() {
                        return;
                    }
                });
        }
        Self {
            overview: (0..LEVELS).map(|_| HashMap::new()).collect(),
            ocean: (0..LEVELS).map(|_| HashSet::new()).collect(),
            overview_rx,
            progress,
            detail: HashMap::new(),
            request_tx,
            result_rx,
            visible,
            frame: 0,
        }
    }

    /// How far the overview has got, 0..1.
    pub fn progress(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / BLOCKS as f32
    }

    /// Takes the tiles the workers finished; whether any arrived (the map is drawn again then).
    pub fn pump(&mut self) -> bool {
        let mut arrived = false;
        while let Ok((level, tx, ty, tile)) = self.overview_rx.try_recv() {
            match tile {
                Some(pixels) => {
                    self.overview[level].insert((tx, ty), pixels);
                }
                None => {
                    self.ocean[level].insert((tx, ty));
                }
            }
            arrived = true;
        }
        while let Ok((block, result)) = self.result_rx.try_recv() {
            match result {
                DetailResult::Drawn(pixels) => {
                    self.detail.insert(block, Detail::Ready(pixels, self.frame));
                    arrived = true;
                }
                DetailResult::Missing => {
                    self.detail.insert(block, Detail::Missing);
                }
                DetailResult::Skipped => {
                    self.detail.remove(&block);
                }
            }
        }
        arrived
    }

    /// Whether blocks are still being drawn for the view.
    pub fn busy(&self) -> bool {
        self.progress() < 1.0 || self.detail.values().any(|d| matches!(d, Detail::Pending))
    }

    /// The map image of `window` at `w` x `h` (RGBA), sampled from the finest imagery the tiles
    /// hold for each pixel; the blocks in view the close view still needs are asked for.
    pub fn render(&mut self, window: &MapWindow, w: i32, h: i32) -> Vec<u8> {
        self.frame += 1;
        let ppc = window.pixels_per_cell;
        let want = if ppc >= 1.0 {
            0
        } else {
            ((1.0 / ppc).log2().floor() as usize).min(LEVELS - 1)
        };
        let detail = ppc >= DETAIL_FROM;
        let mut visible = HashSet::new();
        if detail {
            let (x0, y0) = window.cell_at(w, h, 0.0, f64::from(h));
            let (x1, y1) = window.cell_at(w, h, f64::from(w), 0.0);
            let block = |v: f64| (v / 8.0).floor().clamp(0.0, f64::from(BLOCKS - 1)) as u32;
            for by in block(y0)..=block(y1) {
                for bx in block(x0)..=block(x1) {
                    visible.insert((bx, by));
                }
            }
        }
        *self.visible.lock().unwrap_or_else(|e| e.into_inner()) = visible.clone();
        let mut in_flight = self
            .detail
            .values()
            .filter(|d| matches!(d, Detail::Pending))
            .count();
        for block in &visible {
            match self.detail.get_mut(block) {
                Some(Detail::Ready(_, used)) => *used = self.frame,
                Some(_) => {}
                None if in_flight < DETAIL_IN_FLIGHT && self.request_tx.send(*block).is_ok() => {
                    self.detail.insert(*block, Detail::Pending);
                    in_flight += 1;
                }
                None => {}
            }
        }
        let mut out = vec![255u8; (w.max(0) * h.max(0) * 4) as usize];
        for py in 0..h {
            for px in 0..w {
                let (cx, cy) = window.cell_at(w, h, f64::from(px) + 0.5, f64::from(py) + 0.5);
                let rgb = self.sample(cx, cy, want, detail);
                let o = ((py * w + px) * 4) as usize;
                out[o..o + 3].copy_from_slice(&rgb);
            }
        }
        self.evict();
        out
    }

    /// One land position's colour: its block in detail when wanted and drawn, else the finest
    /// overview level from `want` up that has its tile.
    fn sample(&self, cx: f64, cy: f64, want: usize, detail: bool) -> [u8; 3] {
        let world = WORLD_CELLS as f64;
        if cx < 0.0 || cy < 0.0 || cx >= world || cy >= world {
            return OCEAN;
        }
        if detail {
            let (bx, by) = ((cx / 8.0) as u32, (cy / 8.0) as u32);
            if let Some(Detail::Ready(pixels, _)) = self.detail.get(&(bx, by)) {
                let side = DETAIL_SIDE as f64;
                let x = ((cx - f64::from(bx) * 8.0) / 8.0 * side) as usize;
                let y = ((1.0 - (cy - f64::from(by) * 8.0) / 8.0) * side) as usize;
                let o = (y.min(DETAIL_SIDE - 1) * DETAIL_SIDE + x.min(DETAIL_SIDE - 1)) * 4;
                return [pixels[o], pixels[o + 1], pixels[o + 2]];
            }
        }
        for level in want..LEVELS {
            let scale = (1usize << level) as f64;
            let (lx, ly) = ((cx / scale) as usize, ((world - cy) / scale) as usize);
            let key = (lx / TILE, ly / TILE);
            if let Some(tile) = self.overview[level].get(&key) {
                let side = WORLD_CELLS >> level;
                let tw = TILE.min(side - key.0 * TILE);
                let (x, y) = (lx % TILE, ly % TILE);
                let o = (y * tw + x.min(tw - 1)) * 4;
                if o + 2 < tile.len() {
                    return [tile[o], tile[o + 1], tile[o + 2]];
                }
            }
            if self.ocean[level].contains(&key) {
                return OCEAN;
            }
        }
        OCEAN
    }

    /// Drops the least recently used detail blocks past `DETAIL_CACHE`.
    fn evict(&mut self) {
        let ready = self
            .detail
            .values()
            .filter(|d| matches!(d, Detail::Ready(..)))
            .count();
        if ready <= DETAIL_CACHE {
            return;
        }
        let mut by_age: Vec<((u32, u32), u64)> = self
            .detail
            .iter()
            .filter_map(|(k, d)| match d {
                Detail::Ready(_, used) => Some((*k, *used)),
                _ => None,
            })
            .collect();
        by_age.sort_by_key(|(_, used)| *used);
        for (key, _) in by_age.into_iter().take(ready - DETAIL_CACHE) {
            self.detail.remove(&key);
        }
    }
}

/// The overview builder: block columns from the workers, every `TILE / 8` making a strip cut into
/// the finest level's tiles and halved into the next level's image.
fn build_overview(world: &Ac2WorldSource, progress: &AtomicU32, tx: &Sender<OverviewTile>) {
    const STRIP_BLOCKS: u32 = (TILE / 8) as u32;
    let strips = BLOCKS.div_ceil(STRIP_BLOCKS);
    let half_side = WORLD_CELLS.div_ceil(2);
    let mut coarse = vec![255u8; half_side * half_side * 4];
    let mut pending: HashMap<u32, Vec<u8>> = HashMap::new();
    let mut next_strip = 0;
    let mut open = true;
    raster::world_columns(world, RASTER_THREADS, progress, |bx, column| {
        pending.insert(bx, column);
        while open && next_strip < strips {
            let first = next_strip * STRIP_BLOCKS;
            let last = (first + STRIP_BLOCKS).min(BLOCKS);
            if !(first..last).all(|b| pending.contains_key(&b)) {
                break;
            }
            let w = (last - first) as usize * 8;
            let mut strip = vec![0u8; w * WORLD_CELLS * 4];
            for (i, b) in (first..last).enumerate() {
                let column = pending.remove(&b).unwrap_or_default();
                for row in 0..WORLD_CELLS {
                    let dst = (row * w + i * 8) * 4;
                    strip[dst..dst + 32].copy_from_slice(&column[row * 32..row * 32 + 32]);
                }
            }
            open = cut(&strip, w, WORLD_CELLS, 0, next_strip as usize, tx);
            let (small, sw, sh) = raster::half(&strip, w, WORLD_CELLS);
            let x0 = next_strip as usize * TILE / 2;
            for row in 0..sh {
                let dst = (row * half_side + x0) * 4;
                coarse[dst..dst + sw * 4].copy_from_slice(&small[row * sw * 4..(row + 1) * sw * 4]);
            }
            next_strip += 1;
        }
    });
    let (mut w, mut h) = (half_side, half_side);
    for level in 1..LEVELS {
        if !open {
            return;
        }
        for tx_ in 0..w.div_ceil(TILE) {
            let x0 = tx_ * TILE;
            let tw = TILE.min(w - x0);
            let mut column = Vec::with_capacity(tw * h * 4);
            for row in 0..h {
                column.extend_from_slice(&coarse[(row * w + x0) * 4..(row * w + x0 + tw) * 4]);
            }
            open &= cut(&column, tw, h, level, tx_, tx);
        }
        if level + 1 < LEVELS {
            (coarse, w, h) = raster::half(&coarse, w, h);
        }
    }
}

/// Sends one tile column's tiles (`w <= TILE` wide); false once the receiver is gone.
fn cut(
    image: &[u8],
    w: usize,
    h: usize,
    level: usize,
    tx_: usize,
    tx: &Sender<OverviewTile>,
) -> bool {
    for ty in 0..h.div_ceil(TILE) {
        let y0 = ty * TILE;
        let th = TILE.min(h - y0);
        let rows = &image[y0 * w * 4..(y0 + th) * w * 4];
        let land = rows.chunks_exact(4).any(|p| p[..3] != OCEAN);
        let tile = land.then(|| Arc::<[u8]>::from(rows));
        if tx.send((level, tx_, ty, tile)).is_err() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_maps_cells_and_pixels_both_ways() {
        let window = MapWindow {
            centre: (1000.0, 1200.0),
            pixels_per_cell: 2.5,
        };
        let (x, y) = window.cell_at(238, 289, 10.0, 20.0);
        let (px, py) = window.pixel_of(238, 289, x, y);
        assert!((px - 10.0).abs() < 1e-9 && (py - 20.0).abs() < 1e-9);
        // North is up: a pixel higher on the image is a cell further north.
        assert!(window.cell_at(238, 289, 0.0, 0.0).1 > window.cell_at(238, 289, 0.0, 100.0).1);
    }
}
