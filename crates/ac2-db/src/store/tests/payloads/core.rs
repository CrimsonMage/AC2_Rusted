use std::io::Write;
use std::path::PathBuf;

use ac2_dat::RETAIL_DDD_ITERATION_DID;
use flate2::{write::ZlibEncoder, Compression};

use super::*;

#[derive(Clone, Copy)]
struct RawFileEntry {
    did: u32,
    block_offset: i32,
    size: u32,
    flags: u16,
    version: u16,
    date_stamp: u32,
}

pub(crate) fn build_multi_block_payload() -> Vec<u8> {
    (0..3000).map(|value| (value % 251) as u8).collect()
}

pub(crate) fn build_iteration_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&5i32.to_le_bytes());
    payload.extend_from_slice(&(-3i32).to_le_bytes());
    payload.extend_from_slice(&1i32.to_le_bytes());
    payload.extend_from_slice(&(-2i32).to_le_bytes());
    payload.extend_from_slice(&4i32.to_le_bytes());
    payload
}

pub(crate) fn pack_payload(payload: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&payload).unwrap();
    let compressed = encoder.finish().unwrap();

    let mut packed = (payload.len() as u32).to_le_bytes().to_vec();
    packed.extend_from_slice(&compressed);
    (payload, packed)
}

pub(crate) fn write_fixture_file(
    path: &PathBuf,
    file_type: u32,
    payload_one: &[u8],
    payload_two: &[u8],
    payload_three: &[u8],
    payload_two_uncompressed_len: u32,
) -> std::io::Result<()> {
    let mut bytes = vec![0u8; BLOCK_SIZE * 6];
    let file_size = bytes.len() as i32;
    write_header(&mut bytes, file_type, file_size, 0x1234_5678);
    let mut packed_two = payload_two_uncompressed_len.to_le_bytes().to_vec();
    packed_two.extend_from_slice(payload_two);

    let mut node = vec![0u8; NODE_SIZE];
    write_i32(&mut node, 62 * 4, 3);
    write_file_entry(
        &mut node,
        62 * 4 + 4,
        RawFileEntry {
            did: FILE_ONE_ID,
            block_offset: PAYLOAD_ONE_BLOCK_OFFSET as i32,
            size: payload_one.len() as u32,
            flags: 0,
            version: 1,
            date_stamp: 0x0102_0304,
        },
    );
    write_file_entry(
        &mut node,
        62 * 4 + 28,
        RawFileEntry {
            did: FILE_TWO_ID,
            block_offset: PAYLOAD_TWO_FIRST_BLOCK_OFFSET as i32,
            size: packed_two.len() as u32,
            flags: 1,
            version: 7,
            date_stamp: 0x0506_0708,
        },
    );
    write_file_entry(
        &mut node,
        62 * 4 + 52,
        RawFileEntry {
            did: RETAIL_DDD_ITERATION_DID,
            block_offset: PAYLOAD_THREE_BLOCK_OFFSET as i32,
            size: payload_three.len() as u32,
            flags: 0,
            version: 0,
            date_stamp: 0,
        },
    );

    write_block(&mut bytes, ROOT_BLOCK_OFFSET, 0, &node);
    write_block(&mut bytes, PAYLOAD_ONE_BLOCK_OFFSET, 0, payload_one);
    write_block(
        &mut bytes,
        PAYLOAD_TWO_FIRST_BLOCK_OFFSET,
        PAYLOAD_TWO_SECOND_BLOCK_OFFSET as i32,
        &packed_two[..BLOCK_SIZE - 4],
    );
    write_block(
        &mut bytes,
        PAYLOAD_TWO_SECOND_BLOCK_OFFSET,
        0,
        &packed_two[BLOCK_SIZE - 4..],
    );
    write_block(&mut bytes, PAYLOAD_THREE_BLOCK_OFFSET, 0, payload_three);

    std::fs::write(path, bytes)
}

pub(crate) fn write_fixture_file_entries(
    path: &PathBuf,
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

    write_block(&mut bytes, ROOT_BLOCK_OFFSET, 0, &node);
    for (block_offset, next_offset, payload) in blocks {
        write_block(&mut bytes, block_offset, next_offset, &payload);
    }

    std::fs::write(path, bytes)
}

pub(crate) fn write_header(bytes: &mut [u8], file_type: u32, file_size: i32, master_map_did: u32) {
    bytes[256..320].fill(0);
    bytes[256..260].copy_from_slice(&[0x00, 0x50, 0x4C, 0x00]);
    write_i32(bytes, 320, 0x0000_5442);
    write_i32(bytes, 324, BLOCK_SIZE as i32);
    write_i32(bytes, 328, file_size);
    write_u32(bytes, 332, file_type);
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

fn write_file_entry(bytes: &mut [u8], offset: usize, entry: RawFileEntry) {
    write_u16(bytes, offset, entry.flags);
    write_u16(bytes, offset + 2, entry.version);
    write_u32(bytes, offset + 4, entry.did);
    write_i32(bytes, offset + 8, entry.block_offset);
    write_u32(bytes, offset + 12, entry.size);
    write_u32(bytes, offset + 16, entry.date_stamp);
    write_i32(bytes, offset + 20, 1);
}

pub(crate) fn write_block(bytes: &mut [u8], offset: usize, next_block: i32, payload: &[u8]) {
    write_i32(bytes, offset, next_block);
    let limit = BLOCK_SIZE - 4;
    let end = offset + 4 + payload.len();
    bytes[offset + 4..end].copy_from_slice(payload);
    if payload.len() < limit {
        bytes[end..offset + BLOCK_SIZE].fill(0);
    }
}

pub(crate) fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
