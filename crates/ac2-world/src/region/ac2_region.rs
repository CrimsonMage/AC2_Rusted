//! Asheron's Call 2 region environment records: `CRegionDesc` (`0x0F`), `CSkyDesc` (`0x15`),
//! `CDayDesc` (`0x1A`) and `GameTime` (`0x35`), decoded in their `UnPack`/`StreamPack` order.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

use super::game_time::{GameTime, Season, TimeOfDay};
use super::sky_objects::{SkyObject, SkyObjectReplace};
use super::{DayGroup, RegionDesc, SkyDesc, SkyFogDesc, SkyLightingDesc, SkyRain, SkyTimeOfDay};
use crate::color::set_color32;

const AC2_REGION_DID: DataId = DataId::new(0x0F00_0000);
/// `CRegionDesc::UnPack` info lists in stream order; the sky list is the seventh.
const SOUND_INFO_LIST: usize = 0;
/// `CRegionDesc::UnPack` info lists: encounter 4, detail map table 5, sky 6, water 7.
const DETAIL_MAP_INFO_LIST: usize = 5;
const SKY_INFO_LIST: usize = 6;
const WATER_INFO_LIST: usize = 7;
const INFO_LIST_COUNT: usize = 11;
const MAX_ITEMS: usize = 4096;

/// Builds the active region from an AC2 dataset: the first `CRegionDesc` sky entry, its days
/// in authored order with their sky objects, and its `GameTime`.
pub(crate) fn load_ac2_region(store: &ResourceStore) -> ResourceResult<Option<RegionDesc>> {
    let Some(region) = load::<Ac2RegionRecord>(store, AC2_REGION_DID)? else {
        return Ok(None);
    };
    let Some(sky) = region
        .sky_info
        .first()
        .map(|did| load::<Ac2SkyRecord>(store, *did))
        .transpose()?
        .flatten()
    else {
        return Ok(None);
    };
    let Some(game_time) = load::<Ac2GameTimeRecord>(store, sky.game_time)? else {
        return Ok(None);
    };
    let mut day_groups = Vec::with_capacity(sky.days.len());
    let mut frequencies = Vec::with_capacity(sky.days.len());
    for (did, frequency) in &sky.days {
        if let Some(day) = load::<Ac2DayRecord>(store, *did)? {
            day_groups.push(DayGroup {
                name: Arc::from(format!("{:08X}", did.raw())),
                lightings: Arc::clone(&day.hours),
                sky_objects: Arc::clone(&day.objects),
            });
            frequencies.push(*frequency);
        }
    }
    Ok(Some(RegionDesc {
        qdid: QualifiedDataId::new(AC2_REGION_DID, 0),
        region_number: region.region_number,
        game_time: game_time.game_time.clone(),
        sky: Some(Arc::new(SkyDesc {
            tick_size: sky.tick_size,
            light_tick_size: sky.light_tick_size,
            day_groups: Arc::from(day_groups),
            day_frequencies: Arc::from(frequencies),
        })),
    }))
}

/// `CRegionDesc::GetCurrentSoundInfo`: the region's first `CSoundDesc` (`0x10`).
pub fn ac2_region_sound_desc(store: &ResourceStore) -> ResourceResult<Option<DataId>> {
    Ok(load::<Ac2RegionRecord>(store, AC2_REGION_DID)?
        .and_then(|region| region.sound_info.first().copied()))
}

/// `CRegionDesc::GetGameMap`: the region's game map image (`game_map`), `None` when it has none.
pub fn ac2_region_game_map(store: &ResourceStore) -> ResourceResult<Option<DataId>> {
    Ok(load::<Ac2RegionRecord>(store, AC2_REGION_DID)?
        .and_then(|region| (region.game_map != 0).then(|| DataId::new(region.game_map))))
}

/// `CRegionDesc::GetCurrentDetailMapTable`: the region's first `CDetailMapDesc` (`0x5A`), the
/// table `current_detailmap_table` (0) picks; `None` when the region has none.
pub fn ac2_region_detail_map_table(store: &ResourceStore) -> ResourceResult<Option<DataId>> {
    Ok(load::<Ac2RegionRecord>(store, AC2_REGION_DID)?
        .and_then(|region| region.detail_map_info.first().copied()))
}

