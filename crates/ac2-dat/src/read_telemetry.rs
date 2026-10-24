use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde::Serialize;

use crate::entry::DatFileKind;

/// Diagnostic-only caller category for a storage request.
///
/// DAT deliberately owns this dependency-free vocabulary rather than importing resource, world,
/// or viewer types. It neither selects a cache nor changes any lookup/read behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum DatReadAttribution {
    ActiveWorldScoped,
    GenericResource,
    EphemeralWorldTooling,
    #[default]
    Unattributed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DatReadAttributionSummary {
    pub active_world_scoped_requests: u64,
    pub generic_resource_requests: u64,
    pub ephemeral_world_tooling_requests: u64,
    pub unattributed_requests: u64,
    pub active_world_scoped_returned_bytes: u64,
    pub generic_resource_returned_bytes: u64,
    pub ephemeral_world_tooling_returned_bytes: u64,
    pub unattributed_returned_bytes: u64,
}

/// Aggregate, process-local DAT read observations for one mounted DAT file.
///
/// The counters intentionally use relaxed atomics: they are diagnostic data, not synchronization
/// or correctness state. Path counts describe resolver stages, so a startup-index miss followed by
/// a B-tree miss and full-index retry can increment more than one path counter for one request.
#[derive(Debug)]
pub(crate) struct DatReadTelemetry {
    started_at: Instant,
    snapshot_sequence: AtomicU64,
    total_read_requests: AtomicU64,
    successful_reads: AtomicU64,
    missing_did_reads: AtomicU64,
    direct_single_block_reads: AtomicU64,
    segmented_multi_block_reads: AtomicU64,
    startup_index_reads: AtomicU64,
    direct_btree_fallback_reads: AtomicU64,
    full_index_reads: AtomicU64,
    chain_cache_hits: AtomicU64,
    chain_cache_misses: AtomicU64,
    chain_cache_evictions: AtomicU64,
    contiguous_read_bytes: AtomicU64,
    segmented_read_bytes: AtomicU64,
    contiguous_reads_copied_to_owned_storage: AtomicU64,
    active_world_scoped_requests: AtomicU64,
    generic_resource_requests: AtomicU64,
    ephemeral_world_tooling_requests: AtomicU64,
    unattributed_requests: AtomicU64,
    active_world_scoped_returned_bytes: AtomicU64,
    generic_resource_returned_bytes: AtomicU64,
    ephemeral_world_tooling_returned_bytes: AtomicU64,
    unattributed_returned_bytes: AtomicU64,
}

impl Default for DatReadTelemetry {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            snapshot_sequence: AtomicU64::new(0),
            total_read_requests: AtomicU64::new(0),
            successful_reads: AtomicU64::new(0),
            missing_did_reads: AtomicU64::new(0),
            direct_single_block_reads: AtomicU64::new(0),
            segmented_multi_block_reads: AtomicU64::new(0),
            startup_index_reads: AtomicU64::new(0),
            direct_btree_fallback_reads: AtomicU64::new(0),
            full_index_reads: AtomicU64::new(0),
            chain_cache_hits: AtomicU64::new(0),
            chain_cache_misses: AtomicU64::new(0),
            chain_cache_evictions: AtomicU64::new(0),
            contiguous_read_bytes: AtomicU64::new(0),
            segmented_read_bytes: AtomicU64::new(0),
            contiguous_reads_copied_to_owned_storage: AtomicU64::new(0),
            active_world_scoped_requests: AtomicU64::new(0),
            generic_resource_requests: AtomicU64::new(0),
            ephemeral_world_tooling_requests: AtomicU64::new(0),
            unattributed_requests: AtomicU64::new(0),
            active_world_scoped_returned_bytes: AtomicU64::new(0),
            generic_resource_returned_bytes: AtomicU64::new(0),
            ephemeral_world_tooling_returned_bytes: AtomicU64::new(0),
            unattributed_returned_bytes: AtomicU64::new(0),
        }
    }
}

/// A point-in-time, per-DAT read-path summary for viewer or session diagnostics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct DatReadTelemetrySummary {
    /// Monotonically increasing sequence for non-resetting telemetry snapshots.
    pub snapshot_sequence: u64,
    /// Monotonic elapsed time since this DAT mounted, in microseconds.
    pub elapsed_micros: u64,
    pub total_read_requests: u64,
    pub successful_reads: u64,
    pub missing_did_reads: u64,
    pub direct_single_block_reads: u64,
    pub segmented_multi_block_reads: u64,
    pub startup_index_reads: u64,
    pub direct_btree_fallback_reads: u64,
    pub full_index_reads: u64,
    pub chain_cache_hits: u64,
    pub chain_cache_misses: u64,
    pub chain_cache_evictions: u64,
    pub contiguous_read_bytes: u64,
    pub segmented_read_bytes: u64,
    pub contiguous_reads_copied_to_owned_storage: u64,
    pub attribution: DatReadAttributionSummary,
    pub successful_read_percent: f64,
    pub missing_did_percent: f64,
    pub direct_single_block_percent: f64,
    pub segmented_multi_block_percent: f64,
    pub startup_index_percent: f64,
    pub direct_btree_fallback_percent: f64,
    pub full_index_percent: f64,
    pub chain_cache_hit_percent: f64,
    pub contiguous_byte_percent: f64,
}

