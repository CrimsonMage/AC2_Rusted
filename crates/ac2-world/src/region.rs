//! The Asheron's Call 2 region environment: `CRegionDesc`'s sky (`CSkyDesc` days of
//! `SkyTimeOfDay` lighting and fog keys) and its `GameTime` calendar.

use std::sync::Arc;

use ac2_core::QualifiedDataId;

use crate::color::Rgba;

#[path = "region/ac2_region.rs"]
mod ac2_region;
#[path = "region_environment.rs"]
mod environment;
pub use ac2_region::{ac2_region_detail_map_table, ac2_region_game_map, ac2_region_sound_desc};
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
#[path = "region/day_desc.rs"]
mod day_desc;
#[path = "region/game_time.rs"]
mod game_time;
#[path = "region/sky_objects.rs"]
mod sky_objects;
pub use environment::{SkyEnvironmentCadence, SkyEnvironmentSample, SkyFogDesc, SkyRain};
use game_time::GameTime;
pub use game_time::{GameClock, GameDateTime};
pub use sky_objects::CelestialPosition;

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
