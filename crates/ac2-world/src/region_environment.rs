use ac2_core::ColorArgb;

use super::SkyLightingDesc;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyFogDesc {
    min_distance: f32,
    max_distance: f32,
    color: ColorArgb,
}

impl SkyFogDesc {
    pub fn min_distance(&self) -> f32 {
        self.min_distance
    }
    pub fn max_distance(&self) -> f32 {
        self.max_distance
    }
    pub fn color(&self) -> ColorArgb {
        self.color
    }
    pub(super) fn new(min_distance: f32, max_distance: f32, color: ColorArgb) -> Self {
        Self {
            min_distance,
            max_distance,
            color,
        }
    }
    pub(super) fn interpolate(self, after: Self, ratio: f32) -> Self {
        Self::new(
            super::lerp(self.min_distance, after.min_distance, ratio),
            super::lerp(self.max_distance, after.max_distance, ratio),
            super::lerp_color(self.color, after.color, ratio),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyEnvironmentSample {
    pub(super) ambient_brightness: f32,
    pub(super) ambient_color: ColorArgb,
    pub(super) directional_brightness: f32,
    pub(super) directional_heading_degrees: f32,
    pub(super) directional_pitch_degrees: f32,
    pub(super) directional_color: ColorArgb,
    pub(super) world_fog: Option<SkyFogDesc>,
}

impl SkyEnvironmentSample {
    pub fn ambient_brightness(&self) -> f32 {
        self.ambient_brightness
    }
    pub fn ambient_color(&self) -> ColorArgb {
        self.ambient_color
    }
    pub fn directional_brightness(&self) -> f32 {
        self.directional_brightness
    }
    pub fn directional_heading_degrees(&self) -> f32 {
        self.directional_heading_degrees
    }
    pub fn directional_pitch_degrees(&self) -> f32 {
        self.directional_pitch_degrees
    }
    pub fn directional_color(&self) -> ColorArgb {
        self.directional_color
    }
    pub fn world_fog(&self) -> Option<SkyFogDesc> {
        self.world_fog
    }
    pub fn lighting(&self) -> SkyLightingDesc {
        SkyLightingDesc {
            dir_brightness: self.directional_brightness,
            dir_heading: self.directional_heading_degrees.to_radians(),
            dir_pitch: self.directional_pitch_degrees.to_radians(),
            dir_color: self.directional_color,
            ambient_brightness: self.ambient_brightness,
            ambient_color: self.ambient_color,
        }
    }
}
