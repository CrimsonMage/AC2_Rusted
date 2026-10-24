use std::marker::PhantomData;
use std::sync::Arc;

use crate::{
    ResourceBodyDecode, ResourcePriority, ResourceRequestContext, ResourceRuntime, TypedResourceRef,
};

/// Resource-owned retained cursor for an ordered typed prefetch closure.
///
/// The producer owns root discovery and ordering. This runtime owns request-context lifetime,
/// duplicate generation detection, backpressure, and resumable submission.
#[derive(Debug)]
pub struct TypedPrefetchSequence<T, S = TypedResourceRef> {
    context: Option<ResourceRequestContext>,
    generation: Option<u64>,
    references: Option<Arc<[S]>>,
    next_reference: usize,
    marker: PhantomData<fn() -> T>,
}

impl<T, S> Default for TypedPrefetchSequence<T, S> {
    fn default() -> Self {
        Self {
            context: None,
            generation: None,
            references: None,
            next_reference: 0,
            marker: PhantomData,
        }
    }
}

impl<T: ResourceBodyDecode, S: Copy + Eq> TypedPrefetchSequence<T, S> {
    /// Replaces the retained closure when either source generation or ordered references change,
    /// then advances submission until the resource runtime applies backpressure.
    pub fn synchronize(
        &mut self,
        runtime: &mut ResourceRuntime,
        generation: Option<u64>,
        references: Arc<[S]>,
        priority: ResourcePriority,
        to_typed: impl Fn(S) -> TypedResourceRef + Copy,
    ) {
        if references.is_empty() {
            self.clear(runtime);
            return;
        }
        if self.generation != generation
            || self.references.as_ref().is_none_or(|current| {
                !Arc::ptr_eq(current, &references) && current.as_ref() != references.as_ref()
            })
        {
            self.clear(runtime);
            self.context = Some(runtime.begin_request_context(priority));
            self.generation = generation;
            self.references = Some(references);
        }
        self.pump(runtime, to_typed);
    }

    pub fn pump(
        &mut self,
        runtime: &mut ResourceRuntime,
        to_typed: impl Fn(S) -> TypedResourceRef,
    ) {
        let Some(context) = self.context else {
            return;
        };
        while let Some(reference) = self
            .references
            .as_ref()
            .and_then(|references| references.get(self.next_reference))
            .copied()
        {
            if !runtime.queue_prefetch_typed_for_context::<T>(context, to_typed(reference)) {
                return;
            }
            self.next_reference += 1;
        }
    }

    pub fn clear(&mut self, runtime: &mut ResourceRuntime) {
        if let Some(context) = self.context.take() {
            runtime.release_request_context(context);
        }
        self.generation = None;
        self.references = None;
        self.next_reference = 0;
    }

    pub fn queued_count(&self) -> usize {
        self.next_reference
    }

    pub fn is_complete(&self) -> bool {
        self.next_reference
            == self
                .references
                .as_ref()
                .map_or(0, |references| references.len())
    }
}
