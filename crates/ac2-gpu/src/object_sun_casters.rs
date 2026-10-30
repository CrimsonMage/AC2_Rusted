//! Bodies and objects drawn into the sun shadow cascades (`crate::sun_shadow`): depth only, each
//! part at its posed transform with its vertices blended by the pose's deform matrices as the
//! object pass blends them, so a body's shadow moves with it. Layers that test or blend their
//! alpha cut their texture's transparent texels (leaves, fences) as the landscape map's casters
//! do; layers that neither write depth nor blend by alpha (glows, additive effects) cast nothing.

use ac2_math::{Mat4, Vec3};
use ac2_render::object::PartDegrades;
use bytemuck::{Pod, Zeroable};

use super::{GpuVertex, ObjectInstance, ObjectRenderer, BONE_BYTES};
use crate::sun_shadow::{Cascade, SUN_SHADOW_FORMAT};
use crate::Gpu;

/// Byte stride of one cascade's uniform (the dynamic-offset alignment).
const CASCADE_STRIDE: u64 = 256;
/// `D3DBLEND_SRCALPHA`.
const BLEND_SRC_ALPHA: u32 = 5;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CasterInstance {
    model: [[f32; 4]; 4],
    /// x the first deform matrix, y 1 when the vertices blend, z the first stage's alpha op.
    skin: [u32; 4],
}

const SHADER: &str = "
struct Instance {
    model: mat4x4<f32>,
    skin: vec4<u32>,
};

@group(0) @binding(0) var<uniform> cascade: mat4x4<f32>;
@group(0) @binding(1) var<storage, read> instances: array<Instance>;
@group(0) @binding(2) var<storage, read> bones: array<mat4x4<f32>>;
@group(1) @binding(0) var stage0: texture_2d<f32>;
@group(1) @binding(4) var sampler0: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) bone_index: vec4<u32>,
    @location(3) weights: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) mode: u32,
};

@vertex
fn vs(input: VsIn, @builtin(instance_index) index: u32) -> VsOut {
    let inst = instances[index];
    var position = input.position;
    if inst.skin.y != 0u {
        position = vec3(0.0);
        for (var k = 0u; k < 4u; k = k + 1u) {
            let weight = input.weights[k];
            if weight != 0.0 {
                let h = bones[inst.skin.x + input.bone_index[k]] * vec4(input.position, 1.0);
                position = position + h.xyz * (weight / h.w);
            }
        }
    }
    var out: VsOut;
    out.clip = cascade * (inst.model * vec4(position, 1.0));
    out.uv = input.uv;
    out.mode = inst.skin.z;
    return out;
}

// The landscape map casters' alpha (`object_shadow.rs`): the first stage's alpha op over the
// texture, cut at one half.
@fragment
fn fs(input: VsOut) {
    let texel = textureSample(stage0, sampler0, input.uv).a;
    var alpha = 1.0;
    switch input.mode {
        case 2u, 4u: { alpha = texel; }
        case 5u: { alpha = min(texel * 2.0, 1.0); }
        case 6u: { alpha = min(texel * 4.0, 1.0); }
        default: {}
    }
    if alpha < 0.5 {
        discard;
    }
}
";

/// One part-batch drawn into one cascade.
struct CasterDraw {
    cascade: usize,
    /// The caster (index into the casters drawn) and its block in the instance storage.
    caster: usize,
    instance: u32,
    part: usize,
    batch: usize,
    cut: bool,
}

/// The cascade pipelines (solid and alpha-cut) and their per-frame storage.
pub(crate) struct SunCasterPass {
    solid: wgpu::RenderPipeline,
    cut: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    cascades: wgpu::Buffer,
    instances: wgpu::Buffer,
    bones: wgpu::Buffer,
    group: wgpu::BindGroup,
    scratch_instances: Vec<CasterInstance>,
    scratch_bones: Vec<[[f32; 4]; 4]>,
    scratch_draws: Vec<CasterDraw>,
}

