//! Retail-shaped FIFO transport for immutable preparation work.
//!
//! `ThreadedCache` uses a 1024-slot ring plus a backup list for both requests and replies.
//! This Rust-native transport keeps the same logical FIFO independent of the transport split:
//! while a spill backlog exists, newer work stays behind it even when a fast slot reopens.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;

use crate::ResourceKey;

/// The capacity of each retail `ThreadedCache` request/reply fast transport.
pub const RETAIL_PREPARATION_FAST_QUEUE_CAPACITY: usize = 1024;

/// Monotonic request ordering assigned before a request reaches a fast or spill transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreparationRequestSequence(u64);

impl PreparationRequestSequence {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Monotonic reply ordering assigned before a reply reaches a fast or spill transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreparationReplySequence(u64);

impl PreparationReplySequence {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A request or reply paired with the order that governs observable processing.
#[derive(Debug)]
pub struct SequencedPreparation<T, S> {
    sequence: S,
    value: T,
}

impl<T, S: Copy> SequencedPreparation<T, S> {
    pub const fn sequence(&self) -> S {
        self.sequence
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

/// Outcome returned by a dependency-aware immutable preparation step.
///
/// The sole preparation lane must return `WaitingOn` instead of recursively loading a missing
/// child and waiting for itself. The host resolves the dependency and re-admits the parent at
/// the FIFO tail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparationStep<T> {
    Complete(T),
    WaitingOn(Arc<[ResourceKey]>),
    Failed(PreparationError),
    Cancelled,
}

/// Domain-neutral failure information for a preparation continuation.
///
/// Owners retain their richer typed errors at their own boundaries. This transport-level value
/// is intentionally only the stable result needed to complete, cancel, or retry a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationError {
    message: Arc<str>,
}

impl PreparationError {
    pub fn new(message: impl Into<Arc<str>>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// One logical FIFO with a retail-sized fast transport and an ordered spill transport.
///
/// `fast` is deliberately not refilled from `spill` on a later submission. Once spilling starts,
/// all newer work joins the spill tail until it drains. That simple rule makes the ordering
/// invariant explicit and prevents an older spill entry from being overtaken by a newer fast
/// entry.
#[derive(Debug)]
pub struct OrderedSpillQueue<T, S> {
    fast: VecDeque<SequencedPreparation<T, S>>,
    spill: VecDeque<SequencedPreparation<T, S>>,
    capacity: usize,
}

impl<T, S> OrderedSpillQueue<T, S> {
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "preparation fast queue capacity must be nonzero"
        );
        Self {
            fast: VecDeque::with_capacity(capacity),
            spill: VecDeque::new(),
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.fast.len() + self.spill.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fast.is_empty() && self.spill.is_empty()
    }

    pub fn fast_len(&self) -> usize {
        self.fast.len()
    }

    pub fn spill_len(&self) -> usize {
        self.spill.len()
    }

    pub fn has_spill_backlog(&self) -> bool {
        !self.spill.is_empty()
    }

    fn front(&self) -> Option<&SequencedPreparation<T, S>> {
        self.fast.front().or_else(|| self.spill.front())
    }

    fn push(&mut self, item: SequencedPreparation<T, S>) {
        if self.spill.is_empty() && self.fast.len() < self.capacity {
            self.fast.push_back(item);
        } else {
            self.spill.push_back(item);
        }
    }

    fn pop(&mut self) -> Option<SequencedPreparation<T, S>> {
        self.fast.pop_front().or_else(|| self.spill.pop_front())
    }
}

impl<T> Default for OrderedSpillQueue<T, PreparationRequestSequence> {
    fn default() -> Self {
        Self::with_capacity(RETAIL_PREPARATION_FAST_QUEUE_CAPACITY)
    }
}

impl<T> Default for OrderedSpillQueue<T, PreparationReplySequence> {
    fn default() -> Self {
        Self::with_capacity(RETAIL_PREPARATION_FAST_QUEUE_CAPACITY)
    }
}

/// Separate request and reply FIFO transports, each with an independent monotonic sequence.
#[derive(Debug)]
pub struct PreparationTransport<Request, Reply> {
    requests: OrderedSpillQueue<Request, PreparationRequestSequence>,
    replies: OrderedSpillQueue<Reply, PreparationReplySequence>,
    next_request_sequence: u64,
    next_reply_sequence: u64,
}

impl<Request, Reply> Default for PreparationTransport<Request, Reply> {
    fn default() -> Self {
        Self::new(RETAIL_PREPARATION_FAST_QUEUE_CAPACITY)
    }
}

impl<Request, Reply> PreparationTransport<Request, Reply> {
    pub fn new(fast_capacity: usize) -> Self {
        Self {
            requests: OrderedSpillQueue::with_capacity(fast_capacity),
            replies: OrderedSpillQueue::with_capacity(fast_capacity),
            next_request_sequence: 1,
            next_reply_sequence: 1,
        }
    }

