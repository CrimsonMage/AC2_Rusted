use std::path::Path;
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;

use crate::btree::{collect_block_segments, find_file_entry, DatHeader};
use crate::btree_profile::DatBtreeProfileSnapshot;
use crate::ddd_iteration::parse_payload as parse_ddd_iteration_payload;
use crate::entry::{
    DatEntryIter, DatEntryMeta, DatFileKind, DatFileMeta, DatIdIter, DatReadRef, DatRecordRead,
    DatRecordReadRef, DatSegmentMeta,
};
use crate::error::{DatError, DatResult};
use crate::file_btree_runtime::FileBtreeRuntime;
use crate::file_index::{build_full as build_full_index, build_startup as build_startup_index};
use crate::index::DatFlatIndex;
use crate::iteration::{DatDddIterationList, RETAIL_DDD_ITERATION_DID};
use crate::memory_stats::DatMemoryStats;
use crate::mmap::DatMmapFile;
use crate::paths::DatStorageMode;
use crate::read_telemetry::{DatReadAttribution, DatReadTelemetry, DatReadTelemetrySummary};

#[derive(Debug)]
pub(crate) struct DatFileStorage {
    pub(crate) header: DatHeader,
    pub(crate) block_data_size: usize,
    pub(crate) meta: DatFileMeta,
    pub(crate) path: std::path::PathBuf,
    pub(crate) mmap: Option<DatMmapFile>,
    pub(crate) file_btree: Option<Arc<FileBtreeRuntime>>,
}

pub(crate) struct DatFile {
    storage: Arc<DatFileStorage>,
    telemetry: Arc<DatReadTelemetry>,
    startup_index: DatFlatIndex,
    full_index: OnceLock<DatFlatIndex>,
    full_index_receiver: Mutex<Option<mpsc::Receiver<DatResult<DatFlatIndex>>>>,
    full_index_error: OnceLock<String>,
}

impl std::fmt::Debug for DatFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatFile")
            .field("file_kind", &self.storage.meta.file_kind)
            .field("meta", &self.storage.meta)
            .field("startup_index_ready", &true)
            .field("full_index_ready", &self.full_index.get().is_some())
            .field("full_index_error", &self.full_index_error.get())
            .finish()
    }
}

impl DatFile {
    fn mmap(&self) -> &DatMmapFile {
        self.storage
            .mmap
            .as_ref()
            .expect("mmap-only operation selected for file B-tree storage")
    }

    fn file_btree(&self) -> Option<&FileBtreeRuntime> {
        self.storage.file_btree.as_deref()
    }

    pub(crate) fn open(
        kind: DatFileKind,
        path: impl AsRef<Path>,
        storage_mode: DatStorageMode,
    ) -> DatResult<Self> {
        let path = path.as_ref().to_path_buf();
        let (header, mmap, file_btree) = match storage_mode {
            DatStorageMode::LegacyMmapFlatIndex => {
                let mmap = DatMmapFile::open(&path).map_err(|error| match error {
                    DatError::MissingDatFile { path, .. } => {
                        DatError::MissingDatFile { kind, path }
                    }
                    other => other,
                })?;
                (DatHeader::parse(mmap.bytes(), &path)?, Some(mmap), None)
            }
            DatStorageMode::FileBtreeRuntime => {
                let runtime = FileBtreeRuntime::open(&path).map_err(|error| match error {
                    DatError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        DatError::MissingDatFile {
                            kind,
                            path: path.clone(),
                        }
                    }
                    other => other,
                })?;
                (runtime.header(), None, Some(runtime))
            }
        };
        if let Some(expected_type) = kind.expected_header_type() {
            if header.raw_type != expected_type {
                return Err(DatError::InvalidDatType {
                    path,
                    kind,
                    raw_type: header.raw_type,
                });
            }
        }

        let block_data_size = header.block_data_size(&path)?;
        let storage = Arc::new(DatFileStorage {
            header,
            block_data_size,
            meta: header.file_meta(kind),
            path,
            mmap,
            file_btree,
        });
        let telemetry = Arc::new(DatReadTelemetry::default());
        let startup_index =
            build_startup_index(storage.as_ref(), kind, Some(Arc::clone(&telemetry)))?;
        let full_index_receiver = if storage.file_btree.is_some() {
            None
        } else {
            let (sender, receiver) = mpsc::sync_channel(1);
            let build_storage = Arc::clone(&storage);
            let build_telemetry = Arc::clone(&telemetry);
            thread::spawn(move || {
                let _ = sender.send(build_full_index(
                    build_storage.as_ref(),
                    kind,
                    Some(build_telemetry),
                ));
            });
            Some(receiver)
        };

