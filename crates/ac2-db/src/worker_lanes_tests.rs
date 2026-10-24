use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use super::*;

const TEST_THREADS: usize = 3;

/// Test-only stand-in for the client-built global pool.
fn global_test_pool() {
    static BUILT: OnceLock<()> = OnceLock::new();
    BUILT.get_or_init(|| {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(TEST_THREADS)
            .build_global();
    });
}

fn lanes(frame_parallel: bool) -> WorkerLanes {
    global_test_pool();
    WorkerLanes::new(TEST_THREADS, frame_parallel)
}

#[test]
fn leases_never_exceed_the_thread_budget_and_wake_waiters() {
    struct Counter(AtomicUsize);
    impl WorkerLaneWaiter for Counter {
        fn worker_released(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let lanes = WorkerLanes::new(2, true);
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let waiter: Arc<dyn WorkerLaneWaiter> = counter.clone();
    lanes.register_waiter(Arc::downgrade(&waiter));
    assert!(lanes.try_acquire());
    assert!(lanes.try_acquire());
    assert!(!lanes.try_acquire());
    lanes.release_quiet();
    assert_eq!(counter.0.load(Ordering::SeqCst), 0);
    lanes.release();
    assert_eq!(counter.0.load(Ordering::SeqCst), 1);
    assert_eq!(lanes.busy(), 0);
}

#[test]
fn parallel_frame_map_merges_in_index_order() {
    let lanes = lanes(true);
    let items = (0..10_000_u64).collect::<Vec<_>>();
    let serial = items.iter().map(|value| value * 3 + 1).collect::<Vec<_>>();
    for _ in 0..20 {
        let parallel = frame_map_on(&lanes, &items, 2, |value| value * 3 + 1);
        assert_eq!(parallel, serial);
    }
    assert!(lanes.frame_counts().0 > 0, "an idle pool must take helpers");
    assert_eq!(lanes.busy(), 0, "every helper lease returns");
}

#[test]
fn busy_workers_threshold_and_kill_switch_run_inline() {
    let items = (0..256_u32).collect::<Vec<_>>();
    let expected = items.iter().map(|value| value + 1).collect::<Vec<_>>();

    let saturated = lanes(true);
    for _ in 0..TEST_THREADS {
        assert!(saturated.try_acquire());
    }
    assert_eq!(frame_map_on(&saturated, &items, 2, |v| v + 1), expected);
    assert_eq!(saturated.frame_counts(), (0, 1));

    let disabled = lanes(false);
    assert_eq!(frame_map_on(&disabled, &items, 2, |v| v + 1), expected);
    assert_eq!(disabled.frame_counts(), (0, 1));

    let small = lanes(true);
    assert_eq!(frame_map_on(&small, &items, 1_000, |v| v + 1), expected);
    assert_eq!(small.frame_counts(), (0, 1));
}

#[test]
fn frame_map_from_a_pool_thread_stays_serial() {
    let lanes = lanes(true);
    let items = (0..512_u32).collect::<Vec<_>>();
    let mapped = rayon::scope(|_| frame_map_on(&lanes, &items, 2, |v| v * 2));
    assert_eq!(mapped, items.iter().map(|v| v * 2).collect::<Vec<_>>());
    assert_eq!(lanes.frame_counts(), (0, 1));
}
