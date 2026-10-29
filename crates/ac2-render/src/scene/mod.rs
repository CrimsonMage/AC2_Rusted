//! The drawn world around a viewer, backend-neutral: which landblocks are resident
//! (`LScapeTerrainMesh::UpdateNearTerrain` over the near ring), the far mesh past them, and per
//! frame what to draw and in what order (`WorldRender::Draw`: the sky before the world, the
//! landscape, the objects and frills, the visible interiors, the sky after). All loading runs on
//! the scene's loader thread; the frame thread only drains finished loads and lays out frames.

mod decals;
mod grass;
mod interiors;
mod light_decals;
mod lighting;
mod loader;
mod lod;
mod object_lights;
mod particles;
pub(crate) mod shore_strips;
mod skeleton_effects;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_anim::Ac2Transform;
use ac2_assets::Ac2FxLight;
use ac2_core::{DataId, Random};
use ac2_math::{Frame, Mat4, Vec3};
use ac2_world::Ac2WorldSource;

pub use interiors::{load_block_interiors, BlockInteriors, CellCorona};
pub use loader::{
    BlockKey, CloudShadow, Images, Loaded, LoadedBlock, LoadedFar, LoadedObject,
    LoadedObjectParticle, ParticleSource, Placement,
};
pub use lod::PlacedDetail;
pub use particles::{ParticleAnchor, ParticleBatch};

use crate::frill::{select_far_frills, select_frills, FrillCandidate, FrillSettings, GrassRing};
use crate::frustum::{Frustum, Relation};
use crate::interior_cull::{check_visibility, BlockCull, InteriorCull};
use crate::object::{DegradeOptions, DrawDetail, ObjectPose, PointLight, RenderSettings};
use crate::particles::{CameraBasis, ParticleSystem};
use crate::prefs::SceneOptions;
use crate::rain::{RainField, RAIN_DRAW_THRESHOLD};
use crate::sky::{sky_frame, SkyFrame};
use crate::streak_objects::ObjectStreaks;
use crate::terrain::{shadow_light_bit, FarTerrain, TerrainHeights, FIRST_FAR_CHUNK};
use crate::TextureKey;
use crate::{Camera, SceneLighting};
use loader::{Request, SceneLoader};
use object_lights::ObjectLights;
use particles::SceneParticles;

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
    /// A merged frill's landscape self shadow under its origin (`GetShadowColor`'s self term);
    /// 1 where nothing shadows it.
    pub frill_shade: f32,
    /// The degrade its parts draw at (`CPartArray::Draw`); full for frills.
    pub detail: DrawDetail,
    /// The distance fade, multiplied into its colour's alpha; 1 for frills.
    pub alpha: f32,
    /// A frill's growth ring (`frill::GrassRing`); zero for everything else.
    pub grass: GrassRing,
}

/// Where an object stands this frame, for the particle systems it runs: its emitter (the object's
/// id), its frame, its total scale (`CPartArray::QueueParticleSystems`'s object matrix), its root
/// Setup's posed bones (the holding points attach to these) and every part group's bones by name
/// (the bone attachments search them, `CPartGroup::FindBoneType`).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPlacement {
    pub emitter: u64,
    /// The object's cell (`CPartArray::Draw`'s `_CellID`), for its streaks' base cell.
    pub cell: u32,
    pub frame: Frame,
    pub scale: Vec3,
    pub bones: Vec<Ac2Transform>,
    pub named_bones: Vec<(u32, Ac2Transform)>,
    /// Each of `bones`' parent (`CSetup`'s), for the body's skeleton points
    /// (`CPartArray::CalculateSkeletonPoints`).
    pub parents: Arc<[Option<usize>]>,
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
    /// The env cells of each of those blocks that draw (`CEnvCell::CullCells`); a block missing
    /// here draws all of its cells.
    pub interior_cells: HashMap<BlockKey, Vec<u32>>,
    /// Far mesh chunks past the near ring whose box the frustum does not exclude
    /// (`LScapeTerrainMesh::DrawTerrain`).
    pub far_chunks: Vec<usize>,
    /// The entities' particles, merged per material (`PSManager::RenderParticleSystems`).
    pub particles: Vec<ParticleBatch>,
    /// The coronas of the lights of the interior cells that draw (`CEnvCell::DrawPostEffects`,
    /// `RenderLightCoronas`), with their block.
    pub coronas: Vec<(BlockKey, CellCorona)>,
    /// The water the frame reflects (`LScapeRender::FindReflectionPlane`), when there is one.
    pub reflection: Option<ReflectionPlane>,
    /// The breaking water on the shores, drawn after the water (`LScapeTerrainMesh::DrawWater`
    /// then `DrawShore(true)`).
    pub shore: Option<ParticleBatch>,
}

