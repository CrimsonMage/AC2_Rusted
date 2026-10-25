use std::collections::BTreeMap;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

const AC2_TERRAIN_DESC_DID: DataId = DataId::new(0x1200_0000);

/// AC2 `CTerrainDesc`: maps a landblock vertex's terrain bits to the surface type whose slope
/// bands (`Ac2SurfaceDefs`) select the rendered material.
///
/// Retail `CTerrainDesc::UnPack` reads a version, an optional default `TerrainType`, then a counted
/// list of `TerrainType` records. `CTerrainDesc::GetSurfaceType` resolves a terrain id through
/// `FindTerrainType` and falls back to nothing when the terrain is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Ac2TerrainDesc {
    surface_by_terrain: BTreeMap<u32, u32>,
    default_surface: Option<u32>,
}

impl Ac2TerrainDesc {
    /// Surface type (`0x800000NN`) for a terrain index, or the default terrain's surface.
    pub(crate) fn surface_type(&self, terrain_index: u32) -> Option<u32> {
        self.surface_by_terrain
            .get(&terrain_index)
            .copied()
            .or(self.default_surface)
    }
}

impl ResourcePackedDecode for Ac2TerrainDesc {
    const TYPE_NAME: &'static str = "AC2 TerrainDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode_ac2_terrain_desc(qdid, bytes)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(self.surface_by_terrain.len() * 16)
    }
}

pub(crate) fn load_ac2_terrain_desc(
    store: &ResourceStore,
) -> ResourceResult<Option<CachedResource<Ac2TerrainDesc>>> {
    store.load_cached_packed::<Ac2TerrainDesc>(TypedResourceRef::new(
        DatFileKind::Portal,
        AC2_TERRAIN_DESC_DID,
        0,
    ))
}

struct TerrainTypeRecord {
    surface_info: u32,
    terrain_index: u32,
}

fn decode_ac2_terrain_desc(qdid: QualifiedDataId, bytes: &[u8]) -> ResourceResult<Ac2TerrainDesc> {
    let mut cursor = ArchiveCursor::new(bytes);
    if cursor.read_u32()? != qdid.data_id().raw() {
        return Err(malformed(qdid, "embedded DID did not match DAT identity"));
    }
    let _version = cursor.read_u32()?;
    let has_default = cursor.read_u32()? != 0;
    let default_surface = if has_default {
        Some(read_terrain_type(&mut cursor, bytes.len(), qdid)?.surface_info)
    } else {
        None
    };
    let count = cursor.read_u32()?;
    if count > 1024 {
        return Err(malformed(qdid, "implausible AC2 terrain count"));
    }
    let mut surface_by_terrain = BTreeMap::new();
    for _ in 0..count {
        let terrain = read_terrain_type(&mut cursor, bytes.len(), qdid)?;
        surface_by_terrain.insert(terrain.terrain_index, terrain.surface_info);
    }
    Ok(Ac2TerrainDesc {
        surface_by_terrain,
        default_surface,
    })
}

/// `TerrainType::unpack`: PString name, aligned surface_info, terrain_index, hard_edge_angle,
/// has-default-scene flag, optional `TerrainSceneInfo`, then a counted scene list.
fn read_terrain_type(
    cursor: &mut ArchiveCursor<'_>,
    total_len: usize,
    qdid: QualifiedDataId,
) -> ResourceResult<TerrainTypeRecord> {
    let mut name_len = usize::from(cursor.read_u16()?);
    if name_len == 0xFFFF {
        name_len = usize::try_from(cursor.read_u32()?)
            .map_err(|_| malformed(qdid, "terrain name length overflow"))?;
    }
    if name_len > cursor.remaining() {
        return Err(malformed(qdid, "terrain name exceeded the record"));
    }
    for _ in 0..name_len {
        cursor.read_u8()?;
    }
    align4(cursor, total_len)?;
    let surface_info = cursor.read_u32()?;
    let terrain_index = cursor.read_u32()?;
    let _hard_edge_angle = cursor.read_f32()?;
    let has_default_scene = cursor.read_u32()? != 0;
    if has_default_scene {
        let _scene_enum = cursor.read_u32()?;
        let _scene_index = cursor.read_u32()?;
    }
    let scene_count = cursor.read_u32()?;
    if scene_count > 256 {
        return Err(malformed(qdid, "implausible AC2 terrain scene count"));
    }
    for _ in 0..scene_count {
        let _scene_enum = cursor.read_u32()?;
        let _scene_index = cursor.read_u32()?;
    }
    Ok(TerrainTypeRecord {
        surface_info,
        terrain_index,
    })
}

fn align4(cursor: &mut ArchiveCursor<'_>, total_len: usize) -> ResourceResult<()> {
    let consumed = total_len - cursor.remaining();
    for _ in 0..(4 - consumed % 4) % 4 {
        cursor.read_u8()?;
    }
    Ok(())
}

fn malformed(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: Ac2TerrainDesc::TYPE_NAME,
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_ac2_terrain_desc, AC2_TERRAIN_DESC_DID};
    use ac2_core::QualifiedDataId;

    fn terrain(bytes: &mut Vec<u8>, name: &[u8], surface: u32, index: u32, scenes: u32) {
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name);
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        for word in [surface, index, 0x3F35_04F3, 1, 0x8000_006C, 0, scenes] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for scene in 0..scenes {
            bytes.extend_from_slice(&0x8000_006C_u32.to_le_bytes());
            bytes.extend_from_slice(&(scene + 1).to_le_bytes());
        }
    }

    #[test]
    fn maps_terrain_indices_to_surface_types_like_the_retail_record() {
        let mut bytes = Vec::new();
        for word in [0x1200_0000_u32, 0, 1] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        terrain(&mut bytes, b"default", 0x8000_0001, 0, 0);
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        terrain(&mut bytes, b"dirt", 0x8000_0002, 0, 8);
        terrain(&mut bytes, b"grassland", 0x8000_0003, 1, 8);

        let desc = decode_ac2_terrain_desc(QualifiedDataId::new(AC2_TERRAIN_DESC_DID, 0), &bytes)
            .expect("fixture decodes");
        assert_eq!(desc.surface_type(0), Some(0x8000_0002));
        assert_eq!(desc.surface_type(1), Some(0x8000_0003));
        assert_eq!(desc.surface_type(99), Some(0x8000_0001));
    }
}
