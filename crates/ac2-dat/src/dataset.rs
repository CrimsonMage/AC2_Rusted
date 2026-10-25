use std::collections::BTreeMap;
use std::sync::Arc;

use crate::btree_profile::DatBtreeProfileSnapshot;
use crate::entry::{
    DatEntryIter, DatEntryMeta, DatFileKind, DatFileMeta, DatIdIter, DatRecordRead,
};
use crate::error::DatResult;
use crate::file::DatFile;
use crate::iteration::DatDddIterationList;
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
                tasks.push((kind, scope.spawn(move || DatFile::open(kind, path))));
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

    /// Process-lifetime read telemetry for one mounted DAT.
    pub fn read_telemetry(&self, kind: DatFileKind) -> Option<DatReadTelemetrySummary> {
        self.file(kind).map(DatFile::read_telemetry)
    }

    /// Count-only observations of the B-tree lookup, node-cache and positioned-read path.
    pub fn btree_profile(&self, kind: DatFileKind) -> Option<DatBtreeProfileSnapshot> {
        self.file(kind).map(DatFile::btree_profile)
    }

    /// The record's packed storage bytes, unattributed.
    pub fn read_packed(&self, kind: DatFileKind, did: u32) -> DatResult<Option<Vec<u8>>> {
        match self.file(kind) {
            Some(file) => file.read_packed(did),
            None => Ok(None),
        }
    }

    /// Resolves a DID once and reads its packed bytes into an owned buffer. Typed decode belongs
    /// above DAT. The attribution is diagnostic-only and cannot affect lookup or read behavior.
    pub fn read_record(
        &self,
        kind: DatFileKind,
        did: u32,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordRead>> {
        match self.file(kind) {
            Some(file) => file.read_record(did, attribution),
            None => Ok(None),
        }
    }

    pub fn read_stored_uncompressed(
        &self,
        kind: DatFileKind,
        did: u32,
    ) -> DatResult<Option<Vec<u8>>> {
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
            Some(file) => file.ddd_iteration(),
            None => Ok(None),
        }
    }

    /// Every DID of one DAT, walked from its B-tree on each call (`BTree::Get_Id_List`).
    pub fn iter_ids(&self, kind: DatFileKind) -> DatIdIter {
        self.ids_in_range(kind, 0, u32::MAX)
    }

    pub fn iter_entries(&self, kind: DatFileKind) -> DatEntryIter {
        self.entries_in_range(kind, 0, u32::MAX)
    }

    /// DIDs in `first..=last`, walked from the B-tree on each call (`BTree::Get_Id_List_Range`).
    pub fn ids_in_range(&self, kind: DatFileKind, first: u32, last: u32) -> DatIdIter {
        self.file(kind)
            .map(|file| file.ids_in_range(first, last))
            .unwrap_or_else(DatIdIter::empty)
    }

    pub fn entries_in_range(&self, kind: DatFileKind, first: u32, last: u32) -> DatEntryIter {
        self.file(kind)
            .map(|file| file.entries_in_range(first, last))
            .unwrap_or_else(DatEntryIter::empty)
    }

    fn file(&self, kind: DatFileKind) -> Option<&DatFile> {
        self.files.get(&kind)
    }
}
