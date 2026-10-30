//! `PhysicsMaterial` (DB type `0x40000033`, the `.pmat` records of DIDs `0x37xxxxxx`): the
//! material a landscape vertex, a water type or an interior cell names. Its body is the DBObj
//! identity (`DBObj::UnPack_Type`: the DID, and no category for this type) then the material's
//! `PropertyCollection` (`PhysicsMaterial::StreamPack`). The collision report reads its property
//! groups (`CCollisionManager::UpdatePropertyArray`).

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    read_ac2_property_groups, Ac2PropertyGroup, Ac2PropertyValue, ResourceError,
    ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId, TypedResourceRef,
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
    /// `PhysicsMaterial::UpdateCollection` -> `PropertyCollection::UpdateCollection`: each
    /// group of `update` merges into the group of its name (a property replaces the one of its
    /// name or is appended); a group not yet present is appended whole.
    pub fn update_collection(&mut self, update: &[Ac2PropertyGroup]) {
        ac2_db::update_ac2_property_collection(&mut self.groups, update);
    }

    /// `PhysicsMaterial::UpdateProperty` -> `PropertyCollection::UpdateProperty`: the property
    /// replaces the one of its name in `group` (the group `MasterProperty` files it under), or is
    /// appended; a group the material lacks is made.
    pub fn update_property(&mut self, group: u32, property: &Ac2PropertyValue) {
        self.update_collection(&[Ac2PropertyGroup {
            name: group,
            properties: vec![property.clone()],
        }]);
    }

    /// `PhysicsMaterial::GetProperty` -> `PropertyCollection::GetProperty`: the property of
    /// `name`, looked up in the group `MasterProperty` files it under; `None` when the name has
    /// no desc or the material holds no such property.
    pub fn get_property(&self, name: u32) -> Option<&Ac2PropertyValue> {
        let group = ac2_db::ac2_property_desc(name)?.group;
        self.groups
            .iter()
            .find(|g| g.name == group)?
            .properties
            .iter()
            .find(|p| p.name == name)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn int(name: u32, value: u32) -> Ac2PropertyValue {
        Ac2PropertyValue {
            name,
            value_type: 0x4000_0002,
            bytes: value.to_le_bytes().to_vec().into_boxed_slice(),
        }
    }

    #[test]
    fn a_property_replaces_the_one_of_its_name_or_is_appended() {
        let mut material = PhysicsMaterial::with_id(3);
        material.update_property(7, &int(0x10, 1));
        material.update_property(7, &int(0x11, 2));
        material.update_property(7, &int(0x10, 9));
        material.update_property(8, &int(0x10, 5));
        assert_eq!(material.groups.len(), 2);
        assert_eq!(
            material.groups[0].properties,
            vec![int(0x10, 9), int(0x11, 2)]
        );
        assert_eq!(material.groups[1].properties, vec![int(0x10, 5)]);
    }

    #[test]
    fn a_collection_merges_group_by_group() {
        let mut material = PhysicsMaterial::with_id(3);
        material.update_property(7, &int(0x10, 1));
        material.update_collection(&[
            Ac2PropertyGroup {
                name: 7,
                properties: vec![int(0x12, 4)],
            },
            Ac2PropertyGroup {
                name: 9,
                properties: vec![int(0x20, 6)],
            },
        ]);
        assert_eq!(material.groups.len(), 2);
        assert_eq!(
            material.groups[0].properties,
            vec![int(0x10, 1), int(0x12, 4)]
        );
        assert_eq!(material.groups[1].name, 9);
    }

    #[test]
    fn a_name_without_a_desc_has_no_property() {
        // `GetProperty` files the name under the group `MasterProperty` gives it; with no table
        // installed, no name has a group.
        let mut material = PhysicsMaterial::with_id(3);
        material.update_property(7, &int(0x10, 1));
        assert!(material.get_property(0x10).is_none());
    }
}