    pub fn enqueue_request(&mut self, request: Request) -> PreparationRequestSequence {
        let sequence = PreparationRequestSequence(self.next_request_sequence);
        self.next_request_sequence = self.next_request_sequence.saturating_add(1).max(1);
        self.requests.push(SequencedPreparation {
            sequence,
            value: request,
        });
        sequence
    }

    pub fn dequeue_request(
        &mut self,
    ) -> Option<SequencedPreparation<Request, PreparationRequestSequence>> {
        self.requests.pop()
    }

    /// Observes the oldest request without changing the logical FIFO.
    ///
    /// A client-owned executor uses this only to choose cross-domain admission metadata. The
    /// request is still removed and started by its owner on the shared preparation worker.
    pub fn front_request(&self) -> Option<&Request> {
        self.requests.front().map(|entry| &entry.value)
    }

    pub fn enqueue_reply(&mut self, reply: Reply) -> PreparationReplySequence {
        let sequence = PreparationReplySequence(self.next_reply_sequence);
        self.next_reply_sequence = self.next_reply_sequence.saturating_add(1).max(1);
        self.replies.push(SequencedPreparation {
            sequence,
            value: reply,
        });
        sequence
    }

    pub fn dequeue_reply(
        &mut self,
    ) -> Option<SequencedPreparation<Reply, PreparationReplySequence>> {
        self.replies.pop()
    }

    /// Observes the oldest reply without changing the logical FIFO.
    pub fn front_reply(&self) -> Option<&Reply> {
        self.replies.front().map(|entry| &entry.value)
    }

    pub fn request_len(&self) -> usize {
        self.requests.len()
    }

    pub fn reply_len(&self) -> usize {
        self.replies.len()
    }

    pub fn requests_have_spill_backlog(&self) -> bool {
        self.requests.has_spill_backlog()
    }

    pub fn replies_have_spill_backlog(&self) -> bool {
        self.replies.has_spill_backlog()
    }
}

/// One in-flight exact cache get and every consumer that joined it.
#[derive(Debug)]
pub struct InFlightGet<Consumer> {
    request_sequence: PreparationRequestSequence,
    consumers: Vec<Consumer>,
}

impl<Consumer> InFlightGet<Consumer> {
    pub const fn request_sequence(&self) -> PreparationRequestSequence {
        self.request_sequence
    }

    pub fn consumers(&self) -> &[Consumer] {
        &self.consumers
    }