impl DatReadTelemetrySummary {
    /// Returns the non-resetting counter delta from an earlier snapshot of the same mounted DAT.
    pub fn delta_since(self, previous: Self) -> Self {
        Self {
            snapshot_sequence: self.snapshot_sequence,
            elapsed_micros: self.elapsed_micros.saturating_sub(previous.elapsed_micros),
            total_read_requests: self
                .total_read_requests
                .saturating_sub(previous.total_read_requests),
            successful_reads: self
                .successful_reads
                .saturating_sub(previous.successful_reads),
            missing_did_reads: self
                .missing_did_reads
                .saturating_sub(previous.missing_did_reads),
            direct_single_block_reads: self
                .direct_single_block_reads
                .saturating_sub(previous.direct_single_block_reads),
            segmented_multi_block_reads: self
                .segmented_multi_block_reads
                .saturating_sub(previous.segmented_multi_block_reads),
            startup_index_reads: self
                .startup_index_reads
                .saturating_sub(previous.startup_index_reads),
            direct_btree_fallback_reads: self
                .direct_btree_fallback_reads
                .saturating_sub(previous.direct_btree_fallback_reads),
            full_index_reads: self
                .full_index_reads
                .saturating_sub(previous.full_index_reads),
            chain_cache_hits: self
                .chain_cache_hits
                .saturating_sub(previous.chain_cache_hits),
            chain_cache_misses: self
                .chain_cache_misses
                .saturating_sub(previous.chain_cache_misses),
            chain_cache_evictions: self
                .chain_cache_evictions
                .saturating_sub(previous.chain_cache_evictions),
            contiguous_read_bytes: self
                .contiguous_read_bytes
                .saturating_sub(previous.contiguous_read_bytes),
            segmented_read_bytes: self
                .segmented_read_bytes
                .saturating_sub(previous.segmented_read_bytes),
            contiguous_reads_copied_to_owned_storage: self
                .contiguous_reads_copied_to_owned_storage
                .saturating_sub(previous.contiguous_reads_copied_to_owned_storage),
            attribution: self.attribution.delta_since(previous.attribution),
            successful_read_percent: 0.0,
            missing_did_percent: 0.0,
            direct_single_block_percent: 0.0,
            segmented_multi_block_percent: 0.0,
            startup_index_percent: 0.0,
            direct_btree_fallback_percent: 0.0,
            full_index_percent: 0.0,
            chain_cache_hit_percent: 0.0,
            contiguous_byte_percent: 0.0,
        }
        .with_percentages()
    }

    fn with_percentages(mut self) -> Self {
        let resolved_reads = self.direct_single_block_reads + self.segmented_multi_block_reads;
        let chain_cache_lookups = self.chain_cache_hits + self.chain_cache_misses;
        let returned_bytes = self.contiguous_read_bytes + self.segmented_read_bytes;
        self.successful_read_percent = percent(self.successful_reads, self.total_read_requests);
        self.missing_did_percent = percent(self.missing_did_reads, self.total_read_requests);
        self.direct_single_block_percent = percent(self.direct_single_block_reads, resolved_reads);
        self.segmented_multi_block_percent =
            percent(self.segmented_multi_block_reads, resolved_reads);
        self.startup_index_percent = percent(self.startup_index_reads, self.total_read_requests);
        self.direct_btree_fallback_percent =
            percent(self.direct_btree_fallback_reads, self.total_read_requests);
        self.full_index_percent = percent(self.full_index_reads, self.total_read_requests);
        self.chain_cache_hit_percent = percent(self.chain_cache_hits, chain_cache_lookups);
        self.contiguous_byte_percent = percent(self.contiguous_read_bytes, returned_bytes);
        self
    }

