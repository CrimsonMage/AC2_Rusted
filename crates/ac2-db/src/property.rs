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
pub(crate) const TYPE_BOOL: u32 = 0x4000_0001;
pub(crate) const TYPE_INTEGER: u32 = 0x4000_0002;
pub(crate) const TYPE_FLOAT: u32 = 0x4000_0003;
pub(crate) const TYPE_VECTOR: u32 = 0x4000_0004;
pub(crate) const TYPE_COLOR: u32 = 0x4000_0005;
pub(crate) const TYPE_STRING: u32 = 0x4000_0006;
pub(crate) const TYPE_ENUM: u32 = 0x4000_0007;
pub(crate) const TYPE_DATA_FILE: u32 = 0x4000_0008;
pub(crate) const TYPE_WAVEFORM: u32 = 0x4000_0009;
pub(crate) const TYPE_STRING_INFO: u32 = 0x4000_000A;
pub(crate) const TYPE_PACKAGE_ID: u32 = 0x4000_000B;
pub(crate) const TYPE_LONG_INTEGER: u32 = 0x4000_000D;
pub(crate) const TYPE_POSITION: u32 = 0x4000_000E;

/// The property group of ethereal properties (`CPhysicsObj::ApplyProperty`'s `0x40000003` case):
/// a bool property of this group sets the bit `BasePropertyDesc::m_data` names in a body's
/// blocking mask (`CPhysicsObj::ApplyEtherealProperty`).
pub const ETHEREAL_PROPERTY_GROUP: u32 = 0x4000_0003;

static PROPERTY_TYPES: OnceLock<Arc<Ac2PropertyTypes>> = OnceLock::new();

/// `BasePropertyDesc::m_report_to_weenie` (`PropertyWeenieReportType::Type`): reported to the
/// weenie, inquirable only, or ignored.
pub const REPORT_REPORTABLE: u32 = 0;
pub const REPORT_INQUIRABLE: u32 = 1;
pub const REPORT_IGNORED: u32 = 2;

/// One `BasePropertyDesc` of `MasterProperty`: its value type, its group, its data word and its
/// report-to-weenie type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2PropertyDesc {
    pub value_type: u32,
    pub group: u32,
    pub data: u32,
    pub report_to_weenie: u32,
}

/// Property name -> `BasePropertyDesc`, from `MasterProperty::m_properties`.
#[derive(Debug, Default)]
pub struct Ac2PropertyTypes {
    descs: HashMap<u32, Ac2PropertyDesc>,
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
        // `PAutoGrowHashTable<ulong, BasePropertyDesc>`: name, then type, group, data word
        // (`m_data`, the bit an ethereal property sets), report-to-weenie and required flags.
        let entries = (cursor.read_u32()? & 0xFFFF) as usize;
        let mut descs = HashMap::with_capacity(entries);
        for _ in 0..entries {
            let name = cursor.read_u32()?;
            let value_type = cursor.read_u32()?;
            let group = cursor.read_u32()?;
            let data = cursor.read_u32()?;
            let report_to_weenie = cursor.read_u32()?;
            // `m_bRequired` follows; the port does not read it.
            cursor.read_bytes(4)?;
            descs.insert(
                name,
                Ac2PropertyDesc {
                    value_type,
                    group,
                    data,
                    report_to_weenie,
                },
            );
        }
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after MasterProperty"));
        }
        Ok(Self { descs })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.descs.len() * 24
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
        descs: table.descs.clone(),
    };
    let _ = PROPERTY_TYPES.set(Arc::new(owned));
    Ok(true)
}

/// `MasterProperty::InqPropertyType` against the installed table: the value type of property
/// `name`, if the table is installed and names it.
pub fn ac2_property_type(name: u32) -> Option<u32> {
    ac2_property_desc(name).map(|desc| desc.value_type)
}

/// The `BasePropertyDesc` of property `name` from the installed table.
pub fn ac2_property_desc(name: u32) -> Option<Ac2PropertyDesc> {
    PROPERTY_TYPES.get()?.descs.get(&name).copied()
}

/// `BaseProperty::IsInquirable` (`MasterProperty::IsInquirable`): the property has a desc that
/// is not ignored. Without a desc the answer is no.
pub fn ac2_property_inquirable(name: u32) -> bool {
    ac2_property_desc(name).is_some_and(|desc| desc.report_to_weenie != REPORT_IGNORED)
}

/// `BaseProperty::IsReportedToWeenie` (`MasterProperty::IsReportedToWeenie`): the desc is
/// `Reportable`.
pub fn ac2_property_reported_to_weenie(name: u32) -> bool {
    ac2_property_desc(name).is_some_and(|desc| desc.report_to_weenie == REPORT_REPORTABLE)
}

