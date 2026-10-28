//! Static objects drawn into the landscape's shadow map (`CBaseMesh::Draw` while
//! `SceneTool::m_bRenderingToShadowMap` is set): every batch takes
//! `SceneTool::m_pShadowMapObjectMaterial`, colour black (`SELECTARG2` of the draw colour) and
//! alpha by the batch's own first-stage alpha op over its texture, blended SRCALPHA/INVSRCALPHA
//! with depth off, so each caster multiplies the white map by one minus its coverage.

use ac2_math::Mat4;
use bytemuck::{Pod, Zeroable};

use super::{GpuVertex, ObjectInstance, ObjectRenderer};
use crate::Gpu;

/// Byte stride of one caster draw's uniform (the dynamic-offset alignment).
const CASTER_STRIDE: u64 = 256;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CasterUniform {
    /// Shadow-map clip transform times the instance's and the part's.
    clip: [[f32; 4]; 4],
    /// x the first stage's alpha op.
    mode: [u32; 4],
}

const SHADER: &str = "
struct Caster {
    clip: mat4x4<f32>,
    mode: vec4<u32>,
};

@group(0) @binding(0) var<uniform> caster: Caster;
@group(1) @binding(0) var stage0: texture_2d<f32>;
@group(1) @binding(4) var sampler0: sampler;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VsOut {
    var out: VsOut;
    out.clip = caster.clip * vec4(position, 1.0);
    out.uv = uv;
    return out;
}

// The material's alphaArg1 is the texture and alphaArg2 the opaque draw colour.
@fragment
fn fs(input: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(stage0, sampler0, input.uv).a;
    var alpha = 1.0;
    switch caster.mode.x {
        case 2u, 4u: { alpha = texel; }
        case 5u: { alpha = min(texel * 2.0, 1.0); }
        case 6u: { alpha = min(texel * 4.0, 1.0); }
        default: {}
    }
    return vec4(0.0, 0.0, 0.0, alpha);
}
";

/// The caster pipeline and its per-draw uniforms.
pub(crate) struct CasterPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    group: wgpu::BindGroup,
    capacity: u64,
}

impl CasterPass {
    fn new(gpu: &Gpu, texture_layout: &wgpu::BindGroupLayout, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 shadow caster"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 shadow caster"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 shadow caster"),
            bind_group_layouts: &[Some(&layout), Some(texture_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 shadow caster"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GpuVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
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
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let (buffer, group) = caster_storage(device, &layout, CASTER_STRIDE * 256);
        Self {
            pipeline,
            layout,
            buffer,
            group,
            capacity: CASTER_STRIDE * 256,
        }
    }
}

fn caster_storage(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    size: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 shadow casters"),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 shadow casters"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<CasterUniform>() as u64),
            }),
        }],
    });
    (buffer, group)
}

impl ObjectRenderer {
    /// Draws `instances` as shadow casters into `pass`, a target of `format`, through `clip`
    /// (world to the map's clip space). Rigid parts only: the scenery `LScapeRender` gathers is
    /// unposed.
    pub(crate) fn draw_shadow_casters(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip: Mat4,
        instances: &[ObjectInstance<'_>],
    ) {
        if self.casters.is_none() {
            self.casters = Some(CasterPass::new(
                gpu,
                &self.texture_layout,
                self.shadow_format,
            ));
        }
        let Some(casters) = self.casters.as_mut() else {
            return;
        };
        let mut uniforms = Vec::new();
        let mut draws = Vec::new();
        for instance in instances.iter().filter(|i| i.pose.is_none()) {
            for (part_index, part) in instance.model.parts.iter().enumerate() {
                if part.weighted {
                    continue;
                }
                let matrix = clip * instance.world * part.transform;
                for (batch_index, batch) in part.batches.iter().enumerate() {
                    let Some(first) = batch.passes.first() else {
                        continue;
                    };
                    let alpha_op = first.pass.stages.first().map_or(1, |s| s.alpha_op);
                    uniforms.push(CasterUniform {
                        clip: matrix.to_cols_array_2d(),
                        mode: [alpha_op, 0, 0, 0],
                    });
                    draws.push((part_index, batch_index, instance.model));
                }
            }
        }
        let needed = CASTER_STRIDE * uniforms.len().max(1) as u64;
        if needed > casters.capacity {
            let capacity = needed.next_power_of_two();
            (casters.buffer, casters.group) =
                caster_storage(&gpu.device, &casters.layout, capacity);
            casters.capacity = capacity;
        }
        let mut bytes = vec![0_u8; (CASTER_STRIDE as usize) * uniforms.len()];
        for (index, uniform) in uniforms.iter().enumerate() {
            let at = index * CASTER_STRIDE as usize;
            bytes[at..at + std::mem::size_of::<CasterUniform>()]
                .copy_from_slice(bytemuck::bytes_of(uniform));
        }
        gpu.queue.write_buffer(&casters.buffer, 0, &bytes);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ac2 shadow casters"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: gpu.pass_timestamps("shadow map"),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&casters.pipeline);
        for (index, (part_index, batch_index, model)) in draws.into_iter().enumerate() {
            let part = &model.parts[part_index];
            let batch = &part.batches[batch_index];
            let offset = (index as u64 * CASTER_STRIDE) as u32;
            pass.set_bind_group(0, &casters.group, &[offset]);
            pass.set_bind_group(1, &batch.passes[0].textures, &[]);
            pass.set_vertex_buffer(0, part.vertices.slice(..));
            pass.set_index_buffer(part.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(batch.range.clone(), 0, 0..1);
        }
    }
}
