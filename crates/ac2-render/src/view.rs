//! Per-frame view state handed to the GPU backend: the camera and the region's sky lighting.
//! Everything is in world AC axes (x east, y north, z up).

use ac2_core::ColorArgb;
use ac2_math::{camera, Mat4, Vec3};
use ac2_world::SkyEnvironmentSample;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    /// Vertical field of view in radians.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    /// Clip-space transform (wgpu depth range 0..1).
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let view = camera::look_at_mat4(self.eye, self.target, Vec3::Z);
        camera::perspective(self.fov_y, aspect.max(1e-3), self.near, self.far) * view
    }
}

/// Lighting and fog of one frame (`LScapeTerrainMesh::IlluminateVertices` inputs and the
/// region fog).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneLighting {
    /// Unit vector towards the sun.
    pub sun_direction: Vec3,
    /// Sun colour times its brightness.
    pub sun: [f32; 3],
    /// Ambient colour times its brightness.
    pub ambient: [f32; 3],
    pub fog_color: [f32; 3],
    /// Fog start and full distances in metres; `None` disables fog.
    pub fog_range: Option<(f32, f32)>,
}

impl Default for SceneLighting {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::new(0.3, 0.4, 0.866).normalize(),
            sun: [0.7; 3],
            ambient: [0.4; 3],
            fog_color: [0.6, 0.7, 0.8],
            fog_range: None,
        }
    }
}

impl SceneLighting {
    /// From a `CRegionDesc` sky sample: heading turns from north towards east, pitch rises from
    /// the horizon.
    pub fn from_sky(sample: &SkyEnvironmentSample) -> Self {
        let (sin_h, cos_h) = sample.directional_heading_degrees().to_radians().sin_cos();
        let (sin_p, cos_p) = sample.directional_pitch_degrees().to_radians().sin_cos();
        let scaled = |color: ColorArgb, brightness: f32| rgb(color).map(|c| c * brightness);
        let fog = sample.world_fog();
        Self {
            sun_direction: Vec3::new(sin_h * cos_p, cos_h * cos_p, sin_p).normalize_or_zero(),
            sun: scaled(sample.directional_color(), sample.directional_brightness()),
            ambient: scaled(sample.ambient_color(), sample.ambient_brightness()),
            fog_color: fog.map_or([0.6, 0.7, 0.8], |fog| rgb(fog.color())),
            fog_range: fog.map(|fog| (fog.min_distance(), fog.max_distance())),
        }
    }
}

fn rgb(color: ColorArgb) -> [f32; 3] {
    [color.red, color.green, color.blue].map(|c| f32::from(c) / 255.0)
}
