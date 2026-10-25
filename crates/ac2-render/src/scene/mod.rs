//! The drawn world around a viewer, backend-neutral: which landblocks are resident
//! (`LScapeTerrainMesh::UpdateNearTerrain` over the near ring), the far mesh past them, and per
//! frame what to draw and in what order (`WorldRender::Draw`: the sky before the world, the
//! landscape, the objects and frills, the visible interiors, the sky after). All loading runs on
//! the scene's loader thread; the frame thread only drains finished loads and lays out frames.

mod interiors;
mod loader;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_core::{DataId, Random};
use ac2_math::{Mat4, Vec3};
use ac2_world::Ac2WorldSource;

pub use interiors::{load_block_interiors, BlockInteriors};
pub use loader::{BlockKey, Images, Loaded, LoadedBlock, LoadedFar, LoadedObject, Placement};

use crate::frill::{frill_normal, select_frills, FrillCandidate, FrillSettings};
use crate::frustum::{Frustum, Relation};
use crate::object::RenderSettings;
use crate::sky::{sky_frame, SkyFrame};
use crate::terrain::{far_radius_for_draw_distance, FarTerrain, FIRST_FAR_CHUNK};
use crate::{Camera, SceneLighting};
use loader::{Request, SceneLoader};

/// `GameTime` day length in seconds for the Legions region.
pub const DAY_SECONDS: f64 = 7347.0;
/// Interior groups a frame lays out (the object pass has 16 groups; three are the sky and world).
pub const MAX_INTERIOR_BLOCKS: usize = 13;

/// One placed object of a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FramePlacement {
    pub visual: DataId,
    pub world: Mat4,
    /// A merged frill's normal (`FrillRender::RefreshFrillMesh`).
    pub frill_normal: Option<Vec3>,
}

/// What one frame draws, in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneFrame {
    pub lighting: SceneLighting,
    /// The sky objects before and after the world, and the fog quad between.
    pub sky: SkyFrame,
    /// Scenery and entities, then the frills `RefreshFrillMesh` keeps.
    pub placements: Vec<FramePlacement>,
    /// Resident blocks with near terrain, and those with near water, in a stable order.
    pub terrain: Vec<BlockKey>,
    pub water: Vec<BlockKey>,
    /// Blocks whose visible interiors draw, each under its inside environment.
    pub interiors: Vec<BlockKey>,
    /// Far mesh chunks past the near ring whose box the frustum does not exclude
    /// (`LScapeTerrainMesh::DrawTerrain`).
    pub far_chunks: Vec<usize>,
}

/// What changed since the last drain: loads to upload and blocks that left the ring.
#[derive(Debug, Default)]
pub struct SceneUpdate {
    pub loaded: Vec<Loaded>,
    pub evicted: Vec<BlockKey>,
}

/// A resident block as the frame thread lays it out.
struct BlockInfo {
    placements: Vec<Placement>,
    terrain: bool,
    water: bool,
    interiors: bool,
}

/// The resident world around a centre block.
pub struct WorldScene {
    world: Arc<Ac2WorldSource>,
    loader: SceneLoader,
    radius: i32,
    pub time_of_day: f64,
    center: Option<BlockKey>,
    far_requested: Option<BlockKey>,
    blocks: HashMap<BlockKey, BlockInfo>,
    pending_blocks: HashSet<BlockKey>,
    /// First-list sizes of drawable visuals (`FrillRender` budgets); requested ones pending.
    models: HashMap<DataId, (u32, u32)>,
    pending_models: HashSet<DataId>,
    known_models: HashSet<DataId>,
    outstanding: usize,
    evicted: Vec<BlockKey>,
    /// The loaded far mesh's chunk boxes.
    far_bounds: Vec<(Vec3, Vec3)>,
    /// World objects' models requested by key, and the rig and scale of those that loaded.
    requested_objects: HashSet<u64>,
    object_rigs: HashMap<u64, (crate::object::PoseRig, Vec3)>,
    object_picks: HashMap<u64, Arc<crate::selection::PickMesh>>,
}

fn near(center: BlockKey, block: BlockKey, radius: i32) -> bool {
    (i32::from(block.0) - i32::from(center.0)).abs() <= radius
        && (i32::from(block.1) - i32::from(center.1)).abs() <= radius
}

