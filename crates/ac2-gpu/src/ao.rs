//! Screen-space ambient occlusion for the world (`ao.wgsl`), part of the post chain (`post.rs`):
//! half size, from the scene's depth alone, then a depth-aware blur the tone map multiplies in.
//! Retail had none; it is part of the one modern shading pass (`AGENTS.md`). Without a camera
//! (`ScenePresenter::set_camera` not called) the occlusion is 1 everywhere.

use bytemuck::{Pod, Zeroable};

use crate::{Gpu, HdrTarget};

/// The occlusion's target format.
pub(crate) const AO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
/// How far around a point occluders count, in metres: a character's width, the gap under a
/// table, a doorway's reveal.
const RADIUS: f32 = 1.2;
/// Depth difference ignored as the surface itself, in metres.
const BIAS: f32 = 0.03;
/// Hemisphere samples per half-size pixel.
const SAMPLES: f32 = 12.0;
/// How much of the occlusion darkens the scene.
const STRENGTH: f32 = 0.6;

/// The projection the occlusion reconstructs view-space points with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AoCamera {
    /// Vertical field of view in radians.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct AoUniform {
    projection: [f32; 4],
    sampling: [f32; 4],
    look: [f32; 4],
}

pub(crate) struct AoPipelines {
    occlusion: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl AoPipelines {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 ao"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 ao"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ao.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 ao"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry_point| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ac2 ao"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: AO_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: gpu.pipeline_cache(),
            })
        };
        Self {
            occlusion: pipeline("fs_ao"),
            blur: pipeline("fs_blur"),
            layout,
        }
    }
}

/// The occlusion's targets for one scene size.
pub(crate) struct AoTargets {
    raw: wgpu::TextureView,
    /// The blurred occlusion the tone map reads.
    pub(crate) blurred: wgpu::TextureView,
    /// The occlusion pass's inputs (its `raw` binding is the blurred target, unread there).
    occlusion_inputs: wgpu::BindGroup,
    blur_inputs: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    aspect: f32,
}

impl AoTargets {
    pub(crate) fn new(gpu: &Gpu, pipes: &AoPipelines, hdr: &HdrTarget) -> Self {
        let device = &gpu.device;
        let target = |label| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: (hdr.width / 2).max(1),
                        height: (hdr.height / 2).max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: AO_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let raw = target("ac2 ao raw");
        let blurred = target("ac2 ao");
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 ao"),
            size: std::mem::size_of::<AoUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let inputs = |source: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ac2 ao"),
                layout: &pipes.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&hdr.depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                ],
            })
        };
        Self {
            occlusion_inputs: inputs(&blurred),
            blur_inputs: inputs(&raw),
            raw,
            blurred,
            uniform,
            aspect: hdr.width as f32 / hdr.height.max(1) as f32,
        }
    }

    /// Records the occlusion and its blur for `camera` (none: no occlusion).
    pub(crate) fn record(
        &self,
        gpu: &Gpu,
        pipes: &AoPipelines,
        encoder: &mut wgpu::CommandEncoder,
        camera: Option<AoCamera>,
    ) {
        let uniform = match camera {
            Some(camera) => {
                let tan = f64::from(camera.fov_y * 0.5).tan();
                AoUniform {
                    projection: [
                        (tan * f64::from(self.aspect)) as f32,
                        tan as f32,
                        camera.near,
                        camera.far,
                    ],
                    sampling: [RADIUS, BIAS, SAMPLES, 1.0],
                    look: [STRENGTH, 0.0, 0.0, 0.0],
                }
            }
            None => AoUniform {
                projection: [1.0, 1.0, 0.1, 1.0],
                sampling: [RADIUS, BIAS, SAMPLES, 0.0],
                look: [0.0; 4],
            },
        };
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniform));
        for (pipeline, target, inputs) in [
            (&pipes.occlusion, &self.raw, &self.occlusion_inputs),
            (&pipes.blur, &self.blurred, &self.blur_inputs),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 ao"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: gpu.pass_timestamps("ao"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, inputs, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
