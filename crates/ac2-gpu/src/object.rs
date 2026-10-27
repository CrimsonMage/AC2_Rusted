//! The static-object pass: one pipeline per blend/depth/cull state (`SetSourceBlend`,
//! `SetDestBlend`, `SetDepthWrite`, `SetDepthTest`, `D3DRS_CULLMODE`), one draw per material layer
//! (`RenderIndexedUsingMaterial` draws each drawn layer in turn), with that layer's material,
//! program and texture stages in a dynamic-offset uniform. A posed instance draws each part at its
//! `CPhysicsPart::m_mObject`, and parts with matrix weights blend their vertices by their group's
//! deform matrices (`SceneTool::SetMatrixStack`), read from a per-frame storage buffer.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use ac2_math::{Mat4, Vec3};
use ac2_render::object::{
    LayerPass, ObjectEnvironment, ObjectMaterial, ObjectModel, ObjectPose, ObjectVertex, PointLight,
};
use ac2_render::{Camera, Image};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::object_draw::{
    bone_storage, draw_storage, draw_uniform, mark_sky, set_point_lights, DrawUniform, SkyRole,
};
use crate::object_pipelines::PipelineCompiler;
use crate::object_state::{
    pipeline_key, sampler_key, sky_depth, sky_veil, PipelineKey, SamplerKey, DRAW_STRIDE,
};
use crate::pixel_program::shade_function;
use crate::shading::FRAME_SHADING;
use crate::texture::{TextureCache, TextureKey};
use crate::Gpu;

#[path = "object_shadow.rs"]
mod shadow;

#[cfg(test)]
#[path = "object_stream_tests.rs"]
mod stream_tests;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    uvs: [[f32; 2]; 4],
    tangents: [[f32; 3]; 2],
    /// Up to four (bone, weight) influences; all-zero weights for a rigid part.
    bones: [u8; 4],
    weights: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FrameUniform {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    eye: [f32; 4],
    to_viewer: [f32; 4],
    sun_inv_dir: [f32; 4],
    sun_diffuse: [f32; 4],
    sun_specular: [f32; 4],
    ambient: [f32; 4],
    fog_color: [f32; 4],
    fog: [f32; 4],
    shading: [f32; 4],
    /// World to the landscape's static shadow map (`crate::StaticShadowMap`).
    shadow: [[f32; 4]; 4],
}

/// Byte stride of one draw group's frame uniform (the dynamic-offset alignment; the frame
/// uniform is 272 bytes).
const FRAME_STRIDE: u64 = 512;
/// Draw groups one `prepare` lays out.
const MAX_GROUPS: usize = 16;
/// Bytes of one bone matrix in the storage buffer.
const BONE_BYTES: u64 = 64;

/// One drawn layer of a batch: its state and the bind group of its stage textures.
struct GpuPass {
    pass: LayerPass,
    /// Shader module index: 0 for the stage cascade, else the layer's pixel shader.
    program: u32,
    textures: wgpu::BindGroup,
}

struct GpuBatch {
    range: Range<u32>,
    material: Arc<ObjectMaterial>,
    passes: Vec<GpuPass>,
}

struct GpuPart {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    transform: Mat4,
    batches: Vec<GpuBatch>,
    /// The mesh carries matrix weights.
    weighted: bool,
    /// The buffers were made by `set_geometry` (`COPY_DST`, power-of-two capacity) and are
    /// refilled in place from then on.
    streamed: bool,
}

/// An uploaded `ObjectModel`.
pub struct GpuObjectModel {
    parts: Vec<GpuPart>,
    /// Model-space centre of the vertices, for back-to-front ordering of translucent layers.
    center: Vec3,
    /// Index and vertex counts of the first part's first triangle list (`FrillRender` budgets).
    first_list: (u32, u32),
    /// The farthest vertex from the model's origin.
    radius: f32,
}

/// Every `GpuVertex` is a whole number of 4-byte words, so `write_buffer` sizes need no padding.
const _: () = assert!(std::mem::size_of::<GpuVertex>().is_multiple_of(4));

