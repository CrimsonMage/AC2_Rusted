use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::entry::{
    DatEntryIter, DatEntryMeta, DatFileKind, DatIdIter, DatRawFileEntry, DatReadRef, DatSegmentMeta,
};
use crate::error::{DatError, DatResult};
use crate::memory_stats::DatIndexMemoryStats;
use crate::read_telemetry::DatReadTelemetry;

pub(crate) type DatIndexRecord = DatRawFileEntry;

const PREFIX_BITS: u32 = 12;
const PREFIX_COUNT: usize = 1 << PREFIX_BITS;
const SMALL_BUCKET_LINEAR_LIMIT: usize = 4;
const CHAIN_CACHE_SHARDS: usize = 16;

/// The non-DID fields of a retail B-tree leaf entry.
///
/// Keeping this exactly 20 bytes lets the split index preserve the previous 24-byte-per-entry
/// allocation while searching only the compact DID slice.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct DatIndexPayload {
    flags: u16,
    version: u16,
    offset: i32,
    size: u32,
    raw_date: u32,
    iteration: i32,
}

impl DatIndexPayload {
    pub(crate) fn meta(self, did: u32, file_kind: DatFileKind) -> DatEntryMeta {
        DatEntryMeta {
            did,
            file_kind,
            file_offset: self.offset.max(0) as u64,
            packed_len: self.size,
            flags: self.flags,
            version: self.version,
            date_stamp: self.raw_date,
            iteration: self.iteration,
        }
    }
}

#[derive(Debug)]
struct SplitIndex {
    ids: Box<[u32]>,
    payloads: Box<[DatIndexPayload]>,
    prefix_offsets: Box<[u32]>,
}

#[derive(Debug)]
enum IndexStorage {
    /// High-12 DID directory used by the production-sized DATs.
    Split(SplitIndex),
    /// A tiny collection is cheaper and simpler as one sorted boxed slice.
    TinySorted(Box<[DatIndexRecord]>),
}

#[derive(Debug)]
pub(crate) struct DatFlatIndex {
    file_kind: DatFileKind,
    storage: IndexStorage,
    segment_cache: SegmentCache,
    telemetry: Option<Arc<DatReadTelemetry>>,
    direct_single_block_hits: AtomicU64,
    multi_block_reads: AtomicU64,
}

#[derive(Debug)]
struct SegmentCache {
    shards: Box<[Mutex<ChainCacheShard>]>,
}

#[derive(Debug)]
struct ChainCacheShard {
    capacity_bytes: usize,
    retained_bytes: usize,
    entries: HashMap<u32, Arc<[DatSegmentMeta]>>,
    insertion_order: VecDeque<u32>,
    hits: u64,
    misses: u64,
    evictions: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatReadPathStats {
    pub direct_single_block_hits: u64,
    pub multi_block_reads: u64,
    pub chain_cache_hits: u64,
    pub chain_cache_misses: u64,
    pub chain_cache_evictions: u64,
    pub chain_cache_retained_bytes: usize,
    pub per_shard_entry_count: [u32; CHAIN_CACHE_SHARDS],
    pub per_shard_evictions: [u64; CHAIN_CACHE_SHARDS],
}

impl DatFlatIndex {
    #[cfg(test)]
    pub(crate) fn build(file_kind: DatFileKind, records: Vec<DatIndexRecord>) -> Self {
        Self::build_with_telemetry(file_kind, records, None)
    }

    pub(crate) fn build_with_telemetry(
        file_kind: DatFileKind,
        mut records: Vec<DatIndexRecord>,
        telemetry: Option<Arc<DatReadTelemetry>>,
    ) -> Self {
        records.sort_unstable_by_key(|record| record.id);
        debug_assert!(records.windows(2).all(|pair| pair[0].id < pair[1].id));

        Self {
            file_kind,
            storage: IndexStorage::from_sorted(file_kind, records),
            segment_cache: SegmentCache::new(chain_cache_budget_bytes(file_kind)),
            telemetry,
            direct_single_block_hits: AtomicU64::new(0),
            multi_block_reads: AtomicU64::new(0),
        }
    }

    pub(crate) fn contains(&self, did: u32) -> bool {
        self.find_ordinal(did).is_some()
    }

    pub(crate) fn entry(&self, did: u32) -> Option<DatEntryMeta> {
        self.find_ordinal(did).map(|ordinal| self.meta_at(ordinal))
    }

