//! `StringTable` (`DB_TYPE_STRING_TABLE` 0x40000026, DIDs 0x25000000..=0x26FFFFFE in the local
//! DAT): the client's localised text, keyed by string id.
//!
//! `StringTable::UnPack`: the `DBObj` DID, four header fields (`currentStateID` u32, `m_base`
//! u16, `m_leadingZero` u32, `m_numDecimalDigits` u16; negative number settings defer to the
//! `StringState`), 4-byte alignment, the string count, then each (u32 key,
//! `StringTableString`).
//! `StringTableString::UnPack`: the table DID, a u16 whose low 15 bits count the pieces and whose
//! top bit says variable names follow; each piece is an i16 flag, alignment, and (flag non-zero)
//! a `PStringBase<unsigned short>`; alignment; a u16 count of variable ids (u32 each); alignment;
//! then the variable names (`PSmartArray<PStringBase<unsigned short>>`: u32 count, strings).
//! Every string is descrambled as `PStringBase<unsigned short>::UnPack` does.

use std::collections::HashMap;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};

use crate::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

/// `DB_TYPE_STRING_TABLE`'s DID range.
pub const STRING_TABLE_FIRST: u32 = 0x2500_0000;
pub const STRING_TABLE_LAST: u32 = 0x26FF_FFFE;
const MAX_ITEMS: usize = 1 << 20;

/// One `StringTableString`: text pieces with variables between them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringTableString {
    /// `table`: the string table the entry belongs to (0 for the containing one).
    pub table: u32,
    /// `strings`: UTF-16 pieces; `None` where the pieces array holds a null.
    pub pieces: Vec<Option<Vec<u16>>>,
    /// `variables`: the variable ids between pieces.
    pub variables: Vec<u32>,
    /// `varNames`, when packed.
    pub variable_names: Vec<Vec<u16>>,
}

impl StringTableString {
    /// The pieces joined, for strings without variables (UI labels).
    pub fn text(&self) -> String {
        let units: Vec<u16> = self.pieces.iter().flatten().flatten().copied().collect();
        String::from_utf16_lossy(&units)
    }
}

/// A `StringTable`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringTable {
    /// The four header words in pack order: `currentStateID`, `m_base`, `m_leadingZero`,
    /// `m_numDecimalDigits` (see [`StringTable::number_base`] and its siblings).
    pub header: (u32, u16, u32, u16),
    pub strings: HashMap<u32, StringTableString>,
}

impl StringTable {
    /// `StringTable::GetStringTableString`.
    pub fn get(&self, id: u32) -> Option<&StringTableString> {
        self.strings.get(&id)
    }
}

fn bad(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: StringTable::TYPE_NAME,
        context,
    }
}

/// `PStringBase<unsigned short>::UnPack`: a u16 length (`0xFFFF` escapes to a u32), the UTF-16
/// units descrambled, then 4-byte alignment.
pub fn read_wide_pstring(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<Vec<u16>, ac2_dat::ArchiveError> {
    let mut len = usize::from(cursor.read_u16()?);
    if len == 0xFFFF {
        len = cursor.read_u32()? as usize;
    }
    let mut bytes = cursor.read_bytes(len * 2)?.to_vec();
    ac2_core::pstring::unmunge(&mut bytes);
    cursor.align_to(4)?;
    Ok(bytes
        .chunks_exact(2)
        .map(|unit| u16::from_le_bytes([unit[0], unit[1]]))
        .collect())
}

fn read_entry(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> ResourceResult<StringTableString> {
    let table = cursor.read_u32()?;
    let header = cursor.read_u16()?;
    let mut entry = StringTableString {
        table,
        ..Default::default()
    };
    for _ in 0..(header & 0x7FFF) {
        let present = cursor.read_u16()? != 0;
        cursor.align_to(4)?;
        entry.pieces.push(if present {
            Some(read_wide_pstring(cursor)?)
        } else {
            None
        });
    }
    cursor.align_to(4)?;
    let variables = usize::from(cursor.read_u16()?);
    if variables * 4 > cursor.remaining() {
        return Err(bad(qdid, "variable count"));
    }
    for _ in 0..variables {
        entry.variables.push(cursor.read_u32()?);
    }
    cursor.align_to(4)?;
    if header & 0x8000 != 0 {
        let names = cursor.read_u32()? as usize;
        if names > MAX_ITEMS || names * 2 > cursor.remaining() {
            return Err(bad(qdid, "variable name count"));
        }
        for _ in 0..names {
            entry.variable_names.push(read_wide_pstring(cursor)?);
        }
        cursor.align_to(4)?;
    }
    Ok(entry)
}

impl ResourcePackedDecode for StringTable {
    const TYPE_NAME: &'static str = "AC2 StringTable";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad(qdid, "embedded DID did not match DAT identity"));
        }
        let header = (
            cursor.read_u32()?,
            cursor.read_u16()?,
            cursor.read_u32()?,
            cursor.read_u16()?,
        );
        cursor.align_to(4)?;
        let count = cursor.read_u32()? as usize;
        if count > MAX_ITEMS || count * 8 > cursor.remaining() {
            return Err(bad(qdid, "string count"));
        }
        let mut strings = HashMap::with_capacity(count);
        for _ in 0..count {
            let key = cursor.read_u32()?;
            let entry = read_entry(&mut cursor, qdid)?;
            // The table keeps the first entry of a repeated key.
            strings.entry(key).or_insert(entry);
        }
        if cursor.remaining() != 0 {
            return Err(bad(qdid, "trailing bytes after StringTable"));
        }
        Ok(Self { header, strings })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .strings
                .values()
                .map(|s| {
                    64 + s
                        .pieces
                        .iter()
                        .flatten()
                        .map(|p| p.len() * 2)
                        .sum::<usize>()
                })
                .sum::<usize>()
    }
}

/// The string table `did` from the local DAT.
pub fn load_string_table(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<StringTable>>> {
    store.load_cached_packed::<StringTable>(TypedResourceRef::new(DatFileKind::Local, did, 0))
}
