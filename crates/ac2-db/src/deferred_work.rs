//! Work ahead, commit at the consumption point.
//!
//! A frame owner may start work whose inputs are final at the end of frame N and which frame N's
//! own draw does not consume. The job runs on an idle shared worker while frame N+1's update
//! turn runs. The main thread takes the result where it consumes it today and computes inline
//! instead when the result is not ready, so a late result is never waited for. The owner must
//! validate that the result still matches its inputs before applying it.

use std::sync::{Arc, Mutex};

use crate::worker_lanes::{global_worker_lanes, WorkerLanes};

/// A result being computed on the shared pool.
#[derive(Debug)]
pub struct DeferredResult<R> {
    slot: Arc<Mutex<Option<R>>>,
}

impl<R> DeferredResult<R> {
    pub fn is_ready(&self) -> bool {
        self.slot.try_lock().is_ok_and(|slot| slot.is_some())
    }

    /// The finished result, or `None` if it is still running. Never waits: the caller computes
    /// inline instead.
    pub fn take_if_ready(self) -> Option<R> {
        let mut slot = self.slot.try_lock().ok()?;
        slot.take()
    }
}

/// Starts `job` on an idle worker of the client-built global pool. Returns `None`, running
/// nothing, when lanes are not installed, frame parallelism is disabled, no worker is idle, or
/// the caller is itself a pool thread.
pub fn spawn_deferred<R: Send + 'static>(
    job: impl FnOnce() -> R + Send + 'static,
) -> Option<DeferredResult<R>> {
    spawn_deferred_on(Arc::clone(global_worker_lanes()?), job)
}

/// `spawn_deferred` against explicit lanes over rayon's global pool.
pub fn spawn_deferred_on<R: Send + 'static>(
    lanes: Arc<WorkerLanes>,
    job: impl FnOnce() -> R + Send + 'static,
) -> Option<DeferredResult<R>> {
    if !lanes.frame_parallel() || rayon::current_thread_index().is_some() || !lanes.try_acquire() {
        return None;
    }
    let slot = Arc::new(Mutex::new(None));
    let output = Arc::clone(&slot);
    rayon::spawn(move || {
        let _lease = OwnedLease(lanes);
        let result = job();
        *output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(result);
    });
    Some(DeferredResult { slot })
}

/// Returns a deferred job's lease even if the job unwinds.
struct OwnedLease(Arc<WorkerLanes>);

impl Drop for OwnedLease {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::*;

    fn pool() {
        static BUILT: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        BUILT.get_or_init(|| {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(3)
                .build_global();
        });
    }

    #[test]
    fn deferred_result_arrives_and_returns_its_lease() {
        pool();
        let lanes = Arc::new(WorkerLanes::new(2, true));
        let pending = spawn_deferred_on(Arc::clone(&lanes), || 41 + 1).expect("an idle worker");
        let deadline = Instant::now() + Duration::from_secs(5);
        while !pending.is_ready() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        let value = pending.take_if_ready();
        assert_eq!(value, Some(42));
        while lanes.busy() != 0 && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert_eq!(lanes.busy(), 0);
    }

    #[test]
    fn disabled_or_saturated_lanes_start_nothing() {
        pool();
        let disabled = Arc::new(WorkerLanes::new(2, false));
        assert!(spawn_deferred_on(disabled, || ()).is_none());
        let saturated = Arc::new(WorkerLanes::new(1, true));
        assert!(saturated.try_acquire());
        assert!(spawn_deferred_on(Arc::clone(&saturated), || ()).is_none());
        assert_eq!(saturated.busy(), 1);
    }

    #[test]
    fn an_unfinished_result_is_not_waited_for() {
        pool();
        let lanes = Arc::new(WorkerLanes::new(1, true));
        let (release, gate) = std::sync::mpsc::channel::<()>();
        let pending = spawn_deferred_on(lanes, move || {
            let _ = gate.recv();
            7
        })
        .expect("an idle worker");
        let started = Instant::now();
        assert_eq!(pending.take_if_ready(), None);
        assert!(started.elapsed() < Duration::from_millis(50));
        release.send(()).unwrap();
    }
}