fn storage(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.next_power_of_two(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl SunCasterPass {
    fn new(gpu: &Gpu, texture_layout: &wgpu::BindGroupLayout) -> Self {
        let device = &gpu.device;
        let buffer_entry = |binding, ty, dynamic| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        };
        let read_only = wgpu::BufferBindingType::Storage { read_only: true };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 sun casters"),
            entries: &[
                buffer_entry(0, wgpu::BufferBindingType::Uniform, true),
                buffer_entry(1, read_only, false),
                buffer_entry(2, read_only, false),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 sun casters"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 sun casters"),
            bind_group_layouts: &[Some(&layout), Some(texture_layout)],
            immediate_size: 0,
        });
        let attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: std::mem::offset_of!(GpuVertex, position) as u64,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: std::mem::offset_of!(GpuVertex, uvs) as u64,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint8x4,
                offset: std::mem::offset_of!(GpuVertex, bones) as u64,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::offset_of!(GpuVertex, weights) as u64,
                shader_location: 3,
            },
        ];
        let pipeline = |cut: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if cut {
                    "ac2 sun casters (alpha cut)"
                } else {
                    "ac2 sun casters"
                }),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GpuVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &attributes,
                    })],
                },
                // A solid caster writes depth alone.
                fragment: cut.then_some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[],
                }),
                // Both faces cast: AC2 meshes are not all closed.
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: SUN_SHADOW_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2,
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let (solid, cut) = (pipeline(false), pipeline(true));
        let cascades = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 sun cascades"),
            size: CASCADE_STRIDE * crate::sun_shadow::MAX_CASCADES as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = storage(device, "ac2 sun caster instances", 64 * 1024);
        let bones = storage(device, "ac2 sun caster bones", 64 * 1024);
        let group = Self::bind(device, &layout, [&cascades, &instances, &bones]);
        Self {
            solid,
            cut,
            layout,
            cascades,
            instances,
            bones,
            group,
            scratch_instances: Vec::new(),
            scratch_bones: Vec::new(),
            scratch_draws: Vec::new(),
        }
    }

    fn bind(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        [cascades, instances, bones]: [&wgpu::Buffer; 3],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 sun casters"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: cascades,
                        offset: 0,
                        size: wgpu::BufferSize::new(64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bones.as_entire_binding(),
                },
            ],
        })
    }

    /// Grows the storage to hold the scratch data, rebinding when a buffer is remade.
    fn upload(&mut self, gpu: &Gpu) {
        let device = &gpu.device;
        let instance_bytes =
            (self.scratch_instances.len() * std::mem::size_of::<CasterInstance>()).max(16) as u64;
        let bone_bytes = (BONE_BYTES as usize * self.scratch_bones.len()).max(64) as u64;
        let mut remade = false;
        if instance_bytes > self.instances.size() {
            self.instances = storage(device, "ac2 sun caster instances", instance_bytes);
            remade = true;
        }
        if bone_bytes > self.bones.size() {
            self.bones = storage(device, "ac2 sun caster bones", bone_bytes);
            remade = true;
        }
        if remade {
            self.group = Self::bind(
                device,
                &self.layout,
                [&self.cascades, &self.instances, &self.bones],
            );
        }
        if !self.scratch_instances.is_empty() {
            gpu.queue.write_buffer(
                &self.instances,
                0,
                bytemuck::cast_slice(&self.scratch_instances),
            );
        }
        if !self.scratch_bones.is_empty() {
            gpu.queue
                .write_buffer(&self.bones, 0, bytemuck::cast_slice(&self.scratch_bones));
        }
    }
}

/// The world-space bounding sphere of an instance: its model's radius, scaled, and twice that for
/// a posed body whose limbs reach past its rest pose (as the view culling takes it).
pub(crate) fn bounds(instance: &ObjectInstance<'_>) -> (Vec3, f32) {
    let scale = [
        instance.world.x_axis,
        instance.world.y_axis,
        instance.world.z_axis,
    ]
    .map(|axis| axis.truncate().length())
    .into_iter()
    .fold(0.0_f32, f32::max);
    let pose = if instance.pose.is_some() { 2.0 } else { 1.0 };
    (
        instance.world.w_axis.truncate(),
        instance.model.bounding_radius() * scale * pose,
    )
}

