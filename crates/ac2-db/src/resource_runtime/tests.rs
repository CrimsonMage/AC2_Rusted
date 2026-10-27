use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{
    ResourcePriority, ResourceRequestOutcome, ResourceRuntime, ResourceRuntimeConfig,
    ResourceRuntimeEvent,
};
use crate::test_support::{FixtureEntry, SingleRecordFixture};
use crate::{
    CachedResource, InlinePreparationExecutor, PreparationExecutor, PreparationJob,
    PreparationSubmitError, ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext,
    ResourceDddInvalidation, ResourceKey, ResourceResult, ResourceStore, ResourceTypeId,
    TypedPrefetchSequence, TypedResourceRef,
};

const MASTER_MAP_DID: u32 = 0x0E00_001F;
const STRING_TABLE_DID: u32 = 0x2200_0001;

#[derive(Debug)]
struct RuntimeBodyDecoder;

impl ResourceBodyDecode for RuntimeBodyDecoder {
    const TYPE_NAME: &'static str = "runtime body test";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body(
        _context: ResourceDbObjDecodeContext,
        _body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        Ok(Self)
    }

    fn cache_retained_bytes(&self) -> usize {
        1
    }
}

#[test]
fn worker_executes_context_scoped_body_prefetch_requests() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );
    let key = reference.resource_key();

    let context = runtime.begin_request_context(ResourcePriority::Interactive);
    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));
    let events = pump_until_events(&runtime);
    assert!(events.iter().any(|event| matches!(
        event,
        ResourceRuntimeEvent::RequestCompleted {
            context: completed,
            key: completed_key,
            outcome: ResourceRequestOutcome::Warmed,
        } if *completed == context && *completed_key == key
    )));
}

#[test]
fn worker_prefetch_warms_typed_entries_through_completion_pump() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );

    let context = runtime.begin_request_context(ResourcePriority::Interactive);
    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));
    assert!(!runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));
    let events = pump_until_events(&runtime);
    assert!(events.iter().any(|event| matches!(
        event,
        ResourceRuntimeEvent::RequestCompleted {
            context: completed,
            outcome: ResourceRequestOutcome::Warmed,
            ..
        } if *completed == context
    )));
    assert_eq!(runtime.cache_stats().entry_count(), 1);
}

#[test]
fn scoped_prefetch_retains_only_the_supplied_decode_scope() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );
    let scope = runtime.store().begin_decode_scope();
    let context = runtime.begin_scoped_request_context(ResourcePriority::Critical, scope.clone());

    assert!(
        runtime.queue_scoped_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference)
    );
    let events = pump_until_events(&runtime);
    assert!(events.iter().any(|event| matches!(
        event,
        ResourceRuntimeEvent::RequestCompleted {
            context: completed,
            outcome: ResourceRequestOutcome::Warmed,
            ..
        } if *completed == context
    )));
    assert_eq!(scope.resource_count(), 1);
    assert_eq!(runtime.cache_stats().entry_count(), 0);
}

#[test]
fn request_context_delivers_pumped_completion_events() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );
    let context = runtime.begin_request_context(ResourcePriority::Interactive);

    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));

    let events = runtime.pump_resource_events(8);

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ResourceRuntimeEvent::RequestCompleted {
                context: completed,
                outcome: ResourceRequestOutcome::Warmed,
                ..
            } if *completed == context
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            ResourceRuntimeEvent::ContextCompleted { context: completed }
                if *completed == context
        )
    }));
    assert_eq!(runtime.request_context_progress(context).warmed_count(), 1);
}

#[test]
fn timed_completion_pump_has_no_independent_count_ceiling() {
    let runtime = inline_runtime(Arc::new(test_store(build_lookup_payload(DataId::new(
        MASTER_MAP_DID,
    )))));
    let context = runtime.begin_request_context(ResourcePriority::Interactive);

    for index in 0..40 {
        let reference = TypedResourceRef::from_resource_type(
            DatFileKind::Portal,
            DataId::new(0x2000_0100 + index),
            ResourceTypeId::FixtureA,
        );
        assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));
    }

    let events = runtime.pump_resource_events_for(Duration::from_secs(1));
    let completed = events
        .iter()
        .filter(|event| matches!(event, ResourceRuntimeEvent::RequestCompleted { .. }))
        .count();

    assert_eq!(completed, 40);
    assert!(events.iter().any(|event| matches!(
        event,
        ResourceRuntimeEvent::ContextCompleted { context: completed_context }
            if *completed_context == context
    )));
}

