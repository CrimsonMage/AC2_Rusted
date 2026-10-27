use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ac2_core::DataId;
use ac2_dat::DatFileKind;

use crate::{
    ResourceKey, ResourceOwnershipLane, ResourceOwnershipStats, ResourceScopeOverlapStats,
};

pub(crate) type CacheValue = Arc<dyn Any + Send + Sync>;

#[path = "typed_cache_maintenance.rs"]
mod maintenance;
#[path = "typed_cache_state.rs"]
mod state;

pub use maintenance::{
    ResourceCacheFamilyMaintenanceReport, ResourceCacheMaintenanceOutcome,
    ResourceCacheMaintenanceReport,
};
pub(crate) use state::ResourceCachePin;
use state::{CacheEntry, EntryLifecycle, TypedCacheState};

/// One active checkout of a typed cache entry.
///
/// Clones share the same checkout token, so only the final clone releases the entry back to the
/// retail free list. A separate cache lookup creates a separate checkout.
pub struct CachedResource<T> {
    value: Arc<T>,
    lifecycle: Arc<EntryLifecycle>,
}

impl<T> CachedResource<T> {
    fn new(value: Arc<T>, lifecycle: Arc<EntryLifecycle>) -> Self {
        Self { value, lifecycle }
    }

    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        Arc::ptr_eq(&left.value, &right.value)
    }

    /// Observes immutable allocation identity without extending the resource checkout or
    /// retaining its payload. Derived-data caches use this to reject replaced source values.
    pub fn downgrade(&self) -> std::sync::Weak<T> {
        Arc::downgrade(&self.value)
    }

    pub(crate) fn value_arc(&self) -> Arc<T> {
        Arc::clone(&self.value)
    }

    pub(crate) fn pin(&self) -> ResourceCachePin {
        self.lifecycle.acquire_pin();
        ResourceCachePin::new(Arc::clone(&self.lifecycle))
    }
}

impl<T> Clone for CachedResource<T> {
    fn clone(&self) -> Self {
        self.lifecycle.acquire();
        Self {
            value: Arc::clone(&self.value),
            lifecycle: Arc::clone(&self.lifecycle),
        }
    }
}

impl<T> Drop for CachedResource<T> {
    fn drop(&mut self) {
        self.lifecycle.release();
    }
}

impl<T> Deref for CachedResource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AsRef<T> for CachedResource<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: fmt::Debug> fmt::Debug for CachedResource<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CachedResource")
            .field(&self.value)
            .finish()
    }
}

impl<T: PartialEq> PartialEq for CachedResource<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for CachedResource<T> {}

/// Nonblocking observation of one already-decoded typed cache entry.
///
/// `Busy` is deliberately distinct from `Miss`: frame-budgeted consumers can keep their last
/// published product while cache mutation is in progress instead of retiring valid residency.
#[derive(Debug)]
pub enum ResourceCacheObservation<T> {
    Hit(CachedResource<T>),
    Miss,
    Busy,
}

/// Bytes of decoded values nobody holds that the cache keeps for reuse by default.
pub const DEFAULT_FREE_CACHE_BUDGET_BYTES: usize = 256 * 1024 * 1024;

/// Diagnostic window: a value decoded again this soon after its eviction counts as a
/// reinsertion (the budget was too small for the working set).
const REINSERTION_WINDOW: Duration = Duration::from_secs(30);

/// The cache's memory policy. Replaces retail `DBOCache`'s per-type free-object counts and its
/// 30 second idle aging (`DBOCache::UseTime`): values are shared while held, and once released
/// stay only while the free bytes fit `free_byte_budget`, oldest released first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceCacheConfig {
    free_byte_budget: usize,
}

impl ResourceCacheConfig {
    pub(crate) const fn new(free_byte_budget: usize) -> Self {
        Self { free_byte_budget }
    }

    pub(crate) const fn free_byte_budget(self) -> usize {
        self.free_byte_budget
    }

    /// Free bytes an over-budget pass reaps down to, so one release past the budget does not
    /// cost one pass per insert.
    pub(crate) const fn free_byte_low_water(self) -> usize {
        self.free_byte_budget - self.free_byte_budget / 8
    }
}

