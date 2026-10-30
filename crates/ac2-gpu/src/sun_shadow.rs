//! Sun shadows of bodies and objects: cascaded depth maps looking down the sun, which the near
//! landscape and the objects sample with percentage-closer filtering.
//!
//! Retail shadowed moving objects by `DynamicObjectShadows`: a dark disc decal under each one
//! (`WorldRender::CreateDiscShadows`), or stencil shadow volumes along `SceneTool::m_vShadowDirection`
//! (`SceneTool::GenerateGlobalShadowVolume`, at most 100) that `RenderGlobalVolumetricShadows`
//! darkened with a screen quad of `m_cVolumetricShadowStrength` (the day's shadow colour, blended
//! ZERO/INVSRCCOLOR: a shadowed pixel keeps one minus the strength). Static objects shadowed only
//! the terrain, through the 512 x 512 landscape map (`crate::StaticShadowMap`).
//!
//! Here the sun casts from every near static object and body into one depth array of
//! [`SunShadowTier::cascades`] layers (`DynamicObjectShadows` sizes it), and every receiver near
//! the camera (terrain, objects, bodies themselves) keeps `mix(1 - strength, 1, lit)` of its
//! colour, where `strength` is the static shadow strength the landscape map uses
//! (`SceneTool::m_cStaticShadowStrength`), so the near cascades and the far landscape map agree.

use ac2_math::{Mat4, Vec3, Vec4};
use ac2_render::prefs::ObjectShadows;
use ac2_render::Camera;
use bytemuck::{Pod, Zeroable};

use crate::Gpu;

/// The depth format of the cascades.
pub const SUN_SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth16Unorm;
/// Most cascades a tier uses (the receiver uniform's array).
pub const MAX_CASCADES: usize = 4;
/// How far towards the sun past a cascade's sphere casters still reach it, metres (hills and
/// towers between the sun and the view).
const CASTER_REACH: f32 = 300.0;
/// Far fraction of the last cascade over which the shadows fade out.
const FADE: f32 = 0.15;

/// One `DynamicObjectShadows` level's maps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunShadowTier {
    /// Edge of each cascade's map in texels.
    pub size: u32,
    /// Far distance of each cascade from the eye, metres, nearest first.
    pub splits: &'static [f32],
}

impl SunShadowTier {
    /// The tier of a preference level; `None` draws no sun shadows.
    pub fn of(level: ObjectShadows) -> Option<Self> {
        match level {
            ObjectShadows::Off => None,
            ObjectShadows::Low => Some(Self {
                size: 1024,
                splits: &[16.0, 60.0],
            }),
            ObjectShadows::Medium => Some(Self {
                size: 2048,
                splits: &[20.0, 80.0],
            }),
            ObjectShadows::High => Some(Self {
                size: 2048,
                splits: &[16.0, 50.0, 150.0],
            }),
        }
    }

    pub fn cascades(&self) -> usize {
        self.splits.len()
    }

    /// The one map of a UI viewport's scene (`UIScene::Render`'s volumetric shadows): retail drew
    /// them only from `DynamicObjectShadows` 2 up, so Off and Low draw none. The split is the
    /// placeholder of the cascade's real one, which [`fit_viewport_cascade`] gives per frame.
    pub fn viewport(level: ObjectShadows) -> Option<Self> {
        match level {
            ObjectShadows::Off | ObjectShadows::Low => None,
            ObjectShadows::Medium | ObjectShadows::High => Some(Self {
                size: 1024,
                splits: &[0.0],
            }),
        }
    }
}

/// One cascade: the light-space window over a sphere around a slice of the view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cascade {
    /// World to the cascade's clip space (x, y in -1..1 over the window, depth 0..1 from the sun).
    pub clip: Mat4,
    /// The same from points taken off the eye `fit_cascades` saw: casters and receivers both use
    /// this one, so their depths agree without the world's large coordinates rounding in f32
    /// (`AGENTS.md`: objects and bodies draw eye-relative).
    pub clip_rel: Mat4,
    /// The light-space axes (right, up, along the sun) and the window's centre along them.
    pub axes: [Vec3; 3],
    pub center: Vec3,
    pub radius: f32,
}

