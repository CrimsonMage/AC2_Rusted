use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions, DatReadAttribution};

use super::fixtures::{MalformedTypedFixture, TypedFixture};
use super::payloads::pack_payload;
use super::{RawTestRecord, RejectingTestRecord, ResourceStore, FIXTURE_TYPE_ID, STRING_TABLE_DID};
use crate::typed_cache::ResourceCacheConfig;
use crate::{
    test_support::{FixtureEntry, SingleRecordFixture},
    CachedResource, ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext,
    ResourceError, ResourceKey, ResourceResult, ResourceTypeId, TypedResourceRef,
};

fn typed_reference(key: ResourceKey) -> TypedResourceRef {
    TypedResourceRef::new(key.file_kind(), key.data_id(), key.type_id())
}

#[derive(Debug)]
struct ChargedBodyDecoder(Vec<u8>);

static SCOPED_BODY_DECODE_COUNT: AtomicUsize = AtomicUsize::new(0);
static MIXED_BODY_DECODE_COUNT: AtomicUsize = AtomicUsize::new(0);

macro_rules! counted_body_decoder {
    ($name:ident, $counter:ident) => {
        #[derive(Debug)]
        struct $name(Vec<u8>);

        impl ResourceBodyDecode for $name {
            const TYPE_NAME: &'static str = stringify!($name);
            const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

            fn decode_body(
                _context: ResourceDbObjDecodeContext,
                body: ResourceDbObjBody<'_>,
            ) -> ResourceResult<Self> {
                $counter.fetch_add(1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(40));
                Ok(Self(body.bytes().to_vec()))
            }

            fn cache_retained_bytes(&self) -> usize {
                self.0.len()
            }
        }
    };
}

counted_body_decoder!(ScopedBodyDecoder, SCOPED_BODY_DECODE_COUNT);
counted_body_decoder!(MixedBodyDecoder, MIXED_BODY_DECODE_COUNT);

impl ResourceBodyDecode for ChargedBodyDecoder {
    const TYPE_NAME: &'static str = "charged body test";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body(
        _context: ResourceDbObjDecodeContext,
        body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        Ok(Self(body.bytes().to_vec()))
    }

    fn cache_retained_bytes(&self) -> usize {
        97usize.saturating_add(self.0.len())
    }
}

#[test]
fn cached_body_uses_only_the_decoder_exclusive_retained_charge() {
    const DID: u32 = 0x2200_0A01;
    let mut bytes = DID.to_le_bytes().to_vec();
    bytes.extend_from_slice(&[0xAB; 256]);
    let fixture = SingleRecordFixture::new(FixtureEntry {
        did: DID,
        bytes,
        flags: 0,
        version: 1,
        date_stamp: 0,
    })
    .expect("fixture should build");
    let store = fixture.store().expect("store should open");
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(DID),
        ResourceTypeId::FixtureA,
    );

    let first = store
        .load_cached_body::<ChargedBodyDecoder>(reference)
        .expect("body decode should succeed")
        .expect("record should exist");
    let second = store
        .load_cached_body::<ChargedBodyDecoder>(reference)
        .expect("cached body decode should succeed")
        .expect("record should exist");

    assert!(CachedResource::ptr_eq(&first, &second));
    assert_eq!(store.cache_stats().retained_bytes(), 97 + 256);
    assert_ne!(store.cache_stats().retained_bytes(), 260);
    drop(first);
    drop(second);
    assert!(store.evict_cached_typed::<ChargedBodyDecoder>(reference.resource_key()));
    assert_eq!(store.cache_stats().entry_count(), 0);
}

