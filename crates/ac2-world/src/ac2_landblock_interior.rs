//! The interior facts of an Asheron's Call 2 `CLandBlockInfo` (`xxyyFFFE`), decoded in
//! `CLandBlockInfo::StreamPack` order: the env cell count
//! (`m_numCells`, cells `xxyy0100` onward), the `CLightInfo` DID, the interior lighting and fog
//! properties (`CLandBlockInfo::ApplyProperty`) and `m_aOutsideStabList`, the cells retail
//! draws from outside (`CLandBlockInfo::InqVisibleCells`).

use std::sync::Arc;

use ac2_assets::read_ac2_entity_desc;
use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::read_ac2_property_collection;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

const MAX_ITEMS: usize = 1 << 14;
/// First env cell of a landblock (`LandDefs::first_envcell_id`).
pub(crate) const AC2_FIRST_ENV_CELL: u32 = 0x0100;

const PROPERTY_FOG_TYPE: u32 = 0x4000_0056;
const PROPERTY_FOG_COLOR: u32 = 0x4000_0057;
const PROPERTY_FOG_NEAR: u32 = 0x4000_0058;
const PROPERTY_FOG_FAR: u32 = 0x4000_0059;
const PROPERTY_AMBIENT: u32 = 0x4000_005A;

/// Interior facts of one AC2 landblock.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2LandBlockInterior {
    pub cell_count: u32,
    pub light_info: Option<DataId>,
    /// `m_InteriorAmbientLight` (linear RGB), when authored.
    pub ambient: Option<[f32; 3]>,
    pub fog_type: Option<u32>,
    pub fog_color: Option<[f32; 3]>,
    pub fog_near: Option<f32>,
    pub fog_far: Option<f32>,
    pub outside_stab_cells: Arc<[CellId]>,
}

impl Ac2LandBlockInterior {
    /// Every env cell id of the block, `xxyy0100` onward.
    pub fn cell_ids(&self, block_cell_id: CellId) -> impl Iterator<Item = CellId> {
        let block = block_cell_id.raw() & 0xFFFF_0000;
        (0..self.cell_count).map(move |index| CellId::new(block | (AC2_FIRST_ENV_CELL + index)))
    }
}

impl ResourcePackedDecode for Ac2LandBlockInterior {
    const TYPE_NAME: &'static str = "AC2 LandBlockInfo interior";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode(&mut ArchiveCursor::new(bytes), qdid.data_id().raw()).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.outside_stab_cells.len() * 4
    }
}

fn color(argb: u32) -> [f32; 3] {
    let channel = |shift: u32| ((argb >> shift) & 0xFF) as f32 / 255.0;
    [channel(16), channel(8), channel(0)]
}

fn decode(cursor: &mut ArchiveCursor<'_>, did: u32) -> Result<Ac2LandBlockInterior, &'static str> {
    let word = |cursor: &mut ArchiveCursor<'_>| cursor.read_u32().map_err(|_| "record ended early");
    let bits = word(cursor)?;
    if word(cursor)? != did {
        return Err("embedded DID did not match DAT identity");
    }
    if bits & 1 != 0 {
        word(cursor)?;
    }
    let cell_count = word(cursor)?;
    let light_info = Some(DataId::new(word(cursor)?)).filter(|did| !did.is_null());
    let _group_type_header = word(cursor)?;
    let entities = word(cursor)? as usize;
    if entities > MAX_ITEMS {
        return Err("entity count exceeded the record");
    }
    for _ in 0..entities {
        read_ac2_entity_desc(cursor)?;
    }
    if word(cursor)? != 0 {
        return Err("EntityLinkDesc lists are not supported");
    }
    let mut interior = Ac2LandBlockInterior {
        cell_count,
        light_info,
        ambient: None,
        fog_type: None,
        fog_color: None,
        fog_near: None,
        fog_far: None,
        outside_stab_cells: Arc::from([]),
    };
    for property in read_ac2_property_collection(cursor)? {
        match property.name {
            PROPERTY_AMBIENT => interior.ambient = property.word().map(color),
            PROPERTY_FOG_TYPE => interior.fog_type = property.word(),
            PROPERTY_FOG_COLOR => interior.fog_color = property.word().map(color),
            PROPERTY_FOG_NEAR => interior.fog_near = property.float_at(0),
            PROPERTY_FOG_FAR => interior.fog_far = property.float_at(0),
            _ => {}
        }
    }
    let stabs = word(cursor)? as usize;
    if stabs > MAX_ITEMS || stabs * 4 > cursor.remaining() {
        return Err("outside stab count exceeded the record");
    }
    interior.outside_stab_cells = (0..stabs)
        .map(|_| word(cursor).map(CellId::new))
        .collect::<Result<Arc<[_]>, _>>()?;
    if cursor.remaining() != 0 {
        return Err("trailing bytes after LandBlockInfo");
    }
    Ok(interior)
}
