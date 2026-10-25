//! `PlacesTable` (`DB_TYPE_PLACES_TABLE` 0x40000054): the named places scripts look up
//! (`PlacesTable::InqPlace`), the one record `DBObj::GetByEnum(0x40000007,
//! UNIQUEDB_GROUP_ENUM, ...)` names.

use ac2_core::QualifiedDataId;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_net::protocol::Position;

use crate::unpack::{Reader, UnpackResult};

/// `PlacesTable_UniqueDBID`, in `UNIQUEDB_GROUP_ENUM`.
pub const PLACES_TABLE_ENUM: u32 = 0x4000_0007;
pub const UNIQUEDB_GROUP_ENUM: u32 = 0x4000_0002;

/// `PlacesTable::StreamPack`: the DID, then `m_places`, a `PHashTable<PStringBase<unsigned
/// short>, Position>` (keys lowercase), kept in packed (iteration) order.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacesTable {
    pub did: u32,
    pub bucket_index: u8,
    pub places: Vec<(Vec<u16>, Position)>,
}

impl PlacesTable {
    pub fn unpack(bytes: &[u8]) -> UnpackResult<Self> {
        let mut r = Reader::new(bytes);
        let did = r.u32()?;
        let (n, bucket_index) = r.hash_header(32, "places count")?;
        let places = (0..n)
            .map(|_| {
                let name = r.wpstring()?;
                let cell = r.u32()?;
                let mut f = [0.0f32; 7];
                for v in &mut f {
                    *v = r.f32()?;
                }
                let position = Position {
                    cell,
                    origin: [f[0], f[1], f[2]],
                    rotation: [f[3], f[4], f[5], f[6]],
                };
                Ok((name, position))
            })
            .collect::<UnpackResult<Vec<_>>>()?;
        r.finish("bytes left after the PlacesTable")?;
        Ok(Self {
            did,
            bucket_index,
            places,
        })
    }

    /// `PlacesTable::GetPlace`: the name lowercased (`_wcslwr`, "C" locale: A-Z), then looked
    /// up.
    pub fn place(&self, name: &[u16]) -> Option<&Position> {
        let lower: Vec<u16> = name
            .iter()
            .map(|&c| match c {
                0x41..=0x5A => c + 0x20,
                _ => c,
            })
            .collect();
        self.places
            .iter()
            .find(|(k, _)| *k == lower)
            .map(|(_, p)| p)
    }
}

impl ResourcePackedDecode for PlacesTable {
    const TYPE_NAME: &'static str = "AC2 PlacesTable";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _v: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let table = Self::unpack(bytes).map_err(|e| bad(e.context))?;
        if table.did != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match"));
        }
        Ok(table)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .places
                .iter()
                .map(|(k, _)| k.len() * 2 + 64)
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A table packed as `PlacesTable::StreamPack` writes it decodes, and lookups lowercase
    /// A-Z only.
    #[test]
    fn a_packed_table_decodes_and_lowercases_names() {
        let mut b = Vec::new();
        b.extend(0x0E00_0004u32.to_le_bytes());
        b.extend((1u32 | 2 << 24).to_le_bytes());
        let name: Vec<u8> = "home"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let mut munged = name.clone();
        ac2_core::pstring::munge(&mut munged);
        b.extend(4u16.to_le_bytes());
        b.extend(&munged);
        b.extend([0, 0]);
        b.extend(0x1234_0012u32.to_le_bytes());
        for v in [1.0f32, 2.0, 3.0, 1.0, 0.0, 0.0, 0.0] {
            b.extend(v.to_le_bytes());
        }
        let t = PlacesTable::unpack(&b).unwrap();
        assert_eq!((t.did, t.bucket_index, t.places.len()), (0x0E00_0004, 2, 1));
        let upper: Vec<u16> = "HOME".encode_utf16().collect();
        assert_eq!(t.place(&upper).map(|p| p.cell), Some(0x1234_0012));
    }
}
