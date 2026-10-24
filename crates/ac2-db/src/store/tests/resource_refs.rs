use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};

use super::fixtures::TypedFixture;
use super::{RawTestRecord, FIXTURE_TYPE_ID, STRING_TABLE_DID};
use crate::{
    InlinePreparationExecutor, ResourcePriority, ResourceRuntime, ResourceStore, TypedResourceRef,
};

#[test]
fn loads_typed_resources_through_typed_refs() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = crate::test_support::store_from_options(DatOpenOptions::new(fixture.path()))
        .expect("store should open");
    let reference = TypedResourceRef::new(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        FIXTURE_TYPE_ID,
    );

    let record = store
        .load_cached_body::<RawTestRecord>(reference)
        .expect("typed-ref body load should succeed")
        .expect("typed record should exist");

    assert!(!record.0.is_empty());
}

#[test]
fn caches_typed_refs_and_warms_them_through_runtime_prefetch() {
    let fixture = TypedFixture::new(false).expect("fixture should build");
    let store = Arc::new(ResourceStore::new(
        DatDataset::open(DatOpenOptions::new(fixture.path())).expect("dataset should open"),
    ));
    let runtime = ResourceRuntime::with_preparation_executor(
        store.clone(),
        Arc::new(InlinePreparationExecutor::default()),
    );
    let reference = TypedResourceRef::new(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        FIXTURE_TYPE_ID,
    );

    let context = runtime.begin_request_context(ResourcePriority::Interactive);
    assert!(runtime.queue_prefetch_typed_for_context::<RawTestRecord>(context, reference));
    let _ = runtime.pump_resource_events(8);
    assert_eq!(store.cache_stats().entry_count(), 1);
    assert_eq!(store.cache_stats().miss_count(), 1);

    let cached = store
        .load_cached_body::<RawTestRecord>(reference)
        .expect("cached typed-ref body load should succeed")
        .expect("cached table should exist");
    assert!(!cached.0.is_empty());
    assert_eq!(store.cache_stats().hit_count(), 1);
}
