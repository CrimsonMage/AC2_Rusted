//! Asheron's Call 2 region environment records: `CRegionDesc` (`0x0F`), `CSkyDesc` (`0x15`),
//! `CDayDesc` (`0x1A`) and `GameTime` (`0x35`), decoded in their `UnPack`/`StreamPack` order.

use std::sync::Arc;

use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_core::{ColorArgb, DataId, QualifiedDataId};
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

use super::game_time::{GameTime, Season, TimeOfDay};
use super::{DayGroup, RegionDesc, SkyDesc, SkyFogDesc, SkyLightingDesc, SkyTimeOfDay};

const AC2_REGION_DID: DataId = DataId::new(0x0F00_0000);
/// `CRegionDesc::UnPack` info lists in stream order; the sky list is the seventh.
const SKY_INFO_LIST: usize = 6;
const WATER_INFO_LIST: usize = 7;
const INFO_LIST_COUNT: usize = 11;
const MAX_ITEMS: usize = 4096;

/// Builds the active region from an AC2 dataset: the first `CRegionDesc` sky entry, its days
/// in authored order and its `GameTime`. Sky objects (AC2 `CSetup`s) are not published.
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

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
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
        // game_map, game_map_scale, autotest_map, autotest_map_size, land_defs_id, block length/width.
        cursor.read_bytes(7 * 4)?;
        let mut sky_info = Vec::new();
        let mut water_info = Vec::new();
        for list in 0..INFO_LIST_COUNT {
            let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 8)?;
            let dids = cursor.read_u32_items(count)?;
            cursor.read_bytes(count * 4)?;
            cursor.align_to(4)?;
            let dids = dids.into_iter().filter(|did| *did != 0).map(DataId::new);
            match list {
                SKY_INFO_LIST => sky_info = dids.collect(),
                WATER_INFO_LIST => water_info = dids.collect(),
                _ => {}
            }
        }
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            region_number,
            sky_info,
            water_info,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + (self.sky_info.len() + self.water_info.len()) * 4
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
        let objects = read_count(&mut cursor, qdid, Self::TYPE_NAME, 68)?;
        for _ in 0..objects {
            cursor.read_bytes(16 * 4)?;
            let points = read_count(&mut cursor, qdid, Self::TYPE_NAME, 12)?;
            cursor.read_bytes(points * 12)?;
        }
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            hours: Arc::from(hours),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.hours.len() * std::mem::size_of::<SkyTimeOfDay>()
    }
}

/// `SkyTimeOfDay::StreamPack`. Colours are packed ARGB words; fog is always linear world fog.
fn read_time_of_day(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> ResourceResult<SkyTimeOfDay> {
    let begin = cursor.read_f32()?;
    let fog_min = cursor.read_f32()?;
    let fog_max = cursor.read_f32()?;
    let _fog_sky = cursor.read_f32()?;
    let fog_color = argb(cursor.read_u32()?);
    let dir_brightness = cursor.read_f32()?;
    let dir_heading = cursor.read_f32()?;
    let dir_pitch = cursor.read_f32()?;
    let dir_color = argb(cursor.read_u32()?);
    let ambient_brightness = cursor.read_f32()?;
    let ambient_color = argb(cursor.read_u32()?);
    let _shadow_color = cursor.read_u32()?;
    // `SkyObjectReplace`: object type, three scalars and a colour.
    let replacements = read_count(cursor, qdid, "AC2 DayDesc", 20)?;
    cursor.read_bytes(replacements * 20)?;
    let _rain = (cursor.read_f32()?, cursor.read_f32()?, cursor.read_f32()?);
    Ok(SkyTimeOfDay {
        begin,
        lighting: SkyLightingDesc {
            dir_brightness,
            dir_heading,
            dir_pitch,
            dir_color,
            ambient_brightness,
            ambient_color,
        },
        world_fog: Some(SkyFogDesc::new(fog_min, fog_max, fog_color)),
    })
}

/// `GameTime::StreamPack`. AC2 names are string-table IDs, published as hex text.
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
        let year_spec = string_id(cursor.read_u32()?);
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 12)?;
        let times_of_day = (0..count)
            .map(|_| {
                Ok(TimeOfDay {
                    start: cursor.read_f32()?,
                    is_night: cursor.read_u32()? != 0,
                    name: string_id(cursor.read_u32()?),
                })
            })
            .collect::<ResourceResult<Vec<_>>>()?;
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 4)?;
        let weekdays = cursor
            .read_u32_items(count)?
            .into_iter()
            .map(string_id)
            .collect();
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 8)?;
        let seasons = (0..count)
            .map(|_| {
                Ok(Season {
                    start: cursor.read_u32()?,
                    name: string_id(cursor.read_u32()?),
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

fn string_id(id: u32) -> String {
    format!("{id:08X}")
}

fn argb(word: u32) -> ColorArgb {
    let [blue, green, red, alpha] = word.to_le_bytes();
    ColorArgb {
        alpha,
        red,
        green,
        blue,
    }
}

fn read_count(
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

fn expect_did(qdid: QualifiedDataId, type_name: &'static str, actual: u32) -> ResourceResult<()> {
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

fn expect_end(
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
    use super::{ac2_day_index, argb};

    #[test]
    fn packed_argb_words_unpack_by_channel() {
        let color = argb(0xFF46_5BA3);
        assert_eq!(
            (color.alpha, color.red, color.green, color.blue),
            (0xFF, 0x46, 0x5B, 0xA3)
        );
    }

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
