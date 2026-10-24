use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;

use ac2_dat::{
    DatDataset, DatError, DatFileKind, DatIdStamp, DatOpenOptions, DatReadAttribution,
    DatStorageMode, RETAIL_DDD_ITERATION_DID,
};
use tempfile::TempDir;

const BLOCK_SIZE: usize = 2048;
const ROOT_BLOCK_OFFSET: usize = BLOCK_SIZE;
const LEFT_LEAF_BLOCK_OFFSET: usize = BLOCK_SIZE * 2;
const RIGHT_LEAF_BLOCK_OFFSET: usize = BLOCK_SIZE * 3;
const PAYLOAD_ONE_BLOCK_OFFSET: usize = BLOCK_SIZE * 4;
const PAYLOAD_TWO_FIRST_BLOCK_OFFSET: usize = BLOCK_SIZE * 5;
const PAYLOAD_TWO_SECOND_BLOCK_OFFSET: usize = BLOCK_SIZE * 6;
const PAYLOAD_THREE_BLOCK_OFFSET: usize = BLOCK_SIZE * 7;
const NODE_SIZE: usize = 1720;

const FILE_ONE_ID: u32 = 0x1000_0001;
const FILE_TWO_ID: u32 = 0x1000_0002;

fn legacy_mmap_options(path: &Path) -> DatOpenOptions {
    let mut options = DatOpenOptions::new(path);
    options.storage_mode = DatStorageMode::LegacyMmapFlatIndex;
    options
}

#[test]
fn opens_dataset_builds_flat_indexes_and_iterates_sorted_ids() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");

    assert!(dataset.contains(DatFileKind::Portal, FILE_ONE_ID));
    assert!(dataset.contains(DatFileKind::Portal, FILE_TWO_ID));
    assert!(dataset.contains(DatFileKind::Portal, RETAIL_DDD_ITERATION_DID));

    let ids = dataset.iter_ids(DatFileKind::Portal).collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![FILE_ONE_ID, FILE_TWO_ID, RETAIL_DDD_ITERATION_DID]
    );

    let entries = dataset
        .entries_in_range(DatFileKind::Portal, FILE_ONE_ID, RETAIL_DDD_ITERATION_DID)
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].did, FILE_ONE_ID);
    assert_eq!(entries[1].did, FILE_TWO_ID);
    assert_eq!(entries[2].did, RETAIL_DDD_ITERATION_DID);
    assert_eq!(entries[0].packed_len, fixture.payload_one.len() as u32);
    assert_eq!(
        entries[1].packed_len,
        fixture.payload_two_packed.len() as u32
    );
    assert_eq!(entries[0].version, 1);
    assert_eq!(entries[0].date_stamp, 0x0102_0304);
    assert!(!entries[0].is_compressed());
    assert!(entries[1].is_compressed());
}

