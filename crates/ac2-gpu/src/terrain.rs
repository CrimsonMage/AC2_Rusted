//! The near-landscape pass: one pipeline, one bind group per layer set (the retail per-material
//! index lists of `LScapeTerrainMesh::BuildNearTerrainMesh`), one vertex and index buffer per
//! landblock.

use std::ops::Range;

use ac2_render::terrain::{TerrainLayer, TerrainMesh};
use ac2_render::{Camera, Image, SceneLighting};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::texture::{TextureCache, TextureKey};
use crate::{Gpu, DEPTH_FORMAT};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    grid: [f32; 2],
    corner: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FrameUniform {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    sun_dir: [f32; 4],
    sun: [f32; 4],
    ambient: [f32; 4],
    fog_color: [f32; 4],
    fog: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BatchUniform {
    scale: [f32; 4],
    rot: [u32; 4],
}

/// One landblock's near mesh on the GPU.
pub struct TerrainBlock {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    batches: Vec<(Range<u32>, wgpu::BindGroup)>,
}

pub struct TerrainRenderer {
    pipeline: wgpu::RenderPipeline,
    batch_layout: wgpu::BindGroupLayout,
    frame_buffer: wgpu::Buffer,
    frame_group: wgpu::BindGroup,
    wrap: wgpu::Sampler,
    clamp: wgpu::Sampler,
    pub textures: TextureCache,
}

impl TerrainRenderer {
    pub fn new(gpu: &Gpu, color_format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let uniform = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
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
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 terrain frame"),
            entries: &[uniform(0)],
        });
        let batch_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 terrain batch"),
            entries: &[
                uniform(0),
                texture(1),
                texture(2),
                texture(3),
                texture(4),
                texture(5),
                sampler(6),
                sampler(7),
            ],
        });
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 terrain frame uniform"),
            size: std::mem::size_of::<FrameUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let frame_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 terrain frame"),
            layout: &frame_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 terrain"),
            source: wgpu::ShaderSource::Wgsl(include_str!("terrain.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 terrain"),
            bind_group_layouts: &[&frame_layout, &batch_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 terrain"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GpuVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x2, 4 => Uint32
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(color_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let filtered = |address_mode| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: address_mode,
                address_mode_v: address_mode,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        };
        Self {
            pipeline,
            batch_layout,
            frame_buffer,
            frame_group,
            wrap: filtered(wgpu::AddressMode::Repeat),
            clamp: filtered(wgpu::AddressMode::ClampToEdge),
            textures: TextureCache::new(device, &gpu.queue),
        }
    }

    /// Uploads one landblock's mesh. `block_size` is the block edge in metres; `load` supplies
    /// images for textures not yet on the GPU.
    pub fn upload_block(
        &mut self,
        gpu: &Gpu,
        mesh: &TerrainMesh,
        block_size: f32,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> TerrainBlock {
        let device = &gpu.device;
        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| GpuVertex {
                position: v.position,
                normal: v.normal,
                color: v.color,
                grid: v.grid,
                corner: v.corner,
            })
            .collect();
        let mut indices = Vec::new();
        let mut batches = Vec::with_capacity(mesh.batches.len());
        let spacing = block_size / 16.0;
        for batch in &mesh.batches {
            let start = indices.len() as u32;
            indices.extend_from_slice(&batch.indices);
            let group = self.batch_group(gpu, &batch.layers.0, spacing, load);
            batches.push((start..indices.len() as u32, group));
        }
        TerrainBlock {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ac2 terrain vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ac2 terrain indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            batches,
        }
    }

    fn batch_group(
        &mut self,
        gpu: &Gpu,
        layers: &[TerrainLayer],
        spacing: f32,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> wgpu::BindGroup {
        let mut scale = [0.0; 4];
        let mut rot = [1, 1, layers.len() as u32, 0];
        let mut bases = Vec::new();
        let mut masks = Vec::new();
        for (index, layer) in layers.iter().take(3).enumerate() {
            // `LScapeMaterialInfo::CalcUVInfo`: block_length / ((side_vertex_count - 1) * tiling).
            scale[index] = if layer.tiling() > 0.0 {
                spacing / layer.tiling()
            } else {
                1.0
            };
            let key = TextureKey::Material(layer.material);
            bases.push(self.textures.get(&gpu.device, &gpu.queue, key, load));
            if let Some((mask, rotation)) = layer.alpha {
                rot[index - 1] = rotation as u32;
                let key = TextureKey::Mask(mask);
                masks.push(self.textures.get(&gpu.device, &gpu.queue, key, load));
            }
        }
        let white = self.textures.white().clone();
        bases.resize(3, white.clone());
        masks.resize(2, white);
        let uniform = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ac2 terrain batch uniform"),
                contents: bytemuck::bytes_of(&BatchUniform { scale, rot }),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        fn view(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(view),
            }
        }
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 terrain batch"),
            layout: &self.batch_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                view(1, &bases[0]),
                view(2, &bases[1]),
                view(3, &bases[2]),
                view(4, &masks[0]),
                view(5, &masks[1]),
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&self.wrap),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&self.clamp),
                },
            ],
        })
    }

    /// Writes this frame's camera and lighting.
    pub fn set_frame(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        lighting: &SceneLighting,
        aspect: f32,
    ) {
        let (fog_start, fog_end, fog_on) = match lighting.fog_range {
            Some((start, end)) => (start, end, 1.0),
            None => (0.0, 1.0, 0.0),
        };
        let rgb = |c: [f32; 3]| [c[0], c[1], c[2], 1.0];
        let uniform = FrameUniform {
            view_proj: camera.view_projection(aspect).to_cols_array_2d(),
            eye: camera.eye.extend(1.0).to_array(),
            sun_dir: lighting.sun_direction.extend(0.0).to_array(),
            sun: rgb(lighting.sun),
            ambient: rgb(lighting.ambient),
            fog_color: rgb(lighting.fog_color),
            fog: [fog_start, fog_end, fog_on, 0.0],
        };
        queue.write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Records the near landscape of `blocks` into an open pass.
    pub fn draw<'b>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        blocks: impl IntoIterator<Item = &'b TerrainBlock>,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.frame_group, &[]);
        for block in blocks {
            pass.set_vertex_buffer(0, block.vertices.slice(..));
            pass.set_index_buffer(block.indices.slice(..), wgpu::IndexFormat::Uint32);
            for (range, group) in &block.batches {
                pass.set_bind_group(1, group, &[]);
                pass.draw_indexed(range.clone(), 0, 0..1);
            }
        }
    }
}
