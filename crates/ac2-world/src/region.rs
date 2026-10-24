//! The Asheron's Call 2 region environment: `CRegionDesc`'s sky (`CSkyDesc` days of
//! `SkyTimeOfDay` lighting and fog keys) and its `GameTime` calendar.

use std::sync::Arc;

use ac2_core::{ColorArgb, QualifiedDataId};

#[path = "region/ac2_region.rs"]
mod ac2_region;
#[path = "region_environment.rs"]
mod environment;
pub(crate) use ac2_region::{ac2_region_water_desc, load_ac2_region};
#[path = "region/game_time.rs"]
mod game_time;
pub use environment::{SkyEnvironmentCadence, SkyEnvironmentSample, SkyFogDesc};
use game_time::GameTime;

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDesc {
    qdid: QualifiedDataId,
    region_number: u32,
    game_time: GameTime,
    sky: Option<Arc<SkyDesc>>,
}

impl RegionDesc {
    pub fn qdid(&self) -> QualifiedDataId {
        self.qdid
    }

    pub fn region_number(&self) -> u32 {
        self.region_number
    }

    /// The calendar date and time-of-day names at `server_time`, as two lines.
    pub fn date_time_text(&self, server_time: f64) -> Option<String> {
        let (date, time) = self.game_time.format_date_time_strings(server_time)?;
        Some(format!("Date: {date}\nTime: {time}"))
    }

    /// Retimes an existing server timestamp within its current calendar day.
    pub fn server_time_at_day_fraction(&self, server_time: f64, day_fraction: f64) -> Option<f64> {
        self.game_time
            .server_time_at_day_fraction(server_time, day_fraction)
    }

    /// Samples the authored day/night schedule as a continuous night-light factor: zero is the
    /// day state and one the night state.
    pub fn night_light_factor_at(&self, server_time: f64) -> Option<f32> {
        self.game_time.night_light_factor_at(server_time)
    }

    pub fn sky_lighting(&self) -> Option<&SkyLightingDesc> {
        self.sky.as_ref()?.default_lighting()
    }

    /// The authored sky and light update deadlines.
    pub fn sky_environment_cadence(&self) -> Option<SkyEnvironmentCadence> {
        self.sky.as_ref().map(|sky| sky.environment_cadence())
    }

    /// Samples the sky lighting and fog for a server time. Pure data selection.
    pub fn sample_sky_environment(&self, server_time: f64) -> Option<SkyEnvironmentSample> {
        self.sample_sky_environment_selected(server_time, None)
    }

    /// Calendar day of year and authored day name, for the optional weather simulation.
    pub fn weather_day_context_at(&self, server_time: f64) -> Option<(u32, &str)> {
        let calendar = self.game_time.calendar_at(server_time)?;
        let sky = self.sky.as_ref()?;
        let index = sky.day_group_index(calendar.day_group_seed());
        Some((
            calendar.day_of_year,
            sky.day_groups.get(index)?.name.as_ref(),
        ))
    }

    /// Calendar time of day and authored day name.
    pub fn sky_day_context_at(&self, server_time: f64) -> Option<(f32, &str)> {
        self.sky_day_context_selected(server_time, None)
    }

    /// Like [`Self::sky_day_context_at`], but `name` may force an authored day for previews
    /// (case-insensitive first substring match); an absent name keeps the calendar's day.
    pub fn sky_day_context_selected(
        &self,
        server_time: f64,
        name: Option<&str>,
    ) -> Option<(f32, &str)> {
        let calendar = self.game_time.calendar_at(server_time)?;
        let sky = self.sky.as_ref()?;
        let index = self.selected_sky_group(calendar.day_group_seed(), name)?;
        let group = sky.day_groups.get(index)?;
        Some((calendar.time_of_day as f32, group.name.as_ref()))
    }

    /// Lighting and fog of the selected day at the calendar time of day.
    pub fn sample_sky_environment_selected(
        &self,
        server_time: f64,
        name: Option<&str>,
    ) -> Option<SkyEnvironmentSample> {
        let calendar = self.game_time.calendar_at(server_time)?;
        let sky = self.sky.as_ref()?;
        let index = self.selected_sky_group(calendar.day_group_seed(), name)?;
        let mut sample = sky
            .day_groups
            .get(index)?
            .sample(calendar.time_of_day as f32)?;
        if calendar.time_of_day == 0.0 {
            sample.world_fog = None;
        }
        Some(sample)
    }

