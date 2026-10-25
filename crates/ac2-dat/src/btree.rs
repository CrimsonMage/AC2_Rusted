use std::path::Path;

use crate::entry::{DatFileMeta, DatIdStamp, DatRawFileEntry};
use crate::error::{DatError, DatResult};

pub(crate) const DAT_HEADER_SIZE: usize = 400;
pub(crate) const DAT_HEADER_MAGIC: i32 = 0x0000_5442;
pub(crate) const DAT_BTREE_NODE_SIZE: usize = 1720;
pub(crate) const DAT_BTREE_MAX_BRANCHES: usize = 62;
pub(crate) const DAT_BTREE_MAX_FILES: usize = 61;
const UNUSED_BRANCH_SENTINEL: i32 = 0xCDCD_CDCD_u32 as i32;

/// On-disk container generation of one DAT file, detected from where the header magic sits.
///
/// Asheron's Call 2 uses the same block-chain/B-tree family as the retail client, but its header
/// fields start 20 bytes earlier and its B-tree entries are 16 bytes (DID, first block, length,
/// date) with no flags, version, or iteration words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatContainerLayout {
    Retail,
    Ac2,
}

impl DatContainerLayout {
    const fn header_base(self) -> usize {
        match self {
            Self::Retail => 320,
            Self::Ac2 => 300,
        }
    }

    pub(crate) const fn node_size(self) -> usize {
        match self {
            Self::Retail => DAT_BTREE_NODE_SIZE,
            Self::Ac2 => DAT_BTREE_MAX_BRANCHES * 4 + 4 + DAT_BTREE_MAX_FILES * 16,
        }
    }

