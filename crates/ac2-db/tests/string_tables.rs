//! Every string table in the local DAT decodes to its last byte, and its text reads as text
//! once descrambled (`AC2_DAT_DIR`).

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{load_string_table, ResourceStore, STRING_TABLE_FIRST, STRING_TABLE_LAST};

#[test]
fn every_string_table_decodes_and_reads_as_text() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Local, STRING_TABLE_FIRST, STRING_TABLE_LAST)
        .collect();
    assert!(!ids.is_empty());
    let store = ResourceStore::new(dataset);
    let (mut strings, mut units, mut printable) = (0_usize, 0_usize, 0_usize);
    for did in &ids {
        let table = load_string_table(&store, DataId::new(*did))
            .unwrap_or_else(|e| panic!("{did:08x}: {e}"))
            .unwrap_or_else(|| panic!("{did:08x} missing"));
        strings += table.strings.len();
        for entry in table.strings.values() {
            for unit in entry.pieces.iter().flatten().flatten() {
                units += 1;
                let printable_unit = (0x20..0x7F).contains(unit)
                    || matches!(unit, 0x09 | 0x0A | 0x0D)
                    || *unit >= 0xA0;
                printable += usize::from(printable_unit);
            }
        }
    }
    eprintln!(
        "{} tables, {strings} strings, {units} UTF-16 units",
        ids.len()
    );
    // Scrambled text would be mostly control and odd code points.
    assert!(
        printable * 100 >= units * 99,
        "{printable} of {units} printable"
    );
}
