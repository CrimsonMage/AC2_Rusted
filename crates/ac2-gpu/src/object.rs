//! The static-object pass: one pipeline per blend/depth/cull state (`SetSourceBlend`,
//! `SetDestBlend`, `SetDepthWrite`, `SetDepthTest`, `D3DRS_CULLMODE`), one draw per material layer
//! (`RenderIndexedUsingMaterial` draws each drawn layer in turn), with that layer's material,
//! program and texture stages in a dynamic-offset uniform.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use ac2_math::{Mat4, Vec3};
use ac2_render::object::{LayerPass, ObjectLights, ObjectMaterial, ObjectModel, VertexProgram};
use ac2_render::{Camera, Image, SceneLighting};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::object_state::{pipeline_key, sampler_key, PipelineKey, SamplerKey, DRAW_STRIDE};
use crate::pixel_program::shade_function;
use crate::texture::{TextureCache, TextureKey};
use crate::Gpu;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    uvs: [[f32; 2]; 4],
    tangents: [[f32; 3]; 2],
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
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct DrawUniform {
    model: [[f32; 4]; 4],
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    texture_factor: [f32; 4],
    program: [u32; 4],
    color_ops: [[u32; 4]; 4],
    alpha_ops: [[u32; 4]; 4],
    matrices: [[f32; 4]; 8],
}

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
}

/// An uploaded `ObjectModel`.
pub struct GpuObjectModel {
    parts: Vec<GpuPart>,
    /// Model-space centre of the vertices, for back-to-front ordering of translucent layers.
    center: Vec3,
}

/// One draw of the frame: pipeline, uniform offset, and instance/part/batch/layer indices.
#[derive(Clone, Copy)]
struct QueuedDraw {
    key: PipelineKey,
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
}

impl<'a> ObjectInstance<'a> {
    /// An untinted instance.
    pub fn new(model: &'a GpuObjectModel, world: Mat4) -> Self {
        Self {
            model,
            world,
            tint: [1.0; 4],
        }
    }
}

/// Instances laid out together; `ordered` keeps their order instead of sorting translucency.
#[derive(Clone, Copy)]
pub struct DrawGroup<'a> {
    pub instances: &'a [ObjectInstance<'a>],
    pub ordered: bool,
}

