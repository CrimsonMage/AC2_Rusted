use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use super::state::{CacheEntry, CacheFamilyState, FreeTicket};
use super::TypedResourceCache;

const MAX_LIFECYCLE_EVENTS_PER_PASS: usize = 1_024;
const MAX_EVICTIONS_PER_PASS: usize = 256;
const MAX_EVICTION_BYTES_PER_PASS: usize = 64 * 1024 * 1024;
/// Passes one insert-side trim may run before it leaves the rest to the next insert.
const MAX_TRIM_PASSES: usize = 8;

/// Result of one free-value reaping attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum ResourceCacheMaintenanceOutcome {
    /// The family directory itself was changing, so no maintenance work ran.
    Busy,
    Completed(ResourceCacheMaintenanceReport),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceCacheFamilyMaintenanceReport {
    resource_type: u32,
    releases_accepted: usize,
    stale_releases: usize,
    evicted_pressure: usize,
}

impl ResourceCacheFamilyMaintenanceReport {
    pub const fn resource_type(self) -> u32 {
        self.resource_type
    }
    pub const fn releases_accepted(self) -> usize {
        self.releases_accepted
    }
    pub const fn stale_releases(self) -> usize {
        self.stale_releases
    }
    pub const fn evicted_pressure(self) -> usize {
        self.evicted_pressure
    }
}

/// Bounded lifecycle and retirement evidence for one event-driven maintenance pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceCacheMaintenanceReport {
    entries_before: usize,
    entries_after: usize,
    retained_bytes_before: usize,
    retained_bytes_after: usize,
    lifecycle_events: usize,
    stale_events: usize,
    pending_events: usize,
    families_attempted: usize,
    families_completed: usize,
    families_skipped: usize,
    evicted_entries: usize,
    evicted_bytes: usize,
    evicted_pressure: usize,
    pressure_evicted_bytes: usize,
    free_retained_bytes_before: usize,
    free_retained_bytes_after: usize,
    free_byte_budget: usize,
    free_byte_low_water: usize,
    pressure_blocked: bool,
    eviction_reinsertions_total: u64,
    more_work: bool,
    family_activity: Vec<ResourceCacheFamilyMaintenanceReport>,
    lock_elapsed: Duration,
    drop_elapsed: Duration,
    total_elapsed: Duration,
}

impl ResourceCacheMaintenanceReport {
    pub const fn entries_before(&self) -> usize {
        self.entries_before
    }
    pub const fn entries_after(&self) -> usize {
        self.entries_after
    }
    pub const fn retained_bytes_before(&self) -> usize {
        self.retained_bytes_before
    }
    pub const fn retained_bytes_after(&self) -> usize {
        self.retained_bytes_after
    }
    pub const fn lifecycle_events(&self) -> usize {
        self.lifecycle_events
    }
    pub const fn stale_events(&self) -> usize {
        self.stale_events
    }
    pub const fn pending_events(&self) -> usize {
        self.pending_events
    }
    pub const fn families_attempted(&self) -> usize {
        self.families_attempted
    }
    pub const fn families_completed(&self) -> usize {
        self.families_completed
    }
    pub const fn families_skipped(&self) -> usize {
        self.families_skipped
    }
    pub const fn evicted_entries(&self) -> usize {
        self.evicted_entries
    }
    pub const fn evicted_bytes(&self) -> usize {
        self.evicted_bytes
    }
    pub const fn evicted_pressure(&self) -> usize {
        self.evicted_pressure
    }
    pub const fn pressure_evicted_bytes(&self) -> usize {
        self.pressure_evicted_bytes
    }
    pub const fn free_retained_bytes_before(&self) -> usize {
        self.free_retained_bytes_before
    }
    pub const fn free_retained_bytes_after(&self) -> usize {
        self.free_retained_bytes_after
    }
    pub const fn free_byte_budget(&self) -> usize {
        self.free_byte_budget
    }
    pub const fn free_byte_low_water(&self) -> usize {
        self.free_byte_low_water
    }
    pub const fn pressure_blocked(&self) -> bool {
        self.pressure_blocked
    }
    pub const fn eviction_reinsertions_total(&self) -> u64 {
        self.eviction_reinsertions_total
    }
    pub const fn more_work(&self) -> bool {
        self.more_work
    }
    pub fn family_activity(&self) -> &[ResourceCacheFamilyMaintenanceReport] {
        &self.family_activity
    }
    pub const fn lock_elapsed(&self) -> Duration {
        self.lock_elapsed
    }
    pub const fn drop_elapsed(&self) -> Duration {
        self.drop_elapsed
    }
    pub const fn total_elapsed(&self) -> Duration {
        self.total_elapsed
    }
}

