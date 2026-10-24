use super::payloads::*;
use super::*;
use std::path::Path;
use tempfile::TempDir;

pub(super) struct DatFixture {
    temp: TempDir,
}

impl DatFixture {
    pub(super) fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let payload_one = b"portal-bytes".to_vec();
        let payload_two_raw = build_multi_block_payload();
        let payload_two_uncompressed_len = 6000usize;
        let payload_three_iteration = build_iteration_payload();
        write_fixture_file(
            &temp.path().join(DatFileKind::Portal.default_file_name()),
            1,
            &payload_one,
            &payload_two_raw,
            &payload_three_iteration,
            payload_two_uncompressed_len as u32,
        )?;
        write_fixture_file(
            &temp.path().join(DatFileKind::Cell.default_file_name()),
            2,
            &payload_one,
            &payload_two_raw,
            &payload_three_iteration,
            payload_two_uncompressed_len as u32,
        )?;
        write_fixture_file(
            &temp.path().join(DatFileKind::Local.default_file_name()),
            3,
            &payload_one,
            &payload_two_raw,
            &payload_three_iteration,
            payload_two_uncompressed_len as u32,
        )?;
        Ok(Self { temp })
    }

    pub(super) fn path(&self) -> &Path {
        self.temp.path()
    }
}

pub(super) struct TypedFixture {
    temp: TempDir,
}

impl TypedFixture {
    pub(super) fn new(compressed: bool) -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let (payload, packed) = build_generic_cache_payload(STRING_TABLE_DID);
        let portal_entries = vec![FixtureEntry {
            did: STRING_TABLE_DID,
            bytes: if compressed { packed } else { payload },
            flags: if compressed { 1 } else { 0 },
            version: 1,
            date_stamp: 0x5566_7788,
        }];
        let filler_entries = vec![FixtureEntry {
            did: FILE_ONE_ID,
            bytes: b"filler".to_vec(),
            flags: 0,
            version: 1,
            date_stamp: 0,
        }];

        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Portal.default_file_name()),
            1,
            MASTER_MAP_DID,
            &portal_entries,
        )?;
        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Cell.default_file_name()),
            2,
            MASTER_MAP_DID,
            &filler_entries,
        )?;
        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Local.default_file_name()),
            3,
            MASTER_MAP_DID,
            &filler_entries,
        )?;

        Ok(Self { temp })
    }

    pub(super) fn path(&self) -> &Path {
        self.temp.path()
    }
}

fn build_generic_cache_payload(did: u32) -> (Vec<u8>, Vec<u8>) {
    let mut payload = did.to_le_bytes().to_vec();
    payload.extend(std::iter::repeat_n(b'g', 1024));
    pack_payload(payload)
}

pub(super) struct MalformedTypedFixture {
    temp: TempDir,
}

impl MalformedTypedFixture {
    pub(super) fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let portal_entries = vec![FixtureEntry {
            did: STRING_TABLE_DID,
            bytes: vec![1, 2, 3],
            flags: 0,
            version: 1,
            date_stamp: 0,
        }];
        let filler_entries = vec![FixtureEntry {
            did: FILE_ONE_ID,
            bytes: b"filler".to_vec(),
            flags: 0,
            version: 1,
            date_stamp: 0,
        }];

        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Portal.default_file_name()),
            1,
            MASTER_MAP_DID,
            &portal_entries,
        )?;
        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Cell.default_file_name()),
            2,
            MASTER_MAP_DID,
            &filler_entries,
        )?;
        write_fixture_file_entries(
            &temp.path().join(DatFileKind::Local.default_file_name()),
            3,
            MASTER_MAP_DID,
            &filler_entries,
        )?;

        Ok(Self { temp })
    }

    pub(super) fn path(&self) -> &Path {
        self.temp.path()
    }
}

pub(super) struct FixtureEntry {
    pub(super) did: u32,
    pub(super) bytes: Vec<u8>,
    pub(super) flags: u16,
    pub(super) version: u16,
    pub(super) date_stamp: u32,
}
