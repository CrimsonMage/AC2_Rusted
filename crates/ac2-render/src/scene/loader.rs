//! The scene's loader thread: every DAT read and decode the drawn world needs runs here, never on
//! the frame thread (retail `DBCache`'s asynchronous gets feeding `LScapeTerrainMesh` and
//! `CLandBlock`). One thread serves the requests in order, so an image it sends once arrives
//! before every later load that samples it.

use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use ac2_core::{CellId, DataId};
use ac2_math::Mat4;
use ac2_world::Ac2WorldSource;

use super::interiors::{load_block_interiors, BlockInteriors};
use crate::object::{load_object_model, load_water_model, ObjectModel, RenderSettings};
use crate::terrain::{build_terrain_mesh, FarTerrain, FarVertices, TerrainMesh};
use crate::{load_image, Image, TextureKey};

/// Images decoded alongside a load: each key the loader had not sent before (`None` where the
/// DAT has no usable image).
pub type Images = Vec<(TextureKey, Option<Image>)>;

/// A landblock key: its x and y in the landblock grid.
pub type BlockKey = (u8, u8);

/// One `CLandBlockInfo` placement: what it draws and where.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub visual: DataId,
    pub world: Mat4,
    /// A grass frill (`FrillRender` merges and budgets these).
    pub frill: bool,
}

/// A loaded near landblock.
#[derive(Debug, Clone)]
pub struct LoadedBlock {
    pub key: BlockKey,
    /// The landblock edge in metres.
    pub block_size: f32,
    pub terrain: Option<TerrainMesh>,
    /// Near water (`LScapeTerrainMesh::UpdateNearWater`), already in world space.
    pub water: Option<ObjectModel>,
    /// Placements whose visual draws.
    pub placements: Vec<Placement>,
    /// The models of visuals this load saw first (`None` for a visual that draws nothing).
    pub models: Vec<(DataId, Option<ObjectModel>)>,
    /// Interior cells visible from outside.
    pub interiors: Option<BlockInteriors>,
    pub images: Images,
}

/// The far mesh around one centre block (`BuildFarTerrainMesh`).
#[derive(Debug, Clone)]
pub struct LoadedFar {
    pub center: BlockKey,
    pub layout: Arc<FarTerrain>,
    pub vertices: FarVertices,
    /// `Ac2FarTerrain::texture`.
    pub texture: DataId,
    pub water: Option<ObjectModel>,
    pub images: Images,
}

/// Finished work.
#[derive(Debug, Clone)]
pub enum Loaded {
    Block(LoadedBlock),
    /// A visual the frame asked for (sky objects).
    Model(DataId, Option<ObjectModel>, Images),
    Far(LoadedFar),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    Block(BlockKey),
    Model(DataId),
    Far(BlockKey),
}

/// The loader thread and its queues.
pub(crate) struct SceneLoader {
    requests: Option<Sender<Request>>,
    pub(crate) finished: Receiver<Loaded>,
    worker: Option<JoinHandle<()>>,
}

pub(crate) fn block_id((x, y): BlockKey) -> CellId {
    CellId::new((u32::from(x) << 24) | (u32::from(y) << 16) | 1)
}

/// The worker's state: what it has already sent.
struct Worker {
    world: Arc<Ac2WorldSource>,
    far: Arc<FarTerrain>,
    settings: RenderSettings,
    models: HashSet<DataId>,
    /// Visuals whose model draws.
    drawable: HashSet<DataId>,
    images: HashSet<TextureKey>,
}

impl Worker {
    fn images(&mut self, keys: impl IntoIterator<Item = TextureKey>) -> Images {
        let store = self.world.resource_store().clone();
        keys.into_iter()
            .filter(|key| self.images.insert(*key))
            .map(|key| (key, load_image(&store, key).ok().flatten()))
            .collect()
    }

    fn model(&mut self, visual: DataId) -> Option<ObjectModel> {
        let store = self.world.resource_store().clone();
        load_object_model(&store, visual, &self.settings)
            .ok()
            .flatten()
    }

