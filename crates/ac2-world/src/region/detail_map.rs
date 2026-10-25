//! `CDetailMapDesc` (`0x5A`, `portal/client/src/cregion/cdetailmap/cdetailmapdesc.cpp.c`) and
//! `CMapNoteDesc` (`0x5E`, `cregion/cmapnote`): the detail map's rows, keyed by the value a block
//! map (`BlockMap`, `0x19`) holds over each land block, and the map notes a row lists.
//!
//! Layouts from the `StreamPack` functions: `CDetailMapDesc::StreamPack` (admin 0x51C4E0, Legions
//! 0x509C80) is the block map's DID then `PSmartArray<CDetailMapRow>` (count, rows);
//! `CDetailMapRow::StreamPack` (Legions 0x509A40) is `_value`, `_mapDID`, `_x0`, `_y0`, `_height`,
//! `_width`, then the `_mapNotes` `PSmartArray<unsigned long>`. `CMapNoteDesc` is the string
//! table's DID then `PSmartArray<CMapNoteRow>`; `CMapNoteRow::StreamPack` (admin 0x51BFB0) is
//! `_stringID`, `_iconDID`, `_x0`, `_y0`. Every row's block and land coordinates are unsigned on
//! the wire (the rows in memory are `unsigned long`); the map notes read them as signed land
//! cells, as `AddMapNotes` passes them to `ConvertLandscapeToPixel`.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId};

use super::ac2_region::{expect_did, expect_end, load, read_count};
use crate::ac2_water::Ac2BlockMap;

/// One `CDetailMapRow` (`0x2C` bytes in memory): the block map value it stands for, the image
/// shown for it, its block rectangle (`_x0`, `_y0`, `_width`, `_height`, in land blocks) and the
/// `CMapNoteDesc` DIDs it lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailMapRow {
    pub value: u32,
    pub map_did: u32,
    pub x0: u32,
    pub y0: u32,
    pub height: u32,
    pub width: u32,
    pub map_notes: Vec<u32>,
}

/// A `CDetailMapDesc` with its block map, as `CDetailMapDesc::InqRowFromXY` reads it.
#[derive(Debug, Clone)]
pub struct DetailMap {
    rows: Vec<DetailMapRow>,
    /// `_blockMapDID`'s `BlockMap`; `None` when the DAT has no such block map (retail's
    /// `DBObj::Get` fails and the lookup finds nothing).
    blocks: Option<Ac2BlockMap>,
    /// The `CMapNoteDesc` records the rows list, loaded with the map (`AddMapNotes` gets them
    /// with `DBObj::Get`).
    notes: Vec<(u32, MapNoteDesc)>,
}

impl DetailMap {
    /// `CDetailMapDesc::_rows` in stream order.
    pub fn rows(&self) -> &[DetailMapRow] {
        &self.rows
    }

    /// The `CMapNoteDesc` record `did` a row lists; `None` when it is not in the DAT.
    pub fn map_note(&self, did: u32) -> Option<&MapNoteDesc> {
        self.notes
            .iter()
            .find(|(note, _)| *note == did)
            .map(|(_, desc)| desc)
    }

    /// `CDetailMapDesc::InqRowFromValue`: the first row whose `_value` is `value`.
    pub fn row_from_value(&self, value: u32) -> Option<&DetailMapRow> {
        self.rows.iter().find(|row| row.value == value)
    }

    /// `CDetailMapDesc::InqRowFromXY`: the row of land block `(x, y)`, the block map's value
    /// there (`BlockMap::GetValue`, which answers `map[0]` outside the map) looked up as a row.
    pub fn row_from_block(&self, x: i32, y: i32) -> Option<&DetailMapRow> {
        let blocks = self.blocks.as_ref()?;
        self.row_from_value(u32::from(blocks.value(x as u32, y as u32)))
    }
}

/// One `CMapNoteRow`: the note's string (in the desc's string table), its icon and its land
/// cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapNoteRow {
    pub string_id: u32,
    pub icon_did: u32,
    pub x: i32,
    pub y: i32,
}