/// A buffer for `set_geometry`: at least one copy alignment unit, rounded up to a power of two.
fn stream_buffer(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    bytes: u64,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(wgpu::COPY_BUFFER_ALIGNMENT).next_power_of_two(),
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl GpuObjectModel {
    /// Index and vertex counts of the first part's first triangle list.
    pub fn first_list(&self) -> (u32, u32) {
        self.first_list
    }
}

/// The sky's backdrop among a group of DAT sky objects: the largest, the dome `LScapeSky` draws
/// first (`0x04000022` in the shipped region). The modern sky takes its colour from the
/// atmosphere and grades it by this object's tint; the dome itself adds only its texture's
/// detail (`FLAG_SKY_BACKDROP`).
pub fn sky_backdrop(instances: &[ObjectInstance<'_>]) -> Option<usize> {
    instances
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.model.radius.total_cmp(&b.1.model.radius))
        .map(|(index, _)| index)
}

/// One draw of the frame: pipeline, uniform offset, and instance/part/batch/layer indices.
#[derive(Clone, Copy)]
struct QueuedDraw {
    key: PipelineKey,
    /// The instance waits for a pipeline still compiling, so none of its draws record.
    deferred: bool,
    offset: u32,
    instance: usize,
    part: usize,
    batch: usize,
    layer: usize,
}

/// One placed model.
#[derive(Clone, Copy)]
pub struct ObjectInstance<'a> {
    pub model: &'a GpuObjectModel,
    pub world: Mat4,
    /// Material tint (`CPhysicsObj` diffuse colour and opacity); white leaves it unchanged.
    pub tint: [f32; 4],
    /// A grass frill merged by `FrillRender`: every vertex takes this world normal and the
    /// landscape shadow colour under its origin (`FrillRenderType::AddPos`).
    pub frill_normal: Option<Vec3>,
    /// The device's point lights for this object (`SceneTool::CacheStaticLights`), at most 4.
    pub point_lights: &'a [PointLight],
    /// The model posed for this draw (`ObjectModel::pose`); `None` draws its default placement.
    pub pose: Option<&'a ObjectPose>,
}

impl<'a> ObjectInstance<'a> {
    /// An untinted instance.
    pub fn new(model: &'a GpuObjectModel, world: Mat4) -> Self {
        Self {
            model,
            world,
            tint: [1.0; 4],
            frill_normal: None,
            point_lights: &[],
            pose: None,
        }
    }
}

/// Instances laid out together; `ordered` keeps their order instead of sorting translucency.
#[derive(Clone, Copy)]
pub struct DrawGroup<'a> {
    pub instances: &'a [ObjectInstance<'a>],
    pub ordered: bool,
    /// The environment the group draws in (`WorldRender::SetupEnvironment` before it); `None`
    /// uses the frame's.
    pub environment: Option<&'a ObjectEnvironment>,
    /// The instances are DAT sky objects (`LScapeSky::Draw`): no depth test or write.
    pub sky: bool,
}

pub struct ObjectRenderer {
    /// Shader module indices by translated pixel shader: 0 is the stage cascade, then one per
    /// distinct program, compiled with the pipelines on the compiler's worker.
    program_ids: HashMap<String, u32>,
    pipelines: PipelineCompiler,
    /// Whether `prepare` leaves out an instance whose pipelines are still compiling (the
    /// running client) rather than waiting for them (headless renders and tests).
    stream_pipelines: bool,
    frame_buffer: wgpu::Buffer,
    frame_group: wgpu::BindGroup,
    frame_layout: wgpu::BindGroupLayout,
    shadow_sampler: wgpu::Sampler,
    /// World to the static shadow map, for the frills' landscape shadow colour.
    shadow_matrix: Mat4,
    draw_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    draw_buffer: wgpu::Buffer,
    draw_group: wgpu::BindGroup,
    draw_capacity: u64,
    bone_layout: wgpu::BindGroupLayout,
    bone_buffer: wgpu::Buffer,
    bone_group: wgpu::BindGroup,
    bone_capacity: u64,
    samplers: HashMap<SamplerKey, wgpu::Sampler>,
    pub textures: TextureCache,
    time: f64,
    /// Laid out by `prepare`, recorded by `draw`.
    queue: Vec<QueuedDraw>,
    /// Ranges of `queue` by draw group.
    groups: Vec<Range<usize>>,
    /// The static shadow map's caster pipeline, made on first use.
    casters: Option<shadow::CasterPass>,
    shadow_format: wgpu::TextureFormat,
    /// `prepare`'s working storage, cleared and refilled each frame so steady frames allocate
    /// nothing. Never shrunk.
    scratch_entries: Vec<(DrawUniform, QueuedDraw)>,
    scratch_opaque: Vec<(DrawUniform, QueuedDraw)>,
    scratch_translucent: Vec<(f32, (DrawUniform, QueuedDraw))>,
    scratch_bones: Vec<[[f32; 4]; 4]>,
    scratch_bytes: Vec<u8>,
    /// `set_geometry`'s converted vertices.
    scratch_vertices: Vec<GpuVertex>,
}

