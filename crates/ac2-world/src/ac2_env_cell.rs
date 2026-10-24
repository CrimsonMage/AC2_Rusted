//! Asheron's Call 2 `CEnvCell` (`xxyy0100` and up in the cell DAT): one interior cell of a
//! building or dungeon, decoded in `CEnvCell::StreamPack` order.
//!
//! Record: the cell's `Position` (cell id, then an origin and `w, x, y, z` quaternion relative to
//! its landblock), a pack bitfield (bit 0 the cell reaches the outside, bit 1 static entities,
//! bit 2 properties, bit 3 `m_bNoCull`, bit 4 `m_bDrawSky`), the `CCellMesh` DID
//! (`m_environmentDID`, `0x01`), the counted `CCellPortal`s (`CCellPortal::UnPack`:
//! u16 flags, i16 polygon, u16 other cell, i16 other portal, optional u16
//! stab list, 4-byte aligned), the counted u16 stab list of visible cells, the counted u16
//! shared cells, then the optional `EntityGroupDesc` and `PropertyCollection`.

use std::sync::Arc;

use ac2_assets::{read_ac2_entity_desc, Ac2EntityDesc};
use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::read_ac2_property_collection;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::{Frame, Quat, Vec3};

const MAX_ITEMS: usize = 1 << 14;

const PACK_SEEN_OUTSIDE: u32 = 0x01;
const PACK_ENTITIES: u32 = 0x02;
const PACK_PROPERTIES: u32 = 0x04;
const PACK_NO_CULL: u32 = 0x08;
const PACK_DRAW_SKY: u32 = 0x10;

const PORTAL_EXACT_MATCH: u16 = 0x1;
const PORTAL_OUTSIDE: u16 = 0x2;
const PORTAL_STABS: u16 = 0x4;

/// `CEnvCell::ApplyProperty` names.
const PROPERTY_APPEARANCE_KEY: u32 = 0x4000_005D;
const PROPERTY_APPEARANCE_SCALAR: u32 = 0x4000_005E;
const PROPERTY_PHYSICS_MATERIAL: u32 = 0x4000_005F;
/// `AppliedAppearanceKey` value that applies nothing (`BuildAndApplyAtomCollection`).
const APPEARANCE_KEY_NONE: u32 = 0x4000_0001;

/// One `CCellPortal` of an AC2 env cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2EnvCellPortal {
    /// Index into the cell mesh's portal polygons (`CCellMesh::m_portalPolygons`).
    pub polygon: i16,
    /// The neighbouring env cell, or `None` when the portal opens to the outside.
    pub other_cell: Option<CellId>,
    pub other_portal: i16,
    pub exact_match: bool,
    /// Cells visible through this portal.
    pub stab_cells: Box<[CellId]>,
}

/// A decoded AC2 env cell. The frame is relative to the cell's landblock origin.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2EnvCell {
    pub cell_id: CellId,
    pub frame: Frame,
    /// `CCellMesh` (`0x01`) the cell draws and collides with.
    pub cell_mesh: DataId,
    pub seen_outside: bool,
    pub no_cull: bool,
    pub draw_sky: bool,
    pub portals: Arc<[Ac2EnvCellPortal]>,
    /// The cell's stab list (`CObjCell::m_stabList`): every cell visible from inside it.
    pub visible_cells: Arc<[CellId]>,
    pub shared_cells: Arc<[CellId]>,
    /// Static entities placed in the cell, in cell-local space.
    pub entities: Arc<[Ac2EntityDesc]>,
    /// `AppliedAppearanceKey`: the dungeon appearance key and its scalar (0 when unset).
    pub appearance_key: Option<u32>,
    pub appearance_scalar: f32,
    pub physics_material: Option<DataId>,
}

impl Ac2EnvCell {
    /// Whether a portal of this cell opens to the outside.
    pub fn has_outside_portal(&self) -> bool {
        self.portals
            .iter()
            .any(|portal| portal.other_cell.is_none())
    }
}

impl ResourcePackedDecode for Ac2EnvCell {
    const TYPE_NAME: &'static str = "AC2 EnvCell";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        let cell = decode(&mut cursor, qdid.data_id().raw()).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })?;
        if cursor.remaining() != 0 {
            return Err(ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context: "trailing bytes after EnvCell",
            });
        }
        Ok(cell)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.portals.len() * std::mem::size_of::<Ac2EnvCellPortal>()
            + self
                .portals
                .iter()
                .map(|portal| portal.stab_cells.len() * 4)
                .sum::<usize>()
            + (self.visible_cells.len() + self.shared_cells.len()) * 4
            + self.entities.len() * std::mem::size_of::<Ac2EntityDesc>()
    }
}