#[test]
fn ownership_observation_separates_generic_and_scoped_entries_by_type() {
    const DID: u32 = 0x2200_0A02;
    let mut bytes = DID.to_le_bytes().to_vec();
    bytes.extend_from_slice(&[0xAB; 64]);
    let fixture = SingleRecordFixture::new(FixtureEntry {
        did: DID,
        bytes,
        flags: 0,
        version: 1,
        date_stamp: 0,
    })
    .expect("fixture should build");
    let store = fixture.store().expect("store should open");
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(DID),
        ResourceTypeId::FixtureA,
    );
    let scope = store.begin_decode_scope();

    store
        .load_cached_body::<ChargedBodyDecoder>(reference)
        .expect("generic decode should succeed");
    store
        .load_scoped_body::<ChargedBodyDecoder>(&scope, reference)
        .expect("scoped decode should succeed");

    let generic = store.ownership_stats();
    assert_eq!(generic.entry_count(), 1);
    assert_eq!(generic.retained_bytes(), 97 + 64);
    assert_eq!(generic.by_resource_type().len(), 1);
    assert_eq!(
        generic.by_resource_type()[0].resource_type_id(),
        ResourceTypeId::FixtureA as u32
    );
    assert_eq!(generic.by_resource_type()[0].entry_count(), 1);
    assert_eq!(store.try_ownership_stats(), Some(generic.clone()));

    let scoped = scope.ownership_stats();
    assert_eq!(scoped.entry_count(), 1);
    assert_eq!(scoped.retained_bytes(), 97 + 64);
    assert_eq!(scoped.by_resource_type().len(), 1);
    assert_eq!(
        scoped.by_resource_type()[0].resource_type_id(),
        ResourceTypeId::FixtureA as u32
    );
    assert_eq!(scope.try_ownership_stats(), Some(scoped));

    let overlap = store
        .try_scope_overlap_stats(&scope)
        .expect("uncontended ownership overlap observation should succeed");
    assert_eq!(overlap.entry_count(), 1);
    assert_eq!(
        overlap.shared_value_entry_count() + overlap.distinct_value_entry_count(),
        1
    );
    assert_eq!(overlap.generic_retained_bytes(), 97 + 64);
    assert_eq!(overlap.scoped_retained_bytes(), 97 + 64);
}

#[test]
fn cached_typed_load_reuses_arc_identity() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );

    let first = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");
    assert_eq!(store.cache_stats().entry_count(), 1);
    assert_eq!(store.cache_stats().miss_count(), 1);

    let second = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");

    assert!(CachedResource::ptr_eq(&first, &second));
    assert_eq!(store.cache_stats().entry_count(), 1);
    assert_eq!(store.cache_stats().hit_count(), 1);
}

#[test]
fn ephemeral_body_load_does_not_create_a_generic_cache_owner() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );

    let first = store
        .load_ephemeral_body::<RawTestRecord>(typed_reference(key))
        .expect("ephemeral load should succeed")
        .expect("table should exist");
    let second = store
        .load_ephemeral_body::<RawTestRecord>(typed_reference(key))
        .expect("ephemeral reload should succeed")
        .expect("table should exist");

    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(store.cache_stats().entry_count(), 0);
    assert_eq!(store.cache_stats().retained_bytes(), 0);
}

#[test]
fn scoped_body_reuses_only_its_active_scope_and_never_warms_the_generic_cache() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );
    let first_scope = store.begin_decode_scope();
    let first = store
        .load_scoped_body::<RawTestRecord>(&first_scope, typed_reference(key))
        .expect("scoped load should succeed")
        .expect("table should exist");
    let second = store
        .load_scoped_body::<RawTestRecord>(&first_scope, typed_reference(key))
        .expect("scoped reload should succeed")
        .expect("table should exist");

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first_scope.resource_count(), 1);
    assert_eq!(store.cache_stats().entry_count(), 0);
    drop(first);
    drop(second);
    drop(first_scope);

    let next_scope = store.begin_decode_scope();
    let reloaded = store
        .load_scoped_body::<RawTestRecord>(&next_scope, typed_reference(key))
        .expect("next scoped load should succeed")
        .expect("table should exist");
    assert_eq!(next_scope.resource_count(), 1);
    assert_eq!(store.cache_stats().entry_count(), 0);
    assert_ne!(reloaded.0, Vec::<u8>::new());
}