/// The frame's reflecting water: its height, the `WaterReflections` level, and the resident
/// blocks whose water lies on it (retail stencilled the water mesh's pixels).
#[derive(Debug, Clone, PartialEq)]
pub struct ReflectionPlane {
    pub height: f32,
    pub level: u8,
    pub blocks: Vec<BlockKey>,
    /// The landblock edge in metres.
    pub block_size: f32,
}

/// What changed since the last drain: loads to upload and blocks that left the ring.
#[derive(Debug, Default)]
pub struct SceneUpdate {
    pub loaded: Vec<Loaded>,
    pub evicted: Vec<BlockKey>,
}

/// A world object to draw this frame (a character or item placed by the client): its loaded
/// model's key (`WorldScene::request_object`), world matrix, pose, tint and the point lights it
/// is lit by (`SceneTool::ApplyLights`; none outdoors in the world).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneObject {
    pub key: u64,
    pub world: Mat4,
    pub pose: Option<ObjectPose>,
    pub tint: [f32; 4],
    pub point_lights: Vec<PointLight>,
    /// The degrade its parts draw at (`WorldScene::object_detail`).
    pub detail: DrawDetail,
}

/// A resident block as the frame thread lays it out.
struct BlockInfo {
    placements: Vec<Placement>,
    terrain: bool,
    water: bool,
    interiors: bool,
    cull: Option<Arc<BlockCull>>,
    coronas: Option<Arc<[CellCorona]>>,
    cell_lights: Option<Arc<[(u32, crate::object::PointLight)]>>,
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
    /// Images asked for on their own (lens flare sprites), each once.
    requested_images: HashSet<TextureKey>,
    outstanding: usize,
    evicted: Vec<BlockKey>,
    /// The loaded far mesh's chunk boxes.
    far_bounds: Vec<(Vec3, Vec3)>,
    /// The far ring's radius in blocks (`LandscapeDrawDistance`) and the frill settings the frame
    /// selects with (`LandscapeFrillDistance`).
    far_radius: i32,
    frill: FrillSettings,
    /// World objects' models requested by key, and the rig and scale of those that loaded.
    requested_objects: HashSet<u64>,
    object_rigs: HashMap<u64, (crate::object::PoseRig, Vec3)>,
    /// The loaded models' `DegradeInfo`, the degrade settings and the frame rate.
    degrades: lod::SceneDegrades,
    object_picks: HashMap<u64, Arc<crate::selection::PickMesh>>,
    particles: SceneParticles,
    /// The point lights of object FX (`CPartArray::CreateLightSource`).
    object_lights: ObjectLights,
    /// The emitters' streaks (`CPartArray::CreateStreakEffect`) and whether they draw
    /// (`Render::m_Prefs.StreakEffects`: `StreakEffectManager::RenderStreakEffects` skips them).
    object_streaks: ObjectStreaks,
    streaks_visible: bool,
    /// The decals the FX project (`DecalEffectManager`).
    decals: crate::decal::DecalEffects,
    /// The resident blocks' ground, for the rain.
    heights: HashMap<BlockKey, Arc<TerrainHeights>>,
    rain: RainField,
    /// The rain's material once loaded (`LScapeRender::m_pRainMaterial`).
    rain_material: Option<DataId>,
    /// The light pools' materials at day, dusk and night, once loaded, and whether they draw
    /// (`LightSourceEffects`).
    light_decal_materials: Option<[DataId; 3]>,
    /// The shore's material once loaded, `LandscapeShoreEffects`, and the strips of the near
    /// blocks they were made for.
    shore_material: Option<DataId>,
    shore_effects: u8,
    shore: Option<shore_strips::ShoreCache>,
    light_source_effects: bool,
    /// `WaterReflections`.
    water_reflections: u8,
    /// `WorldRender`'s rain splash and water ripple systems (no fade, at the world's origin).
    rain_splash: Option<ParticleSystem>,
    rain_ripples: Option<ParticleSystem>,
    /// `m_pWaterBubblesParticleSystem`.
    water_bubbles: Option<ParticleSystem>,
    /// The frame's placed bodies, for their skeleton effects, and the dice those roll.
    bodies: Vec<ObjectPlacement>,
    effects_random: Random,
    started: Instant,
}

