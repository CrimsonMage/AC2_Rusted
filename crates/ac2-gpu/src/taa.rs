//! Temporal anti-aliasing of the world (`taa.wgsl`), driven by retail's `Antialiasing` device
//! preference (`ID_Rendering_Antialiasing`). A deliberate deviation from retail (`AGENTS.md`):
//! `RenderDeviceD3D::SetMultiSampleAntialias` set `D3DRS_MULTISAMPLEANTIALIAS` on a multisampled
//! back buffer; here the scene targets stay single-sampled and every world draw is shifted by an
//! 8-phase Halton(2,3) sub-pixel jitter ([`jitter_pixels`]), which a resolve pass after the world
//! accumulates into a history, reprojected from the depth buffer with the camera's motion, or
//! with the object's own screen step where the world pass's velocity target holds one
//! (`velocity.rs`, `object.wgsl`). The tone map sharpens the result (`tonemap.wgsl`).
//!
//! The draws are eye-relative (`crate::sky::camera_at_origin`), so the reprojection is as well:
//! the CPU supplies the current frame's unjittered inverse view-projection, the previous frame's
//! unjittered view-projection and the previous eye minus the current one, taken in f64 (world
//! coordinates are tens of kilometres, where f32 holds millimetres).
//!
//! Only the world takes part. A UI viewport's scene, the sun shadow cascades, the landscape
//! shadow map, the lens flares and the portal space (which draws after the resolve) are not
//! jittered; the history restarts while the portal space shows.

use ac2_math::{Mat4, Vec3};
use ac2_render::Camera;
use bytemuck::{Pod, Zeroable};

use crate::{Gpu, HdrTarget, HDR_FORMAT};

/// Jitter phases before the sequence repeats.
pub const JITTER_PHASES: u32 = 8;
/// The blend towards the current frame at rest, and at `MOTION_PIXELS` of screen motion.
const ALPHA: [f32; 2] = [0.1, 0.25];
/// The screen motion in pixels at which the blend reaches its larger value.
const MOTION_PIXELS: f32 = 4.0;
/// The sharpening strength while the anti-aliasing is on (`tonemap.wgsl`'s CAS lobe, 0..1).
pub(crate) const SHARPEN: f32 = 0.25;
/// An eye step beyond this many metres in one frame is a camera cut: the history is dropped.
const CUT_METRES: f64 = 5.0;

/// The `index`th term of the Halton sequence in `base` (index 1 is the first), in 0..1.
pub fn halton(mut index: u32, base: u32) -> f32 {
    let (mut result, mut fraction) = (0.0_f64, 1.0_f64);
    while index > 0 {
        fraction /= f64::from(base);
        result += fraction * f64::from(index % base);
        index /= base;
    }
    result as f32
}

/// The sub-pixel jitter of `frame` in pixels (x right, y down), centred: -0.5..0.5.
pub fn jitter_pixels(frame: u32) -> [f32; 2] {
    let index = frame % JITTER_PHASES + 1;
    [halton(index, 2) - 0.5, halton(index, 3) - 0.5]
}

/// A jitter of `pixels` (x right, y down) on a `size` target as a clip-space offset: wgpu's NDC
/// has y up, so a step down the screen is a negative y.
pub fn jitter_ndc(pixels: [f32; 2], size: (u32, u32)) -> [f32; 2] {
    [
        2.0 * pixels[0] / size.0.max(1) as f32,
        -2.0 * pixels[1] / size.1.max(1) as f32,
    ]
}

/// `view_projection` with the clip-space translate `ndc` pre-multiplied (x and y move by `ndc`
/// after the perspective divide). Zero gives the matrix back exactly.
pub fn jittered(view_projection: Mat4, ndc: [f32; 2]) -> Mat4 {
    if ndc == [0.0; 2] {
        return view_projection;
    }
    Mat4::from_translation(Vec3::new(ndc[0], ndc[1], 0.0)) * view_projection
}

/// What the resolve needs of one frame's camera.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TaaFrame {
    /// The eye in world coordinates, in f64.
    eye: [f64; 3],
    fov_y: f32,
    /// The unjittered eye-relative view-projection.
    view_projection: Mat4,
}

impl TaaFrame {
    pub(crate) fn new(camera: &Camera, aspect: f32) -> Self {
        Self {
            eye: camera.eye.to_array().map(f64::from),
            fov_y: camera.fov_y,
            view_projection: crate::sky::camera_at_origin(camera).view_projection(aspect),
        }
    }

    /// `self`'s eye minus `other`'s, in f64, stored as f32.
    pub(crate) fn eye_delta(&self, other: &Self) -> [f32; 3] {
        std::array::from_fn(|i| (self.eye[i] - other.eye[i]) as f32)
    }

    fn eye_distance(&self, other: &Self) -> f64 {
        let d: [f64; 3] = std::array::from_fn(|i| self.eye[i] - other.eye[i]);
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }
}

/// Whether `current` does not continue `previous`: the eye moved over [`CUT_METRES`] in one
/// frame (a teleport) or the field of view changed.
pub(crate) fn is_cut(previous: &TaaFrame, current: &TaaFrame) -> bool {
    current.eye_distance(previous) > CUT_METRES || (current.fov_y - previous.fov_y).abs() > 1e-4
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    inv_view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    eye_delta: [f32; 4],
    blend: [f32; 4],
}

