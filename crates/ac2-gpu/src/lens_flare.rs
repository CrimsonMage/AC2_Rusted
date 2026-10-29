//! `LensFlareEffect::Draw`: the sprites of each sky light's flare (`LScapeSky::Draw(1)`) and of
//! each drawn interior cell's light coronas (`CEnvCell::RenderLightCoronas`), drawn last in the
//! world pass, white at an alpha that falls off from the screen's centre, blended
//! SRCALPHA, ONE. Each sprite sits on the axis from the light through the screen centre at
//! `-axis_pos` times the light's position, `size` wide and square in pixels.
//!
//! Retail faded the flare by a pixel scan: a small triangle at the light, drawn after the world,
//! read back from the back buffer the next frame (`SceneTool::RequestPixelScan`,
//! `GetPixelScanResult`); while it showed, `m_fOpacity` rose by the elapsed time times
//! `m_fFadeSpeed`, otherwise it fell. Here a scan pass after the world pass reads the scene's
//! depth around the light (nothing nearer than the light; for a light at infinity the cleared far
//! depth, as the sky draws no depth) and steps the opacity towards the fraction it sees, on the
//! GPU, into one of two small textures; the next frame's sprites read it, a frame late as retail's
//! were. Each light keeps its slot in those textures while it is drawn, so it keeps its fade.

use std::cell::Cell;
use std::collections::HashMap;

use ac2_math::{Mat4, Vec3, Vec4};
use ac2_render::sky::LensFlare;
use ac2_render::{Camera, Image, TextureKey};

use crate::{Gpu, TextureCache, DEPTH_FORMAT};

/// Flares one frame draws: the sky's (the shipped region has one, the sun's) and the nearest
/// coronas in view.
const MAX_FLARES: usize = 64;
/// Sprites across them (the sun's flare has 8, a corona 1).
const MAX_POINTS: usize = 128;
/// How far a corona's light is moved towards the camera before its depth is compared (the lamp
/// around it does not hide it), in metres.
const CORONA_DEPTH_BIAS: f32 = 0.2;
const OPACITY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// The longest step the fade takes at once, in seconds (a stalled frame does not jump it).
const MAX_STEP: f64 = 0.25;

/// `Flares` in `lens_flare.wgsl`: the lights, the slots, the sprites, then the frame's values.
type Uniform = [[f32; 4]; 2 * MAX_FLARES + MAX_POINTS + 1];

/// Where a flare's light is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlareOrigin {
    /// A sky light at infinity, in this direction from the viewer
    /// (`LensFlareEffect::Draw(.., bInfiniteDistance = 1)`).
    Infinite(Vec3),
    /// A light at this place in the scene.
    Point(Vec3),
}

/// One light's flare for a frame: `key` names the light from frame to frame (it keeps its fade),
/// `start_opacity` is the opacity it starts from when it is new.
#[derive(Debug, Clone, Copy)]
pub struct FlareSource<'a> {
    pub key: u64,
    pub origin: FlareOrigin,
    pub start_opacity: f32,
    pub flare: &'a LensFlare,
}

/// The lens flares of the world's lights.
pub struct LensFlares {
    scan_pipeline: wgpu::RenderPipeline,
    sprite_pipeline: wgpu::RenderPipeline,
    scan_layout: wgpu::BindGroupLayout,
    sprite_layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    /// The two opacity textures the scans alternate between, their views, and the sprites' frame
    /// group reading each.
    #[cfg_attr(not(test), allow(dead_code))]
    opacity: [wgpu::Texture; 2],
    opacity_views: [wgpu::TextureView; 2],
    frame_groups: [wgpu::BindGroup; 2],
    /// Which of `opacity` the last scan wrote.
    current: Cell<usize>,
    sampler: wgpu::Sampler,
    textures: TextureCache,
    sprites: HashMap<TextureKey, wgpu::BindGroup>,
    /// The frame's sprites: their texture and their index in the uniform.
    draws: Vec<(TextureKey, u32)>,
    /// The slot each drawn light holds, by key.
    slots: HashMap<u64, usize>,
    flare_count: usize,
    last_time: Option<f64>,
    /// `Render::m_Prefs.LightCoronas`: no flare draws while it is off.
    enabled: bool,
}

