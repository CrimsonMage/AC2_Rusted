//! The world's post chain in linear light (`post.wgsl`), run once per frame after the world draws
//! into the [`HdrTarget`] and read by the tone map: a dual-filter bloom and the eye adaptation.
//! Part of the one modern shading pass (`AGENTS.md`): retail had neither. Every pass is a
//! full-screen triangle, so the chain runs on the GL fallback as on Vulkan and Metal.
//!
//! The UI viewports' scenes (the paper doll) and the offline renders keep a fixed exposure and no
//! bloom: they tone map through [`crate::ToneMap`]'s neutral inputs.

use std::cell::Cell;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};

use crate::{Gpu, HdrTarget, HDR_FORMAT};

/// Bloom levels below the half-size first one, at most; fewer when the output is small.
const MAX_LEVELS: usize = 6;
/// A level is made only while both its sides stay at least this many texels.
const MIN_LEVEL_SIDE: u32 = 8;
/// log2 of the centre-weighted mean scene luminance that takes EV 0: the day-lit landscape, so
/// daylight keeps the exposure the DATs were authored at and dusk, night and interiors open up.
/// Measured on the shipped region at noon: a valley walk meters about -3.6 and an open fly-over
/// about -2.5 for the same intended exposure.
const KEY_LOG2: f32 = -3.0;
/// The share of the metered difference the exposure follows: a daylit view stays within about a
/// quarter stop of the authored exposure, while night and interiors, several stops darker, still
/// reach the limit.
const ADAPT_STRENGTH: f32 = 0.5;
/// The exposure's EV limits: at most one stop darker (a bright sky filling the view) or one
/// brighter (night, a dim interior), so the retail day/night contrast stays.
const EV_LIMITS: [f32; 2] = [-1.0, 1.0];
/// Seconds (e-folding) to adapt towards a brighter scene and towards a darker one: the eye closes
/// quickly and opens slowly.
const ADAPT_SECONDS: [f32; 2] = [0.33, 1.0];
/// How much of the blurred scene the tone map mixes in.
const BLOOM_STRENGTH: f32 = 0.05;
/// How much the vignette darkens the corners.
const VIGNETTE: f32 = 0.25;
/// A longer frame gap (a stall, a hidden window) adapts as if this many seconds had passed.
const MAX_STEP_SECONDS: f32 = 0.25;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Adapt {
    step: [f32; 4],
    limits: [f32; 4],
    look: [f32; 4],
}

/// The chain's pipelines and layouts, made once per presenter.
pub(crate) struct PostPipelines {
    down_first: wgpu::RenderPipeline,
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    adapt: wgpu::RenderPipeline,
    source_layout: wgpu::BindGroupLayout,
    adapt_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl PostPipelines {
    pub(crate) fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let source_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 post source"),
            entries: &[
                texture(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture(2),
            ],
        });
        let adapt_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 post adapt"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture(1),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 post"),
            source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
        });
        let chain_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 post chain"),
            bind_group_layouts: &[Some(&source_layout)],
            immediate_size: 0,
        });
        let adapt_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ac2 post adapt"),
                bind_group_layouts: &[Some(&source_layout), Some(&adapt_layout)],
                immediate_size: 0,
            });
        let pipeline = |layout: &wgpu::PipelineLayout, entry_point| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ac2 post"),
                layout: Some(layout),
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
                        format: HDR_FORMAT,
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
            down_first: pipeline(&chain_layout, "fs_down_first"),
            down: pipeline(&chain_layout, "fs_down"),
            up: pipeline(&chain_layout, "fs_up"),
            adapt: pipeline(&adapt_pipeline_layout, "fs_adapt"),
            source_layout,
            adapt_layout,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("ac2 post linear"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
        }
    }
}

/// The bloom levels' sizes for a `width` x `height` scene: half size first, then halving while
/// both sides stay at least `MIN_LEVEL_SIDE`, at most `MAX_LEVELS`; always one level.
fn level_sizes(width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut sizes = Vec::new();
    let (mut w, mut h) = ((width / 2).max(1), (height / 2).max(1));
    while sizes.len() < MAX_LEVELS
        && (sizes.is_empty() || (w >= MIN_LEVEL_SIDE && h >= MIN_LEVEL_SIDE))
    {
        sizes.push((w, h));
        (w, h) = ((w / 2).max(1), (h / 2).max(1));
    }
    sizes
}

/// One pass of the chain: the target it writes and the inputs it reads.
struct Pass {
    target: wgpu::TextureView,
    inputs: wgpu::BindGroup,
}

/// The chain's targets for one output size, and the adapted exposure carried between frames.
pub(crate) struct PostChain {
    width: u32,
    height: u32,
    /// The downsamples, the first from the scene at half size.
    down: Vec<Pass>,
    /// The upsamples, coarsest first; the last writes the bloom the tone map reads.
    up: Vec<Pass>,
    /// The two exposure texels, written in turn (each frame reads the other).
    exposure: [wgpu::TextureView; 2],
    /// The smallest level as the adaptation's source.
    adapt_source: wgpu::BindGroup,
    /// Per written texel: the uniform and the other texel.
    adapt_inputs: [wgpu::BindGroup; 2],
    /// Per written texel: what the tone map binds (bloom, sampler, that texel).
    tonemap_inputs: [wgpu::BindGroup; 2],
    uniform: wgpu::Buffer,
    levels: usize,
    /// The texel the last frame wrote.
    current: Cell<usize>,
    /// When the last frame adapted; `None` before the first, which takes its target at once.
    last: Cell<Option<Instant>>,
}

