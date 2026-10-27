//! The GPU side of `ac2_render::scene`: uploads what the scene's loader finished and draws a
//! `SceneFrame` in `WorldRender::Draw` order (the sky before the world and its fog quad, the near
//! and far landscape, the objects with near and far water, each block's visible interiors under
//! its inside environment, then the sky after the world). Images arrive decoded with the loads,
//! so nothing here reads the DATs.

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_math::Mat4;
use ac2_render::object::{ObjectEnvironment, ObjectPose, PointLight};
use ac2_render::scene::{BlockKey, Images, Loaded, SceneFrame, SceneUpdate};
use ac2_render::sky::SkyPlacement;
use ac2_render::{Camera, Image, TextureKey};

use crate::sky::{SkyLook, SkyRenderer};
use crate::{
    DrawGroup, Gpu, GpuFarTerrain, GpuObjectModel, ObjectInstance, ObjectRenderer, ScreenQuad,
    StaticShadowMap, TerrainBlock, TerrainRenderer, HDR_FORMAT,
};

/// One resident block on the GPU.
struct GpuBlock {
    terrain: Option<TerrainBlock>,
    water: Option<GpuObjectModel>,
    interiors: Option<GpuInteriors>,
}

/// A block's visible interior cells and the static objects standing in them.
struct GpuInteriors {
    environment: ObjectEnvironment,
    cells: Vec<(GpuObjectModel, Vec<PointLight>)>,
    objects: Vec<(DataId, Mat4, Vec<PointLight>)>,
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
}

/// The scene's GPU resources.
pub struct SceneRenderer {
    pub terrain: TerrainRenderer,
    objects: ObjectRenderer,
    fog_quad: ScreenQuad,
    /// The sky under the DAT sky objects.
    sky: SkyRenderer,
    /// The near landscape's static shadow map.
    shadow_map: StaticShadowMap,
    models: HashMap<DataId, GpuObjectModel>,
    /// World objects' models by request key.
    world_objects: HashMap<u64, GpuObjectModel>,
    frame_objects: Vec<SceneObject>,
    blocks: HashMap<BlockKey, GpuBlock>,
    far: Option<(GpuFarTerrain, Option<GpuObjectModel>)>,
    /// Every image the loader decoded, by key (it sends each once).
    images: HashMap<TextureKey, Image>,
    frame: SceneFrame,
}

