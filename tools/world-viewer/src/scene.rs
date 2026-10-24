//! What the viewer draws: the near landscape and the placed objects of the blocks around the
//! camera, lit by the region sky at the viewer's time of day.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_gpu::{
    DrawGroup, Gpu, GpuObjectModel, ObjectInstance, ObjectRenderer, ScreenQuad, TerrainBlock,
    TerrainRenderer, TextureKey,
};
use ac2_math::{Mat4, Vec3};
use ac2_render::frill::{frill_normal, select_frills, FrillCandidate, FrillSettings};
use ac2_render::object::{load_object_model, load_water_model, ObjectEnvironment, RenderSettings};
use ac2_render::sky::{sky_frame, SkyFrame, SkyPlacement};

use crate::interiors::{load_block_interiors, BlockInteriors};
use ac2_render::terrain::build_terrain_mesh;
use ac2_render::{load_material_image, load_texture_image, Camera, SceneLighting};
use ac2_world::Ac2WorldSource;

/// `GameTime` day length in seconds for the Legions region.
pub const DAY_SECONDS: f64 = 7347.0;
/// `ObjectRenderer` lays out 16 groups; three are the sky and the world.
const MAX_INTERIOR_GROUPS: usize = 13;

/// One resident block: its near terrain and its placed objects (visual, world matrix, frill).
struct Block {
    terrain: Option<TerrainBlock>,
    placements: Vec<(DataId, Mat4, bool)>,
    /// Near water (`LScapeTerrainMesh::UpdateNearWater`), already in world space.
    water: Option<GpuObjectModel>,
    /// Interior cells visible from outside.
    interiors: Option<BlockInteriors>,
}

pub struct Scene {
    pub world: Arc<Ac2WorldSource>,
    pub renderer: TerrainRenderer,
    objects: ObjectRenderer,
    fog_quad: ScreenQuad,
    sky: SkyFrame,
    models: HashMap<DataId, Option<GpuObjectModel>>,
    blocks: HashMap<(u8, u8), Block>,
    /// Blocks whose interiors `prepare` laid out as groups 3 onward.
    frame_interiors: Vec<(u8, u8)>,
    /// The placements `prepare` laid out, in draw order, with a frill's merged normal.
    frame_placements: Vec<(DataId, Mat4, Option<Vec3>)>,
    pub radius: i32,
    pub time_of_day: f64,
}

impl Scene {
    pub fn new(
        world: Arc<Ac2WorldSource>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        radius: i32,
    ) -> Self {
        Self {
            world,
            renderer: TerrainRenderer::new(gpu, format),
            objects: ObjectRenderer::new(gpu, format),
            fog_quad: ScreenQuad::new(gpu, format),
            sky: SkyFrame::default(),
            models: HashMap::new(),
            blocks: HashMap::new(),
            frame_placements: Vec::new(),
            frame_interiors: Vec::new(),
            radius,
            time_of_day: 0.0,
        }
    }

    /// The landblock under a world AC position.
    pub fn block_at(&self, position: Vec3) -> (u8, u8) {
        let size = self.world.block_size();
        let clamp = |v: f32| (v / size).floor().clamp(0.0, 254.0) as u8;
        (clamp(position.x), clamp(position.y))
    }

    /// Highest terrain vertex of a block, for placing the camera above it.
    pub fn block_top(&self, block: (u8, u8)) -> f32 {
        self.world
            .load_landblock(block_id(block))
            .ok()
            .flatten()
            .map_or(0.0, |landblock| {
                landblock.height_grid().into_iter().fold(0.0, f32::max)
            })
    }