#[test]
fn concurrent_scoped_body_loads_join_one_transient_typed_decode() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = Arc::new(
        crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
            .expect("store should open"),
    );
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );
    let scope = store.begin_decode_scope();
    let barrier = Arc::new(Barrier::new(2));
    SCOPED_BODY_DECODE_COUNT.store(0, Ordering::SeqCst);
    let mut joins = Vec::new();
    for _ in 0..2 {
        let store = Arc::clone(&store);
        let scope = scope.clone();
        let barrier = Arc::clone(&barrier);
        joins.push(thread::spawn(move || {
            barrier.wait();
            store
                .load_scoped_body::<ScopedBodyDecoder>(&scope, typed_reference(key))
                .expect("scoped load should succeed")
                .expect("table should exist")
        }));
    }
    let first = joins
        .remove(0)
        .join()
        .expect("first scoped worker should finish");
    let second = joins
        .remove(0)
        .join()
        .expect("second scoped worker should finish");

    assert_eq!(SCOPED_BODY_DECODE_COUNT.load(Ordering::SeqCst), 1);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(scope.resource_count(), 1);
    assert_eq!(store.cache_stats().entry_count(), 0);
}

#[test]
fn scoped_and_generic_body_loads_share_one_transient_typed_decode() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = Arc::new(
        crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
            .expect("store should open"),
    );
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );
    let scope = store.begin_decode_scope();
    let barrier = Arc::new(Barrier::new(2));
    MIXED_BODY_DECODE_COUNT.store(0, Ordering::SeqCst);

    let scoped_store = Arc::clone(&store);
    let scoped_scope = scope.clone();
    let scoped_barrier = Arc::clone(&barrier);
    let scoped = thread::spawn(move || {
        scoped_barrier.wait();
        scoped_store
            .load_scoped_body::<MixedBodyDecoder>(&scoped_scope, typed_reference(key))
            .expect("scoped load should succeed")
            .expect("table should exist")
    });
    let generic_store = Arc::clone(&store);
    let generic = thread::spawn(move || {
        barrier.wait();
        generic_store
            .load_cached_body::<MixedBodyDecoder>(typed_reference(key))
            .expect("generic load should succeed")
            .expect("table should exist")
    });

    let scoped = scoped.join().expect("scoped worker should finish");
    let generic = generic.join().expect("generic worker should finish");
    assert_eq!(MIXED_BODY_DECODE_COUNT.load(Ordering::SeqCst), 1);
    assert!(std::ptr::eq(scoped.as_ref(), generic.as_ref()));
    assert_eq!(scope.resource_count(), 1);
    assert_eq!(store.cache_stats().entry_count(), 1);
}

#[test]
fn cached_compressed_resource_is_budgeted_by_unpacked_record_bytes() {
    let fixture = TypedFixture::new(true).expect("compressed fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );
    let packed_len = store.entry(key).expect("record should exist").packed_len as usize;
    let unpacked_len = store
        .read_record_for_decode(
            DatFileKind::Portal,
            DataId::new(STRING_TABLE_DID),
            DatReadAttribution::Unattributed,
        )
        .expect("record should decode")
        .expect("record should exist")
        .as_slice()
        .len();
    assert!(unpacked_len > packed_len);

    let _ = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");

    assert_eq!(
        store.cache_stats().retained_bytes(),
        unpacked_len - std::mem::size_of::<u32>()
    );
}

#[test]
fn cached_typed_evict_removes_only_matching_type_entry() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );

    let first = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");

    assert!(!store.evict_cached_typed::<ChargedBodyDecoder>(key));
    drop(first);
    assert!(store.evict_cached_typed::<RawTestRecord>(key));

    let second = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");

    assert_eq!(store.cache_stats().entry_count(), 1);
    drop(second);
}