impl Cascade {
    /// Whether a sphere at `center` of `radius` can shadow anything in the cascade.
    pub fn reaches(&self, center: Vec3, radius: f32) -> bool {
        let at = Vec3::new(
            center.dot(self.axes[0]),
            center.dot(self.axes[1]),
            center.dot(self.axes[2]),
        ) - self.center;
        let reach = self.radius + radius;
        at.x.abs() <= reach
            && at.y.abs() <= reach
            && at.z <= reach
            && at.z >= -(reach + CASTER_REACH)
    }

    /// Metres per texel of a map `size` texels wide.
    pub fn texel(&self, size: u32) -> f32 {
        2.0 * self.radius / size as f32
    }
}

/// The light-space axes looking down `sun` (the direction light travels), as
/// `Matrix4::DirectionMatrix(m_finalLightDir, +Z)` builds them for the landscape map.
fn light_axes(sun: Vec3) -> Option<[Vec3; 3]> {
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
    Some([right, right.cross(forward), forward])
}

/// The cascade of a `size` texel map over the sphere at `center` of `radius`, lit along `axes`
/// (`light_axes`), taken off `eye`.
fn cascade_around(eye: Vec3, center: Vec3, radius: f32, axes: [Vec3; 3], size: u32) -> Cascade {
    // Whole metres, so the window keeps its size as the view turns.
    let radius = radius.ceil().max(1.0);
    let texel = 2.0 * radius / size as f32;
    let snap = |v: f32| (v / texel).floor() * texel;
    let light = Vec3::new(
        snap(center.dot(axes[0])),
        snap(center.dot(axes[1])),
        center.dot(axes[2]),
    );
    let depth_near = light.z - radius - CASTER_REACH;
    let depth_range = 2.0 * radius + CASTER_REACH;
    let row = |axis: Vec3, scale: f32, offset: f32| {
        Vec4::new(axis.x * scale, axis.y * scale, axis.z * scale, offset)
    };
    let clip = Mat4::from_cols(
        row(axes[0], 1.0 / radius, -light.x / radius),
        row(axes[1], 1.0 / radius, -light.y / radius),
        row(axes[2], 1.0 / depth_range, -depth_near / depth_range),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    )
    .transpose();
    // The window's offsets with the eye's light-space position taken off in f64, where the
    // difference of two coordinates tens of kilometres out keeps its millimetres.
    let off = |axis: Vec3, at: f32| {
        let eye = f64::from(eye.x) * f64::from(axis.x)
            + f64::from(eye.y) * f64::from(axis.y)
            + f64::from(eye.z) * f64::from(axis.z);
        (f64::from(at) - eye) as f32
    };
    let clip_rel = Mat4::from_cols(
        row(axes[0], 1.0 / radius, -off(axes[0], light.x) / radius),
        row(axes[1], 1.0 / radius, -off(axes[1], light.y) / radius),
        row(
            axes[2],
            1.0 / depth_range,
            -off(axes[2], depth_near) / depth_range,
        ),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    )
    .transpose();
    Cascade {
        clip,
        clip_rel,
        axes,
        center: light,
        radius,
    }
}

/// The one cascade of a UI viewport's scene: the sphere around its objects (`center`, `radius`),
/// seen from `eye` (an orthographic camera sits 10 km back, so the window is fitted to the
/// objects, not to the view) and lit along `sun`. Also the cascade's far distance from the eye
/// for the receivers: past the sphere and its fade, so every point of it is fully covered.
pub fn fit_viewport_cascade(
    eye: Vec3,
    (center, radius): (Vec3, f32),
    sun: Vec3,
    tier: &SunShadowTier,
) -> Option<(Cascade, f32)> {
    let axes = light_axes(sun)?;
    let cascade = cascade_around(eye, center, radius, axes, tier.size);
    let split = (center.distance(eye) + cascade.radius) / (1.0 - FADE) + 1.0;
    Some((cascade, split))
}