#[derive(Debug, Default)]
struct DetachedCacheEntries {
    entries: Vec<CacheEntry>,
    retained_bytes: usize,
}

impl DetachedCacheEntries {
    fn push(&mut self, entry: CacheEntry) {
        self.retained_bytes = self.retained_bytes.saturating_add(entry.retained_bytes);
        self.entries.push(entry);
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn budget_exhausted(&self) -> bool {
        self.entries.len() >= MAX_EVICTIONS_PER_PASS
            || self.retained_bytes >= MAX_EVICTION_BYTES_PER_PASS
    }

    fn can_accept(&self, retained_bytes: usize) -> bool {
        self.entries.is_empty()
            || (self.entries.len() < MAX_EVICTIONS_PER_PASS
                && self.retained_bytes.saturating_add(retained_bytes)
                    <= MAX_EVICTION_BYTES_PER_PASS)
    }

    fn drop_and_measure(self) -> Duration {
        let started = Instant::now();
        drop(self);
        started.elapsed()
    }
}

#[derive(Debug, Clone, Copy)]
struct FreePressureCandidate {
    resource_type: u32,
    ticket: FreeTicket,
    retained_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct FreePressureSnapshot {
    retained_bytes: usize,
    oldest: Option<FreePressureCandidate>,
    complete: bool,
}

fn observe_free_pressure(
    families: &[(u32, Arc<RwLock<CacheFamilyState>>)],
) -> FreePressureSnapshot {
    let mut snapshot = FreePressureSnapshot {
        complete: true,
        ..Default::default()
    };
    for (resource_type, family) in families {
        let Ok(family) = family.try_read() else {
            snapshot.complete = false;
            continue;
        };
        snapshot.retained_bytes = snapshot
            .retained_bytes
            .saturating_add(family.free_retained_bytes());
        let Some((ticket, retained_bytes)) = family.peek_oldest_free() else {
            continue;
        };
        if snapshot
            .oldest
            .is_none_or(|oldest| ticket.released_at < oldest.ticket.released_at)
        {
            snapshot.oldest = Some(FreePressureCandidate {
                resource_type: *resource_type,
                ticket,
                retained_bytes,
            });
        }
    }
    snapshot
}

fn same_ticket(left: FreeTicket, right: FreeTicket) -> bool {
    left.key == right.key && left.epoch == right.epoch && left.released_at == right.released_at
}

fn detach_ticket(
    cache: &TypedResourceCache,
    family: &mut CacheFamilyState,
    ticket: FreeTicket,
    detached: &mut DetachedCacheEntries,
    now: Instant,
) -> bool {
    let Some(entry) = family.entries.get(&ticket.key) else {
        return false;
    };
    if !entry.matches_free_ticket(ticket) {
        return false;
    }
    if !detached.can_accept(entry.retained_bytes) {
        family.restore_oldest(ticket);
        return false;
    }
    family.record_retention_eviction(ticket.key, now);
    let Some(entry) = family.detach_entry(ticket.key) else {
        return false;
    };
    cache.state.record_eviction(entry.retained_bytes, true);
    detached.push(entry);
    true
}

impl TypedResourceCache {
    /// The insert side of the byte budget: once the cache holds more than the free budget in
    /// total (so its free values may exceed it), the inserting thread, a decode worker, takes
    /// the queued releases and reaps the oldest free values. Nothing runs on a frame timer.
    pub(super) fn trim_if_over_budget(&self) {
        let Some(config) = self.state.try_config() else {
            return;
        };
        if self.state.retained_bytes() <= config.free_byte_budget()
            && !self.state.free_pressure_active()
        {
            return;
        }
        if !self.state.try_begin_trim() {
            return;
        }
        for _ in 0..MAX_TRIM_PASSES {
            match self.try_maintain_report() {
                ResourceCacheMaintenanceOutcome::Completed(report)
                    if report.more_work() && report.evicted_entries() > 0 => {}
                _ => break,
            }
        }
        self.state.end_trim();
    }

