//! The game's own map notes: the towns, outposts, gateways, ringways, vaults, dungeons and points
//! of interest the region's detail map lists (`CDetailMapDesc` rows' `CMapNoteDesc` records, which
//! `gmNavigationUI::AddMapNotes` places on the navigation map), with their names from each
//! record's string table and their 9 x 9 icons. These are the map's default layers; the denser
//! kinds appear as you zoom in.

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_guide::MapCoord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteKind {
    Town,
    Outpost,
    Gateway,
    Ringway,
    Dungeon,
    Vault,
    Interest,
    Other,
}

impl NoteKind {
    pub const ALL: [NoteKind; 8] = [
        NoteKind::Town,
        NoteKind::Outpost,
        NoteKind::Gateway,
        NoteKind::Ringway,
        NoteKind::Dungeon,
        NoteKind::Vault,
        NoteKind::Interest,
        NoteKind::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            NoteKind::Town => "Towns",
            NoteKind::Outpost => "Outposts",
            NoteKind::Gateway => "Gateways",
            NoteKind::Ringway => "Ringways",
            NoteKind::Dungeon => "Dungeons",
            NoteKind::Vault => "Vaults",
            NoteKind::Interest => "Points of interest",
            NoteKind::Other => "Other places",
        }
    }

    /// The kind an icon stands for (the icons the Legions notes use).
    fn from_icon(did: u32) -> Self {
        match did {
            0x4100_512C => NoteKind::Town,
            0x4100_512D => NoteKind::Outpost,
            0x4100_512A => NoteKind::Gateway,
            0x4100_512B => NoteKind::Ringway,
            0x4100_512F => NoteKind::Dungeon,
            0x4100_512E => NoteKind::Vault,
            0x4100_5129 => NoteKind::Interest,
            _ => NoteKind::Other,
        }
    }

    /// Screen pixels per land cell from which the kind is drawn, so the map fills in as it zooms.
    pub fn shown_from(self) -> f32 {
        match self {
            NoteKind::Town => 0.0,
            NoteKind::Outpost | NoteKind::Gateway | NoteKind::Dungeon | NoteKind::Vault => 0.9,
            NoteKind::Ringway | NoteKind::Interest | NoteKind::Other => 1.8,
        }
    }

    /// Screen pixels per land cell from which the kind's names are written beside it.
    pub fn named_from(self) -> f32 {
        match self {
            NoteKind::Town => 0.0,
            NoteKind::Outpost | NoteKind::Dungeon | NoteKind::Vault | NoteKind::Gateway => 2.0,
            _ => 3.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub at: MapCoord,
    pub name: String,
    pub kind: NoteKind,
    pub icon: u32,
}

/// Land cells per map unit.
const CELLS_PER_UNIT: f32 = 10.2;

/// Every map note of the region's detail map, each once, and their icons as RGBA (width,
/// height, pixels) by DID.
pub fn load(store: &ResourceStore) -> (Vec<Note>, HashMap<u32, (u32, u32, Vec<u8>)>) {
    let mut notes = Vec::new();
    let mut icons = HashMap::new();
    let Some(map) = ac2_world::ac2_region_detail_map_table(store)
        .ok()
        .flatten()
        .and_then(|table| ac2_world::ac2_detail_map(store, table).ok().flatten())
    else {
        return (notes, icons);
    };
    let mut seen = std::collections::HashSet::new();
    for did in map.rows().iter().flat_map(|r| &r.map_notes) {
        if !seen.insert(*did) {
            continue;
        }
        let Some(desc) = map.map_note(*did) else {
            continue;
        };
        let strings = ac2_db::load_string_table(store, DataId::new(desc.string_table_did))
            .ok()
            .flatten();
        for row in &desc.rows {
            let name = strings
                .as_ref()
                .and_then(|t| t.get(row.string_id))
                .map(|s| s.text())
                .unwrap_or_default();
            // The names end in an empty "()" (a variable slot the client fills with nothing).
            let name = name.trim().trim_end_matches("()").trim().to_string();
            notes.push(Note {
                at: MapCoord {
                    north: row.y as f32 / CELLS_PER_UNIT - 100.0,
                    east: row.x as f32 / CELLS_PER_UNIT - 100.0,
                },
                name,
                kind: NoteKind::from_icon(row.icon_did),
                icon: row.icon_did,
            });
            if let std::collections::hash_map::Entry::Vacant(slot) = icons.entry(row.icon_did) {
                if let Ok(Some(s)) =
                    ac2_assets::load_ac2_ui_surface(store, DataId::new(row.icon_did))
                {
                    slot.insert((s.width, s.height, s.rgba.to_vec()));
                }
            }
        }
    }
    (notes, icons)
}
