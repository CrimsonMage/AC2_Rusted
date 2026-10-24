//! DAT mounting, read-only storage access, DID lookup, entry metadata, packed storage-byte reads,
//! and small range reads.
//!
//! This crate stays at the storage layer:
//! - `read_packed` returns the on-disk DAT payload bytes, with zero-copy mmap borrowing only in
//!   the legacy comparison mode.
//! - `read_stored_uncompressed` only succeeds for records that are already stored uncompressed on disk.
//! - decompression and typed decode belong above this crate.

mod archive;
mod btree;
mod btree_profile;
mod dataset;
mod ddd_iteration;
mod entry;
mod error;
mod file;
mod file_btree_runtime;
mod file_index;
mod index;
mod iteration;
mod memory_stats;
mod mmap;
mod paths;
mod read_telemetry;

pub use archive::{ArchiveCursor, ArchiveError, ArchiveResult};
pub use btree_profile::DatBtreeProfileSnapshot;
pub use dataset::DatDataset;
pub use entry::{
    DatEntryIter, DatFileMeta, DatIdIter, DatIdStamp, DatReadRef, DatReadSegments, DatRecordRead,
    DatRecordReadRef,
};
pub use entry::{DatEntryMeta, DatFileKind};
pub use error::{DatError, DatResult};
pub use index::DatReadPathStats;
pub use iteration::RETAIL_DDD_ITERATION_DID;
pub use iteration::{DatDddIterationList, DatDddIterationPair};
pub use memory_stats::DatIndexMemoryStats;
pub use memory_stats::DatMemoryStats;
pub use paths::is_dat_mount_root;
pub use paths::{DatOpenOptions, DatStorageMode};
pub use read_telemetry::DatReadAttributionSummary;
pub use read_telemetry::{DatReadAttribution, DatReadTelemetrySummary};
