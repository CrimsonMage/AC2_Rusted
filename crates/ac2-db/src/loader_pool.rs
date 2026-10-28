//! A pool of loader threads (retail `DBCache`'s asynchronous gets): the caller queues requests,
//! the workers run the work function on each (every DAT read and decode belongs there, never on
//! the caller's thread), and the caller drains finished results on its own thread, in arrival
//! order, within whatever budget it keeps.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Worker threads over one work function. Dropping the pool stops the queue and joins the
/// workers, so requests still running finish first.
pub struct LoaderPool<R, D> {
    requests: Option<Sender<R>>,
    finished: Receiver<D>,
    workers: Vec<JoinHandle<()>>,
    /// Work handed out and not yet drained.
    outstanding: usize,
}

impl<R: Send + 'static, D: Send + 'static> LoaderPool<R, D> {
    /// `threads` workers (at least one), named `name-<index>`, each running `work` on the
    /// requests it takes from the shared queue.
    pub fn new(name: &str, threads: usize, work: Arc<dyn Fn(R) -> D + Send + Sync>) -> Self {
        let (requests, queue) = channel::<R>();
        let (done, finished) = channel();
        let queue = Arc::new(Mutex::new(queue));
        let workers = (0..threads.max(1))
            .map(|index| {
                let queue = Arc::clone(&queue);
                let done = done.clone();
                let work = Arc::clone(&work);
                std::thread::Builder::new()
                    .name(format!("{name}-{index}"))
                    .spawn(move || loop {
                        let next = queue.lock().map(|queue| queue.recv());
                        let Ok(Ok(request)) = next else {
                            return;
                        };
                        if done.send(work(request)).is_err() {
                            return;
                        }
                    })
                    .expect("spawn a loader thread")
            })
            .collect();
        Self {
            requests: Some(requests),
            finished,
            workers,
            outstanding: 0,
        }
    }

    /// Queues `request` for the next free worker.
    pub fn request(&mut self, request: R) {
        if let Some(requests) = &self.requests {
            if requests.send(request).is_ok() {
                self.outstanding += 1;
            }
        }
    }

    /// Whether any request is still out.
    pub fn busy(&self) -> bool {
        self.outstanding > 0
    }

    /// The next finished result, if any.
    pub fn try_next(&mut self) -> Option<D> {
        let loaded = self.finished.try_recv().ok()?;
        self.outstanding -= 1;
        Some(loaded)
    }

    /// Blocks until a result is ready (tests and probes that wait for the world).
    pub fn wait(&mut self, timeout: Duration) -> Option<D> {
        let loaded = self.finished.recv_timeout(timeout).ok()?;
        self.outstanding -= 1;
        Some(loaded)
    }
}

impl<R, D> Drop for LoaderPool<R, D> {
    fn drop(&mut self) {
        self.requests = None;
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
