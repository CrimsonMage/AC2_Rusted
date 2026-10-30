//! The GPU side of `ac2_render::scene`: uploads what the scene's loader finished and draws a
//! `SceneFrame` in `WorldRender::Draw` order (the sky before the world and its fog quad, the near
//! and far landscape, the objects with near and far water, each block's visible interiors under
//! its inside environment, then the sky after the world and its lens flares). Images arrive
//! decoded with the loads,
//! so nothing here reads the DATs.

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_math::Mat4;
use ac2_render::frustum::{Frustum, Relation};
use ac2_render::object::{ObjectEnvironment, PointLight};
use ac2_render::prefs::ObjectShadows;
use ac2_render::scene::{
    BlockKey, CloudShadow, Images, Loaded, SceneFrame, SceneObject, SceneUpdate,
};
use ac2_render::sky::SkyPlacement;
use ac2_render::{Camera, Image, TextureKey};

use crate::lens_flare::{FlareOrigin, FlareSource, LensFlares};
use crate::sky::{SkyLook, SkyRenderer};
use crate::sun_shadow::{fit_cascades, SunShadows};
use crate::water_reflection::WaterReflection;
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
    /// The env cell each of `cells` and `objects` is in or stands in.
    cell_ids: Vec<u32>,
    object_cells: Vec<u32>,
}

/// The scene's GPU resources.
pub struct SceneRenderer {
    pub terrain: TerrainRenderer,
    objects: ObjectRenderer,
    fog_quad: ScreenQuad,
    /// The sky under the DAT sky objects.
    sky: SkyRenderer,
    /// The sky lights' lens flares and the light coronas over the world.
    flares: LensFlares,
    /// The water's reflection over the world.
    water: WaterReflection,
    /// The near landscape's static shadow map.
    shadow_map: StaticShadowMap,
    /// `LandscapeStaticObjectShadows` and `LandscapeCloudShadows`, the region's cloud shadows
    /// once the far load brings them, and whether the terrain must be bound to them again.
    static_shadows: bool,
    cloud_shadows: bool,
    far_clouds: Option<CloudShadow>,
    rebind_clouds: bool,
    /// The sun shadow cascades of bodies and objects, and the level they are sized for next.
    sun_shadows: SunShadows,
    object_shadows: ObjectShadows,
    models: HashMap<DataId, GpuObjectModel>,
    /// World objects' models by request key.
    world_objects: HashMap<u64, GpuObjectModel>,
    frame_objects: Vec<SceneObject>,
    blocks: HashMap<BlockKey, GpuBlock>,
    far: Option<(GpuFarTerrain, Option<GpuObjectModel>)>,
    /// Every image the loader decoded, by key (it sends each once).
    images: HashMap<TextureKey, Image>,
    frame: SceneFrame,
    /// Which of the frame's world instances `prepare` found in the view (`draw` skips the rest
    /// of the same list).
    world_visible: Vec<bool>,
}