#[test]
fn file_btree_mode_uses_owned_positioned_reads_without_eager_full_index() {
    let fixture = DatFixture::new().expect("fixture should build");
    let mut options = DatOpenOptions::new(fixture.path());
    options.storage_mode = DatStorageMode::FileBtreeRuntime;
    let dataset = DatDataset::open(options).expect("file-backed dataset should open");

    let initial = dataset
        .memory_stats(DatFileKind::Portal)
        .expect("portal memory stats should exist");
    assert_eq!(initial.mapped_bytes, 0);
    assert!(initial.full_index.is_none());

    let single = dataset
        .read_record_owned(
            DatFileKind::Portal,
            FILE_ONE_ID,
            DatReadAttribution::ActiveWorldScoped,
        )
        .expect("positioned read should succeed")
        .expect("fixture payload should exist");
    assert_eq!(single.meta.did, FILE_ONE_ID);
    assert_eq!(single.packed_bytes, fixture.payload_one);

    let multi = dataset
        .read_record_owned(
            DatFileKind::Portal,
            FILE_TWO_ID,
            DatReadAttribution::GenericResource,
        )
        .expect("positioned read should succeed")
        .expect("fixture payload should exist");
    assert_eq!(multi.packed_bytes, fixture.payload_two_packed);
    assert!(dataset.contains(DatFileKind::Portal, RETAIL_DDD_ITERATION_DID));

    let telemetry = dataset
        .read_telemetry(DatFileKind::Portal)
        .expect("portal telemetry should exist");
    assert_eq!(telemetry.full_index_reads, 0);
    assert_eq!(telemetry.attribution.active_world_scoped_requests, 1);
    assert_eq!(telemetry.attribution.generic_resource_requests, 1);

    // Iteration is tooling/rewrite work, not normal runtime lookup. It is therefore the explicit
    // point that materializes a flat index in the file-backed candidate.
    assert_eq!(
        dataset.iter_ids(DatFileKind::Portal).collect::<Vec<_>>(),
        vec![FILE_ONE_ID, FILE_TWO_ID, RETAIL_DDD_ITERATION_DID]
    );
    assert!(dataset
        .memory_stats(DatFileKind::Portal)
        .expect("portal memory stats should exist")
        .full_index
        .is_some());
}

#[test]
fn file_btree_profile_observes_pinned_root_lookup_and_positioned_payload_reads() {
    let fixture = DatFixture::new().expect("fixture should build");
    let mut options = DatOpenOptions::new(fixture.path());
    options.storage_mode = DatStorageMode::FileBtreeRuntime;
    let dataset = DatDataset::open(options).expect("file-backed dataset should open");

    let before = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(before.parsed_node_cache_capacity, 100);
    assert_eq!(before.parsed_node_cache_entries, 0);
    assert!(dataset
        .btree_profile(DatFileKind::Local)
        .is_some_and(|profile| profile.btree_node_loads >= 1));

    assert!(dataset.contains(DatFileKind::Portal, FILE_ONE_ID));
    assert_eq!(
        dataset
            .entry(DatFileKind::Portal, FILE_ONE_ID)
            .expect("fixture entry should exist")
            .did,
        FILE_ONE_ID
    );
    dataset
        .read_record_owned(
            DatFileKind::Portal,
            FILE_ONE_ID,
            DatReadAttribution::ActiveWorldScoped,
        )
        .expect("single positioned read should succeed")
        .expect("fixture payload should exist");
    dataset
        .read_record_owned(
            DatFileKind::Portal,
            FILE_TWO_ID,
            DatReadAttribution::ActiveWorldScoped,
        )
        .expect("multi-block positioned read should succeed")
        .expect("fixture payload should exist");
    assert!(!dataset.contains(DatFileKind::Portal, 0xDEAD_BEEF));

    let after = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(after.parsed_node_cache_capacity, 100);
    assert_eq!(after.parsed_node_cache_entries, 1);
    assert_eq!(
        after.metadata_only_requests - before.metadata_only_requests,
        3
    );
    assert_eq!(after.did_lookup_requests - before.did_lookup_requests, 5);
    assert_eq!(after.did_lookup_found - before.did_lookup_found, 4);
    assert_eq!(after.did_lookup_missing - before.did_lookup_missing, 1);
    assert_eq!(
        after.btree_nodes_visited - before.btree_nodes_visited,
        (after.btree_root_visits - before.btree_root_visits) * 2
    );
    assert_eq!(after.btree_root_visits - before.btree_root_visits, 5);
    assert!(after.btree_key_comparisons > before.btree_key_comparisons);
    assert_eq!(after.payload_read_calls - before.payload_read_calls, 2);
    assert_eq!(after.payload_blocks_total - before.payload_blocks_total, 3);
    assert_eq!(
        after.stored_compressed_reads - before.stored_compressed_reads,
        1
    );
    assert_eq!(after.btree_node_loads - before.btree_node_loads, 1);
    assert_eq!(
        after.btree_node_positioned_read_calls - before.btree_node_positioned_read_calls,
        1
    );
    assert_eq!(
        after.btree_node_positioned_read_bytes - before.btree_node_positioned_read_bytes,
        (NODE_SIZE + 4) as u64
    );
    assert_eq!(
        after.payload_positioned_read_bytes - before.payload_positioned_read_bytes,
        (fixture.payload_one.len() + fixture.payload_two_packed.len() + 12) as u64
    );
    assert_eq!(
        after.payload_positioned_read_calls - before.payload_positioned_read_calls,
        3
    );
}

