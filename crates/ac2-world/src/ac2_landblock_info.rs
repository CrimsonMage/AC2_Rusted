//! Asheron's Call 2 `CLandBlockInfo` (`xxyyFFFE` in the cell DAT), decoded in
//! `CLandBlockInfo::StreamPack` order, and its entities resolved to placed visuals.

use std::sync::Arc;

use ac2_assets::{read_ac2_entity_desc, Ac2EntityDesc};
use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::skip_ac2_property_collection;
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};
use ac2_math::{Frame, Vec3};

const ENTITY_DESC_PREFIX: u32 = 0x4700_0000;
const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
const MAX_ITEMS: usize = 1 << 14;

/// One AC2 entity placed in a landblock, resolved to the `VisualDesc` it draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2OutdoorEntityMarker {
    visual: DataId,
    world_frame: Frame,
    scale: Vec3,
    collides: bool,
    frill: bool,
}

impl Ac2OutdoorEntityMarker {
    pub fn visual(&self) -> DataId {
        self.visual
    }

    /// Placement in world AC axes (x east, y north, z up).
    pub fn world_frame(&self) -> Frame {
        self.world_frame
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    /// Whether the entity takes part in static collision: `CLandBlockInfo` entities and procedural
    /// scenery do, client-only grass frills do not (`CObjCell::AddFrill`).
    pub fn collides(&self) -> bool {
        self.collides
    }

    /// Client-only grass from a `SceneFile`'s frill group (`CLandBlock::ProcFrill` ->
    /// `CObjCell::AddFrill`). Retail draws these through `FrillRender`, not as cell objects.
    pub fn frill(&self) -> bool {
        self.frill
    }
}

/// `CLandBlockInfo::StreamPack`: pack bitfield, DID, optional version, cell count, light-info DID,
/// the embedded `EntityGroupDesc` (type header, entities, links), properties, outside stab list.
#[derive(Debug)]
pub(crate) struct Ac2LandBlockEntities {
    entities: Vec<Ac2EntityDesc>,
}

impl ResourcePackedDecode for Ac2LandBlockEntities {
    const TYPE_NAME: &'static str = "AC2 LandBlockInfo";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode(&mut ArchiveCursor::new(bytes), qdid).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.entities.len() * std::mem::size_of::<Ac2EntityDesc>()
    }
}

fn decode(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> Result<Ac2LandBlockEntities, &'static str> {
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
    if count > MAX_ITEMS {
        return Err("entity count exceeded the record");
    }
    let entities = (0..count)
        .map(|_| read_ac2_entity_desc(cursor))
        .collect::<Result<Vec<_>, _>>()?;
    if word(cursor)? != 0 {
        return Err("EntityLinkDesc lists are not supported");
    }
    skip_ac2_property_collection(cursor)?;
    let stabs = word(cursor)? as usize;
    cursor
        .read_bytes(stabs * 4)
        .map_err(|_| "record ended early")?;
    if cursor.remaining() != 0 {
        return Err("trailing bytes after LandBlockInfo");
    }
    Ok(Ac2LandBlockEntities { entities })
}

/// Placed entities of one AC2 landblock, in world AC axes for blocks of `block_size` metres. Instance entities name a template `EntityDesc` whose
/// data is the `VisualDesc`; scale composes instance over template.
pub(crate) fn ac2_entity_markers(
    store: &ResourceStore,
    block_size: f32,
    block_cell_id: CellId,
) -> ResourceResult<Arc<[Ac2OutdoorEntityMarker]>> {
    let block_origin = Vec3::new(
        f32::from(((block_cell_id.raw() >> 24) & 0xFF) as u8) * block_size,
        f32::from(((block_cell_id.raw() >> 16) & 0xFF) as u8) * block_size,
        0.0,
    );
    let did = DataId::new((block_cell_id.raw() & 0xFFFF_0000) | 0xFFFE);
    let Some(record) = store.load_cached_packed::<Ac2LandBlockEntities>(TypedResourceRef::new(
        DatFileKind::Cell,
        did,
        0,
    ))?
    else {
        return Ok(Arc::from([]));
    };
    let mut markers = Vec::with_capacity(record.entities.len());
    for entity in &record.entities {
        let Some((visual, template_scale)) = resolve_entity_visual(store, entity) else {
            continue;
        };
        markers.push(Ac2OutdoorEntityMarker {
            visual,
            world_frame: Frame::new(block_origin + entity.origin, entity.rotation),
            scale: entity.scale * template_scale,
            collides: true,
            frill: false,
        });
    }
    append_scene_markers(store, block_origin, block_cell_id, &mut markers);
    Ok(Arc::from(markers))
}

/// The `VisualDesc` an entity draws and its template's scale: a placed `VisualDesc` itself, or
/// the `VisualDesc` of the template `EntityDesc` it instances.
fn resolve_entity_visual(store: &ResourceStore, entity: &Ac2EntityDesc) -> Option<(DataId, Vec3)> {
    let data = entity.data?;
    match data.raw() & 0xFF00_0000 {
        VISUAL_DESC_PREFIX => Some((data, Vec3::ONE)),
        ENTITY_DESC_PREFIX => {
            let template = store
                .load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    data,
                    0,
                ))
                .ok()
                .flatten()?;
            match template.data {
                Some(visual) if visual.raw() & 0xFF00_0000 == VISUAL_DESC_PREFIX => {
                    Some((visual, template.scale))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// `CEnvCell::InitStaticObjects`: the cell's entities of the static-object types
/// (`0x40000001`, `0x40000007`; `CreatePhysicsObj`), each placed at its offset as a position in
/// the cell's landblock (`block_origin` is that landblock's world origin).
pub(crate) fn ac2_env_cell_markers(
    store: &ResourceStore,
    block_origin: Vec3,
    entities: &[Ac2EntityDesc],
) -> Vec<Ac2OutdoorEntityMarker> {
    entities
        .iter()
        .filter(|entity| matches!(entity.entity_type, 0x4000_0001 | 0x4000_0007))
        .filter_map(|entity| {
            let (visual, template_scale) = resolve_entity_visual(store, entity)?;
            Some(Ac2OutdoorEntityMarker {
                visual,
                world_frame: Frame::new(block_origin + entity.origin, entity.rotation),
                scale: entity.scale * template_scale,
                collides: true,
                frill: false,
            })
        })
        .collect()
}

/// Appends the block's procedural scenery and grass (`CLandBlock::GenerateSceneObjectList`). A
/// block whose scene tables cannot be read keeps its authored entities only.
fn append_scene_markers(
    store: &ResourceStore,
    block_origin: Vec3,
    block_cell_id: CellId,
    markers: &mut Vec<Ac2OutdoorEntityMarker>,
) {
    let Ok(placements) = crate::ac2_scene_placements(store, block_cell_id) else {
        return;
    };
    let mut visuals = std::collections::HashMap::<DataId, Option<(DataId, Vec3)>>::new();
    for placement in placements {
        let resolved = *visuals.entry(placement.object).or_insert_with(|| {
            crate::ac2_scene_object_visual(store, placement.object)
                .ok()
                .flatten()
        });
        let Some((visual, template_scale)) = resolved else {
            continue;
        };
        markers.push(Ac2OutdoorEntityMarker {
            visual,
            world_frame: Frame::new(block_origin + placement.origin, placement.rotation),
            scale: placement.scale * template_scale,
            collides: !placement.frill,
            frill: placement.frill,
        });
    }
}
