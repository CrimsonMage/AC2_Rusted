//! The UI frameworks' questions answered by the client: the skills panel's and the shortcut
//! bar's (`gmCWInterface`'s queries on the player's script, the progress the session mirrors)
//! and UI text from the string tables, which are all loaded once on a worker so the frame can
//! render them (retail renders UI text from cached tables).

use std::cell::RefCell;

use ac2_db::{PreloadedStringTables, RenderInfo, RenderVar};
use ac2_net::protocol::ShortcutInfo;
use ac2_ui::UiStrings as _;
use ac2_ui::{
    Shortcut, ShortcutOptions, ShortcutSource, SkillNode, SkillPanel, SkillSource, SkillStatus,
    SkillVar,
};

use crate::client::Client;
use crate::frame::ClientFrame;

/// A wire `ShortcutInfo` as the bar's plain data.
pub fn shortcut_from_info(info: &ShortcutInfo) -> Shortcut {
    match info {
        ShortcutInfo::Enum { kind, value } => Shortcut {
            kind: *kind,
            skill: *value,
            ..Shortcut::default()
        },
        ShortcutInfo::Instance(item) => Shortcut {
            kind: ac2_ui::SHORTCUT_ITEM,
            item: *item,
            ..Shortcut::default()
        },
        ShortcutInfo::Text(text) => Shortcut {
            kind: ac2_ui::SHORTCUT_ALIAS,
            text: text.clone(),
            ..Shortcut::default()
        },
        ShortcutInfo::DataId(did) => Shortcut {
            kind: ac2_ui::SHORTCUT_RECIPE,
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
        ac2_ui::SHORTCUT_ITEM => ShortcutInfo::Instance(shortcut.item),
        ac2_ui::SHORTCUT_ALIAS => ShortcutInfo::Text(shortcut.text.clone()),
        ac2_ui::SHORTCUT_RECIPE => ShortcutInfo::DataId(shortcut.data_id),
        kind => ShortcutInfo::Empty(kind),
    }
}

/// The string tables the UI renders from, all loaded on a worker: every table, and the DIDs of
/// the tables the UI names by enum (`StringInfo::SetTableEnum`, group `0x40000004`).
pub(crate) struct UiStringTables {
    pub tables: PreloadedStringTables,
    /// Enum `0x41000018`.
    pub ui_table: u32,
    pub enums: std::collections::HashMap<u32, u32>,
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
pub(crate) const GAME_TIME_TABLE: u32 = 0x4000_0002;

/// `GameTime::GetDateTimeString`: the date (`ID_DateString` with `YEAR`, `YEARSPEC`, `DAY` and
/// `SEASON`) and the time (`ID_TimeString` with `TIME`), as `StringInfo`s of the game time's table.
pub(crate) fn game_date_time(
    date_time: ac2_world::GameDateTime,
) -> (ac2_ui::UiString, ac2_ui::UiString) {
    use ac2_ui::{tokenize, SkillVar, UiString};
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

/// `Position::ToRealMapCoords_Info`'s table enum.
const MAP_COORDS_TABLE: u32 = 0x4000_0007;

/// `Position::ToRealMapCoords_Info` of a cell: "RealMapCoords" with NUM1 / DIR1 (north or
/// south) and NUM2 / DIR2 (east or west), one decimal; "Inside" off the land.
pub(crate) fn map_coords_string(cell: u32) -> ac2_ui::UiString {
    use ac2_ui::{tokenize, SkillVar, UiString};
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

impl UiStringTables {
    /// On a worker: the enums resolved through the DAT's enum mapper.
    pub(crate) fn new(store: &ac2_db::ResourceStore, tables: PreloadedStringTables) -> Self {
        let enums: std::collections::HashMap<u32, u32> = UI_TABLE_ENUMS
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

/// The player as the UI frameworks see it for one frame.
pub(crate) struct ClientPlayer<'a> {
    pub client: RefCell<&'a mut Client>,
    pub frame: &'a ClientFrame,
    pub strings: &'a PreloadedStringTables,
    /// The UI string table's DID (enum `0x41000018`).
    pub ui_table: u32,
    /// The DIDs of the tables the UI names by enum.
    pub enums: &'a std::collections::HashMap<u32, u32>,
}

impl ClientPlayer<'_> {
    /// A wire `StringInfo` rendered from the tables.
    pub(crate) fn render_info(&self, info: &ac2_net::protocol::StringInfo) -> String {
        self.text(info)
    }

    fn text(&self, info: &ac2_net::protocol::StringInfo) -> String {
        let info = crate::chat_text::render_info(info);
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }
}

impl SkillSource for ClientPlayer<'_> {
    fn panel(&self, panel: u32) -> Option<SkillPanel> {
        let data = self.client.borrow_mut().skill_panel(panel).ok()??;
        let nodes = data
            .nodes
            .iter()
            .map(|(_, node)| SkillNode {
                skill: node.skill_enum,
                kind: node.skill_type,
                name: self.text(&node.name),
                icon: node.icon,
                parents: node.parents.clone(),
                dependencies: node.dependencies.clone(),
                x: node.x,
                y: node.y,
                passive: node.is_passive != 0,
                untrainable: node.untrainable != 0,
            })
            .collect();
        Some(SkillPanel {
            name: self.text(&data.name),
            icon: data.icon,
            nodes,
        })
    }

    fn root_panel_from_race(&self, root: u32) -> Option<u32> {
        self.client
            .borrow_mut()
            .root_panel_from_race(root)
            .ok()
            .flatten()
    }

    fn status(&self, skill: u32) -> SkillStatus {
        let skills = &self.frame.progress.skills;
        let mut client = self.client.borrow_mut();
        // A query that fails reads 0, as the retail wrappers' return slots start.
        let level = client.skill_level(skill).ok();
        SkillStatus {
            trained: skills.is_trained(skill),
            untraining: skills.is_being_untrained(skill),
            hero: client.is_hero_skill(skill).unwrap_or(false),
            can_train: client.can_skill_be_trained(skill).unwrap_or(0),
            can_raise: client.can_skill_be_raised(skill).unwrap_or(0),
            can_untrain: client.can_skill_be_untrained(skill).unwrap_or(0),
            level: level.map_or(0, |l| l.raw),
            raw_level: level.map_or(0, |l| l.enchanted),
            cost_to_train: u64::from(client.skill_cost_to_train(skill).unwrap_or(0)),
            xp_to_level: client.skill_experience_to_level(skill).unwrap_or(0),
            skill_xp: skills.xp(skill),
        }
    }

    fn available(&self) -> (u64, u32, u32) {
        let progress = &self.frame.progress;
        (
            progress.experience.available_xp,
            progress.skills.credits,
            progress.skills.hero_credits,
        )
    }

    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        self.render_string(&ac2_ui::UiString::ui(string_id, vars.to_vec()))
    }
}

impl ClientPlayer<'_> {
    /// A UI string as the renderer's `RenderInfo` (its table enum resolved to the table).
    fn ui_info(&self, string: &ac2_ui::UiString) -> RenderInfo {
        let var = |var: &SkillVar| match var {
            SkillVar::UInt(v) => RenderVar::ULInt(*v),
            SkillVar::Int(v) => RenderVar::LInt(*v),
            SkillVar::Text(t) => RenderVar::String(t.encode_utf16().collect()),
            SkillVar::Float { value, precision } => RenderVar::Float {
                value: *value,
                precision: *precision as _,
            },
            SkillVar::Info(inner) => RenderVar::Info(Box::new(self.ui_info(inner))),
            SkillVar::Wire(info) => RenderVar::Info(Box::new(crate::chat_text::render_info(info))),
        };
        RenderInfo {
            string_id: string.string_id,
            table_id: self.enums.get(&string.table_enum).copied().unwrap_or(0),
            literal: string.literal.as_ref().map(|t| t.encode_utf16().collect()),
            variables: string.vars.iter().map(|(id, v)| (*id, var(v))).collect(),
        }
    }
}

impl ac2_ui::UiStrings for ClientPlayer<'_> {
    fn render_string(&self, string: &ac2_ui::UiString) -> String {
        let info = self.ui_info(string);
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }
}