impl ObjectRenderer {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let uniform = |binding, dynamic| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        };
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object frame"),
            entries: &[
                uniform(0, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object draw"),
            entries: &[uniform(0, true)],
        });
        let mut texture_entries = Vec::new();
        for binding in 0..4 {
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
        }
        for binding in 4..8 {
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object stages"),
            entries: &texture_entries,
        });
        let bone_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object bones"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 object frame uniform"),
            size: FRAME_STRIDE * MAX_GROUPS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let textures = TextureCache::new(device, &gpu.queue);
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 object shadow map"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let frame_group = frame_group(
            device,
            &frame_layout,
            &frame_buffer,
            textures.white(),
            &shadow_sampler,
        );
        let module = object_module(device, include_str!("object_stages.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 object"),
            bind_group_layouts: &[
                Some(&frame_layout),
                Some(&draw_layout),
                Some(&texture_layout),
                Some(&bone_layout),
            ],
            immediate_size: 0,
        });
        let (draw_buffer, draw_group) = draw_storage(device, &draw_layout, DRAW_STRIDE * 64);
        let (bone_buffer, bone_group) = bone_storage(device, &bone_layout, BONE_BYTES * 64);
        let pipelines = PipelineCompiler::new(gpu, &layout, format, module);
        Self {
            program_ids: HashMap::new(),
            pipelines,
            stream_pipelines: false,
            frame_buffer,
            frame_group,
            frame_layout,
            shadow_sampler,
            shadow_matrix: Mat4::ZERO,
            draw_layout,
            texture_layout,
            draw_buffer,
            draw_group,
            draw_capacity: DRAW_STRIDE * 64,
            bone_layout,
            bone_buffer,
            bone_group,
            bone_capacity: BONE_BYTES * 64,
            samplers: HashMap::new(),
            textures,
            time: 0.0,
            queue: Vec::new(),
            groups: Vec::new(),
            casters: None,
            shadow_format: crate::SHADOW_MAP_FORMAT,
            scratch_entries: Vec::new(),
            scratch_opaque: Vec::new(),
            scratch_translucent: Vec::new(),
            scratch_bones: Vec::new(),
            scratch_bytes: Vec::new(),
            scratch_vertices: Vec::new(),
        }
    }

    /// Uploads a model; `load` supplies images for stage textures not yet on the GPU.
    /// Binds the landscape's static shadow map, which merged frills read their colour from.
    pub fn set_shadow_map(&mut self, gpu: &Gpu, view: &wgpu::TextureView) {
        self.frame_group = frame_group(
            &gpu.device,
            &self.frame_layout,
            &self.frame_buffer,
            view,
            &self.shadow_sampler,
        );
    }

    /// The world-to-shadow-map transform for the next `prepare` (zero without a map).
    pub fn set_shadow_matrix(&mut self, matrix: Mat4) {
        self.shadow_matrix = matrix;
    }

    pub fn upload(
        &mut self,
        gpu: &Gpu,
        model: &ObjectModel,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> GpuObjectModel {
        let device = &gpu.device;
        let mut parts = Vec::with_capacity(model.parts.len());
        let (mut sum, mut count) = (Vec3::ZERO, 0.0_f32);
        let mut radius = 0.0_f32;
        for part in &model.parts {
            let vertices: Vec<GpuVertex> = part
                .vertices
                .iter()
                .enumerate()
                .map(|(index, v)| {
                    let influences = part
                        .skin
                        .as_ref()
                        .and_then(|skin| skin.get(index))
                        .copied()
                        .unwrap_or_default();
                    GpuVertex {
                        position: v.position,
                        normal: v.normal,
                        color: v.color,
                        uvs: v.uvs,
                        tangents: v.tangents,
                        bones: influences.map(|(bone, _)| bone),
                        weights: influences.map(|(_, weight)| weight),
                    }
                })
                .collect();
            for v in part.vertices.iter() {
                let position = part
                    .transform
                    .transform_point3(Vec3::from_array(v.position));
                sum += position;
                radius = radius.max(position.length());
                count += 1.0;
            }
            let mut indices = Vec::new();
            let mut batches = Vec::with_capacity(part.batches.len());
            for batch in &part.batches {
                let start = indices.len() as u32;
                indices.extend_from_slice(&batch.indices);
                let passes = batch
                    .material
                    .passes([1.0; 4], self.time)
                    .into_iter()
                    .map(|pass| GpuPass {
                        textures: self.stage_group(gpu, &pass, load),
                        program: self.program_id(&pass),
                        pass,
                    })
                    .collect();
                batches.push(GpuBatch {
                    range: start..indices.len() as u32,
                    material: Arc::clone(&batch.material),
                    passes,
                });
            }
            parts.push(GpuPart {
                vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ac2 object vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ac2 object indices"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
                transform: part.transform,
                batches,
                weighted: part.skin.is_some(),
                streamed: false,
            });
        }
        let first_list = model.parts.first().map_or((0, 0), |part| {
            let indices = part.batches.first().map_or(0, |b| b.indices.len());
            (indices as u32, part.vertices.len() as u32)
        });
        GpuObjectModel {
            parts,
            center: if count > 0.0 { sum / count } else { Vec3::ZERO },
            first_list,
            radius,
        }
    }

    /// Replaces the geometry of `model`'s first part and batch: a particle material's model,
    /// whose triangles are laid out again every frame (`ParticleSystem::FillVertexBuffer`).
    pub fn set_geometry(
        &mut self,
        gpu: &Gpu,
        model: &mut GpuObjectModel,
        vertices: &[ObjectVertex],
        indices: &[u32],
    ) {
        let Some(part) = model.parts.first_mut() else {
            return;
        };
        self.scratch_vertices.clear();
        self.scratch_vertices
            .extend(vertices.iter().map(|v| GpuVertex {
                position: v.position,
                normal: v.normal,
                color: v.color,
                uvs: v.uvs,
                tangents: v.tangents,
                bones: [0; 4],
                weights: [0.0; 4],
            }));
        let vertex_bytes: &[u8] = bytemuck::cast_slice(&self.scratch_vertices);
        let index_bytes: &[u8] = bytemuck::cast_slice(indices);
        // The first call replaces the upload's buffers, which lack `COPY_DST`; later calls refill
        // them until the live geometry outgrows their capacity.
        let first = !part.streamed;
        if first || vertex_bytes.len() as u64 > part.vertices.size() {
            part.vertices = stream_buffer(
                &gpu.device,
                "ac2 particle vertices",
                wgpu::BufferUsages::VERTEX,
                vertex_bytes.len() as u64,
            );
        }
        if first || index_bytes.len() as u64 > part.indices.size() {
            part.indices = stream_buffer(
                &gpu.device,
                "ac2 particle indices",
                wgpu::BufferUsages::INDEX,
                index_bytes.len() as u64,
            );
        }
        part.streamed = true;
        // The draw range below bounds the indices drawn, so a larger buffer's tail is never read.
        if !vertex_bytes.is_empty() {
            gpu.queue.write_buffer(&part.vertices, 0, vertex_bytes);
        }
        if !index_bytes.is_empty() {
            gpu.queue.write_buffer(&part.indices, 0, index_bytes);
        }
        if let Some(batch) = part.batches.first_mut() {
            batch.range = 0..indices.len() as u32;
        }
        let sum: Vec3 = vertices.iter().map(|v| Vec3::from_array(v.position)).sum();
        model.center = sum / vertices.len().max(1) as f32;
        model.radius = vertices
            .iter()
            .map(|v| Vec3::from_array(v.position).length())
            .fold(0.0, f32::max);
    }

    /// The module index for a pass's pixel shader, translating and compiling it on first use.
    fn program_id(&mut self, pass: &LayerPass) -> u32 {
        let Some(program) = &pass.pixel_program else {
            return 0;
        };
        let source = shade_function(program);
        if let Some(id) = self.program_ids.get(&source) {
            return *id;
        }
        let id = self.program_ids.len() as u32 + 1;
        self.pipelines.add_module(id, source.clone());
        self.program_ids.insert(source, id);
        id
    }

    fn stage_group(
        &mut self,
        gpu: &Gpu,
        pass: &LayerPass,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> wgpu::BindGroup {
        let mut views = Vec::with_capacity(4);
        let mut samplers = Vec::with_capacity(4);
        for index in 0..4 {
            let stage = pass.stages.get(index);
            // An untextured stage reads `D3DTA_TEXTURE` as white in the fixed-function cascade.
            // Retail binds no texture for an authored pixel shader's empty stage either
            // (`RenderDeviceD3D::SetStageTexture(stage, NULL)`), and the DX8 documentation leaves
            // that `tex` read undefined. The human body and clothing shaders add their stage 2 as
            // a centred detail (`add t1.rgb, t1, t2_bias`) and leave stage 2 empty, so it reads
            // the detail's neutral value: mid grey keeps their authored albedo (white washed the
            // skin out to chalk, black crushed it to brown).
            let view = match stage.and_then(|s| s.texture) {
                Some(did) => {
                    self.textures
                        .get(&gpu.device, &gpu.queue, TextureKey::Texture(did), load)
                }
                None if pass.pixel_program.is_some() => self.textures.neutral().clone(),
                None => self.textures.white().clone(),
            };
            views.push(view);
            let key = sampler_key(stage);
            let sampler = self
                .samplers
                .entry(key)
                .or_insert_with(|| key.create(&gpu.device))
                .clone();
            samplers.push(sampler);
        }
        let mut entries = Vec::with_capacity(8);
        for (binding, view) in views.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
        for (index, sampler) in samplers.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: 4 + index as u32,
                resource: wgpu::BindingResource::Sampler(sampler),
            });
        }
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 object stages"),
            layout: &self.texture_layout,
            entries: &entries,
        })
    }

    /// Writes the frame's camera, lights and fog, re-evaluates animated layers at client time
    /// `time`, and lays out this frame's draws: layers that write depth in instance order, then the
    /// rest back to front.
    /// Writes the frame's camera, lights and fog, re-evaluates animated or tinted layers at
    /// client time `time`, and lays out each group's draws. A group in draw order keeps its
    /// instances' order (the sky); otherwise layers that write depth go first in instance order
    /// and the rest back to front.
    pub fn prepare(
        &mut self,
        gpu: &Gpu,
        camera: &Camera,
        environment: &ObjectEnvironment,
        aspect: f32,
        time: f64,
        groups: &[DrawGroup<'_>],
    ) {
        self.time = time;
        assert!(
            groups.len() <= MAX_GROUPS,
            "at most {MAX_GROUPS} draw groups"
        );
        for (index, group) in groups.iter().enumerate() {
            let environment = group.environment.unwrap_or(environment);
            self.write_frame(&gpu.queue, index, camera, environment, aspect);
        }
        self.scratch_entries.clear();
        self.scratch_bones.clear();
        self.groups.clear();
        for group in groups {
            let start = self.scratch_entries.len();
            self.scratch_opaque.clear();
            self.scratch_translucent.clear();
            let backdrop = group.sky.then(|| sky_backdrop(group.instances)).flatten();
            for (instance_index, instance) in group.instances.iter().enumerate() {
                let center = instance.world.transform_point3(instance.model.center);
                let distance = center.distance_squared(camera.eye);
                let base = self.scratch_bones.len() as u32;
                if let Some(pose) = instance.pose {
                    self.scratch_bones
                        .extend(pose.palette.iter().map(Mat4::to_cols_array_2d));
                }
                for (part_index, part) in instance.model.parts.iter().enumerate() {
                    let (transform, skin) = match instance.pose {
                        Some(pose) => (
                            pose.parts
                                .get(part_index)
                                .copied()
                                .unwrap_or(part.transform),
                            pose.skins
                                .get(part_index)
                                .copied()
                                .flatten()
                                .filter(|_| part.weighted),
                        ),
                        None => (part.transform, None),
                    };
                    let model = instance.world * transform;
                    for (batch_index, batch) in part.batches.iter().enumerate() {
                        let recomputed = (batch.material.animated() || instance.tint != [1.0; 4])
                            .then(|| batch.material.passes(instance.tint, time));
                        for (pass_index, gpu_pass) in batch.passes.iter().enumerate() {
                            let pass = recomputed
                                .as_ref()
                                .map_or(&gpu_pass.pass, |p| &p[pass_index]);
                            let mut key = pipeline_key(pass, gpu_pass.program);
                            let role = SkyRole::of(
                                pass,
                                backdrop == Some(instance_index),
                                instance.model.radius,
                            );
                            if group.sky {
                                key = sky_depth(key);
                                if role == SkyRole::Veil {
                                    key = sky_veil(key);
                                }
                            }
                            let draw = QueuedDraw {
                                key,
                                deferred: false,
                                offset: 0,
                                instance: instance_index,
                                part: part_index,
                                batch: batch_index,
                                layer: pass_index,
                            };
                            let mut uniform = draw_uniform(model, pass);
                            if group.sky {
                                mark_sky(&mut uniform, pass, role);
                            }
                            if let Some(normal) = instance.frill_normal {
                                uniform.frill = normal.extend(1.0).to_array();
                            }
                            set_point_lights(&mut uniform, instance.point_lights);
                            if let Some(first) = skin {
                                uniform.skin = [base + first, 1, 0, 0];
                            }
                            let entry = (uniform, draw);
                            if group.ordered || pass.depth_write {
                                self.scratch_opaque.push(entry);
                            } else {
                                self.scratch_translucent.push((distance, entry));
                            }
                        }
                    }
                }
            }
            self.scratch_translucent.sort_by(|a, b| b.0.total_cmp(&a.0));
            self.scratch_entries.append(&mut self.scratch_opaque);
            self.scratch_entries
                .extend(self.scratch_translucent.drain(..).map(|(_, entry)| entry));
            let end = self.scratch_entries.len();
            self.groups.push(start..end);
        }
        let needed = DRAW_STRIDE * self.scratch_entries.len().max(1) as u64;
        if needed > self.draw_capacity {
            let capacity = needed.next_power_of_two();
            (self.draw_buffer, self.draw_group) =
                draw_storage(&gpu.device, &self.draw_layout, capacity);
            self.draw_capacity = capacity;
        }
        // Zeroed, as the padding between draw uniforms must be.
        self.scratch_bytes.clear();
        self.scratch_bytes.resize(needed as usize, 0);
        self.queue.clear();
        for (index, (uniform, draw)) in self.scratch_entries.iter().enumerate() {
            let offset = index as u64 * DRAW_STRIDE;
            self.scratch_bytes
                [offset as usize..offset as usize + std::mem::size_of::<DrawUniform>()]
                .copy_from_slice(bytemuck::bytes_of(uniform));
            let mut draw = *draw;
            self.pipelines.request(draw.key);
            draw.offset = offset as u32;
            self.queue.push(draw);
        }
        self.pipelines.collect();
        if self.stream_pipelines {
            self.defer_unready_instances();
        } else {
            self.pipelines.wait_all();
        }
        gpu.queue
            .write_buffer(&self.draw_buffer, 0, &self.scratch_bytes);
        let needed = BONE_BYTES * self.scratch_bones.len().max(1) as u64;
        if needed > self.bone_capacity {
            let capacity = needed.next_power_of_two();
            (self.bone_buffer, self.bone_group) =
                bone_storage(&gpu.device, &self.bone_layout, capacity);
            self.bone_capacity = capacity;
        }
        if !self.scratch_bones.is_empty() {
            gpu.queue.write_buffer(
                &self.bone_buffer,
                0,
                bytemuck::cast_slice(&self.scratch_bones),
            );
        }
    }

    /// Leaves out every instance of the frame with a draw whose pipeline is not compiled yet, so
    /// a model appears whole once its pipelines are in (a frame or two after its upload).
    fn defer_unready_instances(&mut self) {
        let mut waiting = HashSet::new();
        for (group, range) in self.groups.iter().enumerate() {
            for draw in &self.queue[range.clone()] {
                if !self.pipelines.is_ready(&draw.key) {
                    waiting.insert((group, draw.instance));
                }
            }
        }
        if waiting.is_empty() {
            return;
        }
        for (group, range) in self.groups.iter().enumerate() {
            for draw in &mut self.queue[range.clone()] {
                draw.deferred = waiting.contains(&(group, draw.instance));
            }
        }
    }

    /// Leave out (instead of waiting for) instances whose pipelines are still compiling: the
    /// running client, which must not stall a frame on a driver compile.
    pub fn set_stream_pipelines(&mut self, stream: bool) {
        self.stream_pipelines = stream;
    }

    /// Whether pipelines are still compiling (a headless render waits for this).
    pub fn pipelines_busy(&self) -> bool {
        self.pipelines.busy()
    }

    fn write_frame(
        &self,
        queue: &wgpu::Queue,
        slot: usize,
        camera: &Camera,
        environment: &ObjectEnvironment,
        aspect: f32,
    ) {
        let lights = environment.lights;
        let (fog_color, (start, end), fog_on) = match environment.fog {
            Some((color, range)) => (color, range, 1.0),
            None => ([1.0; 3], (0.0, 1.0), 0.0),
        };
        let rgb = |c: [f32; 3]| [c[0], c[1], c[2], 1.0];
        let forward = (camera.target - camera.eye).normalize_or_zero();
        let (inv_dir, diffuse, specular) = match lights.sun {
            Some(sun) => (
                (-sun.direction).extend(1.0).to_array(),
                rgb(sun.diffuse),
                rgb(sun.specular),
            ),
            None => ([0.0; 4], [0.0; 4], [0.0; 4]),
        };
        let uniform = FrameUniform {
            view_proj: camera.view_projection(aspect).to_cols_array_2d(),
            view: ac2_math::camera::look_at_mat4(camera.eye, camera.target, Vec3::Z)
                .to_cols_array_2d(),
            eye: camera.eye.extend(1.0).to_array(),
            to_viewer: (-forward).extend(0.0).to_array(),
            sun_inv_dir: inv_dir,
            sun_diffuse: diffuse,
            sun_specular: specular,
            ambient: rgb(lights.ambient),
            fog_color: rgb(fog_color),
            fog: [start, end, 1.0 / (end - start).max(1e-3), fog_on],
            shading: FRAME_SHADING,
            shadow: self.shadow_matrix.to_cols_array_2d(),
        };
        queue.write_buffer(
            &self.frame_buffer,
            slot as u64 * FRAME_STRIDE,
            bytemuck::bytes_of(&uniform),
        );
    }

    /// Records group `group` of the draws laid out by the last `prepare`, which must have been
    /// given the same `instances` for it.
    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        group: usize,
        instances: &[ObjectInstance<'_>],
    ) {
        let Some(range) = self.groups.get(group) else {
            return;
        };
        pass.set_bind_group(
            0,
            &self.frame_group,
            &[(group as u64 * FRAME_STRIDE) as u32],
        );
        pass.set_bind_group(3, &self.bone_group, &[]);
        let mut current: Option<PipelineKey> = None;
        for draw in &self.queue[range.clone()] {
            let Some(part) = instances
                .get(draw.instance)
                .and_then(|instance| instance.model.parts.get(draw.part))
            else {
                continue;
            };
            if draw.deferred {
                continue;
            }
            if current != Some(draw.key) {
                pass.set_pipeline(self.pipelines.pipeline(&draw.key));
                current = Some(draw.key);
            }
            let batch = &part.batches[draw.batch];
            pass.set_bind_group(1, &self.draw_group, &[draw.offset]);
            pass.set_bind_group(2, &batch.passes[draw.layer].textures, &[]);
            pass.set_vertex_buffer(0, part.vertices.slice(..));
            pass.set_index_buffer(part.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(batch.range.clone(), 0, 0..1);
        }
    }
}

/// The object shader with a `shade` function appended.
fn frame_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    shadow_map: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 object frame"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<FrameUniform>() as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(shadow_map),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

pub(crate) fn object_module(device: &wgpu::Device, shade: &str) -> wgpu::ShaderModule {
    let source = format!(
        "{}\n{}\n{}\n{}",
        include_str!("shading.wgsl"),
        include_str!("object.wgsl"),
        include_str!("object_modern.wgsl"),
        shade
    );
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ac2 object"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

impl ObjectRenderer {
    /// Translates and compiles `program`, returning the device's validation error if any.
    pub fn validate_program(
        gpu: &Gpu,
        program: &ac2_render::object::PixelProgram,
    ) -> Option<String> {
        let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _module = object_module(&gpu.device, &shade_function(program));
        pollster::block_on(scope.pop()).map(|error| error.to_string())
    }
}
