//! The landscape's static shadow map (`LScapeRender::RefreshShadowMap`): a 512 x 512 target
//! looking down the sun (`Matrix4::DirectionMatrix(m_finalLightDir, +Z)`), its orthographic
//! window fitted to the near terrain's vertices in light space (`SceneTool::StartShadowMap`),
//! cleared white, the near blocks' static objects drawn black over it, then
//! `SceneTool::FinishShadowMap` adds one minus `m_cStaticShadowStrength` everywhere, so a full
//! shadow keeps that much light. The near terrain modulates by it (texcoord 7).
//!
//! Retail redraws it when the landscape lighting or the near mesh refreshes; here it redraws when
//! the near blocks, their objects, the sun's direction (by more than a degree) or the strength
//! change.

use ac2_math::{Mat4, Vec3, Vec4};

use crate::{Gpu, ObjectInstance, ObjectRenderer, TerrainBlock};

/// The map's edge in texels (`PrepareGenericRenderTarget0`'s 0x200).
pub const SHADOW_MAP_SIZE: u32 = 512;
/// The map's format; the terrain samples its colour.
pub const SHADOW_MAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// `cos(1 degree)`: a smaller sun move keeps the map.
const REFRESH_COS: f32 = 0.999_848;

const FLOOR_SHADER: &str = "
@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4<f32> {
    return color;
}
";

/// What a drawn map was drawn for.
#[derive(Debug, Clone, PartialEq)]
struct ShadowKey {
    blocks: Vec<(u8, u8)>,
    casters: usize,
    sun: Vec3,
    strength: [f32; 3],
}

impl ShadowKey {
    /// Whether a map drawn for `self` still serves `next`.
    fn serves(&self, next: &ShadowKey) -> bool {
        let strength = self
            .strength
            .iter()
            .zip(next.strength)
            .all(|(a, b)| (a - b).abs() < 0.5 / 255.0);
        self.blocks == next.blocks
            && self.casters == next.casters
            && self.sun.dot(next.sun) >= REFRESH_COS
            && strength
    }
}

/// The light-space window of a map: `DirectionMatrix`'s right and up axes and the bounds of the
/// terrain along them.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ShadowWindow {
    right: Vec3,
    up: Vec3,
    min: [f32; 2],
    max: [f32; 2],
}

impl ShadowWindow {
    /// The window looking down `sun` (the direction light travels) over `points`; `None` with no
    /// sun or no points.
    fn fit(sun: Vec3, points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let forward = sun.normalize_or_zero();
        if forward == Vec3::ZERO {
            return None;
        }
        let right = forward.cross(Vec3::Z);
        let right = if right.length() < 1e-4 {
            Vec3::X
        } else {
            right.normalize()
        };
        let up = right.cross(forward);
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for p in points {
            let at = [p.dot(right), p.dot(up)];
            for axis in 0..2 {
                min[axis] = min[axis].min(at[axis]);
                max[axis] = max[axis].max(at[axis]);
            }
        }
        (min[0] < max[0] && min[1] < max[1]).then_some(Self {
            right,
            up,
            min,
            max,
        })
    }

    /// World to map coordinates: `StartShadowMap`'s texcoord 7, `(x - min.x) / (max.x - min.x)`
    /// and `1 - (z - min.z) / (max.z - min.z)` in light space.
    fn texture_matrix(&self) -> Mat4 {
        let width = self.max[0] - self.min[0];
        let height = self.max[1] - self.min[1];
        let u = self.right / width;
        let v = -self.up / height;
        Mat4::from_cols(
            Vec4::new(u.x, v.x, 0.0, 0.0),
            Vec4::new(u.y, v.y, 0.0, 0.0),
            Vec4::new(u.z, v.z, 0.0, 0.0),
            Vec4::new(-self.min[0] / width, 1.0 + self.min[1] / height, 0.0, 1.0),
        )
    }

    /// World to the map's clip space (the orthographic projection the casters draw through),
    /// matching [`Self::texture_matrix`]: texture v runs down, clip y up.
    fn clip_matrix(&self) -> Mat4 {
        let to_clip = Mat4::from_cols(
            Vec4::new(2.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, -2.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 0.0),
            Vec4::new(-1.0, 1.0, 0.5, 1.0),
        );
        to_clip * self.texture_matrix()
    }
}

/// The map target and its floor pass.
pub struct StaticShadowMap {
    view: wgpu::TextureView,
    floor: wgpu::RenderPipeline,
    floor_buffer: wgpu::Buffer,
    floor_group: wgpu::BindGroup,
    drawn: Option<ShadowKey>,
    matrix: Mat4,
}