    /// Compact, allocation-at-snapshot-only text for the world-viewer capture surface.
    pub fn viewer_line(self, kind: DatFileKind) -> String {
        format!(
            "{kind:?}: requests={} success={} ({:.1}%) missing={} ({:.1}%) | direct={} ({:.1}%) segmented={} ({:.1}%) | paths startup={} ({:.1}%) btree={} ({:.1}%) full={} ({:.1}%) | chain hit/miss/evict={}/{}/{} ({:.1}% hit) | bytes contiguous/segmented={}/{} ({:.1}% contiguous) | contiguous owned copies={}",
            self.total_read_requests,
            self.successful_reads,
            self.successful_read_percent,
            self.missing_did_reads,
            self.missing_did_percent,
            self.direct_single_block_reads,
            self.direct_single_block_percent,
            self.segmented_multi_block_reads,
            self.segmented_multi_block_percent,
            self.startup_index_reads,
            self.startup_index_percent,
            self.direct_btree_fallback_reads,
            self.direct_btree_fallback_percent,
            self.full_index_reads,
            self.full_index_percent,
            self.chain_cache_hits,
            self.chain_cache_misses,
            self.chain_cache_evictions,
            self.chain_cache_hit_percent,
            self.contiguous_read_bytes,
            self.segmented_read_bytes,
            self.contiguous_byte_percent,
            self.contiguous_reads_copied_to_owned_storage,
        )
    }
}

impl DatReadTelemetry {
    pub(crate) fn record_request(&self) {
        self.record_request_attributed(DatReadAttribution::Unattributed);
    }

