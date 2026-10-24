use super::SkyLightingDesc;
use crate::color::Rgba;

/// `CSkyDesc` update deadlines consumed by the SmartBox runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyEnvironmentCadence {
    tick_size_seconds: f64,
    light_tick_size_seconds: f64,
}

impl SkyEnvironmentCadence {
    pub const DEFAULT_TICK_SIZE_SECONDS: f64 = 3.0;
    pub const DEFAULT_LIGHT_TICK_SIZE_SECONDS: f64 = 20.0;

    pub(super) const fn new(tick_size_seconds: f64, light_tick_size_seconds: f64) -> Self {
        Self {
            tick_size_seconds,
            light_tick_size_seconds,
        }
    }

    pub const fn tick_size_seconds(self) -> f64 {
        self.tick_size_seconds
    }

    pub const fn light_tick_size_seconds(self) -> f64 {
        self.light_tick_size_seconds
    }
}

impl Default for SkyEnvironmentCadence {
    fn default() -> Self {
        Self::new(
            Self::DEFAULT_TICK_SIZE_SECONDS,
            Self::DEFAULT_LIGHT_TICK_SIZE_SECONDS,
        )
    }
}

/// `FogInfo`: linear world fog distances, sky fog amount and colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyFogDesc {
    pub(super) min_distance: f32,
    pub(super) max_distance: f32,
    pub(super) sky: f32,
    pub(super) color: Rgba,
}

impl SkyFogDesc {
    pub fn min_distance(&self) -> f32 {
        self.min_distance
    }
    pub fn max_distance(&self) -> f32 {
        self.max_distance
    }
    /// `FogInfo::fog_sky`: how far the sky dome is fogged.
    pub fn sky(&self) -> f32 {
        self.sky
    }
    pub fn color(&self) -> Rgba {
        self.color
    }
}

/// `CDayDesc::GetLighting` (`LightInfo` and light vector) and `CDayDesc::GetWorldFog` at one time
/// of day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyEnvironmentSample {
    pub(super) light: SkyLightingDesc,
    pub(super) light_vector: [f32; 3],
    pub(super) world_fog: SkyFogDesc,
}

impl SkyEnvironmentSample {
    pub fn ambient_brightness(&self) -> f32 {
        self.light.ambient_brightness
    }
    pub fn ambient_color(&self) -> Rgba {
        self.light.ambient_color
    }
    pub fn directional_brightness(&self) -> f32 {
        self.light.dir_brightness
    }
    pub fn directional_heading_degrees(&self) -> f32 {
        self.light.dir_heading
    }
    pub fn directional_pitch_degrees(&self) -> f32 {
        self.light.dir_pitch
    }
    pub fn directional_color(&self) -> Rgba {
        self.light.dir_color
    }
    pub fn shadow_color(&self) -> Rgba {
        self.light.shadow_color
    }
    /// The direction sunlight travels: `CDayDesc::CalcLightVector(1, heading, pitch)`, or
    /// `(1, 0, -0.4)` for a day without keys.
    pub fn light_vector(&self) -> [f32; 3] {
        self.light_vector
    }
    pub fn world_fog(&self) -> SkyFogDesc {
        self.world_fog
    }
}
