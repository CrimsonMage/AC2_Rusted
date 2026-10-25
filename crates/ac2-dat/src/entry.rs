use serde::Serialize;

/// The fixed-width entry shape stored in a retail DAT B-tree node (`BTEntry`).
///
/// Lookups and range walks carry this compact source shape and materialize [`DatEntryMeta`] only
/// at the public API boundary.
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

/// One resolved DAT record: its entry metadata and packed storage bytes, read into a caller-owned
/// buffer as `CLBlockAllocator::Load_Data` does. `ac2-dat` retains nothing after it returns.
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

/// DIDs of one `BTree::Get_Id_List_Range` walk, in ascending order.
#[derive(Debug, Clone)]
pub struct DatIdIter {
    entries: std::vec::IntoIter<DatRawFileEntry>,
}

impl DatIdIter {
    pub(crate) fn new(entries: Vec<DatRawFileEntry>) -> Self {
        Self {
            entries: entries.into_iter(),
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(Vec::new())
    }
}

impl Iterator for DatIdIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(|entry| entry.id)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl ExactSizeIterator for DatIdIter {}

/// Entries of one `BTree::Get_Id_List_Range` walk, in ascending DID order.
#[derive(Debug, Clone)]
pub struct DatEntryIter {
    entries: std::vec::IntoIter<DatRawFileEntry>,
    file_kind: DatFileKind,
}

impl DatEntryIter {
    pub(crate) fn new(entries: Vec<DatRawFileEntry>, file_kind: DatFileKind) -> Self {
        Self {
            entries: entries.into_iter(),
            file_kind,
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(Vec::new(), DatFileKind::Portal)
    }
}

impl Iterator for DatEntryIter {
    type Item = DatEntryMeta;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().map(|entry| entry.meta(self.file_kind))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl ExactSizeIterator for DatEntryIter {}