impl Default for ResourceCacheConfig {
    fn default() -> Self {
        Self::new(DEFAULT_FREE_CACHE_BUDGET_BYTES)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceCacheStats {
    entry_count: usize,
    retained_bytes: usize,
    active_entry_count: usize,
    active_retained_bytes: usize,
    free_entry_count: usize,
    free_retained_bytes: usize,
    pinned_entry_count: usize,
    pinned_retained_bytes: usize,
    hit_count: u64,
    miss_count: u64,
    eviction_count: u64,
    retention_eviction_count: u64,
    eviction_reinsertion_count: u64,
}

impl ResourceCacheStats {
    pub const fn entry_count(self) -> usize {
        self.entry_count
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    /// Entries with at least one checkout, including explicitly pinned entries.
    pub const fn active_entry_count(self) -> usize {
        self.active_entry_count
    }

    pub const fn active_retained_bytes(self) -> usize {
        self.active_retained_bytes
    }

    pub const fn free_entry_count(self) -> usize {
        self.free_entry_count
    }

    pub const fn free_retained_bytes(self) -> usize {
        self.free_retained_bytes
    }

    /// Explicitly pinned entries are a subset of the active lane and must not be added to it.
    pub const fn pinned_entry_count(self) -> usize {
        self.pinned_entry_count
    }

    pub const fn pinned_retained_bytes(self) -> usize {
        self.pinned_retained_bytes
    }

    pub const fn hit_count(self) -> u64 {
        self.hit_count
    }

    pub const fn miss_count(self) -> u64 {
        self.miss_count
    }

    pub const fn eviction_count(self) -> u64 {
        self.eviction_count
    }

    pub const fn retention_eviction_count(self) -> u64 {
        self.retention_eviction_count
    }

    /// Number of typed keys decoded back into the cache within the diagnostic correlation
    /// window after retail retention evicted them.
    pub const fn eviction_reinsertion_count(self) -> u64 {
        self.eviction_reinsertion_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TypedCacheKey {
    key: ResourceKey,
    type_id: TypeId,
}

impl TypedCacheKey {
    pub(crate) fn of<T: 'static>(key: ResourceKey) -> Self {
        Self {
            key,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Narrow diagnostic identity for an in-flight typed decode. The concrete Rust type remains
    /// internal cache metadata; resource ownership is identified by its mounted file, DID, and
    /// DBObj type instead of exposing the cache map itself.
    pub(crate) const fn resource_key(self) -> ResourceKey {
        self.key
    }
}

/// Strong decoded values owned only by one caller-defined active scope.
///
/// Unlike `TypedResourceCache`, this has no eviction policy and no session-wide retention:
/// dropping the final scope handle releases every value that the scope itself owns.  The
/// resource store uses the same typed key as its generic cache so one scope never confuses two
/// distinct Rust decoder outputs for the same DID.
#[derive(Debug, Clone)]
pub struct ResourceDecodeScope {
    identity: u64,
    entries: Arc<Mutex<HashMap<TypedCacheKey, ScopedCacheEntry>>>,
}

#[derive(Debug, Clone)]
struct ScopedCacheEntry {
    value: CacheValue,
    retained_bytes: usize,
}

impl ResourceDecodeScope {
    pub(crate) fn identity(&self) -> u64 {
        self.identity
    }

    pub fn resource_count(&self) -> usize {
        self.entries
            .lock()
            .expect("resource decode scope poisoned")
            .len()
    }

    /// Returns per-type retained-byte observation for this caller-owned scope.
    ///
    /// This does not refresh, retain, or evict any value.
    pub fn ownership_stats(&self) -> ResourceOwnershipStats {
        let entries = self.entries.lock().expect("resource decode scope poisoned");
        ResourceOwnershipStats::from_entries(
            ResourceOwnershipLane::DecodeScope,
            entries
                .iter()
                .map(|(key, entry)| (key.key.type_id(), entry.retained_bytes)),
        )
    }

    /// Returns retained-byte observation only when no scope mutation is in progress.
    pub fn try_ownership_stats(&self) -> Option<ResourceOwnershipStats> {
        self.entries.try_lock().ok().map(|entries| {
            ResourceOwnershipStats::from_entries(
                ResourceOwnershipLane::DecodeScope,
                entries
                    .iter()
                    .map(|(key, entry)| (key.key.type_id(), entry.retained_bytes)),
            )
        })
    }

    pub(crate) fn get<T>(&self, key: ResourceKey) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.entries
            .lock()
            .expect("resource decode scope poisoned")
            .get(&TypedCacheKey::of::<T>(key))
            .map(|entry| Arc::clone(&entry.value))
            .and_then(downcast_cache_value)
    }

    pub(crate) fn insert<T>(&self, key: ResourceKey, value: Arc<T>, retained_bytes: usize) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        let typed_key = TypedCacheKey::of::<T>(key);
        let mut entries = self.entries.lock().expect("resource decode scope poisoned");
        if let Some(existing) = entries
            .get(&typed_key)
            .map(|entry| Arc::clone(&entry.value))
            .and_then(downcast_cache_value)
        {
            return existing;
        }
        entries.insert(
            typed_key,
            ScopedCacheEntry {
                value: value.clone(),
                retained_bytes,
            },
        );
        value
    }
}

impl Default for ResourceDecodeScope {
    fn default() -> Self {
        static NEXT_SCOPE_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            identity: NEXT_SCOPE_ID.fetch_add(1, Ordering::Relaxed).max(1),
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedResourceCache {
    state: Arc<TypedCacheState>,
}

impl Default for TypedResourceCache {
    fn default() -> Self {
        Self::new(ResourceCacheConfig::default())
    }
}

impl TypedResourceCache {
    pub(crate) fn new(config: ResourceCacheConfig) -> Self {
        Self {
            state: Arc::new(TypedCacheState::new(config)),
        }
    }

    #[cfg(test)]
    pub(crate) fn config(&self) -> ResourceCacheConfig {
        self.state.config()
    }

    pub fn stats(&self) -> ResourceCacheStats {
        self.state.stats()
    }

    /// Returns a snapshot only when it can be observed without waiting for a decode or eviction.
    pub fn try_stats(&self) -> Option<ResourceCacheStats> {
        self.state.try_stats()
    }

    pub(crate) fn ownership_stats(&self) -> ResourceOwnershipStats {
        let mut entries = Vec::new();
        for (_, family) in self.state.families() {
            let family = family.read().expect("typed resource cache family poisoned");
            entries.extend(
                family
                    .entries
                    .iter()
                    .map(|(key, entry)| (key.key.type_id(), entry.retained_bytes)),
            );
        }
        ResourceOwnershipStats::from_entries(ResourceOwnershipLane::GenericTypedCache, entries)
    }

    pub(crate) fn try_ownership_stats(&self) -> Option<ResourceOwnershipStats> {
        let mut entries = Vec::new();
        for (_, family) in self.state.try_families()? {
            let family = family.try_read().ok()?;
            entries.extend(
                family
                    .entries
                    .iter()
                    .map(|(key, entry)| (key.key.type_id(), entry.retained_bytes)),
            );
        }
        Some(ResourceOwnershipStats::from_entries(
            ResourceOwnershipLane::GenericTypedCache,
            entries,
        ))
    }

    /// Compares generic cache ownership against one scope without waiting for either owner.
    /// This diagnostic deliberately makes no claim about physical duplication when the two
    /// entries share the same erased `Arc` value.
    pub(crate) fn try_scope_overlap_stats(
        &self,
        scope: &ResourceDecodeScope,
    ) -> Option<ResourceScopeOverlapStats> {
        let families = self
            .state
            .try_families()?
            .into_iter()
            .collect::<HashMap<_, _>>();
        let scoped_entries = scope.entries.try_lock().ok()?;
        let mut overlaps = Vec::new();
        for (key, scoped) in scoped_entries.iter() {
            let Some(family) = families.get(&key.key.type_id()) else {
                continue;
            };
            let family = family.try_read().ok()?;
            if let Some(generic) = family.entries.get(key) {
                overlaps.push((
                    key.key.type_id(),
                    generic.retained_bytes,
                    scoped.retained_bytes,
                    Arc::ptr_eq(&generic.value, &scoped.value),
                ));
            }
        }
        Some(ResourceScopeOverlapStats::from_entries(overlaps))
    }

    /// Borrows the cache's current typed value without changing its LRU or cache statistics.
    pub(crate) fn peek<T>(&self, key: ResourceKey) -> Option<CachedResource<T>>
    where
        T: Send + Sync + 'static,
    {
        let typed_key = TypedCacheKey::of::<T>(key);
        let family = self.state.family(key.type_id())?;
        let mut family = family
            .write()
            .expect("typed resource cache family poisoned");
        let entry = family.entries.get(&typed_key)?;
        let value = downcast_cache_value(Arc::clone(&entry.value))?;
        let checkout = entry.acquire_lifecycle();
        family.mark_active(typed_key);
        Some(CachedResource::new(value, checkout))
    }

    /// Borrows the cache only when no decode or eviction mutation is in progress.
    ///
    /// A cold-or-contended result is intentionally indistinguishable from a miss: callers that
    /// must preserve a frame budget can leave the work with the resource preparation owner
    /// instead of waiting behind the cache writer.
    pub(crate) fn try_peek<T>(&self, key: ResourceKey) -> Option<CachedResource<T>>
    where
        T: Send + Sync + 'static,
    {
        match self.try_observe(key) {
            ResourceCacheObservation::Hit(value) => Some(value),
            ResourceCacheObservation::Miss | ResourceCacheObservation::Busy => None,
        }
    }

    pub(crate) fn try_observe<T>(&self, key: ResourceKey) -> ResourceCacheObservation<T>
    where
        T: Send + Sync + 'static,
    {
        let typed_key = TypedCacheKey::of::<T>(key);
        let Ok(family) = self.state.try_family(key.type_id()) else {
            return ResourceCacheObservation::Busy;
        };
        let Some(family) = family else {
            return ResourceCacheObservation::Miss;
        };
        let Ok(mut family) = family.try_write() else {
            return ResourceCacheObservation::Busy;
        };
        let Some(entry) = family.entries.get(&typed_key) else {
            return ResourceCacheObservation::Miss;
        };
        let Some(value) = downcast_cache_value(Arc::clone(&entry.value)) else {
            return ResourceCacheObservation::Miss;
        };
        let checkout = entry.acquire_lifecycle();
        family.mark_active(typed_key);
        ResourceCacheObservation::Hit(CachedResource::new(value, checkout))
    }

    pub fn get<T>(&self, key: ResourceKey) -> Option<CachedResource<T>>
    where
        T: Send + Sync + 'static,
    {
        let typed_key = TypedCacheKey::of::<T>(key);
        let Some(family) = self.state.family(key.type_id()) else {
            self.state.record_miss();
            return None;
        };
        let mut family = family
            .write()
            .expect("typed resource cache family poisoned");
        let (value, checkout) = match family.entries.get(&typed_key) {
            Some(entry) => {
                let Some(value) = downcast_cache_value(entry.value.clone()) else {
                    self.state.record_miss();
                    return None;
                };
                let checkout = entry.acquire_lifecycle();
                (value, checkout)
            }
            None => {
                self.state.record_miss();
                return None;
            }
        };
        family.mark_active(typed_key);
        self.state.record_hit();
        Some(CachedResource::new(value, checkout))
    }

    /// Inserts a decoded resource with the backing byte count it retains. The cache owns the
    /// eviction policy; callers retain normal `Arc` identity whether or not this value fits.
    pub(crate) fn insert_with_retained_bytes<T>(
        &self,
        key: ResourceKey,
        value: Arc<T>,
        retained_bytes: usize,
    ) -> CachedResource<T>
    where
        T: Send + Sync + 'static,
    {
        let typed_key = TypedCacheKey::of::<T>(key);
        let family = self.state.family_or_insert(key.type_id());
        let mut family = family
            .write()
            .expect("typed resource cache family poisoned");
        let accessed_at = Instant::now();

        if let Some(existing) = family.entries.get(&typed_key) {
            let cached_value = existing.value.clone();
            let checkout = existing.acquire_lifecycle();
            family.mark_active(typed_key);
            return downcast_cache_value(cached_value)
                .map(|value| CachedResource::new(value, checkout))
                .expect("typed cache key must downcast to its declared value type");
        }

        let reinserted_after_eviction =
            family.take_recent_eviction(typed_key, accessed_at, REINSERTION_WINDOW);
        family.entries.insert(
            typed_key,
            CacheEntry::new(&self.state, typed_key, value.clone(), retained_bytes),
        );
        family.retained_bytes = family.retained_bytes.saturating_add(retained_bytes);
        let checkout = family.entries[&typed_key].acquire_lifecycle();
        self.state.record_insert(retained_bytes);
        if reinserted_after_eviction {
            self.state.record_eviction_reinsertion();
        }
        drop(family);
        self.trim_if_over_budget();
        CachedResource::new(value, checkout)
    }

    #[cfg(test)]
    pub fn evict<T>(&self, key: ResourceKey) -> bool
    where
        T: Send + Sync + 'static,
    {
        let typed_key = TypedCacheKey::of::<T>(key);
        let Some(family) = self.state.family(key.type_id()) else {
            return false;
        };
        let mut family = family
            .write()
            .expect("typed resource cache family poisoned");
        let Some(entry) = family.entries.get(&typed_key) else {
            return false;
        };
        if !entry.is_free() {
            return false;
        }
        let detached = family.detach_entry(typed_key);
        let removed = detached.is_some();
        if let Some(entry) = detached.as_ref() {
            self.state.record_eviction(entry.retained_bytes, false);
        }
        drop(family);
        drop(detached);
        removed
    }

    /// Retains one already-resident object against cache eviction without exposing its Rust
    /// implementation type to a dependency-plan consumer.
    pub(crate) fn retain_resource(&self, key: ResourceKey) -> Vec<ResourceCachePin> {
        let Some(family) = self.state.family(key.type_id()) else {
            return Vec::new();
        };
        let mut family = family
            .write()
            .expect("typed resource cache family poisoned");
        let typed_keys = family
            .entries
            .keys()
            .filter_map(|typed_key| (typed_key.key == key).then_some(*typed_key))
            .collect::<Vec<_>>();
        let pins = typed_keys
            .iter()
            .map(|typed_key| ResourceCachePin::acquire(&family.entries[typed_key]))
            .collect();
        for typed_key in typed_keys {
            family.mark_active(typed_key);
        }
        pins
    }

    /// Takes queued releases and reaps free values over the byte budget, only when no decode or
    /// cache mutation owns the cache lock: `Busy` means the pass was skipped rather than waiting
    /// behind live work. Inserts run this themselves once the cache could be over budget.
    pub fn try_maintain(&self) -> ResourceCacheMaintenanceOutcome {
        self.try_maintain_report()
    }

    pub fn clear(&self) {
        let mut detached = Vec::new();
        for (_, family) in self.state.families() {
            let mut family = family
                .write()
                .expect("typed resource cache family poisoned");
            let removable = family
                .entries
                .iter()
                .filter_map(|(key, entry)| (!entry.is_pinned()).then_some(*key))
                .collect::<Vec<_>>();
            for key in removable {
                if let Some(entry) = family.detach_entry(key) {
                    self.state.record_eviction(entry.retained_bytes, false);
                    detached.push(entry);
                }
            }
        }
        self.state.reset_observation_stats();
        drop(detached);
    }

    pub fn invalidate_resource(&self, file_kind: DatFileKind, did: DataId) -> usize {
        let mut detached = Vec::new();
        for (_, family) in self.state.families() {
            let mut family = family
                .write()
                .expect("typed resource cache family poisoned");
            let removable = family
                .entries
                .iter()
                .filter(|(typed_key, entry)| {
                    typed_key.key.file_kind() == file_kind
                        && typed_key.key.data_id() == did
                        && !entry.is_pinned()
                })
                .map(|(key, _)| *key)
                .collect::<Vec<_>>();
            for key in removable {
                if let Some(entry) = family.detach_entry(key) {
                    self.state.record_eviction(entry.retained_bytes, false);
                    detached.push(entry);
                }
            }
        }
        let removed_count = detached.len();
        drop(detached);
        removed_count
    }
}

fn downcast_cache_value<T>(value: CacheValue) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    Arc::downcast::<T>(value).ok()
}

#[cfg(test)]
#[path = "typed_cache_tests.rs"]
mod tests;