#[test]
fn file_btree_serializes_concurrent_first_reads_of_one_child_node() {
    let fixture = DatFixture::new().expect("fixture should build");
    let mut options = DatOpenOptions::new(fixture.path());
    options.storage_mode = DatStorageMode::FileBtreeRuntime;
    let dataset = DatDataset::open(options).expect("file-backed dataset should open");
    let before = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(before.parsed_node_cache_entries, 0);

    let start = Arc::new(Barrier::new(3));
    thread::scope(|scope| {
        let mut readers = Vec::new();
        for _ in 0..2 {
            let dataset = Arc::clone(&dataset);
            let start = Arc::clone(&start);
            readers.push(scope.spawn(move || {
                start.wait();
                dataset
                    .read_record_owned(
                        DatFileKind::Portal,
                        FILE_ONE_ID,
                        DatReadAttribution::ActiveWorldScoped,
                    )
                    .expect("concurrent positioned read should succeed")
                    .expect("fixture payload should exist")
            }));
        }
        start.wait();
        for reader in readers {
            assert_eq!(
                reader.join().expect("reader should not panic").packed_bytes,
                fixture.payload_one
            );
        }
    });

    let after = dataset
        .btree_profile(DatFileKind::Portal)
        .expect("file-backed storage should publish its profile");
    assert_eq!(after.btree_node_loads - before.btree_node_loads, 1);
    assert_eq!(
        after.parsed_node_cache_misses - before.parsed_node_cache_misses,
        1
    );
    assert_eq!(
        after.parsed_node_cache_hits - before.parsed_node_cache_hits,
        1
    );
}

#[test]
fn memory_stats_report_mmap_and_logical_index_components_without_waiting() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset = DatDataset::open(legacy_mmap_options(fixture.path()))
        .expect("legacy mmap dataset should open");

    let stats = dataset
        .memory_stats(DatFileKind::Portal)
        .expect("portal memory stats should exist");
    assert_eq!(stats.file_kind, DatFileKind::Portal);
    assert_eq!(
        stats.mapped_path,
        fixture.path().join(DatFileKind::Portal.default_file_name())
    );
    assert_eq!(
        stats.mapped_bytes,
        std::fs::metadata(&stats.mapped_path).unwrap().len() as usize
    );
    assert!(stats.startup_index.record_count > 0);
    assert!(stats.startup_index.did_bytes > 0);
    assert!(stats.startup_index.payload_bytes > 0);
    assert!(stats.startup_index.prefix_directory_bytes > 0);
    assert_eq!(
        stats.startup_index.segment_metadata_capacity_bytes,
        256 * 1024
    );
    // Full-index publication is asynchronous. Sampling must expose its current state, not block
    // a caller or alter index construction merely to make a diagnostic report look complete.
    let _ = stats.full_index;
}

#[test]
fn ddd_iteration_payload_is_visible_through_btree_and_flat_index() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");

    assert!(dataset.contains(DatFileKind::Portal, RETAIL_DDD_ITERATION_DID));

    let from_index = dataset
        .ddd_iteration(DatFileKind::Portal)
        .expect("flat index iteration query should succeed")
        .expect("DDD iteration list should exist");
    let from_btree = dataset
        .ddd_iteration_via_btree(DatFileKind::Portal)
        .expect("B-tree iteration query should succeed")
        .expect("DDD iteration list should exist");

    assert_eq!(from_index, from_btree);
    assert_eq!(from_index.current_iteration, 5);
    assert_eq!(from_index.mostly_consecutive_values(), vec![-3, 1, -2, 4]);
}