    /// Loads the blocks within `radius` of `center` and drops the rest
    /// (`LScapeTerrainMesh::UpdateNearTerrain` over the near ring).
    pub fn update_residency(&mut self, gpu: &Gpu, center: (u8, u8)) {
        let radius = self.radius;
        let near = |block: &(u8, u8)| {
            (i32::from(block.0) - i32::from(center.0)).abs() <= radius
                && (i32::from(block.1) - i32::from(center.1)).abs() <= radius
        };
        self.blocks.retain(|block, _| near(block));
        let store = self.world.resource_store().clone();
        let settings = RenderSettings::default();
        let mut load = |key: TextureKey| match key {
            TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
            TextureKey::Mask(did) | TextureKey::Texture(did) => {
                load_texture_image(&store, did).ok().flatten()
            }
        };
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let (x, y) = (i32::from(center.0) + dx, i32::from(center.1) + dy);
                let (Ok(x), Ok(y)) = (u8::try_from(x), u8::try_from(y)) else {
                    continue;
                };
                if self.blocks.contains_key(&(x, y)) {
                    continue;
                }
                let id = block_id((x, y));
                let landblock = self.world.load_landblock(id).ok().flatten();
                let terrain = landblock.as_ref().map(|landblock| {
                    let mesh = build_terrain_mesh(id, landblock, self.world.block_size());
                    self.renderer
                        .upload_block(gpu, &mesh, self.world.block_size(), &mut load)
                });
                let water = landblock
                    .as_ref()
                    .and_then(|landblock| self.world.block_water(id, landblock))
                    .and_then(|water| {
                        load_water_model(&store, id, &water, self.world.block_size(), &settings)
                            .ok()
                            .flatten()
                    })
                    .map(|model| self.objects.upload(gpu, &model, &mut load));
                let mut placements = Vec::new();
                for marker in self.world.entity_markers(id).iter().flat_map(|m| m.iter()) {
                    let visual = marker.visual();
                    if !self.models.contains_key(&visual) {
                        let model = load_object_model(&store, visual, &settings)
                            .ok()
                            .flatten()
                            .map(|model| self.objects.upload(gpu, &model, &mut load));
                        self.models.insert(visual, model);
                    }
                    if self.models[&visual].is_some() {
                        placements.push((
                            visual,
                            marker.world_frame().to_mat4() * Mat4::from_scale(marker.scale()),
                            marker.frill(),
                        ));
                    }
                }
                let interiors = load_block_interiors(
                    &self.world,
                    &store,
                    &mut self.objects,
                    gpu,
                    id,
                    &settings,
                    &mut load,
                );
                self.blocks.insert(
                    (x, y),
                    Block {
                        terrain,
                        placements,
                        water,
                        interiors,
                    },
                );
            }
        }
    }

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

    /// Writes the frame's camera and lighting and lays out the object draws; `time` is client
    /// time in seconds for animated materials.
    /// Writes the frame's camera and lighting and lays out the draws: the sky before the world
    /// (`LScapeSky::Draw(0)`), the world, then the sky after it (`Draw(1)`). `time` is client time
    /// in seconds for animated materials.
    pub fn prepare(&mut self, gpu: &Gpu, camera: &Camera, aspect: f32, time: f64) {
        let lighting = self.lighting();
        self.renderer
            .set_frame(&gpu.queue, camera, &lighting, aspect);
        // Scenery and entities, then the frills `FrillRender::RefreshFrillMesh` keeps.
        let all = || {
            self.blocks
                .values()
                .flat_map(|block| block.placements.iter())
        };
        let frills: Vec<_> = all().filter(|(_, _, frill)| *frill).collect();
        let candidates: Vec<_> = frills
            .iter()
            .map(|(visual, world, _)| {
                let (indices, vertices) = self.models[visual]
                    .as_ref()
                    .map_or((0, 0), GpuObjectModel::first_list);
                FrillCandidate {
                    world: *world,
                    indices,
                    vertices,
                }
            })
            .collect();
        let chosen = select_frills(&candidates, camera.eye, &FrillSettings::default());
        self.frame_placements = all()
            .filter(|(_, _, frill)| !*frill)
            .map(|(visual, world, _)| (*visual, *world, None))
            .chain(chosen.iter().map(|&index| {
                let (visual, world, _) = frills[index];
                (*visual, *world, Some(frill_normal(world)))
            }))
            .collect();
        let sky_objects = self
            .world
            .region()
            .and_then(|region| region.sky_objects(self.time_of_day))
            .unwrap_or_default();
        self.sky = sky_frame(
            &sky_objects,
            camera.eye,
            false,
            lighting.fog_color,
            lighting.fog_sky,
        );
        let store = self.world.resource_store().clone();
        let settings = RenderSettings::default();
        let mut load = |key: TextureKey| match key {
            TextureKey::Material(did) => load_material_image(&store, did).ok().flatten(),
            TextureKey::Mask(did) | TextureKey::Texture(did) => {
                load_texture_image(&store, did).ok().flatten()
            }
        };
        for placement in self.sky.before.iter().chain(&self.sky.after) {
            if !self.models.contains_key(&placement.gfx) {
                let model = load_object_model(&store, placement.gfx, &settings)
                    .ok()
                    .flatten()
                    .map(|model| self.objects.upload(gpu, &model, &mut load));
                self.models.insert(placement.gfx, model);
            }
        }
        if let Some(color) = self.sky.fog_quad {
            self.fog_quad.set_color(&gpu.queue, color);
        }
        let world = instances(&self.models, &self.frame_placements, &self.blocks);
        let before = sky_instances(&self.models, &self.sky.before);
        let after = sky_instances(&self.models, &self.sky.after);
        // Groups 0-2: sky before, world, sky after; then each block's visible interiors under its
        // inside environment.
        let mut keys: Vec<_> = self
            .blocks
            .iter()
            .filter(|(_, block)| block.interiors.is_some())
            .map(|(key, _)| *key)
            .collect();
        keys.sort_unstable();
        keys.truncate(MAX_INTERIOR_GROUPS);
        let interior_instances: Vec<_> = keys
            .iter()
            .filter_map(|key| self.blocks[key].interiors.as_ref())
            .map(|interiors| (interiors.instances(), &interiors.environment))
            .collect();
        let mut groups = vec![
            DrawGroup {
                instances: &before,
                ordered: true,
                environment: None,
            },
            DrawGroup {
                instances: &world,
                ordered: false,
                environment: None,
            },
            DrawGroup {
                instances: &after,
                ordered: true,
                environment: None,
            },
        ];
        groups.extend(
            interior_instances
                .iter()
                .map(|(instances, environment)| DrawGroup {
                    instances,
                    ordered: false,
                    environment: Some(environment),
                }),
        );
        self.objects.prepare(
            gpu,
            camera,
            &ObjectEnvironment::outside(&lighting),
            aspect,
            time,
            &groups,
        );
        self.frame_interiors = keys;
    }

    /// Records the frame into a pass cleared to `SceneTool::backgroundColor` (black).
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.objects
            .draw(pass, 0, &sky_instances(&self.models, &self.sky.before));
        if self.sky.fog_quad.is_some() {
            self.fog_quad.draw(pass);
        }
        self.renderer.draw(
            pass,
            self.blocks.values().filter_map(|b| b.terrain.as_ref()),
        );
        let instances = instances(&self.models, &self.frame_placements, &self.blocks);
        self.objects.draw(pass, 1, &instances);
        for (index, key) in self.frame_interiors.iter().enumerate() {
            if let Some(interiors) = self.blocks.get(key).and_then(|b| b.interiors.as_ref()) {
                self.objects.draw(pass, 3 + index, &interiors.instances());
            }
        }
        self.objects
            .draw(pass, 2, &sky_instances(&self.models, &self.sky.after));
    }
}

