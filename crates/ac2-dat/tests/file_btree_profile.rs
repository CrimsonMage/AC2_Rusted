use std::path::{Path, PathBuf};

use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions, DatReadAttribution};
use tempfile::TempDir;

const BLOCK_SIZE: usize = 1024;
const BLOCK_DATA_SIZE: usize = BLOCK_SIZE - 4;
const NODE_SIZE: usize = 1720;
const ROOT_BLOCK_OFFSET: usize = BLOCK_SIZE;
const PAYLOAD_BLOCK_OFFSET: usize = BLOCK_SIZE * 3;
const FILE_ID: u32 = 0x1000_0001;

#[test]
fn file_btree_profile_counts_multi_block_root_as_one_logical_node_load() {
    let fixture = SmallBlockNodeFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");

    // `BTree::LoadTree` keeps the root resident. This deliberately uses 1024-byte DAT blocks,
    // requiring two retail-shaped header-plus-data reads for the 1720-byte root node.
    let before = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(before.btree_node_loads, 1);
    assert_eq!(before.btree_node_positioned_read_calls, 2);
    assert_eq!(
        before.btree_node_positioned_read_bytes,
        (NODE_SIZE + 8) as u64
    );
    assert_eq!(before.parsed_node_cache_entries, 0);

    let record = dataset
        .read_record_owned(
            DatFileKind::Portal,
            FILE_ID,
            DatReadAttribution::ActiveWorldScoped,
        )
        .expect("root lookup should succeed")
        .expect("fixture entry should exist");
    assert_eq!(record.packed_bytes, fixture.payload);

    let after = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(after.did_lookup_requests - before.did_lookup_requests, 1);
    assert_eq!(after.did_lookup_found - before.did_lookup_found, 1);
    assert_eq!(after.btree_nodes_visited - before.btree_nodes_visited, 1);
    assert_eq!(after.btree_root_visits - before.btree_root_visits, 1);
    assert_eq!(after.btree_node_loads, before.btree_node_loads);
    assert_eq!(
        after.btree_node_positioned_read_calls,
        before.btree_node_positioned_read_calls
    );
    assert_eq!(
        after.btree_node_positioned_read_bytes,
        before.btree_node_positioned_read_bytes
    );
    assert_eq!(after.payload_read_calls - before.payload_read_calls, 1);
    assert_eq!(after.payload_blocks_total - before.payload_blocks_total, 2);
    assert_eq!(
        after.payload_positioned_read_calls - before.payload_positioned_read_calls,
        2
    );
    assert_eq!(
        after.payload_positioned_read_bytes - before.payload_positioned_read_bytes,
        (fixture.payload.len() + 8) as u64
    );
}

#[test]
fn file_btree_short_range_reads_only_the_required_prefix() {
    let fixture = SmallBlockNodeFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");
    let before = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");

    let range = dataset
        .read_range(DatFileKind::Portal, FILE_ID, 5, 7)
        .expect("short range should succeed")
        .expect("fixture entry should exist");
    assert_eq!(range, fixture.payload[5..12]);

    let after = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(after.range_read_requests - before.range_read_requests, 1);
    assert_eq!(
        after.range_requested_bytes - before.range_requested_bytes,
        7
    );
    assert_eq!(after.payload_read_calls - before.payload_read_calls, 1);
    assert_eq!(after.payload_blocks_total - before.payload_blocks_total, 1);
    assert_eq!(
        after.payload_positioned_read_calls - before.payload_positioned_read_calls,
        1
    );
    // The 5..12 range needs only the first 12 packed bytes plus this DAT block's link header.
    assert_eq!(
        after.payload_positioned_read_bytes - before.payload_positioned_read_bytes,
        16
    );
}

