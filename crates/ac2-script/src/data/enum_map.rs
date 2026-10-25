//! `EnumMapper` (`DB_TYPE_ENUM_MAPPER`, 0x23xxxxxx) and `EnumIDMap` (`DB_TYPE_DID_MAPPER`,
//! 0x28xxxxxx): the enum name tables script glue looks values up in.

use ac2_core::QualifiedDataId;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

use crate::unpack::{Reader, UnpackResult};

/// `EnumMapper::StreamPack`: the DID, `m_base_emp_did`, then `m_id_to_string_map` (a u32 count
/// of id, `PStringBase<char>` pairs). Lookups fall through to the base mapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumMapper {
    pub did: u32,
    /// `m_base_emp_did`, 0 for none.
    pub base: u32,
    /// `m_id_to_string_map`, sorted by id (an `std::map`).
    pub names: Vec<(u32, Vec<u8>)>,
}

impl EnumMapper {
    pub fn unpack(bytes: &[u8]) -> UnpackResult<Self> {
        let mut r = Reader::new(bytes);
        let did = r.u32()?;
        let base = r.u32()?;
        let n = r.count(4, "enum mapper count")?;
        let mut names = (0..n)
            .map(|_| Ok((r.u32()?, r.pstring()?)))
            .collect::<UnpackResult<Vec<_>>>()?;
        r.finish("bytes left after the EnumMapper")?;
        names.sort_by_key(|(id, _)| *id);
        names.dedup_by_key(|(id, _)| *id);
        Ok(Self { did, base, names })
    }

    /// This mapper's own `GetString(id)`.
    pub fn name(&self, id: u32) -> Option<&[u8]> {
        self.names
            .binary_search_by_key(&id, |(i, _)| *i)
            .ok()
            .map(|i| self.names[i].1.as_slice())
    }

    /// This mapper's own `GetEnum(name)` (`CaseInsensitivePStringCompare`; `m_string_to_id_map`
    /// is filled in id order with `operator[]`, so the last id of a repeated name wins).
    pub fn value(&self, name: &[u8]) -> Option<u32> {
        self.names
            .iter()
            .rev()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(id, _)| *id)
    }
}

/// `EnumIDMap::StreamPack`: four `PHashTable`s, `m_EnumToID`, `m_EnumToName`,
/// `m_EnumToIDInternal`, `m_EnumToNameInternal`, kept in packed (iteration) order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnumIdMap {
    pub did: u32,
    pub to_id: Vec<(u32, u32)>,
    pub to_name: Vec<(u32, Vec<u8>)>,
    pub to_id_internal: Vec<(u32, u32)>,
    pub to_name_internal: Vec<(u32, Vec<u8>)>,
}

impl EnumIdMap {
    pub fn unpack(bytes: &[u8]) -> UnpackResult<Self> {
        let mut r = Reader::new(bytes);
        let did = r.u32()?;
        let ids = |r: &mut Reader<'_>| -> UnpackResult<Vec<(u32, u32)>> {
            let n = r.hash_count(8, "enum id count")?;
            (0..n).map(|_| Ok((r.u32()?, r.u32()?))).collect()
        };
        let names = |r: &mut Reader<'_>| -> UnpackResult<Vec<(u32, Vec<u8>)>> {
            let n = r.hash_count(6, "enum name count")?;
            (0..n).map(|_| Ok((r.u32()?, r.pstring()?))).collect()
        };
        let map = Self {
            did,
            to_id: ids(&mut r)?,
            to_name: names(&mut r)?,
            to_id_internal: ids(&mut r)?,
            to_name_internal: names(&mut r)?,
        };
        r.finish("bytes left after the EnumIDMap")?;
        Ok(map)
    }

    /// `EnumIDMap::EnumToDID` (`m_EnumToID`, then `m_EnumToIDInternal`).
    pub fn enum_to_did(&self, value: u32) -> Option<u32> {
        self.to_id
            .iter()
            .chain(&self.to_id_internal)
            .find(|(k, _)| *k == value)
            .map(|(_, v)| *v)
    }

    /// `EnumIDMap::GetName`.
    pub fn name(&self, value: u32) -> Option<&[u8]> {
        self.to_name
            .iter()
            .chain(&self.to_name_internal)
            .find(|(k, _)| *k == value)
            .map(|(_, n)| n.as_slice())
    }

    /// `EnumIDMap::NameToEnum` (`_stricmp`).
    pub fn name_to_enum(&self, name: &[u8]) -> Option<u32> {
        self.to_name
            .iter()
            .chain(&self.to_name_internal)
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(k, _)| *k)
    }
}

fn bad(qdid: QualifiedDataId, type_name: &'static str, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name,
        context,
    }
}

impl ResourcePackedDecode for EnumMapper {
    const TYPE_NAME: &'static str = "AC2 EnumMapper";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _v: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let map = Self::unpack(bytes).map_err(|e| bad(qdid, Self::TYPE_NAME, e.context))?;
        if map.did != qdid.data_id().raw() {
            return Err(bad(qdid, Self::TYPE_NAME, "embedded DID did not match"));
        }
        Ok(map)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.names.iter().map(|n| n.1.len() + 32).sum::<usize>()
    }
}

impl ResourcePackedDecode for EnumIdMap {
    const TYPE_NAME: &'static str = "AC2 EnumIDMap";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _v: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let map = Self::unpack(bytes).map_err(|e| bad(qdid, Self::TYPE_NAME, e.context))?;
        if map.did != qdid.data_id().raw() {
            return Err(bad(qdid, Self::TYPE_NAME, "embedded DID did not match"));
        }
        Ok(map)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.to_id.len() + self.to_id_internal.len()) * 8
            + self
                .to_name
                .iter()
                .chain(&self.to_name_internal)
                .map(|n| n.1.len() + 32)
                .sum::<usize>()
    }
}
