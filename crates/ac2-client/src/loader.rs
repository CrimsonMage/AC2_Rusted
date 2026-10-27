//! The client's loader threads (retail `DBCache`'s asynchronous gets): every DAT read and decode
//! runs here, never on the frame thread. A landblock is built into its own `Terrain` that the
//! frame thread later absorbs whole; the frame thread takes finished work each frame within a
//! time budget (`DBCache::UseTime`, which runs the finished requests' callbacks).

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use ac2_physics::Terrain;

use crate::world_data::{BodyData, WorldData};

/// Work for a loader thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// A near landblock with its sub-data (objects, interiors).
    Block(u32),
    /// A far landblock's data alone.
    FarBlock(u32),
    /// The player body for a visual.
    Body(Option<u32>),
    /// An object's script state (`CWDistributor::Construct`'s WState): object, package, DID.
    ScriptState(u64, u32, u32),
    /// A read a script call stopped for (cache-only script data).
    ScriptData(ac2_script::vm::DataRequest),
    /// The region's ambient sound data (`CRegionDesc::GetCurrentSoundInfo`'s `CSoundDesc`).
    AmbientData,
    /// A landblock's ambient sound points (`CLandBlock::CreateAmbientSoundPoints`).
    AmbientPoints(u32),
    /// An object's `MotionInterpDesc` (`CMotionInterp::SetMIDESC`): object, DID.
    MotionDesc(u64, u32),
}

/// Finished work.
pub(crate) enum Loaded {
    /// The block's world, or `None` where the map has no such block.
    Block(u32, Result<Option<Terrain>, String>),
    FarBlock(u32, Result<(), String>),
    Body(Option<u32>, Result<BodyData, String>),
    /// The script state is in the cache (or does not exist): object, package, DID.
    ScriptState(u64, u32, u32),
    /// The script data read is done (or the DATs lack it).
    ScriptData,
    /// The ambient sound's reads.
    Ambient(crate::client_ambient::AmbientLoaded),
    /// An object's motion description: object, and the description (`None` where the DATs
    /// have no such DBObj, or the read failed).
    MotionDesc(
        u64,
        Result<Option<std::sync::Arc<ac2_world::Ac2MotionInterpDesc>>, String>,
    ),
}

/// A pool of loader threads over one `WorldData`.
pub(crate) struct Loader {
    requests: Option<Sender<Request>>,
    finished: Receiver<Loaded>,
    workers: Vec<JoinHandle<()>>,
    /// Work handed out and not yet drained.
    outstanding: usize,
}

fn load(data: &dyn WorldData, request: Request) -> Loaded {
    match request {
        Request::Block(block) => {
            let mut terrain = Terrain::new();
            let result = data
                .load_block(&mut terrain, block)
                .map(|exists| exists.then_some(terrain));
            Loaded::Block(block, result)
        }
        Request::FarBlock(block) => Loaded::FarBlock(block, data.prefetch_far_block(block)),
        Request::Body(visual) => Loaded::Body(visual, data.body(visual)),
        Request::ScriptState(id, pid, did) => {
            data.preload_script_state(pid, did);
            Loaded::ScriptState(id, pid, did)
        }
        Request::ScriptData(read) => {
            data.load_script_data(read);
            Loaded::ScriptData
        }
        Request::AmbientData => Loaded::Ambient(crate::client_ambient::AmbientLoaded::Data(
            data.ambient_data(),
        )),
        Request::AmbientPoints(block) => Loaded::Ambient(
            crate::client_ambient::AmbientLoaded::Points(block, data.ambient_points(block)),
        ),
        Request::MotionDesc(object, did) => Loaded::MotionDesc(object, data.motion_desc(did)),
    }
}

impl Loader {
    /// `threads` loader threads (at least one) reading `data`.
    pub(crate) fn new(data: Arc<dyn WorldData>, threads: usize) -> Self {
        let (requests, queue) = channel::<Request>();
        let (done, finished) = channel();
        let queue = Arc::new(Mutex::new(queue));
        let workers = (0..threads.max(1))
            .map(|index| {
                let queue = Arc::clone(&queue);
                let done = done.clone();
                let data = Arc::clone(&data);
                std::thread::Builder::new()
                    .name(format!("ac2-loader-{index}"))
                    .spawn(move || loop {
                        let next = queue.lock().map(|queue| queue.recv());
                        let Ok(Ok(request)) = next else {
                            return;
                        };
                        if done.send(load(&*data, request)).is_err() {
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

    pub(crate) fn request(&mut self, request: Request) {
        if let Some(requests) = &self.requests {
            if requests.send(request).is_ok() {
                self.outstanding += 1;
            }
        }
    }

    /// Whether any request is still out.
    pub(crate) fn busy(&self) -> bool {
        self.outstanding > 0
    }

    /// The next finished work, if any.
    pub(crate) fn try_next(&mut self) -> Option<Loaded> {
        let loaded = self.finished.try_recv().ok()?;
        self.outstanding -= 1;
        Some(loaded)
    }

    /// Blocks until a result is ready (tests and probes that wait for the world).
    pub(crate) fn wait(&mut self, timeout: Duration) -> Option<Loaded> {
        let loaded = self.finished.recv_timeout(timeout).ok()?;
        self.outstanding -= 1;
        Some(loaded)
    }
}

impl ac2_smartbox::BlockLoader for Loader {
    fn request_block(&mut self, block: u32) {
        self.request(Request::Block(block));
    }

    fn request_far_block(&mut self, block: u32) {
        self.request(Request::FarBlock(block));
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.requests = None;
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