/// The frame's object instances followed by the resident blocks' water. Both `prepare` and
/// `draw` build it from unchanged state, so the order matches.
fn sky_instances<'a>(
    models: &'a HashMap<DataId, Option<GpuObjectModel>>,
    placements: &[SkyPlacement],
) -> Vec<ObjectInstance<'a>> {
    placements
        .iter()
        .filter_map(|placement| {
            Some(ObjectInstance {
                tint: placement.tint,
                ..ObjectInstance::new(models.get(&placement.gfx)?.as_ref()?, placement.world)
            })
        })
        .collect()
}

fn instances<'a>(
    models: &'a HashMap<DataId, Option<GpuObjectModel>>,
    placements: &[(DataId, Mat4, Option<Vec3>)],
    blocks: &'a HashMap<(u8, u8), Block>,
) -> Vec<ObjectInstance<'a>> {
    placements
        .iter()
        .filter_map(|(visual, world, frill_normal)| {
            Some(ObjectInstance {
                frill_normal: *frill_normal,
                ..ObjectInstance::new(models.get(visual)?.as_ref()?, *world)
            })
        })
        .chain(
            blocks.values().filter_map(|block| {
                Some(ObjectInstance::new(block.water.as_ref()?, Mat4::IDENTITY))
            }),
        )
        .collect()
}

pub fn block_id((x, y): (u8, u8)) -> CellId {
    CellId::new((u32::from(x) << 24) | (u32::from(y) << 16) | 1)
}