#[test]
fn released_context_does_not_receive_stale_completion_events() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));
    let reference = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );
    let context = runtime.begin_request_context(ResourcePriority::Interactive);

    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));
    runtime.release_request_context(context);

    let events = runtime.pump_resource_events(8);

    assert!(events.iter().all(|event| {
        !matches!(
            event,
            ResourceRuntimeEvent::RequestCompleted { context: completed, .. }
                | ResourceRuntimeEvent::ContextCompleted { context: completed }
                if *completed == context
        )
    }));
    assert_eq!(runtime.request_context_progress(context).warmed_count(), 0);
}

#[test]
fn runtime_exposes_the_free_byte_budget_and_bounded_maintenance() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));

    assert_eq!(
        runtime.cache_config().free_byte_budget(),
        crate::DEFAULT_FREE_CACHE_BUDGET_BYTES
    );
    assert!(matches!(
        runtime.try_maintain_cache(),
        crate::ResourceCacheMaintenanceOutcome::Completed(_)
    ));
    runtime.clear_cache();
    assert_eq!(runtime.cache_stats().entry_count(), 0);
}

#[test]
fn tracks_context_scoped_prefetch() {
    let runtime = test_runtime(build_lookup_payload(DataId::new(MASTER_MAP_DID)));
    let reference_a = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(0x2000_0001),
        ResourceTypeId::FixtureA,
    );
    let reference_b = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(0x2000_0002),
        ResourceTypeId::FixtureA,
    );

    let context = runtime.begin_request_context(ResourcePriority::Background);
    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference_a));
    assert!(!runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference_a));
    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference_b));
    assert_eq!(runtime.request_context_progress(context).queued_count(), 2);
    runtime.release_request_context(context);
}

#[test]
fn worker_prefetch_queue_is_bounded() {
    let runtime = ResourceRuntime::with_config_and_preparation_executor(
        Arc::new(test_store(build_lookup_payload(DataId::new(
            MASTER_MAP_DID,
        )))),
        ResourceRuntimeConfig::default().with_max_pending_prefetch(1),
        Arc::new(InlinePreparationExecutor::default()),
    );
    let reference_a = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(0x2000_0001),
        ResourceTypeId::FixtureA,
    );
    let reference_b = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(0x2000_0002),
        ResourceTypeId::FixtureA,
    );

    let context = runtime.begin_request_context(ResourcePriority::Background);
    assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference_a));
    assert!(!runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference_b));
    runtime.release_request_context(context);
}

