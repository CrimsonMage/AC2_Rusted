//! The water's reflection, after the world pass: `WorldRender::RenderReflectionMap` mirrored the
//! camera about the frame's water plane (`LScapeRender::FindReflectionPlane`) and drew the sky and
//! the world into a target, and `LScapeTerrainMesh::DrawWater` blended that target over the water's
//! pixels (stencilled) by their destination alpha, rippled by a bump map at `WaterReflections` 2.
//! Here the world's own pixels on that plane march the mirrored view ray through the world's depth
//! (`water_reflection.wgsl`) and take what they meet, or the sky where the ray leaves the world,
//! blended over the water by a Fresnel weight, with ripples at level 2 (`AGENTS.md`).

use std::cell::RefCell;

use ac2_math::{camera, Mat4, Vec3, Vec4};
use ac2_render::scene::ReflectionPlane;
use ac2_render::Camera;
use bytemuck::{Pod, Zeroable};

use crate::{Gpu, HdrTarget, HDR_FORMAT};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniform {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    plane: [f32; 4],
    params: [f32; 4],
    horizon: [f32; 4],
    blocks: [f32; 4],
}

/// The blocks of the water window, a side (the resident ring is at most 9 wide).
const WINDOW: u32 = 16;

const COMPOSITE: &str = "
@group(0) @binding(0) var reflection: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(reflection, vec2<i32>(position.xy), 0);
}
";

/// The reflection target for one scene size, and the composite's binding of it.
struct Target {
    size: (u32, u32),
    view: wgpu::TextureView,
    composite: wgpu::BindGroup,
}

/// The water reflection's pipelines and per-frame state.
pub struct WaterReflection {
    march: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    march_layout: wgpu::BindGroupLayout,
    composite_layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    /// Which blocks of the window around the water hold it.
    blocks: wgpu::Texture,
    blocks_view: wgpu::TextureView,
    target: RefCell<Option<Target>>,
    /// Whether the frame reflects (a plane was found and the level draws one).
    active: bool,
}

impl WaterReflection {
    pub fn new(gpu: &Gpu) -> Self {
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
        let march_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 water reflection"),
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
                texture(1, wgpu::TextureSampleType::Float { filterable: true }),
                texture(2, wgpu::TextureSampleType::Depth),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture(4, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let composite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 water reflection composite"),
            entries: &[texture(
                0,
                wgpu::TextureSampleType::Float { filterable: false },
            )],
        });
        let pipeline = |label, source: String, layout: &wgpu::BindGroupLayout, entry, blend| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(label),
                        bind_group_layouts: &[Some(layout)],
                        immediate_size: 0,
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend,
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
        let march = pipeline(
            "ac2 water reflection",
            include_str!("water_reflection.wgsl").to_string(),
            &march_layout,
            "fs_march",
            None,
        );
        // Over the water by the reflection's weight; the scene's coverage alpha is kept.
        let over = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let composite = pipeline(
            "ac2 water reflection composite",
            COMPOSITE.to_string(),
            &composite_layout,
            "fs",
            Some(over),
        );
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 water reflection"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 water reflection"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let blocks = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ac2 water reflection blocks"),
            size: wgpu::Extent3d {
                width: WINDOW,
                height: WINDOW,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let blocks_view = blocks.create_view(&Default::default());
        Self {
            march,
            composite,
            march_layout,
            composite_layout,
            uniform,
            sampler,
            blocks,
            blocks_view,
            target: RefCell::new(None),
            active: false,
        }
    }

