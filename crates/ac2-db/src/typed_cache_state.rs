use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::Instant;

use concurrent_queue::ConcurrentQueue;

use super::{CacheValue, ResourceCacheConfig, ResourceCacheStats, TypedCacheKey};

const RECENT_EVICTION_CAPACITY: usize = 64;

#[derive(Debug)]
pub(crate) struct ResourceCachePin {
    lifecycle: Arc<EntryLifecycle>,
}

impl ResourceCachePin {
    pub(super) fn new(lifecycle: Arc<EntryLifecycle>) -> Self {
        Self { lifecycle }
    }

    pub(super) fn acquire(entry: &CacheEntry) -> Self {
        entry.lifecycle.acquire_pin();
        Self::new(Arc::clone(&entry.lifecycle))
    }
}

impl Clone for ResourceCachePin {
    fn clone(&self) -> Self {
        self.lifecycle.acquire_pin();
        Self::new(Arc::clone(&self.lifecycle))
    }
}

impl Drop for ResourceCachePin {
    fn drop(&mut self) {
        self.lifecycle.release_pin();
    }
}

#[derive(Debug)]
pub(super) struct EntryLifecycle {
    state: Weak<TypedCacheState>,
    self_reference: Weak<EntryLifecycle>,
    key: TypedCacheKey,
    active_checkouts: AtomicUsize,
    explicit_pins: AtomicUsize,
    epoch: AtomicU64,
    pending_release: Mutex<PendingRelease>,
}

impl EntryLifecycle {
    pub(super) fn acquire(&self) {
        let previous = self.active_checkouts.fetch_add(1, Ordering::AcqRel);
        if previous == 0 {
            self.epoch.fetch_add(1, Ordering::AcqRel);
        }
    }

