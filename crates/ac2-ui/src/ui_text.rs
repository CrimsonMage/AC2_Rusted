//! UI text and the UI's data from the client's string tables: a UI string as the renderer's
//! `RenderInfo` and its text (`UiStrings`), a wire `StringInfo` rendered, the shortcut bar's wire
//! forms, the game clock's and the map coordinates' strings. The client preloads the tables
//! (`UiStringTables`) on a worker, so the frame renders from them.

use std::collections::HashMap;

use ac2_db::{PreloadedStringTables, RenderInfo, RenderVar};
use ac2_net::protocol::{ExaminationProfile, ExaminationValue, ShortcutInfo, StringInfo};

use crate::chat_text::render_info as wire_render_info;
use crate::{
    tokenize, ExaminationItem, Shortcut, SkillVar, UiString, UiStrings, SHORTCUT_ALIAS,
    SHORTCUT_ITEM, SHORTCUT_RECIPE,
};

/// A wire `ShortcutInfo` as the bar's plain data.
pub fn shortcut_from_info(info: &ShortcutInfo) -> Shortcut {
    match info {
        ShortcutInfo::Enum { kind, value } => Shortcut {
            kind: *kind,
            skill: *value,
            ..Shortcut::default()
        },
        ShortcutInfo::Instance(item) => Shortcut {
            kind: SHORTCUT_ITEM,
            item: *item,
            ..Shortcut::default()
        },
        ShortcutInfo::Text(text) => Shortcut {
            kind: SHORTCUT_ALIAS,
            text: text.clone(),
            ..Shortcut::default()
        },
        ShortcutInfo::DataId(did) => Shortcut {
            kind: SHORTCUT_RECIPE,
            data_id: *did,
            ..Shortcut::default()
        },
        ShortcutInfo::Empty(kind) => Shortcut {
            kind: *kind,
            ..Shortcut::default()
        },
    }
}

/// The bar's shortcut as a wire `ShortcutInfo` (`ShortcutInfo::StreamPack`'s cases).
pub fn shortcut_to_info(shortcut: &Shortcut) -> ShortcutInfo {
    match shortcut.kind {
        0x4000_0001 | 0x4000_0004 => ShortcutInfo::Enum {
            kind: shortcut.kind,
            value: shortcut.skill,
        },
        SHORTCUT_ITEM => ShortcutInfo::Instance(shortcut.item),
        SHORTCUT_ALIAS => ShortcutInfo::Text(shortcut.text.clone()),
        SHORTCUT_RECIPE => ShortcutInfo::DataId(shortcut.data_id),
        kind => ShortcutInfo::Empty(kind),
    }
}

/// The table enums the UI frameworks render from: the UI's, the chat's, race and sex names
/// (`AllegianceDataAdaptor::GetSpeciesGenderString`), the MotD's, and the map coordinates'
/// (`Position::ToRealMapCoords_Info`).
const UI_TABLE_ENUMS: [u32; 6] = [
    0x4100_0018,
    0x4100_0004,
    0x4100_000B,
    0x4100_0001,
    MAP_COORDS_TABLE,
    GAME_TIME_TABLE,
];

/// `GameTime::GetDateTimeString`'s table enum (`0x40000002`): `ID_DateString` and `ID_TimeString`.
pub const GAME_TIME_TABLE: u32 = 0x4000_0002;

/// `Position::ToRealMapCoords_Info`'s table enum.
pub const MAP_COORDS_TABLE: u32 = 0x4000_0007;

/// The string tables the UI renders from, all loaded on a worker: every table, and the DIDs of
/// the tables the UI names by enum (`StringInfo::SetTableEnum`, group `0x40000004`).
pub struct UiStringTables {
    pub tables: PreloadedStringTables,
    /// Enum `0x41000018`.
    pub ui_table: u32,
    pub enums: HashMap<u32, u32>,
}

impl UiStringTables {
    /// On a worker: the enums resolved through the DAT's enum mapper.
    pub fn new(store: &ac2_db::ResourceStore, tables: PreloadedStringTables) -> Self {
        let enums: HashMap<u32, u32> = UI_TABLE_ENUMS
            .iter()
            .filter_map(|&e| {
                let did = ac2_assets::ac2_enum_did(store, 0x4000_0004, e)
                    .ok()
                    .flatten()?;
                Some((e, did.raw()))
            })
            .collect();
        Self {
            tables,
            ui_table: enums.get(&0x4100_0018).copied().unwrap_or(0),
            enums,
        }
    }
}