/// A `CMapNoteDesc`: its string table's DID and its notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapNoteDesc {
    pub string_table_did: u32,
    pub rows: Vec<MapNoteRow>,
}

/// Loads the `CDetailMapDesc` `did` (`DBObj::Get` of `0x5A`) with its block map, on the loader
/// worker. `None` when the record is not in the DAT.
pub fn ac2_detail_map(store: &ResourceStore, did: DataId) -> ResourceResult<Option<DetailMap>> {
    let Some(record) = load::<Ac2DetailMapRecord>(store, did)? else {
        return Ok(None);
    };
    let blocks = if record.block_map_did == 0 {
        None
    } else {
        load::<Ac2BlockMap>(store, DataId::new(record.block_map_did))?.map(|map| (*map).clone())
    };
    let mut notes: Vec<(u32, MapNoteDesc)> = Vec::new();
    for note in record.rows.iter().flat_map(|row| row.map_notes.iter()) {
        if notes.iter().all(|(loaded, _)| loaded != note) {
            if let Some(desc) = ac2_map_note_desc(store, DataId::new(*note))? {
                notes.push((*note, desc));
            }
        }
    }
    Ok(Some(DetailMap {
        rows: record.rows.clone(),
        blocks,
        notes,
    }))
}

/// Loads the `CMapNoteDesc` `did` (`0x5E`), `None` when the DAT has no such record.
pub fn ac2_map_note_desc(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<MapNoteDesc>> {
    Ok(load::<Ac2MapNoteRecord>(store, did)?.map(|record| record.desc.clone()))
}

/// `CDetailMapDesc::StreamPack`: the DID, the block map's DID, the row count and the rows.
#[derive(Debug)]
struct Ac2DetailMapRecord {
    block_map_did: u32,
    rows: Vec<DetailMapRow>,
}

impl ResourcePackedDecode for Ac2DetailMapRecord {
    const TYPE_NAME: &'static str = "AC2 DetailMapDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let block_map_did = cursor.read_u32()?;
        // A row is at least six words and an empty note list.
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 28)?;
        let rows = (0..count)
            .map(|_| {
                let value = cursor.read_u32()?;
                let map_did = cursor.read_u32()?;
                let x0 = cursor.read_u32()?;
                let y0 = cursor.read_u32()?;
                let height = cursor.read_u32()?;
                let width = cursor.read_u32()?;
                let notes = read_count(&mut cursor, qdid, Self::TYPE_NAME, 4)?;
                let map_notes = cursor.read_u32_items(notes)?;
                Ok(DetailMapRow {
                    value,
                    map_did,
                    x0,
                    y0,
                    height,
                    width,
                    map_notes,
                })
            })
            .collect::<ResourceResult<Vec<_>>>()?;
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            block_map_did,
            rows,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .rows
                .iter()
                .map(|row| std::mem::size_of::<DetailMapRow>() + 4 * row.map_notes.len())
                .sum::<usize>()
    }
}

/// `CMapNoteDesc::StreamPack`: the DID, the string table's DID, the row count and the rows.
#[derive(Debug)]
struct Ac2MapNoteRecord {
    desc: MapNoteDesc,
}

impl ResourcePackedDecode for Ac2MapNoteRecord {
    const TYPE_NAME: &'static str = "AC2 MapNoteDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        expect_did(qdid, Self::TYPE_NAME, cursor.read_u32()?)?;
        let string_table_did = cursor.read_u32()?;
        // A note row is four words.
        let count = read_count(&mut cursor, qdid, Self::TYPE_NAME, 16)?;
        let rows = (0..count)
            .map(|_| {
                Ok(MapNoteRow {
                    string_id: cursor.read_u32()?,
                    icon_did: cursor.read_u32()?,
                    x: cursor.read_u32()? as i32,
                    y: cursor.read_u32()? as i32,
                })
            })
            .collect::<ResourceResult<Vec<_>>>()?;
        expect_end(qdid, Self::TYPE_NAME, &cursor)?;
        Ok(Self {
            desc: MapNoteDesc {
                string_table_did,
                rows,
            },
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + 16 * self.desc.rows.len()
    }
}
