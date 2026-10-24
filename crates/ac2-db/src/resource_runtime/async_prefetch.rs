//! Resource-owned FIFO request state over a client-owned immutable-preparation executor.
//!
//! This module deliberately owns no thread. `ResourceRuntime` keeps typed request identity,
//! decode-scope ownership, deduplication, cancellation, and completion fan-in; the client
//! injects the shared executor that decides when a resource step may run. Requests are claimed
//! in FIFO order by up to `parallelism()` concurrent jobs, and completions are published in that
//! same order.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{
    PreparationDomain, PreparationExecutor, PreparationJob, PreparationPriority,
    PreparationTransport, ResourceBodyDecode, ResourceDecodeScope, ResourceKey, ResourceStore,
    TypedResourceRef,
};

use super::completion_order::CompletionOrder;
use super::{PrefetchRequest, ResourceRuntime, ResourceRuntimeConfig};

/// Admission metadata supplied by the owner that discovered a resource request.
///
/// The client executor may prioritize different domains, but a resource request's original
/// FIFO position is never changed when another consumer joins the same get.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourcePriority {
    Background,
    Interactive,
    Critical,
}

impl ResourcePriority {
    const fn preparation_priority(self) -> PreparationPriority {
        match self {
            Self::Background => PreparationPriority::Background,
            Self::Interactive => PreparationPriority::Interactive,
            Self::Critical => PreparationPriority::Critical,
        }
    }
}

/// Opaque handle for a client-owned resource request scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceRequestContext(u64);

impl ResourceRequestContext {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceRequestOutcome {
    Warmed,
    Missing,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceRuntimeEvent {
    RequestCompleted {
        context: ResourceRequestContext,
        key: ResourceKey,
        outcome: ResourceRequestOutcome,
    },
    ContextCompleted {
        context: ResourceRequestContext,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceRequestProgress {
    queued_count: usize,
    warmed_count: usize,
    missing_count: usize,
    failed_count: usize,
    cancelled_count: usize,
}

impl ResourceRequestProgress {
    pub const fn queued_count(self) -> usize {
        self.queued_count
    }

    pub const fn warmed_count(self) -> usize {
        self.warmed_count
    }

    pub const fn missing_count(self) -> usize {
        self.missing_count
    }

    pub const fn failed_count(self) -> usize {
        self.failed_count
    }

    pub const fn cancelled_count(self) -> usize {
        self.cancelled_count
    }

    fn record(&mut self, outcome: ResourceRequestOutcome) {
        match outcome {
            ResourceRequestOutcome::Warmed => self.warmed_count += 1,
            ResourceRequestOutcome::Missing => self.missing_count += 1,
            ResourceRequestOutcome::Failed => self.failed_count += 1,
            ResourceRequestOutcome::Cancelled => self.cancelled_count += 1,
        }
    }
}

#[derive(Debug)]
pub(super) struct RequestContextState {
    priority: ResourcePriority,
    decode_scope: Option<ResourceDecodeScope>,
    pending_count: usize,
    progress: ResourceRequestProgress,
}

impl RequestContextState {
    fn new(priority: ResourcePriority, decode_scope: Option<ResourceDecodeScope>) -> Self {
        Self {
            priority,
            decode_scope,
            pending_count: 0,
            progress: ResourceRequestProgress::default(),
        }
    }
}

/// Scheduler identity deliberately keeps generic cache warming separate from values owned by
/// one active decode scope. Two scopes may share a decode flight, never scheduler ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PendingRequestKey {
    Generic(ResourceKey),
    Scoped {
        key: ResourceKey,
        scope_identity: u64,
    },
}

impl PendingRequestKey {
    const fn resource_key(self) -> ResourceKey {
        match self {
            Self::Generic(key) | Self::Scoped { key, .. } => key,
        }
    }
}

#[derive(Debug)]
pub(super) struct PendingRequest {
    request: PrefetchRequest,
    decode_scope: Option<ResourceDecodeScope>,
    contexts: BTreeSet<ResourceRequestContext>,
    unscoped: bool,
    priority: ResourcePriority,
    started: bool,
}

#[derive(Debug)]
pub(super) struct SchedulerState {
    pending: HashMap<PendingRequestKey, PendingRequest>,
    contexts: BTreeMap<ResourceRequestContext, RequestContextState>,
    next_context_id: u64,
    transport: PreparationTransport<PendingRequestKey, PrefetchCompletion>,
    /// Executor jobs submitted and not yet finished.
    executor_tasks_in_flight: usize,
    /// Executor jobs submitted that have not yet claimed a request.
    executor_tasks_unclaimed: usize,
    completion_order: CompletionOrder<PrefetchCompletion>,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            pending: HashMap::new(),
            contexts: BTreeMap::new(),
            next_context_id: 1,
            transport: PreparationTransport::default(),
            executor_tasks_in_flight: 0,
            executor_tasks_unclaimed: 0,
            completion_order: CompletionOrder::default(),
        }
    }
}

/// A request a worker has claimed, with its place in completion order.
struct ClaimedRequest {
    key: PendingRequestKey,
    request: PrefetchRequest,
    decode_scope: Option<ResourceDecodeScope>,
    cancelled: bool,
    ticket: u64,
}

#[derive(Debug, Clone, Copy)]
struct PrefetchCompletion {
    request_key: PendingRequestKey,
    outcome: ResourceRequestOutcome,
}

/// Resource-owned request and reply state. It contains no executor and never starts a thread.
#[derive(Debug, Default)]
pub(super) struct AsyncPrefetchCoordinator {
    scheduler: Mutex<SchedulerState>,
}

impl AsyncPrefetchCoordinator {
    pub(super) fn begin_context(
        &self,
        priority: ResourcePriority,
        decode_scope: Option<ResourceDecodeScope>,
    ) -> ResourceRequestContext {
        let mut scheduler = self
            .scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned");
        let context = ResourceRequestContext(scheduler.next_context_id);
        scheduler.next_context_id = scheduler.next_context_id.saturating_add(1).max(1);
        scheduler
            .contexts
            .insert(context, RequestContextState::new(priority, decode_scope));
        context
    }

