use std::collections::BTreeMap;
use std::sync::Arc;

use crate::btree_profile::DatBtreeProfileSnapshot;
use crate::entry::{
    DatEntryIter, DatEntryMeta, DatFileKind, DatFileMeta, DatIdIter, DatReadRef, DatRecordRead,
    DatRecordReadRef,
};
use crate::error::DatResult;
use crate::file::DatFile;
use crate::index::DatReadPathStats;
use crate::iteration::DatDddIterationList;
use crate::memory_stats::DatMemoryStats;
use crate::paths::DatOpenOptions;
use crate::read_telemetry::{DatReadAttribution, DatReadTelemetrySummary};

#[derive(Debug)]
pub struct DatDataset {
    files: BTreeMap<DatFileKind, DatFile>,
}

impl DatDataset {
    pub fn open(options: DatOpenOptions) -> DatResult<Arc<Self>> {
        let mut file_kinds = vec![DatFileKind::Portal, DatFileKind::Cell, DatFileKind::Local];
        if options.high_res_enabled {
            file_kinds.push(DatFileKind::HighRes);
        }

        let files = std::thread::scope(|scope| {
            let mut tasks = Vec::with_capacity(file_kinds.len());
            for kind in file_kinds.iter().copied() {
                let path = options.path_for(kind);
                let storage_mode = options.storage_mode;
                tasks.push((
                    kind,
                    scope.spawn(move || DatFile::open(kind, path, storage_mode)),
                ));
            }

            let mut files = BTreeMap::new();
            let mut first_error = None;
            for (kind, task) in tasks {
                let result = match task.join() {
                    Ok(result) => result,
                    Err(panic) => std::panic::resume_unwind(panic),
                };
                match result {
                    Ok(file) => {
                        if first_error.is_none() {
                            files.insert(kind, file);
                        }
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                    }
                }
            }

            first_error.map_or(Ok(files), Err)
        })?;

        Ok(Arc::new(Self { files }))
    }

    pub fn contains(&self, kind: DatFileKind, did: u32) -> bool {
        self.file(kind).is_some_and(|file| file.contains(did))
    }

    pub fn entry(&self, kind: DatFileKind, did: u32) -> Option<DatEntryMeta> {
        self.file(kind).and_then(|file| file.entry(did))
    }

    pub fn file_meta(&self, kind: DatFileKind) -> Option<DatFileMeta> {
        self.file(kind).map(DatFile::meta)
    }

    /// Temporary storage-path counters for validating DAT read/cache behavior in a live viewer.
    pub fn read_path_stats(&self, kind: DatFileKind) -> Option<DatReadPathStats> {
        self.file(kind).map(DatFile::read_path_stats)
    }

    /// Temporary process-lifetime read telemetry for one mounted DAT.
    ///
    /// This is owned by `DatFile`, rather than either flat index, so startup-index and direct
    /// B-tree fallback traffic remains visible after the background full index is published.
    pub fn read_telemetry(&self, kind: DatFileKind) -> Option<DatReadTelemetrySummary> {
        self.file(kind).map(DatFile::read_telemetry)
    }

    /// Count-only observations of the retail-shaped file B-tree path for one mounted DAT.
    ///
    /// Legacy mmap storage has no matching profile and returns `None`.
    pub fn btree_profile(&self, kind: DatFileKind) -> Option<DatBtreeProfileSnapshot> {
        self.file(kind).and_then(DatFile::btree_profile)
    }

    /// Read-only mapping and logical index allocation facts for one mounted DAT.
    ///
    /// This does not wait for background index construction, change mapping residency, or
    /// otherwise affect the mounted file.
    pub fn memory_stats(&self, kind: DatFileKind) -> Option<DatMemoryStats> {
        self.file(kind).map(DatFile::memory_stats)
    }

    /// Releases reclaimable pages of immutable DAT mappings while keeping map/index identity.
    pub fn release_backing_pages(&self) -> usize {
        self.files
            .values()
            .filter(|file| file.release_backing_pages().unwrap_or(false))
            .count()
    }

    pub fn read_packed(&self, kind: DatFileKind, did: u32) -> DatResult<Option<DatReadRef<'_>>> {
        match self.file(kind) {
            Some(file) => file.read_packed(did),
            None => Ok(None),
        }
    }

    /// Resolves a DID once and returns both its immutable metadata and packed mmap-backed view.
    ///
    /// Typed decode belongs above DAT. The attribution is diagnostic-only and cannot affect
    /// lookup, read, or ownership behavior.
    pub fn read_record(
        &self,
        kind: DatFileKind,
        did: u32,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordReadRef<'_>>> {
        match self.file(kind) {
            Some(file) => file.read_record(did, attribution),
            None => Ok(None),
        }
    }

    /// Resolves one DID and returns transient owned packed bytes for decode paths that must not
    /// borrow a mounted mapping.
    pub fn read_record_owned(
        &self,
        kind: DatFileKind,
        did: u32,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordRead>> {
        match self.file(kind) {
            Some(file) => file.read_record_owned(did, attribution),
            None => Ok(None),
        }
    }

    pub fn read_stored_uncompressed(
        &self,
        kind: DatFileKind,
        did: u32,
    ) -> DatResult<Option<DatReadRef<'_>>> {
        match self.file(kind) {
            Some(file) => file.read_stored_uncompressed(did),
            None => Ok(None),
        }
    }

    pub fn read_range(
        &self,
        kind: DatFileKind,
        did: u32,
        offset: u64,
        len: usize,
    ) -> DatResult<Option<Vec<u8>>> {
        match self.file(kind) {
            Some(file) => file.read_range(did, offset, len),
            None => Ok(None),
        }
    }

    pub fn uncompressed_size(&self, kind: DatFileKind, did: u32) -> DatResult<Option<u32>> {
        match self.file(kind) {
            Some(file) => file.uncompressed_size(did),
            None => Ok(None),
        }
    }

    pub fn ddd_iteration(&self, kind: DatFileKind) -> DatResult<Option<DatDddIterationList>> {
        match self.file(kind) {
            Some(file) => file.ddd_iteration_from_index(),
            None => Ok(None),
        }
    }

    pub fn ddd_iteration_via_btree(
        &self,
        kind: DatFileKind,
    ) -> DatResult<Option<DatDddIterationList>> {
        match self.file(kind) {
            Some(file) => file.ddd_iteration_from_btree(),
            None => Ok(None),
        }
    }

    pub fn iter_ids(&self, kind: DatFileKind) -> DatIdIter<'_> {
        self.file(kind)
            .map(DatFile::iter_ids)
            .unwrap_or_else(DatIdIter::empty)
    }

    pub fn iter_entries(&self, kind: DatFileKind) -> DatEntryIter<'_> {
        self.file(kind)
            .map(DatFile::iter_entries)
            .unwrap_or_else(DatEntryIter::empty)
    }

    pub fn ids_in_range(&self, kind: DatFileKind, first: u32, last: u32) -> DatIdIter<'_> {
        self.file(kind)
            .map(|file| file.ids_in_range(first, last))
            .unwrap_or_else(DatIdIter::empty)
    }

    pub fn entries_in_range(&self, kind: DatFileKind, first: u32, last: u32) -> DatEntryIter<'_> {
        self.file(kind)
            .map(|file| file.entries_in_range(first, last))
            .unwrap_or_else(DatEntryIter::empty)
    }

    fn file(&self, kind: DatFileKind) -> Option<&DatFile> {
        self.files.get(&kind)
    }
}
