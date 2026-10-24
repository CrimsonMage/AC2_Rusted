use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;

use crate::db_type::ResourceTypeId;
use crate::ResourceKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedResourceRef {
    file_kind: DatFileKind,
    did: DataId,
    type_id: u32,
}

impl TypedResourceRef {
    pub const fn new(file_kind: DatFileKind, did: DataId, type_id: u32) -> Self {
        Self {
            file_kind,
            did,
            type_id,
        }
    }

    pub const fn from_resource_type(
        file_kind: DatFileKind,
        did: DataId,
        resource_type_id: ResourceTypeId,
    ) -> Self {
        Self::new(file_kind, did, resource_type_id as u32)
    }

    pub const fn file_kind(self) -> DatFileKind {
        self.file_kind
    }

    pub const fn data_id(self) -> DataId {
        self.did
    }

    pub const fn type_id(self) -> u32 {
        self.type_id
    }

    pub const fn qualified_data_id(self) -> QualifiedDataId {
        QualifiedDataId::new(self.did, self.type_id)
    }

    pub const fn resource_key(self) -> ResourceKey {
        ResourceKey::new(self.file_kind, self.qualified_data_id())
    }

    pub const fn resource_type_id(self) -> Option<ResourceTypeId> {
        ResourceTypeId::from_type_id(self.type_id)
    }
}

#[cfg(test)]
mod tests {
    use ac2_core::DataId;
    use ac2_dat::DatFileKind;

    use crate::db_type::ResourceTypeId;

    use super::TypedResourceRef;

    #[test]
    fn preserves_known_type_metadata() {
        let reference = TypedResourceRef::from_resource_type(
            DatFileKind::Portal,
            DataId::new(0x2200_0001),
            ResourceTypeId::FixtureA,
        );

        assert_eq!(reference.file_kind(), DatFileKind::Portal);
        assert_eq!(reference.data_id(), DataId::new(0x2200_0001));
        assert_eq!(reference.type_id(), ResourceTypeId::FixtureA as u32);
        assert_eq!(reference.resource_type_id(), Some(ResourceTypeId::FixtureA));
        assert_eq!(
            reference.resource_key().type_id(),
            ResourceTypeId::FixtureA as u32
        );
    }
}
