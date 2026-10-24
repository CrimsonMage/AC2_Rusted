//! `CellManager::UpdateVisibleCells`: the landblocks resident around the player. When the
//! player's cell leaves its block (or enters an interior), the blocks within one block of it
//! load (`RequestAsyncLoadBlockRadius`; this client loads them at once, as a blocking load
//! does) and those further away are released.

use std::collections::BTreeSet;

use ac2_physics::Terrain;

use crate::world_data::WorldData;

/// The resident landblocks (`xxyy0000`).
#[derive(Debug, Default)]
pub struct CellManager {
    last_cell: Option<u32>,
    resident: BTreeSet<u32>,
}

/// `LandDefs::first_envcell_id`.
const FIRST_ENV_CELL: u32 = 0x100;

/// The block of `cell` and its eight neighbours that lie on the map.
pub fn block_radius(cell: u32, radius: i32) -> Vec<u32> {
    let x = (cell >> 24) as i32;
    let y = ((cell >> 16) & 0xFF) as i32;
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

impl CellManager {
    pub fn resident(&self) -> impl Iterator<Item = u32> + '_ {
        self.resident.iter().copied()
    }

    /// Makes the blocks around `cell` resident; returns whether any changed.
    pub fn update_visible_cells(
        &mut self,
        cell: u32,
        terrain: &mut Terrain,
        data: &dyn WorldData,
    ) -> Result<bool, String> {
        if cell == 0 {
            for block in std::mem::take(&mut self.resident) {
                terrain.remove_block(block);
            }
            self.last_cell = None;
            return Ok(true);
        }
        let same_block = self.last_cell.is_some_and(|last| {
            cell & 0xFFFF < FIRST_ENV_CELL && last & 0xFFFF_0000 == cell & 0xFFFF_0000
        });
        if same_block || self.last_cell == Some(cell) {
            return Ok(false);
        }
        self.last_cell = Some(cell);
        let wanted: BTreeSet<u32> = block_radius(cell, 1).into_iter().collect();
        for block in self
            .resident
            .difference(&wanted)
            .copied()
            .collect::<Vec<_>>()
        {
            terrain.remove_block(block);
            self.resident.remove(&block);
        }
        for block in wanted {
            if !self.resident.contains(&block) && data.load_block(terrain, block)? {
                self.resident.insert(block);
            }
        }
        Ok(true)
    }
}