    pub(crate) fn iter_ids(&self) -> DatIdIter<'_> {
        match &self.storage {
            IndexStorage::Split(index) => DatIdIter::from_ids(&index.ids),
            IndexStorage::TinySorted(records) => DatIdIter::new(records),
        }
    }

    pub(crate) fn iter_entries(&self) -> DatEntryIter<'_> {
        match &self.storage {
            IndexStorage::Split(index) => {
                DatEntryIter::from_split(&index.ids, &index.payloads, self.file_kind)
            }
            IndexStorage::TinySorted(records) => DatEntryIter::new(records, self.file_kind),
        }
    }

    pub(crate) fn ids_in_range(&self, first: u32, last: u32) -> DatIdIter<'_> {
        if first > last {
            return DatIdIter::empty();
        }
        match &self.storage {
            IndexStorage::Split(index) => {
                let (start, end) = range_bounds(&index.ids, first, last);
                DatIdIter::from_ids(&index.ids[start..end])
            }
            IndexStorage::TinySorted(records) => {
                let (start, end) = record_range_bounds(records, first, last);
                DatIdIter::new(&records[start..end])
            }
        }
    }

    pub(crate) fn entries_in_range(&self, first: u32, last: u32) -> DatEntryIter<'_> {
        if first > last {
            return DatEntryIter::empty();
        }
        match &self.storage {
            IndexStorage::Split(index) => {
                let (start, end) = range_bounds(&index.ids, first, last);
                DatEntryIter::from_split(
                    &index.ids[start..end],
                    &index.payloads[start..end],
                    self.file_kind,
                )
            }
            IndexStorage::TinySorted(records) => {
                let (start, end) = record_range_bounds(records, first, last);
                DatEntryIter::new(&records[start..end], self.file_kind)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn read_ref<'a>(
        &'a self,
        mmap: &'a [u8],
        block_data_size: usize,
        did: u32,
        resolve_segments: impl FnOnce(DatEntryMeta) -> DatResult<Arc<[DatSegmentMeta]>>,
    ) -> DatResult<Option<DatReadRef<'a>>> {
        let Some(ordinal) = self.find_ordinal(did) else {
            return Ok(None);
        };
        let meta = self.meta_at(ordinal);
        if let Some(bytes) = direct_single_block_slice(mmap, block_data_size, meta) {
            self.direct_single_block_hits
                .fetch_add(1, Ordering::Relaxed);
            return Ok(Some(DatReadRef::contiguous(bytes)));
        }
        self.multi_block_reads.fetch_add(1, Ordering::Relaxed);
        let segments = self.cached_segments(meta, resolve_segments)?;

        Ok(Some(DatReadRef::segmented(
            mmap,
            Arc::clone(&segments),
            meta.packed_len as usize,
        )))
    }

    /// Resolves packed storage from metadata already found by this index without another DID
    /// search. This preserves the production segment-metadata cache for the combined lookup/read
    /// API.
    pub(crate) fn read_ref_from_meta<'a>(
        &'a self,
        mmap: &'a [u8],
        block_data_size: usize,
        meta: DatEntryMeta,
        resolve_segments: impl FnOnce(DatEntryMeta) -> DatResult<Arc<[DatSegmentMeta]>>,
    ) -> DatResult<DatReadRef<'a>> {
        if let Some(bytes) = direct_single_block_slice(mmap, block_data_size, meta) {
            self.direct_single_block_hits
                .fetch_add(1, Ordering::Relaxed);
            return Ok(DatReadRef::contiguous(bytes));
        }
        self.multi_block_reads.fetch_add(1, Ordering::Relaxed);
        let segments = self.cached_segments(meta, resolve_segments)?;
        Ok(DatReadRef::segmented(
            mmap,
            segments,
            meta.packed_len as usize,
        ))
    }

    pub(crate) fn copy_range(
        &self,
        mmap: &[u8],
        block_data_size: usize,
        did: u32,
        offset: u64,
        len: usize,
        resolve_segments: impl FnOnce(DatEntryMeta) -> DatResult<Arc<[DatSegmentMeta]>>,
    ) -> DatResult<Option<Vec<u8>>> {
        let Some(ordinal) = self.find_ordinal(did) else {
            return Ok(None);
        };
        let meta = self.meta_at(ordinal);
        let packed_len = meta.packed_len as usize;
        let start = usize::try_from(offset).map_err(|_| DatError::RangeOutOfBounds {
            did,
            offset,
            len,
            packed_len: meta.packed_len,
        })?;
        let end = start.checked_add(len).ok_or(DatError::RangeOutOfBounds {
            did,
            offset,
            len,
            packed_len: meta.packed_len,
        })?;
        if end > packed_len {
            return Err(DatError::RangeOutOfBounds {
                did,
                offset,
                len,
                packed_len: meta.packed_len,
            });
        }
        if len == 0 {
            return Ok(Some(Vec::new()));
        }

        if let Some(bytes) = direct_single_block_slice(mmap, block_data_size, meta) {
            self.direct_single_block_hits
                .fetch_add(1, Ordering::Relaxed);
            self.record_contiguous_read(len);
            return Ok(Some(bytes[start..end].to_vec()));
        }

        self.multi_block_reads.fetch_add(1, Ordering::Relaxed);
        let segments = self.cached_segments(meta, resolve_segments)?;
        self.record_segmented_read(len);
        let mut out = Vec::with_capacity(len);
        let mut logical_offset = 0usize;
        for segment in segments.iter().copied() {
            let segment_len = segment.len();
            let segment_end = logical_offset + segment_len;
            if start >= segment_end {
                logical_offset = segment_end;
                continue;
            }

            let take_start = start.saturating_sub(logical_offset);
            let available = segment_len - take_start;
            let remaining = len - out.len();
            let take_len = available.min(remaining);
            let bytes = segment.as_slice(mmap);
            out.extend_from_slice(&bytes[take_start..take_start + take_len]);
            if out.len() == len {
                break;
            }
            logical_offset = segment_end;
        }

        Ok(Some(out))
    }

    fn find_ordinal(&self, did: u32) -> Option<usize> {
        match &self.storage {
            IndexStorage::Split(index) => index.find_ordinal(did),
            IndexStorage::TinySorted(records) => {
                records.binary_search_by_key(&did, |record| record.id).ok()
            }
        }
    }

    fn meta_at(&self, ordinal: usize) -> DatEntryMeta {
        match &self.storage {
            IndexStorage::Split(index) => {
                index.payloads[ordinal].meta(index.ids[ordinal], self.file_kind)
            }
            IndexStorage::TinySorted(records) => records[ordinal].meta(self.file_kind),
        }
    }

    fn cached_segments(
        &self,
        meta: DatEntryMeta,
        resolve_segments: impl FnOnce(DatEntryMeta) -> DatResult<Arc<[DatSegmentMeta]>>,
    ) -> DatResult<Arc<[DatSegmentMeta]>> {
        self.segment_cache
            .resolve(meta, self.telemetry.as_deref(), resolve_segments)
    }

    fn record_contiguous_read(&self, bytes: usize) {
        if let Some(telemetry) = self.telemetry.as_deref() {
            telemetry.record_contiguous_read(bytes);
        }
    }

    fn record_segmented_read(&self, bytes: usize) {
        if let Some(telemetry) = self.telemetry.as_deref() {
            telemetry.record_segmented_read(bytes);
        }
    }

    pub(crate) fn read_path_stats(&self) -> DatReadPathStats {
        self.segment_cache.stats(
            self.direct_single_block_hits.load(Ordering::Relaxed),
            self.multi_block_reads.load(Ordering::Relaxed),
        )
    }

    pub(crate) fn memory_stats(&self) -> DatIndexMemoryStats {
        let mut stats = match &self.storage {
            IndexStorage::Split(index) => DatIndexMemoryStats {
                record_count: index.ids.len(),
                did_bytes: std::mem::size_of_val(index.ids.as_ref()),
                payload_bytes: std::mem::size_of_val(index.payloads.as_ref()),
                prefix_directory_bytes: std::mem::size_of_val(index.prefix_offsets.as_ref()),
                ..DatIndexMemoryStats::default()
            },
            IndexStorage::TinySorted(records) => DatIndexMemoryStats {
                record_count: records.len(),
                flat_record_bytes: std::mem::size_of_val(records.as_ref()),
                ..DatIndexMemoryStats::default()
            },
        };
        let segment_stats = self.segment_cache.memory_stats();
        stats.segment_metadata_retained_bytes = segment_stats.retained_bytes;
        stats.segment_metadata_capacity_bytes = segment_stats.capacity_bytes;
        stats
    }
}

