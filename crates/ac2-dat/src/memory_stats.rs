use std::path::PathBuf;

use serde::Serialize;

use crate::entry::DatFileKind;

/// Logical allocations retained by one mounted DAT DID index.
///
/// This intentionally reports only container payload sizes that DAT owns. Allocator capacity,
/// mutexes, and host page residency belong to their respective diagnostic layers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DatIndexMemoryStats {
    pub record_count: usize,
    pub did_bytes: usize,
    pub payload_bytes: usize,
    pub prefix_directory_bytes: usize,
    pub flat_record_bytes: usize,
    pub segment_metadata_retained_bytes: usize,
    pub segment_metadata_capacity_bytes: usize,
}

/// Read-only logical allocation and mapping facts for one mounted DAT.
///
/// The mmap's OS residency is intentionally not inferred here; host mapping diagnostics own
/// PSS/RSS observation. These values do not change DAT retention or page behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DatMemoryStats {
    pub file_kind: DatFileKind,
    pub mapped_path: PathBuf,
    pub mapped_bytes: usize,
    pub startup_index: DatIndexMemoryStats,
    pub full_index: Option<DatIndexMemoryStats>,
}
