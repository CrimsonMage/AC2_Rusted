use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;

use super::{
    enqueue_request, next_live_request_key, release_context_requests, PendingRequestKey,
    PrefetchRequest, RequestContextState, ResourcePriority, ResourceRequestContext,
    ResourceRuntimeConfig, SchedulerState,
};
use crate::{
    ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext, ResourceDecodeScope,
    ResourceKey, ResourceResult, ResourceTypeId,
};

#[derive(Debug)]
struct GenericPrefetchTest;

impl ResourceBodyDecode for GenericPrefetchTest {
    const TYPE_NAME: &'static str = "generic prefetch test";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::FixtureA;

    fn decode_body(
        _context: ResourceDbObjDecodeContext,
        _body: ResourceDbObjBody<'_>,
    ) -> ResourceResult<Self> {
        Ok(Self)
    }

    fn cache_retained_bytes(&self) -> usize {
        0
    }
}

fn key(raw: u32) -> ResourceKey {
    ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(raw), 34),
    )
}

fn generic(key: ResourceKey) -> PendingRequestKey {
    PendingRequestKey::Generic(key)
}

fn enqueue(
    scheduler: &mut SchedulerState,
    request_key: PendingRequestKey,
    context: Option<ResourceRequestContext>,
    priority: ResourcePriority,
    scope: Option<ResourceDecodeScope>,
) -> bool {
    enqueue_request(
        scheduler,
        ResourceRuntimeConfig::default(),
        request_key,
        PrefetchRequest::body::<GenericPrefetchTest>(),
        context,
        priority,
        scope,
    )
}

#[test]
fn resource_fifo_is_not_reordered_by_later_priority() {
    let mut scheduler = SchedulerState::default();
    let background = generic(key(0x2000_0001));
    let critical = generic(key(0x2000_0002));

    assert!(enqueue(
        &mut scheduler,
        background,
        None,
        ResourcePriority::Background,
        None,
    ));
    assert!(enqueue(
        &mut scheduler,
        critical,
        None,
        ResourcePriority::Critical,
        None,
    ));

    assert_eq!(next_live_request_key(&mut scheduler), Some(background));
    assert_eq!(
        scheduler
            .transport
            .dequeue_request()
            .map(|entry| entry.into_value()),
        Some(background)
    );
    assert_eq!(next_live_request_key(&mut scheduler), Some(critical));
}

#[test]
fn shared_generic_key_joins_without_a_second_fifo_entry_or_promotion() {
    let mut scheduler = SchedulerState::default();
    let background = ResourceRequestContext::new(1);
    let critical = ResourceRequestContext::new(2);
    scheduler.contexts.insert(
        background,
        RequestContextState::new(ResourcePriority::Background, None),
    );
    scheduler.contexts.insert(
        critical,
        RequestContextState::new(ResourcePriority::Critical, None),
    );
    let request_key = generic(key(0x2000_0010));

    assert!(enqueue(
        &mut scheduler,
        request_key,
        Some(background),
        ResourcePriority::Background,
        None,
    ));
    assert!(enqueue(
        &mut scheduler,
        request_key,
        Some(critical),
        ResourcePriority::Critical,
        None,
    ));

    let pending = scheduler
        .pending
        .get(&request_key)
        .expect("request should be pending");
    assert_eq!(pending.contexts.len(), 2);
    assert_eq!(pending.priority, ResourcePriority::Background);
    assert_eq!(scheduler.transport.request_len(), 1);
    assert_eq!(next_live_request_key(&mut scheduler), Some(request_key));
}

#[test]
fn released_context_discards_unstarted_orphaned_request_without_new_work() {
    let mut scheduler = SchedulerState::default();
    let context = ResourceRequestContext::new(1);
    scheduler.contexts.insert(
        context,
        RequestContextState::new(ResourcePriority::Background, None),
    );
    let request_key = generic(key(0x2000_0020));

    assert!(enqueue(
        &mut scheduler,
        request_key,
        Some(context),
        ResourcePriority::Background,
        None,
    ));
    assert!(release_context_requests(&mut scheduler, context));
    assert!(scheduler.pending.is_empty());
    assert_eq!(next_live_request_key(&mut scheduler), None);
}

#[test]
fn distinct_scopes_never_merge_scheduler_ownership() {
    let mut scheduler = SchedulerState::default();
    let first_scope = ResourceDecodeScope::default();
    let second_scope = ResourceDecodeScope::default();
    let first_context = ResourceRequestContext::new(1);
    let second_context = ResourceRequestContext::new(2);
    scheduler.contexts.insert(
        first_context,
        RequestContextState::new(ResourcePriority::Critical, Some(first_scope.clone())),
    );
    scheduler.contexts.insert(
        second_context,
        RequestContextState::new(ResourcePriority::Critical, Some(second_scope.clone())),
    );
    let resource_key = key(0x2000_0030);
    let first_key = PendingRequestKey::Scoped {
        key: resource_key,
        scope_identity: first_scope.identity(),
    };
    let second_key = PendingRequestKey::Scoped {
        key: resource_key,
        scope_identity: second_scope.identity(),
    };

    assert!(enqueue(
        &mut scheduler,
        first_key,
        Some(first_context),
        ResourcePriority::Critical,
        Some(first_scope),
    ));
    assert!(enqueue(
        &mut scheduler,
        second_key,
        Some(second_context),
        ResourcePriority::Critical,
        Some(second_scope),
    ));
    assert_eq!(scheduler.pending.len(), 2);

    assert!(release_context_requests(&mut scheduler, first_context));
    assert_eq!(scheduler.pending.len(), 1);
    assert!(scheduler.pending.contains_key(&second_key));
}