    pub(super) fn release(&self) {
        if self.active_checkouts.fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }
        let Some(state) = self.state.upgrade() else {
            return;
        };
        let mut pending = self
            .pending_release
            .lock()
            .expect("typed resource lifecycle release poisoned");
        pending.event = Some(ReleaseEvent {
            key: self.key,
            epoch: self.epoch.load(Ordering::Acquire),
            released_at: Instant::now(),
        });
        if pending.queued {
            return;
        }
        pending.queued = true;
        drop(pending);
        let token = self.release_token();
        if state.release_events.push(token).is_err() {
            self.pending_release
                .lock()
                .expect("typed resource lifecycle release poisoned")
                .queued = false;
        }
    }

    pub(super) fn acquire_pin(&self) {
        self.explicit_pins.fetch_add(1, Ordering::AcqRel);
        self.acquire();
    }

    fn release_pin(&self) {
        self.explicit_pins.fetch_sub(1, Ordering::AcqRel);
        self.release();
    }

    pub(super) fn take_pending_release(&self) -> Option<ReleaseEvent> {
        self.pending_release
            .lock()
            .expect("typed resource lifecycle release poisoned")
            .event
            .take()
    }

    pub(super) fn finish_release_token(&self, state: &TypedCacheState) {
        let mut pending = self
            .pending_release
            .lock()
            .expect("typed resource lifecycle release poisoned");
        if pending.event.is_none() {
            pending.queued = false;
            return;
        }
        let token = self.release_token();
        drop(pending);
        if state.release_events.push(token).is_err() {
            self.pending_release
                .lock()
                .expect("typed resource lifecycle release poisoned")
                .queued = false;
        }
    }

    fn release_token(&self) -> ReleaseToken {
        ReleaseToken {
            key: self.key,
            lifecycle: self.self_reference.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct PendingRelease {
    event: Option<ReleaseEvent>,
    queued: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ReleaseToken {
    pub(super) key: TypedCacheKey,
    pub(super) lifecycle: Weak<EntryLifecycle>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ReleaseEvent {
    pub(super) key: TypedCacheKey,
    pub(super) epoch: u64,
    pub(super) released_at: Instant,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FreeTicket {
    pub(super) key: TypedCacheKey,
    pub(super) epoch: u64,
    pub(super) released_at: Instant,
}

#[derive(Debug, Clone, Copy)]
struct RecentEviction {
    key: TypedCacheKey,
    at: Instant,
}

#[derive(Debug)]
pub(super) struct CacheEntry {
    pub(super) value: CacheValue,
    pub(super) retained_bytes: usize,
    lifecycle: Arc<EntryLifecycle>,
    free_previous: Option<TypedCacheKey>,
    free_next: Option<TypedCacheKey>,
    free_ticket: Option<FreeTicket>,
}

impl CacheEntry {
    pub(super) fn new(
        state: &Arc<TypedCacheState>,
        key: TypedCacheKey,
        value: CacheValue,
        retained_bytes: usize,
    ) -> Self {
        Self {
            value,
            retained_bytes,
            lifecycle: Arc::new_cyclic(|self_reference| EntryLifecycle {
                state: Arc::downgrade(state),
                self_reference: self_reference.clone(),
                key,
                active_checkouts: AtomicUsize::new(0),
                explicit_pins: AtomicUsize::new(0),
                epoch: AtomicU64::new(0),
                pending_release: Mutex::new(PendingRelease::default()),
            }),
            free_previous: None,
            free_next: None,
            free_ticket: None,
        }
    }

    pub(super) fn acquire_lifecycle(&self) -> Arc<EntryLifecycle> {
        self.lifecycle.acquire();
        Arc::clone(&self.lifecycle)
    }

    pub(super) fn is_free(&self) -> bool {
        self.lifecycle.active_checkouts.load(Ordering::Acquire) == 0
    }

    pub(super) fn is_pinned(&self) -> bool {
        self.lifecycle.explicit_pins.load(Ordering::Acquire) != 0
    }

    pub(super) fn matches_free_ticket(&self, ticket: FreeTicket) -> bool {
        self.is_free() && self.lifecycle.epoch.load(Ordering::Acquire) == ticket.epoch
    }

    pub(super) fn matches_lifecycle(&self, lifecycle: &Arc<EntryLifecycle>) -> bool {
        Arc::ptr_eq(&self.lifecycle, lifecycle)
    }
}

#[derive(Debug, Default)]
pub(super) struct CacheFamilyState {
    pub(super) entries: HashMap<TypedCacheKey, CacheEntry>,
    free_head: Option<TypedCacheKey>,
    free_tail: Option<TypedCacheKey>,
    free_count: usize,
    free_retained_bytes: usize,
    pub(super) retained_bytes: usize,
    recent_evictions: VecDeque<RecentEviction>,
}

impl CacheFamilyState {
    pub(super) fn lifecycle_stats(&self) -> (usize, usize, usize, usize) {
        self.entries.values().fold(
            (0usize, 0usize, 0usize, 0usize),
            |(free_count, free_bytes, pinned_count, pinned_bytes), entry| {
                (
                    free_count + usize::from(entry.is_free()),
                    free_bytes.saturating_add(usize::from(entry.is_free()) * entry.retained_bytes),
                    pinned_count + usize::from(entry.is_pinned()),
                    pinned_bytes
                        .saturating_add(usize::from(entry.is_pinned()) * entry.retained_bytes),
                )
            },
        )
    }
    pub(super) fn detach_entry(&mut self, key: TypedCacheKey) -> Option<CacheEntry> {
        self.mark_active(key);
        let entry = self.entries.remove(&key)?;
        self.retained_bytes = self.retained_bytes.saturating_sub(entry.retained_bytes);
        Some(entry)
    }

    pub(super) fn mark_active(&mut self, key: TypedCacheKey) {
        let Some((previous, next, retained_bytes)) = self.entries.get(&key).and_then(|entry| {
            entry
                .free_ticket
                .map(|_| (entry.free_previous, entry.free_next, entry.retained_bytes))
        }) else {
            return;
        };
        if let Some(previous) = previous {
            if let Some(entry) = self.entries.get_mut(&previous) {
                entry.free_next = next;
            }
        } else {
            self.free_head = next;
        }
        if let Some(next) = next {
            if let Some(entry) = self.entries.get_mut(&next) {
                entry.free_previous = previous;
            }
        } else {
            self.free_tail = previous;
        }
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.free_previous = None;
            entry.free_next = None;
            entry.free_ticket = None;
        }
        self.free_count = self.free_count.saturating_sub(1);
        self.free_retained_bytes = self.free_retained_bytes.saturating_sub(retained_bytes);
    }

    pub(super) fn free_count(&self) -> usize {
        self.free_count
    }

    pub(super) fn free_retained_bytes(&self) -> usize {
        self.free_retained_bytes
    }

    pub(super) fn accept_release(
        &mut self,
        event: ReleaseEvent,
        lifecycle: &Arc<EntryLifecycle>,
    ) -> bool {
        let ticket = FreeTicket {
            key: event.key,
            epoch: event.epoch,
            released_at: event.released_at,
        };
        let Some(entry) = self.entries.get(&event.key) else {
            return false;
        };
        if !entry.matches_lifecycle(lifecycle) || !entry.matches_free_ticket(ticket) {
            return false;
        }
        let retained_bytes = entry.retained_bytes;
        self.mark_active(event.key);
        let previous = self.free_tail;
        if let Some(previous) = previous {
            if let Some(entry) = self.entries.get_mut(&previous) {
                entry.free_next = Some(event.key);
            }
        } else {
            self.free_head = Some(event.key);
        }
        if let Some(entry) = self.entries.get_mut(&event.key) {
            entry.free_previous = previous;
            entry.free_next = None;
            entry.free_ticket = Some(ticket);
        }
        self.free_tail = Some(event.key);
        self.free_count = self.free_count.saturating_add(1);
        self.free_retained_bytes = self.free_retained_bytes.saturating_add(retained_bytes);
        true
    }

    pub(super) fn peek_oldest_free(&self) -> Option<(FreeTicket, usize)> {
        let key = self.free_head?;
        let entry = self.entries.get(&key)?;
        Some((entry.free_ticket?, entry.retained_bytes))
    }

    pub(super) fn oldest_free(&mut self) -> Option<FreeTicket> {
        let key = self.free_head?;
        let ticket = self.entries.get(&key)?.free_ticket?;
        self.mark_active(key);
        Some(ticket)
    }

    pub(super) fn restore_oldest(&mut self, ticket: FreeTicket) {
        let retained_bytes = self
            .entries
            .get(&ticket.key)
            .map(|entry| entry.retained_bytes)
            .unwrap_or_default();
        let next = self.free_head;
        if let Some(next) = next {
            if let Some(entry) = self.entries.get_mut(&next) {
                entry.free_previous = Some(ticket.key);
            }
        } else {
            self.free_tail = Some(ticket.key);
        }
        if let Some(entry) = self.entries.get_mut(&ticket.key) {
            entry.free_previous = None;
            entry.free_next = next;
            entry.free_ticket = Some(ticket);
        }
        self.free_head = Some(ticket.key);
        self.free_count = self.free_count.saturating_add(1);
        self.free_retained_bytes = self.free_retained_bytes.saturating_add(retained_bytes);
    }

    pub(super) fn record_retention_eviction(&mut self, key: TypedCacheKey, at: Instant) {
        if self.recent_evictions.len() == RECENT_EVICTION_CAPACITY {
            self.recent_evictions.pop_front();
        }
        self.recent_evictions.push_back(RecentEviction { key, at });
    }

    pub(super) fn take_recent_eviction(
        &mut self,
        key: TypedCacheKey,
        now: Instant,
        window: std::time::Duration,
    ) -> bool {
        while self
            .recent_evictions
            .front()
            .is_some_and(|entry| now.saturating_duration_since(entry.at) > window)
        {
            self.recent_evictions.pop_front();
        }
        let Some(index) = self
            .recent_evictions
            .iter()
            .position(|entry| entry.key == key)
        else {
            return false;
        };
        self.recent_evictions.remove(index);
        true
    }
}

#[derive(Debug)]
pub(super) struct CacheDirectory {
    pub(super) config: ResourceCacheConfig,
    pub(super) families: BTreeMap<u32, Arc<RwLock<CacheFamilyState>>>,
}

#[derive(Debug)]
pub(super) struct TypedCacheState {
    pub(super) directory: RwLock<CacheDirectory>,
    pub(super) release_events: ConcurrentQueue<ReleaseToken>,
    entry_count: AtomicUsize,
    retained_bytes: AtomicUsize,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    eviction_count: AtomicU64,
    retention_eviction_count: AtomicU64,
    eviction_reinsertion_count: AtomicU64,
    free_pressure_active: AtomicBool,
}

impl TypedCacheState {
    pub(super) fn new(config: ResourceCacheConfig) -> Self {
        Self {
            directory: RwLock::new(CacheDirectory {
                config,
                families: BTreeMap::new(),
            }),
            release_events: ConcurrentQueue::unbounded(),
            entry_count: AtomicUsize::new(0),
            retained_bytes: AtomicUsize::new(0),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            retention_eviction_count: AtomicU64::new(0),
            eviction_reinsertion_count: AtomicU64::new(0),
            free_pressure_active: AtomicBool::new(false),
        }
    }

    #[cfg(test)]
    pub(super) fn config(&self) -> ResourceCacheConfig {
        self.directory
            .read()
            .expect("typed resource cache directory poisoned")
            .config
    }

    pub(super) fn try_config(&self) -> Option<ResourceCacheConfig> {
        self.directory
            .try_read()
            .ok()
            .map(|directory| directory.config)
    }

    pub(super) fn family(&self, resource_type: u32) -> Option<Arc<RwLock<CacheFamilyState>>> {
        self.directory
            .read()
            .expect("typed resource cache directory poisoned")
            .families
            .get(&resource_type)
            .cloned()
    }

    pub(super) fn try_family(
        &self,
        resource_type: u32,
    ) -> Result<Option<Arc<RwLock<CacheFamilyState>>>, ()> {
        self.directory
            .try_read()
            .map(|directory| directory.families.get(&resource_type).cloned())
            .map_err(|_| ())
    }

    pub(super) fn family_or_insert(&self, resource_type: u32) -> Arc<RwLock<CacheFamilyState>> {
        if let Some(family) = self.family(resource_type) {
            return family;
        }
        Arc::clone(
            self.directory
                .write()
                .expect("typed resource cache directory poisoned")
                .families
                .entry(resource_type)
                .or_insert_with(|| Arc::new(RwLock::new(CacheFamilyState::default()))),
        )
    }

    pub(super) fn families(&self) -> Vec<(u32, Arc<RwLock<CacheFamilyState>>)> {
        self.directory
            .read()
            .expect("typed resource cache directory poisoned")
            .families
            .iter()
            .map(|(resource_type, family)| (*resource_type, Arc::clone(family)))
            .collect()
    }

    pub(super) fn try_families(&self) -> Option<Vec<(u32, Arc<RwLock<CacheFamilyState>>)>> {
        self.directory.try_read().ok().map(|directory| {
            directory
                .families
                .iter()
                .map(|(resource_type, family)| (*resource_type, Arc::clone(family)))
                .collect()
        })
    }

    pub(super) fn record_hit(&self) {
        self.hit_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_miss(&self) {
        self.miss_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_insert(&self, retained_bytes: usize) {
        self.entry_count.fetch_add(1, Ordering::AcqRel);
        self.retained_bytes
            .fetch_add(retained_bytes, Ordering::AcqRel);
    }

    pub(super) fn record_eviction(&self, retained_bytes: usize, retention: bool) {
        self.entry_count.fetch_sub(1, Ordering::AcqRel);
        let _ = self
            .retained_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |bytes| {
                Some(bytes.saturating_sub(retained_bytes))
            });
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
        if retention {
            self.retention_eviction_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(super) fn record_eviction_reinsertion(&self) {
        self.eviction_reinsertion_count
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn eviction_reinsertion_count(&self) -> u64 {
        self.eviction_reinsertion_count.load(Ordering::Relaxed)
    }

    pub(super) fn free_pressure_active(&self) -> bool {
        self.free_pressure_active.load(Ordering::Acquire)
    }

    pub(super) fn set_free_pressure_active(&self, active: bool) {
        self.free_pressure_active.store(active, Ordering::Release);
    }

    pub(super) fn entry_count(&self) -> usize {
        self.entry_count.load(Ordering::Acquire)
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.retained_bytes.load(Ordering::Acquire)
    }

    pub(super) fn stats(&self) -> ResourceCacheStats {
        let mut free_entry_count = 0usize;
        let mut free_retained_bytes = 0usize;
        let mut pinned_entry_count = 0usize;
        let mut pinned_retained_bytes = 0usize;
        for (_, family) in self.families() {
            let family = family.read().expect("typed resource cache family poisoned");
            let (free_count, free_bytes, pinned_count, pinned_bytes) = family.lifecycle_stats();
            free_entry_count = free_entry_count.saturating_add(free_count);
            free_retained_bytes = free_retained_bytes.saturating_add(free_bytes);
            pinned_entry_count = pinned_entry_count.saturating_add(pinned_count);
            pinned_retained_bytes = pinned_retained_bytes.saturating_add(pinned_bytes);
        }
        self.stats_with_lifecycle(
            free_entry_count,
            free_retained_bytes,
            pinned_entry_count,
            pinned_retained_bytes,
        )
    }

    pub(super) fn try_stats(&self) -> Option<ResourceCacheStats> {
        let mut free_entry_count = 0usize;
        let mut free_retained_bytes = 0usize;
        let mut pinned_entry_count = 0usize;
        let mut pinned_retained_bytes = 0usize;
        for (_, family) in self.try_families()? {
            let family = family.try_read().ok()?;
            let (free_count, free_bytes, pinned_count, pinned_bytes) = family.lifecycle_stats();
            free_entry_count = free_entry_count.saturating_add(free_count);
            free_retained_bytes = free_retained_bytes.saturating_add(free_bytes);
            pinned_entry_count = pinned_entry_count.saturating_add(pinned_count);
            pinned_retained_bytes = pinned_retained_bytes.saturating_add(pinned_bytes);
        }
        Some(self.stats_with_lifecycle(
            free_entry_count,
            free_retained_bytes,
            pinned_entry_count,
            pinned_retained_bytes,
        ))
    }

    fn stats_with_lifecycle(
        &self,
        free_entry_count: usize,
        free_retained_bytes: usize,
        pinned_entry_count: usize,
        pinned_retained_bytes: usize,
    ) -> ResourceCacheStats {
        let entry_count = self.entry_count();
        let retained_bytes = self.retained_bytes();
        ResourceCacheStats {
            entry_count,
            retained_bytes,
            active_entry_count: entry_count.saturating_sub(free_entry_count),
            active_retained_bytes: retained_bytes.saturating_sub(free_retained_bytes),
            free_entry_count,
            free_retained_bytes,
            pinned_entry_count,
            pinned_retained_bytes,
            hit_count: self.hit_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            eviction_count: self.eviction_count.load(Ordering::Relaxed),
            retention_eviction_count: self.retention_eviction_count.load(Ordering::Relaxed),
            eviction_reinsertion_count: self.eviction_reinsertion_count(),
        }
    }

    pub(super) fn reset_observation_stats(&self) {
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.eviction_count.store(0, Ordering::Relaxed);
        self.retention_eviction_count.store(0, Ordering::Relaxed);
        self.eviction_reinsertion_count.store(0, Ordering::Relaxed);
    }
}