impl StaticShadowMap {
    pub fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ac2 static shadow map"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SHADOW_MAP_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 shadow floor"),
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
        let floor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 shadow floor"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let floor_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 shadow floor"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: floor_buffer.as_entire_binding(),
            }],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 shadow floor"),
            source: wgpu::ShaderSource::Wgsl(FLOOR_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 shadow floor"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        // `FinishShadowMap`'s screen quad: SRCALPHA/ONE at alpha 1.
        let additive = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let floor = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 shadow floor"),
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
                    format: SHADOW_MAP_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: additive,
                        alpha: additive,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let map = Self {
            view,
            floor,
            floor_buffer,
            floor_group,
            drawn: None,
            matrix: Mat4::ZERO,
        };
        let mut encoder = device.create_command_encoder(&Default::default());
        drop(map.clear(&mut encoder));
        gpu.queue.submit([encoder.finish()]);
        map
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// `LScapeRender::RefreshShadowMap` for near terrain `blocks` (by key, in a stable order)
    /// lit along `sun` (the direction light travels) at `strength`: draws `casters` into the map
    /// when what it shows changed, and returns the world-to-map transform the terrain samples by
    /// (zero, the white corner, without terrain).
    pub fn refresh(
        &mut self,
        gpu: &Gpu,
        objects: &mut ObjectRenderer,
        blocks: &[((u8, u8), &TerrainBlock)],
        sun: Vec3,
        strength: [f32; 3],
        casters: &[ObjectInstance<'_>],
    ) -> Mat4 {
        let points = blocks
            .iter()
            .flat_map(|(_, block)| block.positions.iter().copied());
        let Some(window) = ShadowWindow::fit(sun, points) else {
            self.reset(gpu);
            return Mat4::ZERO;
        };
        let key = ShadowKey {
            blocks: blocks.iter().map(|(key, _)| *key).collect(),
            casters: casters.len(),
            sun: sun.normalize_or_zero(),
            strength,
        };
        if !self.drawn.as_ref().is_some_and(|drawn| drawn.serves(&key)) {
            self.draw(gpu, key, &window, |encoder, view, clip| {
                objects.draw_shadow_casters(gpu, encoder, view, clip, casters);
            });
        }
        self.matrix
    }

    /// `StartShadowMap`'s clear to white.
    fn clear<'e>(&self, encoder: &'e mut wgpu::CommandEncoder) -> wgpu::RenderPass<'e> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ac2 static shadow map"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
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
        })
    }

    /// Draws the map for `key` over `window`: clear, `casters` (recorded into the encoder
    /// against the map's view and clip matrix), then the floor.
    fn draw(
        &mut self,
        gpu: &Gpu,
        key: ShadowKey,
        window: &ShadowWindow,
        casters: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, Mat4),
    ) {
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        drop(self.clear(&mut encoder));
        casters(&mut encoder, &self.view, window.clip_matrix());
        let [r, g, b] = key.strength.map(|s| 1.0 - s);
        gpu.queue
            .write_buffer(&self.floor_buffer, 0, bytemuck::cast_slice(&[r, g, b, 1.0]));
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 static shadow floor"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.floor);
            pass.set_bind_group(0, &self.floor_group, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
        self.matrix = window.texture_matrix();
        self.drawn = Some(key);
    }

    /// Forgets the map (a scene without near terrain): the terrain samples white again.
    fn reset(&mut self, gpu: &Gpu) {
        if self.drawn.take().is_some() {
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            drop(self.clear(&mut encoder));
            gpu.queue.submit([encoder.finish()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_maps_the_terrain_bounds_onto_the_unit_square() {
        let sun = Vec3::new(0.3, 0.4, -1.0);
        let points = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(192.0, 0.0, 10.0),
            Vec3::new(0.0, 192.0, 5.0),
            Vec3::new(192.0, 192.0, 0.0),
        ];
        let window = ShadowWindow::fit(sun, points).unwrap();
        let matrix = window.texture_matrix();
        let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        for p in points {
            let uv = matrix.transform_point3(p);
            lo = lo.min(uv);
            hi = hi.max(uv);
        }
        assert!(lo.x.abs() < 1e-5 && lo.y.abs() < 1e-5, "{lo:?}");
        assert!(
            (hi.x - 1.0).abs() < 1e-5 && (hi.y - 1.0).abs() < 1e-5,
            "{hi:?}"
        );
        // A point and its shadow along the sun land on the same texel.
        let top = Vec3::new(96.0, 96.0, 20.0);
        let ground = top + sun * (20.0 / -sun.z);
        let (a, b) = (
            matrix.transform_point3(top),
            matrix.transform_point3(ground),
        );
        assert!((a.truncate() - b.truncate()).length() < 1e-5);
    }

    #[test]
    fn clip_space_matches_texture_space() {
        let window = ShadowWindow::fit(
            Vec3::new(0.0, 0.5, -1.0),
            [Vec3::ZERO, Vec3::new(100.0, 100.0, 0.0)],
        )
        .unwrap();
        let p = Vec3::new(30.0, 70.0, 4.0);
        let uv = window.texture_matrix().transform_point3(p);
        let clip = window.clip_matrix().transform_point3(p);
        assert!((clip.x - (uv.x * 2.0 - 1.0)).abs() < 1e-5);
        assert!((clip.y - (1.0 - uv.y * 2.0)).abs() < 1e-5);
    }

    #[test]
    fn small_sun_moves_keep_the_map() {
        let key = ShadowKey {
            blocks: vec![(1, 2)],
            casters: 10,
            sun: Vec3::new(0.0, 0.6, -0.8),
            strength: [0.5; 3],
        };
        let nudged = ShadowKey {
            sun: Vec3::new(0.005, 0.6, -0.8).normalize(),
            ..key.clone()
        };
        assert!(key.serves(&nudged));
        let moved = ShadowKey {
            sun: Vec3::new(0.1, 0.6, -0.8).normalize(),
            ..key.clone()
        };
        assert!(!key.serves(&moved));
        assert!(!key.serves(&ShadowKey {
            casters: 11,
            ..key.clone()
        }));
    }
}