/// `CRegionDesc::GetCurrentWaterInfo`: the region's first `CWaterDesc`.
pub(crate) fn ac2_region_water_desc(store: &ResourceStore) -> ResourceResult<Option<DataId>> {
    Ok(load::<Ac2RegionRecord>(store, AC2_REGION_DID)?
        .and_then(|region| region.water_info.first().copied()))
}

/// `CSkyDesc::GetCurrentDay`: a hashed day number weighted by each day's authored frequency.
pub(super) fn ac2_day_index(seed: u32, frequencies: &[f32]) -> usize {
    if frequencies.len() <= 1 {
        return 0;
    }
    let total: f32 = frequencies.iter().sum();
    let hash = seed
        .wrapping_mul(0x3FBB_657F)
        .wrapping_sub(0x6B3C_5F11)
        .wrapping_mul(seed)
        .wrapping_add(0x0719_BAF0);
    let target = hash as f32 * 2.328_306_4e-10 * total;
    let mut accumulated = 0.0;
    for (index, frequency) in frequencies[..frequencies.len() - 1].iter().enumerate() {
        accumulated += frequency;
        if target < accumulated {
            return index;
        }
    }
    frequencies.len() - 1
}

pub(super) fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<ac2_db::CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}

/// `CRegionDesc::UnPack`: DID, region number, name, map/land-defs words, then eleven
/// `UnPackInfoList`s (count, DIDs, region indices, 4-byte aligned).
#[derive(Debug)]
struct Ac2RegionRecord {
    region_number: u32,
    /// `CRegionDesc::game_map`: the map image's DID (0 for none).
    game_map: u32,
    sound_info: Vec<DataId>,
    detail_map_info: Vec<DataId>,
    sky_info: Vec<DataId>,
    water_info: Vec<DataId>,
}

impl ResourcePackedDecode for Ac2RegionRecord {
    const TYPE_NAME: &'static str = "AC2 RegionDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let region_number = cursor.read_u32()?;
        // `PStringBase<char>::UnPack`: u16 length (0xFFFF escapes to a u32), bytes, 4-byte align.
        let short_len = usize::from(cursor.read_u16()?);
        let name_len = if short_len == 0xFFFF {
            cursor.read_u32_count()?
        } else {
            short_len
        };
        cursor.read_bytes(name_len)?;
        cursor.align_to(4)?;
        // `CRegionDesc::UnPack`: game_map, game_map_scale, autotest_map, autotest_map_size,
        // land_defs_id, num_block_length, num_block_width.
        let game_map = cursor.read_u32()?;
        cursor.read_bytes(6 * 4)?;
        let mut sound_info = Vec::new();
        let mut detail_map_info = Vec::new();
        let mut sky_info = Vec::new();
        let mut water_info = Vec::new();
        for list in 0..INFO_LIST_COUNT {
            let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 8)?;
            let dids = cursor.read_u32_items(count)?;
            cursor.read_bytes(count * 4)?;
            cursor.align_to(4)?;
            let dids = dids.into_iter().filter(|did| *did != 0).map(DataId::new);
            match list {
                SOUND_INFO_LIST => sound_info = dids.collect(),
                DETAIL_MAP_INFO_LIST => detail_map_info = dids.collect(),
                SKY_INFO_LIST => sky_info = dids.collect(),
                WATER_INFO_LIST => water_info = dids.collect(),
                _ => {}
            }
        }
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            region_number,
            game_map,
            sound_info,
            detail_map_info,
            sky_info,
            water_info,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.sound_info.len()
                + self.detail_map_info.len()
                + self.sky_info.len()
                + self.water_info.len())
                * 4
    }
}

/// `CSkyDesc::StreamPack`: version, DID, min/max tick and light-tick sizes (f64), the `GameTime`
/// DID, then `SkyDay`s (`CDayDesc` DID, frequency).
#[derive(Debug)]
struct Ac2SkyRecord {
    tick_size: f64,
    light_tick_size: f64,
    game_time: DataId,
    days: Vec<(DataId, f32)>,
}