    const fn entry_size(self) -> usize {
        match self {
            Self::Retail => 24,
            Self::Ac2 => 16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DatHeader {
    pub layout: DatContainerLayout,
    pub block_size: i32,
    pub file_size: i32,
    pub raw_type: u32,
    pub subset: u32,
    pub root_block: i32,
    pub master_map_did: u32,
    pub engine_pack_version: i32,
    pub game_pack_version: i32,
    pub dat_id_stamp: DatIdStamp,
}

impl DatHeader {
    pub(crate) fn parse(bytes: &[u8], path: &Path) -> DatResult<Self> {
        if bytes.len() < DAT_HEADER_SIZE {
            return Err(DatError::InvalidBTreeNode {
                path: path.to_path_buf(),
                block_offset: 0,
                context: "DAT file smaller than header",
            });
        }

        let magic = read_i32_le(bytes, DatContainerLayout::Retail.header_base())?;
        let layout = if magic == DAT_HEADER_MAGIC {
            DatContainerLayout::Retail
        } else if read_i32_le(bytes, DatContainerLayout::Ac2.header_base())? == DAT_HEADER_MAGIC {
            DatContainerLayout::Ac2
        } else {
            return Err(DatError::InvalidHeaderMagic {
                path: path.to_path_buf(),
                magic,
            });
        };

        let base = layout.header_base();
        let block_size = read_i32_le(bytes, base + 4)?;
        let file_size = read_i32_le(bytes, base + 8)?;
        let raw_type = read_u32_le(bytes, base + 12)?;
        let subset = read_u32_le(bytes, base + 16)?;
        let root_block = read_i32_le(bytes, base + 32)?;
        let master_map_did = read_u32_le(bytes, base + 48)?;
        let engine_pack_version = read_i32_le(bytes, base + 52)?;
        let game_pack_version = read_i32_le(bytes, base + 56)?;
        let dat_id_stamp = DatIdStamp {
            major: bytes[base + 60..base + 76].try_into().unwrap(),
            minor: read_u32_le(bytes, base + 76)?,
        };
        Ok(Self {
            layout,
            block_size,
            file_size,
            raw_type,
            subset,
            root_block,
            master_map_did,
            engine_pack_version,
            game_pack_version,
            dat_id_stamp,
        })
    }

    pub(crate) fn block_data_size(self, path: &Path) -> DatResult<usize> {
        if self.block_size <= 4 {
            return Err(DatError::InvalidBlockSize {
                path: path.to_path_buf(),
                block_size: self.block_size,
            });
        }
        Ok(self.block_size as usize - 4)
    }

    pub(crate) fn file_meta(self, file_kind: crate::entry::DatFileKind) -> DatFileMeta {
        DatFileMeta {
            file_kind,
            dataset_type: self.raw_type,
            subset: self.subset,
            block_size: self.block_size,
            file_size: self.file_size.max(0) as u32,
            root_block_offset: self.root_block,
            master_map_did: self.master_map_did,
            engine_pack_version: self.engine_pack_version,
            game_pack_version: self.game_pack_version,
            dat_id_stamp: self.dat_id_stamp,
        }
    }
}

pub(crate) type RawDatFileEntry = DatRawFileEntry;

#[derive(Debug)]
pub(crate) struct DatBTreeNode {
    pub(crate) branches: [i32; DAT_BTREE_MAX_BRANCHES],
    branch_count: usize,
    pub(crate) files: [RawDatFileEntry; DAT_BTREE_MAX_FILES],
    pub(crate) file_count: usize,
}

impl DatBTreeNode {
    pub(crate) fn parse(
        bytes: &[u8],
        path: &Path,
        block_offset: usize,
        layout: DatContainerLayout,
    ) -> DatResult<Self> {
        if bytes.len() < layout.node_size() {
            return Err(DatError::InvalidBTreeNode {
                path: path.to_path_buf(),
                block_offset,
                context: "node shorter than expected size",
            });
        }

        let mut branches = [0i32; DAT_BTREE_MAX_BRANCHES];
        let mut files = [RawDatFileEntry::default(); DAT_BTREE_MAX_FILES];
        let stored_file_count = read_i32_le(bytes, DAT_BTREE_MAX_BRANCHES * 4)?;
        if !(0..=DAT_BTREE_MAX_FILES as i32).contains(&stored_file_count) {
            return Err(DatError::InvalidBTreeNode {
                path: path.to_path_buf(),
                block_offset,
                context: "stored file count is out of bounds",
            });
        }

        let file_count = stored_file_count as usize;
        let mut branch_count = 0usize;
        let mut did_find_end = false;
        let mut last_branch = 0i32;
        for (index, branch_slot) in branches.iter_mut().enumerate() {
            let offset = index * 4;
            let branch = read_i32_le(bytes, offset)?;
            if branch == 0 || branch == last_branch || branch == UNUSED_BRANCH_SENTINEL {
                did_find_end = true;
            }
            if !did_find_end {
                last_branch = branch;
                *branch_slot = branch;
                branch_count += 1;
            }
        }

        let mut cursor = DAT_BTREE_MAX_BRANCHES * 4 + 4;
        for file_slot in files.iter_mut().take(file_count) {
            *file_slot = parse_file_entry(bytes, cursor, path, block_offset, layout)?;
            cursor += layout.entry_size();
        }

        if branch_count > 0 && file_count > 0 {
            branch_count = branch_count.min(file_count + 1);
        }

        Ok(Self {
            branches,
            branch_count,
            files,
            file_count,
        })
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.branch_count == 0
    }
}

fn parse_file_entry(
    bytes: &[u8],
    offset: usize,
    path: &Path,
    block_offset: usize,
    layout: DatContainerLayout,
) -> DatResult<RawDatFileEntry> {
    if offset + layout.entry_size() > bytes.len() {
        return Err(DatError::InvalidBTreeNode {
            path: path.to_path_buf(),
            block_offset,
            context: "file entry exceeds node bounds",
        });
    }

    if layout == DatContainerLayout::Ac2 {
        return Ok(RawDatFileEntry {
            flags: 0,
            version: 0,
            id: read_u32_le(bytes, offset)?,
            offset: read_i32_le(bytes, offset + 4)?,
            size: read_u32_le(bytes, offset + 8)?,
            raw_date: read_u32_le(bytes, offset + 12)?,
            iteration: 0,
        });
    }

    Ok(RawDatFileEntry {
        flags: read_u16_le(bytes, offset)?,
        version: read_u16_le(bytes, offset + 2)?,
        id: read_u32_le(bytes, offset + 4)?,
        offset: read_i32_le(bytes, offset + 8)?,
        size: read_u32_le(bytes, offset + 12)?,
        raw_date: read_u32_le(bytes, offset + 16)?,
        iteration: read_i32_le(bytes, offset + 20)?,
    })
}

fn read_u16_le(bytes: &[u8], offset: usize) -> DatResult<u16> {
    let slice = bytes.get(offset..offset + 2).ok_or_else(|| {
        DatError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "u16 read exceeded buffer",
        ))
    })?;
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> DatResult<u32> {
    let slice = bytes.get(offset..offset + 4).ok_or_else(|| {
        DatError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "u32 read exceeded buffer",
        ))
    })?;
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

fn read_i32_le(bytes: &[u8], offset: usize) -> DatResult<i32> {
    Ok(read_u32_le(bytes, offset)? as i32)
}
