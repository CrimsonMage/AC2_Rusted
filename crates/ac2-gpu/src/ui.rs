//! The UI over the scene: one texture per root element's surface, drawn the way
//! `UISurfaceObject::Render` draws it with the material `UISurfaceObject::GenerateMaterial`
//! builds (texture modulated by white, alpha modulated, point sampled, clamped) blended
//! SRCALPHA, INVSRCALPHA at its screen position, pixel for pixel, ignoring depth. A surface turns
//! about its centre (`UISurfaceObject::BuildRotateMatrix` applied to the unit quad before
//! `BuildScreenMatrix` places it).

use crate::{Gpu, DEPTH_FORMAT};

const SHADER: &str = "
struct Layer {
    // x, y, width, height in pixels; then the screen width and height and the rotation about
    // z in radians.
    rect: vec4<f32>,
    screen: vec4<f32>,
};

@group(0) @binding(0) var<uniform> layer: Layer;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VertexOut {
    let corner = vec2(f32(index & 1u), f32((index >> 1u) & 1u));
    // Turned about the quad's centre, y up as in clip space.
    let local = vec2(corner.x - 0.5, 0.5 - corner.y);
    let c = cos(layer.screen.z);
    let s = sin(layer.screen.z);
    let turned = vec2(c * local.x - s * local.y, s * local.x + c * local.y);
    let pixel = layer.rect.xy + vec2(turned.x + 0.5, 0.5 - turned.y) * layer.rect.zw;
    var out: VertexOut;
    out.position = vec4(
        pixel.x / layer.screen.x * 2.0 - 1.0,
        1.0 - pixel.y / layer.screen.y * 2.0,
        0.0,
        1.0,
    );
    out.uv = corner;
    return out;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(image, image_sampler, in.uv);
}
";

/// One root surface: its screen position and size, and RGBA8 pixels, rows top-down.
pub struct UiLayer<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// Degrees about the surface's centre, and whether it is shown.
    pub rotation: i32,
    pub shown: bool,
    pub rgba: &'a [u8],
}

struct Uploaded {
    rect: [f32; 4],
    rotation: f32,
    shown: bool,
    uniform: wgpu::Buffer,
    group: wgpu::BindGroup,
}

/// The drawn UI layers.
pub struct UiOverlay {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    layers: Vec<Uploaded>,
}

impl UiOverlay {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let entry = |binding, visibility, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 ui layer"),
            entries: &[
                entry(
                    0,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(
                    1,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 ui point clamp"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 ui layer"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 ui layer"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 ui layer"),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        Self {
            pipeline,
            layout,
            sampler,
            layers: Vec::new(),
        }
    }

    /// Replaces the drawn layers, in drawing order.
    pub fn set_layers<'a>(&mut self, gpu: &Gpu, layers: impl IntoIterator<Item = UiLayer<'a>>) {
        self.layers.clear();
        for layer in layers {
            if layer.width == 0
                || layer.height == 0
                || layer.rgba.len() != (layer.width * layer.height * 4) as usize
            {
                continue;
            }
            let size = wgpu::Extent3d {
                width: layer.width,
                height: layer.height,
                depth_or_array_layers: 1,
            };
            let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ac2 ui surface"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            gpu.queue.write_texture(
                texture.as_image_copy(),
                layer.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layer.width * 4),
                    rows_per_image: Some(layer.height),
                },
                size,
            );
            let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ac2 ui layer rect"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let view = texture.create_view(&Default::default());
            let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ac2 ui layer"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.layers.push(Uploaded {
                rect: [
                    layer.x as f32,
                    layer.y as f32,
                    layer.width as f32,
                    layer.height as f32,
                ],
                rotation: (layer.rotation as f32).to_radians(),
                shown: layer.shown,
                uniform,
                group,
            });
        }
    }

    /// New positions, rotations and shown flags for the uploaded layers, in their order
    /// (`UISurfaceObject::SetScreenPosition` / `SetRotateVals` / show without new pixels).
    pub fn set_placements(&mut self, placements: impl IntoIterator<Item = (i32, i32, i32, bool)>) {
        for (layer, (x, y, rotation, shown)) in self.layers.iter_mut().zip(placements) {
            layer.rect[0] = x as f32;
            layer.rect[1] = y as f32;
            layer.rotation = (rotation as f32).to_radians();
            layer.shown = shown;
        }
    }

    /// How many layers are shown.
    pub fn shown_layers(&self) -> usize {
        self.layers.iter().filter(|layer| layer.shown).count()
    }

    /// How many layers are drawn.
    pub fn layers(&self) -> usize {
        self.layers.len()
    }

    /// Lays the layers out on a `width` x `height` target (before the pass).
    pub fn prepare(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        for layer in &self.layers {
            let data = [
                layer.rect[0],
                layer.rect[1],
                layer.rect[2],
                layer.rect[3],
                width as f32,
                height as f32,
                layer.rotation,
                0.0,
            ];
            queue.write_buffer(&layer.uniform, 0, bytemuck::cast_slice(&data));
        }
    }

    /// Records the layers over whatever the pass has drawn.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        for layer in self.layers.iter().filter(|layer| layer.shown) {
            pass.set_bind_group(0, &layer.group, &[]);
            pass.draw(0..4, 0..1);
        }
    }
}