impl SceneRenderer {
    /// Leave out objects whose pipelines are still compiling instead of waiting for them
    /// (`ObjectRenderer::set_stream_pipelines`).
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.objects.set_stream_pipelines(stream);
    }

    /// The renderers of a scene drawn into an [`crate::HdrTarget`] (`HDR_FORMAT`).
    pub fn new(gpu: &Gpu) -> Self {
        let mut terrain = TerrainRenderer::new(gpu, HDR_FORMAT);
        let shadow_map = StaticShadowMap::new(gpu);
        terrain.set_shadows(gpu, None, shadow_map.view(), &mut |_| None);
        let mut objects = ObjectRenderer::new(gpu, HDR_FORMAT);
        objects.set_shadow_map(gpu, shadow_map.view());
        Self {
            terrain,
            shadow_map,
            objects,
            fog_quad: ScreenQuad::new(gpu, HDR_FORMAT),
            sky: SkyRenderer::new(gpu, HDR_FORMAT),
            models: HashMap::new(),
            world_objects: HashMap::new(),
            frame_objects: Vec::new(),
            blocks: HashMap::new(),
            far: None,
            images: HashMap::new(),
            frame: SceneFrame::default(),
        }
    }

    fn take_images(&mut self, images: Images) {
        for (key, image) in images {
            if let Some(image) = image {
                self.images.insert(key, image);
            }
        }
    }

    /// Uploads finished loads and drops evicted blocks.
    pub fn apply(&mut self, gpu: &Gpu, update: SceneUpdate) {
        for loaded in update.loaded {
            match loaded {
                Loaded::Block(block) => {
                    self.take_images(block.images);
                    let images = &self.images;
                    let mut load = |key: TextureKey| images.get(&key).cloned();
                    for (visual, model) in block.models {
                        if let Some(model) = model {
                            let gpu_model = self.objects.upload(gpu, &model, &mut load);
                            self.models.insert(visual, gpu_model);
                        }
                    }
                    let terrain = block.terrain.map(|mesh| {
                        self.terrain
                            .upload_block(gpu, &mesh, block.block_size, &mut load)
                    });
                    let water = block
                        .water
                        .map(|model| self.objects.upload(gpu, &model, &mut load));
                    let interiors = block.interiors.map(|interiors| {
                        let cells = interiors
                            .cells
                            .into_iter()
                            .map(|(model, lights)| {
                                (self.objects.upload(gpu, &model, &mut load), lights)
                            })
                            .collect();
                        GpuInteriors {
                            environment: interiors.environment,
                            cells,
                            objects: interiors.objects,
                        }
                    });
                    self.blocks.insert(
                        block.key,
                        GpuBlock {
                            terrain,
                            water,
                            interiors,
                        },
                    );
                }
                // `WorldScene` hands the rain's loads on as models; a far layout change has no data.
                // The scene keeps an object FX's system; its material arrives as a model.
                Loaded::Layout | Loaded::Rain(_) | Loaded::ObjectParticle(_) => {}
                Loaded::Model(visual, model, images) => {
                    self.take_images(images);
                    let images = &self.images;
                    let mut load = |key: TextureKey| images.get(&key).cloned();
                    if let Some(model) = model {
                        let gpu_model = self.objects.upload(gpu, &model, &mut load);
                        self.models.insert(visual, gpu_model);
                    }
                }
                Loaded::Far(far) => {
                    self.take_images(far.images);
                    let images = &self.images;
                    let mut load = |key: TextureKey| images.get(&key).cloned();
                    let terrain = self.terrain.upload_far(
                        gpu,
                        &far.layout,
                        &far.vertices,
                        far.texture,
                        &mut load,
                    );
                    self.terrain.set_shadows(
                        gpu,
                        Some(far.clouds),
                        self.shadow_map.view(),
                        &mut load,
                    );
                    let water = far
                        .water
                        .map(|model| self.objects.upload(gpu, &model, &mut load));
                    self.far = Some((terrain, water));
                }
                Loaded::Object(object) => {
                    self.take_images(object.images);
                    let images = &self.images;
                    let mut load = |key: TextureKey| images.get(&key).cloned();
                    if let Some(model) = object.model {
                        let gpu_model = self.objects.upload(gpu, &model, &mut load);
                        self.world_objects.insert(object.key, gpu_model);
                    }
                }
            }
        }
        for key in update.evicted {
            self.blocks.remove(&key);
        }
    }

    /// Writes the frame's camera and lighting and lays out its draws; `time` is client time in
    /// seconds for animated materials.
    pub fn prepare(
        &mut self,
        gpu: &Gpu,
        camera: &Camera,
        aspect: f32,
        time: f64,
        frame: SceneFrame,
        objects: Vec<SceneObject>,
    ) {
        self.frame = frame;
        self.frame_objects = objects;
        // The particles of the frame go into their materials' models.
        for batch in &self.frame.particles {
            if let Some(model) = self.models.get_mut(&batch.material) {
                self.objects.set_geometry(
                    gpu,
                    model,
                    &batch.geometry.vertices,
                    &batch.geometry.indices,
                );
            }
        }
        let lighting = self.frame.lighting;
        self.refresh_shadow_map(gpu);
        if let Some([r, g, b, a]) = self.frame.sky.fog_quad {
            // The authored fog colour, decoded for the linear scene; its alpha is `fog_sky`.
            let linear = [r, g, b].map(|c| crate::shading::srgb_to_linear(c.clamp(0.0, 1.0)));
            self.fog_quad
                .set_color(&gpu.queue, [linear[0], linear[1], linear[2], a]);
        }
        // `WorldRender::SetupEnvironment(SkyEnvironment)`: the sky objects draw with
        // `FOGMODE_NONE`; the sky's only fog is `LScapeSky::Draw`'s `fog_sky` screen quad.
        let sky_environment = ObjectEnvironment::sky(&lighting);
        let sky_group_environment = Some(&sky_environment);
        let (models, blocks, far, frame) = (&self.models, &self.blocks, &self.far, &self.frame);
        let placed = (&self.world_objects, self.frame_objects.as_slice());
        let before = sky_instances(models, &frame.sky.before);
        let look = SkyLook {
            backdrop: crate::sky_backdrop(&before).map(|index| before[index].tint),
            time,
            wind: self
                .terrain
                .clouds()
                .map_or(crate::sky::DEFAULT_CLOUD_WIND, |c| {
                    crate::sky::cloud_wind(&c)
                }),
            ..SkyLook::default()
        };
        self.sky
            .set_frame(&gpu.queue, camera, aspect, &lighting, &look);
        self.terrain
            .set_frame(&gpu.queue, camera, &lighting, aspect, time);
        let world = world_instances(models, placed, blocks, far, frame);
        let after = sky_instances(models, &frame.sky.after);
        let interiors = interior_instances(models, blocks, frame);
        let mut groups = vec![
            DrawGroup {
                instances: &before,
                ordered: true,
                environment: sky_group_environment,
                sky: true,
            },
            DrawGroup {
                instances: &world,
                ordered: false,
                environment: None,
                sky: false,
            },
            DrawGroup {
                instances: &after,
                ordered: true,
                environment: sky_group_environment,
                sky: true,
            },
        ];
        groups.extend(interiors.iter().map(|(instances, environment)| DrawGroup {
            instances,
            ordered: false,
            environment: Some(environment),
            sky: false,
        }));
        self.objects.prepare(
            gpu,
            camera,
            &ObjectEnvironment::outside(&lighting),
            aspect,
            time,
            &groups,
        );
    }

    /// `LScapeRender::RefreshShadowMap`: draws the near blocks' static objects (no frills, not
    /// culled; `CLandBlock::GatherObjects` while rendering to the shadow map) into the map when
    /// what it shows changed, and hands the terrain its transform.
    fn refresh_shadow_map(&mut self, gpu: &Gpu) {
        let frame = &self.frame;
        let terrain: Vec<(BlockKey, &TerrainBlock)> = frame
            .terrain
            .iter()
            .filter_map(|key| Some((*key, self.blocks.get(key)?.terrain.as_ref()?)))
            .collect();
        let casters: Vec<ObjectInstance<'_>> = frame
            .placements
            .iter()
            .filter(|p| p.frill_normal.is_none())
            .filter_map(|p| Some(ObjectInstance::new(self.models.get(&p.visual)?, p.world)))
            .collect();
        let matrix = self.shadow_map.refresh(
            gpu,
            &mut self.objects,
            &terrain,
            frame.lighting.light_dir,
            frame.lighting.shadow_strength,
            &casters,
        );
        self.terrain.set_shadow_matrix(matrix);
        self.objects.set_shadow_matrix(matrix);
    }

    /// Records the frame laid out by the last `prepare` into a pass cleared to
    /// `SceneTool::backgroundColor` (black).
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        let (models, blocks, far, frame) = (&self.models, &self.blocks, &self.far, &self.frame);
        let placed = (&self.world_objects, self.frame_objects.as_slice());
        // The sky under `LScapeSky`'s objects, only where there is a landscape sky (not in a
        // dungeon or a UI viewport's scene, which clear to black).
        if landscape_sky(&frame.sky) {
            self.sky.draw(pass);
        }
        self.objects
            .draw(pass, 0, &sky_instances(models, &frame.sky.before));
        // `LScapeSky::Draw`: the sky fog, a fog-coloured quad at alpha `fog_sky` over the
        // before-world sky objects.
        if frame.sky.fog_quad.is_some() {
            self.fog_quad.draw(pass);
        }
        if landscape_sky(&frame.sky) {
            self.sky.draw_clouds(pass);
        }
        self.terrain.draw(
            pass,
            frame
                .terrain
                .iter()
                .filter_map(|key| blocks.get(key)?.terrain.as_ref()),
        );
        if let Some((terrain, _)) = far {
            self.terrain
                .draw_far_chunks(pass, terrain, &frame.far_chunks);
        }
        self.objects.draw(
            pass,
            1,
            &world_instances(models, placed, blocks, far, frame),
        );
        for (index, (instances, _)) in interior_instances(models, blocks, frame).iter().enumerate()
        {
            self.objects.draw(pass, 3 + index, instances);
        }
        self.objects
            .draw(pass, 2, &sky_instances(models, &frame.sky.after));
    }
}