/// Payload chains are read outside the lookup lock; concurrent loaders must still see exactly
/// the serial bytes for whole records and short ranges.
#[test]
fn concurrent_file_btree_reads_match_serial_bytes() {
    let fixture = SmallBlockNodeFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");
    let before = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    const THREADS: u64 = 8;
    const READS: u64 = 64;
    std::thread::scope(|scope| {
        for thread in 0..THREADS {
            let (dataset, payload) = (&dataset, &fixture.payload);
            scope.spawn(move || {
                for read in 0..READS {
                    if (thread + read) % 2 == 0 {
                        let record = dataset
                            .read_record_owned(
                                DatFileKind::Portal,
                                FILE_ID,
                                DatReadAttribution::ActiveWorldScoped,
                            )
                            .expect("record read should succeed")
                            .expect("fixture entry should exist");
                        assert_eq!(record.packed_bytes, *payload);
                    } else {
                        let range = dataset
                            .read_range(DatFileKind::Portal, FILE_ID, 1000, 50)
                            .expect("range read should succeed")
                            .expect("fixture entry should exist");
                        assert_eq!(range, payload[1000..1050]);
                    }
                }
            });
        }
    });
    let after = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(
        after.did_lookup_found - before.did_lookup_found,
        THREADS * READS
    );
    assert_eq!(after.btree_node_loads, before.btree_node_loads);
}

struct SmallBlockNodeFixture {
    temp: TempDir,
    payload: Vec<u8>,
}

impl SmallBlockNodeFixture {
    fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let payload = (0..BLOCK_DATA_SIZE + 80)
            .map(|index| index as u8)
            .collect::<Vec<_>>();
        for (kind, dataset_type) in [
            (DatFileKind::Portal, 1),
            (DatFileKind::Cell, 2),
            (DatFileKind::Local, 3),
        ] {
            write_small_block_fixture(
                &temp.path().join(kind.default_file_name()),
                dataset_type,
                &payload,
            )?;
        }
        Ok(Self { temp, payload })
    }

    fn path(&self) -> &Path {
        self.temp.path()
    }
}

fn write_small_block_fixture(
    path: &PathBuf,
    file_type: u32,
    payload: &[u8],
) -> std::io::Result<()> {
    let mut bytes = vec![0u8; BLOCK_SIZE * 5];
    write_header(&mut bytes, file_type);

    let mut root = vec![0u8; NODE_SIZE];
    write_i32(&mut root, 62 * 4, 1);
    write_file_entry(
        &mut root,
        62 * 4 + 4,
        FILE_ID,
        PAYLOAD_BLOCK_OFFSET as i32,
        payload.len() as u32,
    );
    write_block_chain(&mut bytes, ROOT_BLOCK_OFFSET, &root);
    write_block_chain(&mut bytes, PAYLOAD_BLOCK_OFFSET, payload);

    std::fs::write(path, bytes)
}

fn write_header(bytes: &mut [u8], file_type: u32) {
    write_i32(bytes, 320, 0x0000_5442);
    write_i32(bytes, 324, BLOCK_SIZE as i32);
    write_i32(bytes, 328, bytes.len() as i32);
    bytes[332..336].copy_from_slice(&file_type.to_le_bytes());
    write_i32(bytes, 352, ROOT_BLOCK_OFFSET as i32);
}

fn write_file_entry(bytes: &mut [u8], offset: usize, did: u32, block_offset: i32, size: u32) {
    bytes[offset + 4..offset + 8].copy_from_slice(&did.to_le_bytes());
    write_i32(bytes, offset + 8, block_offset);
    bytes[offset + 12..offset + 16].copy_from_slice(&size.to_le_bytes());
    write_i32(bytes, offset + 20, 1);
}

fn write_block_chain(bytes: &mut [u8], start: usize, payload: &[u8]) {
    for (index, chunk) in payload.chunks(BLOCK_DATA_SIZE).enumerate() {
        let offset = start + index * BLOCK_SIZE;
        let next = if index + 1 < payload.len().div_ceil(BLOCK_DATA_SIZE) {
            (offset + BLOCK_SIZE) as i32
        } else {
            0
        };
        write_i32(bytes, offset, next);
        bytes[offset + 4..offset + 4 + chunk.len()].copy_from_slice(chunk);
    }
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
