//! The client's loader threads (retail `DBCache`'s asynchronous gets): every DAT read and decode
//! runs here, never on the frame thread. A landblock is built into its own `Terrain` that the
//! frame thread later absorbs whole; the frame thread takes finished work each frame within a
//! time budget (`DBCache::UseTime`, which runs the finished requests' callbacks).

use std::sync::Arc;
use std::time::Duration;

use ac2_db::LoaderPool;
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
    /// An object's `EntityDesc` ethereal bits (`CPhysicsObj::InitVisualDesc`): object, DID.
    EntityMask(u64, u32),
}

/// Finished work.
pub(crate) enum Loaded {
    /// The block's world, or `None` where the map has no such block.
    Block(u32, Result<Option<Terrain>, String>),
    FarBlock(u32, Result<(), String>),
    Body(Option<u32>, Result<BodyData, String>),
    /// The script state is in the cache (or does not exist): object, package, DID.
    ScriptState(u64, u32, u32),
    /// The script data read is done: `true` when the world answered it (the record is resident
    /// or known absent), `false` when the world never answers (its reads are dropped).
    ScriptData(bool),
    /// The ambient sound's reads.
    Ambient(crate::client_ambient::AmbientLoaded),
    /// An object's motion description: object, and the description (`None` where the DATs
    /// have no such DBObj, or the read failed).
    MotionDesc(
        u64,
        Result<Option<std::sync::Arc<ac2_world::Ac2MotionInterpDesc>>, String>,
    ),
    /// An object's `EntityDesc` ethereal bits: object, and the bits or the read's error.
    EntityMask(u64, Result<u64, String>),
}

/// The client's loader threads over one `WorldData` (`ac2_db::LoaderPool`, named `ac2-loader`).
pub(crate) struct Loader {
    pool: LoaderPool<Request, Loaded>,
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
        Request::ScriptData(read) => Loaded::ScriptData(data.load_script_data(read)),
        Request::AmbientData => Loaded::Ambient(crate::client_ambient::AmbientLoaded::Data(
            data.ambient_data(),
        )),
        Request::AmbientPoints(block) => Loaded::Ambient(
            crate::client_ambient::AmbientLoaded::Points(block, data.ambient_points(block)),
        ),
        Request::MotionDesc(object, did) => Loaded::MotionDesc(object, data.motion_desc(did)),
        Request::EntityMask(object, did) => {
            Loaded::EntityMask(object, data.entity_ethereal_mask(did))
        }
    }
}

impl Loader {
    /// `threads` loader threads (at least one) reading `data`.
    pub(crate) fn new(data: Arc<dyn WorldData>, threads: usize) -> Self {
        let work = Arc::new(move |request: Request| load(&*data, request));
        Self {
            pool: LoaderPool::new("ac2-loader", threads, work),
        }
    }

    pub(crate) fn request(&mut self, request: Request) {
        self.pool.request(request);
    }

    /// Whether any request is still out.
    pub(crate) fn busy(&self) -> bool {
        self.pool.busy()
    }

    /// The next finished work, if any.
    pub(crate) fn try_next(&mut self) -> Option<Loaded> {
        self.pool.try_next()
    }

    /// Blocks until a result is ready (tests and probes that wait for the world).
    pub(crate) fn wait(&mut self, timeout: Duration) -> Option<Loaded> {
        self.pool.wait(timeout)
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