    pub fn into_consumers(self) -> Vec<Consumer> {
        self.consumers
    }
}

/// Exact-key deduplication preceding FIFO admission.
///
/// A duplicate never consumes a second sequence or queue position: it is attached to the
/// original request and receives the original completion fan-in.
#[derive(Debug, Default)]
pub struct InFlightGetRegistry<Key, Consumer> {
    gets: HashMap<Key, InFlightGet<Consumer>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetAdmission {
    Queued(PreparationRequestSequence),
    Coalesced(PreparationRequestSequence),
}

impl<Key, Consumer> InFlightGetRegistry<Key, Consumer>
where
    Key: Eq + Hash,
{
    pub fn admit(
        &mut self,
        key: Key,
        consumer: Consumer,
        enqueue: impl FnOnce() -> PreparationRequestSequence,
    ) -> GetAdmission {
        if let Some(existing) = self.gets.get_mut(&key) {
            existing.consumers.push(consumer);
            return GetAdmission::Coalesced(existing.request_sequence);
        }
        let request_sequence = enqueue();
        self.gets.insert(
            key,
            InFlightGet {
                request_sequence,
                consumers: vec![consumer],
            },
        );
        GetAdmission::Queued(request_sequence)
    }

    pub fn complete(&mut self, key: &Key) -> Option<InFlightGet<Consumer>> {
        self.gets.remove(key)
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.gets.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.gets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.gets.is_empty()
    }
}

/// Parent dependency fan-in. A parent returns only after *every* dependency from its most
/// recent `WaitingOn` outcome has completed. Returned parents preserve their original wait
/// order; callers must re-enqueue those continuations at the request FIFO tail.
///
/// A single child can unblock several parents, but a parent which waits on `A` and `B` must not
/// resume after just `A`: doing so would silently recreate nested load/retry behavior inside the
/// sole preparation worker.
#[derive(Debug)]
pub struct PreparationDependencyFanIn<Parent> {
    parents_by_dependency: HashMap<ResourceKey, Vec<Parent>>,
    remaining_by_parent: HashMap<Parent, PendingParentDependencies>,
    next_wait_sequence: u64,
}

#[derive(Debug)]
struct PendingParentDependencies {
    remaining: Vec<ResourceKey>,
    wait_sequence: u64,
}

impl<Parent> Default for PreparationDependencyFanIn<Parent> {
    fn default() -> Self {
        Self {
            parents_by_dependency: HashMap::new(),
            remaining_by_parent: HashMap::new(),
            next_wait_sequence: 1,
        }
    }
}

impl<Parent> PreparationDependencyFanIn<Parent>
where
    Parent: Clone + Eq + Hash,
{
    /// Records a fresh dependency set for a suspended parent.
    ///
    /// Re-registering a continuation replaces its old unresolved set. Stale dependency list
    /// entries are ignored at resolution time, which keeps cancellation/retry cleanup cheap
    /// without ever releasing a parent early.
    pub fn wait_on(&mut self, parent: Parent, dependencies: impl IntoIterator<Item = ResourceKey>) {
        let mut remaining = Vec::new();
        for dependency in dependencies {
            if !remaining.contains(&dependency) {
                remaining.push(dependency);
            }
        }
        if remaining.is_empty() {
            return;
        }

        let wait_sequence = self.next_wait_sequence;
        self.next_wait_sequence = self.next_wait_sequence.saturating_add(1).max(1);
        for dependency in &remaining {
            self.parents_by_dependency
                .entry(*dependency)
                .or_default()
                .push(parent.clone());
        }
        self.remaining_by_parent.insert(
            parent,
            PendingParentDependencies {
                remaining,
                wait_sequence,
            },
        );
    }

    /// Marks one dependency complete and returns only parents whose full dependency set is now
    /// complete. The caller inserts each result at the logical FIFO tail.
    pub fn resolve(&mut self, dependency: ResourceKey) -> Vec<Parent> {
        let Some(parents) = self.parents_by_dependency.remove(&dependency) else {
            return Vec::new();
        };
        let mut ready = Vec::new();
        for parent in parents {
            let Some(pending) = self.remaining_by_parent.get_mut(&parent) else {
                continue;
            };
            let Some(index) = pending
                .remaining
                .iter()
                .position(|candidate| *candidate == dependency)
            else {
                // This is an entry from an older dependency set superseded by a retry.
                continue;
            };
            pending.remaining.swap_remove(index);
            if pending.remaining.is_empty() {
                let wait_sequence = pending.wait_sequence;
                self.remaining_by_parent.remove(&parent);
                ready.push((wait_sequence, parent));
            }
        }
        ready.sort_by_key(|(wait_sequence, _)| *wait_sequence);
        ready.into_iter().map(|(_, parent)| parent).collect()
    }

    pub fn waiting_dependency_count(&self) -> usize {
        self.parents_by_dependency.len()
    }

    pub fn waiting_parent_count(&self) -> usize {
        self.remaining_by_parent.len()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_core::{DataId, QualifiedDataId};
    use ac2_dat::DatFileKind;

    use super::{
        GetAdmission, InFlightGetRegistry, OrderedSpillQueue, PreparationDependencyFanIn,
        PreparationStep, PreparationTransport, RETAIL_PREPARATION_FAST_QUEUE_CAPACITY,
    };
    use crate::ResourceKey;

    #[test]
    fn spill_backlog_never_allows_a_new_fast_item_to_overtake() {
        let mut transport = PreparationTransport::<u32, u32>::new(2);
        for value in 0..=2 {
            transport.enqueue_request(value);
        }
        assert!(transport.requests_have_spill_backlog());
        assert_eq!(
            transport.dequeue_request().map(|item| item.into_value()),
            Some(0)
        );
        transport.enqueue_request(3);

        assert_eq!(
            (0..3)
                .map(|_| transport.dequeue_request().map(|item| item.into_value()))
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(3)]
        );
        assert!(transport.dequeue_request().is_none());
    }

    #[test]
    fn replies_obey_the_same_spill_order() {
        let mut transport = PreparationTransport::<(), u32>::new(2);
        for value in 0..=2 {
            transport.enqueue_reply(value);
        }
        assert_eq!(
            transport.dequeue_reply().map(|item| item.into_value()),
            Some(0)
        );
        transport.enqueue_reply(3);
        assert_eq!(
            (0..3)
                .map(|_| transport.dequeue_reply().map(|item| item.into_value()))
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(3)]
        );
    }