#[test]
fn packed_reads_use_zero_copy_for_single_block_and_segmented_views_for_multi_block() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset = DatDataset::open(legacy_mmap_options(fixture.path()))
        .expect("legacy mmap dataset should open");

    let single = dataset
        .read_packed(DatFileKind::Portal, FILE_ONE_ID)
        .expect("read should succeed")
        .expect("single-block payload should exist");
    assert_eq!(single.as_slice(), Some(fixture.payload_one.as_slice()));

    let multi = dataset
        .read_packed(DatFileKind::Portal, FILE_TWO_ID)
        .expect("read should succeed")
        .expect("multi-block payload should exist");
    assert!(multi.as_slice().is_none());
    assert_eq!(multi.copy_to_vec(), fixture.payload_two_packed);
    assert_eq!(multi.segments().count(), 2);
}

#[test]
fn combined_record_read_returns_one_metadata_and_packed_view_with_attribution() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset = DatDataset::open(legacy_mmap_options(fixture.path()))
        .expect("legacy mmap dataset should open");

    let record = dataset
        .read_record(
            DatFileKind::Portal,
            FILE_ONE_ID,
            DatReadAttribution::ActiveWorldScoped,
        )
        .expect("combined read should succeed")
        .expect("fixture record should exist");
    assert_eq!(record.meta.did, FILE_ONE_ID);
    assert_eq!(record.meta.version, 1);
    assert_eq!(
        record.packed.as_slice(),
        Some(fixture.payload_one.as_slice())
    );

    let multi = dataset
        .read_record(
            DatFileKind::Portal,
            FILE_TWO_ID,
            DatReadAttribution::GenericResource,
        )
        .expect("combined multi-block read should succeed")
        .expect("fixture record should exist");
    assert!(multi.meta.is_stored_compressed());
    assert!(multi.packed.as_slice().is_none());
    assert_eq!(multi.packed.copy_to_vec(), fixture.payload_two_packed);

    let telemetry = dataset
        .read_telemetry(DatFileKind::Portal)
        .expect("portal telemetry should exist");
    assert_eq!(telemetry.attribution.active_world_scoped_requests, 1);
    assert_eq!(
        telemetry.attribution.active_world_scoped_returned_bytes,
        fixture.payload_one.len() as u64
    );
    assert_eq!(telemetry.attribution.generic_resource_requests, 1);
    assert_eq!(
        telemetry.attribution.generic_resource_returned_bytes,
        fixture.payload_two_packed.len() as u64
    );
}

#[test]
fn read_telemetry_aggregates_read_paths_and_observable_owned_copies() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset = DatDataset::open(legacy_mmap_options(fixture.path()))
        .expect("legacy mmap dataset should open");

    let single = dataset
        .read_packed(DatFileKind::Portal, FILE_ONE_ID)
        .expect("read should succeed")
        .expect("single-block payload should exist");
    assert_eq!(single.copy_to_vec(), fixture.payload_one);

    let multi = dataset
        .read_packed(DatFileKind::Portal, FILE_TWO_ID)
        .expect("read should succeed")
        .expect("multi-block payload should exist");
    assert_eq!(multi.copy_to_vec(), fixture.payload_two_packed);

    assert!(dataset
        .read_packed(DatFileKind::Portal, 0xDEAD_BEEF)
        .expect("missing read should succeed")
        .is_none());

    let summary = dataset
        .read_telemetry(DatFileKind::Portal)
        .expect("portal telemetry should exist");
    assert_eq!(summary.total_read_requests, 3);
    assert_eq!(summary.successful_reads, 2);
    assert_eq!(summary.missing_did_reads, 1);
    assert_eq!(summary.direct_single_block_reads, 1);
    assert_eq!(summary.segmented_multi_block_reads, 1);
    assert_eq!(summary.contiguous_reads_copied_to_owned_storage, 1);
    assert_eq!(
        summary.contiguous_read_bytes,
        fixture.payload_one.len() as u64
    );
    assert_eq!(
        summary.segmented_read_bytes,
        fixture.payload_two_packed.len() as u64
    );
    assert!(summary.startup_index_reads + summary.full_index_reads >= 3);
    // Startup-index hits and direct B-tree fallbacks may satisfy this short fixture before the
    // background full index is published, so this sequence does not promise a chain-cache miss.
}