impl ShortcutSource for ClientPlayer<'_> {
    fn shortcuts(&self) -> Vec<Shortcut> {
        self.client
            .borrow_mut()
            .shortcut_array()
            .map(|all| all.iter().map(shortcut_from_info).collect())
            .unwrap_or_default()
    }

    fn icon(&self, index: usize, _: &Shortcut) -> u32 {
        self.client
            .borrow_mut()
            .shortcut_info(index as u32)
            .map_or(0, |info| info.icon)
    }

    fn options(&self) -> ShortcutOptions {
        let settings = self.client.borrow_mut().shortcut_settings().ok();
        settings.map_or(
            ShortcutOptions {
                which_set: 1,
                rows: 1,
                ..ShortcutOptions::default()
            },
            |s| ShortcutOptions {
                which_set: s.which_set,
                show_numbers: s.show_numbers,
                insta_scroll: s.insta_scroll,
                rows: s.height as i32,
            },
        )
    }

    fn skill_trained(&self, skill: u32) -> bool {
        self.frame.progress.skills.is_trained(skill)
    }

    /// `gmCWInterface::InqSkillTimeLeft`: the skill's total and time left, while it is on reset.
    fn skill_reset(&self, skill: u32) -> Option<(f64, f64)> {
        let time = self.client.borrow_mut().skill_time_left(skill).ok()?;
        (time.ok && time.left > 0.0).then_some((f64::from(time.total), f64::from(time.left)))
    }

    /// `gmCWInterface::DoIHaveItem`: in a pack the player lists or equipped.
    fn has_item(&self, item: u64) -> bool {
        let inventory = &self.frame.inventory;
        inventory.pack.items.contains(&item)
            || inventory.pack.packs.contains(&item)
            || inventory
                .containers
                .iter()
                .any(|c| c.items.contains(&item) || c.packs.contains(&item))
            || inventory.equipped.iter().any(|(_, i)| *i == item)
    }
}

impl ClientPlayer<'_> {
    /// A string of the UI table with one `value` variable.
    fn ui_string(&self, name: &str, value: Option<RenderVar>) -> String {
        let info = RenderInfo {
            string_id: ac2_ui::tokenize(name),
            table_id: self.ui_table,
            literal: None,
            variables: value
                .map(|v| vec![(ac2_ui::tokenize("value"), v)])
                .unwrap_or_default(),
        };
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }

    /// `gmExaminationUI::MakeString` and `ProcessNode`'s cases: a wire profile as the panel's
    /// items. Effect nodes (`ProcessEffectIDNode`: `AddEffectDataToProfileByEID`) are not
    /// expanded.
    pub(crate) fn examination_items(
        &self,
        profile: &ac2_net::protocol::ExaminationProfile,
    ) -> Vec<ac2_ui::ExaminationItem> {
        use ac2_net::protocol::ExaminationValue as V;
        use ac2_ui::ExaminationItem as I;
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
                            if *b {
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