pub struct ObjectRenderer {
    format: wgpu::TextureFormat,
    /// Shader modules by program index: 0 is the stage cascade, then one per distinct
    /// translated pixel shader.
    modules: Vec<wgpu::ShaderModule>,
    program_ids: HashMap<String, u32>,
    layout: wgpu::PipelineLayout,
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    frame_buffer: wgpu::Buffer,
    frame_group: wgpu::BindGroup,
    draw_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    draw_buffer: wgpu::Buffer,
    draw_group: wgpu::BindGroup,
    draw_capacity: u64,
    samplers: HashMap<SamplerKey, wgpu::Sampler>,
    pub textures: TextureCache,
    time: f64,
    /// Laid out by `prepare`, recorded by `draw`.
    queue: Vec<QueuedDraw>,
    /// Ranges of `queue` by draw group.
    groups: Vec<Range<usize>>,
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
            entries: &[uniform(0, false)],
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
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 object frame uniform"),
            size: std::mem::size_of::<FrameUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let frame_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 object frame"),
            layout: &frame_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });
        let module = object_module(device, include_str!("object_stages.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 object"),
            bind_group_layouts: &[&frame_layout, &draw_layout, &texture_layout],
            push_constant_ranges: &[],
        });
        let (draw_buffer, draw_group) = draw_storage(device, &draw_layout, DRAW_STRIDE * 64);
        Self {
            format,
            modules: vec![module],
            program_ids: HashMap::new(),
            layout,
            pipelines: HashMap::new(),
            frame_buffer,
            frame_group,
            draw_layout,
            texture_layout,
            draw_buffer,
            draw_group,
            draw_capacity: DRAW_STRIDE * 64,
            samplers: HashMap::new(),
            textures: TextureCache::new(device, &gpu.queue),
            time: 0.0,
            queue: Vec::new(),
            groups: Vec::new(),
        }
    }

    /// Uploads a model; `load` supplies images for stage textures not yet on the GPU.
    pub fn upload(
        &mut self,
        gpu: &Gpu,
        model: &ObjectModel,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> GpuObjectModel {
        let device = &gpu.device;
        let mut parts = Vec::with_capacity(model.parts.len());
        let (mut sum, mut count) = (Vec3::ZERO, 0.0_f32);
        for part in &model.parts {
            let vertices: Vec<GpuVertex> = part
                .vertices
                .iter()
                .map(|v| GpuVertex {
                    position: v.position,
                    normal: v.normal,
                    color: v.color,
                    uvs: v.uvs,
                    tangents: v.tangents,
                })
                .collect();
            for v in part.vertices.iter() {
                sum += part
                    .transform
                    .transform_point3(Vec3::from_array(v.position));
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
                        program: self.program_id(gpu, &pass),
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
            });
        }
        GpuObjectModel {
            parts,
            center: if count > 0.0 { sum / count } else { Vec3::ZERO },
        }
    }

    /// The module index for a pass's pixel shader, translating and compiling it on first use.
    fn program_id(&mut self, gpu: &Gpu, pass: &LayerPass) -> u32 {
        let Some(program) = &pass.pixel_program else {
            return 0;
        };
        let source = shade_function(program);
        if let Some(id) = self.program_ids.get(&source) {
            return *id;
        }
        let id = self.modules.len() as u32;
        self.modules.push(object_module(&gpu.device, &source));
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
            // An untextured stage reads `D3DTA_TEXTURE` as white.
            let view = match stage.and_then(|s| s.texture) {
                Some(did) => {
                    self.textures
                        .get(&gpu.device, &gpu.queue, TextureKey::Texture(did), load)
                }
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
        lighting: &SceneLighting,
        aspect: f32,
        time: f64,
        groups: &[DrawGroup<'_>],
    ) {
        self.time = time;
        self.write_frame(&gpu.queue, camera, lighting, aspect);
        let mut entries = Vec::new();
        let mut bounds = Vec::with_capacity(groups.len());
        for group in groups {
            let start = entries.len();
            let mut opaque = Vec::new();
            let mut translucent = Vec::new();
            for (instance_index, instance) in group.instances.iter().enumerate() {
                let center = instance.world.transform_point3(instance.model.center);
                let distance = center.distance_squared(camera.eye);
                for (part_index, part) in instance.model.parts.iter().enumerate() {
                    let model = instance.world * part.transform;
                    for (batch_index, batch) in part.batches.iter().enumerate() {
                        let recomputed = (batch.material.animated() || instance.tint != [1.0; 4])
                            .then(|| batch.material.passes(instance.tint, time));
                        for (pass_index, gpu_pass) in batch.passes.iter().enumerate() {
                            let pass = recomputed
                                .as_ref()
                                .map_or(&gpu_pass.pass, |p| &p[pass_index]);
                            let draw = QueuedDraw {
                                key: pipeline_key(pass, gpu_pass.program),
                                offset: 0,
                                instance: instance_index,
                                part: part_index,
                                batch: batch_index,
                                layer: pass_index,
                            };
                            let entry = (draw_uniform(model, pass), draw);
                            if group.ordered || pass.depth_write {
                                opaque.push(entry);
                            } else {
                                translucent.push((distance, entry));
                            }
                        }
                    }
                }
            }
            translucent.sort_by(|a, b| b.0.total_cmp(&a.0));
            entries.extend(opaque);
            entries.extend(translucent.into_iter().map(|(_, entry)| entry));
            bounds.push(start..entries.len());
        }
        let needed = DRAW_STRIDE * entries.len().max(1) as u64;
        if needed > self.draw_capacity {
            let capacity = needed.next_power_of_two();
            (self.draw_buffer, self.draw_group) =
                draw_storage(&gpu.device, &self.draw_layout, capacity);
            self.draw_capacity = capacity;
        }
        let mut bytes = vec![0_u8; needed as usize];
        self.queue.clear();
        for (index, (uniform, mut draw)) in entries.into_iter().enumerate() {
            let offset = index as u64 * DRAW_STRIDE;
            bytes[offset as usize..offset as usize + std::mem::size_of::<DrawUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&uniform));
            let key = draw.key;
            if !self.pipelines.contains_key(&key) {
                let module = &self.modules[key.program as usize];
                let pipeline = key.create(&gpu.device, &self.layout, module, self.format);
                self.pipelines.insert(key, pipeline);
            }
            draw.offset = offset as u32;
            self.queue.push(draw);
        }
        self.groups = bounds;
        gpu.queue.write_buffer(&self.draw_buffer, 0, &bytes);
    }

    fn write_frame(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        lighting: &SceneLighting,
        aspect: f32,
    ) {
        let lights = ObjectLights::outdoor(lighting);
        let (start, end) = lighting.fog_range;
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
            fog_color: rgb(lighting.fog_color),
            fog: [start, end, 1.0 / (end - start).max(1e-3), 0.0],
        };
        queue.write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&uniform));
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
        pass.set_bind_group(0, &self.frame_group, &[]);
        let mut current: Option<PipelineKey> = None;
        for draw in &self.queue[range.clone()] {
            let Some(part) = instances
                .get(draw.instance)
                .and_then(|instance| instance.model.parts.get(draw.part))
            else {
                continue;
            };
            if current != Some(draw.key) {
                pass.set_pipeline(&self.pipelines[&draw.key]);
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
fn object_module(device: &wgpu::Device, shade: &str) -> wgpu::ShaderModule {
    let source = format!("{}\n{}", include_str!("object.wgsl"), shade);
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ac2 object"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

fn draw_storage(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    size: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 object draws"),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 object draws"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<DrawUniform>() as u64),
            }),
        }],
    });
    (buffer, group)
}

