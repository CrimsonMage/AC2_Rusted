//! `CDetailMapDesc` and `CMapNoteDesc` over the Legions DATs (`AC2_DAT_DIR`). The expectations
//! come from the record's structure: a row is a rectangle of blocks in the block map, every block
//! of that rectangle holds the row's value, and a map note lists a `CMapNoteDesc` record.

use std::collections::HashSet;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use super::detail_map::{ac2_detail_map, ac2_map_note_desc, DetailMap};

/// `CDetailMapDesc` records (`0x59000000..=0x59000FFE`, `GetDBOType` 0x5A).
const DETAIL_MAP_FIRST: u32 = 0x5900_0000;
const DETAIL_MAP_LAST: u32 = 0x5900_0FFE;
/// `CMapNoteDesc` records (`0x5900F000..=0x59FFFFFE`, DB type 0x5E).
const MAP_NOTE_FIRST: u32 = 0x5900_F000;
const MAP_NOTE_LAST: u32 = 0x59FF_FFFE;
/// The block map's blocks per side: the land's 255 blocks (`LandDefs::land_width`, 8 cells each)
/// and the edge block the last tiles reach.
const BLOCK_SIDE: i32 = 256;

fn store() -> Option<ResourceStore> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(ResourceStore::new(Arc::clone(&dataset)))
}

fn detail_maps(store: &ResourceStore) -> Vec<(u32, DetailMap)> {
    store
        .dataset()
        .ids_in_range(DatFileKind::Portal, DETAIL_MAP_FIRST, DETAIL_MAP_LAST)
        .map(|did| {
            let map = ac2_detail_map(store, DataId::new(did))
                .expect("detail map decodes to the last byte")
                .expect("detail map record exists");
            (did, map)
        })
        .collect()
}

#[test]
fn every_detail_map_record_decodes_and_the_region_names_one() {
    let Some(store) = store() else {
        return;
    };
    let maps = detail_maps(&store);
    assert_eq!(maps.len(), 1, "one CDetailMapDesc in the Legions DAT");
    let table = crate::ac2_region_detail_map_table(&store)
        .expect("region decodes")
        .expect("region names a detail map table");
    assert_eq!(table.raw(), maps[0].0, "the region's table is that record");
    for (did, map) in &maps {
        assert!(!map.rows().is_empty(), "{did:08X} has rows");
        let values: HashSet<u32> = map.rows().iter().map(|row| row.value).collect();
        assert_eq!(values.len(), map.rows().len(), "row values are unique");
    }
}

#[test]
fn every_row_is_a_tile_of_its_own_value_in_the_block_map() {
    let Some(store) = store() else {
        return;
    };
    for (did, map) in detail_maps(&store) {
        for row in map.rows() {
            assert!(
                row.width > 0 && row.height > 0,
                "{did:08X} row {}",
                row.value
            );
            assert!(
                (row.x0 + row.width) as i32 <= BLOCK_SIDE
                    && (row.y0 + row.height) as i32 <= BLOCK_SIDE,
                "row {} lies on the land",
                row.value
            );
            for x in row.x0..row.x0 + row.width {
                for y in row.y0..row.y0 + row.height {
                    let found = map.row_from_block(x as i32, y as i32);
                    assert_eq!(
                        found.map(|r| r.value),
                        Some(row.value),
                        "block ({x}, {y}) is in row {}",
                        row.value
                    );
                }
            }
        }
        // Blocks no row covers look up nothing (the block map holds 0 there).
        assert!(map.row_from_block(0, 0).is_none());
        assert!(map.row_from_block(BLOCK_SIDE, BLOCK_SIDE).is_none());
    }
}

#[test]
fn a_known_cell_finds_the_row_it_stands_in() {
    let Some(store) = store() else {
        return;
    };
    let maps = detail_maps(&store);
    let map = &maps[0].1;
    // The first row, probed at its middle block and at the middle of its middle cell: the
    // cell's land coordinates are the block's coordinates times 8 (`lblock_shift` 3).
    let row = &map.rows()[0];
    let block = (
        (row.x0 + row.width / 2) as i32,
        (row.y0 + row.height / 2) as i32,
    );
    let cell = (block.0 * 8 + 4, block.1 * 8 + 4);
    let found = map
        .row_from_block(cell.0 >> 3, cell.1 >> 3)
        .expect("the cell's block is in a row");
    assert_eq!(found, row);
    assert_eq!(map.row_from_value(row.value), Some(row));
    assert_eq!(map.row_from_value(u32::MAX), None);
}

#[test]
fn every_map_note_record_decodes_and_each_row_lists_known_notes() {
    let Some(store) = store() else {
        return;
    };
    let notes: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, MAP_NOTE_FIRST, MAP_NOTE_LAST)
        .collect();
    assert_eq!(
        notes.len(),
        25,
        "the Legions DAT has 25 CMapNoteDesc records"
    );
    let decoded: HashSet<u32> = notes
        .iter()
        .map(|did| {
            let desc = ac2_map_note_desc(&store, DataId::new(*did))
                .expect("map note decodes to the last byte")
                .expect("map note record exists");
            assert_ne!(desc.string_table_did, 0, "{did:08X} names its table");
            for note in &desc.rows {
                assert!(
                    (0..255 * 8).contains(&note.x) && (0..255 * 8).contains(&note.y),
                    "{did:08X} note {:08X} lies on the land",
                    note.string_id
                );
            }
            *did
        })
        .collect();
    for (_, map) in detail_maps(&store) {
        for row in map.rows() {
            for note in &row.map_notes {
                assert!(
                    decoded.contains(note),
                    "row lists the note record {note:08X}"
                );
            }
        }
    }
}
