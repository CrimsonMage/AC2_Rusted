use super::ResourceStore;
use crate::{
    ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext, ResourceError, ResourceKey,
    ResourceResult, ResourceTypeId,
};
use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{DatFileKind, DatOpenOptions};

mod cache;
mod decode_input;
mod fixtures;
mod payloads;
mod resource_refs;

use fixtures::*;

const BLOCK_SIZE: usize = 2048;
const ROOT_BLOCK_OFFSET: usize = BLOCK_SIZE;
const PAYLOAD_ONE_BLOCK_OFFSET: usize = BLOCK_SIZE * 2;
const PAYLOAD_TWO_FIRST_BLOCK_OFFSET: usize = BLOCK_SIZE * 3;
const PAYLOAD_TWO_SECOND_BLOCK_OFFSET: usize = BLOCK_SIZE * 4;
const PAYLOAD_THREE_BLOCK_OFFSET: usize = BLOCK_SIZE * 5;
const NODE_SIZE: usize = 1720;

const FILE_ONE_ID: u32 = 0x1000_0001;
const FILE_TWO_ID: u32 = 0x1000_0002;

const MASTER_MAP_DID: u32 = 0x0E00_001F;
const STRING_TABLE_DID: u32 = 0x2200_0001;
const FIXTURE_TYPE_ID: u32 = ResourceTypeId::FixtureA as u32;

#[derive(Debug)]
struct RawTestRecord(Vec<u8>);

impl ResourceBodyDecode for RawTestRecord {
    const TYPE_NAME: &'static str = "generic cache test record";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body(
        _context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        Ok(Self(body.bytes().to_vec()))
    }

    fn cache_retained_bytes(&self) -> usize {
        self.0.capacity()
    }
}

#[derive(Debug)]
struct RejectingTestRecord;

impl ResourceBodyDecode for RejectingTestRecord {
    const TYPE_NAME: &'static str = "rejecting cache test record";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body(
        context: ResourceDbObjDecodeContext,
        _body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        Err(ResourceError::MalformedTypedObject {
            qdid: context.qdid(),
            type_name: Self::TYPE_NAME,
            context: "intentional generic decode failure",
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        0
    }
}

#[test]
fn resource_store_validates_internal_entry_and_type_queries() {
    let fixture = DatFixture::new().expect("fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(FILE_ONE_ID), FIXTURE_TYPE_ID),
    );

    assert_eq!(key.resource_type(), Some(ResourceTypeId::FixtureA));

    let entry = store.entry(key).expect("entry should exist");
    assert_eq!(entry.did, FILE_ONE_ID);
    assert_eq!(entry.version, 1);
    assert_eq!(entry.date_stamp, 0x0102_0304);
}
