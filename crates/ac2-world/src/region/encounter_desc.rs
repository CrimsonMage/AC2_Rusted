//! `CEncounterDesc` (the region's monster-spawn table): rows of encounter elements with their
//! terrain and water exceptions, in `StreamPack` order.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

/// `EncounterExcept`: a terrain or water index whose encounter id replaces the element's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncounterExcept {
    pub index: u32,
    pub encount_id: u32,
}

/// `EncounterElem`.
#[derive(Debug, Clone, PartialEq)]
pub struct EncounterElem {
    pub index: u32,
    pub encount_id: u32,
    pub adjust_chance: f32,
    pub day_based: bool,
    pub terrain_except: Vec<EncounterExcept>,
    pub water_except: Vec<EncounterExcept>,
}

impl EncounterElem {
    /// `EncounterElem::GetSubDataIDs`: its encounter id, then every terrain and water exception's.
    pub fn sub_data_ids(&self) -> Vec<DataId> {
        std::iter::once(self.encount_id)
            .chain(self.terrain_except.iter().map(|e| e.encount_id))
            .chain(self.water_except.iter().map(|e| e.encount_id))
            .map(DataId::new)
            .collect()
    }
}

/// `EncounterRow`: a named row of elements.
#[derive(Debug, Clone, PartialEq)]
pub struct EncounterRow {
    pub row_index: u32,
    pub name: Vec<u8>,
    pub encounters: Vec<EncounterElem>,
}

impl EncounterRow {
    /// `EncounterRow::GetSubDataIDs`.
    pub fn sub_data_ids(&self) -> Vec<DataId> {
        self.encounters
            .iter()
            .flat_map(EncounterElem::sub_data_ids)
            .collect()
    }
}

/// `CEncounterDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct EncounterDesc {
    pub version: u32,
    pub block_encounter_map: DataId,
    pub rows: Vec<EncounterRow>,
}

impl EncounterDesc {
    /// `CEncounterDesc::GetSubDataIDs`: the block encounter map, then each row's ids.
    pub fn sub_data_ids(&self) -> Vec<DataId> {
        std::iter::once(self.block_encounter_map)
            .chain(self.rows.iter().flat_map(EncounterRow::sub_data_ids))
            .collect()
    }
}

const MAX_ITEMS: usize = 65_536;

fn bad(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: "AC2 EncounterDesc",
        context,
    }
}

fn count(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
    min_item: usize,
) -> ResourceResult<usize> {
    let n = cursor.read_u32_count()?;
    if n > MAX_ITEMS || n * min_item > cursor.remaining() {
        return Err(bad(qdid, "item count exceeded the record"));
    }
    Ok(n)
}

fn excepts(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> ResourceResult<Vec<EncounterExcept>> {
    (0..count(cursor, qdid, 8)?)
        .map(|_| {
            Ok(EncounterExcept {
                index: cursor.read_u32()?,
                encount_id: cursor.read_u32()?,
            })
        })
        .collect()
}

impl ResourcePackedDecode for EncounterDesc {
    const TYPE_NAME: &'static str = "AC2 EncounterDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `CEncounterDesc::StreamPack`: version, DID, block encounter map, then the rows
    /// (`EncounterRow::StreamPack`: row index, name, elements; `EncounterElem::StreamPack`: index,
    /// encounter id, adjust chance, day-based flag, terrain and water exceptions).
    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        let version = cursor.read_u32()?;
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad(qdid, "embedded DID did not match DAT identity"));
        }
        let block_encounter_map = DataId::new(cursor.read_u32()?);
        let mut rows = Vec::new();
        for _ in 0..count(&mut cursor, qdid, 12)? {
            let row_index = cursor.read_u32()?;
            let short = usize::from(cursor.read_u16()?);
            let len = if short == 0xFFFF {
                cursor.read_u32_count()?
            } else {
                short
            };
            let mut name = cursor.read_bytes(len)?.to_vec();
            ac2_core::pstring::unmunge(&mut name);
            cursor.align_to(4)?;
            let mut encounters = Vec::new();
            for _ in 0..count(&mut cursor, qdid, 24)? {
                encounters.push(EncounterElem {
                    index: cursor.read_u32()?,
                    encount_id: cursor.read_u32()?,
                    adjust_chance: cursor.read_f32()?,
                    day_based: cursor.read_u32()? != 0,
                    terrain_except: excepts(&mut cursor, qdid)?,
                    water_except: excepts(&mut cursor, qdid)?,
                });
            }
            rows.push(EncounterRow {
                row_index,
                name,
                encounters,
            });
        }
        if cursor.remaining() != 0 {
            return Err(bad(qdid, "trailing bytes after EncounterDesc"));
        }
        Ok(Self {
            version,
            block_encounter_map,
            rows,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .rows
                .iter()
                .map(|row| row.encounters.len() * std::mem::size_of::<EncounterElem>() + 64)
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_data_ids_list_the_encounter_then_its_exceptions() {
        let elem = EncounterElem {
            index: 1,
            encount_id: 0x100,
            adjust_chance: 1.0,
            day_based: true,
            terrain_except: vec![EncounterExcept {
                index: 7,
                encount_id: 0x200,
            }],
            water_except: vec![EncounterExcept {
                index: 2,
                encount_id: 0x300,
            }],
        };
        let ids: Vec<u32> = elem.sub_data_ids().iter().map(|d| d.raw()).collect();
        assert_eq!(ids, [0x100, 0x200, 0x300]);
    }

    fn words(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn a_record_decodes_rows_elements_and_exceptions_in_stream_order() {
        let did = 0x1400_0000;
        let mut bytes = words(&[3, did, 0x1900_0001, 1, 7]);
        // The row name "ab": u16 length, munged characters, padded to 4 bytes.
        let mut name = *b"ab";
        ac2_core::pstring::munge(&mut name);
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&name);
        bytes.extend(words(&[
            1, // one element
            4,
            0x2000_0001,
            0.5_f32.to_bits(),
            1, // index, id, chance, day based
            1,
            9,
            0x2000_0002, // one terrain exception
            0,           // no water exceptions
        ]));
        let qdid = QualifiedDataId::new(DataId::new(did), 0);
        let desc = EncounterDesc::decode_packed(qdid, 0, &bytes).unwrap();
        assert_eq!(desc.version, 3);
        assert_eq!(desc.block_encounter_map, DataId::new(0x1900_0001));
        let row = &desc.rows[0];
        assert_eq!((row.row_index, row.name.as_slice()), (7, &b"ab"[..]));
        let elem = &row.encounters[0];
        assert_eq!(
            (elem.index, elem.encount_id, elem.adjust_chance),
            (4, 0x2000_0001, 0.5)
        );
        assert!(elem.day_based && elem.water_except.is_empty());
        assert_eq!(
            elem.terrain_except,
            [EncounterExcept {
                index: 9,
                encount_id: 0x2000_0002
            }]
        );
        let ids: Vec<u32> = desc.sub_data_ids().iter().map(|d| d.raw()).collect();
        assert_eq!(ids, [0x1900_0001, 0x2000_0001, 0x2000_0002]);
        // A record with bytes past the last row is refused.
        let mut long = bytes.clone();
        long.extend(words(&[0]));
        assert!(EncounterDesc::decode_packed(qdid, 0, &long).is_err());
    }
}
