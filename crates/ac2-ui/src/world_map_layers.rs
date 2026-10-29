//! What the world map shows over its terrain, besides the player and the server's markers:
//!
//! - The game's own map notes (`CDetailMapDesc` rows' `CMapNoteDesc` records, which
//!   `gmNavigationUI::AddMapNotes` places): towns, outposts, gateways, ringways, dungeons, vaults
//!   and points of interest, with their names from each record's string table and their icons.
//!   These are the default layers; the denser kinds appear as the map zooms in.
//! - The AC2Hero guide's places (`ac2_guide::atlas`), off by default, with the DAT's coloured map
//!   markers (`marker_yellow` and the others).
//! - Zone levels, off by default: the levels of the AC2Hero quests that start in each square of
//!   16 x 16 landblocks (the game data has no zone levels; the server decides encounters).

use ac2_core::DataId;
use ac2_db::ResourceStore;

/// The kind of a map note, from its icon (the icons the Legions notes use).
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

    /// Pixels per land cell from which the kind is shown (the map fills in as it zooms).
    pub fn shown_from(self) -> f32 {
        match self {
            NoteKind::Town => 0.0,
            NoteKind::Outpost | NoteKind::Gateway | NoteKind::Dungeon | NoteKind::Vault => 0.6,
            NoteKind::Ringway | NoteKind::Interest | NoteKind::Other => 1.2,
        }
    }

    /// Pixels per land cell from which the kind's names are written.
    pub fn named_from(self) -> f32 {
        match self {
            NoteKind::Town => 0.25,
            NoteKind::Outpost | NoteKind::Dungeon | NoteKind::Vault | NoteKind::Gateway => 2.0,
            _ => 4.0,
        }
    }
}

/// One map note: its land cell, name, kind and icon.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldNote {
    pub land: (i32, i32),
    pub name: String,
    pub kind: NoteKind,
    pub icon: u32,
}

/// Every note of the region's detail map, each once. Reads the DAT: run it on a worker.
pub fn load_notes(store: &ResourceStore) -> Vec<WorldNote> {
    let mut notes = Vec::new();
    let Some(map) = ac2_world::ac2_region_detail_map_table(store)
        .ok()
        .flatten()
        .and_then(|table| ac2_world::ac2_detail_map(store, table).ok().flatten())
    else {
        return notes;
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
            // The names end in an empty "()" (a variable slot left empty).
            let name = name.trim().trim_end_matches("()").trim().to_string();
            notes.push(WorldNote {
                land: (row.x, row.y),
                name,
                kind: NoteKind::from_icon(row.icon_did),
                icon: row.icon_did,
            });
        }
    }
    notes
}

/// The AC2Hero guide's layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuideLayer {
    QuestStarts,
    Npcs,
    QuestSteps,
    GuidePoints,
}

impl GuideLayer {
    pub const ALL: [GuideLayer; 4] = [
        GuideLayer::QuestStarts,
        GuideLayer::Npcs,
        GuideLayer::QuestSteps,
        GuideLayer::GuidePoints,
    ];

    pub fn label(self) -> &'static str {
        match self {
            GuideLayer::QuestStarts => "Quest starts",
            GuideLayer::Npcs => "NPCs",
            GuideLayer::QuestSteps => "Quest steps",
            GuideLayer::GuidePoints => "Guide points",
        }
    }

    /// The DAT's map marker the layer draws (`marker_yellow.tga` and the others, 9 x 9).
    pub fn icon(self) -> u32 {
        match self {
            GuideLayer::QuestStarts => 0x4100_924A,
            GuideLayer::Npcs => 0x4100_9249,
            GuideLayer::QuestSteps => 0x4100_924B,
            GuideLayer::GuidePoints => 0x4100_92C6,
        }
    }

    fn of(kind: ac2_guide::atlas::MarkerKind) -> Self {
        use ac2_guide::atlas::MarkerKind;
        match kind {
            MarkerKind::QuestStart => GuideLayer::QuestStarts,
            MarkerKind::Npc => GuideLayer::Npcs,
            MarkerKind::QuestStep => GuideLayer::QuestSteps,
            MarkerKind::GuidePoint => GuideLayer::GuidePoints,
        }
    }
}

/// One guide place on the map, in land cells (continuous), with its tooltip.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideMarker {
    pub land: (f64, f64),
    pub layer: GuideLayer,
    pub tooltip: String,
}

/// A zone square's quest levels: its square (16 x 16 landblocks, by index) and the lowest,
/// typical and highest level of the quests starting there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zone {
    pub square: (i32, i32),
    pub low: u32,
    pub median: u32,
    pub high: u32,
}

/// Land cells per zone square side.
pub const ZONE_CELLS: i32 = 128;

/// Land cells from a map coordinate (percent of the half map from the middle).
fn land_of(at: ac2_guide::MapCoord) -> (f64, f64) {
    let cells = |v: f32| (f64::from(v) + 100.0) * 10.2;
    (cells(at.east), cells(at.north))
}

/// The guide's places and zones from the baked guide (decoded here; dropped with the result's
/// owner).
pub fn guide_layers() -> (Vec<GuideMarker>, Vec<Zone>) {
    let guide = ac2_guide::baked();
    let markers: Vec<GuideMarker> = ac2_guide::atlas::markers(&guide)
        .into_iter()
        .map(|m| {
            let level = m.level.map_or(String::new(), |l| format!(" (level {l})"));
            let context = if m.context.is_empty() {
                String::new()
            } else {
                format!("\n{}", m.context)
            };
            GuideMarker {
                land: land_of(m.at),
                layer: GuideLayer::of(m.kind),
                tooltip: format!("{}{level}{context}", m.title),
            }
        })
        .collect();
    let mut squares: std::collections::HashMap<(i32, i32), Vec<u32>> = Default::default();
    for m in ac2_guide::atlas::markers(&guide) {
        if m.kind != ac2_guide::atlas::MarkerKind::QuestStart {
            continue;
        }
        let Some(level) = m.level else {
            continue;
        };
        let (x, y) = land_of(m.at);
        let square = ((x as i32) / ZONE_CELLS, (y as i32) / ZONE_CELLS);
        squares.entry(square).or_default().push(level);
    }
    let zones = squares
        .into_iter()
        .map(|(square, mut levels)| {
            levels.sort_unstable();
            Zone {
                square,
                low: levels[0],
                median: levels[levels.len() / 2],
                high: levels[levels.len() - 1],
            }
        })
        .collect();
    (markers, zones)
}

/// A zone's tint by its typical level: green, yellow, orange, red as the levels rise.
pub fn zone_tint(median: u32) -> [u8; 3] {
    match median {
        0..=19 => [80, 200, 90],
        20..=39 => [225, 210, 70],
        40..=59 => [235, 140, 50],
        _ => [220, 60, 50],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_kinds_fill_in_as_the_map_zooms() {
        assert!(NoteKind::Town.shown_from() < NoteKind::Outpost.shown_from());
        assert!(NoteKind::Outpost.shown_from() < NoteKind::Interest.shown_from());
        assert_eq!(NoteKind::from_icon(0x4100_512C), NoteKind::Town);
        assert_eq!(NoteKind::from_icon(0x4100_0000), NoteKind::Other);
        assert_eq!(zone_tint(25), [225, 210, 70]);
    }

    #[test]
    fn map_coordinates_land_on_cells() {
        let (x, y) = land_of(ac2_guide::MapCoord {
            north: 0.0,
            east: 0.0,
        });
        assert!((x - 1020.0).abs() < 1e-9 && (y - 1020.0).abs() < 1e-9);
    }
}
