use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ac2_physics::Terrain;

use super::*;
use crate::blocks::{BlockPool, BlockSource};

/// Flat blocks everywhere; counts the near and far loads.
#[derive(Default)]
struct Counting {
    near: AtomicUsize,
    far: AtomicUsize,
}

impl BlockSource for Counting {
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String> {
        self.near.fetch_add(1, Ordering::SeqCst);
        terrain.insert_heights(block, &[0.0; 17 * 17], |_, _| false);
        Ok(true)
    }

    fn prefetch_far_block(&self, _block: u32) -> Result<(), String> {
        self.far.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Takes every load in as it finishes.
fn settle(cells: &mut CellManager, terrain: &mut Terrain, loader: &mut BlockPool) {
    while loader.busy() {
        let loaded = loader.wait(Duration::from_secs(5)).expect("load");
        cells.loaded(loaded, terrain, loader).unwrap();
    }
}

#[test]
fn rings_load_out_to_the_far_radius_before_the_wait_ends() {
    let data = Arc::new(Counting::default());
    let mut loader = BlockPool::new(data.clone(), 2);
    let mut terrain = Terrain::new();
    let mut cells = CellManager::default();
    let cell = 0x7F7F_001C;
    cells.update_visible_cells(cell, true, &mut terrain, &mut loader);
    assert!(cells.is_blocking() && cells.is_waiting());
    assert_eq!(cells.resident().count(), 0);
    settle(&mut cells, &mut terrain, &mut loader);
    assert!(!cells.is_waiting());
    // Still blocking until the next `UpdateVisibleCells` finishes the update.
    assert!(cells.is_blocking());
    cells.update_visible_cells(cell, false, &mut terrain, &mut loader);
    assert!(!cells.is_blocking());
    assert_eq!(cells.resident().count(), 9);
    assert!(terrain.land_cell(cell).is_some());
    // Rings 0 and 1 with sub-data, rings 2 to 6 alone (13 x 13 - 3 x 3 = 160 blocks).
    assert_eq!(data.near.load(Ordering::SeqCst), 9);
    assert_eq!(data.far.load(Ordering::SeqCst), 160);

    // A move within the block asks for nothing; into the next block east the window follows
    // once that block's rings are in, and only the new column loads with sub-data.
    cells.update_visible_cells(0x7F7F_0024, false, &mut terrain, &mut loader);
    assert!(!loader.busy());
    let east = 0x807F_0004;
    cells.update_visible_cells(east, false, &mut terrain, &mut loader);
    assert!(cells.is_waiting() && !cells.is_blocking());
    // While the request is out the old window stays.
    assert!(cells.resident().any(|b| b == 0x7E7F_0000));
    settle(&mut cells, &mut terrain, &mut loader);
    cells.update_visible_cells(east, false, &mut terrain, &mut loader);
    let resident: Vec<u32> = cells.resident().collect();
    assert_eq!(resident, block_radius(east, 1));
    assert_eq!(data.near.load(Ordering::SeqCst), 12);
    assert!(terrain.land_cell(0x7E7F_0001).is_none());
}

#[test]
fn releasing_the_cells_empties_the_world() {
    let data = Arc::new(Counting::default());
    let mut loader = BlockPool::new(data, 1);
    let mut terrain = Terrain::new();
    let mut cells = CellManager::default();
    cells.update_visible_cells(0x7F7F_001C, true, &mut terrain, &mut loader);
    settle(&mut cells, &mut terrain, &mut loader);
    cells.update_visible_cells(0x7F7F_001C, false, &mut terrain, &mut loader);
    assert_eq!(cells.resident().count(), 9);
    cells.update_visible_cells(0, false, &mut terrain, &mut loader);
    assert_eq!(cells.resident().count(), 0);
    assert!(terrain.land_cell(0x7F7F_001C).is_none());
}

#[test]
fn rings_hold_the_blocks_at_their_radius() {
    assert_eq!(ring(0x7F7F_0000, 0), vec![0x7F7F_0000]);
    let first = ring(0x7F7F_0000, 1);
    assert_eq!(first.len(), 8);
    assert!(!first.contains(&0x7F7F_0000));
    assert_eq!(ring(0x7F7F_0000, 2).len(), 16);
    // At the map's corner only the on-map quarter is left.
    assert_eq!(
        ring(0x0000_0000, 1),
        vec![0x0001_0000, 0x0100_0000, 0x0101_0000]
    );
}