    fn block(&mut self, key: BlockKey) -> LoadedBlock {
        let world = Arc::clone(&self.world);
        let store = world.resource_store().clone();
        let id = block_id(key);
        let landblock = world.load_landblock(id).ok().flatten();
        let terrain = landblock
            .as_ref()
            .map(|landblock| build_terrain_mesh(id, landblock, world.block_size()));
        let water = landblock
            .as_ref()
            .and_then(|landblock| world.block_water(id, landblock))
            .and_then(|water| {
                load_water_model(&store, id, &water, world.block_size(), &self.settings)
                    .ok()
                    .flatten()
            });
        let mut models = Vec::new();
        let mut placements = Vec::new();
        for marker in world.entity_markers(id).iter().flat_map(|m| m.iter()) {
            let visual = marker.visual();
            if self.models.insert(visual) {
                let model = self.model(visual);
                if model.is_some() {
                    self.drawable.insert(visual);
                }
                models.push((visual, model));
            }
            if !self.drawable.contains(&visual) {
                continue;
            }
            placements.push(Placement {
                visual,
                world: marker.world_frame().to_mat4() * Mat4::from_scale(marker.scale()),
                frill: marker.frill(),
            });
        }
        let mut interiors = load_block_interiors(&world, &store, id, &self.settings);
        if let Some(interiors) = interiors.as_mut() {
            for (visual, _, _) in &interiors.objects {
                if self.models.insert(*visual) {
                    let model = self.model(*visual);
                    if model.is_some() {
                        self.drawable.insert(*visual);
                    }
                    models.push((*visual, model));
                }
            }
            let drawable = &self.drawable;
            interiors
                .objects
                .retain(|(visual, _, _)| drawable.contains(visual));
        }
        let mut keys: Vec<TextureKey> =
            terrain.iter().flat_map(TerrainMesh::texture_keys).collect();
        keys.extend(water.iter().flat_map(ObjectModel::texture_keys));
        keys.extend(
            models
                .iter()
                .filter_map(|(_, model)| model.as_ref())
                .flat_map(ObjectModel::texture_keys),
        );
        keys.extend(
            interiors
                .iter()
                .flat_map(|interiors| interiors.cells.iter())
                .flat_map(|(model, _)| model.texture_keys()),
        );
        let images = self.images(keys);
        LoadedBlock {
            key,
            block_size: world.block_size(),
            terrain,
            water,
            placements,
            models,
            interiors,
            images,
        }
    }

    fn far(&mut self, center: BlockKey) -> LoadedFar {
        let world = Arc::clone(&self.world);
        let store = world.resource_store().clone();
        let desc = world.far_terrain();
        let at = (i32::from(center.0), i32::from(center.1));
        let vertices = self.far.vertices(&world, at, &desc);
        let water = self
            .far
            .water_model(&store, &desc, &vertices, &self.settings)
            .ok()
            .flatten();
        let mut keys = vec![TextureKey::Texture(desc.texture)];
        keys.extend(water.iter().flat_map(ObjectModel::texture_keys));
        let images = self.images(keys);
        LoadedFar {
            center,
            layout: Arc::clone(&self.far),
            vertices,
            texture: desc.texture,
            water,
            images,
        }
    }

    fn run(&mut self, request: Request) -> Loaded {
        match request {
            Request::Block(key) => Loaded::Block(self.block(key)),
            Request::Model(visual) => {
                let model = self
                    .models
                    .insert(visual)
                    .then(|| self.model(visual))
                    .flatten();
                if model.is_some() {
                    self.drawable.insert(visual);
                }
                let images = self.images(model.iter().flat_map(ObjectModel::texture_keys));
                Loaded::Model(visual, model, images)
            }
            Request::Far(center) => Loaded::Far(self.far(center)),
        }
    }
}

impl SceneLoader {
    pub(crate) fn new(
        world: Arc<Ac2WorldSource>,
        far: Arc<FarTerrain>,
        settings: RenderSettings,
    ) -> Self {
        let (requests, queue) = channel::<Request>();
        let (done, finished) = channel();
        let mut worker = Worker {
            world,
            far,
            settings,
            models: HashSet::new(),
            drawable: HashSet::new(),
            images: HashSet::new(),
        };
        let worker = std::thread::Builder::new()
            .name("ac2-scene-loader".into())
            .spawn(move || {
                while let Ok(request) = queue.recv() {
                    if done.send(worker.run(request)).is_err() {
                        return;
                    }
                }
            })
            .expect("spawn the scene loader thread");
        Self {
            requests: Some(requests),
            finished,
            worker: Some(worker),
        }
    }

    pub(crate) fn request(&self, request: Request) {
        if let Some(requests) = &self.requests {
            // A loader that has stopped leaves the request unanswered.
            let _ = requests.send(request);
        }
    }
}

impl Drop for SceneLoader {
    fn drop(&mut self) {
        self.requests = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
