use std::path::Path;

use ac2_dat::{is_dat_mount_root, DatDataset, DatFileKind, DatOpenOptions, DatReadAttribution};
use tempfile::TempDir;

const HEADER_BASE: usize = 300;
const HEADER_MAGIC: i32 = 0x0000_5442;
const BLOCK_SIZE: usize = 1024;
const NODE_SIZE: usize = 1228;
const ROOT_BLOCK: usize = BLOCK_SIZE;
const ROOT_TAIL_BLOCK: usize = BLOCK_SIZE * 2;
const PAYLOAD_BLOCK: usize = BLOCK_SIZE * 3;
const TEST_DID: u32 = 0x4100_0042;

#[test]
fn ac2_header_node_entry_and_native_filenames_resolve() {
    let root = TempDir::new().expect("AC2 fixture root");
    for (kind, raw_type) in [
        (DatFileKind::Portal, 1),
        (DatFileKind::Cell, 2),
        (DatFileKind::Local, 3),
    ] {
        write_ac2_dat(&root.path().join(kind.default_file_name()), raw_type);
    }
    assert!(is_dat_mount_root(root.path()));

    let options = DatOpenOptions::new(root.path());
    assert_eq!(
        options.path_for(DatFileKind::Portal),
        root.path().join(DatFileKind::Portal.default_file_name())
    );

    let dataset = DatDataset::open(options).expect("AC2 dataset should mount");
    let entry = dataset
        .entry(DatFileKind::Portal, TEST_DID)
        .expect("AC2 16-byte B-tree entry should resolve");
    assert_eq!(entry.packed_len, 12);
    assert_eq!(entry.flags, 0);
    assert_eq!(entry.version, 0);
    assert_eq!(entry.iteration, 0);
    assert_eq!(entry.date_stamp, 0x4238_B78C);
    assert_eq!(
        dataset.iter_ids(DatFileKind::Portal).collect::<Vec<_>>(),
        vec![TEST_DID]
    );

    let record = dataset
        .read_record(
            DatFileKind::Portal,
            TEST_DID,
            DatReadAttribution::EphemeralWorldTooling,
        )
        .expect("AC2 payload read should succeed")
        .expect("AC2 payload should exist");
    assert_eq!(record.packed_bytes, b"ac2-payload!");
}

fn write_ac2_dat(path: &Path, raw_type: u32) {
    let mut bytes = vec![0u8; BLOCK_SIZE * 4];
    let file_size = bytes.len() as i32;
    write_i32(&mut bytes, HEADER_BASE, HEADER_MAGIC);
    write_i32(&mut bytes, HEADER_BASE + 4, BLOCK_SIZE as i32);
    write_i32(&mut bytes, HEADER_BASE + 8, file_size);
    write_u32(&mut bytes, HEADER_BASE + 12, raw_type);
    write_u32(&mut bytes, HEADER_BASE + 16, 1);
    write_i32(&mut bytes, HEADER_BASE + 32, ROOT_BLOCK as i32);
    write_u32(&mut bytes, HEADER_BASE + 48, 0x2800_0000);

    let mut node = vec![0u8; NODE_SIZE];
    write_i32(&mut node, 62 * 4, 1);
    let entry = 62 * 4 + 4;
    write_u32(&mut node, entry, TEST_DID);
    write_i32(&mut node, entry + 4, PAYLOAD_BLOCK as i32);
    write_u32(&mut node, entry + 8, 12);
    write_u32(&mut node, entry + 12, 0x4238_B78C);

    write_block_chain(&mut bytes, &node);
    write_i32(&mut bytes, PAYLOAD_BLOCK, 0);
    bytes[PAYLOAD_BLOCK + 4..PAYLOAD_BLOCK + 16].copy_from_slice(b"ac2-payload!");
    std::fs::write(path, bytes).expect("AC2 fixture DAT should write");
}

fn write_block_chain(bytes: &mut [u8], node: &[u8]) {
    let first_len = BLOCK_SIZE - 4;
    write_i32(bytes, ROOT_BLOCK, ROOT_TAIL_BLOCK as i32);
    bytes[ROOT_BLOCK + 4..ROOT_BLOCK + 4 + first_len].copy_from_slice(&node[..first_len]);
    write_i32(bytes, ROOT_TAIL_BLOCK, 0);
    bytes[ROOT_TAIL_BLOCK + 4..ROOT_TAIL_BLOCK + 4 + NODE_SIZE - first_len]
        .copy_from_slice(&node[first_len..]);
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Real-data check against an AC2 installation; runs only when `AC2_DAT_DIR` is set.
#[test]
fn ac2_installation_landblock_reads_when_available() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 installation mounts");
    let record = dataset
        .read_record(
            DatFileKind::Cell,
            0x7F7F_FFFF,
            DatReadAttribution::EphemeralWorldTooling,
        )
        .expect("AC2 landblock read")
        .expect("AC2 landblock exists");
    // DBObj id, landblock-info id, path-map id, 17x17 height bytes, 17x17 cell-info words.
    assert_eq!(record.packed_bytes.len(), 1468);
    assert_eq!(&record.packed_bytes[..4], &0x7F7F_FFFF_u32.to_le_bytes());
}