fn near(center: BlockKey, block: BlockKey, radius: i32) -> bool {
    (i32::from(block.0) - i32::from(center.0)).abs() <= radius
        && (i32::from(block.1) - i32::from(center.1)).abs() <= radius
}

impl WorldScene {
    /// A scene with a near ring of `radius` blocks (retail's is 1) and the far ring of the
    /// default `LandscapeDrawDistance`.
    pub fn new(world: Arc<Ac2WorldSource>, radius: i32) -> Self {
        let defaults = SceneOptions::default();
        let far = Arc::new(FarTerrain::new(
            radius,
            defaults.far_radius,
            16,
            world.block_size(),
            &mut Random::default(),
        ));
        let loader = SceneLoader::new(Arc::clone(&world), far, RenderSettings::default());
        let mut scene = Self {
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
            requested_images: HashSet::new(),
            outstanding: 0,
            evicted: Vec::new(),
            far_bounds: Vec::new(),
            far_radius: defaults.far_radius,
            frill: defaults.frill,
            requested_objects: HashSet::new(),
            object_rigs: HashMap::new(),
            degrades: lod::SceneDegrades::default(),
            object_picks: HashMap::new(),
            particles: SceneParticles::new(),
            object_lights: ObjectLights::default(),
            object_streaks: ObjectStreaks::default(),
            streaks_visible: true,
            decals: crate::decal::DecalEffects::default(),
            heights: HashMap::new(),
            rain: RainField::default(),
            rain_material: None,
            light_decal_materials: None,
            shore_material: None,
            shore_effects: defaults.shore_effects,
            shore: None,
            light_source_effects: defaults.light_source_effects,
            water_reflections: defaults.water_reflections,
            rain_splash: None,
            rain_ripples: None,
            water_bubbles: None,
            bodies: Vec::new(),
            effects_random: Random::default(),
            started: Instant::now(),
        };
        scene.request(Request::Rain);
        scene.request(Request::LightDecalMaterials);
        scene.request(Request::ShoreMaterial);
        scene
    }

