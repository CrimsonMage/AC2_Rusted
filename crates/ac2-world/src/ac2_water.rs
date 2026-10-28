//! Asheron's Call 2 water: `CWaterDesc` (`0x16`, `WaterDesc::StreamPack`) with its `WaterType`s
//! and the two `BlockMap`s (`0x19`) it names, resolved per landblock as
//! `CLandBlockData::CacheFarValues` does: the type map selects the water type at the block's
//! (x, y), the height map selects a `LandscapeDefs` height, and the block has water when its
//! lowest vertex lies below that height + 0.95.

use std::sync::Arc;

use ac2_core::{CellId, DataId, QualifiedDataId};

use crate::region::LightFogInfo;
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

/// `CLandBlockData::CacheFarValues` raises the surface above the mapped height.
const WATER_SURFACE_OFFSET: f32 = 0.95;
const MAX_TYPES: usize = 256;

/// The water of one landblock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2BlockWater {
    /// Surface height (metres, AC z).
    pub height: f32,
    /// `MaterialInstance` drawn above and below the surface.
    pub above_material: DataId,
    pub below_material: DataId,
    /// `CWaterDesc::GetPhysicsMaterial`'s water-type physics material (`0` when none).
    pub physics_material: DataId,
    /// Metres per texture repeat (`m_fWaterTiling`).
    pub tiling: f32,
    /// Packed ARGB water colour.
    pub color: u32,
    /// `CLandBlockData::m_waterIndex`: the block's water type map value, which
    /// `CLandBlockMesh::GetWaterIndex` answers.
    pub type_index: u32,
}

impl Ac2BlockWater {
    /// `CLandBlockMesh::GetWaterIndex`: the water type map index of the block's data.
    pub fn water_index(&self) -> u32 {
        self.type_index
    }
}

/// `WaterLightInfo`: the light and fog under water from `depth` down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterLightInfo {
    pub depth: f32,
    pub water_light: LightFogInfo,
}

#[derive(Debug, Clone, PartialEq)]
struct WaterType {
    above_material: DataId,
    below_material: DataId,
    physics_material: DataId,
    tiling: f32,
    color: u32,
    lighting: Arc<[WaterLightInfo]>,
}

/// `CWaterDesc` with its height and type maps resolved.
#[derive(Debug)]
pub struct Ac2WaterSource {
    types: Vec<Option<WaterType>>,
    indices: Vec<u32>,
    /// The default type (`CWaterDesc::default_water`); its inner `None` has no water.
    default_slot: Option<Option<WaterType>>,
    default: Option<WaterType>,
    height_map: Arc<Ac2BlockMap>,
    type_map: Arc<Ac2BlockMap>,
}

impl Ac2WaterSource {
    /// `DBObj::Get` of the `CWaterDesc` and its two block maps.
    pub fn load(store: &ResourceStore, water_desc: DataId) -> ResourceResult<Option<Self>> {
        let Some(desc) = load::<Ac2WaterDescRecord>(store, water_desc)? else {
            return Ok(None);
        };
        let (Some(height_map), Some(type_map)) = (
            load::<Ac2BlockMap>(store, desc.height_map)?,
            load::<Ac2BlockMap>(store, desc.type_map)?,
        ) else {
            return Ok(None);
        };
        Ok(Some(Self {
            indices: desc.types.iter().map(|(index, _)| *index).collect(),
            types: desc.types.iter().map(|(_, water)| water.clone()).collect(),
            default_slot: desc.default.clone(),
            default: desc.default.clone().flatten(),
            height_map: Arc::new((*height_map).clone()),
            type_map: Arc::new((*type_map).clone()),
        }))
    }

    /// `CWaterDesc::FindWaterType`: the type with that index, else the default type. The inner
    /// `None` is a type that has no water.
    fn find_water_type(&self, index: u32) -> Option<&Option<WaterType>> {
        match self
            .indices
            .iter()
            .position(|candidate| *candidate == index)
        {
            Some(slot) => Some(&self.types[slot]),
            None => self.default_slot.as_ref(),
        }
    }

    /// `CWaterDesc::HasWater`: false for a type with no water and for no type at all.
    pub fn has_water(&self, index: u32) -> bool {
        matches!(self.find_water_type(index), Some(Some(_)))
    }

    /// `CWaterDesc::GetTileWidth`: metres per texture repeat, 0 without water.
    pub fn tile_width(&self, index: u32) -> f32 {
        self.water(index).map_or(0.0, |water| water.tiling)
    }

    /// `CWaterDesc::GetAboveMaterial`: 0 without water.
    pub fn above_material(&self, index: u32) -> DataId {
        self.water(index)
            .map_or(DataId::new(0), |water| water.above_material)
    }

    /// `CWaterDesc::GetBelowMaterial`: 0 without water.
    pub fn below_material(&self, index: u32) -> DataId {
        self.water(index)
            .map_or(DataId::new(0), |water| water.below_material)
    }

    /// `CWaterDesc::GetWaterLighting` (`WaterType::GetWaterLighting`): the depth-keyed lights of
    /// the type in stream order, `None` for no type.
    pub fn water_lighting(&self, index: u32) -> Option<Vec<WaterLightInfo>> {
        self.water(index).map(|water| water.lighting.to_vec())
    }

    /// `CWaterDesc::GetVertexColor`: the packed ARGB water colour, `None` without water (retail
    /// answers a static default colour then).
    pub fn vertex_color(&self, index: u32) -> Option<u32> {
        self.water(index).map(|water| water.color)
    }