    pub(super) fn release_context(&self, context: ResourceRequestContext) -> bool {
        let mut scheduler = self
            .scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned");
        release_context_requests(&mut scheduler, context)
    }

    pub(super) fn progress(&self, context: ResourceRequestContext) -> ResourceRequestProgress {
        self.scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned")
            .contexts
            .get(&context)
            .map(|state| state.progress)
            .unwrap_or_default()
    }

    fn enqueue(
        &self,
        config: ResourceRuntimeConfig,
        key: PendingRequestKey,
        request: PrefetchRequest,
        context: Option<ResourceRequestContext>,
        priority: ResourcePriority,
        decode_scope: Option<ResourceDecodeScope>,
    ) -> bool {
        let mut scheduler = self
            .scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned");
        enqueue_request(
            &mut scheduler,
            config,
            key,
            request,
            context,
            priority,
            decode_scope,
        )
    }

    /// Fills the executor with resource jobs, up to its parallelism. Each job claims the next
    /// live request in FIFO order when it runs, and completions are published in claim order
    /// (see [`CompletionOrder`]), so concurrency never reorders what consumers observe.
    pub(super) fn schedule_next(
        self: &Arc<Self>,
        store: Arc<ResourceStore>,
        executor: Arc<dyn PreparationExecutor>,
    ) {
        let parallelism = executor.parallelism().max(1);
        loop {
            let priority = {
                let mut scheduler = self
                    .scheduler
                    .lock()
                    .expect("resource prefetch scheduler poisoned");
                // Queued keys bound how many waiting jobs can still find a request; stale keys
                // only make a job return without work.
                let claimable = scheduler
                    .transport
                    .request_len()
                    .saturating_sub(scheduler.executor_tasks_unclaimed);
                if scheduler.executor_tasks_in_flight >= parallelism || claimable == 0 {
                    return;
                }
                let Some(key) = next_live_request_key(&mut scheduler) else {
                    return;
                };
                let Some(pending) = scheduler.pending.get(&key) else {
                    return;
                };
                let priority = pending.priority.preparation_priority();
                scheduler.executor_tasks_in_flight += 1;
                scheduler.executor_tasks_unclaimed += 1;
                priority
            };
            let coordinator = Arc::clone(self);
            let follow_on_store = Arc::clone(&store);
            let follow_on_executor = Arc::clone(&executor);
            let job = PreparationJob::new(PreparationDomain::Resource, priority, move || {
                coordinator.run_one(&follow_on_store);
                coordinator.finish_executor_task();
                coordinator.schedule_next(follow_on_store, follow_on_executor);
            });
            if executor.try_submit(job).is_err() {
                let mut scheduler = self
                    .scheduler
                    .lock()
                    .expect("resource prefetch scheduler poisoned");
                scheduler.executor_tasks_in_flight -= 1;
                scheduler.executor_tasks_unclaimed -= 1;
                return;
            }
        }
    }

