//! Narrow cross-domain submission contract for immutable preparation work.
//!
//! This crate defines only the request vocabulary so lower owners can submit immutable jobs
//! without creating their own threads. The top-level composition root owns the executor's
//! lifetime, capacity, worker count, and cross-domain admission policy.

use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;

/// The owner family of an immutable preparation request.
///
/// The label is diagnostic and admission metadata only. It does not move resource, SmartBox,
/// render, or UI ownership into the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreparationDomain {
    Resource,
    WorldScope,
    Render,
    UiMedia,
    Refinement,
}

/// Cross-domain admission hint interpreted by the client-owned executor.
///
/// FIFO remains mandatory within the same class. A domain keeps its own typed request,
/// generation, cancellation, and completed-product policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreparationPriority {
    Critical,
    Interactive,
    Background,
    Refinement,
}

/// One sendable immutable preparation closure.
///
/// The closure must neither wait on the shared preparation executor nor synchronously ask a
/// missing child to be decoded. It returns by publishing a typed owner-owned completion or by
/// recording a `PreparationStep::WaitingOn` continuation.
pub struct PreparationJob {
    domain: PreparationDomain,
    priority: PreparationPriority,
    run: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl PreparationJob {
    pub fn new(
        domain: PreparationDomain,
        priority: PreparationPriority,
        run: impl FnOnce() + Send + 'static,
    ) -> Self {
        Self {
            domain,
            priority,
            run: Some(Box::new(run)),
        }
    }

    pub const fn domain(&self) -> PreparationDomain {
        self.domain
    }

    pub const fn priority(&self) -> PreparationPriority {
        self.priority
    }

    pub fn run(mut self) {
        if let Some(run) = self.run.take() {
            run();
        }
    }
}

impl fmt::Debug for PreparationJob {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparationJob")
            .field("domain", &self.domain)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}

/// Why an owner could not submit more immutable work right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationSubmitError {
    Full,
    Stopped,
}

/// Why a best-effort host-turn refinement could not be admitted without waiting.
///
/// This is separate from [`PreparationSubmitError`] because ordinary preparation owners may
/// briefly wait for the executor's admission lock. Host-turn maintenance must instead treat
/// contention as a skipped attempt so input and presentation remain responsive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonblockingPreparationSubmitError {
    Busy,
    Full,
    Stopped,
}

impl From<PreparationSubmitError> for NonblockingPreparationSubmitError {
    fn from(error: PreparationSubmitError) -> Self {
        match error {
            PreparationSubmitError::Full => Self::Full,
            PreparationSubmitError::Stopped => Self::Stopped,
        }
    }
}

/// Client-injected admission seam for immutable work.
///
/// Implementations are deliberately not supplied by this crate: the client composition root
/// owns the one shared worker and its policy. Tools may inject an inline/testing executor.
pub trait PreparationExecutor: fmt::Debug + Send + Sync {
    fn try_submit(&self, job: PreparationJob) -> Result<(), PreparationSubmitError>;

    /// Best-effort refinement admission from a latency-sensitive host turn.
    ///
    /// Executors that can prove nonblocking admission override this method. The conservative
    /// default declines the job rather than calling the potentially contended ordinary seam.
    fn try_submit_refinement_nonblocking(
        &self,
        job: PreparationJob,
    ) -> Result<(), NonblockingPreparationSubmitError> {
        debug_assert_eq!(job.priority(), PreparationPriority::Refinement);
        Err(NonblockingPreparationSubmitError::Busy)
    }

    /// How many concurrent jobs this executor can run. Owners size their in-flight preparation
    /// to it; the default of one keeps tool and test executors serial.
    fn parallelism(&self) -> usize {
        1
    }
}

/// Executes an immutable job in the calling thread.
///
/// This is intended only for explicit test/tool composition. Production client composition must
/// inject its bounded Worker-2 executor instead.
#[derive(Debug, Default)]
pub struct InlinePreparationExecutor {
    state: Mutex<InlinePreparationState>,
}

#[derive(Debug, Default)]
struct InlinePreparationState {
    jobs: VecDeque<PreparationJob>,
    draining: bool,
}

impl PreparationExecutor for InlinePreparationExecutor {
    fn try_submit(&self, job: PreparationJob) -> Result<(), PreparationSubmitError> {
        let mut state = self
            .state
            .lock()
            .expect("inline preparation queue poisoned");
        state.jobs.push_back(job);
        if state.draining {
            return Ok(());
        }
        state.draining = true;
        drop(state);

        loop {
            let job = {
                let mut state = self
                    .state
                    .lock()
                    .expect("inline preparation queue poisoned");
                state.jobs.pop_front()
            };
            let Some(job) = job else {
                let mut state = self
                    .state
                    .lock()
                    .expect("inline preparation queue poisoned");
                state.draining = false;
                break;
            };
            job.run();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use super::{
        InlinePreparationExecutor, PreparationDomain, PreparationExecutor, PreparationJob,
        PreparationPriority,
    };

    #[test]
    fn inline_executor_is_explicit_and_runs_one_job() {
        let completed = Arc::new(AtomicBool::new(false));
        let completion = Arc::clone(&completed);
        InlinePreparationExecutor::default()
            .try_submit(PreparationJob::new(
                PreparationDomain::Resource,
                PreparationPriority::Interactive,
                move || completion.store(true, Ordering::Release),
            ))
            .expect("inline submission should run");
        assert!(completed.load(Ordering::Acquire));
    }
}
