use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;

use crate::db_type::ResourceTypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    file_kind: DatFileKind,
    qdid: QualifiedDataId,
}

impl ResourceKey {
    pub const fn new(file_kind: DatFileKind, qdid: QualifiedDataId) -> Self {
        Self { file_kind, qdid }
    }

    pub const fn file_kind(self) -> DatFileKind {
        self.file_kind
    }

    pub const fn qualified_data_id(self) -> QualifiedDataId {
        self.qdid
    }

    pub const fn data_id(self) -> DataId {
        self.qdid.data_id()
    }

    pub const fn type_id(self) -> u32 {
        self.qdid.type_id()
    }

    pub const fn is_invalid(self) -> bool {
        self.qdid.is_invalid()
    }

    pub const fn resource_type(self) -> Option<ResourceTypeId> {
        ResourceTypeId::from_type_id(self.qdid.type_id())
    }
}

#[cfg(test)]
mod tests {
    use ac2_core::{DataId, QualifiedDataId};
    use ac2_dat::DatFileKind;

    use crate::db_type::ResourceTypeId;

    use super::ResourceKey;

    #[test]
    fn preserves_file_kind_and_qdid_parts() {
        let qdid = QualifiedDataId::new(DataId::new(0x1234_5678), ResourceTypeId::FixtureB as u32);
        let key = ResourceKey::new(DatFileKind::Portal, qdid);

        assert_eq!(key.file_kind(), DatFileKind::Portal);
        assert_eq!(key.qualified_data_id(), qdid);
        assert_eq!(key.data_id().raw(), 0x1234_5678);
        assert_eq!(key.type_id(), ResourceTypeId::FixtureB as u32);
        assert_eq!(key.resource_type(), Some(ResourceTypeId::FixtureB));
    }

    #[test]
    fn preserves_unknown_type_ids() {
        let qdid = QualifiedDataId::new(DataId::new(0x8765_4321), 0x1FFF_FFFF);
        let key = ResourceKey::new(DatFileKind::Cell, qdid);

        assert_eq!(key.type_id(), 0x1FFF_FFFF);
        assert_eq!(key.resource_type(), None);
    }
}