    fn run_one(&self, store: &ResourceStore) {
        let Some(claimed) = self.take_next_request() else {
            return;
        };
        let request_key = claimed.key;
        let outcome = if claimed.cancelled {
            ResourceRequestOutcome::Cancelled
        } else {
            // A development panic must still release its ticket, or every later completion
            // would wait behind it forever.
            let executed = catch_unwind(AssertUnwindSafe(|| {
                claimed.request.execute(
                    store,
                    claimed.decode_scope.as_ref(),
                    request_key.resource_key(),
                )
            }));
            match executed {
                Ok(Ok(true)) => ResourceRequestOutcome::Warmed,
                Ok(Ok(false)) => ResourceRequestOutcome::Missing,
                Ok(Err(_)) | Err(_) => ResourceRequestOutcome::Failed,
            }
        };
        let mut scheduler = self
            .scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned");
        let SchedulerState {
            transport,
            completion_order,
            ..
        } = &mut *scheduler;
        completion_order.finish(
            claimed.ticket,
            PrefetchCompletion {
                request_key,
                outcome,
            },
            |completion| {
                transport.enqueue_reply(completion);
            },
        );
    }

    fn take_next_request(&self) -> Option<ClaimedRequest> {
        let mut scheduler = self
            .scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned");
        scheduler.executor_tasks_unclaimed -= 1;
        loop {
            let key = scheduler.transport.dequeue_request()?.into_value();
            let Some(pending) = scheduler.pending.get_mut(&key) else {
                continue;
            };
            if pending.started {
                continue;
            }
            pending.started = true;
            let request = pending.request;
            let decode_scope = pending.decode_scope.clone();
            let cancelled = pending.contexts.is_empty() && !pending.unscoped;
            return Some(ClaimedRequest {
                key,
                request,
                decode_scope,
                cancelled,
                ticket: scheduler.completion_order.issue(),
            });
        }
    }

    fn finish_executor_task(&self) {
        self.scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned")
            .executor_tasks_in_flight -= 1;
    }

    fn take_completion(&self) -> Option<PrefetchCompletion> {
        self.scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned")
            .transport
            .dequeue_reply()
            .map(|reply| reply.into_value())
    }

    fn apply_completion(
        &self,
        completion: PrefetchCompletion,
        events: &mut Vec<ResourceRuntimeEvent>,
    ) {
        let mut scheduler = self
            .scheduler
            .lock()
            .expect("resource prefetch scheduler poisoned");
        let Some(pending) = scheduler.pending.remove(&completion.request_key) else {
            return;
        };
        for context in pending.contexts {
            let Some(context_state) = scheduler.contexts.get_mut(&context) else {
                continue;
            };
            context_state.progress.record(completion.outcome);
            context_state.pending_count = context_state.pending_count.saturating_sub(1);
            events.push(ResourceRuntimeEvent::RequestCompleted {
                context,
                key: completion.request_key.resource_key(),
                outcome: completion.outcome,
            });
            if context_state.pending_count == 0 {
                events.push(ResourceRuntimeEvent::ContextCompleted { context });
            }
        }
    }
}

impl ResourceRuntime {
    /// Creates a request scope owned by the caller until it is explicitly released.
    pub fn begin_request_context(&self, priority: ResourcePriority) -> ResourceRequestContext {
        self.begin_request_context_inner(priority, None)
    }

