//! The map's terrain on the GPU at the detail the zoom needs.
//!
//! - The overview: `terrain::world_raster` (one pixel per land cell), drawn once on start by
//!   `RASTER_THREADS` workers and cut into a pyramid of `TILE`-pixel tiles at 1, 2, 4 and 8 cells
//!   per pixel. Tiles of open ocean are not uploaded (the ocean is a flat fill); the raster is
//!   dropped once cut, so the CPU keeps none of it.
//! - The close view: from `DETAIL_FROM` screen pixels per land cell, every visible landblock as
//!   a `terrain::detail_tile` (8 pixels per cell), drawn on demand by `DETAIL_THREADS` workers
//!   that skip blocks scrolled out of view, kept in a least-recently-drawn cache of
//!   `DETAIL_CACHE` textures.
//!
//! Workers wake the UI when a tile is ready; nothing polls.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use ac2_world::Ac2WorldSource;
use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};

use ac2_world::map_raster::{self as terrain, BLOCKS, DETAIL_SIDE, OCEAN, WORLD_CELLS};

const TILE: usize = 256;
const LEVELS: usize = 4;
const RASTER_THREADS: usize = 4;
const DETAIL_THREADS: usize = 2;
const DETAIL_CACHE: usize = 600;
/// Requests in flight at once; more wait for the next frame.
const DETAIL_IN_FLIGHT: usize = 48;
/// Screen pixels per land cell from which landblocks are drawn in detail.
pub const DETAIL_FROM: f32 = 6.0;
/// Uploads per frame, so a burst of tiles does not stall one frame.
const UPLOADS_PER_FRAME: usize = 24;

pub fn ocean() -> Color32 {
    Color32::from_rgb(OCEAN[0], OCEAN[1], OCEAN[2])
}

/// A finished overview tile: its level, column and row, and pixels unless it is all ocean.
type OverviewTile = (usize, usize, usize, Option<ColorImage>);

/// A worker's answer for one block.
enum DetailResult {
    /// The block had scrolled out of view, so it was not drawn.
    Skipped,
    /// The DATs hold no such block.
    Missing,
    Drawn(ColorImage),
}

enum Detail {
    Pending,
    Ready(TextureHandle, u64),
    Missing,
}

pub struct Tiles {
    overview: Vec<HashMap<(usize, usize), Option<TextureHandle>>>,
    overview_rx: Receiver<OverviewTile>,
    progress: Arc<AtomicU32>,
    detail: HashMap<(u32, u32), Detail>,
    request_tx: Sender<(u32, u32)>,
    result_rx: Receiver<((u32, u32), DetailResult)>,
    visible: Arc<Mutex<HashSet<(u32, u32)>>>,
    /// Whether the last frame left visible blocks unrequested (past the in-flight limit).
    waiting: bool,
    frame: u64,
}

/// A view of the land: where land cell (x east, y north) falls on the screen.
pub trait View {
    fn cell_to_screen(&self, x: f64, y: f64) -> Pos2;
    fn screen_to_cell(&self, p: Pos2) -> (f64, f64);
    /// Screen pixels per land cell.
    fn pixels_per_cell(&self) -> f32;
}