pub(crate) struct TaaPipelines {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl TaaPipelines {
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
            label: Some("ac2 taa"),
            entries: &[
                texture(0, wgpu::TextureSampleType::Float { filterable: false }),
                texture(1, wgpu::TextureSampleType::Depth),
                texture(2, wgpu::TextureSampleType::Float { filterable: true }),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture(5, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 taa"),
            source: wgpu::ShaderSource::Wgsl(include_str!("taa.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 taa"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 taa"),
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
                    format: HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 taa history"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
        }
    }
}

/// The two history textures for one scene size.
struct TaaTargets {
    size: (u32, u32),
    textures: [wgpu::Texture; 2],
    views: [wgpu::TextureView; 2],
    uniform: wgpu::Buffer,
    /// The index of the texture holding the latest resolved frame.
    latest: usize,
}

impl TaaTargets {
    fn new(gpu: &Gpu, size: (u32, u32)) -> Self {
        let texture = || {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ac2 taa history"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: HDR_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let textures = [texture(), texture()];
        let views = [
            textures[0].create_view(&Default::default()),
            textures[1].create_view(&Default::default()),
        ];
        let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 taa"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            size,
            textures,
            views,
            uniform,
            latest: 0,
        }
    }
}

/// The temporal anti-aliasing's state: the preference, the frame counter that picks the jitter,
/// the camera of the frame being drawn and of the last one resolved, and the history.
pub(crate) struct Temporal {
    enabled: bool,
    frame: u32,
    /// The output size of the last scene target requested.
    size: (u32, u32),
    /// The camera of the frame drawn with a jitter, until it is resolved.
    pending: Option<TaaFrame>,
    /// The camera of the last resolved frame: the history holds it. `None`: no history.
    previous: Option<TaaFrame>,
    targets: Option<TaaTargets>,
}

impl Temporal {
    pub(crate) fn new() -> Self {
        Self {
            enabled: false,
            frame: 0,
            size: (0, 0),
            pending: None,
            previous: None,
            targets: None,
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    /// Turns the anti-aliasing on or off; the history restarts either way.
    pub(crate) fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        self.pending = None;
        self.previous = None;
    }

    /// The size of the scene target the world draws into.
    pub(crate) fn set_size(&mut self, size: (u32, u32)) {
        self.size = size;
    }

    /// Starts a world frame seen by `camera`: the clip-space jitter every world draw of the frame
    /// shifts by (zero while off, before the first scene target, or `suppressed` by the portal
    /// space, which restarts the history).
    pub(crate) fn begin_frame(
        &mut self,
        camera: &Camera,
        aspect: f32,
        suppressed: bool,
    ) -> [f32; 2] {
        self.pending = None;
        if !self.enabled || suppressed {
            self.previous = None;
            return [0.0; 2];
        }
        if self.size.0 == 0 || self.size.1 == 0 {
            return [0.0; 2];
        }
        let pixels = jitter_pixels(self.frame);
        self.frame = self.frame.wrapping_add(1);
        self.pending = Some(TaaFrame::new(camera, aspect));
        jitter_ndc(pixels, self.size)
    }

    /// Records the resolve of the world drawn in `hdr` into the history and copies the result
    /// back over the scene colour, once per frame after the last world contribution.
    pub(crate) fn resolve(
        &mut self,
        gpu: &Gpu,
        pipes: &TaaPipelines,
        encoder: &mut wgpu::CommandEncoder,
        hdr: &HdrTarget,
    ) {
        let Some(current) = self.pending.take() else {
            return;
        };
        let size = (hdr.width, hdr.height);
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            self.targets = Some(TaaTargets::new(gpu, size));
            self.previous = None;
        }
        let Some(targets) = self.targets.as_mut() else {
            return;
        };
        let previous = self.previous.filter(|p| !is_cut(p, &current));
        let (prev_view_proj, delta) = match &previous {
            Some(p) => (p.view_projection, p.eye_delta(&current)),
            None => (current.view_projection, [0.0; 3]),
        };
        let params = Params {
            inv_view_proj: current.view_projection.inverse().to_cols_array_2d(),
            prev_view_proj: prev_view_proj.to_cols_array_2d(),
            eye_delta: [delta[0], delta[1], delta[2], 0.0],
            blend: [
                f32::from(u8::from(previous.is_some())),
                ALPHA[0],
                ALPHA[1],
                MOTION_PIXELS,
            ],
        };
        gpu.queue
            .write_buffer(&targets.uniform, 0, bytemuck::bytes_of(&params));
        let (read, write) = (targets.latest, 1 - targets.latest);
        let inputs = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 taa"),
            layout: &pipes.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr.color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&hdr.depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&targets.views[read]),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&pipes.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: targets.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&hdr.velocity_view),
                },
            ],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 taa"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.views[write],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: gpu.pass_timestamps("taa"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipes.pipeline);
            pass.set_bind_group(0, &inputs, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.copy_texture_to_texture(
            targets.textures[write].as_image_copy(),
            hdr.color.as_image_copy(),
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        targets.latest = write;
        self.previous = Some(current);
    }
}

#[cfg(test)]
#[path = "taa_tests.rs"]
mod tests;