impl WorldScene {
    /// A scene with a near ring of `radius` blocks (retail's is 1) and the far ring of the
    /// default `LandscapeDrawDistance`.
    pub fn new(world: Arc<Ac2WorldSource>, radius: i32) -> Self {
        let far = Arc::new(FarTerrain::new(
            radius,
            far_radius_for_draw_distance(2),
            16,
            world.block_size(),
            &mut Random::default(),
        ));
        let loader = SceneLoader::new(Arc::clone(&world), far, RenderSettings::default());
        Self {
            world,
            loader,
            radius,
            time_of_day: 0.0,
            center: None,
            far_requested: None,
            blocks: HashMap::new(),
            pending_blocks: HashSet::new(),
            models: HashMap::new(),
            pending_models: HashSet::new(),
            known_models: HashSet::new(),
            outstanding: 0,
            evicted: Vec::new(),
            far_bounds: Vec::new(),
            requested_objects: HashSet::new(),
            object_rigs: HashMap::new(),
            object_picks: HashMap::new(),
        }
    }

    pub fn world(&self) -> &Arc<Ac2WorldSource> {
        &self.world
    }

    fn request(&mut self, request: Request) {
        self.outstanding += 1;
        self.loader.request(request);
    }

    /// Centres the ring on `center`: blocks outside it leave, missing ones and the far mesh
    /// around the new centre are requested.
    pub fn set_center(&mut self, center: BlockKey) {
        let radius = self.radius;
        let leaving: Vec<BlockKey> = self
            .blocks
            .keys()
            .copied()
            .filter(|block| !near(center, *block, radius))
            .collect();
        for block in leaving {
            self.blocks.remove(&block);
            self.evicted.push(block);
        }
        self.center = Some(center);
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let (x, y) = (i32::from(center.0) + dx, i32::from(center.1) + dy);
                let (Ok(x), Ok(y)) = (u8::try_from(x), u8::try_from(y)) else {
                    continue;
                };
                if !self.blocks.contains_key(&(x, y)) && self.pending_blocks.insert((x, y)) {
                    self.request(Request::Block((x, y)));
                }
            }
        }
        if self.far_requested != Some(center) {
            self.far_requested = Some(center);
            self.request(Request::Far(center));
        }
    }

    /// Takes the finished loads, for at most `budget` (the per-frame time budget of
    /// `DBCache::UseTime`'s pump).
    pub fn poll(&mut self, budget: Duration) -> SceneUpdate {
        let start = Instant::now();
        let mut update = SceneUpdate {
            evicted: std::mem::take(&mut self.evicted),
            ..SceneUpdate::default()
        };
        while start.elapsed() < budget {
            let Ok(loaded) = self.loader.finished.try_recv() else {
                break;
            };
            self.absorb(loaded, &mut update);
        }
        update
    }

    /// Takes every finished load, waiting until none is outstanding (headless snapshots).
    pub fn wait(&mut self) -> SceneUpdate {
        let mut update = SceneUpdate {
            evicted: std::mem::take(&mut self.evicted),
            ..SceneUpdate::default()
        };
        while self.outstanding > 0 {
            let Ok(loaded) = self.loader.finished.recv() else {
                break;
            };
            self.absorb(loaded, &mut update);
        }
        update
    }

    fn note_model(&mut self, visual: DataId, model: Option<&crate::object::ObjectModel>) {
        self.known_models.insert(visual);
        self.pending_models.remove(&visual);
        if let Some(model) = model {
            self.models.insert(visual, model.first_list());
        }
    }

    fn absorb(&mut self, loaded: Loaded, update: &mut SceneUpdate) {
        self.outstanding = self.outstanding.saturating_sub(1);
        match &loaded {
            Loaded::Block(block) => {
                self.pending_blocks.remove(&block.key);
                for (visual, model) in &block.models {
                    self.note_model(*visual, model.as_ref());
                }
                let in_ring = self
                    .center
                    .is_some_and(|center| near(center, block.key, self.radius));
                if !in_ring {
                    // Its models stay known; the block itself is dropped.
                    update.loaded.push(Loaded::Block(LoadedBlock {
                        terrain: None,
                        water: None,
                        placements: Vec::new(),
                        interiors: None,
                        ..block.clone()
                    }));
                    update.evicted.push(block.key);
                    return;
                }
                self.blocks.insert(
                    block.key,
                    BlockInfo {
                        placements: block.placements.clone(),
                        terrain: block.terrain.is_some(),
                        water: block.water.is_some(),
                        interiors: block.interiors.is_some(),
                    },
                );
            }
            Loaded::Model(visual, model, _) => self.note_model(*visual, model.as_ref()),
            Loaded::Far(far) => self.far_bounds = far.layout.chunk_bounds(&far.vertices),
            Loaded::Object(object) => {
                if let Some(rig) = object.model.as_ref().and_then(|model| model.rig()) {
                    self.object_rigs.insert(object.key, (rig, object.scale));
                }
                if let Some(pick) = &object.pick {
                    self.object_picks.insert(object.key, Arc::clone(pick));
                }
            }
        }
        update.loaded.push(loaded);
    }

    /// Loads a world object's model off the frame thread: `visual`'s DAT chain with `local` (the
    /// object's local `VisualDesc`) on top. `key` names the result; a key already asked for is
    /// not loaded again.
    pub fn request_object(
        &mut self,
        key: u64,
        visual: DataId,
        local: Option<ac2_assets::Ac2VisualDesc>,
    ) {
        if self.requested_objects.insert(key) {
            self.request(Request::Object(key, visual, local));
        }
    }

    /// The posing rig and total scale of a loaded world object's model.
    pub fn object_rig(&self, key: u64) -> Option<&(crate::object::PoseRig, Vec3)> {
        self.object_rigs.get(&key)
    }

    /// The object model `key`'s pick mesh, once loaded.
    pub fn object_pick(&self, key: u64) -> Option<&Arc<crate::selection::PickMesh>> {
        self.object_picks.get(&key)
    }

    /// The region's lighting at the scene's time of day.
    pub fn lighting(&self) -> SceneLighting {
        self.world
            .region()
            .and_then(|region| region.sample_sky_environment(self.time_of_day))
            .map(|sample| SceneLighting::from_sky(&sample))
            .unwrap_or_default()
    }

    /// Starts at the brightest daylight of the region's day.
    pub fn set_daytime(&mut self) {
        let Some(region) = self.world.region() else {
            return;
        };
        let brightness = |time: f64| {
            region
                .sample_sky_environment(time)
                .map(|sample| {
                    SceneLighting::from_sky(&sample)
                        .fog_color
                        .iter()
                        .sum::<f32>()
                })
                .unwrap_or(0.0)
        };
        self.time_of_day = (0..48)
            .map(|step| f64::from(step) * DAY_SECONDS / 48.0)
            .max_by(|a, b| brightness(*a).total_cmp(&brightness(*b)))
            .unwrap_or(0.0);
    }

    /// Lays out a frame seen by `camera` at `aspect`: requests sky objects not yet loaded.
    pub fn frame(&mut self, camera: &Camera, aspect: f32) -> SceneFrame {
        let eye = camera.eye;
        let frustum = Frustum::from_camera(camera, aspect);
        let far_chunks = self
            .far_bounds
            .iter()
            .enumerate()
            .skip(FIRST_FAR_CHUNK)
            .filter(|(_, (min, max))| frustum.check(*min, *max) != Relation::Outside)
            .map(|(index, _)| index)
            .collect();
        let lighting = self.lighting();
        let mut keys: Vec<BlockKey> = self.blocks.keys().copied().collect();
        keys.sort_unstable();
        let all = || {
            keys.iter()
                .flat_map(|key| self.blocks[key].placements.iter())
                .filter(|p| self.models.contains_key(&p.visual))
        };
        let frills: Vec<&Placement> = all().filter(|p| p.frill).collect();
        let candidates: Vec<FrillCandidate> = frills
            .iter()
            .map(|p| {
                let (indices, vertices) = self.models[&p.visual];
                FrillCandidate {
                    world: p.world,
                    indices,
                    vertices,
                }
            })
            .collect();
        let chosen = select_frills(&candidates, eye, &FrillSettings::default());
        let placements: Vec<FramePlacement> = all()
            .filter(|p| !p.frill)
            .map(|p| FramePlacement {
                visual: p.visual,
                world: p.world,
                frill_normal: None,
            })
            .chain(chosen.iter().map(|&index| FramePlacement {
                visual: frills[index].visual,
                world: frills[index].world,
                frill_normal: Some(frill_normal(&frills[index].world)),
            }))
            .collect();
        let sky_objects = self
            .world
            .region()
            .and_then(|region| region.sky_objects(self.time_of_day))
            .unwrap_or_default();
        let sky = sky_frame(
            &sky_objects,
            eye,
            false,
            lighting.fog_color,
            lighting.fog_sky,
        );
        let wanted: Vec<DataId> = sky
            .before
            .iter()
            .chain(&sky.after)
            .map(|p| p.gfx)
            .filter(|gfx| !self.known_models.contains(gfx) && !self.pending_models.contains(gfx))
            .collect();
        for gfx in wanted {
            if self.pending_models.insert(gfx) {
                self.request(Request::Model(gfx));
            }
        }
        let terrain = keys
            .iter()
            .copied()
            .filter(|k| self.blocks[k].terrain)
            .collect();
        let water = keys
            .iter()
            .copied()
            .filter(|k| self.blocks[k].water)
            .collect();
        let mut interiors: Vec<BlockKey> = keys
            .iter()
            .copied()
            .filter(|k| self.blocks[k].interiors)
            .collect();
        interiors.truncate(MAX_INTERIOR_BLOCKS);
        SceneFrame {
            lighting,
            sky,
            placements,
            terrain,
            water,
            interiors,
            far_chunks,
        }
    }
}