#[test]
fn typed_sequence_reuses_equal_roots_and_pumps_after_backpressure() {
    let store = Arc::new(test_store(build_string_table_payload()));
    let mut runtime = ResourceRuntime::with_config(
        store,
        ResourceRuntimeConfig::default().with_max_pending_prefetch(1),
    );
    let first = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID),
        ResourceTypeId::FixtureA,
    );
    let second = TypedResourceRef::from_resource_type(
        DatFileKind::Portal,
        DataId::new(STRING_TABLE_DID + 1),
        ResourceTypeId::FixtureA,
    );
    let roots: Arc<[TypedResourceRef]> = Arc::from([first, second]);
    let mut sequence = TypedPrefetchSequence::<RuntimeBodyDecoder>::default();
    sequence.synchronize(
        &mut runtime,
        None,
        Arc::clone(&roots),
        ResourcePriority::Critical,
        |reference| reference,
    );
    assert_eq!(sequence.queued_count(), 1);
    assert!(!sequence.is_complete());

    runtime.set_preparation_executor(Arc::new(InlinePreparationExecutor::default()));
    let events = runtime.pump_resource_events_for(Duration::from_secs(1));
    let context = events
        .iter()
        .find_map(|event| match event {
            ResourceRuntimeEvent::RequestCompleted { context, .. } => Some(*context),
            _ => None,
        })
        .expect("first request must complete");
    sequence.synchronize(
        &mut runtime,
        None,
        Arc::clone(&roots),
        ResourcePriority::Critical,
        |reference| reference,
    );
    assert_eq!(sequence.queued_count(), 2);
    assert!(sequence.is_complete());
    let equal_roots: Arc<[TypedResourceRef]> = Arc::from([first, second]);
    sequence.synchronize(
        &mut runtime,
        None,
        equal_roots,
        ResourcePriority::Critical,
        |reference| reference,
    );
    assert_eq!(runtime.request_context_progress(context).queued_count(), 2);

    let changed: Arc<[TypedResourceRef]> = Arc::from([second]);
    sequence.synchronize(
        &mut runtime,
        None,
        changed,
        ResourcePriority::Critical,
        |reference| reference,
    );
    assert_eq!(runtime.request_context_progress(context).queued_count(), 0);
    assert!(sequence.is_complete());
    let empty: Arc<[TypedResourceRef]> = Arc::from([]);
    sequence.synchronize(
        &mut runtime,
        None,
        empty,
        ResourcePriority::Critical,
        |reference| reference,
    );
    assert_eq!(sequence.queued_count(), 0);
    assert!(sequence.is_complete());
}

fn pump_until_events(runtime: &ResourceRuntime) -> Vec<ResourceRuntimeEvent> {
    runtime.pump_resource_events(8)
}

#[test]
fn generic_ddd_invalidation_invalidates_typed_cache() {
    let runtime = inline_runtime(Arc::new(test_store(build_string_table_payload())));
    let key = ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(
            DataId::new(STRING_TABLE_DID),
            ResourceTypeId::FixtureA as u32,
        ),
    );

    let cached = runtime
        .store()
        .load_cached_body::<RuntimeBodyDecoder>(TypedResourceRef::from_resource_type(
            key.file_kind(),
            key.data_id(),
            ResourceTypeId::FixtureA,
        ))
        .expect("cached load should succeed")
        .expect("table should exist");
    assert_eq!(runtime.cache_stats().entry_count(), 1);

    let outcome = runtime.apply_ddd_invalidation(ResourceDddInvalidation::new(key));

    assert_eq!(outcome.invalidated_typed_entries(), 1);
    assert_eq!(runtime.cache_stats().entry_count(), 0);

    let reloaded = runtime
        .store()
        .load_cached_body::<RuntimeBodyDecoder>(TypedResourceRef::from_resource_type(
            key.file_kind(),
            key.data_id(),
            ResourceTypeId::FixtureA,
        ))
        .expect("reload should succeed")
        .expect("table should exist");
    assert!(!CachedResource::ptr_eq(&cached, &reloaded));
}

/// Holds submitted jobs so the test decides when they run; reports four concurrent workers.
#[derive(Debug, Default)]
struct HeldExecutor {
    jobs: Mutex<Vec<PreparationJob>>,
}

impl PreparationExecutor for HeldExecutor {
    fn try_submit(&self, job: PreparationJob) -> Result<(), PreparationSubmitError> {
        self.jobs.lock().unwrap().push(job);
        Ok(())
    }

    fn parallelism(&self) -> usize {
        4
    }
}

