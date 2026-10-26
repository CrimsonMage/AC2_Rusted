//! Per-frame view state handed to the GPU backend: the camera and the region's sky lighting.
//! Everything is in world AC axes (x east, y north, z up).

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

/// Lighting and fog of one frame, as `LScapeLighting::UpdateLighting` hands them to
/// `LScapeTerrainMesh::IlluminateVertices` and the fog state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneLighting {
    /// `m_finalLightDir`: the direction light travels scaled by the direct brightness.
    pub light_dir: Vec3,
    /// `m_finalDirColor`.
    pub dir_color: [f32; 3],
    /// `m_finalDirBright`: the sun's diffuse intensity (`SceneTool::SetSunlight`).
    pub dir_brightness: f32,
    /// `GetAmbientLight`: ambient colour times ambient brightness.
    pub ambient: [f32; 3],
    pub fog_color: [f32; 3],
    /// Linear fog start and end distances in metres.
    pub fog_range: (f32, f32),
    /// `m_finalFogSky`: how far the sky fades to the fog colour (`LScapeSky::SetSkyFog`).
    pub fog_sky: f32,
    /// `WorldRender::m_fRainAmount` (`CDayDesc::GetRain`): 0 dry, 1 the heaviest rain.
    pub rain: f32,
    /// The rain's wind x and y (`PhysicsGlobals::SetWind`).
    pub rain_wind: [f32; 2],
}

impl Default for SceneLighting {
    /// `LScapeRender::SetDefaultLighting`: white light, ambient 0.7, direct 0.1 along
    /// `(0.6, 0, 0.8)`; `CDayDesc::GetWorldFog`'s fallback fog.
    fn default() -> Self {
        Self {
            light_dir: Vec3::new(0.6, 0.0, 0.8) * 0.1,
            dir_color: [1.0; 3],
            dir_brightness: 0.1,
            ambient: [0.7; 3],
            fog_color: [1.0; 3],
            fog_range: (0.0, 10_000.0),
            fog_sky: 0.0,
            rain: 0.0,
            rain_wind: [0.0; 2],
        }
    }
}

impl SceneLighting {
    /// From a `CDayDesc` sample (`GetLighting`, `GetWorldFog`), without region fog blocks.
    pub fn from_sky(sample: &SkyEnvironmentSample) -> Self {
        let rgb = |c: [f32; 4]| [c[0], c[1], c[2]];
        let fog = sample.world_fog();
        Self {
            light_dir: Vec3::from_array(sample.light_vector()) * sample.directional_brightness(),
            dir_color: rgb(sample.directional_color()),
            dir_brightness: sample.directional_brightness(),
            ambient: rgb(sample.ambient_color()).map(|c| c * sample.ambient_brightness()),
            fog_color: rgb(fog.color()),
            fog_range: (fog.min_distance(), fog.max_distance()),
            fog_sky: fog.sky(),
            rain: sample.rain().amount(),
            rain_wind: sample.rain().wind(),
        }
    }
}