        Ok(Self {
            storage,
            telemetry,
            startup_index,
            full_index: OnceLock::new(),
            full_index_receiver: Mutex::new(full_index_receiver),
            full_index_error: OnceLock::new(),
        })
    }

    pub(crate) fn release_backing_pages(&self) -> DatResult<bool> {
        self.storage
            .mmap
            .as_ref()
            .map(DatMmapFile::release_backing_pages)
            .transpose()
            .map(|result| result.is_some())
    }

    pub(crate) fn meta(&self) -> DatFileMeta {
        self.storage.meta
    }

    pub(crate) fn read_path_stats(&self) -> crate::index::DatReadPathStats {
        self.try_full_index()
            .unwrap_or(&self.startup_index)
            .read_path_stats()
    }

    pub(crate) fn read_telemetry(&self) -> DatReadTelemetrySummary {
        self.telemetry.summary()
    }

    pub(crate) fn btree_profile(&self) -> Option<DatBtreeProfileSnapshot> {
        self.file_btree().map(FileBtreeRuntime::profile_snapshot)
    }

    pub(crate) fn memory_stats(&self) -> DatMemoryStats {
        DatMemoryStats {
            file_kind: self.storage.meta.file_kind,
            mapped_path: self.storage.path.clone(),
            mapped_bytes: self
                .storage
                .mmap
                .as_ref()
                .map_or(0, |mmap| mmap.bytes().len()),
            startup_index: self.startup_index.memory_stats(),
            full_index: self.full_index.get().map(DatFlatIndex::memory_stats),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.storage.path
    }

    pub(crate) fn contains(&self, did: u32) -> bool {
        if let Some(runtime) = self.file_btree() {
            runtime.record_metadata_only_request();
            return runtime.find_file_entry(did).ok().flatten().is_some();
        }
        if let Some(index) = self.try_full_index() {
            return index.contains(did);
        }
        if self.startup_index.contains(did) {
            return true;
        }
        if self.find_entry_via_btree(did).ok().flatten().is_some() {
            return true;
        }
        self.wait_for_full_index()
            .map(|index| index.contains(did))
            .unwrap_or(false)
    }

    pub(crate) fn entry(&self, did: u32) -> Option<DatEntryMeta> {
        if let Some(runtime) = self.file_btree() {
            runtime.record_metadata_only_request();
            return runtime
                .find_file_entry(did)
                .ok()
                .flatten()
                .map(|entry| entry.meta(self.storage.meta.file_kind));
        }
        if let Some(index) = self.try_full_index() {
            return index.entry(did);
        }
        self.startup_index
            .entry(did)
            .or_else(|| self.find_entry_via_btree(did).ok().flatten())
            .or_else(|| {
                self.wait_for_full_index()
                    .ok()
                    .and_then(|index| index.entry(did))
            })
    }

    pub(crate) fn read_packed(&self, did: u32) -> DatResult<Option<DatReadRef<'_>>> {
        self.read_record(did, DatReadAttribution::Unattributed)
            .map(|record| record.map(|record| record.packed))
    }

    pub(crate) fn read_record_owned(
        &self,
        did: u32,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordRead>> {
        if let Some(runtime) = self.file_btree() {
            self.telemetry.record_request_attributed(attribution);
            self.telemetry.record_direct_btree_fallback_read();
            let Some((entry, packed_bytes)) = runtime.read_record(did)? else {
                self.telemetry.record_missing();
                return Ok(None);
            };
            self.telemetry.record_success();
            self.telemetry.record_contiguous_read(packed_bytes.len());
            self.telemetry
                .record_attributed_success(attribution, packed_bytes.len());
            return Ok(Some(DatRecordRead {
                meta: entry.meta(self.storage.meta.file_kind),
                packed_bytes,
            }));
        }
        let Some(record) = self.read_record(did, attribution)? else {
            return Ok(None);
        };
        Ok(Some(DatRecordRead {
            meta: record.meta,
            packed_bytes: record.packed.copy_to_vec(),
        }))
    }

    pub(crate) fn read_record(
        &self,
        did: u32,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordReadRef<'_>>> {
        if self.file_btree().is_some() {
            let Some(record) = self.read_record_owned(did, attribution)? else {
                return Ok(None);
            };
            return Ok(Some(DatRecordReadRef {
                meta: record.meta,
                packed: DatReadRef::owned(record.packed_bytes)
                    .with_telemetry(self.telemetry.as_ref()),
            }));
        }
        self.telemetry.record_request_attributed(attribution);
        let Some((meta, index)) = self.resolve_entry_for_read(did)? else {
            self.telemetry.record_missing();
            return Ok(None);
        };

        let read = match index {
            Some(index) => index.read_ref_from_meta(
                self.mmap().bytes(),
                self.storage.block_data_size,
                meta,
                |meta| self.resolve_segments(meta),
            ),
            None => self.read_ref_from_meta(meta),
        }?;
        self.complete_record_read(meta, read, attribution)
    }

    fn resolve_entry_for_read(
        &self,
        did: u32,
    ) -> DatResult<Option<(DatEntryMeta, Option<&DatFlatIndex>)>> {
        if let Some(index) = self.try_full_index() {
            self.telemetry.record_full_index_read();
            return Ok(index.entry(did).map(|meta| (meta, Some(index))));
        }
        self.telemetry.record_startup_index_read();
        if let Some(meta) = self.startup_index.entry(did) {
            return Ok(Some((meta, Some(&self.startup_index))));
        }

        self.telemetry.record_direct_btree_fallback_read();
        if let Some(meta) = self.find_entry_via_btree(did)? {
            return Ok(Some((meta, None)));
        }
        match self.wait_for_full_index() {
            Ok(index) => {
                self.telemetry.record_full_index_read();
                Ok(index.entry(did).map(|meta| (meta, Some(index))))
            }
            Err(_) => Ok(None),
        }
    }

    pub(crate) fn read_stored_uncompressed(&self, did: u32) -> DatResult<Option<DatReadRef<'_>>> {
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
        if let Some(runtime) = self.file_btree() {
            runtime.record_range_request(len);
            self.telemetry.record_request();
            self.telemetry.record_direct_btree_fallback_read();
            let Some(range) = runtime.read_payload_range_for_did(did, offset, len)? else {
                self.telemetry.record_missing();
                return Ok(None);
            };
            self.telemetry.record_success();
            self.telemetry.record_contiguous_read(range.len());
            return Ok(Some(range));
        }
        self.telemetry.record_request();
        if let Some(index) = self.try_full_index() {
            self.telemetry.record_full_index_read();
            return self.complete_range_read(index.copy_range(
                self.mmap().bytes(),
                self.storage.block_data_size,
                did,
                offset,
                len,
                |meta| self.resolve_segments(meta),
            ));
        }
        self.telemetry.record_startup_index_read();
        if let Some(range) = self.startup_index.copy_range(
            self.mmap().bytes(),
            self.storage.block_data_size,
            did,
            offset,
            len,
            |meta| self.resolve_segments(meta),
        )? {
            return self.complete_range_read(Ok(Some(range)));
        }

        self.telemetry.record_direct_btree_fallback_read();
        if let Some(meta) = self.find_entry_via_btree(did)? {
            let contiguous = self.direct_single_block_slice(meta).is_some();
            let result = self.copy_range_from_meta(meta, offset, len);
            return self.complete_btree_range_read(result, contiguous);
        }
        match self.wait_for_full_index() {
            Ok(index) => {
                self.telemetry.record_full_index_read();
                self.complete_range_read(index.copy_range(
                    self.mmap().bytes(),
                    self.storage.block_data_size,
                    did,
                    offset,
                    len,
                    |meta| self.resolve_segments(meta),
                ))
            }
            Err(_) => {
                self.telemetry.record_missing();
                Ok(None)
            }
        }
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

    pub(crate) fn ddd_iteration_from_index(&self) -> DatResult<Option<DatDddIterationList>> {
        let Some(bytes) = self.read_packed(RETAIL_DDD_ITERATION_DID)? else {
            return Ok(None);
        };
        parse_ddd_iteration_payload(&bytes.copy_to_vec())
    }

    pub(crate) fn ddd_iteration_from_btree(&self) -> DatResult<Option<DatDddIterationList>> {
        if let Some(runtime) = self.file_btree() {
            let Some((_entry, bytes)) = runtime.read_record(RETAIL_DDD_ITERATION_DID)? else {
                return Ok(None);
            };
            return parse_ddd_iteration_payload(&bytes);
        }
        let Some(entry) = find_file_entry(
            self.mmap().bytes(),
            self.storage.header,
            self.mmap().path(),
            RETAIL_DDD_ITERATION_DID,
        )?
        else {
            return Ok(None);
        };
        let segments = collect_block_segments(
            self.mmap().bytes(),
            self.storage.header,
            self.mmap().path(),
            entry.offset.max(0) as usize,
            entry.size as usize,
            Some(entry.id),
            "DDD iteration payload",
        )?;
        let mut bytes = Vec::with_capacity(entry.size as usize);
        for segment in segments {
            bytes.extend_from_slice(segment.as_slice(self.mmap().bytes()));
        }
        parse_ddd_iteration_payload(&bytes)
    }

    pub(crate) fn iter_ids(&self) -> DatIdIter<'_> {
        self.blocking_full_index().iter_ids()
    }

    pub(crate) fn iter_entries(&self) -> DatEntryIter<'_> {
        self.blocking_full_index().iter_entries()
    }

    pub(crate) fn ids_in_range(&self, first: u32, last: u32) -> DatIdIter<'_> {
        self.blocking_full_index().ids_in_range(first, last)
    }

    pub(crate) fn entries_in_range(&self, first: u32, last: u32) -> DatEntryIter<'_> {
        self.blocking_full_index().entries_in_range(first, last)
    }

    fn resolve_segments(&self, meta: DatEntryMeta) -> DatResult<Arc<[DatSegmentMeta]>> {
        let start =
            usize::try_from(meta.file_offset).map_err(|_| DatError::InvalidBlockOffset {
                path: self.mmap().path().to_path_buf(),
                block_offset: usize::MAX,
                context: "file payload",
            })?;
        let segments = collect_block_segments(
            self.mmap().bytes(),
            self.storage.header,
            self.mmap().path(),
            start,
            meta.packed_len as usize,
            Some(meta.did),
            "file payload",
        )?;
        Ok(Arc::from(segments))
    }

    fn read_ref_from_meta<'a>(&'a self, meta: DatEntryMeta) -> DatResult<DatReadRef<'a>> {
        if let Some(bytes) = self.direct_single_block_slice(meta) {
            return Ok(DatReadRef::contiguous(bytes));
        }
        let segments = self.resolve_segments(meta)?;
        if segments.len() == 1 {
            return Ok(DatReadRef::contiguous(
                segments[0].as_slice(self.mmap().bytes()),
            ));
        }

        Ok(DatReadRef::segmented(
            self.mmap().bytes(),
            segments,
            meta.packed_len as usize,
        ))
    }

    fn complete_record_read<'a>(
        &'a self,
        meta: DatEntryMeta,
        read: DatReadRef<'a>,
        attribution: DatReadAttribution,
    ) -> DatResult<Option<DatRecordReadRef<'a>>> {
        self.telemetry.record_success();
        if read.is_contiguous() {
            self.telemetry.record_contiguous_read(read.len());
        } else {
            self.telemetry.record_segmented_read(read.len());
        }
        self.telemetry
            .record_attributed_success(attribution, read.len());
        Ok(Some(DatRecordReadRef {
            meta,
            packed: read.with_telemetry(self.telemetry.as_ref()),
        }))
    }

    fn complete_range_read(
        &self,
        result: DatResult<Option<Vec<u8>>>,
    ) -> DatResult<Option<Vec<u8>>> {
        match result? {
            Some(range) => {
                self.telemetry.record_success();
                Ok(Some(range))
            }
            None => {
                self.telemetry.record_missing();
                Ok(None)
            }
        }
    }

    fn complete_btree_range_read(
        &self,
        result: DatResult<Vec<u8>>,
        contiguous: bool,
    ) -> DatResult<Option<Vec<u8>>> {
        let range = result?;
        self.telemetry.record_success();
        if contiguous {
            self.telemetry.record_contiguous_read(range.len());
        } else {
            self.telemetry.record_segmented_read(range.len());
        }
        Ok(Some(range))
    }

    fn copy_range_from_meta(
        &self,
        meta: DatEntryMeta,
        offset: u64,
        len: usize,
    ) -> DatResult<Vec<u8>> {
        let packed_len = meta.packed_len as usize;
        let start = usize::try_from(offset).map_err(|_| DatError::RangeOutOfBounds {
            did: meta.did,
            offset,
            len,
            packed_len: meta.packed_len,
        })?;
        let end = start.checked_add(len).ok_or(DatError::RangeOutOfBounds {
            did: meta.did,
            offset,
            len,
            packed_len: meta.packed_len,
        })?;
        if end > packed_len {
            return Err(DatError::RangeOutOfBounds {
                did: meta.did,
                offset,
                len,
                packed_len: meta.packed_len,
            });
        }
        if len == 0 {
            return Ok(Vec::new());
        }

        if let Some(bytes) = self.direct_single_block_slice(meta) {
            return Ok(bytes[start..end].to_vec());
        }

        let segments = self.resolve_segments(meta)?;
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
            let bytes = segment.as_slice(self.mmap().bytes());
            out.extend_from_slice(&bytes[take_start..take_start + take_len]);
            if out.len() == len {
                break;
            }
            logical_offset = segment_end;
        }

        Ok(out)
    }

    fn direct_single_block_slice(&self, meta: DatEntryMeta) -> Option<&[u8]> {
        if meta.file_offset == 0
            || meta.packed_len == 0
            || meta.packed_len as usize > self.storage.block_data_size
        {
            return None;
        }
        let start = usize::try_from(meta.file_offset).ok()?.checked_add(4)?;
        let end = start.checked_add(meta.packed_len as usize)?;
        // Malformed B-tree fallback entries retain the resolver's established error behavior.
        (end <= self.mmap().bytes().len()).then(|| &self.mmap().bytes()[start..end])
    }

    fn try_full_index(&self) -> Option<&DatFlatIndex> {
        self.publish_full_index_if_ready();
        self.full_index.get()
    }

    fn blocking_full_index(&self) -> &DatFlatIndex {
        self.wait_for_full_index().unwrap_or_else(|error| {
            panic!(
                "DAT full index build failed for {:?} {}: {error}",
                self.storage.meta.file_kind,
                self.path().display()
            )
        })
    }

    fn wait_for_full_index(&self) -> DatResult<&DatFlatIndex> {
        if let Some(index) = self.full_index.get() {
            return Ok(index);
        }
        if let Some(error) = self.full_index_error.get() {
            return Err(background_index_error(error));
        }

        let receiver = self
            .full_index_receiver
            .lock()
            .expect("full index receiver mutex should not be poisoned")
            .take();
        let Some(receiver) = receiver else {
            if self.file_btree().is_some() {
                let index = build_full_index(
                    self.storage.as_ref(),
                    self.storage.meta.file_kind,
                    Some(Arc::clone(&self.telemetry)),
                )?;
                let _ = self.full_index.set(index);
                return Ok(self
                    .full_index
                    .get()
                    .expect("full index was just published"));
            }
            return self
                .full_index
                .get()
                .ok_or_else(|| background_index_error("background full index was not available"));
        };

        match receiver.recv() {
            Ok(Ok(index)) => {
                let _ = self.full_index.set(index);
                Ok(self
                    .full_index
                    .get()
                    .expect("full index should be populated after a successful build"))
            }
            Ok(Err(error)) => {
                let message = error.to_string();
                let _ = self.full_index_error.set(message.clone());
                Err(background_index_error(&message))
            }
            Err(_) => {
                let message = "background full index worker disconnected".to_string();
                let _ = self.full_index_error.set(message.clone());
                Err(background_index_error(&message))
            }
        }
    }

    fn publish_full_index_if_ready(&self) {
        if self.full_index.get().is_some() || self.full_index_error.get().is_some() {
            return;
        }

        let mut receiver = self
            .full_index_receiver
            .lock()
            .expect("full index receiver mutex should not be poisoned");
        let Some(active) = receiver.as_ref() else {
            return;
        };

        match active.try_recv() {
            Ok(Ok(index)) => {
                let _ = self.full_index.set(index);
                receiver.take();
            }
            Ok(Err(error)) => {
                let _ = self.full_index_error.set(error.to_string());
                receiver.take();
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                let _ = self
                    .full_index_error
                    .set("background full index worker disconnected".to_string());
                receiver.take();
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }

    fn find_entry_via_btree(&self, did: u32) -> DatResult<Option<DatEntryMeta>> {
        if let Some(runtime) = self.file_btree() {
            return Ok(runtime
                .find_file_entry(did)?
                .map(|entry| entry.meta(self.storage.meta.file_kind)));
        }
        let Some(raw) = find_file_entry(
            self.mmap().bytes(),
            self.storage.header,
            self.mmap().path(),
            did,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(raw.meta(self.storage.meta.file_kind)))
    }
}

fn background_index_error(message: &str) -> DatError {
    DatError::Io(std::io::Error::other(message.to_string()))
}