impl Tiles {
    pub fn new(ctx: &egui::Context, world: Arc<Ac2WorldSource>) -> Self {
        let (overview_tx, overview_rx) = channel();
        let progress = Arc::new(AtomicU32::new(0));
        {
            let (world, progress, ctx) = (Arc::clone(&world), Arc::clone(&progress), ctx.clone());
            std::thread::Builder::new()
                .name("map overview".into())
                .spawn(move || build_overview(&world, &progress, &overview_tx, &ctx))
                .expect("spawn the map overview builder");
        }
        let (request_tx, request_rx) = channel::<(u32, u32)>();
        let (result_tx, result_rx) = channel();
        let request_rx = Arc::new(Mutex::new(request_rx));
        let visible = Arc::new(Mutex::new(HashSet::new()));
        for n in 0..DETAIL_THREADS {
            let (world, rx, tx, visible, ctx) = (
                Arc::clone(&world),
                Arc::clone(&request_rx),
                result_tx.clone(),
                Arc::clone(&visible),
                ctx.clone(),
            );
            std::thread::Builder::new()
                .name(format!("map detail {n}"))
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
                        match terrain::detail_tile(&world, block.0, block.1) {
                            Some(rgba) => DetailResult::Drawn(ColorImage::from_rgba_unmultiplied(
                                [DETAIL_SIDE, DETAIL_SIDE],
                                &rgba,
                            )),
                            None => DetailResult::Missing,
                        }
                    };
                    if tx.send((block, result)).is_err() {
                        return;
                    }
                    if wanted {
                        ctx.request_repaint();
                    }
                })
                .expect("spawn a map detail worker");
        }
        Self {
            overview: (0..LEVELS).map(|_| HashMap::new()).collect(),
            overview_rx,
            progress,
            detail: HashMap::new(),
            request_tx,
            result_rx,
            visible,
            waiting: false,
            frame: 0,
        }
    }

    /// Whether tiles are still being drawn or uploaded.
    pub fn busy(&self) -> bool {
        self.progress() < 1.0
            || self.overview[LEVELS - 1].is_empty()
            || self.waiting
            || self.detail.values().any(|d| matches!(d, Detail::Pending))
    }

    /// How far the overview has got, 0..1.
    pub fn progress(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / BLOCKS as f32
    }

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        rect: Rect,
        view: &impl View,
    ) {
        self.frame += 1;
        self.upload(ctx);

        // The overview: the coarsest level first, then finer ones over it up to the zoom's.
        let ppc = view.pixels_per_cell();
        let want = if ppc >= 1.0 {
            0
        } else {
            ((1.0 / ppc).log2().floor() as usize).min(LEVELS - 1)
        };
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        let (x_min, x_max, y_min, y_max) = visible_cells(rect, view);
        let world = WORLD_CELLS as f64;
        for level in (want..LEVELS).rev() {
            let cells = (TILE << level) as f64;
            let tiles = |a: f64, b: f64| {
                ((a / cells).floor().max(0.0) as usize)..=((b / cells).floor().max(0.0) as usize)
            };
            // Tile rows count south from the world's north edge.
            for ty in tiles(world - y_max, world - y_min) {
                for tx in tiles(x_min, x_max) {
                    let Some(Some(texture)) = self.overview[level].get(&(tx, ty)) else {
                        continue;
                    };
                    let size = texture.size();
                    let x0 = tx as f64 * cells;
                    let y0 = ty as f64 * cells;
                    let x1 = x0 + (size[0] << level) as f64;
                    let y1 = y0 + (size[1] << level) as f64;
                    // Tile rows run south from the world's north edge.
                    let top = WORLD_CELLS as f64 - y0;
                    let bottom = WORLD_CELLS as f64 - y1;
                    let r = Rect::from_two_pos(
                        view.cell_to_screen(x0, top),
                        view.cell_to_screen(x1, bottom),
                    );
                    painter.image(texture.id(), r, uv, Color32::WHITE);
                }
            }
        }

        // The close view. The workers read the visible set before drawing a block, so it is
        // published before any request goes out.
        let blocks = |a: f64, b: f64| {
            ((a / 8.0).floor().max(0.0) as u32)
                ..=((b / 8.0).floor().min(f64::from(BLOCKS - 1)) as u32)
        };
        let visible: HashSet<(u32, u32)> = if ppc >= DETAIL_FROM {
            blocks(y_min, y_max)
                .flat_map(|by| blocks(x_min, x_max).map(move |bx| (bx, by)))
                .collect()
        } else {
            HashSet::new()
        };
        *self.visible.lock().unwrap_or_else(|e| e.into_inner()) = visible;
        self.waiting = false;
        if ppc >= DETAIL_FROM {
            let mut in_flight = self
                .detail
                .values()
                .filter(|d| matches!(d, Detail::Pending))
                .count();
            for by in blocks(y_min, y_max) {
                for bx in blocks(x_min, x_max) {
                    match self.detail.get_mut(&(bx, by)) {
                        Some(Detail::Ready(texture, used)) => {
                            *used = self.frame;
                            let (x0, y0) = (f64::from(bx) * 8.0, f64::from(by) * 8.0);
                            let r = Rect::from_two_pos(
                                view.cell_to_screen(x0, y0 + 8.0),
                                view.cell_to_screen(x0 + 8.0, y0),
                            );
                            painter.image(texture.id(), r, uv, Color32::WHITE);
                        }
                        Some(Detail::Pending | Detail::Missing) => {}
                        None if in_flight < DETAIL_IN_FLIGHT => {
                            if self.request_tx.send((bx, by)).is_ok() {
                                self.detail.insert((bx, by), Detail::Pending);
                                in_flight += 1;
                            }
                        }
                        None => {
                            self.waiting = true;
                            ctx.request_repaint();
                        }
                    }
                }
            }
        }
        self.evict();
    }

    fn upload(&mut self, ctx: &egui::Context) {
        let mut budget = UPLOADS_PER_FRAME;
        while budget > 0 {
            let Ok((level, tx, ty, image)) = self.overview_rx.try_recv() else {
                break;
            };
            let texture = image.map(|i| {
                ctx.load_texture(
                    format!("overview {level} {tx} {ty}"),
                    i,
                    TextureOptions::LINEAR,
                )
            });
            self.overview[level].insert((tx, ty), texture);
            budget -= 1;
        }
        while budget > 0 {
            let Ok((block, image)) = self.result_rx.try_recv() else {
                break;
            };
            let entry = match image {
                DetailResult::Drawn(i) => Detail::Ready(
                    ctx.load_texture(
                        format!("detail {} {}", block.0, block.1),
                        i,
                        TextureOptions::LINEAR,
                    ),
                    self.frame,
                ),
                DetailResult::Missing => Detail::Missing,
                // Asked for again if it comes back into view.
                DetailResult::Skipped => {
                    self.detail.remove(&block);
                    continue;
                }
            };
            self.detail.insert(block, entry);
            budget -= 1;
        }
        if budget == 0 {
            ctx.request_repaint();
        }
    }

    /// Drops the least recently drawn detail textures past `DETAIL_CACHE`.
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