impl ResourcePackedDecode for Ac2SkyRecord {
    const TYPE_NAME: &'static str = "AC2 SkyDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        let _version = cursor.read_u32()?;
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let min_tick = cursor.read_f64()?;
        let max_tick = cursor.read_f64()?;
        let min_light_tick = cursor.read_f64()?;
        let max_light_tick = cursor.read_f64()?;
        let game_time = DataId::new(cursor.read_u32()?);
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 8)?;
        let days = (0..count)
            .map(|_| Ok((DataId::new(cursor.read_u32()?), cursor.read_f32()?)))
            .collect::<ResourceResult<Vec<_>>>()?;
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            tick_size: min_tick.max(max_tick),
            light_tick_size: min_light_tick.max(max_light_tick),
            game_time,
            days,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.days.len() * 8
    }
}

/// `CDayDesc::StreamPack`: version, DID, `SkyTimeOfDay` hours, then `SkyObject`s.
#[derive(Debug)]
struct Ac2DayRecord {
    hours: Arc<[SkyTimeOfDay]>,
    objects: Arc<[SkyObject]>,
}

impl ResourcePackedDecode for Ac2DayRecord {
    const TYPE_NAME: &'static str = "AC2 DayDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        let _version = cursor.read_u32()?;
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 64)?;
        let hours = (0..count)
            .map(|_| read_time_of_day(&mut cursor, qdid))
            .collect::<ResourceResult<Vec<_>>>()?;
        // `SkyObject::StreamPack`: 15 words, then `LensFlareEffect` (fade speed, counted points of
        // axis position, texture and size).
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 68)?;
        let mut objects = Vec::with_capacity(count);
        for _ in 0..count {
            objects.push(SkyObject {
                name: cursor.read_u32()?,
                default_gfx: cursor.read_u32()?,
                begin_time: cursor.read_f32()?,
                end_time: cursor.read_f32()?,
                begin_pitch: cursor.read_f32()?,
                end_pitch: cursor.read_f32()?,
                begin_heading: cursor.read_f32()?,
                end_heading: cursor.read_f32()?,
                height_mod: cursor.read_f32()?,
                after: cursor.read_u32()? != 0,
                fixed_pos: cursor.read_u32()? != 0,
                water_effect: cursor.read_u32()?,
                fixed_x: cursor.read_f32()?,
                fixed_y: cursor.read_f32()?,
                west_to_east: cursor.read_u32()? != 0,
            });
            // `LensFlareEffect::StreamPack`: fade speed, then counted points (not drawn yet).
            let _fade_speed = cursor.read_f32()?;
            let points = read_count(&mut cursor, qdid, Self::TYPE_NAME, 12)?;
            cursor.read_bytes(points * 12)?;
        }
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            hours: Arc::from(hours),
            objects: Arc::from(objects),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.hours.len() * std::mem::size_of::<SkyTimeOfDay>()
    }
}