#[test]
fn cache_misses_and_decode_failures_do_not_consume_capacity() {
    let missing_fixture = TypedFixture::new(false).expect("fixture should build");
    let missing_store = ResourceStore::with_cache_config(
        DatDataset::open(DatOpenOptions::new(missing_fixture.path())).expect("dataset should open"),
        ResourceCacheConfig::default(),
    );
    let missing_key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(0x2200_0FFF), FIXTURE_TYPE_ID),
    );

    assert!(missing_store
        .load_cached_body::<RawTestRecord>(typed_reference(missing_key))
        .expect("missing cache lookup should succeed")
        .is_none());
    assert_eq!(missing_store.cache_stats().entry_count(), 0);
    assert_eq!(missing_store.cache_stats().miss_count(), 1);

    let malformed_fixture = MalformedTypedFixture::new().expect("fixture should build");
    let malformed_store = ResourceStore::with_cache_config(
        DatDataset::open(DatOpenOptions::new(malformed_fixture.path()))
            .expect("dataset should open"),
        ResourceCacheConfig::default(),
    );
    let malformed_key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );

    let error = malformed_store
        .load_cached_body::<RejectingTestRecord>(typed_reference(malformed_key))
        .expect_err("malformed decode should fail");
    assert!(matches!(
        error,
        crate::ResourceError::MalformedTypedObject {
            type_name: "rejecting cache test record",
            ..
        }
    ));
    assert_eq!(malformed_store.cache_stats().entry_count(), 0);
    assert_eq!(malformed_store.cache_stats().miss_count(), 1);
}

#[test]
fn clear_typed_cache_resets_entries_and_stats() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = ResourceStore::with_cache_config(
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open"),
        ResourceCacheConfig::default(),
    );
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    );

    let first = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");
    let _ = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached reload should succeed")
        .expect("table should exist");
    assert_eq!(store.cache_stats().entry_count(), 1);
    assert_eq!(store.cache_stats().hit_count(), 1);

    drop(first);
    store.clear_typed_cache();

    assert_eq!(store.cache_stats().entry_count(), 0);
    assert_eq!(store.cache_stats().hit_count(), 0);
    assert_eq!(store.cache_stats().miss_count(), 0);
    assert_eq!(store.cache_stats().eviction_count(), 0);

    let second = store
        .load_cached_body::<RawTestRecord>(typed_reference(key))
        .expect("cached load should succeed")
        .expect("table should exist");
    assert!(!second.0.is_empty());
}

#[test]
fn compressed_reads_reject_missing_length_prefix() {
    let store = single_record_store(FixtureEntry {
        did: STRING_TABLE_DID,
        bytes: vec![1, 2, 3],
        flags: 1,
        version: 1,
        date_stamp: 0,
    });

    let error = store
        .read_record_for_decode(
            DatFileKind::Portal,
            DataId::new(STRING_TABLE_DID),
            DatReadAttribution::Unattributed,
        )
        .expect_err("compressed read should fail");
    assert!(matches!(error, ResourceError::DecompressionIo { .. }));
}

#[test]
fn compressed_reads_reject_length_mismatches() {
    let (_, mut packed) = pack_payload(vec![1, 2, 3]);
    packed[..4].copy_from_slice(&4u32.to_le_bytes());
    let store = single_record_store(FixtureEntry {
        did: STRING_TABLE_DID,
        bytes: packed,
        flags: 1,
        version: 1,
        date_stamp: 0,
    });

    let error = store
        .read_record_for_decode(
            DatFileKind::Portal,
            DataId::new(STRING_TABLE_DID),
            DatReadAttribution::Unattributed,
        )
        .expect_err("compressed read should fail");
    assert!(matches!(
        error,
        ResourceError::DecompressionLengthMismatch {
            expected_len: 4,
            actual_len: 3,
            ..
        }
    ));
}

