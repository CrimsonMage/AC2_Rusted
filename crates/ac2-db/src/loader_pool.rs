//! A pool of loader threads (retail `DBCache`'s asynchronous gets): the caller queues requests,
//! the workers run the work function on each (every DAT read and decode belongs there, never on
//! the caller's thread), and the caller drains finished results on its own thread, in arrival
//! order, within whatever budget it keeps.
//!
//! # DAT-reading threads
//!
//! Retail read the DATs on one worker (`ThreadedCache::ThreadedCache` starts a single thread;
//! `ThreadedCache::WorkerExecuteGetRequest` reads, the main thread decodes). Here decoding runs
//! on the readers too, so there are a few of them, but their counts are fixed, never scaled with
//! the core count: the DAT read study (`docs/dat-read-study.md`) measured cold-cache read time
//! halving from one thread to two and nothing gained past four, while every thread adds a
//! contender for the B-tree lookup lock. The readers in the client:
//!
//! - `ac2-loader` ([`WORLD_LOADER_THREADS`], `ac2_client::loader`): landblocks, cells, bodies,
//!   script data, ambient sound data and points, motion and entity descriptions.
//! - `ac2-scene-loader` (1, `ac2_render::scene::loader`): one thread on purpose, its requests are
//!   served in order so an image arrives before every later load that samples it.
//! - `ac2-sound-loader` (1, `ac2_audio::loader`): `SoundInfo`/`DBWave` gets.
//! - Single-thread one-off or bursty readers: `ac2 ui images`, `ac2 chat text` (`ac2_ui`),
//!   `ac2 ui strings`, `ac2 hud`, `ac2 pregame`, `ac2 pregame scenes` (`ac2_client`).
//!
//! - `ac2 map overview` with its `RASTER_THREADS` (4) column workers, once while the world map
//!   draws the whole world at one pixel per cell, then gone; `ac2 map detail` (`DETAIL_THREADS`,
//!   2, `ac2_ui::world_map_tiles`): the landblocks the navigation map shows close up, only while
//!   it is zoomed in. They read through their own store with a small cache, so the sweep does not
//!   flush the world's.
//!
//! In the world that is four steady readers (two loaders, scene, sound), plus the map's two while
//! it shows a close view; the one-off readers run at startup and login. The frame helpers (`install_frame_workers`, `frame_worker_count`) are
//! for frame work and never read DATs. The world viewer's block pool uses
//! [`WORLD_LOADER_THREADS`] as the client does.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Threads of the client's world loader pool (`ac2-loader`), and of the world viewer's block pool
/// that stands in for it: see the module notes for why it is fixed at two.
pub const WORLD_LOADER_THREADS: usize = 2;

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