fn decode(cursor: &mut ArchiveCursor<'_>, did: u32) -> Result<Ac2EnvCell, &'static str> {
    let word = |cursor: &mut ArchiveCursor<'_>| cursor.read_u32().map_err(|_| "record ended early");
    let float =
        |cursor: &mut ArchiveCursor<'_>| cursor.read_f32().map_err(|_| "record ended early");
    let half = |cursor: &mut ArchiveCursor<'_>| cursor.read_u16().map_err(|_| "record ended early");
    let align =
        |cursor: &mut ArchiveCursor<'_>| cursor.align_to(4).map_err(|_| "record ended early");
    if word(cursor)? != did {
        return Err("embedded cell id did not match DAT identity");
    }
    let block = did & 0xFFFF_0000;
    let cell = |low: u16| CellId::new(block | u32::from(low));
    let origin = Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?);
    let w = float(cursor)?;
    let rotation = Quat::from_xyzw(float(cursor)?, float(cursor)?, float(cursor)?, w);
    let rotation = if rotation.length_squared() > 0.0 {
        rotation.normalize()
    } else {
        Quat::IDENTITY
    };
    let bits = word(cursor)?;
    let cell_mesh = DataId::new(word(cursor)?);
    let portal_count = count(cursor, 8)?;
    let mut portals = Vec::with_capacity(portal_count);
    for _ in 0..portal_count {
        let flags = half(cursor)?;
        let polygon = half(cursor)? as i16;
        let other = half(cursor)?;
        let other_portal = half(cursor)? as i16;
        let stab_cells = if flags & PORTAL_STABS != 0 {
            let stabs = usize::from(half(cursor)?);
            let cells = (0..stabs)
                .map(|_| half(cursor).map(cell))
                .collect::<Result<Box<[_]>, _>>()?;
            align(cursor)?;
            cells
        } else {
            Box::default()
        };
        portals.push(Ac2EnvCellPortal {
            polygon,
            other_cell: (flags & PORTAL_OUTSIDE == 0).then(|| cell(other)),
            other_portal,
            exact_match: flags & PORTAL_EXACT_MATCH != 0,
            stab_cells,
        });
    }
    align(cursor)?;
    let cell_list = |cursor: &mut ArchiveCursor<'_>| -> Result<Arc<[CellId]>, &'static str> {
        let n = count(cursor, 2)?;
        let cells = (0..n)
            .map(|_| half(cursor).map(cell))
            .collect::<Result<Arc<[_]>, _>>()?;
        align(cursor)?;
        Ok(cells)
    };
    let visible_cells = cell_list(cursor)?;
    let shared_cells = cell_list(cursor)?;
    let entities = if bits & PACK_ENTITIES != 0 {
        let _group_type = word(cursor)?;
        let n = count(cursor, 4)?;
        let entities = (0..n)
            .map(|_| read_ac2_entity_desc(cursor))
            .collect::<Result<Arc<[_]>, _>>()?;
        if word(cursor)? != 0 {
            return Err("EntityLinkDesc lists are not supported");
        }
        entities
    } else {
        Arc::from([])
    };
    let (mut appearance_key, mut appearance_scalar, mut physics_material) = (None, 0.0, None);
    if bits & PACK_PROPERTIES != 0 {
        for property in read_ac2_property_collection(cursor)? {
            match property.name {
                PROPERTY_APPEARANCE_KEY => {
                    appearance_key = property
                        .word()
                        .filter(|key| *key != 0 && *key != APPEARANCE_KEY_NONE);
                }
                PROPERTY_APPEARANCE_SCALAR => {
                    appearance_scalar = property.float_at(0).unwrap_or(0.0)
                }
                PROPERTY_PHYSICS_MATERIAL => {
                    physics_material = property.word().filter(|did| *did != 0).map(DataId::new);
                }
                _ => {}
            }
        }
    }
    Ok(Ac2EnvCell {
        cell_id: CellId::new(did),
        frame: Frame::new(origin, rotation),
        cell_mesh,
        seen_outside: bits & PACK_SEEN_OUTSIDE != 0,
        no_cull: bits & PACK_NO_CULL != 0,
        draw_sky: bits & PACK_DRAW_SKY != 0,
        portals: Arc::from(portals),
        visible_cells,
        shared_cells,
        entities,
        appearance_key,
        appearance_scalar,
        physics_material,
    })
}

fn count(cursor: &mut ArchiveCursor<'_>, min_item_bytes: usize) -> Result<usize, &'static str> {
    let n = cursor.read_u32().map_err(|_| "record ended early")? as usize;
    if n > MAX_ITEMS || n * min_item_bytes > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(n)
}