/// Whether the frame has a landscape sky (`LScapeSky`): sky objects, or the fog quad that
/// replaces the before-world ones when the sky is fogged out.
fn landscape_sky(sky: &ac2_render::sky::SkyFrame) -> bool {
    !(sky.before.is_empty() && sky.after.is_empty() && sky.fog_quad.is_none())
}

fn sky_instances<'a>(
    models: &'a HashMap<DataId, GpuObjectModel>,
    placements: &[SkyPlacement],
) -> Vec<ObjectInstance<'a>> {
    placements
        .iter()
        .filter_map(|placement| {
            Some(ObjectInstance {
                tint: placement.tint,
                ..ObjectInstance::new(models.get(&placement.gfx)?, placement.world)
            })
        })
        .collect()
}

/// The frame's placements, then the client's world objects, then the near water of its blocks
/// and the far water (`DrawWaterMesh`). `prepare` and `draw` build it from the same state, so
/// the order matches.
fn world_instances<'a>(
    models: &'a HashMap<DataId, GpuObjectModel>,
    (world_objects, objects): (&'a HashMap<u64, GpuObjectModel>, &'a [SceneObject]),
    blocks: &'a HashMap<BlockKey, GpuBlock>,
    far: &'a Option<(GpuFarTerrain, Option<GpuObjectModel>)>,
    frame: &SceneFrame,
) -> Vec<ObjectInstance<'a>> {
    frame
        .placements
        .iter()
        .filter_map(|p| {
            Some(ObjectInstance {
                frill_normal: p.frill_normal,
                ..ObjectInstance::new(models.get(&p.visual)?, p.world)
            })
        })
        .chain(objects.iter().filter_map(|o| {
            Some(ObjectInstance {
                tint: o.tint,
                pose: o.pose.as_ref(),
                point_lights: &o.point_lights,
                ..ObjectInstance::new(world_objects.get(&o.key)?, o.world)
            })
        }))
        .chain(frame.particles.iter().filter_map(|batch| {
            Some(ObjectInstance::new(
                models.get(&batch.material)?,
                Mat4::IDENTITY,
            ))
        }))
        .chain(
            frame
                .water
                .iter()
                .filter_map(|key| blocks.get(key)?.water.as_ref())
                .chain(far.as_ref().and_then(|(_, water)| water.as_ref()))
                .map(|water| ObjectInstance::new(water, Mat4::IDENTITY)),
        )
        .collect()
}

/// Each drawn interior block's cells and static objects with their point lights, and its inside
/// environment.
fn interior_instances<'a>(
    models: &'a HashMap<DataId, GpuObjectModel>,
    blocks: &'a HashMap<BlockKey, GpuBlock>,
    frame: &SceneFrame,
) -> Vec<(Vec<ObjectInstance<'a>>, ObjectEnvironment)> {
    frame
        .interiors
        .iter()
        .filter_map(|key| blocks.get(key)?.interiors.as_ref())
        .map(|interiors| {
            let cells = interiors
                .cells
                .iter()
                .map(|(model, lights)| ObjectInstance {
                    point_lights: lights,
                    ..ObjectInstance::new(model, Mat4::IDENTITY)
                });
            let objects = interiors
                .objects
                .iter()
                .filter_map(|(visual, world, lights)| {
                    Some(ObjectInstance {
                        point_lights: lights,
                        ..ObjectInstance::new(models.get(visual)?, *world)
                    })
                });
            (cells.chain(objects).collect(), interiors.environment)
        })
        .collect()
}