impl IndexStorage {
    fn from_sorted(file_kind: DatFileKind, records: Vec<DatIndexRecord>) -> Self {
        match file_kind {
            DatFileKind::Local => Self::TinySorted(records.into_boxed_slice()),
            DatFileKind::Portal | DatFileKind::Cell | DatFileKind::HighRes => {
                Self::Split(SplitIndex::from_sorted(records))
            }
        }
    }
}

impl SplitIndex {
    fn from_sorted(records: Vec<DatIndexRecord>) -> Self {
        let mut ids = Vec::with_capacity(records.len());
        let mut payloads = Vec::with_capacity(records.len());
        for record in records {
            ids.push(record.id);
            payloads.push(DatIndexPayload {
                flags: record.flags,
                version: record.version,
                offset: record.offset,
                size: record.size,
                raw_date: record.raw_date,
                iteration: record.iteration,
            });
        }
        Self {
            prefix_offsets: prefix_offsets(&ids),
            ids: ids.into_boxed_slice(),
            payloads: payloads.into_boxed_slice(),
        }
    }

    fn find_ordinal(&self, did: u32) -> Option<usize> {
        let prefix = (did >> (32 - PREFIX_BITS)) as usize;
        let start = self.prefix_offsets[prefix] as usize;
        let end = self.prefix_offsets[prefix + 1] as usize;
        let bucket = &self.ids[start..end];
        let relative = match bucket.len() {
            0 => return None,
            1 => (bucket[0] == did).then_some(0),
            2..=SMALL_BUCKET_LINEAR_LIMIT => bucket.iter().position(|candidate| *candidate == did),
            _ => bucket.binary_search(&did).ok(),
        }?;
        Some(start + relative)
    }
}