impl PostChain {
    pub(crate) fn new(
        gpu: &Gpu,
        pipes: &PostPipelines,
        tonemap_layout: &wgpu::BindGroupLayout,
        hdr: &HdrTarget,
    ) -> Self {
        let device = &gpu.device;
        let target = |label, width: u32, height: u32| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: width.max(1),
                        height: height.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let inputs = |source: &wgpu::TextureView, coarser: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ac2 post inputs"),
                layout: &pipes.source_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&pipes.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(coarser),
                    },
                ],
            })
        };
        let sizes = level_sizes(hdr.width, hdr.height);
        let down_views: Vec<_> = sizes
            .iter()
            .map(|&(w, h)| target("ac2 bloom down", w, h))
            .collect();
        let mut down = Vec::with_capacity(sizes.len());
        for (index, view) in down_views.iter().enumerate() {
            let source = if index == 0 {
                &hdr.color_view
            } else {
                &down_views[index - 1]
            };
            down.push(Pass {
                target: view.clone(),
                inputs: inputs(source, source),
            });
        }
        // Up level i adds the tent of level i + 1's sum to down level i; the coarsest level's sum
        // is its own downsample.
        let levels = sizes.len();
        let mut up = Vec::with_capacity(levels.saturating_sub(1));
        let mut coarser = down_views[levels - 1].clone();
        for index in (0..levels - 1).rev() {
            let (w, h) = sizes[index];
            let view = target("ac2 bloom up", w, h);
            up.push(Pass {
                target: view.clone(),
                inputs: inputs(&down_views[index], &coarser),
            });
            coarser = view;
        }
        let bloom = coarser;
        let exposure = [target("ac2 exposure", 1, 1), target("ac2 exposure", 1, 1)];
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 post adapt"),
            size: std::mem::size_of::<Adapt>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let adapt_inputs = [1, 0].map(|other: usize| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ac2 post adapt"),
                layout: &pipes.adapt_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&exposure[other]),
                    },
                ],
            })
        });
        let tonemap_inputs = [0, 1].map(|written: usize| {
            crate::tonemap::post_inputs(
                device,
                tonemap_layout,
                &bloom,
                &pipes.sampler,
                &exposure[written],
            )
        });
        let smallest = &down_views[levels - 1];
        Self {
            width: hdr.width,
            height: hdr.height,
            adapt_source: inputs(smallest, smallest),
            down,
            up,
            exposure,
            adapt_inputs,
            tonemap_inputs,
            uniform,
            levels,
            current: Cell::new(0),
            last: Cell::new(None),
        }
    }

    /// Whether the chain was made for `hdr`'s size.
    pub(crate) fn fits(&self, hdr: &HdrTarget) -> bool {
        (self.width, self.height) == (hdr.width, hdr.height)
    }

    /// Records the frame's bloom and adaptation after the world drew into the scene target.
    pub(crate) fn record(
        &self,
        gpu: &Gpu,
        pipes: &PostPipelines,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let now = Instant::now();
        let (step, instant) = match self.last.replace(Some(now)) {
            Some(last) => ((now - last).as_secs_f32().min(MAX_STEP_SECONDS), 0.0),
            None => (0.0, 1.0),
        };
        let adapt = Adapt {
            step: [step, instant, KEY_LOG2, BLOOM_STRENGTH],
            limits: [
                EV_LIMITS[0],
                EV_LIMITS[1],
                ADAPT_SECONDS[0],
                ADAPT_SECONDS[1],
            ],
            look: [VIGNETTE, 1.0 / self.levels as f32, ADAPT_STRENGTH, 0.0],
        };
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&adapt));
        let run = |encoder: &mut wgpu::CommandEncoder,
                   pipeline: &wgpu::RenderPipeline,
                   target: &wgpu::TextureView,
                   groups: &[&wgpu::BindGroup]| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: gpu.pass_timestamps("post"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            for (index, group) in groups.iter().enumerate() {
                pass.set_bind_group(index as u32, *group, &[]);
            }
            pass.draw(0..3, 0..1);
        };
        for (index, level) in self.down.iter().enumerate() {
            let pipeline = if index == 0 {
                &pipes.down_first
            } else {
                &pipes.down
            };
            run(encoder, pipeline, &level.target, &[&level.inputs]);
        }
        for level in &self.up {
            run(encoder, &pipes.up, &level.target, &[&level.inputs]);
        }
        let written = 1 - self.current.get();
        run(
            encoder,
            &pipes.adapt,
            &self.exposure[written],
            &[&self.adapt_source, &self.adapt_inputs[written]],
        );
        self.current.set(written);
    }

    /// What the tone map binds for the world: this frame's bloom and exposure.
    pub(crate) fn tonemap_inputs(&self) -> &wgpu::BindGroup {
        &self.tonemap_inputs[self.current.get()]
    }
}

#[cfg(test)]
mod tests {
    use super::level_sizes;

    #[test]
    fn bloom_levels_halve_from_half_size_and_stop_at_six_or_small_sides() {
        assert_eq!(
            level_sizes(1920, 1080),
            vec![
                (960, 540),
                (480, 270),
                (240, 135),
                (120, 67),
                (60, 33),
                (30, 16)
            ]
        );
        assert_eq!(level_sizes(64, 40), vec![(32, 20), (16, 10)]);
        assert_eq!(level_sizes(1, 1), vec![(1, 1)]);
    }
}