impl LensFlares {
    /// Flares drawn into a target of `format` (the scene's HDR target).
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let uniform_entry = |binding, visibility| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture_entry = |binding, visibility, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let opacity_sample = wgpu::TextureSampleType::Float { filterable: false };
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 lens flare frame"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX),
                texture_entry(1, wgpu::ShaderStages::VERTEX, opacity_sample),
            ],
        });
        let scan_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 lens flare scan"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::FRAGMENT),
                texture_entry(1, wgpu::ShaderStages::FRAGMENT, opacity_sample),
                texture_entry(
                    2,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::TextureSampleType::Depth,
                ),
            ],
        });
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 lens flare sprite"),
            entries: &[
                texture_entry(
                    0,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::TextureSampleType::Float { filterable: true },
                ),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 lens flare"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}",
                    include_str!("shading.wgsl"),
                    include_str!("lens_flare.wgsl")
                )
                .into(),
            ),
        });
        let scan_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 lens flare scan"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ac2 lens flare scan"),
                    bind_group_layouts: &[Some(&scan_layout)],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_scan"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_scan"),
                compilation_options: Default::default(),
                targets: &[Some(OPACITY_FORMAT.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        // `DrawTexturedPolygon(..., BLEND_SRCALPHA, BLEND_ONE)` with no depth test; the target's
        // coverage alpha is kept.
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 lens flare sprites"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ac2 lens flare sprites"),
                    bind_group_layouts: &[Some(&frame_layout), Some(&sprite_layout)],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_sprite"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_sprite"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(additive),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
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
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 lens flare"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let opacity = [0, 1].map(|_| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ac2 lens flare opacity"),
                size: wgpu::Extent3d {
                    width: MAX_FLARES as u32,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OPACITY_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        });
        let opacity_views = opacity
            .each_ref()
            .map(|t| t.create_view(&Default::default()));
        let frame_groups = [0, 1].map(|index| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ac2 lens flare frame"),
                layout: &frame_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&opacity_views[index]),
                    },
                ],
            })
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 lens flare"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        Self {
            scan_pipeline,
            sprite_pipeline,
            scan_layout,
            sprite_layout,
            uniform,
            opacity,
            opacity_views,
            frame_groups,
            current: Cell::new(0),
            sampler,
            textures: TextureCache::new(device, &gpu.queue),
            sprites: HashMap::new(),
            draws: Vec::new(),
            slots: HashMap::new(),
            flare_count: 0,
            last_time: None,
            enabled: true,
        }
    }

    /// `Render::m_Prefs.LightCoronas`: whether flares draw, from the next `prepare`.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Writes the frame's flares seen by `camera` at client `time` (seconds): every infinite
    /// light's, and the nearest point lights' that are in the view, up to the slots there are;
    /// `load` hands over the sprites' images once the loader has decoded them (a sprite waits until
    /// then).
    pub fn prepare(
        &mut self,
        gpu: &Gpu,
        camera: &Camera,
        aspect: f32,
        time: f64,
        sources: &[FlareSource<'_>],
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) {
        let dt = self
            .last_time
            .map_or(0.0, |last| (time - last).clamp(0.0, MAX_STEP));
        self.last_time = Some(time);
        self.draws.clear();
        let view_projection = camera.view_projection(aspect);
        // (source, where it is on the screen, the depth it is seen at, distance)
        let mut chosen: Vec<(&FlareSource<'_>, [f32; 2], f32, f32)> = Vec::new();
        if self.enabled {
            for source in sources {
                match source.origin {
                    FlareOrigin::Infinite(direction) => {
                        let (ndc, in_view) = light_ndc(view_projection, direction);
                        chosen.push((source, ndc, if in_view { 1.0 } else { -1.0 }, 0.0));
                    }
                    FlareOrigin::Point(origin) => {
                        if let Some((ndc, depth)) = point_ndc(view_projection, camera.eye, origin) {
                            chosen.push((source, ndc, depth, camera.eye.distance(origin)));
                        }
                    }
                }
            }
        }
        chosen.sort_by(|a, b| a.3.total_cmp(&b.3));
        chosen.truncate(MAX_FLARES);
        // Lights drawn last frame keep their slots; new ones take the free slots.
        let kept: HashMap<u64, usize> = chosen
            .iter()
            .filter_map(|(source, ..)| Some((source.key, *self.slots.get(&source.key)?)))
            .collect();
        let mut free = (0..MAX_FLARES).filter(|slot| !kept.values().any(|s| s == slot));
        let mut slots = HashMap::with_capacity(chosen.len());
        let mut uniform: Uniform = [[0.0; 4]; 2 * MAX_FLARES + MAX_POINTS + 1];
        for (source, ndc, depth, _) in &chosen {
            let (slot, start) = match kept.get(&source.key) {
                Some(slot) => (*slot, -1.0),
                None => match free.next() {
                    Some(slot) => (slot, source.start_opacity.clamp(0.0, 1.0)),
                    None => continue,
                },
            };
            slots.insert(source.key, slot);
            let finite = matches!(source.origin, FlareOrigin::Point(_));
            uniform[slot] = [ndc[0], ndc[1], *depth, source.flare.fade_speed];
            uniform[MAX_FLARES + slot] = [start, if finite { 1.0 } else { 0.0 }, 1.0, 0.0];
            for point in source.flare.points.iter() {
                let index = self.draws.len();
                if index == MAX_POINTS {
                    break;
                }
                let Some(key) = point.texture.map(TextureKey::Texture) else {
                    continue;
                };
                if !self.sprites.contains_key(&key) {
                    let Some(image) = load(key) else {
                        continue;
                    };
                    let view = self
                        .textures
                        .get(&gpu.device, &gpu.queue, key, &mut |_| Some(image.clone()));
                    let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("ac2 lens flare sprite"),
                        layout: &self.sprite_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    });
                    self.sprites.insert(key, group);
                }
                uniform[2 * MAX_FLARES + index] = [point.axis_pos, point.size, slot as f32, 0.0];
                self.draws.push((key, index as u32));
            }
        }
        self.flare_count = slots.len();
        self.slots = slots;
        uniform[2 * MAX_FLARES + MAX_POINTS] = [dt as f32, aspect, 0.0, 0.0];
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::cast_slice(&uniform));
    }

    /// Records the sprites into the world pass, at the opacity the last scan left.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.draws.is_empty() {
            return;
        }
        pass.set_pipeline(&self.sprite_pipeline);
        pass.set_bind_group(0, &self.frame_groups[self.current.get()], &[]);
        for (key, slot) in &self.draws {
            if let Some(group) = self.sprites.get(key) {
                pass.set_bind_group(1, group, &[]);
                pass.draw(0..4, *slot..*slot + 1);
            }
        }
    }

    /// The scan after the world pass: steps each flare's opacity towards how much sky `depth`
    /// (the world's depth, before anything clears it) shows around its light.
    pub fn scan(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, depth: &wgpu::TextureView) {
        if self.flare_count == 0 {
            return;
        }
        let read = self.current.get();
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 lens flare scan"),
            layout: &self.scan_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.opacity_views[read]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth),
                },
            ],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ac2 lens flare scan"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.opacity_views[1 - read],
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.scan_pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.current.set(1 - read);
    }
}