fn draw_uniform(model: Mat4, pass: &LayerPass) -> DrawUniform {
    let program = match pass.program {
        VertexProgram::Unlit => 0,
        VertexProgram::Lit => 1,
        VertexProgram::PixelLit => 2,
        VertexProgram::LitVertexDiffuse => 3,
        VertexProgram::PixelLitMultiple => 4,
    };
    let f = &pass.flags;
    let bits = [
        f.material_diffuse,
        f.vertex_diffuse,
        f.modulate_vertex_diffuse,
        f.specular,
        pass.fog,
        f.diffuse_fog_amount,
        f.alpha_fog,
        f.ignore_directional,
        pass.specular_enable,
        f.depth_fog,
        f.pixel_specular,
        f.unmodulated_color1,
        f.dynamic,
    ]
    .iter()
    .enumerate()
    .fold(0_u32, |acc, (bit, on)| acc | (u32::from(*on) << bit));
    let mut uniform = DrawUniform {
        model: model.to_cols_array_2d(),
        ambient: pass.ambient,
        diffuse: pass.diffuse,
        specular: pass.specular,
        texture_factor: pass.texture_factor,
        program: [
            program,
            bits,
            pass.alpha_test.unwrap_or(256).min(256),
            pass.power.to_bits(),
        ],
        color_ops: [[1, 2, 1, 1]; 4],
        alpha_ops: [[2, 1, 0xFFFF_FFFF, 0]; 4],
        matrices: [[0.0; 4]; 8],
    };
    for (index, stage) in pass.stages.iter().enumerate().take(4) {
        uniform.color_ops[index] = [
            stage.color_op,
            stage.color_arg1,
            stage.color_arg2,
            stage.alpha_op,
        ];
        uniform.alpha_ops[index] = [
            stage.alpha_arg1,
            stage.alpha_arg2,
            stage.tex_coord,
            u32::from(stage.texture_matrix.is_some()),
        ];
        if let Some([row0, row1]) = stage.texture_matrix {
            uniform.matrices[index * 2] = row0;
            uniform.matrices[index * 2 + 1] = row1;
        }
    }
    uniform
}

impl ObjectRenderer {
    /// Translates and compiles `program`, returning the device's validation error if any.
    pub fn validate_program(
        gpu: &Gpu,
        program: &ac2_render::object::PixelProgram,
    ) -> Option<String> {
        gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _module = object_module(&gpu.device, &shade_function(program));
        pollster::block_on(gpu.device.pop_error_scope()).map(|error| error.to_string())
    }
}
