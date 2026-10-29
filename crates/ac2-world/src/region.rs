//! The Asheron's Call 2 region environment: `CRegionDesc`'s sky (`CSkyDesc` days of
//! `SkyTimeOfDay` lighting and fog keys) and its `GameTime` calendar.

use std::sync::Arc;

use ac2_core::QualifiedDataId;

use crate::color::Rgba;

#[path = "region/ac2_region.rs"]
mod ac2_region;
#[path = "region_environment.rs"]
mod environment;
pub use ac2_region::{
    ac2_region_detail_map_table, ac2_region_game_map, ac2_region_info, ac2_region_sound_desc,
};
#[path = "region/detail_map.rs"]
mod detail_map;
#[cfg(test)]
#[path = "region/detail_map_tests.rs"]
mod detail_map_tests;
#[cfg(test)]
#[path = "region/game_time_real_tests.rs"]
mod game_time_real_tests;
pub(crate) use ac2_region::{ac2_region_water_desc, load_ac2_region};
pub use detail_map::{
    ac2_detail_map, ac2_map_note_desc, DetailMap, DetailMapRow, MapNoteDesc, MapNoteRow,
};
#[path = "region/block_fog.rs"]
mod block_fog;
#[cfg(test)]
#[path = "region/descs_real_tests.rs"]
mod descs_real_tests;
#[path = "region/encounter_desc.rs"]
mod encounter_desc;
#[path = "region/fog_desc.rs"]
mod fog_desc;
#[path = "region/property_desc.rs"]
mod property_desc;
pub use block_fog::{BlockFogCorners, RegionFog};
pub use encounter_desc::{EncounterDesc, EncounterElem, EncounterExcept, EncounterRow};
pub use fog_desc::{
    FogCombine, FogCombineType, FogDesc, FogInfo, FogType, LightColors, LightFogInfo,
    LSCAPE_FOG_LINEAR,
};
pub use property_desc::{
    color_corner_interp, float_corner_interp, integer_corner_interp, vector_corner_interp,
    BlockProperty, PropertyDesc,
};
#[path = "region/info_lists.rs"]
mod info_lists;
pub use info_lists::{DidLookup, InfoList, RegionInfo};
#[path = "region/day_desc.rs"]
mod day_desc;
#[path = "region/game_time.rs"]
mod game_time;
#[path = "region/sky_objects.rs"]
mod sky_objects;
pub use environment::{SkyEnvironmentCadence, SkyEnvironmentSample, SkyFogDesc, SkyRain};
use game_time::GameTime;
pub use game_time::{GameClock, GameDateTime, TimeStampCalendar};
pub use sky_objects::{CelestialPosition, LensFlare, LensFlarePoint};

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

    /// The region's `GameTime` as a running clock for `WorldManager::Update`.
    pub fn game_clock(&self) -> GameClock {
        GameClock::new(&self.game_time)
    }

    /// Retimes an existing server timestamp within its current calendar day.
    pub fn server_time_at_day_fraction(&self, server_time: f64, day_fraction: f64) -> Option<f64> {
        self.game_time
            .server_time_at_day_fraction(server_time, day_fraction)
    }

    pub fn sky_lighting(&self) -> Option<&SkyLightingDesc> {
        self.sky.as_ref()?.default_lighting()
    }

    /// `CSkyDesc::GetTickSize(quality)` of the region's sky.
    pub fn sky_tick_size(&self, quality: f32) -> Option<f64> {
        Some(tick_size(self.sky.as_ref()?.tick_sizes, quality))
    }

    /// `CSkyDesc::GetLightTickSize(quality)` of the region's sky.
    pub fn sky_light_tick_size(&self, quality: f32) -> Option<f64> {
        Some(tick_size(self.sky.as_ref()?.light_tick_sizes, quality))
    }

    /// `CSkyDesc::GetDayByIndex`: the data id of the sky's day at `index`, 0 past the end.
    pub fn sky_day_by_index(&self, index: usize) -> Option<u32> {
        let sky = self.sky.as_ref()?;
        let day = sky.day_groups.get(index)?;
        u32::from_str_radix(&day.name, 16).ok()
    }

    /// The authored sky and light update deadlines.
    pub fn sky_environment_cadence(&self) -> Option<SkyEnvironmentCadence> {
        self.sky.as_ref().map(|sky| sky.environment_cadence())
    }

    /// Samples the sky lighting and fog for a server time. Pure data selection.
    pub fn sample_sky_environment(&self, server_time: f64) -> Option<SkyEnvironmentSample> {
        self.sample_sky_environment_selected(server_time, None)
    }

    /// `WorldRender::UpdateTimeOfDay`'s `PhysicsGlobals::SetWind` at a server time: the rain's
    /// wind `(x, y, 0)` while it rains (`CDayDesc::GetRain`), else a Perlin breeze over the time
    /// of day (`WAVEFORM_PERLIN`, amplitude 2, frequency 1, roughness 1, sampled at phases
    /// 1000, 2000 and 3000 at time `time_of_day * 30`, scaled by 4, 4 and 0.5).
    pub fn physics_wind(&self, server_time: f64) -> [f32; 3] {
        let rain = self
            .sample_sky_environment(server_time)
            .map(|sample| sample.rain())
            .unwrap_or_default();
        if rain.amount() != 0.0 {
            let [x, y] = rain.wind();
            return [x, y, 0.0];
        }
        let time_of_day = self
            .game_time
            .calendar_at(server_time)
            .map_or(0.0, |calendar| calendar.time_of_day as f32);
        breeze(time_of_day)
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
        Some(day_desc::sample(
            &sky.day_groups.get(index)?.lightings,
            calendar.time_of_day as f32,
        ))
    }

    /// `CDayDesc::GetSky`: the selected day's sky objects at the calendar time of day.
    pub fn sky_objects_selected(
        &self,
        server_time: f64,
        name: Option<&str>,
    ) -> Option<Vec<CelestialPosition>> {
        let calendar = self.game_time.calendar_at(server_time)?;
        let sky = self.sky.as_ref()?;
        let index = self.selected_sky_group(calendar.day_group_seed(), name)?;
        let group = sky.day_groups.get(index)?;
        Some(sky_objects::get_sky(
            &group.sky_objects,
            &group.lightings,
            calendar.time_of_day as f32,
        ))
    }

    /// [`Self::sky_objects_selected`] for the calendar's own day.
    pub fn sky_objects(&self, server_time: f64) -> Option<Vec<CelestialPosition>> {
        self.sky_objects_selected(server_time, None)
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

/// AC2 `LightInfo`: direct light brightness, heading and pitch (degrees), colours, ambient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyLightingDesc {
    pub(super) dir_brightness: f32,
    pub(super) dir_heading: f32,
    pub(super) dir_pitch: f32,
    pub(super) dir_color: Rgba,
    pub(super) ambient_brightness: f32,
    pub(super) ambient_color: Rgba,
    pub(super) shadow_color: Rgba,
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

    pub fn dir_color(&self) -> Rgba {
        self.dir_color
    }

    pub fn ambient_brightness(&self) -> f32 {
        self.ambient_brightness
    }

    pub fn ambient_color(&self) -> Rgba {
        self.ambient_color
    }
}

