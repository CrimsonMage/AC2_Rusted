//! Shared worker lanes and the frame-work entry point.
//!
//! The client composition root builds rayon's global pool, sizes it, and installs one
//! `WorkerLanes` budget for it. Loading pull tasks and frame helpers lease that budget, so a
//! spawned task always finds a free thread. Lower crates call [`frame_map`] to split independent
//! per-item frame work. They never build a pool, and every frame piece also runs on the calling
//! thread, so a frame never waits behind a loading job:
//!
//! - The caller claims chunks itself and spawns helpers only for leases that are free right now.
//! - With every worker busy (or one worker, or the kill switch), the work runs inline at its serial
//!   cost.
//! - Results are merged by item index, never in completion order.
//!
//! Usage rules for `frame_map` closures: no blocking I/O, no waiting on preparation jobs, and no
//! shared mutable state (`f` only receives `&T`).

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

/// Inputs shorter than this stay serial: spawning costs more than tiny per-item work.
pub const FRAME_PARALLEL_MIN_ITEMS: usize = 64;

/// Something that can start queued work when a leased worker returns.
pub trait WorkerLaneWaiter: Send + Sync {
    fn worker_released(&self);
}

/// Leasable worker budget for one thread pool.
pub struct WorkerLanes {
    threads: usize,
    busy: AtomicUsize,
    frame_parallel: bool,
    waiters: Mutex<Vec<Weak<dyn WorkerLaneWaiter>>>,
    parallel_frames: AtomicU64,
    inline_frames: AtomicU64,
}

impl std::fmt::Debug for WorkerLanes {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkerLanes")
            .field("threads", &self.threads)
            .field("busy", &self.busy.load(Ordering::Relaxed))
            .field("frame_parallel", &self.frame_parallel)
            .finish()
    }
}

impl WorkerLanes {
    pub fn new(threads: usize, frame_parallel: bool) -> Self {
        Self {
            threads,
            busy: AtomicUsize::new(0),
            frame_parallel,
            waiters: Mutex::new(Vec::new()),
            parallel_frames: AtomicU64::new(0),
            inline_frames: AtomicU64::new(0),
        }
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn frame_parallel(&self) -> bool {
        self.frame_parallel
    }

    /// Workers currently leased (running or about to run a spawned task).
    pub fn busy(&self) -> usize {
        self.busy.load(Ordering::Acquire)
    }

    /// Leases one worker if any is free. Spawn exactly one task per successful lease.
    pub fn try_acquire(&self) -> bool {
        self.busy
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |busy| {
                (busy < self.threads).then_some(busy + 1)
            })
            .is_ok()
    }

    /// Returns a lease without notifying waiters. Loading pull tasks use this while holding
    /// their own queue lock, where they already decide whether more work can start.
    pub fn release_quiet(&self) {
        let previous = self.busy.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0, "worker lease released twice");
    }

    /// Returns a lease and lets registered waiters start queued work on it.
    pub fn release(&self) {
        self.release_quiet();
        let waiters = self
            .waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for waiter in waiters {
            waiter.worker_released();
        }
    }

    pub fn register_waiter(&self, waiter: Weak<dyn WorkerLaneWaiter>) {
        let mut waiters = self
            .waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        waiters.retain(|waiter| waiter.strong_count() != 0);
        waiters.push(waiter);
    }

    /// Frame splits that used helpers, and splits that ran inline.
    pub fn frame_counts(&self) -> (u64, u64) {
        (
            self.parallel_frames.load(Ordering::Relaxed),
            self.inline_frames.load(Ordering::Relaxed),
        )
    }
}

static INSTALLED: OnceLock<Arc<WorkerLanes>> = OnceLock::new();

/// Installs the lanes for rayon's global pool. Only the composition root that built that pool
/// calls this; the first installation wins.
pub fn install_global_worker_lanes(lanes: Arc<WorkerLanes>) -> Result<(), Arc<WorkerLanes>> {
    INSTALLED.set(lanes)
}

pub fn global_worker_lanes() -> Option<&'static Arc<WorkerLanes>> {
    INSTALLED.get()
}

/// Maps `items` in index order, splitting across idle shared workers when worthwhile.
pub fn frame_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    frame_map_with_threshold(items, FRAME_PARALLEL_MIN_ITEMS, f)
}

pub fn frame_map_with_threshold<T: Sync, R: Send>(
    items: &[T],
    min_items: usize,
    f: impl Fn(&T) -> R + Sync,
) -> Vec<R> {
    match global_worker_lanes() {
        Some(lanes) => frame_map_on(lanes, items, min_items, f),
        None => items.iter().map(f).collect(),
    }
}

/// `frame_map` against explicit lanes; the caller must be outside the lanes' pool, which must be
/// rayon's global pool.
pub fn frame_map_on<T: Sync, R: Send>(
    lanes: &WorkerLanes,
    items: &[T],
    min_items: usize,
    f: impl Fn(&T) -> R + Sync,
) -> Vec<R> {
    // A pool thread waiting in a scope would execute unrelated pool tasks (loading jobs) while
    // it waits; frame work from such a thread stays serial.
    let helpers = if !lanes.frame_parallel
        || items.len() < min_items.max(2)
        || rayon::current_thread_index().is_some()
    {
        0
    } else {
        let wanted = (items.len() - 1).min(lanes.threads);
        (0..wanted).take_while(|_| lanes.try_acquire()).count()
    };
    if helpers == 0 {
        lanes.inline_frames.fetch_add(1, Ordering::Relaxed);
        return items.iter().map(f).collect();
    }
    lanes.parallel_frames.fetch_add(1, Ordering::Relaxed);
    let chunk_count = items.len().min((helpers + 1) * 4);
    let next = AtomicUsize::new(0);
    let slots = (0..chunk_count)
        .map(|_| Mutex::new(None::<Vec<R>>))
        .collect::<Vec<_>>();
    let run_chunks = || loop {
        let chunk = next.fetch_add(1, Ordering::Relaxed);
        if chunk >= chunk_count {
            break;
        }
        let start = chunk * items.len() / chunk_count;
        let end = (chunk + 1) * items.len() / chunk_count;
        let output = items[start..end].iter().map(&f).collect::<Vec<_>>();
        *slots[chunk]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(output);
    };
    rayon::in_place_scope(|scope| {
        for _ in 0..helpers {
            scope.spawn(|_| {
                let _lease = HelperLease(lanes);
                run_chunks();
            });
        }
        run_chunks();
    });
    let mut merged = Vec::with_capacity(items.len());
    for slot in slots {
        merged.extend(
            slot.into_inner()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .expect("every frame chunk completes inside the scope"),
        );
    }
    merged
}

/// Returns a helper's lease even if its closure unwinds.
struct HelperLease<'a>(&'a WorkerLanes);

impl Drop for HelperLease<'_> {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[cfg(test)]
#[path = "worker_lanes_tests.rs"]
mod tests;
