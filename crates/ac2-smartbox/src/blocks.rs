//! Landblock loading off the frame thread (retail `DBCache`'s asynchronous gets): a block is
//! built into a `Terrain` of its own on a loader thread, and the frame thread later absorbs it
//! whole (`Terrain::absorb`). [`CellManager`](crate::CellManager) asks through [`BlockLoader`];
//! [`BlockPool`] is the plain thread pool over a [`BlockSource`] that answers it.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use ac2_core::CellId;
use ac2_physics::{load_block_interiors, load_block_objects, load_terrain_block, Terrain};
use ac2_world::Ac2WorldSource;

/// Where blocks come from; read on loader threads only.
pub trait BlockSource: Send + Sync {
    /// Loads landblock `block` (`xxyy0000`) into `terrain`: its terrain, static objects and
    /// interior cells (`CLandBlock` with `CreateStaticObjects` and `InitCellConnections`).
    /// Returns whether the block exists.
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String>;
    /// A far landblock's `CLandBlockData` without its sub-data, read so the cache holds it.
    fn prefetch_far_block(&self, _block: u32) -> Result<(), String> {
        Ok(())
    }
}

/// The AC2 DATs, through `ac2-world` and `ac2-physics`' loaders.
pub fn load_dat_block(
    world: &Ac2WorldSource,
    terrain: &mut Terrain,
    block: u32,
) -> Result<bool, String> {
    let id = CellId::new(block | 0xFFFF);
    if !load_terrain_block(world, terrain, id).map_err(|e| e.to_string())? {
        return Ok(false);
    }
    load_block_objects(world, terrain, id).map_err(|e| e.to_string())?;
    load_block_interiors(world, terrain, id).map_err(|e| e.to_string())?;
    Ok(true)
}

/// A far block's data alone, read so the DB cache holds it.
pub fn prefetch_dat_far_block(world: &Ac2WorldSource, block: u32) -> Result<(), String> {
    world
        .load_landblock(CellId::new(block | 0xFFFF))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

impl BlockSource for Ac2WorldSource {
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String> {
        load_dat_block(self, terrain, block)
    }

    fn prefetch_far_block(&self, block: u32) -> Result<(), String> {
        prefetch_dat_far_block(self, block)
    }
}

/// What the cell manager asks of its loader.
pub trait BlockLoader {
    /// A near landblock with its sub-data (objects, interiors).
    fn request_block(&mut self, block: u32);
    /// A far landblock's data alone.
    fn request_far_block(&mut self, block: u32);
}

/// Finished block work.
pub enum BlockLoaded {
    /// The block's world, or `None` where the map has no such block.
    Block(u32, Result<Option<Terrain>, String>),
    FarBlock(u32, Result<(), String>),
}

#[derive(Debug, Clone, Copy)]
enum Request {
    Block(u32),
    FarBlock(u32),
}

fn load(source: &dyn BlockSource, request: Request) -> BlockLoaded {
    match request {
        Request::Block(block) => {
            let mut terrain = Terrain::new();
            let result = source
                .load_block(&mut terrain, block)
                .map(|exists| exists.then_some(terrain));
            BlockLoaded::Block(block, result)
        }
        Request::FarBlock(block) => BlockLoaded::FarBlock(block, source.prefetch_far_block(block)),
    }
}

/// A pool of loader threads over one [`BlockSource`].
pub struct BlockPool {
    requests: Option<Sender<Request>>,
    finished: Receiver<BlockLoaded>,
    workers: Vec<JoinHandle<()>>,
    /// Work handed out and not yet drained.
    outstanding: usize,
}

impl BlockPool {
    /// `threads` loader threads (at least one) reading `source`.
    pub fn new(source: Arc<dyn BlockSource>, threads: usize) -> Self {
        let (requests, queue) = channel::<Request>();
        let (done, finished) = channel();
        let queue = Arc::new(Mutex::new(queue));
        let workers = (0..threads.max(1))
            .map(|index| {
                let queue = Arc::clone(&queue);
                let done = done.clone();
                let source = Arc::clone(&source);
                std::thread::Builder::new()
                    .name(format!("ac2-blocks-{index}"))
                    .spawn(move || loop {
                        let next = queue.lock().map(|queue| queue.recv());
                        let Ok(Ok(request)) = next else {
                            return;
                        };
                        if done.send(load(&*source, request)).is_err() {
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

    fn send(&mut self, request: Request) {
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

    /// The next finished work, if any.
    pub fn try_next(&mut self) -> Option<BlockLoaded> {
        let loaded = self.finished.try_recv().ok()?;
        self.outstanding -= 1;
        Some(loaded)
    }

    /// Blocks until a result is ready (tests and probes that wait for the world).
    pub fn wait(&mut self, timeout: Duration) -> Option<BlockLoaded> {
        let loaded = self.finished.recv_timeout(timeout).ok()?;
        self.outstanding -= 1;
        Some(loaded)
    }
}

impl BlockLoader for BlockPool {
    fn request_block(&mut self, block: u32) {
        self.send(Request::Block(block));
    }

    fn request_far_block(&mut self, block: u32) {
        self.send(Request::FarBlock(block));
    }
}

impl Drop for BlockPool {
    fn drop(&mut self) {
        self.requests = None;
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