#[test]
fn compressed_reads_reject_invalid_zlib_payloads() {
    let mut packed = 4u32.to_le_bytes().to_vec();
    packed.extend_from_slice(&[0xFF, 0x00, 0xAA, 0x55]);
    let store = single_record_store(FixtureEntry {
        did: STRING_TABLE_DID,
        bytes: packed,
        flags: 1,
        version: 1,
        date_stamp: 0,
    });

    let error = store
        .read_record_for_decode(
            DatFileKind::Portal,
            DataId::new(STRING_TABLE_DID),
            DatReadAttribution::Unattributed,
        )
        .expect_err("compressed read should fail");
    assert!(matches!(error, ResourceError::DecompressionIo { .. }));
}

#[test]
fn typed_load_lanes_resolve_once_and_report_their_dat_attribution() {
    let fixture_entry = || FixtureEntry {
        did: STRING_TABLE_DID,
        bytes: STRING_TABLE_DID.to_le_bytes().to_vec(),
        flags: 0,
        version: 1,
        date_stamp: 0,
    };
    let reference = typed_reference(ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(STRING_TABLE_DID), FIXTURE_TYPE_ID),
    ));

    let generic = single_record_store(fixture_entry());
    generic
        .load_cached_body::<RawTestRecord>(reference)
        .expect("generic decode should succeed");
    let generic_summary = generic
        .dataset
        .read_telemetry(DatFileKind::Portal)
        .expect("fixture Portal DAT should be mounted");
    assert_eq!(generic_summary.attribution.generic_resource_requests, 1);
    assert_eq!(generic_summary.total_read_requests, 1);

    let scoped = single_record_store(fixture_entry());
    let scope = scoped.begin_decode_scope();
    scoped
        .load_scoped_body::<RawTestRecord>(&scope, reference)
        .expect("scoped decode should succeed");
    let scoped_summary = scoped
        .dataset
        .read_telemetry(DatFileKind::Portal)
        .expect("fixture Portal DAT should be mounted");
    assert_eq!(scoped_summary.attribution.active_world_scoped_requests, 1);
    assert_eq!(scoped_summary.total_read_requests, 1);

    let ephemeral = single_record_store(fixture_entry());
    ephemeral
        .load_ephemeral_body::<RawTestRecord>(reference)
        .expect("ephemeral decode should succeed");
    let ephemeral_summary = ephemeral
        .dataset
        .read_telemetry(DatFileKind::Portal)
        .expect("fixture Portal DAT should be mounted");
    assert_eq!(
        ephemeral_summary
            .attribution
            .ephemeral_world_tooling_requests,
        1
    );
    assert_eq!(ephemeral_summary.total_read_requests, 1);
}

#[test]
fn generic_residency_lease_pins_only_cached_typed_references() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = ResourceStore::with_cache_config(
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open"),
        ResourceCacheConfig::default(),
    );
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );
    let missing_reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(0x2200_0FFF),
        ResourceTypeId::FixtureA,
    );

    assert!(store
        .load_cached_body::<RawTestRecord>(reference)
        .expect("cached load should succeed")
        .is_some());
    let stats_before_peek = store.cache_stats();
    assert!(store.peek_cached::<RawTestRecord>(reference).is_some());
    assert_eq!(store.cache_stats(), stats_before_peek);
    assert!(
        store
            .peek_cached::<RawTestRecord>(missing_reference)
            .is_none(),
        "non-refreshing residency observation must not decode a missing reference"
    );

    let lease = store.retain_cached_resources(&[reference, missing_reference]);
    assert_eq!(lease.resource_count(), 1);
    assert!(!lease.is_empty());
    assert!(
        !store.evict_cached_typed::<RawTestRecord>(reference.resource_key()),
        "a generic residency lease must prevent eviction without decoding the missing reference"
    );

    drop(lease);
    assert!(store.evict_cached_typed::<RawTestRecord>(reference.resource_key()));
    assert!(
        store.peek_cached::<RawTestRecord>(reference).is_none(),
        "residency observation must not pin an entry against eviction"
    );
}

fn single_record_store(entry: FixtureEntry) -> ResourceStore {
    let fixture = SingleRecordFixture::new(entry).expect("fixture should build");
    fixture.store().expect("store should open")
}