    pub(crate) fn record_request_attributed(&self, attribution: DatReadAttribution) {
        self.total_read_requests.fetch_add(1, Ordering::Relaxed);
        self.attribution_requests(attribution)
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_success(&self) {
        self.successful_reads.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_missing(&self) {
        self.missing_did_reads.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_startup_index_read(&self) {
        self.startup_index_reads.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_direct_btree_fallback_read(&self) {
        self.direct_btree_fallback_reads
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_full_index_read(&self) {
        self.full_index_reads.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_contiguous_read(&self, bytes: usize) {
        self.direct_single_block_reads
            .fetch_add(1, Ordering::Relaxed);
        self.contiguous_read_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_segmented_read(&self, bytes: usize) {
        self.segmented_multi_block_reads
            .fetch_add(1, Ordering::Relaxed);
        self.segmented_read_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_chain_cache_hit(&self) {
        self.chain_cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_chain_cache_miss(&self) {
        self.chain_cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_chain_cache_eviction(&self) {
        self.chain_cache_evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_contiguous_owned_copy(&self) {
        self.contiguous_reads_copied_to_owned_storage
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_attributed_success(&self, attribution: DatReadAttribution, bytes: usize) {
        self.attribution_returned_bytes(attribution)
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn summary(&self) -> DatReadTelemetrySummary {
        let total_read_requests = load(&self.total_read_requests);
        let successful_reads = load(&self.successful_reads);
        let missing_did_reads = load(&self.missing_did_reads);
        let direct_single_block_reads = load(&self.direct_single_block_reads);
        let segmented_multi_block_reads = load(&self.segmented_multi_block_reads);
        let startup_index_reads = load(&self.startup_index_reads);
        let direct_btree_fallback_reads = load(&self.direct_btree_fallback_reads);
        let full_index_reads = load(&self.full_index_reads);
        let chain_cache_hits = load(&self.chain_cache_hits);
        let chain_cache_misses = load(&self.chain_cache_misses);
        let chain_cache_evictions = load(&self.chain_cache_evictions);
        let contiguous_read_bytes = load(&self.contiguous_read_bytes);
        let segmented_read_bytes = load(&self.segmented_read_bytes);
        let contiguous_reads_copied_to_owned_storage =
            load(&self.contiguous_reads_copied_to_owned_storage);
        DatReadTelemetrySummary {
            snapshot_sequence: self.snapshot_sequence.fetch_add(1, Ordering::Relaxed) + 1,
            elapsed_micros: self
                .started_at
                .elapsed()
                .as_micros()
                .try_into()
                .unwrap_or(u64::MAX),
            total_read_requests,
            successful_reads,
            missing_did_reads,
            direct_single_block_reads,
            segmented_multi_block_reads,
            startup_index_reads,
            direct_btree_fallback_reads,
            full_index_reads,
            chain_cache_hits,
            chain_cache_misses,
            chain_cache_evictions,
            contiguous_read_bytes,
            segmented_read_bytes,
            contiguous_reads_copied_to_owned_storage,
            attribution: DatReadAttributionSummary {
                active_world_scoped_requests: load(&self.active_world_scoped_requests),
                generic_resource_requests: load(&self.generic_resource_requests),
                ephemeral_world_tooling_requests: load(&self.ephemeral_world_tooling_requests),
                unattributed_requests: load(&self.unattributed_requests),
                active_world_scoped_returned_bytes: load(&self.active_world_scoped_returned_bytes),
                generic_resource_returned_bytes: load(&self.generic_resource_returned_bytes),
                ephemeral_world_tooling_returned_bytes: load(
                    &self.ephemeral_world_tooling_returned_bytes,
                ),
                unattributed_returned_bytes: load(&self.unattributed_returned_bytes),
            },
            successful_read_percent: 0.0,
            missing_did_percent: 0.0,
            direct_single_block_percent: 0.0,
            segmented_multi_block_percent: 0.0,
            startup_index_percent: 0.0,
            direct_btree_fallback_percent: 0.0,
            full_index_percent: 0.0,
            chain_cache_hit_percent: 0.0,
            contiguous_byte_percent: 0.0,
        }
        .with_percentages()
    }

    fn attribution_requests(&self, attribution: DatReadAttribution) -> &AtomicU64 {
        match attribution {
            DatReadAttribution::ActiveWorldScoped => &self.active_world_scoped_requests,
            DatReadAttribution::GenericResource => &self.generic_resource_requests,
            DatReadAttribution::EphemeralWorldTooling => &self.ephemeral_world_tooling_requests,
            DatReadAttribution::Unattributed => &self.unattributed_requests,
        }
    }

    fn attribution_returned_bytes(&self, attribution: DatReadAttribution) -> &AtomicU64 {
        match attribution {
            DatReadAttribution::ActiveWorldScoped => &self.active_world_scoped_returned_bytes,
            DatReadAttribution::GenericResource => &self.generic_resource_returned_bytes,
            DatReadAttribution::EphemeralWorldTooling => {
                &self.ephemeral_world_tooling_returned_bytes
            }
            DatReadAttribution::Unattributed => &self.unattributed_returned_bytes,
        }
    }
}

impl DatReadAttributionSummary {
    fn delta_since(self, previous: Self) -> Self {
        Self {
            active_world_scoped_requests: self
                .active_world_scoped_requests
                .saturating_sub(previous.active_world_scoped_requests),
            generic_resource_requests: self
                .generic_resource_requests
                .saturating_sub(previous.generic_resource_requests),
            ephemeral_world_tooling_requests: self
                .ephemeral_world_tooling_requests
                .saturating_sub(previous.ephemeral_world_tooling_requests),
            unattributed_requests: self
                .unattributed_requests
                .saturating_sub(previous.unattributed_requests),
            active_world_scoped_returned_bytes: self
                .active_world_scoped_returned_bytes
                .saturating_sub(previous.active_world_scoped_returned_bytes),
            generic_resource_returned_bytes: self
                .generic_resource_returned_bytes
                .saturating_sub(previous.generic_resource_returned_bytes),
            ephemeral_world_tooling_returned_bytes: self
                .ephemeral_world_tooling_returned_bytes
                .saturating_sub(previous.ephemeral_world_tooling_returned_bytes),
            unattributed_returned_bytes: self
                .unattributed_returned_bytes
                .saturating_sub(previous.unattributed_returned_bytes),
        }
    }
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

fn percent(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::{DatFileKind, DatReadAttribution, DatReadTelemetry};

    #[test]
    fn snapshot_derives_viewer_percentages() {
        let telemetry = DatReadTelemetry::default();
        telemetry.record_request();
        telemetry.record_request();
        telemetry.record_success();
        telemetry.record_missing();
        telemetry.record_contiguous_read(8);
        telemetry.record_segmented_read(2);
        telemetry.record_chain_cache_hit();
        telemetry.record_chain_cache_miss();

        let summary = telemetry.summary();
        assert_eq!(summary.total_read_requests, 2);
        assert_eq!(summary.successful_read_percent, 50.0);
        assert_eq!(summary.direct_single_block_percent, 50.0);
        assert_eq!(summary.contiguous_byte_percent, 80.0);
        assert_eq!(summary.chain_cache_hit_percent, 50.0);
        assert!(summary
            .viewer_line(DatFileKind::Portal)
            .starts_with("Portal:"));
    }

    #[test]
    fn snapshots_are_monotonic_and_deltas_do_not_reset_counters() {
        let telemetry = DatReadTelemetry::default();
        telemetry.record_request_attributed(DatReadAttribution::ActiveWorldScoped);
        telemetry.record_success();
        telemetry.record_attributed_success(DatReadAttribution::ActiveWorldScoped, 12);
        let before = telemetry.summary();

        telemetry.record_request_attributed(DatReadAttribution::GenericResource);
        telemetry.record_success();
        telemetry.record_attributed_success(DatReadAttribution::GenericResource, 7);
        let after = telemetry.summary();
        let delta = after.delta_since(before);

        assert!(after.snapshot_sequence > before.snapshot_sequence);
        assert_eq!(after.attribution.active_world_scoped_requests, 1);
        assert_eq!(delta.total_read_requests, 1);
        assert_eq!(delta.attribution.generic_resource_requests, 1);
        assert_eq!(delta.attribution.generic_resource_returned_bytes, 7);
        assert_eq!(after.total_read_requests, 2);
    }
}