impl SceneRenderer {
    /// Leave out objects whose pipelines are still compiling instead of waiting for them
    /// (`ObjectRenderer::set_stream_pipelines`).
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.objects.set_stream_pipelines(stream);
    }

    /// Object draw calls the last `prepare` laid out (bench statistics).
    pub fn object_draw_count(&self) -> usize {
        self.objects.draw_count()
    }

    /// World instances the last `prepare` left out as outside the view (bench statistics).
    pub fn culled_count(&self) -> usize {
        self.world_visible.iter().filter(|shown| !**shown).count()
    }

    /// Sizes the sun shadows of bodies and objects for the `DynamicObjectShadows` level, from
    /// the next `prepare`.
    pub fn set_object_shadows(&mut self, level: ObjectShadows) {
        self.object_shadows = level;
    }

    /// Whether the near landscape takes the static shadow map (`LandscapeStaticObjectShadows`)
    /// and the cloud shadows (`LandscapeCloudShadows`, only with the map:
    /// `LScapeTerrainMesh::InitFarMesh`, `LScapeMaterialInfo::CombineMaterials`), from the next
    /// `prepare`.
    pub fn set_landscape_shadows(&mut self, static_shadows: bool, cloud_shadows: bool) {
        self.rebind_clouds |=
            (static_shadows, cloud_shadows) != (self.static_shadows, self.cloud_shadows);
        self.static_shadows = static_shadows;
        self.cloud_shadows = cloud_shadows;
    }

    /// Binds the terrain to the region's cloud shadows when both preferences draw them, else
    /// to none (the far mesh's stage 0 is then `ADDSIGNED2X`).
    fn bind_clouds(&mut self, gpu: &Gpu) {
        let clouds = self
            .far_clouds
            .filter(|_| self.static_shadows && self.cloud_shadows);
        let images = &self.images;
        self.terrain
            .set_shadows(gpu, clouds, self.shadow_map.view(), &mut |key| {
                images.get(&key).cloned()
            });
        self.rebind_clouds = false;
    }

    /// Whether the sky lights draw their lens flares and the cell lights their coronas
    /// (`LightCoronas`), from the next `prepare`.
    pub fn set_light_coronas(&mut self, enabled: bool) {
        self.flares.set_enabled(enabled);
    }

    /// What follows the world pass over `hdr`: the water's reflection, then the lens flares'
    /// scan of the world's depth. Record it after the world pass and before anything clears that
    /// depth (the portal space's pass).
    pub fn after_world(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        hdr: &crate::HdrTarget,
    ) {
        self.water.record(gpu, encoder, hdr);
        self.flares.scan(gpu, encoder, &hdr.depth_view);
    }

    /// The renderers of a scene drawn into an [`crate::HdrTarget`] (`HDR_FORMAT`).
    pub fn new(gpu: &Gpu) -> Self {
        let mut terrain = TerrainRenderer::new(gpu, HDR_FORMAT);
        let shadow_map = StaticShadowMap::new(gpu);
        terrain.set_shadows(gpu, None, shadow_map.view(), &mut |_| None);
        let mut objects = ObjectRenderer::new(gpu, HDR_FORMAT);
        objects.set_shadow_map(gpu, shadow_map.view());
        let sun_shadows = SunShadows::new(gpu);
        Self {
            object_shadows: ac2_render::prefs::SceneOptions::default().object_shadows,
            static_shadows: ac2_render::prefs::SceneOptions::default().static_shadows,
            cloud_shadows: ac2_render::prefs::SceneOptions::default().cloud_shadows,
            far_clouds: None,
            rebind_clouds: false,
            terrain,
            shadow_map,
            sun_shadows,
            objects,
            fog_quad: ScreenQuad::new(gpu, HDR_FORMAT),
            sky: SkyRenderer::new(gpu, HDR_FORMAT),
            flares: LensFlares::new(gpu, HDR_FORMAT),
            water: WaterReflection::new(gpu),
            models: HashMap::new(),
            world_objects: HashMap::new(),
            frame_objects: Vec::new(),
            blocks: HashMap::new(),
            far: None,
            images: HashMap::new(),
            frame: SceneFrame::default(),
            world_visible: Vec::new(),
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
                            cell_ids: interiors.cell_ids,
                            object_cells: interiors.object_cells,
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
                Loaded::Images(images) => self.take_images(images),
                Loaded::LightDecalMaterials(_) | Loaded::ShoreMaterial(_) => {}
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
                    self.far_clouds = Some(far.clouds);
                    self.bind_clouds(gpu);
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
        for batch in self.frame.particles.iter().chain(&self.frame.shore) {
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
        if self.rebind_clouds {
            self.bind_clouds(gpu);
        }
        self.refresh_shadow_map(gpu);
        self.draw_sun_shadows(gpu, camera, aspect);
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
            wind: self.far_clouds.map_or(crate::sky::DEFAULT_CLOUD_WIND, |c| {
                crate::sky::cloud_wind(&c)
            }),
            ..SkyLook::default()
        };
        self.sky
            .set_frame(&gpu.queue, camera, aspect, &lighting, &look);
        let images = &self.images;
        self.flares.prepare(
            gpu,
            camera,
            aspect,
            time,
            &flare_sources(frame),
            &mut |key| images.get(&key).cloned(),
        );
        self.terrain
            .set_frame(&gpu.queue, camera, &lighting, aspect, time);
        let horizon = lighting
            .fog_color
            .map(|c| crate::shading::srgb_to_linear(c.clamp(0.0, 1.0)));
        self.water.prepare(
            gpu,
            camera,
            aspect,
            time,
            frame.reflection.as_ref(),
            horizon,
        );
        let all_world = world_instances(models, placed, blocks, far, frame);
        let world_visible = in_view(&all_world, &Frustum::from_camera(camera, aspect));
        let world = visible(&all_world, &world_visible);
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
        // `DrawShore(true)`, after the water.
        let shore = shore_instances(models, frame);
        groups.push(DrawGroup {
            instances: &shore,
            ordered: true,
            environment: None,
            sky: false,
        });
        self.objects.set_grass_pushers(&frame.grass_pushers);
        self.objects.prepare(
            gpu,
            camera,
            &ObjectEnvironment::outside(&lighting),
            aspect,
            time,
            &groups,
        );
        self.world_visible = world_visible;
    }

    /// `LScapeRender::RefreshShadowMap`: draws the near blocks' static objects (no frills, not
    /// culled; `CLandBlock::GatherObjects` while rendering to the shadow map) into the map when
    /// what it shows changed, and hands the terrain its transform.
    fn refresh_shadow_map(&mut self, gpu: &Gpu) {
        let frame = &self.frame;
        // Without `LandscapeStaticObjectShadows` there is no map: the terrain samples it white.
        let shown: &[BlockKey] = if self.static_shadows {
            &frame.terrain
        } else {
            &[]
        };
        let terrain: Vec<(BlockKey, &TerrainBlock)> = shown
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

    /// The sun shadow cascades around `camera`: the near blocks' static objects (no frills) and the
    /// frame's bodies and world objects cast into them, and the receivers read them at the static
    /// shadow strength. Nothing casts in a scene without a sun (dungeons, UI viewports).
    fn draw_sun_shadows(&mut self, gpu: &Gpu, camera: &Camera, aspect: f32) {
        let lighting = &self.frame.lighting;
        let lit = lighting.light_dir.length() > 1e-4
            && lighting.shadow_strength.iter().any(|s| *s > 0.0)
            && !self.frame.terrain.is_empty();
        // The maps are made the first time a sunlit landscape needs them (a UI viewport's scene
        // never does), and dropped when the level turns them off.
        let level = self.object_shadows;
        if (lit || level == ObjectShadows::Off) && self.sun_shadows.set_level(gpu, level) {
            self.terrain.set_sun_shadow(gpu, self.sun_shadows.binding());
            self.objects.set_sun_shadow(gpu, self.sun_shadows.binding());
        }
        let frame = &self.frame;
        let lighting = &frame.lighting;
        let tier = self.sun_shadows.tier();
        let cascades = tier
            .filter(|_| lit)
            .and_then(|tier| fit_cascades(camera, aspect, lighting.light_dir, &tier))
            .unwrap_or_default();
        if !cascades.is_empty() {
            let casters: Vec<ObjectInstance<'_>> = frame
                .placements
                .iter()
                .filter(|p| p.frill_normal.is_none())
                .filter_map(|p| {
                    Some(ObjectInstance {
                        tint: [1.0, 1.0, 1.0, p.alpha],
                        detail: p.detail,
                        ..ObjectInstance::new(self.models.get(&p.visual)?, p.world)
                    })
                })
                .chain(self.frame_objects.iter().filter_map(|o| {
                    Some(ObjectInstance {
                        tint: o.tint,
                        pose: o.pose.as_ref(),
                        detail: o.detail,
                        ..ObjectInstance::new(self.world_objects.get(&o.key)?, o.world)
                    })
                }))
                .collect();
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            self.objects.draw_sun_casters(
                gpu,
                &mut encoder,
                self.sun_shadows.layer_views(),
                (camera.eye, &cascades),
                &casters,
            );
            gpu.queue.submit([encoder.finish()]);
        }
        self.sun_shadows
            .write(gpu, camera.eye, &cascades, lighting.shadow_strength);
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
            &visible(
                &world_instances(models, placed, blocks, far, frame),
                &self.world_visible,
            ),
        );
        let interiors = interior_instances(models, blocks, frame);
        for (index, (instances, _)) in interiors.iter().enumerate() {
            self.objects.draw(pass, 3 + index, instances);
        }
        self.objects
            .draw(pass, 3 + interiors.len(), &shore_instances(models, frame));
        self.objects
            .draw(pass, 2, &sky_instances(models, &frame.sky.after));
        // `LScapeSky::Draw(1)`: then each sky light's `LensFlareEffect::Draw`, with the drawn
        // cells' coronas (`CEnvCell::DrawPostEffects`), indoors as well.
        self.flares.draw(pass);
    }
}