/// The blocking-mask bits a `PropertyCollection`'s groups set on a physics body
/// (`CPhysicsObj::ApplyPropertyCollection` -> `ApplyEtherealProperty`): each property of
/// the ethereal group whose desc is a bool sets bit `data` when its value is true and that bit
/// is below 64 (`EtherealPhysicsType::SetBit` sets nothing for a false value).
pub fn ac2_ethereal_mask(groups: &[Ac2PropertyGroup]) -> u64 {
    ethereal_mask_with(groups, ac2_property_desc)
}

/// [`ac2_ethereal_mask`] against the descs `desc_of` gives (the installed table in the client).
pub fn ethereal_mask_with(
    groups: &[Ac2PropertyGroup],
    desc_of: impl Fn(u32) -> Option<Ac2PropertyDesc>,
) -> u64 {
    groups
        .iter()
        .flat_map(|group| &group.properties)
        .filter_map(|property| desc_of(property.name).map(|desc| (desc, property)))
        .filter(|(desc, _)| desc.group == ETHEREAL_PROPERTY_GROUP && desc.value_type == TYPE_BOOL)
        .filter(|(desc, _)| desc.data < 64)
        .filter(|(_, property)| property.word().is_some_and(|value| value != 0))
        .fold(0, |mask, (desc, _)| mask | (1 << desc.data))
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
                .descs
                .get(&name)
                .map(|desc| desc.value_type)
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

    /// A float property's value (`BasePropertyValue::GetValue(float&)`); `None` for other types.
    pub fn float(&self) -> Option<f32> {
        (self.value_type == TYPE_FLOAT)
            .then(|| self.float_at(0))
            .flatten()
    }

    /// A word-sized integer property's value as `BasePropertyValue::GetValue(ulong&)` widens it
    /// (`Integer`, `Enum`, `DataFile` and `PackageID` words, `LongInteger` in full); `None` for
    /// other types.
    pub fn integer(&self) -> Option<u64> {
        match self.value_type {
            TYPE_INTEGER | TYPE_ENUM | TYPE_DATA_FILE | TYPE_PACKAGE_ID => {
                self.word().map(u64::from)
            }
            TYPE_LONG_INTEGER => self
                .bytes
                .get(..8)
                .map(|b| u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])),
            _ => None,
        }
    }

    /// A vector property's value (`BasePropertyValue::GetValue(Vector&)`); `None` for other types.
    pub fn vector(&self) -> Option<[f32; 3]> {
        (self.value_type == TYPE_VECTOR)
            .then(|| Some([self.float_at(0)?, self.float_at(1)?, self.float_at(2)?]))
            .flatten()
    }
}

/// One `PropertyGroup`: its group name and properties in stream order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ac2PropertyGroup {
    pub name: u32,
    pub properties: Vec<Ac2PropertyValue>,
}

/// `PropertyCollection::UpdateCollection`: each group of `update` merges into the group of the
/// same name (`PropertyGroup::UpdateProperty`: a property replaces the one of its name, or is
/// appended), and a group not yet present is appended whole.
pub fn update_ac2_property_collection(
    base: &mut Vec<Ac2PropertyGroup>,
    update: &[Ac2PropertyGroup],
) {
    for group in update {
        match base.iter_mut().find(|g| g.name == group.name) {
            Some(existing) => {
                for property in &group.properties {
                    match existing
                        .properties
                        .iter_mut()
                        .find(|p| p.name == property.name)
                    {
                        Some(slot) => *slot = property.clone(),
                        None => existing.properties.push(property.clone()),
                    }
                }
            }
            None => base.push(group.clone()),
        }
    }
}

/// Reads a `PropertyCollection` into its groups.
pub fn read_ac2_property_groups(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<Vec<Ac2PropertyGroup>, &'static str> {
    let types = PROPERTY_TYPES
        .get()
        .ok_or("AC2 MasterProperty table is not installed")?;
    let mut groups = Vec::new();
    for _ in 0..count(cursor, 8)? {
        let mut group = Ac2PropertyGroup {
            name: word(cursor)?,
            properties: Vec::new(),
        };
        for _ in 0..count(cursor, 4)? {
            let name = word(cursor)?;
            let value_type = types
                .descs
                .get(&name)
                .map(|desc| desc.value_type)
                .ok_or("property name is not in MasterProperty")?;
            let start = cursor.position();
            skip_value(cursor, value_type)?;
            let end = cursor.position();
            group.properties.push(Ac2PropertyValue {
                name,
                value_type,
                bytes: cursor.bytes()[start..end].into(),
            });
        }
        groups.push(group);
    }
    Ok(groups)
}

/// Reads a `PropertyCollection` into its properties, in stream order across groups.
pub fn read_ac2_property_collection(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<Vec<Ac2PropertyValue>, &'static str> {
    Ok(read_ac2_property_groups(cursor)?
        .into_iter()
        .flat_map(|group| group.properties)
        .collect())
}
