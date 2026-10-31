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
/// Template `EntityDesc` links followed before a chain is cut (a malformed cycle ends there).
const MAX_TEMPLATE_DEPTH: usize = 8;

/// One AC2 entity placed in a landblock, resolved to the `VisualDesc` it draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2OutdoorEntityMarker {
    visual: DataId,
    world_frame: Frame,
    scale: Vec3,
    collides: bool,
    frill: bool,
    ethereal: u64,
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

    /// Whether the entity takes part in static collision: `CLandBlockInfo` entities of the static
    /// object types ([`is_static_object_type`]) and procedural scenery do (`CLandBlock::CreateStaticObjects`), client-only grass frills do not (`CObjCell::AddFrill`).
    pub fn collides(&self) -> bool {
        self.collides
    }

    /// Client-only grass from a `SceneFile`'s frill group (`CLandBlock::ProcFrill` ->
    /// `CObjCell::AddFrill`). Retail draws these through `FrillRender`, not as cell objects.
    pub fn frill(&self) -> bool {
        self.frill
    }

    /// The object's blocking physics-type bits (`CPhysicsObj::m_blocking_mask`), set from its
    /// entity's ethereal properties (`ac2_db::ac2_ethereal_mask`). Zero for scenery and grass.
    pub fn ethereal_mask(&self) -> u64 {
        self.ethereal
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

/// Whether an entity of `entity_type` becomes a static physics object: the types
/// `CLandBlock::GenerateLBIObjectList` queues as non-weenie clearance entries (`0x40000001`,
/// `0x40000007`), which `CLandBlock::CreateStaticObjects` then makes objects of, and the types
/// `CEnvCell::InitStaticObjects` passes to `CEnvCell::CreatePhysicsObj`. Type `0x40000002` is
/// queued only as a weenie (`_include_weenie`), which `CreateStaticObjects` skips; every other
/// type is never queued.
pub(crate) fn is_static_object_type(entity_type: u32) -> bool {
    matches!(entity_type, 0x4000_0001 | 0x4000_0007)
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
            collides: is_static_object_type(entity.entity_type),
            frill: false,
            ethereal: entity_ethereal_mask(store, entity),
        });
    }
    append_scene_markers(store, block_origin, block_cell_id, &mut markers);
    Ok(Arc::from(markers))
}

/// The blocking-mask bits an entity's `PropertyCollection` sets
/// (`CPhysicsObj::ApplyPropertyCollection` on the object its `EntityDesc` makes), over its
/// merged properties (`EntityDesc::GetTotalProperties`). An entity whose properties do not
/// decode sets none.
pub(crate) fn entity_ethereal_mask(store: &ResourceStore, entity: &Ac2EntityDesc) -> u64 {
    ac2_db::ac2_ethereal_mask(&entity_total_properties(store, entity, 0))
}

/// `EntityDesc::GetTotalProperties`: a placed entity's template `EntityDesc` (its data is one
/// of type `0x40000049`) contributes first, recursively, and the entity's own properties then
/// update that collection (`PropertyCollection::UpdateCollection`), so a property of the same
/// name and group replaces the template's. A missing or undecodable template or properties
/// contribute nothing.
pub(crate) fn entity_total_properties(
    store: &ResourceStore,
    entity: &Ac2EntityDesc,
    depth: usize,
) -> Vec<ac2_db::Ac2PropertyGroup> {
    let mut groups = match entity.data {
        Some(data)
            if depth < MAX_TEMPLATE_DEPTH && data.raw() & 0xFF00_0000 == ENTITY_DESC_PREFIX =>
        {
            let template = store
                .load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    data,
                    0,
                ))
                .ok()
                .flatten();
            template.map_or_else(Vec::new, |t| entity_total_properties(store, &t, depth + 1))
        }
        _ => Vec::new(),
    };
    ac2_db::update_ac2_property_collection(&mut groups, &entity_own_groups(entity));
    groups
}

/// An `EntityDesc`'s own `PropertyCollection` (`EntityDesc::m_properties`), decoded; empty when
/// it has none or it does not decode.
pub(crate) fn entity_own_groups(entity: &Ac2EntityDesc) -> Vec<ac2_db::Ac2PropertyGroup> {
    entity
        .properties
        .as_deref()
        .and_then(|properties| {
            ac2_db::read_ac2_property_groups(&mut ArchiveCursor::new(properties)).ok()
        })
        .unwrap_or_default()
}

/// The ethereal blocking bits of the `EntityDesc` `did`'s own properties, as
/// `CPhysicsObj::InitVisualDesc` applies them to a body an `EntityDesc` makes
/// (`ApplyPropertyCollection` on `EntityDesc::m_properties`, not the template chain). A DID with
/// no `EntityDesc` in the DATs sets none.
pub fn load_entity_ethereal_mask(store: &ResourceStore, did: DataId) -> ResourceResult<u64> {
    let entity = store.load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
        DatFileKind::Portal,
        did,
        0,
    ))?;
    Ok(entity.map_or(0, |entity| {
        ac2_db::ac2_ethereal_mask(&entity_own_groups(&entity))
    }))
}

/// The `EntityDesc` property that, present and false on a template, makes its instances no
/// object at all (`CPhysicsObj::makeObjectInternal`).
const NO_OBJECT_PROPERTY: u32 = 0x4000_006B;

/// `CPhysicsObj::makeObjectInternal`'s check on a template: its own `0x4000006B` property is
/// present and false (or has no value).
pub(crate) fn template_makes_no_object(template: &Ac2EntityDesc) -> bool {
    entity_own_groups(template)
        .iter()
        .flat_map(|group| &group.properties)
        .find(|property| property.name == NO_OBJECT_PROPERTY)
        .is_some_and(|property| property.word().is_none_or(|value| value == 0))
}

/// The `VisualDesc` an entity draws and its templates' scale: a placed `VisualDesc` itself, or
/// the `VisualDesc` at the end of the chain of template `EntityDesc`s it instances. Each template's
/// scale multiplies in (`makeObjectInternal` scales the object by the description's scale times its
/// current one at every level), and a template that makes no object ends the chain
/// ([`template_makes_no_object`]).
fn resolve_entity_visual(store: &ResourceStore, entity: &Ac2EntityDesc) -> Option<(DataId, Vec3)> {
    resolve_entity_chain(store, entity, 0)
}

fn resolve_entity_chain(
    store: &ResourceStore,
    entity: &Ac2EntityDesc,
    depth: usize,
) -> Option<(DataId, Vec3)> {
    let data = entity.data?;
    match data.raw() & 0xFF00_0000 {
        VISUAL_DESC_PREFIX => Some((data, Vec3::ONE)),
        ENTITY_DESC_PREFIX if depth < MAX_TEMPLATE_DEPTH => {
            let template = store
                .load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    data,
                    0,
                ))
                .ok()
                .flatten()?;
            if template_makes_no_object(&template) {
                return None;
            }
            let (visual, scale) = resolve_entity_chain(store, &template, depth + 1)?;
            Some((visual, scale * template.scale))
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
        .filter(|entity| is_static_object_type(entity.entity_type))
        .filter_map(|entity| {
            let (visual, template_scale) = resolve_entity_visual(store, entity)?;
            Some(Ac2OutdoorEntityMarker {
                visual,
                world_frame: Frame::new(block_origin + entity.origin, entity.rotation),
                scale: entity.scale * template_scale,
                collides: true,
                frill: false,
                ethereal: entity_ethereal_mask(store, entity),
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
            ethereal: 0,
        });
    }
}

#[cfg(test)]
#[path = "ac2_landblock_info_tests.rs"]
mod tests;