/// `GameTime::GetDateTimeString`: the date (`ID_DateString` with `YEAR`, `YEARSPEC`, `DAY` and
/// `SEASON`) and the time (`ID_TimeString` with `TIME`), as strings of the game time's table.
pub fn game_date_time(date_time: ac2_world::GameDateTime) -> (UiString, UiString) {
    let name = |id: u32| SkillVar::Info(Box::new(UiString::new(id, GAME_TIME_TABLE, Vec::new())));
    let date = UiString::new(
        tokenize("ID_DateString"),
        GAME_TIME_TABLE,
        vec![
            (tokenize("YEAR"), SkillVar::Int(i64::from(date_time.year))),
            (tokenize("YEARSPEC"), name(date_time.year_spec)),
            (tokenize("DAY"), SkillVar::Int(i64::from(date_time.day))),
            (tokenize("SEASON"), name(date_time.season)),
        ],
    );
    let time = UiString::new(
        tokenize("ID_TimeString"),
        GAME_TIME_TABLE,
        vec![(tokenize("TIME"), name(date_time.time))],
    );
    (date, time)
}

/// `Position::ToRealMapCoords_Info` of a cell: "RealMapCoords" with NUM1 / DIR1 (north or
/// south) and NUM2 / DIR2 (east or west), one decimal; "Inside" off the land.
pub fn map_coords_string(cell: u32) -> UiString {
    let Some((x, y)) = ac2_physics::landdefs::gid_to_game_map_coords(cell) else {
        return UiString::new(tokenize("Inside"), MAP_COORDS_TABLE, Vec::new());
    };
    let number = |v: f32| SkillVar::Float {
        value: f64::from(v.abs()),
        precision: 1,
    };
    let word = |s: &str| {
        SkillVar::Info(Box::new(UiString::new(
            tokenize(s),
            MAP_COORDS_TABLE,
            Vec::new(),
        )))
    };
    UiString::new(
        tokenize("RealMapCoords"),
        MAP_COORDS_TABLE,
        vec![
            (tokenize("NUM1"), number(y)),
            (
                tokenize("DIR1"),
                word(if 0.0 <= y { "North" } else { "South" }),
            ),
            (tokenize("NUM2"), number(x)),
            (
                tokenize("DIR2"),
                word(if 0.0 <= x { "East" } else { "West" }),
            ),
        ],
    )
}

/// The rendering view over the preloaded tables: the UI string table, the enum map and the
/// tables themselves.
#[derive(Clone, Copy)]
pub struct UiText<'a> {
    pub strings: &'a PreloadedStringTables,
    /// The UI string table's DID (enum `0x41000018`).
    pub ui_table: u32,
    /// The DIDs of the tables the UI names by enum.
    pub enums: &'a HashMap<u32, u32>,
}

impl<'a> UiText<'a> {
    pub fn new(
        strings: &'a PreloadedStringTables,
        ui_table: u32,
        enums: &'a HashMap<u32, u32>,
    ) -> Self {
        Self {
            strings,
            ui_table,
            enums,
        }
    }

    /// A wire `StringInfo` rendered from the tables.
    pub fn render_info(&self, info: &StringInfo) -> String {
        self.text(info)
    }

    /// A wire `StringInfo` rendered from the tables (the chat renderer's `RenderInfo`).
    pub fn text(&self, info: &StringInfo) -> String {
        let info = wire_render_info(info);
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }

