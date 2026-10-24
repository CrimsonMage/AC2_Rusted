use std::sync::Arc;

use serde::Serialize;

use crate::index::{DatIndexPayload, DatIndexRecord};
use crate::read_telemetry::DatReadTelemetry;

/// The fixed-width entry shape stored in a retail DAT B-tree leaf.
///
/// The DID index retains this compact source shape and materializes [`DatEntryMeta`] only at the
/// public API boundary. A mounted DAT has one file kind, so repeating that value in every index
/// record would waste eight bytes after alignment.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DatRawFileEntry {
    pub(crate) flags: u16,
    pub(crate) version: u16,
    pub(crate) id: u32,
    pub(crate) offset: i32,
    pub(crate) size: u32,
    pub(crate) raw_date: u32,
    pub(crate) iteration: i32,
}

impl DatRawFileEntry {
    pub(crate) fn meta(self, file_kind: DatFileKind) -> DatEntryMeta {
        DatEntryMeta {
            did: self.id,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum DatFileKind {
    Portal,
    Cell,
    Local,
    HighRes,
}

impl DatFileKind {
    pub const ALL: [Self; 4] = [Self::Portal, Self::Cell, Self::Local, Self::HighRes];

    /// File name an Asheron's Call 2 installation uses for this DAT role.
    pub const fn default_file_name(self) -> &'static str {
        match self {
            Self::Portal => "portal.dat",
            Self::Cell => "cell_1.dat",
            Self::Local => "local_English.dat",
            Self::HighRes => "highres.dat",
        }
    }

    pub(crate) const fn expected_header_type(self) -> Option<u32> {
        match self {
            Self::Portal => Some(1),
            Self::Cell => Some(2),
            Self::Local => Some(3),
            Self::HighRes => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DatIdStamp {
    pub major: [u8; 16],
    pub minor: u32,
}

impl DatIdStamp {
    pub fn is_zero(self) -> bool {
        self.major == [0; 16] && self.minor == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatFileMeta {
    pub file_kind: DatFileKind,
    pub dataset_type: u32,
    pub subset: u32,
    pub block_size: i32,
    pub file_size: u32,
    pub root_block_offset: i32,
    pub master_map_did: u32,
    pub engine_pack_version: i32,
    pub game_pack_version: i32,
    pub dat_id_stamp: DatIdStamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatEntryMeta {
    pub did: u32,
    pub file_kind: DatFileKind,
    pub file_offset: u64,
    pub packed_len: u32,
    pub flags: u16,
    pub version: u16,
    pub date_stamp: u32,
    pub iteration: i32,
}

/// One resolved DAT record: immutable entry metadata and its packed storage-byte view.
///
/// This keeps one caller request from repeating DID resolution before decode. The packed view is
/// still mmap-backed whenever the stored record is contiguous; it does not retain decoded data or
/// introduce a raw-byte cache.
#[derive(Debug, Clone)]
pub struct DatRecordReadRef<'a> {
    pub meta: DatEntryMeta,
    pub packed: DatReadRef<'a>,
}

/// One resolved DAT record with transient owned packed storage bytes.
///
/// This is the storage boundary used by the file-handle candidate. The bytes have exactly the
/// caller's lifetime and are never retained by `ac2-dat` after the value is dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatRecordRead {
    pub meta: DatEntryMeta,
    pub packed_bytes: Vec<u8>,
}

impl DatEntryMeta {
    /// Returns `true` when the on-disk DAT payload is compressed.
    pub const fn is_compressed(self) -> bool {
        self.flags & 1 != 0
    }

    /// Returns `true` when the on-disk DAT payload is compressed.
    pub const fn is_stored_compressed(self) -> bool {
        self.is_compressed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DatSegmentMeta {
    pub start: usize,
    pub end: usize,
}

impl DatSegmentMeta {
    pub(crate) fn len(self) -> usize {
        self.end - self.start
    }

    pub(crate) fn as_slice(self, mmap: &[u8]) -> &[u8] {
        &mmap[self.start..self.end]
    }
}

#[derive(Debug, Clone)]
enum DatReadRefInner<'a> {
    Contiguous(&'a [u8]),
    Owned(Vec<u8>),
    Segmented {
        mmap: &'a [u8],
        segments: Arc<[DatSegmentMeta]>,
    },
}

#[derive(Debug, Clone)]
pub struct DatReadRef<'a> {
    inner: DatReadRefInner<'a>,
    len: usize,
    telemetry: Option<&'a DatReadTelemetry>,
}

impl<'a> DatReadRef<'a> {
    pub(crate) fn contiguous(bytes: &'a [u8]) -> Self {
        Self {
            inner: DatReadRefInner::Contiguous(bytes),
            len: bytes.len(),
            telemetry: None,
        }
    }

    pub(crate) fn owned(bytes: Vec<u8>) -> Self {
        let len = bytes.len();
        Self {
            inner: DatReadRefInner::Owned(bytes),
            len,
            telemetry: None,
        }
    }

    pub(crate) fn segmented(mmap: &'a [u8], segments: Arc<[DatSegmentMeta]>, len: usize) -> Self {
        Self {
            inner: DatReadRefInner::Segmented { mmap, segments },
            len,
            telemetry: None,
        }
    }

    pub(crate) fn with_telemetry(mut self, telemetry: &'a DatReadTelemetry) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` when the payload is one contiguous mmap-backed slice.
    pub const fn is_contiguous(&self) -> bool {
        matches!(self.inner, DatReadRefInner::Contiguous(_))
    }

    /// Returns the raw packed DAT bytes when they are one contiguous mmap-backed slice.
    pub fn as_slice(&self) -> Option<&[u8]> {
        match &self.inner {
            DatReadRefInner::Contiguous(bytes) => Some(bytes),
            DatReadRefInner::Owned(bytes) => Some(bytes),
            DatReadRefInner::Segmented { .. } => None,
        }
    }

    /// Materializes the raw packed DAT bytes into one owned buffer.
    pub fn copy_to_vec(&self) -> Vec<u8> {
        match &self.inner {
            DatReadRefInner::Contiguous(bytes) => {
                if let Some(telemetry) = self.telemetry {
                    telemetry.record_contiguous_owned_copy();
                }
                bytes.to_vec()
            }
            DatReadRefInner::Owned(bytes) => bytes.clone(),
            DatReadRefInner::Segmented { mmap, segments } => {
                let mut out = Vec::with_capacity(self.len);
                for segment in segments.iter().copied() {
                    out.extend_from_slice(segment.as_slice(mmap));
                }
                out
            }
        }
    }

    pub fn copy_to_slice(&self, target: &mut [u8]) -> bool {
        if target.len() != self.len {
            return false;
        }

        match &self.inner {
            DatReadRefInner::Contiguous(bytes) => target.copy_from_slice(bytes),
            DatReadRefInner::Owned(bytes) => target.copy_from_slice(bytes),
            DatReadRefInner::Segmented { mmap, segments } => {
                let mut offset = 0usize;
                for segment in segments.iter().copied() {
                    let bytes = segment.as_slice(mmap);
                    let next = offset + bytes.len();
                    target[offset..next].copy_from_slice(bytes);
                    offset = next;
                }
            }
        }
        true
    }

    /// Iterates the raw packed DAT bytes as zero-copy mmap-backed segments.
    pub fn segments(&'a self) -> DatReadSegments<'a> {
        match &self.inner {
            DatReadRefInner::Contiguous(bytes) => DatReadSegments::contiguous(bytes),
            DatReadRefInner::Owned(bytes) => DatReadSegments::owned(bytes),
            DatReadRefInner::Segmented { mmap, segments } => {
                DatReadSegments::segmented(mmap, Arc::clone(segments))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum DatReadSegmentsInner<'a> {
    Contiguous(Option<&'a [u8]>),
    Owned(Option<&'a [u8]>),
    Segmented {
        mmap: &'a [u8],
        segments: Arc<[DatSegmentMeta]>,
        index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct DatReadSegments<'a> {
    inner: DatReadSegmentsInner<'a>,
}

impl<'a> DatReadSegments<'a> {
    fn contiguous(bytes: &'a [u8]) -> Self {
        Self {
            inner: DatReadSegmentsInner::Contiguous(Some(bytes)),
        }
    }

    fn segmented(mmap: &'a [u8], segments: Arc<[DatSegmentMeta]>) -> Self {
        Self {
            inner: DatReadSegmentsInner::Segmented {
                mmap,
                segments,
                index: 0,
            },
        }
    }

    fn owned(bytes: &'a [u8]) -> Self {
        Self {
            inner: DatReadSegmentsInner::Owned(Some(bytes)),
        }
    }
}

impl<'a> Iterator for DatReadSegments<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            DatReadSegmentsInner::Contiguous(bytes) => bytes.take(),
            DatReadSegmentsInner::Owned(bytes) => bytes.take(),
            DatReadSegmentsInner::Segmented {
                mmap,
                segments,
                index,
            } => {
                let segment = segments.get(*index).copied()?;
                *index += 1;
                Some(segment.as_slice(mmap))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DatIdIterSource<'a> {
    Records(&'a [DatIndexRecord]),
    Ids(&'a [u32]),
}

#[derive(Debug, Clone, Copy)]
pub struct DatIdIter<'a> {
    source: DatIdIterSource<'a>,
    index: usize,
}

impl<'a> DatIdIter<'a> {
    pub(crate) const fn new(records: &'a [DatIndexRecord]) -> Self {
        Self {
            source: DatIdIterSource::Records(records),
            index: 0,
        }
    }

    pub(crate) const fn from_ids(ids: &'a [u32]) -> Self {
        Self {
            source: DatIdIterSource::Ids(ids),
            index: 0,
        }
    }

    pub(crate) const fn empty() -> Self {
        Self::new(&[])
    }
}

impl<'a> Iterator for DatIdIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let id = match self.source {
            DatIdIterSource::Records(records) => records.get(self.index)?.id,
            DatIdIterSource::Ids(ids) => *ids.get(self.index)?,
        };
        self.index += 1;
        Some(id)
    }
}

#[derive(Debug, Clone, Copy)]
enum DatEntryIterSource<'a> {
    Records(&'a [DatIndexRecord]),
    Split {
        ids: &'a [u32],
        payloads: &'a [DatIndexPayload],
    },
}

#[derive(Debug, Clone, Copy)]
pub struct DatEntryIter<'a> {
    source: DatEntryIterSource<'a>,
    file_kind: DatFileKind,
    index: usize,
}

impl<'a> DatEntryIter<'a> {
    pub(crate) const fn new(records: &'a [DatIndexRecord], file_kind: DatFileKind) -> Self {
        Self {
            source: DatEntryIterSource::Records(records),
            file_kind,
            index: 0,
        }
    }

    pub(crate) const fn from_split(
        ids: &'a [u32],
        payloads: &'a [DatIndexPayload],
        file_kind: DatFileKind,
    ) -> Self {
        Self {
            source: DatEntryIterSource::Split { ids, payloads },
            file_kind,
            index: 0,
        }
    }

    pub(crate) const fn empty() -> Self {
        Self::new(&[], DatFileKind::Portal)
    }
}

impl<'a> Iterator for DatEntryIter<'a> {
    type Item = DatEntryMeta;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = match self.source {
            DatEntryIterSource::Records(records) => records.get(self.index)?.meta(self.file_kind),
            DatEntryIterSource::Split { ids, payloads } => {
                let did = *ids.get(self.index)?;
                payloads.get(self.index)?.meta(did, self.file_kind)
            }
        };
        self.index += 1;
        Some(entry)
    }
}
