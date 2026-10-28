use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{DatFileKind, DatReadAttribution};

use super::payloads::pack_payload;
use super::ResourceStore;
use crate::{
    test_support::{FixtureEntry, SingleRecordFixture},
    CachedResource, ResourceBodyDecode, ResourceBodyDecodeWith, ResourceDbObjBody,
    ResourceDbObjDecodeContext, ResourcePackedDecode, ResourceResult, ResourceTypeId,
    TypedResourceRef,
};

#[derive(Debug, PartialEq, Eq)]
struct EchoBody(Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
struct PackedEcho(Vec<u8>);

impl ResourceBodyDecode for EchoBody {
    const TYPE_NAME: &'static str = "decode-input echo body";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body(
        _context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        Ok(Self(body.bytes().to_vec()))
    }

    fn cache_retained_bytes(&self) -> usize {
        self.0.len()
    }
}

impl ResourceBodyDecodeWith for EchoBody {
    type DecodeState = AtomicUsize;

    const TYPE_NAME: &'static str = "stateful decode-input echo body";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body_with(
        decode_count: &Self::DecodeState,
        context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        decode_count.fetch_add(1, Ordering::Relaxed);
        assert_eq!(context.dbobj_pack_version(), 3);
        Ok(Self(body.bytes().to_vec()))
    }

    fn cache_retained_bytes(&self) -> usize {
        self.0.len()
    }
}

impl ResourcePackedDecode for PackedEcho {
    const TYPE_NAME: &'static str = "packed decode-input echo";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_packed(
        qdid: QualifiedDataId,
        pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        assert_eq!(qdid.data_id(), DataId::new(0x2200_0B05));
        assert_eq!(pack_version, 7);
        Ok(Self(bytes.to_vec()))
    }

    fn cache_retained_bytes(&self) -> usize {
        self.0.len()
    }
}

#[test]
fn typed_decode_inputs_preserve_bytes_from_each_dat_storage_layout() {
    let contiguous = body_payload(0x2200_0B01, 32);
    let segmented = body_payload(0x2200_0B02, 3_000);
    let (decompressed, packed) = pack_payload(body_payload(0x2200_0B03, 48));

    for (did, stored, flags, expected) in [
        (0x2200_0B01, contiguous.clone(), 0, contiguous),
        (0x2200_0B02, segmented.clone(), 0, segmented),
        (0x2200_0B03, packed, 1, decompressed),
    ] {
        let store = store_for(FixtureEntry {
            did,
            bytes: stored,
            flags,
            version: 1,
            date_stamp: 0,
        });
        let input = store
            .read_record_for_decode(
                DatFileKind::Portal,
                DataId::new(did),
                DatReadAttribution::Unattributed,
            )
            .expect("decode input read should succeed")
            .expect("record should exist");
        assert_eq!(input.as_slice(), expected);
        drop(input);

        let reference = body_reference(did);
        let decoded = store
            .load_cached_body::<EchoBody>(reference)
            .expect("body decode should succeed")
            .expect("body should exist");
        assert_eq!(decoded.0, expected[4..]);
    }
}

#[test]
fn stateful_body_decoder_preserves_typed_cache_identity() {
    let did = 0x2200_0B04;
    let store = store_for(FixtureEntry {
        did,
        bytes: body_payload(did, 8),
        flags: 0,
        version: 3,
        date_stamp: 0,
    });
    let decode_count = Arc::new(AtomicUsize::new(0));
    let first = store
        .load_cached_body_with::<EchoBody>(body_reference(did), decode_count.as_ref())
        .expect("stateful body decode should succeed")
        .expect("record should exist");
    let second = store
        .load_cached_body_with::<EchoBody>(body_reference(did), decode_count.as_ref())
        .expect("cached stateful body load should succeed")
        .expect("cached record should exist");

    assert!(CachedResource::ptr_eq(&first, &second));
    assert_eq!(decode_count.load(Ordering::Relaxed), 1);
}