#[test]
fn stored_uncompressed_reads_stay_zero_copy_and_reject_compressed_storage() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");

    let single = dataset
        .read_stored_uncompressed(DatFileKind::Portal, FILE_ONE_ID)
        .expect("uncompressed storage read should succeed")
        .expect("single-block payload should exist");
    assert_eq!(single.as_slice(), Some(fixture.payload_one.as_slice()));

    let error = dataset
        .read_stored_uncompressed(DatFileKind::Portal, FILE_TWO_ID)
        .expect_err("compressed storage should be rejected");
    assert!(matches!(
        error,
        DatError::StoredCompressedEntry {
            kind: DatFileKind::Portal,
            did: FILE_TWO_ID,
        }
    ));
}

#[test]
fn small_range_reads_copy_exact_subranges_across_block_boundaries() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");

    let offset = 2000u64;
    let len = 128usize;
    let range = dataset
        .read_range(DatFileKind::Portal, FILE_TWO_ID, offset, len)
        .expect("range read should succeed")
        .expect("payload should exist");

    assert_eq!(
        range,
        fixture.payload_two_packed[offset as usize..offset as usize + len].to_vec()
    );
}

#[test]
fn exposes_file_metadata_and_uncompressed_sizes() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");

    let meta = dataset
        .file_meta(DatFileKind::Portal)
        .expect("portal metadata should exist");
    assert_eq!(meta.dataset_type, 1);
    assert_eq!(meta.subset, 0xCAFE_BABE);
    assert_eq!(meta.master_map_did, 0x1234_5678);
    assert_eq!(meta.engine_pack_version, 77);
    assert_eq!(meta.game_pack_version, 88);
    assert_eq!(
        meta.dat_id_stamp,
        DatIdStamp {
            major: [0xAB; 16],
            minor: 0xDEAD_BEEF,
        }
    );

    assert_eq!(
        dataset
            .uncompressed_size(DatFileKind::Portal, FILE_ONE_ID)
            .expect("size query should succeed"),
        Some(fixture.payload_one.len() as u32)
    );
    assert_eq!(
        dataset
            .uncompressed_size(DatFileKind::Portal, FILE_TWO_ID)
            .expect("size query should succeed"),
        Some(fixture.payload_two_uncompressed_len as u32)
    );
}

#[test]
fn open_reports_missing_required_files() {
    let temp = TempDir::new().expect("tempdir should exist");
    write_fixture_file(
        &temp.path().join(DatFileKind::Portal.default_file_name()),
        1,
        b"portal",
        &build_multi_block_payload(),
        &build_iteration_payload(),
        6000,
    )
    .expect("portal fixture should write");

    let error = DatDataset::open(DatOpenOptions::new(temp.path())).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("missing"));
    assert!(message.contains("Cell"));
}

#[test]
fn open_reports_missing_local_after_portal_and_cell_exist() {
    let temp = TempDir::new().expect("tempdir should exist");
    for (kind, dataset_type) in [(DatFileKind::Portal, 1), (DatFileKind::Cell, 2)] {
        write_fixture_file(
            &temp.path().join(kind.default_file_name()),
            dataset_type,
            b"payload",
            &build_multi_block_payload(),
            &build_iteration_payload(),
            6000,
        )
        .expect("fixture DAT should write");
    }

    let error = DatDataset::open(DatOpenOptions::new(temp.path())).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("missing"));
    assert!(message.contains("Local"));
}

