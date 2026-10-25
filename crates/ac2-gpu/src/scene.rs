//! The GPU side of `ac2_render::scene`: uploads what the scene's loader finished and draws a
//! `SceneFrame` in `WorldRender::Draw` order (the sky before the world and its fog quad, the near
//! and far landscape, the objects with near and far water, each block's visible interiors under
//! its inside environment, then the sky after the world). Images arrive decoded with the loads,
//! so nothing here reads the DATs.

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_math::Mat4;
use ac2_render::object::{ObjectEnvironment, PointLight};
use ac2_render::scene::{BlockKey, Images, Loaded, SceneFrame, SceneUpdate};
use ac2_render::sky::SkyPlacement;
use ac2_render::{Camera, Image, TextureKey};

use crate::{
    DrawGroup, Gpu, GpuFarTerrain, GpuObjectModel, ObjectInstance, ObjectRenderer, ScreenQuad,
    TerrainBlock, TerrainRenderer,
};

/// One resident block on the GPU.
struct GpuBlock {
    terrain: Option<TerrainBlock>,
    water: Option<GpuObjectModel>,
    interiors: Option<(ObjectEnvironment, Vec<(GpuObjectModel, Vec<PointLight>)>)>,
}

/// The scene's GPU resources.
pub struct SceneRenderer {
    pub terrain: TerrainRenderer,
    objects: ObjectRenderer,
    fog_quad: ScreenQuad,
    models: HashMap<DataId, GpuObjectModel>,
    blocks: HashMap<BlockKey, GpuBlock>,
    far: Option<(GpuFarTerrain, Option<GpuObjectModel>)>,
    /// Every image the loader decoded, by key (it sends each once).
    images: HashMap<TextureKey, Image>,
    frame: SceneFrame,
}

impl SceneRenderer {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        Self {
            terrain: TerrainRenderer::new(gpu, format),
            objects: ObjectRenderer::new(gpu, format),
            fog_quad: ScreenQuad::new(gpu, format),
            models: HashMap::new(),
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
                        (interiors.environment, cells)
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
                    let water = far
                        .water
                        .map(|model| self.objects.upload(gpu, &model, &mut load));
                    self.far = Some((terrain, water));
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
    ) {
        self.frame = frame;
        let lighting = self.frame.lighting;
        self.terrain
            .set_frame(&gpu.queue, camera, &lighting, aspect);
        if let Some(color) = self.frame.sky.fog_quad {
            self.fog_quad.set_color(&gpu.queue, color);
        }
        let (models, blocks, far, frame) = (&self.models, &self.blocks, &self.far, &self.frame);
        let before = sky_instances(models, &frame.sky.before);
        let world = world_instances(models, blocks, far, frame);
        let after = sky_instances(models, &frame.sky.after);
        let interiors = interior_instances(blocks, frame);
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
        groups.extend(interiors.iter().map(|(instances, environment)| DrawGroup {
            instances,
            ordered: false,
            environment: Some(environment),
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

    /// Records the frame laid out by the last `prepare` into a pass cleared to
    /// `SceneTool::backgroundColor` (black).
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        let (models, blocks, far, frame) = (&self.models, &self.blocks, &self.far, &self.frame);
        self.objects
            .draw(pass, 0, &sky_instances(models, &frame.sky.before));
        if frame.sky.fog_quad.is_some() {
            self.fog_quad.draw(pass);
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
        self.objects
            .draw(pass, 1, &world_instances(models, blocks, far, frame));
        for (index, (instances, _)) in interior_instances(blocks, frame).iter().enumerate() {
            self.objects.draw(pass, 3 + index, instances);
        }
        self.objects
            .draw(pass, 2, &sky_instances(models, &frame.sky.after));
    }
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

/// The frame's object instances followed by the near water of its blocks and the far water
/// (`DrawWaterMesh`). `prepare` and `draw` build it from the same state, so the order matches.
fn world_instances<'a>(
    models: &'a HashMap<DataId, GpuObjectModel>,
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

/// Each drawn interior block's cells with their point lights, and its inside environment.
fn interior_instances<'a>(
    blocks: &'a HashMap<BlockKey, GpuBlock>,
    frame: &SceneFrame,
) -> Vec<(Vec<ObjectInstance<'a>>, ObjectEnvironment)> {
    frame
        .interiors
        .iter()
        .filter_map(|key| blocks.get(key)?.interiors.as_ref())
        .map(|(environment, cells)| {
            let instances = cells
                .iter()
                .map(|(model, lights)| ObjectInstance {
                    point_lights: lights,
                    ..ObjectInstance::new(model, Mat4::IDENTITY)
                })
                .collect();
            (instances, *environment)
        })
        .collect()
}
