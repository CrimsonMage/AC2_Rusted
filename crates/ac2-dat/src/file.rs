use std::path::Path;

use crate::btree_profile::DatBtreeProfileSnapshot;
use crate::ddd_iteration::parse_payload as parse_ddd_iteration_payload;
use crate::entry::{
    DatEntryIter, DatEntryMeta, DatFileKind, DatFileMeta, DatIdIter, DatRawFileEntry, DatRecordRead,
};
use crate::error::{DatError, DatResult};
use crate::file_btree_runtime::FileBtreeRuntime;
use crate::iteration::{DatDddIterationList, RETAIL_DDD_ITERATION_DID};
use crate::read_telemetry::{DatReadAttribution, DatReadTelemetry, DatReadTelemetrySummary};

/// One mounted DAT (`DiskController`): every lookup walks the on-disk B-tree and every record is
/// read on demand with positioned reads (`DiskDev::SyncRead`). Nothing is indexed at open.
#[derive(Debug)]
pub(crate) struct DatFile {
    runtime: FileBtreeRuntime,
    meta: DatFileMeta,
    block_data_size: usize,
    telemetry: DatReadTelemetry,
}

impl DatFile {
    pub(crate) fn open(kind: DatFileKind, path: impl AsRef<Path>) -> DatResult<Self> {
        let path = path.as_ref().to_path_buf();
        let runtime = FileBtreeRuntime::open(&path).map_err(|error| match error {
            DatError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
                DatError::MissingDatFile {
                    kind,
                    path: path.clone(),
                }
            }
            other => other,
        })?;
        let header = runtime.header();
        if let Some(expected_type) = kind.expected_header_type() {
            if header.raw_type != expected_type {
                return Err(DatError::InvalidDatType {
                    path,
                    kind,
                    raw_type: header.raw_type,
                });
            }
        }
        Ok(Self {
            block_data_size: header.block_data_size(&path)?,
            meta: header.file_meta(kind),
            runtime,
            telemetry: DatReadTelemetry::default(),
        })
    }

    pub(crate) fn meta(&self) -> DatFileMeta {
        self.meta
    }

    pub(crate) fn read_telemetry(&self) -> DatReadTelemetrySummary {
        self.telemetry.summary()
    }

    pub(crate) fn btree_profile(&self) -> DatBtreeProfileSnapshot {
        self.runtime.profile_snapshot()
    }

    pub(crate) fn contains(&self, did: u32) -> bool {
        self.entry(did).is_some()
    }

    pub(crate) fn entry(&self, did: u32) -> Option<DatEntryMeta> {
        self.runtime.record_metadata_only_request();
        self.runtime
            .find_file_entry(did)
            .ok()
            .flatten()
            .map(|entry| entry.meta(self.meta.file_kind))
    }

    /// `DiskController::LoadDataEx`: one B-tree lookup, then the record's block chain.
    pub(crate) fn read_record(
        &self,
        did: u32,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordRead>> {
        self.telemetry.record_request_attributed(attribution);
        let Some((entry, packed_bytes)) = self.runtime.read_record(did)? else {
            self.telemetry.record_missing();
            return Ok(None);
        };
        self.telemetry.record_success();
        self.record_returned_bytes(packed_bytes.len());
        self.telemetry
            .record_attributed_success(attribution, packed_bytes.len());
        Ok(Some(DatRecordRead {
            meta: entry.meta(self.meta.file_kind),
            packed_bytes,
        }))
    }

    pub(crate) fn read_packed(&self, did: u32) -> DatResult<Option<Vec<u8>>> {
        self.read_record(did, DatReadAttribution::Unattributed)
            .map(|record| record.map(|record| record.packed_bytes))
    }

    pub(crate) fn read_stored_uncompressed(&self, did: u32) -> DatResult<Option<Vec<u8>>> {
        let Some(entry) = self.entry(did) else {
            return Ok(None);
        };
        if entry.is_stored_compressed() {
            return Err(DatError::StoredCompressedEntry {
                kind: entry.file_kind,
                did,
            });
        }
        self.read_packed(did)
    }

    pub(crate) fn read_range(
        &self,
        did: u32,
        offset: u64,
        len: usize,
    ) -> DatResult<Option<Vec<u8>>> {
        self.runtime.record_range_request(len);
        self.telemetry.record_request();
        let Some(range) = self.runtime.read_payload_range_for_did(did, offset, len)? else {
            self.telemetry.record_missing();
            return Ok(None);
        };
        self.telemetry.record_success();
        self.telemetry.record_contiguous_read(range.len());
        Ok(Some(range))
    }

    pub(crate) fn uncompressed_size(&self, did: u32) -> DatResult<Option<u32>> {
        let Some(entry) = self.entry(did) else {
            return Ok(None);
        };
        if !entry.is_compressed() {
            return Ok(Some(entry.packed_len));
        }

        let Some(header_bytes) = self.read_range(did, 0, 4)? else {
            return Ok(None);
        };
        Ok(Some(u32::from_le_bytes(
            header_bytes.as_slice().try_into().unwrap(),
        )))
    }

    /// `DiskController::LoadIterationList`.
    pub(crate) fn ddd_iteration(&self) -> DatResult<Option<DatDddIterationList>> {
        let Some(bytes) = self.read_packed(RETAIL_DDD_ITERATION_DID)? else {
            return Ok(None);
        };
        parse_ddd_iteration_payload(&bytes)
    }

    pub(crate) fn ids_in_range(&self, first: u32, last: u32) -> DatIdIter {
        DatIdIter::new(self.entries_between(first, last))
    }

    pub(crate) fn entries_in_range(&self, first: u32, last: u32) -> DatEntryIter {
        DatEntryIter::new(self.entries_between(first, last), self.meta.file_kind)
    }

    fn entries_between(&self, first: u32, last: u32) -> Vec<DatRawFileEntry> {
        self.runtime
            .collect_entries_in_range(first, last)
            .unwrap_or_else(|error| {
                panic!(
                    "DAT B-tree walk failed for {:?} {first:08X}..={last:08X}: {error}",
                    self.meta.file_kind
                )
            })
    }

    /// A record that fits one block is a single `SyncRead`; a longer one follows its chain.
    fn record_returned_bytes(&self, bytes: usize) {
        if bytes <= self.block_data_size {
            self.telemetry.record_contiguous_read(bytes);
        } else {
            self.telemetry.record_segmented_read(bytes);
        }
    }
}
