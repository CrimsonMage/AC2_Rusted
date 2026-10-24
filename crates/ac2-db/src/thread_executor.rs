use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::thread;

use crate::{PreparationExecutor, PreparationJob, PreparationSubmitError};

const PREPARATION_CAPACITY: usize = 256;

/// A single background thread running submitted preparation jobs in order. The channel owns the
/// worker's lifetime: dropping the final executor sender stops the thread.
#[derive(Debug)]
pub struct ThreadPreparationExecutor {
    sender: SyncSender<PreparationJob>,
}

impl ThreadPreparationExecutor {
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::sync_channel::<PreparationJob>(PREPARATION_CAPACITY);
        thread::Builder::new()
            .name("ac2-preparation".to_owned())
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    job.run();
                }
            })
            .expect("preparation worker thread must start");
        Self { sender }
    }
}

impl PreparationExecutor for ThreadPreparationExecutor {
    fn try_submit(&self, job: PreparationJob) -> Result<(), PreparationSubmitError> {
        self.sender.try_send(job).map_err(|error| match error {
            TrySendError::Full(_) => PreparationSubmitError::Full,
            TrySendError::Disconnected(_) => PreparationSubmitError::Stopped,
        })
    }
}