    /// A UI string as the renderer's `RenderInfo` (its table enum resolved to the table).
    fn ui_info(&self, string: &UiString) -> RenderInfo {
        let var = |var: &SkillVar| match var {
            SkillVar::UInt(v) => RenderVar::ULInt(*v),
            SkillVar::Int(v) => RenderVar::LInt(*v),
            SkillVar::Text(t) => RenderVar::String(t.encode_utf16().collect()),
            SkillVar::Float { value, precision } => RenderVar::Float {
                value: *value,
                precision: *precision as _,
            },
            SkillVar::Info(inner) => RenderVar::Info(Box::new(self.ui_info(inner))),
            SkillVar::Wire(info) => RenderVar::Info(Box::new(wire_render_info(info))),
        };
        RenderInfo {
            string_id: string.string_id,
            table_id: self.enums.get(&string.table_enum).copied().unwrap_or(0),
            literal: string.literal.as_ref().map(|t| t.encode_utf16().collect()),
            variables: string.vars.iter().map(|(id, v)| (*id, var(v))).collect(),
        }
    }

    /// `gmNavigationUI::AddMarker`'s tooltip: `ID_Navigation_Marker` with the marker's text
    /// (`message`) and the map coordinates of its cell (`position`).
    pub fn marker_tooltip(&self, message: &StringInfo, cell: u32) -> String {
        let position = self.render_string(&map_coords_string(cell));
        let string = UiString::new(
            tokenize("ID_Navigation_Marker"),
            self.ui_table,
            vec![
                (tokenize("message"), SkillVar::Text(self.text(message))),
                (tokenize("position"), SkillVar::Text(position)),
            ],
        );
        self.render_string(&string)
    }

    /// A string of the UI table with one `value` variable.
    pub fn ui_string(&self, name: &str, value: Option<RenderVar>) -> String {
        let info = RenderInfo {
            string_id: tokenize(name),
            table_id: self.ui_table,
            literal: None,
            variables: value
                .map(|v| vec![(tokenize("value"), v)])
                .unwrap_or_default(),
        };
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }

    /// `ID_Inventory_TT_Durability` with `current` and `max`, as the text shown.
    pub fn durability_tooltip(&self, current: i32, max: i32) -> Option<String> {
        let info = RenderInfo {
            string_id: tokenize("ID_Inventory_TT_Durability"),
            table_id: self.ui_table,
            literal: None,
            variables: vec![
                (tokenize("current"), RenderVar::Int(current)),
                (tokenize("max"), RenderVar::Int(max)),
            ],
        };
        Some(String::from_utf16_lossy(&ac2_db::get_string(
            self.strings,
            &info,
            false,
        )))
    }

    /// `gmExaminationUI::MakeString` and `ProcessNode`'s cases: a wire profile as the panel's
    /// items. Effect nodes are expanded before this (`ac2_ui::splice_effects`); one left is
    /// skipped.
    pub fn examination_items(&self, profile: &ExaminationProfile) -> Vec<ExaminationItem> {
        use ExaminationItem as I;
        use ExaminationValue as V;
        let text = |appearance: u32, text: String| I::Text { appearance, text };
        profile
            .nodes
            .iter()
            .filter_map(|node| {
                let a = node.appearance;
                Some(match &node.value {
                    V::Int(v) => text(
                        a,
                        self.ui_string("ID_Examination_Number", Some(RenderVar::Int(*v))),
                    ),
                    V::Float(v) => text(
                        a,
                        self.ui_string(
                            "ID_Examination_Number",
                            Some(RenderVar::Float {
                                value: f64::from(*v),
                                precision: 2,
                            }),
                        ),
                    ),
                    V::Bool(b) => text(
                        a,
                        self.ui_string(
                            if *b != 0 {
                                "ID_Examination_True"
                            } else {
                                "ID_Examination_False"
                            },
                            None,
                        ),
                    ),
                    V::Text(info) => text(a, self.text(info)),
                    V::LongInt(v) => text(
                        a,
                        self.ui_string("ID_Examination_Number", Some(RenderVar::LInt(*v))),
                    ),
                    V::Image(file) => I::Image(*file),
                    V::Icon(layers) => I::Icon(layers.iter().map(|l| l.image).collect()),
                    V::Break => I::Break,
                    V::Tabs(n) => I::Tabs(*n),
                    V::Countdown(seconds) => I::Countdown {
                        appearance: a,
                        seconds: *seconds,
                    },
                    V::Effect(_) | V::Other(_) => return None,
                })
            })
            .collect()
    }
}

impl UiStrings for UiText<'_> {
    fn render_string(&self, string: &UiString) -> String {
        let info = self.ui_info(string);
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }
}
