use std::fmt;
use std::sync::{Arc, Mutex};

#[cfg(test)]
use crate::typed_cache::ResourceCacheConfig;
use crate::{
    PreparationExecutor, ResourceBodyDecode, ResourceCacheMaintenanceOutcome, ResourceCacheStats,
    ResourceDecodeScope, ResourceKey, ResourceResult, ResourceStore,
};

mod apply;
mod async_prefetch;
mod completion_order;
mod prefetch;
#[cfg(test)]
mod tests;

pub use async_prefetch::{
    ResourcePriority, ResourceRequestContext, ResourceRequestOutcome, ResourceRequestProgress,
    ResourceRuntimeEvent,
};
type TypedPrefetchExecutor = fn(&ResourceStore, ResourceKey) -> ResourceResult<bool>;
type ScopedTypedPrefetchExecutor =
    fn(&ResourceStore, &ResourceDecodeScope, ResourceKey) -> ResourceResult<bool>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRuntimeConfig {
    max_pending_prefetch: usize,
}

impl ResourceRuntimeConfig {
    pub const fn new() -> Self {
        Self {
            // The retail 1024-slot structures are transport acceleration, not a total admission
            // cap: an ordered spill backlog remains valid behind them. A caller may still impose
            // an explicit safety admission limit for a tool or a bounded test fixture.
            max_pending_prefetch: usize::MAX,
        }
    }

    pub const fn max_pending_prefetch(self) -> usize {
        self.max_pending_prefetch
    }

    pub const fn with_max_pending_prefetch(mut self, max_pending_prefetch: usize) -> Self {
        self.max_pending_prefetch = max_pending_prefetch;
        self
    }
}

impl Default for ResourceRuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum PrefetchRequest {
    GenericTyped(TypedPrefetchExecutor),
    ScopedTyped(ScopedTypedPrefetchExecutor),
}

impl PrefetchRequest {
    fn body<T>() -> Self
    where
        T: ResourceBodyDecode,
    {
        Self::GenericTyped(prefetch::prefetch_body::<T>)
    }

    fn scoped_body<T>() -> Self
    where
        T: ResourceBodyDecode,
    {
        Self::ScopedTyped(prefetch::prefetch_scoped_body::<T>)
    }

    fn execute(
        self,
        store: &ResourceStore,
        scope: Option<&ResourceDecodeScope>,
        key: ResourceKey,
    ) -> ResourceResult<bool> {
        match self {
            Self::GenericTyped(executor) => executor(store, key),
            Self::ScopedTyped(executor) => executor(
                store,
                scope.expect("scoped prefetch request must retain its decode scope"),
                key,
            ),
        }
    }
}

#[derive(Clone, Default)]
struct PreparationExecutorSlot {
    inner: Arc<Mutex<Option<Arc<dyn PreparationExecutor>>>>,
}

impl PreparationExecutorSlot {
    fn get(&self) -> Option<Arc<dyn PreparationExecutor>> {
        self.inner
            .lock()
            .expect("resource preparation executor slot poisoned")
            .clone()
    }

    fn replace(&self, executor: Option<Arc<dyn PreparationExecutor>>) {
        *self
            .inner
            .lock()
            .expect("resource preparation executor slot poisoned") = executor;
    }
}

impl fmt::Debug for PreparationExecutorSlot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparationExecutorSlot")
            .field("installed", &self.get().is_some())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ResourceRuntime {
    store: Arc<ResourceStore>,
    config: ResourceRuntimeConfig,
    coordinator: Arc<async_prefetch::AsyncPrefetchCoordinator>,
    executor: PreparationExecutorSlot,
}

impl ResourceRuntime {
    pub fn store(&self) -> &ResourceStore {
        &self.store
    }
    pub fn new(store: Arc<ResourceStore>) -> Self {
        Self::with_config(store, ResourceRuntimeConfig::default())
    }

    pub fn with_config(store: Arc<ResourceStore>, config: ResourceRuntimeConfig) -> Self {
        Self {
            coordinator: Arc::new(async_prefetch::AsyncPrefetchCoordinator::default()),
            store,
            config,
            executor: PreparationExecutorSlot::default(),
        }
    }

    /// Creates a resource runtime wired to the client-owned immutable-preparation executor.
    ///
    /// Production client composition supplies its single Worker-2 executor here. The plain
    /// constructors intentionally install no hidden worker or inline fallback.
    pub fn with_preparation_executor(
        store: Arc<ResourceStore>,
        executor: Arc<dyn PreparationExecutor>,
    ) -> Self {
        Self::with_config_and_preparation_executor(
            store,
            ResourceRuntimeConfig::default(),
            executor,
        )
    }

    pub fn with_config_and_preparation_executor(
        store: Arc<ResourceStore>,
        config: ResourceRuntimeConfig,
        executor: Arc<dyn PreparationExecutor>,
    ) -> Self {
        let runtime = Self::with_config(store, config);
        runtime.set_preparation_executor(executor);
        runtime
    }

    /// Replaces the client-injected immutable-preparation executor without changing resource
    /// request ownership or FIFO state. This is used while a client session is being composed.
    pub fn set_preparation_executor(&self, executor: Arc<dyn PreparationExecutor>) {
        self.executor.replace(Some(executor));
        self.schedule_preparation();
    }

    /// Removes the executor during controlled session teardown. Queued requests remain typed
    /// resource-owned state and resume only after another executor is explicitly installed.
    pub fn clear_preparation_executor(&self) {
        self.executor.replace(None);
    }

    pub fn has_preparation_executor(&self) -> bool {
        self.executor.get().is_some()
    }

    pub(crate) fn schedule_preparation(&self) {
        let Some(executor) = self.executor.get() else {
            return;
        };
        self.coordinator
            .schedule_next(Arc::clone(&self.store), executor);
    }

    pub const fn config(&self) -> ResourceRuntimeConfig {
        self.config
    }

    pub fn cache_stats(&self) -> ResourceCacheStats {
        self.store.cache_stats()
    }

    /// Returns a non-blocking typed-cache snapshot for diagnostics.
    pub fn try_cache_stats(&self) -> Option<ResourceCacheStats> {
        self.store.try_cache_stats()
    }

    #[cfg(test)]
    pub(crate) fn cache_config(&self) -> ResourceCacheConfig {
        self.store.cache_config()
    }

    /// Processes one bounded event-driven retail free-list pass without waiting for the cache.
    pub fn try_maintain_cache(&self) -> ResourceCacheMaintenanceOutcome {
        self.store.try_maintain_typed_cache()
    }

    /// Drops all retained typed objects while preserving the selected cache capacity.
    pub fn clear_cache(&self) {
        self.store.clear_typed_cache();
    }
}
