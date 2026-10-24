//! Asheron's Call 2 property system: `MasterProperty` names every property's value
//! type, which `PropertyCollection`s (`PropertyGroup`s of `BaseProperty`s) need to be read. The
//! table is installed once per process from the mounted AC2 dataset, like the land lattice.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};
use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;

const MASTER_PROPERTY_DID: DataId = DataId::new(0x3400_0000);
const MAX_ITEMS: usize = 1 << 16;

// `MasterProperty::AllocatePropertyValue` type enums.
const TYPE_BOOL: u32 = 0x4000_0001;
const TYPE_INTEGER: u32 = 0x4000_0002;
const TYPE_FLOAT: u32 = 0x4000_0003;
const TYPE_VECTOR: u32 = 0x4000_0004;
const TYPE_COLOR: u32 = 0x4000_0005;
const TYPE_STRING: u32 = 0x4000_0006;
const TYPE_ENUM: u32 = 0x4000_0007;
const TYPE_DATA_FILE: u32 = 0x4000_0008;
const TYPE_WAVEFORM: u32 = 0x4000_0009;
const TYPE_STRING_INFO: u32 = 0x4000_000A;
const TYPE_PACKAGE_ID: u32 = 0x4000_000B;
const TYPE_LONG_INTEGER: u32 = 0x4000_000D;
const TYPE_POSITION: u32 = 0x4000_000E;

static PROPERTY_TYPES: OnceLock<Arc<Ac2PropertyTypes>> = OnceLock::new();

/// Property name -> value type, from `MasterProperty::m_properties`.
#[derive(Debug, Default)]
pub struct Ac2PropertyTypes {
    types: HashMap<u32, u32>,
}

impl ResourcePackedDecode for Ac2PropertyTypes {
    const TYPE_NAME: &'static str = "AC2 MasterProperty";
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
        // `EnumMapper`: base enum, reserved word, then (id, name) pairs of the property names.
        let _base = cursor.read_u32()?;
        let _reserved = cursor.read_u32()?;
        let names = count(&mut cursor, 6).map_err(malformed)?;
        for _ in 0..names {
            cursor.read_u32()?;
            skip_pstring(&mut cursor, 1).map_err(malformed)?;
        }
        // `PAutoGrowHashTable<ulong, BasePropertyDesc>`: name, then type, group, default-value
        // word, report-to-weenie and required flags.
        let entries = (cursor.read_u32()? & 0xFFFF) as usize;
        let mut types = HashMap::with_capacity(entries);
        for _ in 0..entries {
            let name = cursor.read_u32()?;
            types.insert(name, cursor.read_u32()?);
            cursor.read_bytes(16)?;
        }
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after MasterProperty"));
        }
        Ok(Self { types })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.types.len() * 16
    }
}

/// Installs the AC2 property type table from `store`. Returns whether a table is available.
pub fn install_ac2_property_types(store: &ResourceStore) -> ResourceResult<bool> {
    if PROPERTY_TYPES.get().is_some() {
        return Ok(true);
    }
    let Some(table) = store.load_cached_packed::<Ac2PropertyTypes>(TypedResourceRef::new(
        DatFileKind::Portal,
        MASTER_PROPERTY_DID,
        0,
    ))?
    else {
        return Ok(false);
    };
    let owned = Ac2PropertyTypes {
        types: table.types.clone(),
    };
    let _ = PROPERTY_TYPES.set(Arc::new(owned));
    Ok(true)
}

