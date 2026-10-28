//! The portal space's tunnel rings (`PortalSpace::Effect::Render`'s two `RenderIndexedUsingMaterial`
//! calls): a triangle list over the tunnel's vertex buffer, each vertex's colour (the ring's fade)
//! modulated with its material's texture, alpha blended (SRCALPHA over INVSRCALPHA), no depth
//! write. The vertex buffers are rewritten each frame from the tunnel's geometry.

use ac2_render::portal::TunnelVertex;

use crate::{Gpu, DEPTH_FORMAT};

/// The vertices of one tunnel (`TunnelPortalType` rings x 17).
pub const TUNNEL_VERTICES: usize = 2550;
/// The vertex's stride: position (3 floats), the colour as RGBA bytes, the texture coordinates.
const VERTEX_BYTES: u64 = 24;
/// The indices drawn: 149 rings of 16 quads (`0x12A0` triangles' worth of indices, the wrap
/// ring excluded), per `Render`'s draw of the tunnel.
pub const DRAWN_INDICES: u32 = 14_304;

const SHADER: &str = "
struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs(v: VsIn) -> VsOut {
    var o: VsOut;
    o.clip = u.view_proj * vec4<f32>(v.position, 1.0);
    o.color = v.color;
    o.uv = v.uv;
    return o;
}

@fragment
fn fs(i: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, i.uv) * i.color;
}
";

/// One tunnel's buffers: its vertices (rewritten per frame) and its draw's index count.
struct Mesh {
    vertices: wgpu::Buffer,
}

/// The tunnel pipeline and its shared index buffer.
pub struct PortalTunnel {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    indices: wgpu::Buffer,
    /// The inner and the outer tunnel.
    meshes: [Mesh; 2],
    /// The bind groups of the outer and inner tunnel (texture and view), set by `update`.
    groups: [Option<wgpu::BindGroup>; 2],
}

impl PortalTunnel {
    /// The pipeline for `format`, with the tunnel's index buffer (`indices`, as the effect builds it).
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat, indices: &[u16]) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 portal tunnel"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 portal tunnel view"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 portal tunnel"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let index_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 portal tunnel indices"),
            size: index_bytes.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&index_buffer, 0, &index_bytes);
        let vertex_buffer = || {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ac2 portal tunnel vertices"),
                size: TUNNEL_VERTICES as u64 * VERTEX_BYTES,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 portal tunnel"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 portal tunnel"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Unorm8x4,
                offset: 12,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 portal tunnel"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: VERTEX_BYTES,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
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
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
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
            layout,
            uniform,
            sampler,
            indices: index_buffer,
            meshes: [
                Mesh {
                    vertices: vertex_buffer(),
                },
                Mesh {
                    vertices: vertex_buffer(),
                },
            ],
            groups: [None, None],
        }
    }

    /// Writes the frame's geometry (`inner` and `outer`, each rewritten to [`TUNNEL_VERTICES`]),
    /// the view-projection, and the bind groups that sample each tunnel's material texture
    /// (`inner_texture`, `outer_texture`).
    pub fn update(
        &mut self,
        gpu: &Gpu,
        view_proj: &[f32; 16],
        inner: &[TunnelVertex],
        outer: &[TunnelVertex],
        textures: [&wgpu::TextureView; 2],
    ) {
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::cast_slice(view_proj));
        for (mesh, vertices) in self.meshes.iter().zip([inner, outer]) {
            let bytes = encode(vertices);
            gpu.queue.write_buffer(&mesh.vertices, 0, &bytes);
        }
        // The outer tunnel is drawn first, as `Render` does.
        for (group, texture) in self.groups.iter_mut().zip([textures[1], textures[0]]) {
            *group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ac2 portal tunnel"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            }));
        }
    }

    /// Draws the outer tunnel, then the inner (`PortalSpace::Effect::Render`'s order), with the
    /// last [`PortalTunnel::update`]'s geometry and textures.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
        for (mesh, group) in [
            (&self.meshes[1], &self.groups[0]),
            (&self.meshes[0], &self.groups[1]),
        ] {
            let Some(group) = group else {
                continue;
            };
            pass.set_bind_group(0, group, &[]);
            pass.set_vertex_buffer(0, mesh.vertices.slice(..));
            pass.draw_indexed(0..DRAWN_INDICES, 0, 0..1);
        }
    }
}

/// The vertices as the pipeline reads them: position, the colour bytes (`R, G, B, A`, the
/// packed ARGB `TunnelVertex::color` unpacked), and the texture coordinates. Padded to
/// [`TUNNEL_VERTICES`].
fn encode(vertices: &[TunnelVertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(TUNNEL_VERTICES * VERTEX_BYTES as usize);
    for v in vertices.iter().take(TUNNEL_VERTICES) {
        for p in v.position {
            bytes.extend(p.to_le_bytes());
        }
        let argb = v.color;
        bytes.extend([
            (argb >> 16) as u8,
            (argb >> 8) as u8,
            argb as u8,
            (argb >> 24) as u8,
        ]);
        for uv in v.uv {
            bytes.extend(uv.to_le_bytes());
        }
    }
    bytes.resize(TUNNEL_VERTICES * VERTEX_BYTES as usize, 0);
    bytes
}
