use std::fmt;

use crate::constants::{
    RETAIL_INVALID_CELL_ID, RETAIL_INVALID_DATA_ID, RETAIL_INVALID_INSTANCE_ID,
    RETAIL_INVALID_OBJECT_ID, RETAIL_INVALID_QUALIFIED_DATA_TYPE,
};

macro_rules! define_hex_u32_id {
    (
        $(#[$meta:meta])*
        $name:ident,
        $invalid_raw:expr
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const INVALID_RAW: u32 = $invalid_raw;
            pub const INVALID: Self = Self(Self::INVALID_RAW);

            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u32 {
                self.0
            }

            pub const fn is_invalid(self) -> bool {
                self.0 == Self::INVALID_RAW
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::INVALID
            }
        }

        impl From<u32> for $name {
            fn from(value: u32) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> Self {
                value.raw()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:08X}", self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}(0x{:08X})", stringify!($name), self.0)
            }
        }

        impl fmt::UpperHex for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::UpperHex::fmt(&self.0, f)
            }
        }

        impl fmt::LowerHex for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::LowerHex::fmt(&self.0, f)
            }
        }
    };
}

define_hex_u32_id!(
    /// Shared data-file/resource identifier.
    DataId,
    RETAIL_INVALID_DATA_ID
);

define_hex_u32_id!(
    /// Shared runtime instance identifier.
    InstanceId,
    RETAIL_INVALID_INSTANCE_ID
);

define_hex_u32_id!(
    /// Shared live object identifier.
    ObjectId,
    RETAIL_INVALID_OBJECT_ID
);

define_hex_u32_id!(
    /// Shared world cell identifier.
    CellId,
    RETAIL_INVALID_CELL_ID
);

define_hex_u32_id!(
    /// Shared archive/resource version-handle identifier.
    VersionHandle,
    RETAIL_INVALID_DATA_ID
);

impl DataId {
    pub const NULL: Self = Self::INVALID;

    pub const fn is_null(self) -> bool {
        self.is_invalid()
    }
}

impl InstanceId {
    pub const NULL: Self = Self::INVALID;

    pub const fn is_null(self) -> bool {
        self.is_invalid()
    }
}

impl ObjectId {
    pub const NULL: Self = Self::INVALID;

    pub const fn is_null(self) -> bool {
        self.is_invalid()
    }

    pub const fn is_player(self) -> bool {
        (self.raw() & 0xF000_0000) == 0x5000_0000
    }

    pub const fn is_item(self) -> bool {
        let raw = self.raw();
        raw > 0 && raw < 0x5000_0000
    }
}

impl CellId {
    pub const fn is_valid(self) -> bool {
        !self.is_invalid()
    }
}

impl VersionHandle {
    // Replaces IDClass<_tagVersionHandle,32,0>.
    pub const NULL: Self = Self::INVALID;

    pub const fn is_null(self) -> bool {
        self.is_invalid()
    }
}

/// A passive pairing of a data id with its database/type discriminator.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct QualifiedDataId {
    type_id: u32,
    data_id: DataId,
}

impl QualifiedDataId {
    pub const INVALID: Self = Self {
        type_id: RETAIL_INVALID_QUALIFIED_DATA_TYPE,
        data_id: DataId::INVALID,
    };

    pub const fn new(data_id: DataId, type_id: u32) -> Self {
        Self { type_id, data_id }
    }

    pub const fn from_raw_parts(data_id: u32, type_id: u32) -> Self {
        Self::new(DataId::new(data_id), type_id)
    }

    pub const fn data_id(self) -> DataId {
        self.data_id
    }

    pub const fn type_id(self) -> u32 {
        self.type_id
    }

    pub const fn raw_parts(self) -> (u32, u32) {
        (self.data_id.raw(), self.type_id)
    }

    pub const fn is_invalid(self) -> bool {
        self.data_id.is_invalid() && self.type_id == RETAIL_INVALID_QUALIFIED_DATA_TYPE
    }
}

impl fmt::Display for QualifiedDataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:08X}", self.data_id, self.type_id)
    }
}

impl fmt::Debug for QualifiedDataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QualifiedDataId {{ data_id: 0x{:08X}, type_id: 0x{:08X} }}",
            self.data_id.raw(),
            self.type_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CellId, DataId, InstanceId, ObjectId, QualifiedDataId, VersionHandle};

    #[test]
    fn ids_round_trip_and_format_as_hex() {
        let did = DataId::new(0x0102_0304);
        let instance = InstanceId::new(0x1122_3344);

        assert_eq!(did.raw(), 0x0102_0304);
        assert_eq!(u32::from(instance), 0x1122_3344);
        assert_eq!(did.to_string(), "01020304");
        assert_eq!(format!("{did:?}"), "DataId(0x01020304)");
    }

    #[test]
    fn cell_id_defaults_to_retail_invalid_value() {
        let cell = CellId::default();

        assert_eq!(cell.raw(), 0x0000_FFFF);
        assert!(cell.is_invalid());
        assert!(!cell.is_valid());
    }

    #[test]
    fn object_id_helpers_preserve_retail_ranges() {
        let player = ObjectId::new(0x5000_1234);
        let item = ObjectId::new(0x0000_1234);

        assert!(player.is_player());
        assert!(!player.is_item());
        assert!(item.is_item());
        assert!(!item.is_player());
    }

    #[test]
    fn qualified_data_id_preserves_raw_parts() {
        let qdid = QualifiedDataId::new(DataId::new(0x0200_0001), 0x0000_000C);

        assert_eq!(qdid.data_id(), DataId::new(0x0200_0001));
        assert_eq!(qdid.type_id(), 0x0000_000C);
        assert_eq!(qdid.raw_parts(), (0x0200_0001, 0x0000_000C));
        assert!(!qdid.is_invalid());
    }

    #[test]
    fn version_handle_defaults_to_invalid() {
        let handle = VersionHandle::default();

        assert_eq!(handle.raw(), 0);
        assert!(handle.is_null());
        assert_eq!(format!("{handle:?}"), "VersionHandle(0x00000000)");
    }
}