impl SegmentCache {
    fn new(capacity_bytes: usize) -> Self {
        let base = capacity_bytes / CHAIN_CACHE_SHARDS;
        let remainder = capacity_bytes % CHAIN_CACHE_SHARDS;
        let shards = (0..CHAIN_CACHE_SHARDS)
            .map(|index| {
                Mutex::new(ChainCacheShard {
                    capacity_bytes: base + usize::from(index < remainder),
                    retained_bytes: 0,
                    entries: HashMap::new(),
                    insertion_order: VecDeque::new(),
                    hits: 0,
                    misses: 0,
                    evictions: 0,
                })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { shards }
    }

    fn resolve(
        &self,
        meta: DatEntryMeta,
        telemetry: Option<&DatReadTelemetry>,
        resolve_segments: impl FnOnce(DatEntryMeta) -> DatResult<Arc<[DatSegmentMeta]>>,
    ) -> DatResult<Arc<[DatSegmentMeta]>> {
        let shard = &self.shards[meta.did as usize % self.shards.len()];
        let mut shard = shard
            .lock()
            .expect("DAT chain-cache shard mutex should not be poisoned");
        if let Some(segments) = shard.entries.get(&meta.did).cloned() {
            shard.hits += 1;
            if let Some(telemetry) = telemetry {
                telemetry.record_chain_cache_hit();
            }
            return Ok(segments);
        }
        shard.misses += 1;
        if let Some(telemetry) = telemetry {
            telemetry.record_chain_cache_miss();
        }
        let segments = resolve_segments(meta)?;
        let bytes = segment_metadata_bytes(&segments);
        if bytes > shard.capacity_bytes {
            return Ok(segments);
        }
        while shard.retained_bytes + bytes > shard.capacity_bytes {
            let Some(oldest) = shard.insertion_order.pop_front() else {
                break;
            };
            if let Some(evicted) = shard.entries.remove(&oldest) {
                shard.retained_bytes -= segment_metadata_bytes(&evicted);
                shard.evictions += 1;
                if let Some(telemetry) = telemetry {
                    telemetry.record_chain_cache_eviction();
                }
            }
        }
        shard.retained_bytes += bytes;
        shard.insertion_order.push_back(meta.did);
        shard.entries.insert(meta.did, Arc::clone(&segments));
        Ok(segments)
    }

    fn stats(&self, direct_single_block_hits: u64, multi_block_reads: u64) -> DatReadPathStats {
        let mut stats = DatReadPathStats {
            direct_single_block_hits,
            multi_block_reads,
            chain_cache_hits: 0,
            chain_cache_misses: 0,
            chain_cache_evictions: 0,
            chain_cache_retained_bytes: 0,
            per_shard_entry_count: [0; CHAIN_CACHE_SHARDS],
            per_shard_evictions: [0; CHAIN_CACHE_SHARDS],
        };
        for (index, shard) in self.shards.iter().enumerate() {
            let shard = shard
                .lock()
                .expect("DAT chain-cache shard mutex should not be poisoned");
            stats.chain_cache_hits += shard.hits;
            stats.chain_cache_misses += shard.misses;
            stats.chain_cache_evictions += shard.evictions;
            stats.chain_cache_retained_bytes += shard.retained_bytes;
            stats.per_shard_entry_count[index] = shard.entries.len() as u32;
            stats.per_shard_evictions[index] = shard.evictions;
        }
        stats
    }

    fn memory_stats(&self) -> SegmentCacheMemoryStats {
        let mut stats = SegmentCacheMemoryStats::default();
        for shard in self.shards.iter() {
            let shard = shard
                .lock()
                .expect("DAT chain-cache shard mutex should not be poisoned");
            stats.retained_bytes += shard.retained_bytes;
            stats.capacity_bytes += shard.capacity_bytes;
        }
        stats
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SegmentCacheMemoryStats {
    retained_bytes: usize,
    capacity_bytes: usize,
}

fn chain_cache_budget_bytes(file_kind: DatFileKind) -> usize {
    match file_kind {
        DatFileKind::Portal | DatFileKind::Cell => 256 * 1024,
        DatFileKind::Local => 64 * 1024,
        DatFileKind::HighRes => 1024 * 1024,
    }
}

fn segment_metadata_bytes(segments: &[DatSegmentMeta]) -> usize {
    std::mem::size_of_val(segments)
}

fn direct_single_block_slice(
    mmap: &[u8],
    block_data_size: usize,
    meta: DatEntryMeta,
) -> Option<&[u8]> {
    if meta.file_offset == 0 || meta.packed_len == 0 || meta.packed_len as usize > block_data_size {
        return None;
    }
    let start = usize::try_from(meta.file_offset).ok()?.checked_add(4)?;
    let end = start.checked_add(meta.packed_len as usize)?;
    // Malformed candidates fall through to the legacy resolver so its established error
    // reporting remains authoritative for corrupt offsets and mmap-end violations.
    (end <= mmap.len()).then(|| &mmap[start..end])
}

fn prefix_offsets(ids: &[u32]) -> Box<[u32]> {
    let mut offsets = vec![0u32; PREFIX_COUNT + 1];
    for did in ids {
        offsets[(did >> (32 - PREFIX_BITS)) as usize + 1] += 1;
    }
    for index in 1..offsets.len() {
        offsets[index] += offsets[index - 1];
    }
    offsets.into_boxed_slice()
}

fn range_bounds(ids: &[u32], first: u32, last: u32) -> (usize, usize) {
    let start = ids.partition_point(|did| *did < first);
    let end = ids.partition_point(|did| *did <= last);
    (start, end)
}

fn record_range_bounds(records: &[DatIndexRecord], first: u32, last: u32) -> (usize, usize) {
    let start = records.partition_point(|record| record.id < first);
    let end = records.partition_point(|record| record.id <= last);
    (start, end)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::{chain_cache_budget_bytes, DatFlatIndex, DatIndexPayload, DatIndexRecord};
    use crate::entry::{DatFileKind, DatSegmentMeta};

    const TEST_DID: u32 = 0x1000_0001;

    #[test]
    fn defers_segment_resolution_until_first_read() {
        let index = DatFlatIndex::build(DatFileKind::Portal, vec![test_index_record()]);
        let calls = AtomicUsize::new(0);

        assert!(index.contains(TEST_DID));
        assert_eq!(calls.load(Ordering::Relaxed), 0);

        let first = index
            .read_ref(b"abcdefghijklmnop", 3, TEST_DID, |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(Arc::<[DatSegmentMeta]>::from([
                    DatSegmentMeta { start: 1, end: 3 },
                    DatSegmentMeta { start: 6, end: 8 },
                ]))
            })
            .expect("segment resolution should succeed")
            .expect("record should exist");
        assert_eq!(first.copy_to_vec(), b"bcgh");
        assert_eq!(calls.load(Ordering::Relaxed), 1);

        let second = index
            .read_ref(b"abcdefghijklmnop", 3, TEST_DID, |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(Arc::<[DatSegmentMeta]>::from([]))
            })
            .expect("cached segment reuse should succeed")
            .expect("record should exist");
        assert_eq!(second.copy_to_vec(), b"bcgh");
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        let stats = index.read_path_stats();
        assert_eq!(stats.direct_single_block_hits, 0);
        assert_eq!(stats.multi_block_reads, 2);
        assert_eq!(stats.chain_cache_hits, 1);
        assert_eq!(stats.chain_cache_misses, 1);
        assert_eq!(
            stats.chain_cache_retained_bytes,
            2 * std::mem::size_of::<DatSegmentMeta>()
        );
    }

    #[test]
    fn range_reads_reuse_cached_segments() {
        let index = DatFlatIndex::build(DatFileKind::Portal, vec![test_index_record()]);
        let calls = AtomicUsize::new(0);

        let first = index
            .copy_range(b"abcdefghijklmnop", 3, TEST_DID, 1, 2, |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(Arc::<[DatSegmentMeta]>::from([
                    DatSegmentMeta { start: 1, end: 3 },
                    DatSegmentMeta { start: 6, end: 8 },
                ]))
            })
            .expect("range read should succeed")
            .expect("range should exist");
        assert_eq!(first, b"cg");
        assert_eq!(calls.load(Ordering::Relaxed), 1);

        let second = index
            .copy_range(b"abcdefghijklmnop", 3, TEST_DID, 0, 4, |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(Arc::<[DatSegmentMeta]>::from([]))
            })
            .expect("cached range reuse should succeed")
            .expect("range should exist");
        assert_eq!(second, b"bcgh");
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn direct_single_block_reads_bypass_chain_resolution() {
        let index = DatFlatIndex::build(
            DatFileKind::Portal,
            vec![DatIndexRecord {
                offset: 4,
                ..test_index_record()
            }],
        );
        let read = index
            .read_ref(b"abcdefgh0123", 4, TEST_DID, |_| {
                panic!("single-block read must not resolve a chain")
            })
            .expect("direct read should succeed")
            .expect("record should exist");
        assert!(read.is_contiguous());
        assert_eq!(read.as_slice(), Some(&b"0123"[..]));
        let stats = index.read_path_stats();
        assert_eq!(stats.direct_single_block_hits, 1);
        assert_eq!(stats.multi_block_reads, 0);
    }

    #[test]
    fn split_and_tiny_indexes_preserve_lookup_and_order() {
        let records = vec![
            record(0x1000_0001),
            record(0x1000_0003),
            record(0xA000_0001),
        ];
        for kind in [DatFileKind::Portal, DatFileKind::Local] {
            let index = DatFlatIndex::build(kind, records.clone());
            assert!(index.contains(0x1000_0001));
            assert!(!index.contains(0x1000_0002));
            assert_eq!(
                index.entry(0xA000_0001),
                Some(record(0xA000_0001).meta(kind))
            );
            assert_eq!(
                index.iter_ids().collect::<Vec<_>>(),
                vec![0x1000_0001, 0x1000_0003, 0xA000_0001]
            );
            assert_eq!(
                index
                    .ids_in_range(0x1000_0002, 0xA000_0001)
                    .collect::<Vec<_>>(),
                vec![0x1000_0003, 0xA000_0001]
            );
        }
    }

    #[test]
    fn split_index_keeps_the_24_byte_record_equivalent() {
        assert_eq!(std::mem::size_of::<DatIndexRecord>(), 24);
        assert_eq!(std::mem::size_of::<DatIndexPayload>(), 20);
    }

    #[test]
    fn chain_cache_budgets_match_the_measured_per_dat_configuration() {
        assert_eq!(chain_cache_budget_bytes(DatFileKind::Portal), 256 * 1024);
        assert_eq!(chain_cache_budget_bytes(DatFileKind::Cell), 256 * 1024);
        assert_eq!(chain_cache_budget_bytes(DatFileKind::Local), 64 * 1024);
        assert_eq!(chain_cache_budget_bytes(DatFileKind::HighRes), 1024 * 1024);
    }

    fn test_index_record() -> DatIndexRecord {
        record(TEST_DID)
    }

    fn record(id: u32) -> DatIndexRecord {
        DatIndexRecord {
            flags: 0,
            version: 1,
            id,
            offset: 2048,
            size: 4,
            raw_date: 0,
            iteration: 0,
        }
    }
}