/// The cascades of `tier` for `camera` at `aspect` lit along `sun`: each the sphere around its
/// slice of the view (stable as the camera turns), its centre snapped to whole texels so the map
/// does not shimmer as the camera moves.
pub fn fit_cascades(
    camera: &Camera,
    aspect: f32,
    sun: Vec3,
    tier: &SunShadowTier,
) -> Option<Vec<Cascade>> {
    let axes = light_axes(sun)?;
    let forward = (camera.target - camera.eye).normalize_or_zero();
    if forward == Vec3::ZERO {
        return None;
    }
    let side = forward.cross(Vec3::Z);
    let side = if side.length() < 1e-4 {
        Vec3::X
    } else {
        side.normalize()
    };
    let up = side.cross(forward);
    let tan_y = (camera.fov_y * 0.5).tan();
    let tan_x = tan_y * aspect.max(1e-3);
    let mut near = camera.near.max(0.0);
    let mut cascades = Vec::with_capacity(tier.cascades());
    for &far in tier.splits {
        // The slice's eight corners.
        let corners = [near, far].into_iter().flat_map(|d| {
            [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sy)| {
                camera.eye + forward * d + side * (sx * tan_x * d) + up * (sy * tan_y * d)
            })
        });
        let corners: Vec<Vec3> = corners.collect();
        let center = corners.iter().copied().sum::<Vec3>() / corners.len() as f32;
        let radius = corners
            .iter()
            .map(|c| c.distance(center))
            .fold(0.0_f32, f32::max);
        cascades.push(cascade_around(camera.eye, center, radius, axes, tier.size));
        near = far;
    }
    Some(cascades)
}

/// What the receivers read (`sun_shadow.wgsl`'s `SunShadow`).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SunShadowUniform {
    cascades: [[[f32; 4]; 4]; MAX_CASCADES],
    /// Far distance of each cascade from the eye.
    splits: [f32; 4],
    /// Metres per texel of each cascade (the receiver's normal offset).
    texels: [f32; 4],
    /// The shadow strength (rgb); w is 1 / map size.
    strength: [f32; 4],
    /// The eye the receivers' points are taken off (xyz); w is the cascade count (0: no sun
    /// shadows).
    eye: [f32; 4],
}

impl SunShadowUniform {
    const OFF: Self = Self {
        cascades: [[[0.0; 4]; 4]; MAX_CASCADES],
        splits: [0.0; 4],
        texels: [0.0; 4],
        strength: [0.0; 4],
        eye: [0.0; 4],
    };
}

/// The bindings a receiver's frame group takes: the uniform, the depth array and its comparison
/// sampler. Handles are shared, so the scene's map and every receiver's copy are one resource.
#[derive(Clone)]
pub struct SunShadowBinding {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

impl SunShadowBinding {
    /// An empty binding (no cascades) for a renderer that draws without the scene's map.
    pub fn placeholder(gpu: &Gpu) -> Self {
        let (_, view) = depth_array(gpu, 1, 1);
        Self::with_view(gpu, view)
    }

