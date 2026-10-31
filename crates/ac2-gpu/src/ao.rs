//! Ground-truth ambient occlusion for the world (`ao.wgsl`), part of the post chain (`post.rs`):
//! GTAO (Jimenez et al. 2016) at half size from the scene's depth alone, a depth-aware blur and,
//! while the temporal anti-aliasing runs, an accumulation with the previous frames reprojected by
//! its camera motion ([`crate::taa::Reprojection`]). The tone map multiplies the result in.
//! Retail had none; it is part of the one modern shading pass (`AGENTS.md`). Without a camera
//! (`ScenePresenter::set_camera` not called) or at quality 0 the occlusion is 1 everywhere and
//! no pass is recorded.

use std::cell::Cell;

use bytemuck::{Pod, Zeroable};

use crate::taa::Reprojection;
use crate::{Gpu, HdrTarget};

/// The occlusion's target format.
pub(crate) const AO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
/// The view-space depth the accumulation keeps beside the occlusion.
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
/// How far around a point occluders count, in metres: a character's width, the gap under a
/// table, a doorway's reveal.
const RADIUS: f32 = 1.2;
/// How much of the occlusion darkens the scene.
const STRENGTH: f32 = 0.6;
/// The blend towards the current frame while accumulating.
const BLEND: f32 = 0.1;
/// The relative depth difference beyond which the history is rejected.
const DEPTH_TOLERANCE: f32 = 0.04;

/// The slices per pixel and horizon steps per side of an ambient occlusion tier
/// (`SceneOptions::ao_quality`: 0 off, 1 low, 2 and above high), `None` when off.
pub(crate) fn tier(ao_quality: u8) -> Option<(u32, u32)> {
    match ao_quality {
        0 => None,
        1 => Some((2, 4)),
        _ => Some((3, 6)),
    }
}

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
    state: [f32; 4],
    inv_view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    eye_delta: [f32; 4],
}

pub(crate) struct AoPipelines {
    occlusion: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    accumulate: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl AoPipelines {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let texture = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 ao"),
            entries: &[
                texture(0, wgpu::TextureSampleType::Depth),
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
                texture(2, wgpu::TextureSampleType::Float { filterable: true }),
                texture(3, wgpu::TextureSampleType::Float { filterable: true }),
                texture(4, wgpu::TextureSampleType::Float { filterable: false }),
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
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let pipeline = |entry_point, targets: &[Option<wgpu::ColorTargetState>]| {
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
                    targets,
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: gpu.pipeline_cache(),
            })
        };
        Self {
            occlusion: pipeline("fs_ao", &[target(AO_FORMAT)]),
            blur: pipeline("fs_blur", &[target(AO_FORMAT)]),
            accumulate: pipeline("fs_accumulate", &[target(AO_FORMAT), target(DEPTH_FORMAT)]),
            layout,
        }
    }
}

/// The occlusion's targets for one scene size.
pub(crate) struct AoTargets {
    raw: wgpu::TextureView,
    blurred: wgpu::TextureView,
    /// The occlusion the tone map reads: the blur's result, or the accumulation of it.
    pub(crate) output: wgpu::TextureView,
    output_texture: wgpu::Texture,
    /// The previous frame's occlusion and view-space depth, copied from `output` and `depth`.
    history_texture: wgpu::Texture,
    depth_texture: wgpu::Texture,
    depth_history_texture: wgpu::Texture,
    depth: wgpu::TextureView,
    occlusion_inputs: wgpu::BindGroup,
    blur_inputs: wgpu::BindGroup,
    accumulate_inputs: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    aspect: f32,
    size: wgpu::Extent3d,
    /// `output` is cleared to white (no pass has drawn into it since).
    white: Cell<bool>,
    /// The history holds the previous frame's accumulated occlusion.
    history_ready: Cell<bool>,
}

