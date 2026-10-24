use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

/// Read-only, process-local observations of the file-backed DAT B-tree path.
///
/// These counters mirror retail's lookup, resident-node, and synchronous-read boundaries. They
/// do not time requests, retain request history, or participate in cache policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DatBtreeProfileSnapshot {
    pub did_lookup_requests: u64,
    pub did_lookup_found: u64,
    pub did_lookup_missing: u64,
    pub did_lookup_failures: u64,
    pub metadata_only_requests: u64,
    pub range_read_requests: u64,
    pub range_requested_bytes: u64,
    pub stored_compressed_reads: u64,
    pub btree_nodes_visited: u64,
    pub btree_root_visits: u64,
    pub btree_key_comparisons: u64,
    pub parsed_node_cache_hits: u64,
    pub parsed_node_cache_misses: u64,
    pub parsed_node_cache_evictions: u64,
    pub parsed_node_cache_entries: usize,
    /// Excludes the separately pinned root node, matching retail's 100 replaceable nodes.
    pub parsed_node_cache_capacity: usize,
    pub btree_node_loads: u64,
    pub btree_node_positioned_read_calls: u64,
    pub btree_node_positioned_read_bytes: u64,
    pub payload_read_calls: u64,
    pub payload_blocks_total: u64,
    pub payload_positioned_read_calls: u64,
    pub payload_positioned_read_bytes: u64,
}

#[derive(Debug, Default)]
pub(crate) struct DatBtreeProfile {
    did_lookup_requests: AtomicU64,
    did_lookup_found: AtomicU64,
    did_lookup_missing: AtomicU64,
    did_lookup_failures: AtomicU64,
    metadata_only_requests: AtomicU64,
    range_read_requests: AtomicU64,
    range_requested_bytes: AtomicU64,
    stored_compressed_reads: AtomicU64,
    btree_nodes_visited: AtomicU64,
    btree_root_visits: AtomicU64,
    btree_key_comparisons: AtomicU64,
    parsed_node_cache_hits: AtomicU64,
    parsed_node_cache_misses: AtomicU64,
    parsed_node_cache_evictions: AtomicU64,
    btree_node_loads: AtomicU64,
    btree_node_positioned_read_calls: AtomicU64,
    btree_node_positioned_read_bytes: AtomicU64,
    payload_read_calls: AtomicU64,
    payload_blocks_total: AtomicU64,
    payload_positioned_read_calls: AtomicU64,
    payload_positioned_read_bytes: AtomicU64,
}

impl DatBtreeProfile {
    pub(crate) fn record_lookup(&self) {
        self.did_lookup_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_lookup_found(&self) {
        self.did_lookup_found.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_lookup_missing(&self) {
        self.did_lookup_missing.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_lookup_failure(&self) {
        self.did_lookup_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_metadata_only_request(&self) {
        self.metadata_only_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_range_request(&self, bytes: usize) {
        self.range_read_requests.fetch_add(1, Ordering::Relaxed);
        self.range_requested_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_node_visit(&self, is_root: bool) {
        self.btree_nodes_visited.fetch_add(1, Ordering::Relaxed);
        if is_root {
            self.btree_root_visits.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(crate) fn record_key_comparison(&self) {
        self.btree_key_comparisons.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_cache_hit(&self) {
        self.parsed_node_cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_cache_miss(&self) {
        self.parsed_node_cache_misses
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_cache_eviction(&self) {
        self.parsed_node_cache_evictions
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_node_load(&self) {
        self.btree_node_loads.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_node_positioned_read(&self, bytes: usize) {
        self.btree_node_positioned_read_calls
            .fetch_add(1, Ordering::Relaxed);
        self.btree_node_positioned_read_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_payload_read(&self, blocks: usize, stored_compressed: bool) {
        self.payload_read_calls.fetch_add(1, Ordering::Relaxed);
        self.payload_blocks_total
            .fetch_add(blocks as u64, Ordering::Relaxed);
        if stored_compressed {
            self.stored_compressed_reads.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(crate) fn record_payload_positioned_read(&self, bytes: usize) {
        self.payload_positioned_read_calls
            .fetch_add(1, Ordering::Relaxed);
        self.payload_positioned_read_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(
        &self,
        parsed_node_cache_entries: usize,
        parsed_node_cache_capacity: usize,
    ) -> DatBtreeProfileSnapshot {
        DatBtreeProfileSnapshot {
            did_lookup_requests: self.did_lookup_requests.load(Ordering::Relaxed),
            did_lookup_found: self.did_lookup_found.load(Ordering::Relaxed),
            did_lookup_missing: self.did_lookup_missing.load(Ordering::Relaxed),
            did_lookup_failures: self.did_lookup_failures.load(Ordering::Relaxed),
            metadata_only_requests: self.metadata_only_requests.load(Ordering::Relaxed),
            range_read_requests: self.range_read_requests.load(Ordering::Relaxed),
            range_requested_bytes: self.range_requested_bytes.load(Ordering::Relaxed),
            stored_compressed_reads: self.stored_compressed_reads.load(Ordering::Relaxed),
            btree_nodes_visited: self.btree_nodes_visited.load(Ordering::Relaxed),
            btree_root_visits: self.btree_root_visits.load(Ordering::Relaxed),
            btree_key_comparisons: self.btree_key_comparisons.load(Ordering::Relaxed),
            parsed_node_cache_hits: self.parsed_node_cache_hits.load(Ordering::Relaxed),
            parsed_node_cache_misses: self.parsed_node_cache_misses.load(Ordering::Relaxed),
            parsed_node_cache_evictions: self.parsed_node_cache_evictions.load(Ordering::Relaxed),
            parsed_node_cache_entries,
            parsed_node_cache_capacity,
            btree_node_loads: self.btree_node_loads.load(Ordering::Relaxed),
            btree_node_positioned_read_calls: self
                .btree_node_positioned_read_calls
                .load(Ordering::Relaxed),
            btree_node_positioned_read_bytes: self
                .btree_node_positioned_read_bytes
                .load(Ordering::Relaxed),
            payload_read_calls: self.payload_read_calls.load(Ordering::Relaxed),
            payload_blocks_total: self.payload_blocks_total.load(Ordering::Relaxed),
            payload_positioned_read_calls: self
                .payload_positioned_read_calls
                .load(Ordering::Relaxed),
            payload_positioned_read_bytes: self
                .payload_positioned_read_bytes
                .load(Ordering::Relaxed),
        }
    }
}