#[test]
fn arc_clone_shares_one_open_dataset_handle() {
    let fixture = DatFixture::new().expect("fixture should build");
    let dataset =
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open");
    let cloned = Arc::clone(&dataset);

    assert!(Arc::ptr_eq(&dataset, &cloned));
    assert_eq!(
        cloned
            .read_packed(DatFileKind::Portal, FILE_ONE_ID)
            .expect("read should succeed")
            .expect("payload should exist")
            .as_slice(),
        Some(fixture.payload_one.as_slice())
    );
}

struct DatFixture {
    temp: TempDir,
    payload_one: Vec<u8>,
    payload_two_packed: Vec<u8>,
    payload_two_uncompressed_len: usize,
}

impl DatFixture {
    fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let payload_one = b"portal-bytes".to_vec();
        let payload_two_raw = build_multi_block_payload();
        let payload_two_uncompressed_len = 6000usize;
        let mut payload_two_packed = (payload_two_uncompressed_len as u32).to_le_bytes().to_vec();
        payload_two_packed.extend_from_slice(&payload_two_raw);
        let payload_three_iteration = build_iteration_payload();
        write_fixture_file(
            &temp.path().join(DatFileKind::Portal.default_file_name()),
            1,
            &payload_one,
            &payload_two_raw,
            &payload_three_iteration,
            payload_two_uncompressed_len as u32,
        )?;
        write_fixture_file(
            &temp.path().join(DatFileKind::Cell.default_file_name()),
            2,
            &payload_one,
            &payload_two_raw,
            &payload_three_iteration,
            payload_two_uncompressed_len as u32,
        )?;
        write_fixture_file(
            &temp.path().join(DatFileKind::Local.default_file_name()),
            3,
            &payload_one,
            &payload_two_raw,
            &payload_three_iteration,
            payload_two_uncompressed_len as u32,
        )?;
        Ok(Self {
            temp,
            payload_one,
            payload_two_packed,
            payload_two_uncompressed_len,
        })
    }

    fn path(&self) -> &Path {
        self.temp.path()
    }
}

fn build_multi_block_payload() -> Vec<u8> {
    (0..3000).map(|value| (value % 251) as u8).collect()
}

fn build_iteration_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&5i32.to_le_bytes());
    payload.extend_from_slice(&(-3i32).to_le_bytes());
    payload.extend_from_slice(&1i32.to_le_bytes());
    payload.extend_from_slice(&(-2i32).to_le_bytes());
    payload.extend_from_slice(&4i32.to_le_bytes());
    payload
}