    /// Creates a request context whose body results belong exclusively to `decode_scope`.
    pub fn begin_scoped_request_context(
        &self,
        priority: ResourcePriority,
        decode_scope: ResourceDecodeScope,
    ) -> ResourceRequestContext {
        self.begin_request_context_inner(priority, Some(decode_scope))
    }

    fn begin_request_context_inner(
        &self,
        priority: ResourcePriority,
        decode_scope: Option<ResourceDecodeScope>,
    ) -> ResourceRequestContext {
        self.coordinator.begin_context(priority, decode_scope)
    }

    /// Releases a superseded scope. A job already admitted to Worker 2 may finish warming a
    /// cache entry, but this context receives no stale completion event.
    pub fn release_request_context(&self, context: ResourceRequestContext) {
        if self.coordinator.release_context(context) {
            self.schedule_preparation();
        }
    }

    pub fn request_context_progress(
        &self,
        context: ResourceRequestContext,
    ) -> ResourceRequestProgress {
        self.coordinator.progress(context)
    }

    /// Queues a concrete domain body decoder through the resource FIFO.
    pub fn queue_prefetch_typed_for_context<T>(
        &self,
        context: ResourceRequestContext,
        reference: TypedResourceRef,
    ) -> bool
    where
        T: ResourceBodyDecode,
    {
        self.enqueue_context_request(
            context,
            reference.resource_key(),
            PrefetchRequest::body::<T>(),
        )
    }

    /// Queues a concrete DBObj body into the supplied context's active decode scope.
    pub fn queue_scoped_prefetch_typed_for_context<T>(
        &self,
        context: ResourceRequestContext,
        reference: TypedResourceRef,
    ) -> bool
    where
        T: ResourceBodyDecode,
    {
        self.enqueue_scoped_context_request(
            context,
            reference.resource_key(),
            PrefetchRequest::scoped_body::<T>(),
        )
    }

    /// Submits an unscoped typed cache-warming request to the resource FIFO.
    pub fn queue_prefetch_typed_async<T>(&self, reference: TypedResourceRef) -> bool
    where
        T: ResourceBodyDecode,
    {
        self.enqueue_unscoped_request(reference.resource_key(), PrefetchRequest::body::<T>())
    }

    /// Applies at most `max_count` Worker-2 completions without waiting for new work.
    pub fn pump_resource_events(&self, max_count: usize) -> Vec<ResourceRuntimeEvent> {
        self.schedule_preparation();
        let mut events = Vec::new();
        for _ in 0..max_count {
            let Some(completion) = self.coordinator.take_completion() else {
                break;
            };
            self.coordinator.apply_completion(completion, &mut events);
        }
        self.schedule_preparation();
        events
    }

    /// Retail-shaped nonblocking completion use-time pump.
    ///
    /// It stops as soon as replies are empty or the caller's retail 25 ms window expires, and
    /// never waits for Worker 2.
    pub fn pump_resource_events_for(&self, budget: Duration) -> Vec<ResourceRuntimeEvent> {
        self.schedule_preparation();
        let deadline = Instant::now().checked_add(budget);
        let mut events = Vec::new();
        loop {
            if deadline.is_some_and(|limit| Instant::now() >= limit) {
                break;
            }
            let Some(completion) = self.coordinator.take_completion() else {
                break;
            };
            self.coordinator.apply_completion(completion, &mut events);
        }
        self.schedule_preparation();
        events
    }

    pub(super) fn enqueue_context_request(
        &self,
        context: ResourceRequestContext,
        key: ResourceKey,
        request: PrefetchRequest,
    ) -> bool {
        let (priority, decode_scope) = {
            let scheduler = self
                .coordinator
                .scheduler
                .lock()
                .expect("resource prefetch scheduler poisoned");
            let Some(context_state) = scheduler.contexts.get(&context) else {
                return false;
            };
            if context_state.decode_scope.is_some() {
                return false;
            }
            (context_state.priority, None)
        };
        let accepted = self.coordinator.enqueue(
            self.config,
            PendingRequestKey::Generic(key),
            request,
            Some(context),
            priority,
            decode_scope,
        );
        if accepted {
            self.schedule_preparation();
        }
        accepted
    }

