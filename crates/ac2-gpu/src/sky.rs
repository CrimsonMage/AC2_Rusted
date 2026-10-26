//! The sky pass under the DAT sky objects: a single-scattering atmosphere lit by the region's
//! sun, graded by the backdrop dome's tint, with a procedural cloud deck (`sky.wgsl`). Retail
//! drew only the DAT objects and the fog quad over a black clear; both still draw over this.

use ac2_math::Mat4;
use ac2_render::{Camera, SceneLighting};
use bytemuck::{Pod, Zeroable};

use crate::shading::srgb_to_linear;
use crate::{Gpu, DEPTH_FORMAT};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SkyUniform {
    inv_view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    fog: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    grade: [f32; 4],
    clouds: [f32; 4],
}

/// What the frame's sky shows beyond the region's lighting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyLook {
    /// The backdrop dome's tint and opacity (`crate::sky_backdrop`), sRGB as authored; `None`
    /// when no dome draws.
    pub backdrop: Option<[f32; 4]>,
    /// The cloud deck's cover, 0 clear to 1 overcast.
    pub cloud_cover: f32,
    /// Client seconds; the clouds drift with it.
    pub time: f64,
}

impl Default for SkyLook {
    fn default() -> Self {
        Self {
            backdrop: None,
            cloud_cover: DEFAULT_CLOUD_COVER,
            time: 0.0,
        }
    }
}

/// The cloud cover until the region's weather drives it.
pub const DEFAULT_CLOUD_COVER: f32 = 0.25;

/// The full-screen sky gradient, its colours set each frame.
pub struct SkyRenderer {
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    group: wgpu::BindGroup,
}

impl SkyRenderer {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 sky"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 sky frame"),
            size: std::mem::size_of::<SkyUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 sky"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 sky"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}",
                    include_str!("shading.wgsl"),
                    include_str!("sky.wgsl")
                )
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 sky"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 sky"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            buffer,
            group,
        }
    }

    /// Writes the frame's sky colours and camera. The sun is `lighting.light_dir` (its travel
    /// direction) with `lighting.dir_color`.
    pub fn set_frame(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        aspect: f32,
        lighting: &SceneLighting,
        look: &SkyLook,
    ) {
        let sun = lighting.light_dir.normalize_or_zero();
        let uniform = SkyUniform {
            inv_view_proj: inverse_view_projection(camera, aspect).to_cols_array_2d(),
            eye: camera.eye.extend(1.0).to_array(),
            fog: {
                let c = lighting.fog_color.map(|c| c.clamp(0.0, 1.0));
                [c[0], c[1], c[2], 1.0]
            },
            sun_dir: (-sun).extend(0.0).to_array(),
            sun_color: {
                let c = lighting.dir_color.map(srgb_to_linear);
                [c[0], c[1], c[2], 0.0]
            },
            grade: match look.backdrop {
                Some([r, g, b, a]) => {
                    let [r, g, b] = [r, g, b].map(|c| srgb_to_linear(c.clamp(0.0, 1.0)));
                    [r, g, b, a.clamp(0.0, 1.0)]
                }
                None => [1.0, 1.0, 1.0, 0.0],
            },
            // The drift wraps once a day so the noise keeps its precision in f32.
            clouds: [look.cloud_cover, (look.time % 86_400.0) as f32, 0.0, 0.0],
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn inverse_view_projection(camera: &Camera, aspect: f32) -> Mat4 {
    camera.view_projection(aspect).inverse()
}