/// `CSkyDesc`: update cadence and the authored days with their frequencies.
/// The quality `WorldRender::UpdateTimeOfDay` passes `CSkyDesc::GetTickSize` and
/// `GetLightTickSize`.
const WORLD_RENDER_TICK_QUALITY: f32 = 0.5;

/// `CSkyDesc::GetTickSize` / `GetLightTickSize`: below 0 the minimum, at 1 and over the maximum,
/// between them the maximum less the span scaled by the quality.
pub(crate) fn tick_size((min, max): (f64, f64), quality: f32) -> f64 {
    if quality < 0.0 {
        min
    } else if quality < 1.0 {
        max - (max - min) * f64::from(quality)
    } else {
        max
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SkyDesc {
    /// `CSkyDesc::min_tick_size` and `max_tick_size`.
    tick_sizes: (f64, f64),
    /// `CSkyDesc::min_light_tick_size` and `max_light_tick_size`.
    light_tick_sizes: (f64, f64),
    day_groups: Arc<[DayGroup]>,
    day_frequencies: Arc<[f32]>,
}

impl SkyDesc {
    fn day_group_index(&self, seed: u32) -> usize {
        ac2_region::ac2_day_index(seed, &self.day_frequencies)
    }

    fn environment_cadence(&self) -> SkyEnvironmentCadence {
        SkyEnvironmentCadence::new(
            tick_size(self.tick_sizes, WORLD_RENDER_TICK_QUALITY),
            tick_size(self.light_tick_sizes, WORLD_RENDER_TICK_QUALITY),
        )
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
    sky_objects: Arc<[sky_objects::SkyObject]>,
}

impl DayGroup {
    fn default_lighting(&self) -> Option<&SkyLightingDesc> {
        self.lightings.first().map(|key| &key.lighting)
    }
}

/// One `SkyTimeOfDay` key: lighting and linear world fog from `begin` (fraction of the day).
#[derive(Debug, Clone, PartialEq)]
struct SkyTimeOfDay {
    begin: f32,
    lighting: SkyLightingDesc,
    world_fog: SkyFogDesc,
    replacements: Arc<[sky_objects::SkyObjectReplace]>,
    rain: SkyRain,
}

/// The dry-weather breeze of `WorldRender::UpdateTimeOfDay` at `time_of_day` (fraction of the
/// day): the Perlin waveform's values at three phases, the time a `float` store of
/// `time_of_day * 30`.
fn breeze(time_of_day: f32) -> [f32; 3] {
    let wave = ac2_math::Waveform {
        kind: ac2_math::waveform::kind::PERLIN,
        amplitude: 2.0,
        frequency: 1.0,
        roughness: 1.0,
        ..ac2_math::Waveform::default()
    };
    let time = f64::from((f64::from(time_of_day) * 30.0) as f32);
    let at = |phase: f32, scale: f32| {
        (f64::from(wave.value_for_phase(phase, time)) * f64::from(scale)) as f32
    };
    [at(1000.0, 4.0), at(2000.0, 4.0), at(3000.0, 0.5)]
}

#[cfg(test)]
mod wind_tests {
    use super::breeze;

    #[test]
    fn the_breeze_is_a_gentle_wind_that_changes_over_the_day() {
        let morning = breeze(0.25);
        let evening = breeze(0.75);
        assert_ne!(morning, evening);
        for wind in [morning, evening] {
            // Perlin values stay within the amplitude (2): x, y below 8, z below 1.
            assert!(
                wind[0].abs() <= 8.0 && wind[1].abs() <= 8.0 && wind[2].abs() <= 1.0,
                "{wind:?}"
            );
        }
    }
}