    fn enqueue_scoped_context_request(
        &self,
        context: ResourceRequestContext,
        key: ResourceKey,
        request: PrefetchRequest,
    ) -> bool {
        let (priority, decode_scope) = {
            let scheduler = self
                .coordinator
                .scheduler
                .lock()
                .expect("resource prefetch scheduler poisoned");
            let Some(context_state) = scheduler.contexts.get(&context) else {
                return false;
            };
            let Some(decode_scope) = context_state.decode_scope.clone() else {
                return false;
            };
            (context_state.priority, decode_scope)
        };
        let accepted = self.coordinator.enqueue(
            self.config,
            PendingRequestKey::Scoped {
                key,
                scope_identity: decode_scope.identity(),
            },
            request,
            Some(context),
            priority,
            Some(decode_scope),
        );
        if accepted {
            self.schedule_preparation();
        }
        accepted
    }

    fn enqueue_unscoped_request(&self, key: ResourceKey, request: PrefetchRequest) -> bool {
        let accepted = self.coordinator.enqueue(
            self.config,
            PendingRequestKey::Generic(key),
            request,
            None,
            ResourcePriority::Background,
            None,
        );
        if accepted {
            self.schedule_preparation();
        }
        accepted
    }
}

/// Drops prefetches whose only consumer was released before Worker 2 starts them. Stale queue
/// keys are skipped at consumption; they never create a new FIFO request or overtake live work.
fn discard_unstarted_orphaned_requests(scheduler: &mut SchedulerState) {
    scheduler
        .pending
        .retain(|_, pending| pending.started || pending.unscoped || !pending.contexts.is_empty());
}

pub(super) fn release_context_requests(
    scheduler: &mut SchedulerState,
    context: ResourceRequestContext,
) -> bool {
    if scheduler.contexts.remove(&context).is_none() {
        return false;
    }
    for pending in scheduler.pending.values_mut() {
        pending.contexts.remove(&context);
    }
    discard_unstarted_orphaned_requests(scheduler);
    true
}

pub(super) fn enqueue_request(
    scheduler: &mut SchedulerState,
    config: ResourceRuntimeConfig,
    key: PendingRequestKey,
    request: PrefetchRequest,
    context: Option<ResourceRequestContext>,
    priority: ResourcePriority,
    decode_scope: Option<ResourceDecodeScope>,
) -> bool {
    if let Some(pending) = scheduler.pending.get_mut(&key) {
        let was_new_interest = match context {
            Some(context) => pending.contexts.insert(context),
            None if pending.unscoped => false,
            None => {
                pending.unscoped = true;
                true
            }
        };
        if !was_new_interest {
            return false;
        }
        if let Some(context) = context {
            if let Some(state) = scheduler.contexts.get_mut(&context) {
                state.pending_count += 1;
                state.progress.queued_count += 1;
            }
        }
        // A duplicate joins the original get. It deliberately receives no new queue entry and
        // cannot promote or otherwise reorder that original request.
        return true;
    }

    if scheduler.pending.len() >= config.max_pending_prefetch() {
        return false;
    }
    let mut contexts = BTreeSet::new();
    if let Some(context) = context {
        contexts.insert(context);
        if let Some(state) = scheduler.contexts.get_mut(&context) {
            state.pending_count += 1;
            state.progress.queued_count += 1;
        }
    }
    scheduler.pending.insert(
        key,
        PendingRequest {
            request,
            decode_scope,
            contexts,
            unscoped: context.is_none(),
            priority,
            started: false,
        },
    );
    scheduler.transport.enqueue_request(key);
    true
}

fn next_live_request_key(scheduler: &mut SchedulerState) -> Option<PendingRequestKey> {
    loop {
        let key = *scheduler.transport.front_request()?;
        if scheduler
            .pending
            .get(&key)
            .is_some_and(|pending| !pending.started)
        {
            return Some(key);
        }
        let _ = scheduler.transport.dequeue_request();
    }
}

#[cfg(test)]
#[path = "async_prefetch_tests.rs"]
mod tests;