/// The frame's lens flares: the sky lights' (`LScapeSky::Draw(1)`, starting unseen as
/// `LensFlareEffect::LensFlareEffect` does) and the drawn cells' light coronas, each named by its
/// block and light so it keeps its fade.
fn flare_sources(frame: &SceneFrame) -> Vec<FlareSource<'_>> {
    const SKY: u64 = 1 << 63;
    let sky = frame
        .sky
        .flares
        .iter()
        .enumerate()
        .map(|(index, flare)| FlareSource {
            key: SKY | index as u64,
            origin: FlareOrigin::Infinite(flare.direction),
            start_opacity: 0.0,
            flare: &flare.flare,
        });
    let coronas = frame.coronas.iter().map(|((x, y), corona)| FlareSource {
        key: (u64::from(*x) << 40) | (u64::from(*y) << 32) | u64::from(corona.light),
        origin: FlareOrigin::Point(corona.origin),
        start_opacity: corona.opacity,
        flare: &corona.flare,
    });
    sky.chain(coronas).collect()
}

/// The shore's breaking water (`DrawShore(true)`), drawn in world space after the water.
fn shore_instances<'a>(
    models: &'a HashMap<DataId, GpuObjectModel>,
    frame: &SceneFrame,
) -> Vec<ObjectInstance<'a>> {
    frame
        .shore
        .iter()
        .filter_map(|batch| {
            Some(ObjectInstance::new(
                models.get(&batch.material)?,
                Mat4::IDENTITY,
            ))
        })
        .collect()
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