/// `SkyTimeOfDay::StreamPack`. Colours are `RGBAColor::StreamPack` ARGB words.
fn read_time_of_day(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> ResourceResult<SkyTimeOfDay> {
    let begin = cursor.read_f32()?;
    let fog_min = cursor.read_f32()?;
    let fog_max = cursor.read_f32()?;
    let fog_sky = cursor.read_f32()?;
    let fog_color = set_color32(cursor.read_u32()?);
    let dir_brightness = cursor.read_f32()?;
    let dir_heading = cursor.read_f32()?;
    let dir_pitch = cursor.read_f32()?;
    let dir_color = set_color32(cursor.read_u32()?);
    let ambient_brightness = cursor.read_f32()?;
    let ambient_color = set_color32(cursor.read_u32()?);
    let shadow_color = set_color32(cursor.read_u32()?);
    // `SkyObjectReplace::StreamPack`: object name, graphic, pitch and heading offsets, colour.
    let count = read_count(cursor, qdid, "AC2 DayDesc", 20)?;
    let mut replacements = Vec::with_capacity(count);
    for _ in 0..count {
        replacements.push(SkyObjectReplace {
            name: cursor.read_u32()?,
            replace_gfx: cursor.read_u32()?,
            pitch_offset: cursor.read_f32()?,
            heading_offset: cursor.read_f32()?,
            color: set_color32(cursor.read_u32()?),
        });
    }
    let rain = SkyRain {
        amount: cursor.read_f32()?,
        wind_x: cursor.read_f32()?,
        wind_y: cursor.read_f32()?,
    };
    Ok(SkyTimeOfDay {
        begin,
        lighting: SkyLightingDesc {
            dir_brightness,
            dir_heading,
            dir_pitch,
            dir_color,
            ambient_brightness,
            ambient_color,
            shadow_color,
        },
        world_fog: SkyFogDesc {
            min_distance: fog_min,
            max_distance: fog_max,
            sky: fog_sky,
            color: fog_color,
        },
        replacements: Arc::from(replacements),
        rain,
    })
}

/// `GameTime::StreamPack`. The names are string-table IDs, kept as they are.
#[derive(Debug)]
struct Ac2GameTimeRecord {
    game_time: GameTime,
}

impl ResourcePackedDecode for Ac2GameTimeRecord {
    const TYPE_NAME: &'static str = "AC2 GameTime";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let zero_time_of_year = cursor.read_f64()?;
        let zero_year = cursor.read_u32()? as i32;
        let day_length = cursor.read_f32()?;
        let days_per_year = cursor.read_u32()?;
        let year_spec = cursor.read_u32()?;
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 12)?;
        let times_of_day = (0..count)
            .map(|_| {
                Ok(TimeOfDay {
                    start: cursor.read_f32()?,
                    is_night: cursor.read_u32()? != 0,
                    name: cursor.read_u32()?,
                })
            })
            .collect::<ResourceResult<Vec<_>>>()?;
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 4)?;
        let weekdays = cursor.read_u32_items(count)?;
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 8)?;
        let seasons = (0..count)
            .map(|_| {
                Ok(Season {
                    start: cursor.read_u32()?,
                    name: cursor.read_u32()?,
                })
            })
            .collect::<ResourceResult<Vec<_>>>()?;
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            game_time: GameTime {
                zero_time_of_year,
                zero_year,
                day_length,
                days_per_year,
                year_spec,
                times_of_day,
                weekdays,
                seasons,
            },
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + 64 * (self.game_time.seasons.len() + self.game_time.times_of_day.len())
    }
}

pub(super) fn read_count(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
    type_name: &'static str,
    min_item_bytes: usize,
) -> ResourceResult<usize> {
    let count = cursor.read_u32_count()?;
    if count > MAX_ITEMS || count * min_item_bytes > cursor.remaining() {
        return Err(ResourceError::MalformedTypedObject {
            qdid,
            type_name,
            context: "item count exceeded the record",
        });
    }
    Ok(count)
}

pub(super) fn expect_did(
    qdid: QualifiedDataId,
    type_name: &'static str,
    actual: u32,
) -> ResourceResult<()> {
    if actual == qdid.data_id().raw() {
        Ok(())
    } else {
        Err(ResourceError::MalformedTypedObject {
            qdid,
            type_name,
            context: "embedded DID did not match DAT identity",
        })
    }
}

pub(super) fn expect_end(
    qdid: QualifiedDataId,
    type_name: &'static str,
    cursor: &ArchiveCursor<'_>,
) -> ResourceResult<()> {
    if cursor.remaining() == 0 {
        Ok(())
    } else {
        Err(ResourceError::MalformedTypedObject {
            qdid,
            type_name,
            context: "trailing bytes after record",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ac2_day_index;

    #[test]
    fn day_choice_follows_authored_frequencies() {
        let frequencies = [25.0, 25.0, 50.0];
        let mut counts = [0_usize; 3];
        for seed in 0..3600 {
            counts[ac2_day_index(seed, &frequencies)] += 1;
        }
        assert!(counts.iter().all(|count| *count > 0));
        assert!(counts[2] > counts[0] && counts[2] > counts[1], "{counts:?}");
        assert_eq!(ac2_day_index(7, &[1.0]), 0);
    }
}