    /// `CLandBlockData::m_waterIndex`: the water type map value (`BlockMap::GetValue`) of the
    /// block at the start of `block_cell_id`, set whether or not the block ends up with water.
    pub fn block_water_index(&self, block_cell_id: CellId) -> u32 {
        let x = (block_cell_id.raw() >> 24) & 0xFF;
        let y = (block_cell_id.raw() >> 16) & 0xFF;
        u32::from(self.type_map.value(x, y))
    }

    fn water(&self, index: u32) -> Option<&WaterType> {
        self.find_water_type(index)?.as_ref()
    }

    /// The block's water, if its lowest height lies below the water surface.
    pub(crate) fn block_water(
        &self,
        block_cell_id: CellId,
        height_table: &[f32],
        min_height: f32,
    ) -> Option<Ac2BlockWater> {
        let x = (block_cell_id.raw() >> 24) & 0xFF;
        let y = (block_cell_id.raw() >> 16) & 0xFF;
        let index = u32::from(self.type_map.value(x, y));
        // `CWaterDesc::FindWaterType`: linear search on index, else the default type.
        let water = match self
            .indices
            .iter()
            .position(|candidate| *candidate == index)
        {
            Some(slot) => self.types[slot].clone(),
            None => self.default.clone(),
        }?;
        let height =
            height_table.get(usize::from(self.height_map.value(x, y)))? + WATER_SURFACE_OFFSET;
        (min_height < height).then_some(Ac2BlockWater {
            height,
            above_material: water.above_material,
            below_material: water.below_material,
            physics_material: water.physics_material,
            tiling: water.tiling,
            color: water.color,
            type_index: index,
        })
    }
}

/// `CWaterDesc::StreamPack`: version, DID, height-map and type-map DIDs, an optional default
/// `WaterType`, then the counted `WaterType`s.
#[derive(Debug)]
struct Ac2WaterDescRecord {
    height_map: DataId,
    type_map: DataId,
    default: Option<Option<WaterType>>,
    types: Vec<(u32, Option<WaterType>)>,
}

impl ResourcePackedDecode for Ac2WaterDescRecord {
    const TYPE_NAME: &'static str = "AC2 WaterDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        let _version = cursor.read_u32()?;
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let height_map = DataId::new(cursor.read_u32()?);
        let type_map = DataId::new(cursor.read_u32()?);
        let has_default = cursor.read_u32()? != 0;
        cursor.align_to(4)?;
        let default = if has_default {
            Some(read_water_type(&mut cursor).map_err(malformed)?.1)
        } else {
            None
        };
        let count = cursor.read_u32_count()?;
        if count > MAX_TYPES {
            return Err(malformed("water type count"));
        }
        let types = (0..count)
            .map(|_| read_water_type(&mut cursor).map_err(malformed))
            .collect::<ResourceResult<Vec<_>>>()?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after WaterDesc"));
        }
        Ok(Self {
            height_map,
            type_map,
            default,
            types,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.types.len() * std::mem::size_of::<(u32, Option<WaterType>)>()
    }
}

/// `WaterType::StreamPack`: index, no-water flag, then (when water) above/below/physics materials,
/// tile and polygon widths, colour and the depth-keyed `WaterLighting` list.
fn read_water_type(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<(u32, Option<WaterType>), &'static str> {
    let word = |cursor: &mut ArchiveCursor<'_>| cursor.read_u32().map_err(|_| "record ended early");
    let index = word(cursor)?;
    if word(cursor)? != 0 {
        return Ok((index, None));
    }
    let above_material = DataId::new(word(cursor)?);
    let below_material = DataId::new(word(cursor)?);
    let physics_material = DataId::new(word(cursor)?);
    let tiling = f32::from_bits(word(cursor)?);
    let _polygon_width = word(cursor)?;
    let color = word(cursor)?;
    let lights = word(cursor)? as usize;
    if lights == 0 || lights > MAX_TYPES {
        return Err("water lighting count");
    }
    let mut lighting = Vec::with_capacity(lights);
    for _ in 0..lights {
        // `WaterLighting`: depth, then `LightFogInfo`.
        let depth = f32::from_bits(word(cursor)?);
        lighting.push(WaterLightInfo {
            depth,
            water_light: LightFogInfo::unpack(cursor),
        });
    }
    Ok((
        index,
        Some(WaterType {
            above_material,
            below_material,
            physics_material,
            tiling,
            color,
            lighting: lighting.into(),
        }),
    ))
}

/// `BlockMap::StreamPack`: DID, height, width, then `width * height` bytes.
#[derive(Debug, Clone)]
pub struct Ac2BlockMap {
    height: u32,
    width: u32,
    values: Arc<[u8]>,
}

impl Ac2BlockMap {
    /// `BlockMap::GetValue`: `map[height * x + y]` inside the map, else `map[0]`.
    pub fn value(&self, x: u32, y: u32) -> u8 {
        let index = if x < self.width && y < self.height {
            (self.height * x + y) as usize
        } else {
            0
        };
        self.values.get(index).copied().unwrap_or(0)
    }
}

/// `DBObj::Get` of a `BlockMap` (`0x40000015`) by data id.
pub fn load_ac2_block_map(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<Ac2BlockMap>> {
    Ok(load::<Ac2BlockMap>(store, did)?.map(|map| (*map).clone()))
}

impl ResourcePackedDecode for Ac2BlockMap {
    const TYPE_NAME: &'static str = "AC2 BlockMap";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let height = cursor.read_u32()?;
        let width = cursor.read_u32()?;
        let len = (height as usize)
            .checked_mul(width as usize)
            .ok_or_else(|| malformed("block map size"))?;
        let values = Arc::from(cursor.read_bytes(len)?);
        Ok(Self {
            height,
            width,
            values,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.values.len()
    }
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<ac2_db::CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}
