//! DAT mounting, DID lookup, entry metadata, packed storage-byte reads, and small range reads.
//!
//! This crate stays at the storage layer and reads the way the retail `DiskController` does:
//! every DID lookup walks the on-disk B-tree (root pinned, 100 replaceable nodes cached), every
//! record is read on demand with positioned reads into a caller-owned buffer, and nothing is
//! indexed or mapped at open.
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
mod iteration;
mod paths;
mod read_telemetry;
#[cfg(feature = "test-support")]
pub mod test_support;

pub use archive::{ArchiveCursor, ArchiveError, ArchiveResult};
pub use btree_profile::DatBtreeProfileSnapshot;
pub use dataset::DatDataset;
pub use entry::{DatEntryIter, DatFileMeta, DatIdIter, DatIdStamp, DatRecordRead};
pub use entry::{DatEntryMeta, DatFileKind};
pub use error::{DatError, DatResult};
pub use iteration::RETAIL_DDD_ITERATION_DID;
pub use iteration::{DatDddIterationList, DatDddIterationPair};
pub use paths::is_dat_mount_root;
pub use paths::DatOpenOptions;
pub use read_telemetry::DatReadAttributionSummary;
pub use read_telemetry::{DatReadAttribution, DatReadTelemetrySummary};
