//! The client's loader threads (retail `DBCache`'s asynchronous gets): every DAT read and decode
//! runs here, never on the frame thread. A landblock is built into its own `Terrain` that the
//! frame thread later absorbs whole; the frame thread takes finished work each frame within a
//! time budget (`DBCache::UseTime`, which runs the finished requests' callbacks).

use std::sync::Arc;
use std::time::Duration;

use ac2_db::LoaderPool;
use ac2_physics::Terrain;

use crate::world_data::{BodyData, WorldData};

/// A local description's global appearance: appearance table DID -> its keys and scalars.
pub(crate) type GlobalAppearance = Vec<(u32, Vec<(u32, f32)>)>;

/// Work for a loader thread.
#[derive(Debug, Clone)]
pub(crate) enum Request {
    /// A near landblock with its sub-data (objects, interiors).
    Block(u32),
    /// A far landblock's data alone.
    FarBlock(u32),
    /// The player body for a visual.
    Body(Option<u32>),
    /// An object's script state (`CWDistributor::Construct`'s WState): object, package, DID, and
    /// the DID of its qualities' class defaults (`CBaseQualities::InitDefaults`; 0 for none).
    ScriptState(u64, u32, u32, u32),
    /// A read a script call stopped for (cache-only script data).
    ScriptData(ac2_script::vm::DataRequest),
    /// The DAT halves of an object's total icon description (`VisualDesc::GetTotalIconDesc`,
    /// which `ItemDisplay::Update` draws): object, the visual DID, and the local description's
    /// global appearance (tables, keys and scalars) whose icon atoms are read.
    TotalIcon(u64, u32, GlobalAppearance),
    /// The region's ambient sound data (`CRegionDesc::GetCurrentSoundInfo`'s `CSoundDesc`).
    AmbientData,
    /// A landblock's ambient sound points (`CLandBlock::CreateAmbientSoundPoints`).
    AmbientPoints(u32),
    /// An object's `MotionInterpDesc` (`CMotionInterp::SetMIDESC`): object, DID.
    MotionDesc(u64, u32),
    /// An object's `EntityDesc` ethereal bits (`CPhysicsObj::InitVisualDesc`): object, DID.
    EntityMask(u64, u32),
    /// A camera effect (`CameraManager::DoCameraFX`): DID, the scalar's bits, the source.
    CameraFx(u32, u32, u64),
    /// `SmartBox::Init`'s camera object shape.
    CameraObject,
    /// An object's part tree followed to its changed description (`CPhysicsObj::ApplyVisualDesc`):
    /// object, the description's structure key, and the job.
    PartTree(u64, u64, Box<PartTreeJob>),
}

/// What the part-tree rebuild needs: the body's tree, its visual and the local description.
#[derive(Debug, Clone)]
pub(crate) struct PartTreeJob {
    pub tree: ac2_physics::PartGroupTree,
    pub visual: u32,
    pub local: ac2_assets::Ac2VisualDesc,
    /// The body's behaviour table DID (`PHYSICSOBJREP::m_behaviorTableDID`).
    pub current_table: u32,
}

/// Finished work.
pub(crate) enum Loaded {
    /// The block's world, or `None` where the map has no such block.
    Block(u32, Result<Option<Terrain>, String>),
    FarBlock(u32, Result<(), String>),
    Body(Option<u32>, Result<BodyData, String>),
    /// The script state is in the cache (or does not exist): object, package, DID.
    ScriptState(u64, u32, u32, Option<Arc<ac2_weenie::WeenieQualities>>),
    /// An object's chain icon layers and global icon atoms, with the global appearance asked for.
    TotalIcon(
        u64,
        GlobalAppearance,
        Vec<(u32, u32, u32)>,
        Vec<(u32, u32, u32)>,
    ),
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
    /// A camera effect's record: the source, the scalar, and the record (`None` where the DATs
    /// have none).
    CameraFx(
        u64,
        f32,
        Result<Option<ac2_smartbox::camera_fx::CameraFx>, String>,
    ),
    /// The camera object's collision shape (`None` where the DATs have none).
    CameraObject(Result<Option<ac2_physics::BodyShape>, String>),
    /// A rebuilt part tree: object, the structure key it was asked for, the tree (`None` where
    /// nothing was followed).
    PartTree(u64, u64, Result<Option<ac2_physics::RebuiltVisual>, String>),
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
        Request::ScriptState(id, pid, did, defaults) => {
            data.preload_script_state(pid, did);
            // `CBaseQualities::InitDefaults` read the class defaults when the qualities were
            // made; resident before the process starts, the script's reads find them as
            // retail's did instead of waiting on the loader mid-call.
            let defaults = (defaults != 0)
                .then(|| {
                    data.load_script_data(ac2_script::vm::DataRequest::ClassDefaults(defaults));
                    data.class_defaults(defaults)
                })
                .flatten();
            Loaded::ScriptState(id, pid, did, defaults)
        }
        Request::TotalIcon(id, visual, global) => {
            let (chain, atoms) = data.total_icon(visual, &global);
            Loaded::TotalIcon(id, global, chain, atoms)
        }
        Request::ScriptData(read) => Loaded::ScriptData(data.load_script_data(read)),
        Request::AmbientData => Loaded::Ambient(crate::client_ambient::AmbientLoaded::Data(
            data.ambient_data(),
        )),
        Request::AmbientPoints(block) => Loaded::Ambient(
            crate::client_ambient::AmbientLoaded::Points(block, data.ambient_points(block)),
        ),
        Request::MotionDesc(object, did) => Loaded::MotionDesc(object, data.motion_desc(did)),
        Request::CameraFx(did, scalar, source) => {
            Loaded::CameraFx(source, f32::from_bits(scalar), data.camera_fx(did))
        }
        Request::CameraObject => Loaded::CameraObject(data.camera_object_shape()),
        Request::PartTree(object, key, job) => Loaded::PartTree(
            object,
            key,
            data.rebuild_parts(&job.tree, job.visual, &job.local, job.current_table),
        ),
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
