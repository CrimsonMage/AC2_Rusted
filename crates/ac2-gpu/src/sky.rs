//! The sky around the DAT sky objects (`sky.wgsl`): a single-scattering atmosphere lit by the
//! region's sun and graded by the backdrop dome's tint, drawn first; and a procedural cloud deck
//! drawn after the before-world objects and `LScapeSky::Draw`'s fog quad, its cover thickening
//! with the region's rain (`CDayDesc::GetRain`). Retail drew only the DAT objects and the fog
//! quad over a black clear.

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
    wind: [f32; 4],
}

/// What the frame's sky shows beyond the region's lighting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyLook {
    /// The backdrop dome's tint and opacity (`crate::sky_backdrop`), sRGB as authored; `None`
    /// when no dome draws.
    pub backdrop: Option<[f32; 4]>,
    /// The cloud deck's cover on a dry day, 0 clear to 1 overcast; rain closes it
    /// ([`cloud_cover`]).
    pub cloud_cover: f32,
    /// Client seconds; the clouds drift with it.
    pub time: f64,
    /// The deck's drift in m/s ([`cloud_wind`]).
    pub wind: [f32; 2],
}

impl Default for SkyLook {
    fn default() -> Self {
        Self {
            backdrop: None,
            cloud_cover: DEFAULT_CLOUD_COVER,
            time: 0.0,
            wind: DEFAULT_CLOUD_WIND,
        }
    }
}

/// The cloud cover of a dry day.
pub const DEFAULT_CLOUD_COVER: f32 = 0.25;

/// The deck's drift before the region's cloud shadows load, m/s.
pub const DEFAULT_CLOUD_WIND: [f32; 2] = [9.0, 3.5];

/// The ground speed of the region's cloud shadows (`CSurfaceDesc::m_fCloudScrollU`/`V` texture
/// units per second at `m_fCloudTextureScale` units per metre): a texture offset falling by the
/// scroll carries the pattern the other way.
pub fn cloud_wind(clouds: &ac2_render::scene::CloudShadow) -> [f32; 2] {
    if clouds.scale > 0.0 {
        clouds.scroll.map(|rate| -rate / clouds.scale)
    } else {
        DEFAULT_CLOUD_WIND
    }
}

/// The cover over `dry` cover in `rain` (`WorldRender::m_fRainAmount`): overcast by the time
/// the rain is heavy enough to draw (`RAIN_DRAW_THRESHOLD`, 0.4).
pub fn cloud_cover(dry: f32, rain: f32) -> f32 {
    let t = (rain / 0.45).clamp(0.0, 1.0);
    let wet = t * t * (3.0 - 2.0 * t);
    dry + (1.0 - dry) * wet
}

/// The cloud deck's uniform: x cover (rain closing it), y drift seconds (wrapping once a day
/// so the noise keeps its precision in f32), z rain.
pub(crate) fn cloud_uniform(look: &SkyLook, lighting: &SceneLighting) -> [f32; 4] {
    [
        cloud_cover(look.cloud_cover, lighting.rain),
        (look.time % 86_400.0) as f32,
        lighting.rain.clamp(0.0, 1.0),
        0.0,
    ]
}

/// The full-screen atmosphere and cloud passes, their colours set each frame.
pub struct SkyRenderer {
    pipeline: wgpu::RenderPipeline,
    clouds: wgpu::RenderPipeline,
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
        let pipeline = |entry_point, blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
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
            })
        };
        Self {
            pipeline: pipeline("fs", None),
            clouds: pipeline(
                "fs_clouds",
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            ),
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
            // The fog colour, and `fog_sky` (how far the sky is fogged out) in alpha.
            fog: {
                let c = lighting.fog_color.map(|c| c.clamp(0.0, 1.0));
                [c[0], c[1], c[2], lighting.fog_sky.clamp(0.0, 1.0)]
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
            clouds: cloud_uniform(look, lighting),
            // z the dome's opacity: the atmosphere stands in for the dome, so it fades with it.
            wind: [
                look.wind[0],
                look.wind[1],
                look.backdrop.map_or(1.0, |[.., a]| a.clamp(0.0, 1.0)),
                0.0,
            ],
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// The atmosphere, first in the frame.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.draw(0..3, 0..1);
    }

    /// The cloud deck, after the before-world sky objects and the fog quad.
    pub fn draw_clouds(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.clouds);
        pass.set_bind_group(0, &self.group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn inverse_view_projection(camera: &Camera, aspect: f32) -> Mat4 {
    camera.view_projection(aspect).inverse()
}

#[cfg(test)]
mod tests {
    use super::{cloud_cover, cloud_wind};
    use ac2_render::scene::CloudShadow;

    #[test]
    fn the_deck_drifts_with_the_regions_cloud_shadows() {
        let clouds = CloudShadow {
            texture: None,
            scale: 0.001,
            scroll: [-0.006, -0.002],
        };
        let [x, y] = cloud_wind(&clouds);
        assert!((x - 6.0).abs() < 1e-4 && (y - 2.0).abs() < 1e-4);
    }

    #[test]
    fn rain_closes_the_cover_by_the_time_it_draws() {
        assert_eq!(cloud_cover(0.25, 0.0), 0.25);
        assert!(cloud_cover(0.25, 0.2) > 0.4);
        assert_eq!(cloud_cover(0.25, 0.45), 1.0);
        assert_eq!(cloud_cover(0.25, 1.0), 1.0);
    }
}
