//! Per-landblock inputs of the AC2 scenery generator: the `CLandBlockData` record, the
//! `CLandBlockInfo` entities that seed the clearance list, and the block's water surface.

use ac2_assets::read_ac2_entity_desc;
use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::skip_ac2_property_collection;
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

use std::sync::Arc;

use ac2_assets::Ac2EntityDesc;

use super::clearance::{clearance_shapes, total_properties, Clearance};
use super::geometry::{Q, SIDE};

const SAMPLES: usize = SIDE * SIDE;
const MAX_ENTITIES: usize = 1 << 14;
/// `CLandBlockInfo` entity types that `CLandBlock::AddEntityToClearList` registers.
const CLEARANCE_ENTITY_TYPES: [u32; 2] = [0x4000_0001, 0x4000_0007];

/// `CLandBlockData::StreamPack`: DID, landblock-info DID, path-map DID, counted height bytes,
/// counted cell-info words.
#[derive(Debug)]
pub(super) struct Ac2SceneLandBlock {
    pub(super) landblock_info: u32,
    pub(super) heights: Vec<u8>,
    pub(super) cell_info: Vec<u32>,
}

impl ResourcePackedDecode for Ac2SceneLandBlock {
    const TYPE_NAME: &'static str = "AC2 LandBlockData scenery";
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
        let landblock_info = cursor.read_u32()?;
        let _path_map = cursor.read_u32()?;
        if cursor.read_u32()? as usize != SAMPLES {
            return Err(malformed("height count was not 17 x 17"));
        }
        let heights = cursor.read_bytes(SAMPLES)?.to_vec();
        if cursor.read_u32()? as usize != SAMPLES {
            return Err(malformed("cell-info count was not 17 x 17"));
        }
        let cell_info = cursor.read_u32_items(SAMPLES)?;
        Ok(Self {
            landblock_info,
            heights,
            cell_info,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.heights.len() + self.cell_info.len() * 4
    }
}

/// One `CLandBlockInfo` entity of a clearance type: its block-local frame, template and packed
/// properties.
#[derive(Debug, Clone)]
pub(super) struct ClearEntity {
    origin: [f64; 3],
    rotation: Q,
    template: Option<u32>,
    properties: Option<Arc<[u8]>>,
}

#[derive(Debug)]
pub(super) struct Ac2SceneClearEntities {
    entities: Vec<ClearEntity>,
}

impl ResourcePackedDecode for Ac2SceneClearEntities {
    const TYPE_NAME: &'static str = "AC2 LandBlockInfo clearances";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode_clear_entities(&mut ArchiveCursor::new(bytes), qdid).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .entities
                .iter()
                .map(|e| {
                    std::mem::size_of::<ClearEntity>()
                        + e.properties.as_ref().map_or(0, |p| p.len())
                })
                .sum::<usize>()
    }
}

/// `CLandBlockInfo::StreamPack` up to the `EntityGroupDesc` entities (see `ac2_landblock_info.rs`),
/// keeping the entities `CLandBlock::GenerateLBIObjectList` hands to `AddEntityToClearList`
/// (types `0x40000001` and `0x40000007`; weenies are excluded for static objects).
fn decode_clear_entities(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> Result<Ac2SceneClearEntities, &'static str> {
    let word = |cursor: &mut ArchiveCursor<'_>| cursor.read_u32().map_err(|_| "record ended early");
    let bits = word(cursor)?;
    if word(cursor)? != qdid.data_id().raw() {
        return Err("embedded DID did not match DAT identity");
    }
    if bits & 1 != 0 {
        word(cursor)?;
    }
    let _num_cells = word(cursor)?;
    let _light_info = word(cursor)?;
    let _group_type_header = word(cursor)?;
    let count = word(cursor)? as usize;
    if count > MAX_ENTITIES {
        return Err("entity count exceeded the record");
    }
    let mut entities = Vec::new();
    for _ in 0..count {
        let entity = read_ac2_entity_desc(cursor)?;
        if CLEARANCE_ENTITY_TYPES.contains(&entity.entity_type) {
            entities.push(ClearEntity {
                origin: entity.origin.to_array().map(f64::from),
                rotation: quat(&entity),
                template: entity.data.map(|did| did.raw()),
                properties: entity.properties,
            });
        }
    }
    if word(cursor)? != 0 {
        return Err("EntityLinkDesc lists are not supported");
    }
    skip_ac2_property_collection(cursor)?;
    Ok(Ac2SceneClearEntities { entities })
}

fn quat(entity: &Ac2EntityDesc) -> Q {
    let r = entity.rotation;
    [r.w, r.x, r.y, r.z].map(f64::from)
}

/// The clearance list `CLandBlock::GenerateLBIObjectList` seeds: one `ObjectClearance` per
/// clearance-type entity, with the boxes and cylinders of its total properties
/// (`EntityDesc::GetTotalProperties`, `CLandBlock::InqClearanceInfo`).
pub(super) fn load_clearances(
    store: &ResourceStore,
    landblock_info: u32,
) -> ResourceResult<Vec<Clearance>> {
    if landblock_info == 0 {
        return Ok(Vec::new());
    }
    let reference = TypedResourceRef::new(DatFileKind::Cell, DataId::new(landblock_info), 0);
    let Some(record) = store.load_cached_packed::<Ac2SceneClearEntities>(reference)? else {
        return Ok(Vec::new());
    };
    let resolve = |did: u32| {
        store
            .load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .ok()
            .flatten()
            .map(|t| (t.properties.clone(), t.data.map(|d| d.raw())))
    };
    Ok(record
        .entities
        .iter()
        .map(|e| {
            let groups = total_properties(e.properties.as_deref(), e.template, &resolve);
            Clearance::entity(e.origin, e.rotation, clearance_shapes(&groups))
        })
        .collect())
}

/// The block's water surface (`CLandBlockData::CacheFarValues`), when it has water.
pub(super) fn block_water_height(
    store: &ResourceStore,
    block_cell_id: CellId,
    height_table: &[f64],
    min_height: f64,
) -> ResourceResult<Option<f64>> {
    let Some(desc) = crate::region::ac2_region_water_desc(store)? else {
        return Ok(None);
    };
    let Some(water) = crate::ac2_water::Ac2WaterSource::load(store, desc)? else {
        return Ok(None);
    };
    let table = height_table.iter().map(|h| *h as f32).collect::<Vec<_>>();
    Ok(water
        .block_water(block_cell_id, &table, min_height as f32)
        .map(|w| f64::from(w.height)))
}