impl ObjectRenderer {
    /// Draws `casters` into each of `layers` (one per cascade, cleared first) through `cascades`
    /// fitted around `eye`: each part is placed relative to `eye` and drawn through
    /// `Cascade::clip_rel`, as the receivers read it.
    pub(crate) fn draw_sun_casters(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        layers: &[wgpu::TextureView],
        (eye, cascades): (Vec3, &[Cascade]),
        casters: &[ObjectInstance<'_>],
    ) {
        if self.sun_casters.is_none() {
            self.sun_casters = Some(SunCasterPass::new(gpu, &self.texture_layout));
        }
        let Some(pass) = self.sun_casters.as_mut() else {
            return;
        };
        pass.scratch_instances.clear();
        pass.scratch_bones.clear();
        pass.scratch_draws.clear();
        let mut matrices = vec![0_u8; CASCADE_STRIDE as usize * cascades.len()];
        for (index, cascade) in cascades.iter().enumerate() {
            let at = index * CASCADE_STRIDE as usize;
            matrices[at..at + 64]
                .copy_from_slice(bytemuck::bytes_of(&cascade.clip_rel.to_cols_array_2d()));
        }
        if !matrices.is_empty() {
            gpu.queue.write_buffer(&pass.cascades, 0, &matrices);
        }
        for (cascade_index, cascade) in cascades.iter().enumerate() {
            for (instance_index, instance) in casters.iter().enumerate() {
                let (center, radius) = bounds(instance);
                if instance.tint[3] < 0.5 || !cascade.reaches(center, radius) {
                    continue;
                }
                let mut world = instance.world;
                world.w_axis -= eye.extend(0.0);
                let base = pass.scratch_bones.len() as u32;
                if let Some(pose) = instance.pose {
                    pass.scratch_bones
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
                    let drawn = PartDegrades::drawn(
                        part.degrades.as_deref(),
                        &instance.detail,
                        part.batches.len(),
                    );
                    for (batch_index, batch) in part
                        .batches
                        .iter()
                        .enumerate()
                        .take(drawn.end)
                        .skip(drawn.start)
                    {
                        let Some(first) = batch.passes.first() else {
                            continue;
                        };
                        let layer = &first.pass;
                        let cut =
                            layer.alpha_test.is_some() || layer.source_blend == BLEND_SRC_ALPHA;
                        if !cut && !layer.depth_write {
                            continue;
                        }
                        let alpha_op = layer.stages.first().map_or(1, |s| s.alpha_op);
                        pass.scratch_draws.push(CasterDraw {
                            cascade: cascade_index,
                            caster: instance_index,
                            instance: pass.scratch_instances.len() as u32,
                            part: part_index,
                            batch: batch_index,
                            cut,
                        });
                        pass.scratch_instances.push(CasterInstance {
                            model: (world * transform).to_cols_array_2d(),
                            skin: match skin {
                                Some(first) => [base + first, 1, alpha_op, 0],
                                None => [0, 0, alpha_op, 0],
                            },
                        });
                    }
                }
            }
        }
        pass.upload(gpu);
        let mut draws = pass.scratch_draws.iter().peekable();
        for (cascade_index, target) in layers.iter().enumerate().take(cascades.len()) {
            let mut render = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 sun shadow cascade"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: target,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: gpu.pass_timestamps("sun shadow"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let offset = (cascade_index as u64 * CASCADE_STRIDE) as u32;
            let mut bound: Option<bool> = None;
            while let Some(draw) = draws.next_if(|draw| draw.cascade == cascade_index) {
                if bound != Some(draw.cut) {
                    render.set_pipeline(if draw.cut { &pass.cut } else { &pass.solid });
                    render.set_bind_group(0, &pass.group, &[offset]);
                    bound = Some(draw.cut);
                }
                let part = &casters[draw.caster].model.parts[draw.part];
                let batch = &part.batches[draw.batch];
                render.set_bind_group(1, &batch.passes[0].textures, &[]);
                render.set_vertex_buffer(0, part.vertices.slice(..));
                render.set_index_buffer(part.indices.slice(..), wgpu::IndexFormat::Uint32);
                render.draw_indexed(batch.range.clone(), 0, draw.instance..draw.instance + 1);
            }
            drop(render);
        }
    }
}
