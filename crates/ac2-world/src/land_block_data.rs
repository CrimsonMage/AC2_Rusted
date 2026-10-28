//! The decoded `CLandBlockData` record of one landblock (cell DAT `xxxxFFFF`).
//!
//! Retail keeps one object per ID: `CLandBlock` derives from `CLandBlockData`, and the terrain
//! (`CLandBlockData::CacheFarValues`), the scenery (`CLandBlock::GenerateSceneObjectList`) and the ambient
//! sound points (`CLandBlock::CreateAmbientSoundPoints`) all read that one object. Here every user
//! loads this one type through the typed cache, so the record is read and decoded once per block.

use ac2_core::{CellId, DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

/// AC2 `LandDefs::side_vertex_count` squared: 17 x 17 vertices per block.
pub(crate) const LAND_BLOCK_SAMPLES: usize = 17 * 17;

/// AC2 `CLandBlockData::StreamPack`: DID, landblock-info DID, path-map DID, then counted height
/// bytes and counted cell-info words (terrain/scene/encounter/color/road/replace bit fields).
#[derive(Debug)]
pub(crate) struct Ac2LandBlockData {
    /// `m_lbiID`: the block's `CLandBlockInfo` DID.
    pub(crate) landblock_info: u32,
    /// `m_height`: indices into `LandDefs::Land_Height_Table`, x-major.
    pub(crate) heights: Vec<u8>,
    /// `m_cellInfo`, x-major.
    pub(crate) cell_info: Vec<u32>,
}

impl ResourcePackedDecode for Ac2LandBlockData {
    const TYPE_NAME: &'static str = "AC2 LandBlockData";
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
        if cursor.read_u32()? as usize != LAND_BLOCK_SAMPLES {
            return Err(malformed("AC2 landblock height count was not 17 x 17"));
        }
        let heights = cursor.read_bytes(LAND_BLOCK_SAMPLES)?.to_vec();
        if cursor.read_u32()? as usize != LAND_BLOCK_SAMPLES {
            return Err(malformed("AC2 landblock cell-info count was not 17 x 17"));
        }
        let cell_info = cursor.read_u32_items(LAND_BLOCK_SAMPLES)?;
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

/// The landblock DID (`xxxxFFFF`) of the block holding `block_cell_id`.
pub(crate) fn land_block_did(block_cell_id: CellId) -> DataId {
    DataId::new((block_cell_id.raw() & 0xFFFF_0000) | 0xFFFF)
}

/// The one shared `CLandBlockData` of the block holding `block_cell_id`, `None` where the cell DAT
/// has no record for it.
pub(crate) fn load_land_block_data(
    store: &ResourceStore,
    block_cell_id: CellId,
) -> ResourceResult<Option<CachedResource<Ac2LandBlockData>>> {
    store.load_cached_packed::<Ac2LandBlockData>(TypedResourceRef::new(
        DatFileKind::Cell,
        land_block_did(block_cell_id),
        0,
    ))
}
