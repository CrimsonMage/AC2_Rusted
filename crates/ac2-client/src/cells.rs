//! `CellManager` (Legions `portal/client/src/smartbox/worldmanager/cellmanager.cpp`, exact
//! instruction matches of admin 7148) over the client's loader threads.
//!
//! - `UpdateVisibleCells`: when the player's cell leaves the block of the last request (or is
//!   an interior), `AsyncGetCells` asks for its block. While that request is out the resident
//!   world stays as it is, and a blocking caller (world entry, teleport) makes
//!   `IsBlockingForCells` true, which holds the player's physics.
//! - `OnTopLevelRequestFinished`: each finished ring asks for the next one around the same
//!   centre: the near radius with sub-data (objects, interiors), then the far radius alone, then
//!   the near blocks' scene objects. `OnContextFinished` ends the wait.
//! - `FinishUpdateVisibleCells` / `LScapeManager::UpdateVisibleCells`: the blocks within the
//!   near radius of the player's cell become resident and the rest are released.
//!
//! A near block is built into its own `Terrain` on a loader thread and absorbed here whole.

use std::collections::{BTreeSet, HashMap};

use ac2_physics::{CellSource, Terrain};

use crate::loader::{Loaded, Loader, Request};

/// `LandDefs::first_envcell_id`.
const FIRST_ENV_CELL: u32 = 0x100;
/// `LScapeManager::Init`: one block around the player is near (3 x 3), four more are far.
pub const NEAR_RADIUS: i32 = 1;
pub const FAR_RADIUS: i32 = 4;

fn is_outside(cell: u32) -> bool {
    cell & 0xFFFF < FIRST_ENV_CELL
}

fn block_of(cell: u32) -> u32 {
    cell & 0xFFFF_0000
}

/// The block of `cell` and every block within `radius` of it that lies on the map.
pub fn block_radius(cell: u32, radius: i32) -> Vec<u32> {
    let (x, y) = ((cell >> 24) as i32, ((cell >> 16) & 0xFF) as i32);
    let mut blocks = Vec::new();
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let (bx, by) = (x + dx, y + dy);
            if (0..=0xFF).contains(&bx) && (0..=0xFF).contains(&by) {
                blocks.push((bx as u32) << 24 | (by as u32) << 16);
            }
        }
    }
    blocks
}

/// `RequestAsyncLoadBlockRadius`' ring: the on-map blocks exactly `radius` from the centre.
fn ring(center: u32, radius: i32) -> Vec<u32> {
    let (x, y) = ((center >> 24) as i32, ((center >> 16) & 0xFF) as i32);
    let mut blocks = Vec::new();
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let edge = dx == -radius || dx == radius || dy == -radius || dy == radius;
            let (bx, by) = (x + dx, y + dy);
            if edge && (0..=0xFF).contains(&bx) && (0..=0xFF).contains(&by) {
                blocks.push((bx as u32) << 24 | (by as u32) << 16);
            }
        }
    }
    blocks
}

/// A landblock's cells joining or leaving the resident world (`CObjCell::InitDynamicObjects` /
/// `ReleaseDynamicObjects`, which tell `CObjectMaint` through `InitObjCell` / `ReleaseObjCell`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockChange {
    Resident(u32),
    Released(u32),
}

/// One `AsyncGetCells` request and its ring-by-ring follow-ups (`m_lastAsyncContext`).
#[derive(Debug)]
struct Context {
    /// The requested cell.
    cell: u32,
    /// `_nTimesFinished` so far.
    times_finished: i32,
    /// The current ring's blocks still loading.
    outstanding: BTreeSet<u32>,
}

/// `CellManager` with `LScapeManager`'s resident window.
#[derive(Debug, Default)]
pub struct CellManager {
    last_async_cell: u32,
    context: Option<Context>,
    waiting: bool,
    blocking: bool,
    /// `m_loadPos.objcell_id` and whether `m_currCell` was found.
    load_cell: u32,
    current_cell: bool,
    /// The resident window (`LScapeManager::m_landBlocks`).
    window: BTreeSet<u32>,
    resident: BTreeSet<u32>,
    /// Near blocks loaded and not resident (`DBCache`); `None` where the map has no block.
    prepared: HashMap<u32, Option<Terrain>>,
    pending: BTreeSet<u32>,
    far_pending: BTreeSet<u32>,
    far_cached: BTreeSet<u32>,
    changes: Vec<BlockChange>,
}