    pub(super) fn try_maintain_report(&self) -> ResourceCacheMaintenanceOutcome {
        let total_started = Instant::now();
        let Some(config) = self.state.try_config() else {
            return ResourceCacheMaintenanceOutcome::Busy;
        };
        let Some(families) = self.state.try_families() else {
            return ResourceCacheMaintenanceOutcome::Busy;
        };
        let entries_before = self.state.entry_count();
        let retained_bytes_before = self.state.retained_bytes();
        let mut reports = HashMap::<u32, ResourceCacheFamilyMaintenanceReport>::new();
        let mut lifecycle_events = 0_usize;
        let mut stale_events = 0_usize;
        let mut skipped = 0_usize;
        let mut more_work = false;

        while lifecycle_events < MAX_LIFECYCLE_EVENTS_PER_PASS {
            let Ok(token) = self.state.release_events.pop() else {
                break;
            };
            let Some(lifecycle) = token.lifecycle.upgrade() else {
                stale_events = stale_events.saturating_add(1);
                lifecycle_events = lifecycle_events.saturating_add(1);
                continue;
            };
            let resource_type = token.key.resource_key().type_id();
            let family = match self.state.try_family(resource_type) {
                Ok(Some(family)) => family,
                Ok(None) | Err(()) => {
                    // A release may race the first insertion of a resource family after the
                    // maintenance family snapshot. Families are never deleted, so consuming
                    // this token would strand the entry outside the event-driven free FIFO.
                    let _ = self.state.release_events.push(token);
                    skipped = skipped.saturating_add(1);
                    more_work = true;
                    break;
                }
            };
            let Ok(mut family) = family.try_write() else {
                let _ = self.state.release_events.push(token);
                skipped = skipped.saturating_add(1);
                more_work = true;
                break;
            };
            let report = reports.entry(token.key.resource_key().type_id()).or_insert(
                ResourceCacheFamilyMaintenanceReport {
                    resource_type: token.key.resource_key().type_id(),
                    ..Default::default()
                },
            );
            let Some(event) = lifecycle.take_pending_release() else {
                lifecycle.finish_release_token(&self.state);
                report.stale_releases = report.stale_releases.saturating_add(1);
                stale_events = stale_events.saturating_add(1);
                lifecycle_events = lifecycle_events.saturating_add(1);
                continue;
            };
            if family.accept_release(event, &lifecycle) {
                report.releases_accepted = report.releases_accepted.saturating_add(1);
            } else {
                report.stale_releases = report.stale_releases.saturating_add(1);
                stale_events = stale_events.saturating_add(1);
            }
            lifecycle.finish_release_token(&self.state);
            lifecycle_events = lifecycle_events.saturating_add(1);
        }
        if !self.state.release_events.is_empty() {
            more_work = true;
        }

        let free_pressure_before = observe_free_pressure(&families);
        let mut pressure_blocked =
            self.state.free_pressure_active() && !free_pressure_before.complete;
        if free_pressure_before.complete
            && free_pressure_before.retained_bytes > config.free_byte_budget()
        {
            self.state.set_free_pressure_active(true);
        }

        let now = Instant::now();
        let mut detached = DetachedCacheEntries::default();
        let completed = families.len().saturating_sub(skipped);

        let mut evicted_pressure = 0_usize;
        let mut pressure_evicted_bytes = 0_usize;
        while self.state.free_pressure_active() {
            let pressure = observe_free_pressure(&families);
            if !pressure.complete {
                pressure_blocked = true;
                more_work = true;
                break;
            }
            if pressure.retained_bytes <= config.free_byte_low_water() {
                self.state.set_free_pressure_active(false);
                break;
            }
            if detached.budget_exhausted() {
                more_work = true;
                break;
            }
            let Some(candidate) = pressure.oldest else {
                pressure_blocked = true;
                more_work = true;
                break;
            };
            let Some((_, family)) = families
                .iter()
                .find(|(resource_type, _)| *resource_type == candidate.resource_type)
            else {
                pressure_blocked = true;
                more_work = true;
                break;
            };
            let Ok(mut family) = family.try_write() else {
                pressure_blocked = true;
                more_work = true;
                break;
            };
            let Some((current, retained_bytes)) = family.peek_oldest_free() else {
                pressure_blocked = true;
                more_work = true;
                break;
            };
            if !same_ticket(current, candidate.ticket) || retained_bytes != candidate.retained_bytes
            {
                pressure_blocked = true;
                more_work = true;
                break;
            }
            let Some(ticket) = family.oldest_free() else {
                pressure_blocked = true;
                more_work = true;
                break;
            };
            if !detach_ticket(self, &mut family, ticket, &mut detached, now) {
                pressure_blocked = true;
                more_work = true;
                break;
            }
            let report = reports.entry(candidate.resource_type).or_insert(
                ResourceCacheFamilyMaintenanceReport {
                    resource_type: candidate.resource_type,
                    ..Default::default()
                },
            );
            report.evicted_pressure = report.evicted_pressure.saturating_add(1);
            evicted_pressure = evicted_pressure.saturating_add(1);
            pressure_evicted_bytes = pressure_evicted_bytes.saturating_add(retained_bytes);
        }

        let free_pressure_after = observe_free_pressure(&families);
        if self.state.free_pressure_active() && !free_pressure_after.complete {
            pressure_blocked = true;
        }
        if self.state.free_pressure_active()
            && free_pressure_after.complete
            && free_pressure_after.retained_bytes > config.free_byte_low_water()
        {
            more_work = true;
        }

        let entries_after = self.state.entry_count();
        let retained_bytes_after = self.state.retained_bytes();
        let evicted_entries = detached.len();
        let evicted_bytes = detached.retained_bytes;
        let lock_elapsed = total_started.elapsed();
        let drop_elapsed = detached.drop_and_measure();
        let mut family_activity = reports.into_values().collect::<Vec<_>>();
        family_activity.sort_unstable_by_key(|report| report.resource_type);
        ResourceCacheMaintenanceOutcome::Completed(ResourceCacheMaintenanceReport {
            entries_before,
            entries_after,
            retained_bytes_before,
            retained_bytes_after,
            lifecycle_events,
            stale_events,
            pending_events: self.state.release_events.len(),
            families_attempted: families.len(),
            families_completed: completed,
            families_skipped: skipped,
            evicted_entries,
            evicted_bytes,
            evicted_pressure,
            pressure_evicted_bytes,
            free_retained_bytes_before: free_pressure_before.retained_bytes,
            free_retained_bytes_after: free_pressure_after.retained_bytes,
            free_byte_budget: config.free_byte_budget(),
            free_byte_low_water: config.free_byte_low_water(),
            pressure_blocked,
            eviction_reinsertions_total: self.state.eviction_reinsertion_count(),
            more_work,
            family_activity,
            lock_elapsed,
            drop_elapsed,
            total_elapsed: total_started.elapsed(),
        })
    }
}