    /// `WorldScene`'s graphics options (`SmartBox::InitPreferences` and the preference monitors):
    /// the frill distance applies to the next frame. A changed landscape draw distance lays the far
    /// ring out again: the new layout is queued to the loader, the far chunks it loads replace the
    /// old ones, and until then no far chunk is drawn. The near ring and the objects stay.
    pub fn set_options(&mut self, options: SceneOptions) {
        self.frill = options.frill;
        self.light_source_effects = options.light_source_effects;
        self.water_reflections = options.water_reflections;
        self.shore_effects = options.shore_effects;
        self.degrades.options = options.degrade;
        if options.far_radius == self.far_radius {
            return;
        }
        self.far_radius = options.far_radius;
        let far = Arc::new(FarTerrain::new(
            self.radius,
            options.far_radius,
            16,
            self.world.block_size(),
            &mut Random::default(),
        ));
        self.far_bounds.clear();
        self.request(Request::Layout(far));
        // The loader runs the queued layout before this load, so the load takes the new one.
        self.far_requested = self.center;
        if let Some(center) = self.center {
            self.request(Request::Far(center));
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
            self.heights.remove(&block);
            self.particles.remove_block(&block);
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
            self.degrades.note_visual(visual, model);
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
                        heights: None,
                        water: None,
                        placements: Vec::new(),
                        interiors: None,
                        ..block.clone()
                    }));
                    update.evicted.push(block.key);
                    return;
                }
                self.particles.add_block(block.key, &block.particles);
                if let Some(heights) = &block.heights {
                    self.heights.insert(block.key, Arc::clone(heights));
                }
                self.blocks.insert(
                    block.key,
                    BlockInfo {
                        placements: block.placements.clone(),
                        terrain: block.terrain.is_some(),
                        water: block.water.is_some(),
                        interiors: block.interiors.is_some(),
                        cull: block.interiors.as_ref().map(|i| Arc::clone(&i.cull)),
                        coronas: block.interiors.as_ref().map(|i| Arc::clone(&i.coronas)),
                        cell_lights: block.interiors.as_ref().map(|i| Arc::clone(&i.cell_lights)),
                    },
                );
            }
            Loaded::Model(visual, model, _) => self.note_model(*visual, model.as_ref()),
            Loaded::Far(far) => self.far_bounds = far.layout.chunk_bounds(&far.vertices),
            Loaded::Layout | Loaded::Rain(..) | Loaded::Images(_) => {}
            Loaded::ShoreMaterial(material) => {
                self.shore_material = Some(*material);
                if material.raw() != 0 {
                    self.request_material(*material);
                }
            }
            Loaded::LightDecalMaterials(materials) => {
                self.light_decal_materials = Some(*materials);
                for material in *materials {
                    if material.raw() != 0 {
                        self.request_material(material);
                    }
                }
            }
            Loaded::Object(object) => {
                if let Some(model) = &object.model {
                    self.degrades.note_object(object.key, model);
                }
                if let Some(rig) = object.model.as_ref().and_then(|model| model.rig()) {
                    self.object_rigs.insert(object.key, (rig, object.scale));
                }
                if let Some(pick) = &object.pick {
                    self.object_picks.insert(object.key, Arc::clone(pick));
                }
            }
            // The systems take their description; their material arrives as a model below.
            Loaded::ObjectParticle(_) => {}
        }
        match loaded {
            Loaded::ObjectParticle(particle) => {
                let particle = *particle;
                let mut images = Some(particle.images);
                for (material, model) in particle.models {
                    self.note_model(material, model.as_ref());
                    update.loaded.push(Loaded::Model(
                        material,
                        model,
                        images.take().unwrap_or_default(),
                    ));
                }
                self.particles.object_loaded(particle.handle, particle.desc);
            }
            // The renderer takes the rain's material like a model.
            Loaded::Rain(rain) => {
                let rain = *rain;
                if rain.model.is_some() {
                    self.rain_material = Some(rain.material);
                }
                let now = self.started.elapsed().as_secs_f64();
                let system = |desc: Option<Arc<ac2_assets::Ac2ParticleSystemDesc>>| {
                    let mut system = ParticleSystem::new(
                        desc?,
                        ac2_math::Frame::IDENTITY,
                        Vec3::ONE,
                        1.0,
                        0,
                        now,
                    );
                    system.set_no_fade(true);
                    Some(system)
                };
                self.rain_splash = system(rain.splash);
                self.rain_ripples = system(rain.ripples);
                self.water_bubbles = system(rain.bubbles);
                // The images ride with the first model, so the others find them uploaded.
                update
                    .loaded
                    .push(Loaded::Model(rain.material, rain.model, rain.images));
                for (material, model) in rain.models {
                    self.note_model(material, model.as_ref());
                    update
                        .loaded
                        .push(Loaded::Model(material, model, Vec::new()));
                }
            }
            loaded => update.loaded.push(loaded),
        }
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

    /// The degrade settings the scene draws with (`SceneOptions::degrade`).
    pub fn degrade_options(&self) -> DegradeOptions {
        self.degrades.options
    }

    /// `CPartArray::Draw` for world object `key` `distance` from the eye
    /// (`CPhysicsObj::UpdateViewerDistance`; a held object takes its holder's): `None` when it is
    /// culled, else its degrade and fade. `adaptive` is `CPhysicsObj::Draw`'s
    /// `_bUseAdaptiveDegrade` (not temporary, mobile and not static); `player` is
    /// `HACK_bDrawingPlayer`. A model without a Setup draws in full.
    pub fn object_detail(
        &self,
        key: u64,
        distance: f32,
        adaptive: bool,
        player: bool,
    ) -> Option<PlacedDetail> {
        self.degrades.object(key, distance, adaptive, player)
    }

    /// The object model `key`'s pick mesh, once loaded.
    pub fn object_pick(&self, key: u64) -> Option<&Arc<crate::selection::PickMesh>> {
        self.object_picks.get(&key)
    }

    /// Pins the particles' client time (`Timer::local_time`) to `now`, the frame's time, so that
    /// the systems run on the client's clock rather than the wall clock.
    pub fn set_clock(&mut self, now: f64) {
        self.particles.set_clock(now);
    }

    /// `CPartArray::CreateParticleSystem` for an object FX: a system `handle` run by the object
    /// `emitter`, attached at `anchor` (posed with the object's bones each frame), its description `desc` loading on the loader
    /// thread. It starts at the scene's clock now and is drawn where the object is placed;
    /// `follow` is false for a fire-and-forget system, which stays where its object first stood.
    pub fn start_object_particle(
        &mut self,
        handle: u64,
        emitter: u64,
        desc: DataId,
        anchor: ParticleAnchor,
        follow: bool,
    ) {
        if self
            .particles
            .request_object(handle, emitter, anchor, follow)
        {
            self.request(Request::ObjectParticle(handle, desc));
        }
    }

    /// `CPartArray::DestroyParticleSystem` for an object FX's system (`handle`): it stops
    /// emitting and runs out its particles at its last place.
    pub fn stop_object_particle(&mut self, handle: u64) {
        self.particles.stop_object(handle);
    }

    /// `FXManager::DoStreakFX`: a streak on `emitter` (`CPartArray::CreateStreakEffect`). Its
    /// material loads off the frame thread; it thinks and draws from the next frame.
    pub fn start_object_streak(&mut self, emitter: u64, spec: crate::streak::StreakSpec) {
        self.object_streaks
            .start(emitter, spec, self.particles.now());
        self.request_material(spec.material);
    }

    /// `PHYSICSOBJREP::StartStreakInternal` (`on`) and `StopStreakInternal`: the emitter's streaks
    /// activate or deactivate.
    pub fn set_object_streaks_active(&mut self, emitter: u64, on: bool) {
        self.object_streaks.set_active(emitter, on);
    }

    /// `CPartArray`'s destructor: the emitter left the world, so its streaks are destroyed.
    pub fn remove_object_streaks(&mut self, emitter: u64) {
        self.object_streaks.remove_emitter(emitter);
    }

    /// How many streaks the emitter holds.
    pub fn object_streak_count(&self, emitter: u64) -> usize {
        self.object_streaks.count(emitter)
    }

    /// `Render::m_Prefs.StreakEffects`: while off the streaks are not drawn (they keep thinking).
    pub fn set_streaks_visible(&mut self, visible: bool) {
        self.streaks_visible = visible;
    }

    /// A streak's or decal's material, loaded once like a particle material (`ParticleEmitter`'s
    /// material).
    fn request_material(&mut self, material: DataId) {
        if self.known_models.contains(&material) || self.pending_models.contains(&material) {
            return;
        }
        self.pending_models.insert(material);
        self.request(Request::StreakMaterial(material));
    }

    /// `CPartArray::CreateLightSource` for an object FX: a point light on `emitter` at the node's
    /// offset, created at the scene's clock now.
    pub fn start_object_light(&mut self, emitter: u64, light: Ac2FxLight) {
        self.object_lights
            .start(emitter, light, self.particles.now());
    }

    /// `CPartArray::DestroyAllTransientFX`: the object left the world, so its FX lights go. Its
    /// particle systems are stopped by their handles.
    pub fn remove_object_lights(&mut self, emitter: u64) {
        self.object_lights.remove_emitter(emitter);
    }

    /// The frame's object placements (see [`ObjectPlacement`]): the object systems are drawn
    /// where their object is placed, and the object lights take the object's frame.
    pub fn place_objects(&mut self, placements: &[ObjectPlacement]) {
        self.bodies = placements.to_vec();
        self.particles.place_objects(placements);
        self.object_lights.place(placements, self.particles.now());
        self.object_streaks.place(placements.iter().map(|p| {
            let scale = p.scale.x.max(p.scale.y).max(p.scale.z);
            (p.emitter, p.cell, p.frame, scale)
        }));
    }

    /// The point lights of the FX lights that reach an object at `position` with bounding radius
    /// `radius` at the scene's clock (the heaviest in range, at most the device's point lights).
    pub fn object_point_lights(&self, position: Vec3, radius: f32) -> Vec<PointLight> {
        self.object_lights
            .point_lights(self.particles.now(), position, radius)
    }

    /// How many FX lights the scene holds.
    pub fn object_light_count(&self) -> usize {
        self.object_lights.count()
    }

    /// How many object particle systems the scene holds (loading, running or running out).
    pub fn object_particle_systems(&self) -> usize {
        self.particles.object_systems()
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
        let lighting = self.lighting_at(eye);
        let mut keys: Vec<BlockKey> = self.blocks.keys().copied().collect();
        keys.sort_unstable();
        let all = || {
            keys.iter()
                .flat_map(|key| self.blocks[key].placements.iter())
                .filter(|p| self.models.contains_key(&p.visual))
        };
        // Each frill with the block it stands in, for that block's terrain under it.
        let frills: Vec<(BlockKey, &Placement)> = keys
            .iter()
            .flat_map(|key| {
                self.blocks[key]
                    .placements
                    .iter()
                    .filter(|p| self.models.contains_key(&p.visual))
                    .map(move |p| (*key, p))
            })
            .filter(|(_, p)| p.frill)
            .collect();
        let candidates: Vec<FrillCandidate> = frills
            .iter()
            .map(|(_, p)| {
                let (indices, vertices) = self.models[&p.visual];
                FrillCandidate {
                    world: p.world,
                    indices,
                    vertices,
                }
            })
            .collect();
        let chosen = select_frills(&candidates, eye, &self.frill);
        // The client's grass past retail's ring (`AGENTS.md`), thinning and growing out with
        // distance; every frill carries the ring so the near ones stay whole.
        let far_frills = select_far_frills(&candidates, eye, &self.frill, &chosen);
        // The self shadow's light bit this frame (`LScapeRender::UpdateLighting`).
        let shadow_bit = shadow_light_bit(lighting.light_dir);
        self.degrades.tick();
        let degrades = &self.degrades;
        // Scenery is client-made (temporary ids): `CPartArray::Draw` without adaptive degrade.
        let placements: Vec<FramePlacement> = all()
            .filter(|p| !p.frill)
            .filter_map(|p| {
                let placed = degrades.placed(p.visual, p.world.w_axis.truncate(), eye)?;
                Some(FramePlacement {
                    visual: p.visual,
                    world: p.world,
                    frill_normal: None,
                    frill_shade: 1.0,
                    detail: placed.detail,
                    alpha: placed.alpha,
                    grass: [0.0; 3],
                })
            })
            .chain(self.frill_placements(&frills, &chosen, &far_frills, shadow_bit))
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
        let flare_images: Vec<TextureKey> = sky
            .flares
            .iter()
            .flat_map(|flare| flare.flare.points.iter())
            .filter_map(|point| point.texture.map(TextureKey::Texture))
            .filter(|key| self.requested_images.insert(*key))
            .collect();
        if !flare_images.is_empty() {
            self.request(Request::Images(flare_images));
        }
        let terrain: Vec<BlockKey> = keys
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
        let interior_cells = self.visible_interior_cells(camera, &frustum, &interiors);
        let mut particles = self.particles.frame(camera);
        self.skeleton_effects(lighting.rain, lighting.rain_wind);
        particles.extend(self.rain_frame(camera, &lighting));
        self.object_streaks.think(self.particles.now());
        if self.streaks_visible {
            particles.extend(
                self.object_streaks
                    .geometry()
                    .into_iter()
                    .map(|(material, geometry)| ParticleBatch { material, geometry }),
            );
        }
        self.pool_lights(&interiors, &interior_cells);
        let shore = self.shore_batch(&terrain);
        particles.extend(self.decal_batches());
        let coronas = self.frame_coronas(&interiors, &interior_cells);
        let reflection = self.reflection_plane(eye);
        SceneFrame {
            lighting,
            sky,
            placements,
            terrain,
            water,
            interiors,
            interior_cells,
            far_chunks,
            particles,
            coronas,
            reflection,
            shore,
        }
    }

    /// Which env cells of `blocks` draw (`WorldRender::Render`'s gather): from the camera's own
    /// cell when it stands in one (`CEnvCell::CullCells`), whose outside portals open the
    /// viewports the other blocks' cells are looked at through; from the outside, each outside-
    /// stab cell that touches the outside and passes `CEnvCell::CheckVisibility`.
    fn visible_interior_cells(
        &self,
        camera: &Camera,
        frustum: &Frustum,
        blocks: &[BlockKey],
    ) -> HashMap<BlockKey, Vec<u32>> {
        let mut visible: HashMap<BlockKey, Vec<u32>> = HashMap::new();
        let mut viewports = vec![*frustum];
        let mut viewer_inside = false;
        for key in blocks {
            let Some(cull) = self.blocks.get(key).and_then(|b| b.cull.as_deref()) else {
                continue;
            };
            let Some(start) = cull.cell_at(camera.eye) else {
                continue;
            };
            viewer_inside = true;
            let mut walk = InteriorCull::new(&cull.cells, &cull.groups, camera.eye);
            walk.cull_cells(start, frustum, cull.culling_plane);
            let result = walk.finish();
            viewports = result.outside_viewports;
            visible.entry(*key).or_default().extend(result.visible);
            break;
        }
        for key in blocks {
            let Some(cull) = self.blocks.get(key).and_then(|b| b.cull.as_deref()) else {
                continue;
            };
            let seen = visible.entry(*key).or_default();
            for view in &viewports {
                for id in &cull.outside_stab {
                    let Some(cell) = cull.cells.get(id) else {
                        continue;
                    };
                    if seen.contains(id) || !cell.touches_outside() {
                        continue;
                    }
                    if !check_visibility(cell, viewer_inside, view) {
                        continue;
                    }
                    let mut walk = InteriorCull::new(&cull.cells, &cull.groups, camera.eye);
                    walk.cull_cells(*id, view, cull.culling_plane);
                    for cell in walk.finish().visible {
                        if !seen.contains(&cell) {
                            seen.push(cell);
                        }
                    }
                }
            }
        }
        visible
    }

    /// `WorldRender::Update`'s `UpdateRain` and `WorldRender::Render`'s `DrawRain`: the frame's
    /// streaks as a batch of the rain material, when it rains hard enough to draw.
    fn rain_frame(&mut self, camera: &Camera, lighting: &SceneLighting) -> Vec<ParticleBatch> {
        let mut batches = Vec::new();
        let Some(basis) = CameraBasis::look_at(camera.eye, camera.target) else {
            return batches;
        };
        batches.extend(self.rain_streaks(&basis, lighting));
        // `FillRainVertices`' discharges: a splash where each streak lands (the systems' second
        // emitter), a ripple on near water; then the systems think and draw like the others.
        let now = self.started.elapsed().as_secs_f64();
        for splash in self.rain.splashes().to_vec() {
            if let Some(system) = self.rain_splash.as_mut() {
                system.discharge(1, now, splash.at);
            }
            if let (true, Some(system)) = (splash.ripple, self.rain_ripples.as_mut()) {
                system.discharge(1, now, splash.at);
            }
        }
        for system in self
            .rain_splash
            .iter_mut()
            .chain(self.rain_ripples.iter_mut())
            .chain(self.water_bubbles.iter_mut())
        {
            system.think(now);
            let (Some(material), true) = (system.material(), system.alive() > 0) else {
                continue;
            };
            let mut geometry = crate::particles::ParticleGeometry::default();
            system.fill(&basis, now, 1.0, &mut geometry);
            if !geometry.is_empty() {
                batches.push(ParticleBatch { material, geometry });
            }
        }
        batches
    }

    /// The rain's streaks as a batch of its material, when it rains hard enough to draw.
    fn rain_streaks(
        &mut self,
        basis: &CameraBasis,
        lighting: &SceneLighting,
    ) -> Option<ParticleBatch> {
        let block_size = self.world.block_size();
        let heights = &self.heights;
        let ground = |x: f32, y: f32| {
            let key = (
                u8::try_from((x / block_size).floor() as i32).ok()?,
                u8::try_from((y / block_size).floor() as i32).ok()?,
            );
            let field = heights.get(&key)?;
            Some((field.height_at(x, y)?, field.water))
        };
        let now = self.started.elapsed().as_secs_f64();
        let geometry = self.rain.update(
            now,
            basis,
            lighting.rain,
            lighting.rain_wind,
            block_size,
            &ground,
        );
        let material = self.rain_material?;
        (lighting.rain > RAIN_DRAW_THRESHOLD && !geometry.is_empty())
            .then_some(ParticleBatch { material, geometry })
    }

    /// The particles alive in the systems the last frame drew.
    pub fn particles_alive(&self) -> usize {
        self.particles.alive()
    }
}