#[test]
fn stateful_body_load_still_validates_the_requested_resource_type() {
    let did = 0x2200_0B06;
    let store = store_for(FixtureEntry {
        did,
        bytes: body_payload(did, 8),
        flags: 0,
        version: 3,
        date_stamp: 0,
    });

    let decode_count = AtomicUsize::new(0);
    store
        .load_cached_body_with::<EchoBody>(body_reference(did), &decode_count)
        .expect("initial body decode should succeed")
        .expect("record should exist");

    let wrong_reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(did),
        ResourceTypeId::FixtureD,
    );
    let error = store
        .load_cached_body_with::<EchoBody>(wrong_reference, &decode_count)
        .expect_err("a cache hit must not bypass type validation");

    assert!(matches!(
        error,
        crate::ResourceError::MalformedTypedObject {
            context: "qualified data id type did not match the requested decoder",
            ..
        }
    ));
}

#[test]
fn packed_decoder_receives_record_bytes_without_a_dbobj_header() {
    let did = 0x2200_0B05;
    let packed_bytes = vec![0xF0, 0x0D, 0xCA, 0xFE];
    let store = store_for(FixtureEntry {
        did,
        bytes: packed_bytes.clone(),
        flags: 0,
        version: 7,
        date_stamp: 0,
    });

    let decoded = store
        .load_cached_packed::<PackedEcho>(body_reference(did))
        .expect("packed decode should succeed")
        .expect("packed record should exist");

    assert_eq!(decoded.0, packed_bytes);
}

/// Counts its decodes, slowly enough that concurrent first gets overlap.
#[derive(Debug)]
struct SlowPackedCount;

static SLOW_PACKED_DECODES: AtomicUsize = AtomicUsize::new(0);

impl ResourcePackedDecode for SlowPackedCount {
    const TYPE_NAME: &'static str = "slow counted packed decode";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_packed(_: QualifiedDataId, _: u16, _: &[u8]) -> ResourceResult<Self> {
        SLOW_PACKED_DECODES.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(50));
        Ok(Self)
    }

    fn cache_retained_bytes(&self) -> usize {
        1
    }
}

/// One `Arc` per DID with in-flight dedupe: loader threads asking for the same packed record at
/// once read and decode it once and share the result.
#[test]
fn concurrent_packed_loads_join_one_read_and_decode() {
    let did = 0x2200_0B07;
    let store = Arc::new(store_for(FixtureEntry {
        did,
        bytes: vec![1, 2, 3, 4],
        flags: 0,
        version: 1,
        date_stamp: 0,
    }));
    let barrier = Arc::new(std::sync::Barrier::new(4));
    let loads: Vec<_> = (0..4)
        .map(|_| {
            let (store, barrier) = (Arc::clone(&store), Arc::clone(&barrier));
            std::thread::spawn(move || {
                barrier.wait();
                store
                    .load_cached_packed::<SlowPackedCount>(body_reference(did))
                    .expect("packed decode should succeed")
                    .expect("packed record should exist")
            })
        })
        .collect();
    let loaded: Vec<_> = loads
        .into_iter()
        .map(|load| load.join().expect("loader finishes"))
        .collect();
    assert_eq!(SLOW_PACKED_DECODES.load(Ordering::SeqCst), 1);
    assert!(loaded
        .iter()
        .all(|value| CachedResource::ptr_eq(value, &loaded[0])));
    let reads = store
        .dataset()
        .read_telemetry(DatFileKind::Portal)
        .expect("portal DAT")
        .total_read_requests;
    assert_eq!(reads, 1, "one DAT read for four concurrent gets");
}

fn body_payload(did: u32, body_len: usize) -> Vec<u8> {
    let mut payload = did.to_le_bytes().to_vec();
    payload.extend((0..body_len).map(|index| index as u8));
    payload
}

fn body_reference(did: u32) -> TypedResourceRef {
    TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(did),
        ResourceTypeId::FixtureA,
    )
}

fn store_for(entry: FixtureEntry) -> ResourceStore {
    let fixture = SingleRecordFixture::new(entry).expect("fixture should build");
    fixture.store().expect("store should open")
}
