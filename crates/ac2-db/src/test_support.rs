use std::path::Path;

use crate::ResourceStore;
use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};
use tempfile::TempDir;

const BLOCK_SIZE: usize = 2048;
const NODE_SIZE: usize = 1720;
const ROOT_BLOCK_OFFSET: usize = BLOCK_SIZE;
const MASTER_MAP_DID: u32 = 0x0E00_001F;
const FILLER_DID: u32 = 0x1000_0001;

pub(crate) struct FixtureEntry {
    pub(crate) did: u32,
    pub(crate) bytes: Vec<u8>,
    pub(crate) flags: u16,
    pub(crate) version: u16,
    pub(crate) date_stamp: u32,
}

#[derive(Clone, Copy)]
struct RawFileEntry {
    did: u32,
    block_offset: i32,
    size: u32,
    flags: u16,
    version: u16,
    date_stamp: u32,
}

pub(crate) fn store_from_options(options: DatOpenOptions) -> ac2_dat::DatResult<ResourceStore> {
    Ok(ResourceStore::new(DatDataset::open(options)?))
}

pub(crate) struct SingleRecordFixture {
    temp: TempDir,
}

impl SingleRecordFixture {
    pub(crate) fn new(entry: FixtureEntry) -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let filler = FixtureEntry {
            did: FILLER_DID,
            bytes: b"filler".to_vec(),
            flags: 0,
            version: 1,
            date_stamp: 0,
        };

        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Portal.default_file_name()),
            1,
            MASTER_MAP_DID,
            &[entry],
        )?;
        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Cell.default_file_name()),
            2,
            MASTER_MAP_DID,
            &[FixtureEntry { ..filler.clone() }],
        )?;
        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Local.default_file_name()),
            3,
            MASTER_MAP_DID,
            &[filler],
        )?;
        Ok(Self { temp })
    }

    pub(crate) fn store(&self) -> ac2_dat::DatResult<ResourceStore> {
        Ok(ResourceStore::new(DatDataset::open(DatOpenOptions::new(
            self.temp.path(),
        ))?))
    }
}

impl Clone for FixtureEntry {
    fn clone(&self) -> Self {
        Self {
            did: self.did,
            bytes: self.bytes.clone(),
            flags: self.flags,
            version: self.version,
            date_stamp: self.date_stamp,
        }
    }
}

fn write_fixture_file_entries(
    path: &Path,
    file_type: u32,
    master_map_did: u32,
    entries: &[FixtureEntry],
) -> std::io::Result<()> {
    let mut file_entries = Vec::new();
    let mut blocks = Vec::new();
    let mut next_block_offset = BLOCK_SIZE * 2;

    for entry in entries {
        let first_block_offset = next_block_offset as i32;
        let chunks: Vec<&[u8]> = entry.bytes.chunks(BLOCK_SIZE - 4).collect();
        for (index, chunk) in chunks.iter().enumerate() {
            let block_offset = next_block_offset;
            next_block_offset += BLOCK_SIZE;
            let next = if index + 1 < chunks.len() {
                next_block_offset as i32
            } else {
                0
            };
            blocks.push((block_offset, next, (*chunk).to_vec()));
        }
        file_entries.push(RawFileEntry {
            did: entry.did,
            block_offset: first_block_offset,
            size: entry.bytes.len() as u32,
            flags: entry.flags,
            version: entry.version,
            date_stamp: entry.date_stamp,
        });
    }

    let mut bytes = vec![0u8; next_block_offset];
    let file_size = bytes.len() as i32;
    write_header(&mut bytes, file_type, file_size, master_map_did);

    let mut node = vec![0u8; NODE_SIZE];
    write_i32(&mut node, 62 * 4, file_entries.len() as i32);
    for (index, entry) in file_entries.iter().enumerate() {
        write_file_entry(&mut node, 62 * 4 + 4 + index * 24, *entry);
    }

    write_block(&mut bytes, BLOCK_SIZE, 0, &node);
    for (offset, next, block_bytes) in blocks {
        write_block(&mut bytes, offset, next, &block_bytes);
    }

    std::fs::write(path, bytes)
}

fn write_header(bytes: &mut [u8], dataset_type: u32, file_size: i32, master_map_did: u32) {
    bytes[256..320].fill(0);
    bytes[256..260].copy_from_slice(&[0x00, 0x50, 0x4C, 0x00]);
    write_i32(bytes, 320, 0x0000_5442);
    write_i32(bytes, 324, BLOCK_SIZE as i32);
    write_i32(bytes, 328, file_size);
    write_u32(bytes, 332, dataset_type);
    write_u32(bytes, 336, 0xCAFE_BABE);
    write_i32(bytes, 340, 0);
    write_i32(bytes, 344, 0);
    write_i32(bytes, 348, 0);
    write_i32(bytes, 352, ROOT_BLOCK_OFFSET as i32);
    write_i32(bytes, 356, 0);
    write_i32(bytes, 360, 0);
    write_i32(bytes, 364, 0);
    write_u32(bytes, 368, master_map_did);
    write_i32(bytes, 372, 77);
    write_i32(bytes, 376, 88);
    bytes[380..396].fill(0xAB);
    write_u32(bytes, 396, 0xDEAD_BEEF);
}

fn write_file_entry(node: &mut [u8], offset: usize, entry: RawFileEntry) {
    write_u16(node, offset, entry.flags);
    write_u16(node, offset + 2, entry.version);
    write_u32(node, offset + 4, entry.did);
    write_i32(node, offset + 8, entry.block_offset);
    write_u32(node, offset + 12, entry.size);
    write_u32(node, offset + 16, entry.date_stamp);
    write_i32(node, offset + 20, 1);
}

fn write_block(bytes: &mut [u8], offset: usize, next_block: i32, payload: &[u8]) {
    write_i32(bytes, offset, next_block);
    let end = offset + 4 + payload.len();
    bytes[offset + 4..end].copy_from_slice(payload);
    if payload.len() < BLOCK_SIZE - 4 {
        bytes[end..offset + BLOCK_SIZE].fill(0);
    }
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