    /// Writes the frame's reflection (`None`: nothing reflects): `horizon` is the linear colour a
    /// ray that meets nothing takes, `time` drives the ripples.
    pub fn prepare(
        &mut self,
        gpu: &Gpu,
        view: &Camera,
        aspect: f32,
        time: f64,
        reflection: Option<&ReflectionPlane>,
        horizon: [f32; 3],
    ) {
        self.active = false;
        let Some(plane) = reflection.filter(|plane| plane.level > 0 && !plane.blocks.is_empty())
        else {
            return;
        };
        let (height, level, block_size) = (plane.height, plane.level, plane.block_size);
        if view.eye.z <= height {
            return;
        }
        let origin = (
            plane.blocks.iter().map(|b| b.0).min().unwrap_or(0),
            plane.blocks.iter().map(|b| b.1).min().unwrap_or(0),
        );
        let mut mask = [0_u8; (WINDOW * WINDOW) as usize];
        for (x, y) in &plane.blocks {
            let (dx, dy) = (u32::from(x - origin.0), u32::from(y - origin.1));
            if dx < WINDOW && dy < WINDOW {
                mask[(dy * WINDOW + dx) as usize] = 255;
            }
        }
        gpu.queue.write_texture(
            self.blocks.as_image_copy(),
            &mask,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WINDOW),
                rows_per_image: Some(WINDOW),
            },
            wgpu::Extent3d {
                width: WINDOW,
                height: WINDOW,
                depth_or_array_layers: 1,
            },
        );
        let view_matrix = camera::look_at_mat4(view.eye, view.target, Vec3::Z);
        let proj = camera::perspective(view.fov_y, aspect.max(1e-3), view.near, view.far);
        let plane = view_plane(view_matrix, view.eye, height);
        let uniform = Uniform {
            view: view_matrix.to_cols_array_2d(),
            proj: proj.to_cols_array_2d(),
            inv_proj: proj.inverse().to_cols_array_2d(),
            inv_view: view_matrix.inverse().to_cols_array_2d(),
            plane: plane.to_array(),
            params: [(time % 3600.0) as f32, f32::from(level), view.near, 0.0],
            horizon: [horizon[0], horizon[1], horizon[2], 1.0],
            blocks: [
                f32::from(origin.0) * block_size,
                f32::from(origin.1) * block_size,
                block_size,
                WINDOW as f32,
            ],
        };
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniform));
        self.active = true;
    }

    /// Records the reflection over the world's water in `hdr`, after the world pass and before
    /// anything clears its depth.
    pub fn record(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, hdr: &HdrTarget) {
        if !self.active {
            return;
        }
        let size = (hdr.width, hdr.height);
        let mut target = self.target.borrow_mut();
        if target.as_ref().is_none_or(|t| t.size != size) {
            *target = Some(self.make_target(gpu, size));
        }
        let Some(target) = target.as_ref() else {
            return;
        };
        let inputs = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 water reflection"),
            layout: &self.march_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&hdr.color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&hdr.depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.blocks_view),
                },
            ],
        });
        let pass = |encoder: &mut wgpu::CommandEncoder,
                    view: &wgpu::TextureView,
                    load,
                    pipeline: &wgpu::RenderPipeline,
                    group: &wgpu::BindGroup| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 water reflection"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        };
        pass(
            encoder,
            &target.view,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            &self.march,
            &inputs,
        );
        pass(
            encoder,
            &hdr.color_view,
            wgpu::LoadOp::Load,
            &self.composite,
            &target.composite,
        );
    }

    fn make_target(&self, gpu: &Gpu, size: (u32, u32)) -> Target {
        let view = gpu
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ac2 water reflection"),
                size: wgpu::Extent3d {
                    width: size.0.max(1),
                    height: size.1.max(1),
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
            .create_view(&Default::default());
        let composite = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 water reflection composite"),
            layout: &self.composite_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        Target {
            size,
            view,
            composite,
        }
    }
}

/// The view-space plane of `z = height` seen by `view` from `eye`: the up normal turned into
/// view space, and its distance through the point of the plane under the eye.
fn view_plane(view: Mat4, eye: Vec3, height: f32) -> Vec4 {
    let normal = view.transform_vector3(Vec3::Z).normalize();
    let point = view.transform_point3(Vec3::new(eye.x, eye.y, height));
    Vec4::new(normal.x, normal.y, normal.z, -normal.dot(point))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plane_measures_height_above_the_water_in_view_space() {
        let eye = Vec3::new(10.0, 20.0, 15.0);
        let view = camera::look_at_mat4(eye, eye + Vec3::new(0.0, 1.0, -0.3), Vec3::Z);
        let plane = view_plane(view, eye, 5.0);
        let at = |world: Vec3| {
            let p = view.transform_point3(world);
            plane.truncate().dot(p) + plane.w
        };
        assert!(at(Vec3::new(40.0, 70.0, 5.0)).abs() < 1e-4);
        assert!((at(Vec3::new(3.0, 1.0, 7.0)) - 2.0).abs() < 1e-4);
        assert!((at(eye) - 10.0).abs() < 1e-4);
    }
}