    fn selected_sky_group(&self, seed: u32, name: Option<&str>) -> Option<usize> {
        let sky = self.sky.as_ref()?;
        let selected = name.and_then(|name| {
            sky.day_groups.iter().position(|group| {
                name.is_empty()
                    || group
                        .name
                        .as_bytes()
                        .windows(name.len().max(1))
                        .any(|part| part.eq_ignore_ascii_case(name.as_bytes()))
            })
        });
        Some(selected.unwrap_or_else(|| sky.day_group_index(seed)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyLightingDesc {
    pub(super) dir_brightness: f32,
    pub(super) dir_heading: f32,
    pub(super) dir_pitch: f32,
    pub(super) dir_color: ColorArgb,
    pub(super) ambient_brightness: f32,
    pub(super) ambient_color: ColorArgb,
}

impl SkyLightingDesc {
    pub fn dir_brightness(&self) -> f32 {
        self.dir_brightness
    }

    pub fn dir_heading(&self) -> f32 {
        self.dir_heading
    }

    pub fn dir_pitch(&self) -> f32 {
        self.dir_pitch
    }

    pub fn dir_color(&self) -> ColorArgb {
        self.dir_color
    }

    pub fn ambient_brightness(&self) -> f32 {
        self.ambient_brightness
    }

    pub fn ambient_color(&self) -> ColorArgb {
        self.ambient_color
    }

    fn interpolate(&self, after: &Self, ratio: f32) -> Self {
        Self {
            dir_brightness: lerp(self.dir_brightness, after.dir_brightness, ratio),
            dir_heading: lerp(self.dir_heading, after.dir_heading, ratio),
            dir_pitch: lerp(self.dir_pitch, after.dir_pitch, ratio),
            dir_color: lerp_color(self.dir_color, after.dir_color, ratio),
            ambient_brightness: lerp(self.ambient_brightness, after.ambient_brightness, ratio),
            ambient_color: lerp_color(self.ambient_color, after.ambient_color, ratio),
        }
    }
}

/// `CSkyDesc`: update cadence and the authored days with their frequencies.
#[derive(Debug, Clone, PartialEq)]
struct SkyDesc {
    tick_size: f64,
    light_tick_size: f64,
    day_groups: Arc<[DayGroup]>,
    day_frequencies: Arc<[f32]>,
}

impl SkyDesc {
    fn day_group_index(&self, seed: u32) -> usize {
        ac2_region::ac2_day_index(seed, &self.day_frequencies)
    }

    fn environment_cadence(&self) -> SkyEnvironmentCadence {
        SkyEnvironmentCadence::new(self.tick_size, self.light_tick_size)
    }

    fn default_lighting(&self) -> Option<&SkyLightingDesc> {
        self.day_groups.iter().find_map(DayGroup::default_lighting)
    }
}

/// One `CDayDesc`: its `SkyTimeOfDay` keys in authored order.
#[derive(Debug, Clone, PartialEq)]
struct DayGroup {
    name: Arc<str>,
    lightings: Arc<[SkyTimeOfDay]>,
}

impl DayGroup {
    fn default_lighting(&self) -> Option<&SkyLightingDesc> {
        self.lightings.first().map(|key| &key.lighting)
    }

    fn sample(&self, time_of_day: f32) -> Option<SkyEnvironmentSample> {
        let (before, after, ratio) = time_key_pair(&self.lightings, time_of_day)?;
        let lighting = before.lighting.interpolate(&after.lighting, ratio);
        let world_fog = before
            .world_fog
            .zip(after.world_fog)
            .map(|(before, after)| before.interpolate(after, ratio));
        Some(SkyEnvironmentSample {
            ambient_brightness: lighting.ambient_brightness,
            ambient_color: lighting.ambient_color,
            directional_brightness: lighting.dir_brightness,
            directional_heading_degrees: lighting.dir_heading,
            directional_pitch_degrees: lighting.dir_pitch,
            directional_color: lighting.dir_color,
            world_fog,
        })
    }
}

/// One `SkyTimeOfDay` key: lighting and linear world fog from `begin` (fraction of the day).
#[derive(Debug, Clone, PartialEq)]
struct SkyTimeOfDay {
    begin: f32,
    lighting: SkyLightingDesc,
    world_fog: Option<SkyFogDesc>,
}

fn time_key_pair(keys: &[SkyTimeOfDay], time: f32) -> Option<(&SkyTimeOfDay, &SkyTimeOfDay, f32)> {
    let key_count = keys.len();
    let _ = keys.first()?;
    let mut before_index = 0;
    if key_count != 1 {
        for candidate in keys.iter().skip(1) {
            if candidate.begin > time {
                break;
            }
            before_index += 1;
        }
    }
    let before = keys.get(before_index)?;
    let after = keys.get((before_index + 1) % key_count)?;
    let span = if before_index + 1 == key_count {
        1.0 - before.begin
    } else {
        after.begin - before.begin
    };
    Some((before, after, (time - before.begin) / span))
}

fn lerp(before: f32, after: f32, ratio: f32) -> f32 {
    before + (after - before) * ratio
}

fn lerp_color(before: ColorArgb, after: ColorArgb, ratio: f32) -> ColorArgb {
    ColorArgb {
        alpha: lerp(f32::from(before.alpha), f32::from(after.alpha), ratio) as u8,
        red: lerp(f32::from(before.red), f32::from(after.red), ratio) as u8,
        green: lerp(f32::from(before.green), f32::from(after.green), ratio) as u8,
        blue: lerp(f32::from(before.blue), f32::from(after.blue), ratio) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(begin: f32, brightness: f32) -> SkyTimeOfDay {
        SkyTimeOfDay {
            begin,
            lighting: SkyLightingDesc {
                dir_brightness: brightness,
                dir_heading: 0.0,
                dir_pitch: 0.0,
                dir_color: ColorArgb::default(),
                ambient_brightness: brightness,
                ambient_color: ColorArgb::default(),
            },
            world_fog: None,
        }
    }

    #[test]
    fn time_keys_interpolate_and_wrap_to_the_first_key() {
        let day = DayGroup {
            name: Arc::from("day"),
            lightings: Arc::from([key(0.25, 0.0), key(0.75, 1.0)]),
        };
        let midday = day.sample(0.5).unwrap();
        assert!((midday.directional_brightness() - 0.5).abs() < 1e-6);
        let late = day.sample(0.875).unwrap();
        assert!((late.directional_brightness() - 0.5).abs() < 1e-6);
    }
}