    fn with_view(gpu: &Gpu, view: wgpu::TextureView) -> Self {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 sun shadow uniform"),
            size: std::mem::size_of::<SunShadowUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&buffer, 0, bytemuck::bytes_of(&SunShadowUniform::OFF));
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 sun shadow"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        Self {
            buffer,
            view,
            sampler,
        }
    }

    /// The three layout entries from `first` (uniform, depth array, comparison sampler), seen by
    /// the fragment stage.
    pub(crate) fn layout_entries(first: u32) -> [wgpu::BindGroupLayoutEntry; 3] {
        let visibility = wgpu::ShaderStages::FRAGMENT;
        [
            wgpu::BindGroupLayoutEntry {
                binding: first,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: first + 1,
                visibility,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: first + 2,
                visibility,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ]
    }

    /// The three group entries from `first`, matching [`Self::layout_entries`].
    pub(crate) fn entries(&self, first: u32) -> [wgpu::BindGroupEntry<'_>; 3] {
        [
            wgpu::BindGroupEntry {
                binding: first,
                resource: self.buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: first + 1,
                resource: wgpu::BindingResource::TextureView(&self.view),
            },
            wgpu::BindGroupEntry {
                binding: first + 2,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

fn depth_array(gpu: &Gpu, size: u32, layers: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ac2 sun shadow cascades"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SUN_SHADOW_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    (texture, view)
}

/// The scene's cascades: the depth array for the current tier and the receivers' binding.
pub struct SunShadows {
    tier: Option<SunShadowTier>,
    texture: Option<wgpu::Texture>,
    layers: Vec<wgpu::TextureView>,
    binding: SunShadowBinding,
}

impl SunShadows {
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            tier: None,
            texture: None,
            layers: Vec::new(),
            binding: SunShadowBinding::placeholder(gpu),
        }
    }

    /// The receivers' binding; it changes when [`Self::set_tier`] remakes the maps.
    pub fn binding(&self) -> &SunShadowBinding {
        &self.binding
    }

    pub fn tier(&self) -> Option<SunShadowTier> {
        self.tier
    }

    /// Sizes the maps for `tier` (`None`: no maps); returns whether the binding changed.
    pub fn set_tier(&mut self, gpu: &Gpu, tier: Option<SunShadowTier>) -> bool {
        if tier == self.tier {
            return false;
        }
        self.tier = tier;
        match tier {
            Some(tier) => {
                let layers = tier.cascades() as u32;
                let (texture, view) = depth_array(gpu, tier.size, layers);
                self.layers = (0..layers)
                    .map(|layer| {
                        texture.create_view(&wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            base_array_layer: layer,
                            array_layer_count: Some(1),
                            ..Default::default()
                        })
                    })
                    .collect();
                self.texture = Some(texture);
                self.binding = SunShadowBinding::with_view(gpu, view);
            }
            None => {
                self.texture = None;
                self.layers.clear();
                self.binding = SunShadowBinding::placeholder(gpu);
            }
        }
        true
    }

    /// One view per cascade, for the caster passes.
    pub(crate) fn layer_views(&self) -> &[wgpu::TextureView] {
        &self.layers
    }

    /// Writes the receivers' uniform for `cascades` fitted around `eye` at `strength`; no cascades
    /// turns the shadows off. The receivers pass points relative to `eye` (`Cascade::clip_rel`).
    pub(crate) fn write(
        &self,
        gpu: &Gpu,
        eye: Vec3,
        (cascades, splits): (&[Cascade], &[f32]),
        strength: [f32; 3],
    ) {
        let mut uniform = SunShadowUniform::OFF;
        if let Some(tier) = self.tier.filter(|_| !cascades.is_empty()) {
            for (index, cascade) in cascades.iter().take(MAX_CASCADES).enumerate() {
                uniform.cascades[index] = cascade.clip_rel.to_cols_array_2d();
                uniform.splits[index] = splits.get(index).copied().unwrap_or(f32::MAX);
                uniform.texels[index] = cascade.texel(tier.size);
            }
            uniform.strength = [
                strength[0],
                strength[1],
                strength[2],
                1.0 / tier.size as f32,
            ];
            uniform.eye = [eye.x, eye.y, eye.z, cascades.len().min(MAX_CASCADES) as f32];
        }
        gpu.queue
            .write_buffer(&self.binding.buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

/// The shared receiver functions; each shader declares `sun_shadow`, `sun_shadow_map` and
/// `sun_shadow_sampler` at its own bindings.
pub(crate) const RECEIVER_WGSL: &str = include_str!("sun_shadow.wgsl");

/// `FADE` as the shader reads it.
pub(crate) fn receiver_source() -> String {
    RECEIVER_WGSL.replace("SUN_SHADOW_FADE", &format!("{FADE:?}"))
}

#[cfg(test)]
#[path = "sun_shadow_far_tests.rs"]
mod far_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Camera {
        Camera {
            eye: Vec3::new(100.0, 100.0, 50.0),
            target: Vec3::new(100.0, 110.0, 48.0),
            fov_y: 1.0,
            near: 0.5,
            far: 2000.0,
        }
    }

    #[test]
    fn a_cascade_maps_its_slice_inside_the_window() {
        let tier = SunShadowTier::of(ObjectShadows::Medium).unwrap();
        let sun = Vec3::new(0.3, 0.4, -0.8);
        let cascades = fit_cascades(&camera(), 16.0 / 9.0, sun, &tier).unwrap();
        assert_eq!(cascades.len(), 2);
        // A point 10 m ahead in the first cascade, 50 m ahead in the second.
        for (cascade, distance) in cascades.iter().zip([10.0, 50.0]) {
            let p = camera().eye + Vec3::new(0.0, distance, -2.0);
            let clip = cascade.clip.transform_point3(p);
            assert!(clip.x.abs() < 1.0 && clip.y.abs() < 1.0, "{clip:?}");
            assert!(clip.z > 0.0 && clip.z < 1.0, "{clip:?}");
            // Nearer the sun is nearer in depth.
            let toward = cascade.clip.transform_point3(p - sun.normalize() * 5.0);
            assert!(toward.z < clip.z);
        }
    }

    #[test]
    fn a_point_and_its_shadow_share_a_texel() {
        let tier = SunShadowTier::of(ObjectShadows::High).unwrap();
        let sun = Vec3::new(-0.2, 0.5, -0.7);
        let cascades = fit_cascades(&camera(), 1.5, sun, &tier).unwrap();
        let top = camera().eye + Vec3::new(2.0, 12.0, 5.0);
        let ground = top + sun * (5.0 / -sun.z);
        let (a, b) = (
            cascades[0].clip.transform_point3(top),
            cascades[0].clip.transform_point3(ground),
        );
        assert!((a.truncate() - b.truncate()).length() < 1e-4);
    }

    #[test]
    fn small_camera_moves_keep_the_window_on_whole_texels() {
        let tier = SunShadowTier::of(ObjectShadows::Low).unwrap();
        let sun = Vec3::new(0.3, 0.4, -0.8);
        let a = fit_cascades(&camera(), 1.5, sun, &tier).unwrap();
        let mut moved = camera();
        moved.eye.x += 0.013;
        moved.target.x += 0.013;
        let b = fit_cascades(&moved, 1.5, sun, &tier).unwrap();
        let texel = a[0].texel(tier.size);
        let shift = (b[0].center - a[0].center).truncate() / texel;
        assert!((shift.x - shift.x.round()).abs() < 1e-3, "{shift:?}");
        assert!((shift.y - shift.y.round()).abs() < 1e-3, "{shift:?}");
    }

    #[test]
    fn casters_outside_the_window_are_left_out() {
        let tier = SunShadowTier::of(ObjectShadows::Medium).unwrap();
        let sun = Vec3::new(0.0, 0.0, -1.0);
        let cascades = fit_cascades(&camera(), 1.5, sun, &tier).unwrap();
        let near = &cascades[0];
        assert!(near.reaches(camera().eye + Vec3::new(0.0, 8.0, -2.0), 1.0));
        assert!(!near.reaches(camera().eye + Vec3::new(500.0, 8.0, -2.0), 1.0));
        // Straight up the sun's path, far above: still casts.
        assert!(near.reaches(camera().eye + Vec3::new(0.0, 8.0, 150.0), 1.0));
    }

    #[test]
    fn far_from_the_origin_the_relative_window_matches_the_absolute_one() {
        let tier = SunShadowTier::of(ObjectShadows::High).unwrap();
        let sun = Vec3::new(-0.2, 0.5, -0.7);
        let at = |eye: Vec3| {
            let camera = Camera {
                eye,
                target: eye + Vec3::new(0.0, 10.0, -2.0),
                ..camera()
            };
            fit_cascades(&camera, 1.5, sun, &tier).unwrap()
        };
        // Near the origin both matrices place a point alike.
        let eye = camera().eye;
        let near = at(eye);
        let offset = Vec3::new(1.5, 9.0, -1.8);
        let (a, b) = (
            near[0].clip.transform_point3(eye + offset),
            near[0].clip_rel.transform_point3(offset),
        );
        assert!((a - b).length() < 1e-5, "{a:?} {b:?}");
        // 24 km out, a millimetre's move of a point moves it in the relative window by that
        // millimetre, where the absolute matrix rounds it away.
        let far_eye = Vec3::new(24_123.4, 23_987.6, 120.0);
        let far = at(far_eye);
        let step = Vec3::new(0.001, 0.0, 0.0);
        let moved = far[0].clip_rel.transform_point3(offset + step)
            - far[0].clip_rel.transform_point3(offset);
        let texel = far[0].texel(tier.size);
        let expected = step.dot(far[0].axes[0]) / far[0].radius;
        assert!((moved.x - expected).abs() < 0.01 * texel / far[0].radius);
    }

    #[test]
    fn the_preference_levels_size_the_maps() {
        assert_eq!(SunShadowTier::of(ObjectShadows::Off), None);
        let low = SunShadowTier::of(ObjectShadows::Low).unwrap();
        let high = SunShadowTier::of(ObjectShadows::High).unwrap();
        assert!(low.size < high.size && low.cascades() <= high.cascades());
        assert!(high.cascades() <= MAX_CASCADES);
    }
}