impl AoTargets {
    pub(crate) fn new(gpu: &Gpu, pipes: &AoPipelines, hdr: &HdrTarget) -> Self {
        let device = &gpu.device;
        let size = wgpu::Extent3d {
            width: (hdr.width / 2).max(1),
            height: (hdr.height / 2).max(1),
            depth_or_array_layers: 1,
        };
        let texture = |label, format, copy: wgpu::TextureUsages| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | copy,
                view_formats: &[],
            })
        };
        let view = |texture: &wgpu::Texture| texture.create_view(&Default::default());
        let none = wgpu::TextureUsages::empty();
        let raw = view(&texture("ac2 ao raw", AO_FORMAT, none));
        let blurred = view(&texture("ac2 ao blurred", AO_FORMAT, none));
        let output_texture = texture("ac2 ao", AO_FORMAT, wgpu::TextureUsages::COPY_SRC);
        let history_texture = texture("ac2 ao history", AO_FORMAT, wgpu::TextureUsages::COPY_DST);
        let depth_texture = texture("ac2 ao depth", DEPTH_FORMAT, wgpu::TextureUsages::COPY_SRC);
        let depth_history_texture = texture(
            "ac2 ao depth history",
            DEPTH_FORMAT,
            wgpu::TextureUsages::COPY_DST,
        );
        let (output, history, depth, depth_history) = (
            view(&output_texture),
            view(&history_texture),
            view(&depth_texture),
            view(&depth_history_texture),
        );
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 ao"),
            size: std::mem::size_of::<AoUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Every pass binds the same set; the textures it draws into are never the ones it reads.
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
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&history),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&depth_history),
                    },
                ],
            })
        };
        Self {
            occlusion_inputs: inputs(&blurred),
            blur_inputs: inputs(&raw),
            accumulate_inputs: inputs(&blurred),
            raw,
            blurred,
            output,
            output_texture,
            history_texture,
            depth_texture,
            depth_history_texture,
            depth,
            uniform,
            aspect: hdr.width as f32 / hdr.height.max(1) as f32,
            size,
            white: Cell::new(false),
            history_ready: Cell::new(false),
        }
    }

    /// One full-screen pass of `pipeline` over `inputs` into `targets`.
    fn pass(
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        inputs: &wgpu::BindGroup,
        targets: &[&wgpu::TextureView],
    ) {
        let attachments: Vec<_> = targets
            .iter()
            .map(|view| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })
            })
            .collect();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ac2 ao"),
            color_attachments: &attachments,
            depth_stencil_attachment: None,
            timestamp_writes: gpu.pass_timestamps("ao"),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, inputs, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Records the occlusion of tier `quality` (`tier`) for `camera`, and while `motion` is known
    /// (the temporal anti-aliasing runs) its accumulation with the previous frames. With no
    /// camera or at quality 0 nothing is recorded: the occlusion is cleared to 1 once.
    pub(crate) fn record(
        &self,
        gpu: &Gpu,
        pipes: &AoPipelines,
        encoder: &mut wgpu::CommandEncoder,
        camera: Option<AoCamera>,
        quality: u8,
        motion: Option<Reprojection>,
    ) {
        let (Some(camera), Some((slices, steps))) = (camera, tier(quality)) else {
            self.history_ready.set(false);
            if !self.white.replace(true) {
                drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ac2 ao off"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.output,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                }));
            }
            return;
        };
        self.white.set(false);
        let tan = f64::from(camera.fov_y * 0.5).tan();
        let reading = motion.is_some_and(|m| m.has_history) && self.history_ready.get();
        let mut uniform = AoUniform {
            projection: [
                (tan * f64::from(self.aspect)) as f32,
                tan as f32,
                camera.near,
                camera.far,
            ],
            sampling: [RADIUS, STRENGTH, slices as f32, steps as f32],
            state: [
                motion.map_or(0.0, |m| (m.frame % 64) as f32),
                f32::from(u8::from(reading)),
                DEPTH_TOLERANCE,
                BLEND,
            ],
            inv_view_proj: [[0.0; 4]; 4],
            prev_view_proj: [[0.0; 4]; 4],
            eye_delta: [0.0; 4],
        };
        if let Some(m) = motion {
            uniform.inv_view_proj = m.inv_view_proj.to_cols_array_2d();
            uniform.prev_view_proj = m.prev_view_proj.to_cols_array_2d();
            uniform.eye_delta = [m.eye_delta[0], m.eye_delta[1], m.eye_delta[2], 0.0];
        }
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniform));
        Self::pass(
            gpu,
            encoder,
            &pipes.occlusion,
            &self.occlusion_inputs,
            &[&self.raw],
        );
        if motion.is_none() {
            self.history_ready.set(false);
            Self::pass(
                gpu,
                encoder,
                &pipes.blur,
                &self.blur_inputs,
                &[&self.output],
            );
            return;
        }
        Self::pass(
            gpu,
            encoder,
            &pipes.blur,
            &self.blur_inputs,
            &[&self.blurred],
        );
        Self::pass(
            gpu,
            encoder,
            &pipes.accumulate,
            &self.accumulate_inputs,
            &[&self.output, &self.depth],
        );
        for (from, to) in [
            (&self.output_texture, &self.history_texture),
            (&self.depth_texture, &self.depth_history_texture),
        ] {
            encoder.copy_texture_to_texture(from.as_image_copy(), to.as_image_copy(), self.size);
        }
        self.history_ready.set(true);
    }
}

#[cfg(test)]
#[path = "ao_tests.rs"]
mod tests;