#[test]
fn resource_requests_fill_executor_parallelism_and_complete_in_fifo_order() {
    let executor = Arc::new(HeldExecutor::default());
    let runtime = ResourceRuntime::with_preparation_executor(
        Arc::new(test_store(build_lookup_payload(DataId::new(
            MASTER_MAP_DID,
        )))),
        executor.clone(),
    );
    let context = runtime.begin_request_context(ResourcePriority::Interactive);
    let dids: Vec<u32> = (0..8).map(|index| 0x2000_0200 + index).collect();
    for did in &dids {
        let reference = TypedResourceRef::from_resource_type(
            DatFileKind::Portal,
            DataId::new(*did),
            ResourceTypeId::FixtureA,
        );
        assert!(runtime.queue_prefetch_typed_for_context::<RuntimeBodyDecoder>(context, reference));
    }

    let mut peak = 0;
    loop {
        let held = std::mem::take(&mut *executor.jobs.lock().unwrap());
        if held.is_empty() {
            break;
        }
        peak = peak.max(held.len());
        for job in held.into_iter().rev() {
            job.run();
        }
    }
    assert_eq!(
        peak, 4,
        "submissions fill, and never exceed, the executor's parallelism"
    );

    let completed: Vec<u32> = runtime
        .pump_resource_events(16)
        .into_iter()
        .filter_map(|event| match event {
            ResourceRuntimeEvent::RequestCompleted { key, .. } => Some(key.data_id().raw()),
            _ => None,
        })
        .collect();
    assert_eq!(completed, dids);
}

fn test_runtime(master_map_payload: Vec<u8>) -> ResourceRuntime {
    inline_runtime(Arc::new(test_store(master_map_payload)))
}

fn inline_runtime(store: Arc<ResourceStore>) -> ResourceRuntime {
    ResourceRuntime::with_preparation_executor(
        store,
        Arc::new(InlinePreparationExecutor::default()),
    )
}

fn test_store(record_payload: Vec<u8>) -> ResourceStore {
    let fixture = SingleRecordFixture::new(FixtureEntry {
        did: if record_payload.starts_with(&MASTER_MAP_DID.to_le_bytes()) {
            MASTER_MAP_DID
        } else {
            STRING_TABLE_DID
        },
        bytes: record_payload,
        flags: 0,
        version: 1,
        date_stamp: 0x1122_3344,
    })
    .expect("fixture should build");
    fixture.store().expect("store should open")
}

fn build_lookup_payload(did: DataId) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&did.raw().to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());

    out.extend_from_slice(&34u32.to_le_bytes());
    write_pstring(&mut out, "string");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&0x2000_0001u32.to_le_bytes());
    write_pstring(&mut out, "strings\\alpha.string");
    out.extend_from_slice(&0x2000_0002u32.to_le_bytes());
    write_pstring(&mut out, "strings\\beta.string");

    out
}

fn build_string_table_payload() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&STRING_TABLE_DID.to_le_bytes());
    out.extend_from_slice(&9u32.to_le_bytes());
    out.push(0);
    write_retail_compressed_u32(&mut out, 1);
    out.extend_from_slice(&4u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    write_utf16_string(&mut out, "hello");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.push(0);
    out
}

fn write_pstring(out: &mut Vec<u8>, value: &str) {
    write_compressed_u32(out, value.len() as u32);
    out.extend_from_slice(value.as_bytes());
}

fn write_utf16_string(out: &mut Vec<u8>, value: &str) {
    let encoded = value.encode_utf16().collect::<Vec<_>>();
    write_retail_compressed_u32(out, encoded.len() as u32);
    for code_unit in encoded {
        out.extend_from_slice(&code_unit.to_le_bytes());
    }
}

fn write_compressed_u32(out: &mut Vec<u8>, value: u32) {
    if value <= 0x7FFF {
        out.extend_from_slice(&(value as u16).to_le_bytes());
    } else {
        let first = ((value >> 16) as u16) | 0x8000;
        let second = value as u16;
        out.extend_from_slice(&first.to_le_bytes());
        out.extend_from_slice(&second.to_le_bytes());
    }
}

fn write_retail_compressed_u32(out: &mut Vec<u8>, value: u32) {
    if value < 0x80 {
        out.push(value as u8);
    } else if value < 0x4000 {
        out.push(((value >> 8) as u8) | 0x80);
        out.push((value & 0xFF) as u8);
    } else {
        out.push(((value >> 24) as u8) | 0xC0);
        out.push(((value >> 16) & 0xFF) as u8);
        out.push(((value >> 8) & 0xFF) as u8);
        out.push((value & 0xFF) as u8);
    }
}