impl CellManager {
    pub fn resident(&self) -> impl Iterator<Item = u32> + '_ {
        self.resident.iter().copied()
    }

    /// The blocks that became resident or were released since the last call, in order.
    pub(crate) fn take_changes(&mut self) -> Vec<BlockChange> {
        std::mem::take(&mut self.changes)
    }

    /// `CellManager::IsBlockingForCells` (`m_bBlockingForCells`).
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    /// `m_bWaitingForAsyncGet`.
    pub fn is_waiting(&self) -> bool {
        self.waiting
    }

    /// `CellManager::UpdateVisibleCells(pos, _bBlocking)`.
    pub(crate) fn update_visible_cells(
        &mut self,
        cell: u32,
        blocking: bool,
        terrain: &mut Terrain,
        loader: &mut Loader,
    ) {
        if cell == 0 {
            self.release_cells(terrain);
            return;
        }
        let blocking = self.blocking || blocking;
        let moved = cell != self.last_async_cell
            && !(is_outside(cell) && block_of(self.last_async_cell) == block_of(cell));
        if moved && self.context.is_none() {
            self.async_get_cells(cell, terrain, loader);
            self.last_async_cell = cell;
        }
        if self.waiting {
            self.blocking = blocking;
        } else {
            self.finish_update_visible_cells(cell, terrain, loader);
            self.blocking = false;
            self.context = None;
        }
    }

    /// `CellManager::ReleaseCells`.
    fn release_cells(&mut self, terrain: &mut Terrain) {
        for block in std::mem::take(&mut self.resident) {
            terrain.remove_block(block);
            self.changes.push(BlockChange::Released(block));
        }
        self.window.clear();
        self.context = None;
        self.last_async_cell = 0;
        self.waiting = false;
        self.load_cell = 0;
        self.current_cell = false;
        self.blocking = false;
    }

    /// `CellManager::AsyncGetCells`: the cell's block (an interior's with it) as ring 0.
    fn async_get_cells(&mut self, cell: u32, terrain: &mut Terrain, loader: &mut Loader) {
        self.waiting = true;
        self.context = Some(Context {
            cell,
            times_finished: 0,
            outstanding: BTreeSet::new(),
        });
        if !self.request_ring(block_of(cell), 0, true, loader) {
            self.waiting = false;
            return;
        }
        self.advance(terrain, loader);
    }

    /// Asks for one ring around `center`; returns whether the ring has any block on the map.
    /// Blocks already loaded count as done at once.
    fn request_ring(
        &mut self,
        center: u32,
        radius: i32,
        sub_data: bool,
        loader: &mut Loader,
    ) -> bool {
        let blocks = ring(center, radius);
        let Some(context) = &mut self.context else {
            return false;
        };
        for &block in &blocks {
            if sub_data {
                if self.resident.contains(&block) || self.prepared.contains_key(&block) {
                    continue;
                }
                if self.pending.insert(block) {
                    loader.request(Request::Block(block));
                }
            } else {
                let near = self.resident.contains(&block) || self.prepared.contains_key(&block);
                if near || self.far_cached.contains(&block) {
                    continue;
                }
                if self.far_pending.insert(block) {
                    loader.request(Request::FarBlock(block));
                }
            }
            context.outstanding.insert(block);
        }
        !blocks.is_empty()
    }

    /// Runs `OnTopLevelRequestFinished` for every ring that has nothing left to load.
    fn advance(&mut self, terrain: &Terrain, loader: &mut Loader) {
        loop {
            let Some(context) = &mut self.context else {
                return;
            };
            if !self.waiting || !context.outstanding.is_empty() {
                return;
            }
            context.times_finished += 1;
            let (cell, n) = (context.cell, context.times_finished);
            if !is_outside(cell) && n == 1 {
                // An interior seen only from inside loads no landscape around it.
                let seen_outside = terrain
                    .env_cell(cell)
                    .or_else(|| self.prepared_env_cell(cell))
                    .is_some_and(|env| env.seen_outside);
                if !seen_outside {
                    self.waiting = false;
                    return;
                }
            }
            let max = FAR_RADIUS + 1 + NEAR_RADIUS;
            let more = if n - 1 < NEAR_RADIUS {
                self.request_ring(block_of(cell), n, true, loader)
            } else if n - 1 < max {
                self.request_ring(block_of(cell), n, false, loader)
            } else {
                // `RequestAsyncLoadBlockRadiusSceneObjects`: the near blocks' objects came with
                // their sub-data, so nothing is left.
                false
            };
            if !more {
                // `OnContextFinished`.
                self.waiting = false;
                return;
            }
        }
    }

    fn prepared_env_cell(&self, cell: u32) -> Option<&ac2_physics::EnvCell> {
        self.prepared.get(&block_of(cell))?.as_ref()?.env_cell(cell)
    }

    /// A loader result: a near block joins the resident window it belongs to (or waits in the
    /// cache), and the current ring moves on once it is complete.
    pub(crate) fn loaded(
        &mut self,
        loaded: Loaded,
        terrain: &mut Terrain,
        loader: &mut Loader,
    ) -> Result<(), String> {
        let block = match loaded {
            Loaded::Block(block, result) => {
                self.pending.remove(&block);
                let world = result?;
                if self.window.contains(&block) && !self.resident.contains(&block) {
                    if let Some(world) = world {
                        terrain.absorb(world);
                        self.resident.insert(block);
                        self.changes.push(BlockChange::Resident(block));
                    }
                } else {
                    self.prepared.insert(block, world);
                }
                block
            }
            Loaded::FarBlock(block, result) => {
                self.far_pending.remove(&block);
                result?;
                self.far_cached.insert(block);
                block
            }
            Loaded::Body(..) | Loaded::ScriptState(..) => return Ok(()),
        };
        if let Some(context) = &mut self.context {
            if context.outstanding.remove(&block) {
                self.advance(terrain, loader);
            }
        }
        Ok(())
    }

    /// `CellManager::FinishUpdateVisibleCells`: when the player's cell changed (or was not
    /// resident), the near window moves around it.
    fn finish_update_visible_cells(
        &mut self,
        cell: u32,
        terrain: &mut Terrain,
        loader: &mut Loader,
    ) {
        if self.load_cell != cell || !self.current_cell {
            self.update_window(cell, terrain, loader);
            self.current_cell = if is_outside(cell) {
                terrain.land_cell(cell).is_some()
            } else {
                terrain.env_cell(cell).is_some()
            };
        }
        self.load_cell = cell;
    }

    /// `LScapeManager::UpdateVisibleCells`: the blocks within the near radius become resident
    /// (from the cache, or when their load finishes); the others are released.
    fn update_window(&mut self, cell: u32, terrain: &mut Terrain, loader: &mut Loader) {
        let window: BTreeSet<u32> = block_radius(cell, NEAR_RADIUS).into_iter().collect();
        for block in self
            .resident
            .difference(&window)
            .copied()
            .collect::<Vec<_>>()
        {
            terrain.remove_block(block);
            self.resident.remove(&block);
            self.changes.push(BlockChange::Released(block));
        }
        for &block in &window {
            if self.resident.contains(&block) {
                continue;
            }
            match self.prepared.remove(&block) {
                Some(Some(world)) => {
                    terrain.absorb(world);
                    self.resident.insert(block);
                    self.changes.push(BlockChange::Resident(block));
                }
                Some(None) => {
                    self.prepared.insert(block, None);
                }
                None => {
                    if self.pending.insert(block) {
                        loader.request(Request::Block(block));
                    }
                }
            }
        }
        self.window = window;
        // The cache keeps one block beyond the window, the far set the whole far radius.
        let keep: BTreeSet<u32> = block_radius(cell, NEAR_RADIUS + 1).into_iter().collect();
        self.prepared.retain(|block, _| keep.contains(block));
        let far: BTreeSet<u32> = block_radius(cell, NEAR_RADIUS + FAR_RADIUS + 1)
            .into_iter()
            .collect();
        self.far_cached.retain(|block| far.contains(block));
    }
}

#[cfg(test)]
#[path = "cells_tests.rs"]
mod tests;
