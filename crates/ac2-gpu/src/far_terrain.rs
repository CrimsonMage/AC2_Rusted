//! The far landscape pass (`LScapeTerrainMesh::DrawTerrain`'s far chunks): one vertex and index
//! buffer for the whole far mesh, one draw per chunk, sharing the near pass's frame uniform.

use std::ops::Range;

use ac2_core::DataId;
use ac2_render::terrain::{FarTerrain, FarVertices};
use ac2_render::Image;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::TerrainRenderer;
use crate::texture::TextureKey;
use crate::{Gpu, DEPTH_FORMAT};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuFarVertex {
    position: [f32; 3],
    normal: [f32; 3],
    far_color: [f32; 4],
    uv: [f32; 2],
}

pub(super) struct FarPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl FarPipeline {
    pub(super) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 far terrain"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 far terrain"),
            source: wgpu::ShaderSource::Wgsl(include_str!("far_terrain.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 far terrain"),
            bind_group_layouts: &[frame_layout, &layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 far terrain"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GpuFarVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x2
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
        Self { pipeline, layout }
    }
}

/// The far mesh on the GPU.
pub struct GpuFarTerrain {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    chunks: Vec<Range<u32>>,
    group: wgpu::BindGroup,
}

impl TerrainRenderer {
    /// Uploads the far mesh around one centre block; `texture` is the far texture
    /// (`Ac2FarTerrain::texture`).
    pub fn upload_far(
        &mut self,
        gpu: &Gpu,
        far: &FarTerrain,
        vertices: &FarVertices,
        texture: DataId,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> GpuFarTerrain {
        let device = &gpu.device;
        let gpu_vertices: Vec<GpuFarVertex> = vertices
            .terrain
            .iter()
            .map(|v| GpuFarVertex {
                position: v.position,
                normal: v.normal,
                far_color: v.color,
                uv: v.uv,
            })
            .collect();
        let view = self
            .textures
            .get(device, &gpu.queue, TextureKey::Texture(texture), load);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 far terrain"),
            layout: &self.far.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.wrap),
                },
            ],
        });
        GpuFarTerrain {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ac2 far terrain vertices"),
                contents: bytemuck::cast_slice(&gpu_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ac2 far terrain indices"),
                contents: bytemuck::cast_slice(far.indices()),
                usage: wgpu::BufferUsages::INDEX,
            }),
            chunks: far.chunk_ranges().to_vec(),
            group,
        }
    }

    /// Records the far chunks from `first_chunk` on (`FIRST_FAR_CHUNK` beside a drawn near mesh,
    /// 1 without one).
    pub fn draw_far(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        far: &GpuFarTerrain,
        first_chunk: usize,
    ) {
        let chunks: Vec<usize> = (first_chunk..far.chunks.len()).collect();
        self.draw_far_chunks(pass, far, &chunks);
    }

    /// Draws the listed chunks of the far mesh (`DrawMeshDegrade` per chunk that passed the
    /// frustum test).
    pub fn draw_far_chunks(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        far: &GpuFarTerrain,
        chunks: &[usize],
    ) {
        pass.set_pipeline(&self.far.pipeline);
        pass.set_bind_group(0, &self.frame_group, &[]);
        pass.set_bind_group(1, &far.group, &[]);
        pass.set_vertex_buffer(0, far.vertices.slice(..));
        pass.set_index_buffer(far.indices.slice(..), wgpu::IndexFormat::Uint32);
        for range in chunks.iter().filter_map(|&chunk| far.chunks.get(chunk)) {
            pass.draw_indexed(range.clone(), 0, 0..1);
        }
    }
}