/// Steps over a `PropertyCollection` embedded in another record (see
/// [`skip_property_collection`]).
pub fn skip_ac2_property_collection(cursor: &mut ArchiveCursor<'_>) -> Result<(), &'static str> {
    skip_property_collection(cursor)
}

/// Steps over a `PropertyCollection`: counted `PropertyGroup`s of (group name, counted
/// `BaseProperty`s of (name, typed value)).
pub fn skip_property_collection(cursor: &mut ArchiveCursor<'_>) -> Result<(), &'static str> {
    let types = PROPERTY_TYPES
        .get()
        .ok_or("AC2 MasterProperty table is not installed")?;
    for _ in 0..count(cursor, 8)? {
        word(cursor)?;
        for _ in 0..count(cursor, 4)? {
            let name = word(cursor)?;
            let value_type = types
                .types
                .get(&name)
                .copied()
                .ok_or("property name is not in MasterProperty")?;
            skip_value(cursor, value_type)?;
        }
    }
    Ok(())
}

fn skip_value(cursor: &mut ArchiveCursor<'_>, value_type: u32) -> Result<(), &'static str> {
    let len = match value_type {
        TYPE_BOOL | TYPE_INTEGER | TYPE_FLOAT | TYPE_COLOR | TYPE_ENUM | TYPE_DATA_FILE
        | TYPE_PACKAGE_ID => 4,
        TYPE_VECTOR => 12,
        TYPE_LONG_INTEGER => 8,
        TYPE_WAVEFORM => 44,
        // `Position`: cell id and `Frame` (origin, quaternion).
        TYPE_POSITION => 32,
        TYPE_STRING => return skip_pstring(cursor, 1),
        TYPE_STRING_INFO => return skip_string_info(cursor),
        _ => return Err("unknown AC2 property value type"),
    };
    skip(cursor, len)
}

/// `StringInfo::UnPack`: table and string ids, variable and literal counts, then the wide
/// literal. Variables (formatted substitutions) are not supported.
fn skip_string_info(cursor: &mut ArchiveCursor<'_>) -> Result<(), &'static str> {
    skip(cursor, 8)?;
    let counts = word(cursor)?;
    let (variables, literal) = (counts & 0xFFFF, counts >> 16);
    if literal != 0 {
        skip_pstring(cursor, 2)?;
    }
    if variables != 0 {
        return Err("StringInfo variables are not supported");
    }
    Ok(())
}

/// `PStringBase::UnPack`: u16 length (0xFFFF escapes to a u32), characters, 4-byte align.
fn skip_pstring(cursor: &mut ArchiveCursor<'_>, char_bytes: usize) -> Result<(), &'static str> {
    let short = cursor.read_u16().map_err(|_| "record ended early")?;
    let len = if short == 0xFFFF {
        word(cursor)? as usize
    } else {
        usize::from(short)
    };
    skip(cursor, len * char_bytes)?;
    cursor.align_to(4).map_err(|_| "record ended early")
}

fn count(cursor: &mut ArchiveCursor<'_>, item: usize) -> Result<usize, &'static str> {
    let n = word(cursor)? as usize;
    if n > MAX_ITEMS || n * item > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(n)
}

fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, &'static str> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn skip(cursor: &mut ArchiveCursor<'_>, len: usize) -> Result<(), &'static str> {
    cursor
        .read_bytes(len)
        .map(|_| ())
        .map_err(|_| "record ended early")
}

/// One `BaseProperty` of a `PropertyCollection`: its name, `MasterProperty` value type and the
/// packed value bytes (a 4-byte word for bool/integer/float/color/enum/data-file values, 12 for a
/// vector, 44 for a `Waveform`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac2PropertyValue {
    pub name: u32,
    pub value_type: u32,
    pub bytes: Box<[u8]>,
}

impl Ac2PropertyValue {
    /// The first packed word (bool, integer, enum, color32 or data-file value).
    pub fn word(&self) -> Option<u32> {
        self.bytes
            .get(..4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// The packed `f32` at word `index` (a float value, or a `Waveform` term).
    pub fn float_at(&self, index: usize) -> Option<f32> {
        self.bytes
            .get(index * 4..index * 4 + 4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

/// Reads a `PropertyCollection` into its properties, in stream order across groups.
pub fn read_ac2_property_collection(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<Vec<Ac2PropertyValue>, &'static str> {
    let types = PROPERTY_TYPES
        .get()
        .ok_or("AC2 MasterProperty table is not installed")?;
    let mut values = Vec::new();
    for _ in 0..count(cursor, 8)? {
        word(cursor)?;
        for _ in 0..count(cursor, 4)? {
            let name = word(cursor)?;
            let value_type = types
                .types
                .get(&name)
                .copied()
                .ok_or("property name is not in MasterProperty")?;
            let start = cursor.position();
            skip_value(cursor, value_type)?;
            let end = cursor.position();
            values.push(Ac2PropertyValue {
                name,
                value_type,
                bytes: cursor.bytes()[start..end].into(),
            });
        }
    }
    Ok(values)
}