/// Where a light at `origin` falls in normalised device coordinates and the depth it is seen at
/// (moved `CORONA_DEPTH_BIAS` towards the camera), or `None` when it is behind the camera or off
/// the screen (no scan is asked for it and its sprite would not show).
fn point_ndc(view_projection: Mat4, eye: Vec3, origin: Vec3) -> Option<([f32; 2], f32)> {
    let clip = view_projection * origin.extend(1.0);
    if clip.w <= 1e-4 {
        return None;
    }
    let ndc = [clip.x / clip.w, clip.y / clip.w];
    if ndc.iter().any(|c| c.abs() > 1.0) {
        return None;
    }
    let toward = eye - origin;
    let bias = CORONA_DEPTH_BIAS.min(toward.length() * 0.5);
    let near = view_projection * (origin + toward.normalize_or_zero() * bias).extend(1.0);
    let depth = if near.w > 1e-4 {
        near.z / near.w
    } else {
        clip.z / clip.w
    };
    Some((ndc, depth.clamp(0.0, 1.0)))
}

/// Where a light at infinity in `direction` falls in normalised device coordinates, and whether
/// it is in the view (`SceneTool::TransformVector`'s result): in front of the camera and on the
/// screen. A light behind the camera is pushed off the screen on its side.
fn light_ndc(view_projection: Mat4, direction: Vec3) -> ([f32; 2], bool) {
    let clip = view_projection * Vec4::new(direction.x, direction.y, direction.z, 0.0);
    if clip.w <= 1e-6 {
        let side = clip.truncate().truncate().normalize_or(ac2_math::Vec2::X) * 4.0;
        return ([side.x, side.y], false);
    }
    let ndc = [clip.x / clip.w, clip.y / clip.w];
    let on_screen = ndc.iter().all(|c| c.abs() <= 1.0);
    (ndc, on_screen)
}

#[cfg(test)]
#[path = "lens_flare_tests.rs"]
mod tests;
