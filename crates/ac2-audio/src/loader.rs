//! The sound manager's asynchronous gets (`DBCache` with `AsyncCacheCallback`): every read and
//! decode runs on this thread, never the frame's.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::data::{LoadedSound, SoundSource};

/// A finished get: its context and the sound, or `None` when the DATs lack it.
pub(crate) type Finished = (u32, Option<LoadedSound>);

pub(crate) struct SoundLoader {
    requests: Option<Sender<(u32, u32)>>,
    finished: Receiver<Finished>,
    worker: Option<JoinHandle<()>>,
}

impl SoundLoader {
    pub fn new(source: Arc<dyn SoundSource>) -> Self {
        let (requests, queue) = channel::<(u32, u32)>();
        let (done, finished) = channel();
        let worker = std::thread::Builder::new()
            .name("ac2-sound-loader".into())
            .spawn(move || {
                while let Ok((context, did)) = queue.recv() {
                    if done.send((context, source.load(did))).is_err() {
                        return;
                    }
                }
            })
            .ok();
        Self {
            requests: Some(requests),
            finished,
            worker,
        }
    }

    /// `DBCache::GetAsync` of `did` for `context`.
    pub fn request(&self, context: u32, did: u32) {
        if let Some(requests) = &self.requests {
            let _ = requests.send((context, did));
        }
    }

    pub fn try_next(&self) -> Option<Finished> {
        self.finished.try_recv().ok()
    }

    pub fn wait(&self, timeout: Duration) -> Option<Finished> {
        self.finished.recv_timeout(timeout).ok()
    }
}

impl Drop for SoundLoader {
    fn drop(&mut self) {
        self.requests = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