fn write_fixture_file(
    path: &PathBuf,
    file_type: u32,
    payload_one: &[u8],
    payload_two: &[u8],
    payload_three: &[u8],
    payload_two_uncompressed_len: u32,
) -> std::io::Result<()> {
    let mut bytes = vec![0u8; BLOCK_SIZE * 8];
    let file_size = bytes.len() as i32;
    write_header(&mut bytes, file_type, file_size);
    let mut packed_two = payload_two_uncompressed_len.to_le_bytes().to_vec();
    packed_two.extend_from_slice(payload_two);

    let mut root = vec![0u8; NODE_SIZE];
    write_i32(&mut root, 0, LEFT_LEAF_BLOCK_OFFSET as i32);
    write_i32(&mut root, 4, RIGHT_LEAF_BLOCK_OFFSET as i32);
    write_i32(&mut root, 62 * 4, 1);
    write_file_entry(
        &mut root,
        62 * 4 + 4,
        FileEntryFixture {
            id: RETAIL_DDD_ITERATION_DID,
            block_offset: PAYLOAD_THREE_BLOCK_OFFSET as i32,
            size: payload_three.len() as u32,
            flags: 0,
            version: 0,
            date_stamp: 0,
        },
    );

    let mut left_leaf = vec![0u8; NODE_SIZE];
    write_i32(&mut left_leaf, 62 * 4, 2);
    write_file_entry(
        &mut left_leaf,
        62 * 4 + 4,
        FileEntryFixture {
            id: FILE_ONE_ID,
            block_offset: PAYLOAD_ONE_BLOCK_OFFSET as i32,
            size: payload_one.len() as u32,
            flags: 0,
            version: 1,
            date_stamp: 0x0102_0304,
        },
    );
    write_file_entry(
        &mut left_leaf,
        62 * 4 + 28,
        FileEntryFixture {
            id: FILE_TWO_ID,
            block_offset: PAYLOAD_TWO_FIRST_BLOCK_OFFSET as i32,
            size: packed_two.len() as u32,
            flags: 1,
            version: 7,
            date_stamp: 0x0506_0708,
        },
    );
    let mut right_leaf = vec![0u8; NODE_SIZE];
    write_i32(&mut right_leaf, 62 * 4, 0);

    write_block(&mut bytes, ROOT_BLOCK_OFFSET, 0, &root);
    write_block(&mut bytes, LEFT_LEAF_BLOCK_OFFSET, 0, &left_leaf);
    write_block(&mut bytes, RIGHT_LEAF_BLOCK_OFFSET, 0, &right_leaf);
    write_block(&mut bytes, PAYLOAD_ONE_BLOCK_OFFSET, 0, payload_one);
    write_block(
        &mut bytes,
        PAYLOAD_TWO_FIRST_BLOCK_OFFSET,
        PAYLOAD_TWO_SECOND_BLOCK_OFFSET as i32,
        &packed_two[..BLOCK_SIZE - 4],
    );
    write_block(
        &mut bytes,
        PAYLOAD_TWO_SECOND_BLOCK_OFFSET,
        0,
        &packed_two[BLOCK_SIZE - 4..],
    );
    write_block(&mut bytes, PAYLOAD_THREE_BLOCK_OFFSET, 0, payload_three);

    std::fs::write(path, bytes)
}

fn write_header(bytes: &mut [u8], file_type: u32, file_size: i32) {
    bytes[256..320].fill(0);
    bytes[256..260].copy_from_slice(&[0x00, 0x50, 0x4C, 0x00]);
    write_i32(bytes, 320, 0x0000_5442);
    write_i32(bytes, 324, BLOCK_SIZE as i32);
    write_i32(bytes, 328, file_size);
    write_u32(bytes, 332, file_type);
    write_u32(bytes, 336, 0xCAFE_BABE);
    write_i32(bytes, 340, 0);
    write_i32(bytes, 344, 0);
    write_i32(bytes, 348, 0);
    write_i32(bytes, 352, ROOT_BLOCK_OFFSET as i32);
    write_i32(bytes, 356, 0);
    write_i32(bytes, 360, 0);
    write_i32(bytes, 364, 0);
    write_u32(bytes, 368, 0x1234_5678);
    write_i32(bytes, 372, 77);
    write_i32(bytes, 376, 88);
    bytes[380..396].fill(0xAB);
    write_u32(bytes, 396, 0xDEAD_BEEF);
}

struct FileEntryFixture {
    id: u32,
    block_offset: i32,
    size: u32,
    flags: u16,
    version: u16,
    date_stamp: u32,
}

fn write_file_entry(bytes: &mut [u8], offset: usize, entry: FileEntryFixture) {
    write_u16(bytes, offset, entry.flags);
    write_u16(bytes, offset + 2, entry.version);
    write_u32(bytes, offset + 4, entry.id);
    write_i32(bytes, offset + 8, entry.block_offset);
    write_u32(bytes, offset + 12, entry.size);
    write_u32(bytes, offset + 16, entry.date_stamp);
    write_i32(bytes, offset + 20, 1);
}

fn write_block(bytes: &mut [u8], block_offset: usize, next_block: i32, payload: &[u8]) {
    write_i32(bytes, block_offset, next_block);
    let payload_start = block_offset + 4;
    let payload_end = payload_start + payload.len();
    bytes[payload_start..payload_end].copy_from_slice(payload);
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
