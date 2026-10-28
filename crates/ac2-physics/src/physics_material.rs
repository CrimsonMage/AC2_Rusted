//! `PhysicsMaterial` (DB type `0x40000033`, the `.pmat` records of DIDs `0x37xxxxxx`): the
//! material a landscape vertex, a water type or an interior cell names. Its body is the DBObj
//! identity (`DBObj::UnPack_Type`: the DID, and no category for this type) then the material's
//! `PropertyCollection` (`PhysicsMaterial::StreamPack`). The collision report reads its property
//! groups (`CCollisionManager::UpdatePropertyArray`).

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    read_ac2_property_groups, Ac2PropertyGroup, ResourceError, ResourcePackedDecode,
    ResourceResult, ResourceStore, ResourceTypeId, TypedResourceRef,
};

/// A physics material as the collision report uses it: its physics id (the DID, widened to the
/// report's `u64`) and its property groups. A material the DAT lacks is the id with no groups.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhysicsMaterial {
    /// `m_pmat_id`: the material's DID.
    pub id: u64,
    /// `m_collection.m_propertyGroup`.
    pub groups: Vec<Ac2PropertyGroup>,
}

impl PhysicsMaterial {
    /// The id with no property groups (a material the DATs do not hold).
    pub fn with_id(id: u64) -> Self {
        Self {
            id,
            groups: Vec::new(),
        }
    }
}

/// The decoded `.pmat` record: the DBObj identity's DID check, then the property collection.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PhysicsMaterialRecord {
    pub(crate) groups: Vec<Ac2PropertyGroup>,
}

impl ResourcePackedDecode for PhysicsMaterialRecord {
    const TYPE_NAME: &'static str = "AC2 PhysicsMaterial";
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
        let groups = read_ac2_property_groups(&mut cursor).map_err(malformed)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after PhysicsMaterial"));
        }
        Ok(Self { groups })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .groups
                .iter()
                .map(|group| {
                    std::mem::size_of::<Ac2PropertyGroup>()
                        + group
                            .properties
                            .iter()
                            .map(|p| std::mem::size_of_val(p) + p.bytes.len())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}

/// Loads the physics material `did` from the portal DAT (on the loader thread). `None` where the
/// DAT has no such record.
pub fn load_physics_material(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<Arc<PhysicsMaterial>>> {
    let record = store.load_cached_packed::<PhysicsMaterialRecord>(TypedResourceRef::new(
        DatFileKind::Portal,
        did,
        0,
    ))?;
    Ok(record.map(|record| {
        Arc::new(PhysicsMaterial {
            id: u64::from(did.raw()),
            groups: record.groups.clone(),
        })
    }))
}