/// Whether each instance can be seen: `Frustum::CheckRelationship` of the box around its
/// bounding sphere (the model's farthest vertex from its origin, scaled by the placement; a posed
/// model's limbs may reach past its rest pose, so twice that). Instances drawn in world space
/// (particles, water) are kept.
fn in_view(instances: &[ObjectInstance<'_>], frustum: &Frustum) -> Vec<bool> {
    instances
        .iter()
        .map(|instance| {
            if instance.world == Mat4::IDENTITY {
                return true;
            }
            let scale = [
                instance.world.x_axis,
                instance.world.y_axis,
                instance.world.z_axis,
            ]
            .map(|axis| axis.truncate().length())
            .into_iter()
            .fold(0.0_f32, f32::max);
            let pose = if instance.pose.is_some() { 2.0 } else { 1.0 };
            let radius = instance.model.bounding_radius() * scale * pose;
            let center = instance.world.w_axis.truncate();
            let reach = ac2_math::Vec3::splat(radius);
            frustum.check(center - reach, center + reach) != Relation::Outside
        })
        .collect()
}

/// The instances `mask` keeps, in order.
fn visible<'a>(instances: &[ObjectInstance<'a>], mask: &[bool]) -> Vec<ObjectInstance<'a>> {
    instances
        .iter()
        .zip(mask)
        .filter(|(_, shown)| **shown)
        .map(|(instance, _)| *instance)
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
                frill_shade: p.frill_shade,
                grass: p.grass,
                // `CPartArray::Draw`'s fade on a white colour.
                tint: [1.0, 1.0, 1.0, p.alpha],
                detail: p.detail,
                ..ObjectInstance::new(models.get(&p.visual)?, p.world)
            })
        })
        .chain(objects.iter().filter_map(|o| {
            Some(ObjectInstance {
                tint: o.tint,
                pose: o.pose.as_ref(),
                point_lights: &o.point_lights,
                detail: o.detail,
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
        .filter_map(|key| {
            Some((
                blocks.get(key)?.interiors.as_ref()?,
                frame.interior_cells.get(key),
            ))
        })
        .map(|(interiors, visible)| {
            // A block without a visibility decision draws all of its cells.
            let drawn = |id: &u32| visible.is_none_or(|ids| ids.contains(id));
            let cells = interiors
                .cells
                .iter()
                .zip(&interiors.cell_ids)
                .filter(|(_, id)| drawn(id))
                .map(|((model, lights), _)| ObjectInstance {
                    point_lights: lights,
                    ..ObjectInstance::new(model, Mat4::IDENTITY)
                });
            let objects = interiors
                .objects
                .iter()
                .zip(&interiors.object_cells)
                .filter(|(_, id)| drawn(id))
                .filter_map(|((visual, world, lights), _)| {
                    Some(ObjectInstance {
                        point_lights: lights,
                        ..ObjectInstance::new(models.get(visual)?, *world)
                    })
                });
            (cells.chain(objects).collect(), interiors.environment)
        })
        .collect()
}