/// The land cells the screen rectangle covers: x min, x max, y min, y max (y north).
fn visible_cells(rect: Rect, view: &impl View) -> (f64, f64, f64, f64) {
    let a = view.screen_to_cell(rect.left_top());
    let b = view.screen_to_cell(rect.right_bottom());
    (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1))
}

/// The overview builder. Block columns arrive from the workers; every `TILE / 8` of them make a
/// strip `TILE` cells wide, which is cut into the finest level's tiles and halved into the next
/// level's image, so the whole world is never held at full size.
fn build_overview(
    world: &Ac2WorldSource,
    progress: &AtomicU32,
    tx: &Sender<OverviewTile>,
    ctx: &egui::Context,
) {
    const STRIP_BLOCKS: u32 = (TILE / 8) as u32;
    let strips = BLOCKS.div_ceil(STRIP_BLOCKS);
    let half_side = WORLD_CELLS.div_ceil(2);
    let mut coarse = vec![255u8; half_side * half_side * 4];
    let mut pending: HashMap<u32, Vec<u8>> = HashMap::new();
    let mut next_strip = 0;
    let mut open = true;
    terrain::world_columns(world, RASTER_THREADS, progress, |bx, column| {
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
            // Halve the strip into its place in the next level's image.
            let (small, sw, sh) = terrain::half(&strip, w, WORLD_CELLS);
            let x0 = next_strip as usize * TILE / 2;
            for row in 0..sh {
                let dst = (row * half_side + x0) * 4;
                coarse[dst..dst + sw * 4].copy_from_slice(&small[row * sw * 4..(row + 1) * sw * 4]);
            }
            ctx.request_repaint();
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
        ctx.request_repaint();
        if level + 1 < LEVELS {
            (coarse, w, h) = terrain::half(&coarse, w, h);
        }
    }
}

/// Sends the tiles of one tile column (`w <= TILE` wide, `h` tall); false once the UI is gone.
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
        let tile = land.then(|| ColorImage::from_rgba_unmultiplied([w, th], rows));
        if tx.send((level, tx_, ty, tile)).is_err() {
            return false;
        }
    }
    true
}
