//! The drawn world around a viewer, backend-neutral: which landblocks are resident
//! (`LScapeTerrainMesh::UpdateNearTerrain` over the near ring), the far mesh past them, and per
//! frame what to draw and in what order (`WorldRender::Draw`: the sky before the world, the
//! landscape, the objects and frills, the visible interiors, the sky after). All loading runs on
//! the scene's loader thread; the frame thread only drains finished loads and lays out frames.

mod interiors;
mod loader;
mod object_lights;
mod particles;
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

pub use interiors::{load_block_interiors, BlockInteriors};
pub use loader::{
    BlockKey, CloudShadow, Images, Loaded, LoadedBlock, LoadedFar, LoadedObject,
    LoadedObjectParticle, ParticleSource, Placement,
};
pub use particles::ParticleBatch;

use crate::frill::{frill_normal, select_frills, FrillCandidate, FrillSettings};
use crate::frustum::{Frustum, Relation};
use crate::object::{PointLight, RenderSettings};
use crate::particles::{CameraBasis, ParticleSystem};
use crate::prefs::SceneOptions;
use crate::rain::{RainField, RAIN_DRAW_THRESHOLD};
use crate::sky::{sky_frame, SkyFrame};
use crate::terrain::{shadow_light_bit, FarTerrain, TerrainHeights, FIRST_FAR_CHUNK};
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
}

/// Where an object stands this frame, for the particle systems it runs: its emitter (the object's
/// id), its frame and its total scale (`CPartArray::QueueParticleSystems`'s object matrix).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectPlacement {
    pub emitter: u64,
    pub frame: Frame,
    pub scale: Vec3,
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
    /// The entities' particles, merged per material (`PSManager::RenderParticleSystems`).
    pub particles: Vec<ParticleBatch>,
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
    /// The far ring's radius in blocks (`LandscapeDrawDistance`) and the frill settings the frame
    /// selects with (`LandscapeFrillDistance`).
    far_radius: i32,
    frill: FrillSettings,
    /// World objects' models requested by key, and the rig and scale of those that loaded.
    requested_objects: HashSet<u64>,
    object_rigs: HashMap<u64, (crate::object::PoseRig, Vec3)>,
    object_picks: HashMap<u64, Arc<crate::selection::PickMesh>>,
    particles: SceneParticles,
    /// The point lights of object FX (`CPartArray::CreateLightSource`).
    object_lights: ObjectLights,
    /// The resident blocks' ground, for the rain.
    heights: HashMap<BlockKey, Arc<TerrainHeights>>,
    rain: RainField,
    /// The rain's material once loaded (`LScapeRender::m_pRainMaterial`).
    rain_material: Option<DataId>,
    /// `WorldRender`'s rain splash and water ripple systems (no fade, at the world's origin).
    rain_splash: Option<ParticleSystem>,
    rain_ripples: Option<ParticleSystem>,
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
            outstanding: 0,
            evicted: Vec::new(),
            far_bounds: Vec::new(),
            far_radius: defaults.far_radius,
            frill: defaults.frill,
            requested_objects: HashSet::new(),
            object_rigs: HashMap::new(),
            object_picks: HashMap::new(),
            particles: SceneParticles::new(),
            object_lights: ObjectLights::default(),
            heights: HashMap::new(),
            rain: RainField::default(),
            rain_material: None,
            rain_splash: None,
            rain_ripples: None,
            started: Instant::now(),
        };
        scene.request(Request::Rain);
        scene
    }

    /// `WorldScene`'s graphics options (`SmartBox::InitPreferences` and the preference monitors):
    /// the frill distance applies to the next frame. A changed landscape draw distance lays the far
    /// ring out again: the new layout is queued to the loader, the far chunks it loads replace the
    /// old ones, and until then no far chunk is drawn. The near ring and the objects stay.
    pub fn set_options(&mut self, options: SceneOptions) {
        self.frill = options.frill;
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
                    },
                );
            }
            Loaded::Model(visual, model, _) => self.note_model(*visual, model.as_ref()),
            Loaded::Far(far) => self.far_bounds = far.layout.chunk_bounds(&far.vertices),
            Loaded::Layout | Loaded::Rain(..) => {}
            Loaded::Object(object) => {
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
    /// `emitter` at `placement` (object space), its description `desc` loading on the loader
    /// thread. It starts at the scene's clock now and is drawn where the object is placed;
    /// `follow` is false for a fire-and-forget system, which stays where its object first stood.
    pub fn start_object_particle(
        &mut self,
        handle: u64,
        emitter: u64,
        desc: DataId,
        placement: Ac2Transform,
        follow: bool,
    ) {
        if self
            .particles
            .request_object(handle, emitter, placement, follow)
        {
            self.request(Request::ObjectParticle(handle, desc));
        }
    }

    /// `CPartArray::DestroyParticleSystem` for an object FX's system (`handle`): it stops
    /// emitting and runs out its particles at its last place.
    pub fn stop_object_particle(&mut self, handle: u64) {
        self.particles.stop_object(handle);
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
        self.particles.place_objects(placements);
        self.object_lights.place(placements, self.particles.now());
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
        // The self shadow's light bit this frame (`LScapeRender::UpdateLighting`).
        let shadow_bit = shadow_light_bit(lighting.light_dir);
        let placements: Vec<FramePlacement> = all()
            .filter(|p| !p.frill)
            .map(|p| FramePlacement {
                visual: p.visual,
                world: p.world,
                frill_normal: None,
                frill_shade: 1.0,
            })
            .chain(chosen.iter().map(|&index| {
                let (key, p) = frills[index];
                let frill_shade = self.heights.get(&key).map_or(1.0, |heights| {
                    heights.self_shadow(p.world.w_axis.x, p.world.w_axis.y, shadow_bit)
                });
                FramePlacement {
                    visual: p.visual,
                    world: p.world,
                    frill_normal: Some(frill_normal(&p.world)),
                    frill_shade,
                }
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
        let mut particles = self.particles.frame(camera);
        particles.extend(self.rain_frame(camera, &lighting));
        SceneFrame {
            lighting,
            sky,
            placements,
            terrain,
            water,
            interiors,
            far_chunks,
            particles,
        }
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