    #[test]
    fn default_fast_capacity_matches_retail() {
        let queue = OrderedSpillQueue::<u32, super::PreparationRequestSequence>::default();
        assert_eq!(queue.fast_len(), 0);
        assert_eq!(RETAIL_PREPARATION_FAST_QUEUE_CAPACITY, 1024);
    }

    #[test]
    fn retail_sized_spill_keeps_older_tail_before_reopened_fast_slot() {
        let mut transport = PreparationTransport::<usize, ()>::default();
        for value in 0..=RETAIL_PREPARATION_FAST_QUEUE_CAPACITY {
            transport.enqueue_request(value);
        }
        assert!(transport.requests_have_spill_backlog());
        assert_eq!(
            transport.dequeue_request().map(|entry| entry.into_value()),
            Some(0)
        );
        transport.enqueue_request(RETAIL_PREPARATION_FAST_QUEUE_CAPACITY + 1);

        for expected in 1..=RETAIL_PREPARATION_FAST_QUEUE_CAPACITY + 1 {
            assert_eq!(
                transport.dequeue_request().map(|entry| entry.into_value()),
                Some(expected)
            );
        }
        assert!(transport.dequeue_request().is_none());
    }

    #[test]
    fn duplicate_gets_share_one_fifo_request() {
        let mut transport = PreparationTransport::<u32, ()>::new(2);
        let mut gets = InFlightGetRegistry::default();

        let first = gets.admit(7u32, "first", || transport.enqueue_request(7));
        let second = gets.admit(7u32, "second", || transport.enqueue_request(7));

        let GetAdmission::Queued(first_sequence) = first else {
            panic!("first exact get must receive a FIFO position");
        };
        let GetAdmission::Coalesced(second_sequence) = second else {
            panic!("duplicate exact get must coalesce");
        };
        assert_eq!(first_sequence, second_sequence);
        assert_eq!(transport.request_len(), 1);
        let consumers = gets
            .complete(&7)
            .expect("get should be in flight")
            .into_consumers();
        assert_eq!(consumers, vec!["first", "second"]);
    }

    #[test]
    fn dependency_continuation_is_requeued_at_the_tail() {
        let key = ResourceKey::new(
            DatFileKind::Portal,
            QualifiedDataId::new(DataId::new(0x2200_0001), 37),
        );
        let mut fan_in = PreparationDependencyFanIn::default();
        fan_in.wait_on("parent", [key]);
        let waiting = PreparationStep::<()>::WaitingOn(Arc::from([key]));
        assert!(matches!(waiting, PreparationStep::WaitingOn(_)));

        let mut transport = PreparationTransport::<&str, ()>::new(4);
        transport.enqueue_request("older");
        for continuation in fan_in.resolve(key) {
            transport.enqueue_request(continuation);
        }
        assert_eq!(
            transport.dequeue_request().map(|item| item.into_value()),
            Some("older")
        );
        assert_eq!(
            transport.dequeue_request().map(|item| item.into_value()),
            Some("parent")
        );
    }

    #[test]
    fn dependency_fan_in_waits_for_every_child_before_requeueing_parent() {
        let first = ResourceKey::new(
            DatFileKind::Portal,
            QualifiedDataId::new(DataId::new(0x2200_0002), 37),
        );
        let second = ResourceKey::new(
            DatFileKind::Portal,
            QualifiedDataId::new(DataId::new(0x2200_0003), 37),
        );
        let mut fan_in = PreparationDependencyFanIn::default();
        fan_in.wait_on("parent", [first, second, first]);

        assert_eq!(fan_in.waiting_dependency_count(), 2);
        assert_eq!(fan_in.waiting_parent_count(), 1);
        assert!(fan_in.resolve(first).is_empty());
        assert_eq!(fan_in.waiting_dependency_count(), 1);
        assert_eq!(fan_in.waiting_parent_count(), 1);
        assert_eq!(fan_in.resolve(second), vec!["parent"]);
        assert_eq!(fan_in.waiting_parent_count(), 0);
    }
}
