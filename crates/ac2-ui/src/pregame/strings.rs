//! The pregame screens' text: strings of the UI string table (enum `0x41000018`), rendered with
//! their variables the way `StringInfo::SetStringIDandTableEnum` + `GetString` do. The tables
//! are read on a worker ([`UiStrings::load`]) so the frame thread only renders.

use ac2_db::{PreloadedStringTables, RenderInfo, RenderVar, ResourceResult, ResourceStore};

use crate::chat::STRING_TABLE_GROUP;
use crate::token::tokenize;

/// `StringInfo::SetTableEnum(0x41000018)`: the UI strings.
pub const UI_STRING_TABLE: u32 = 0x4100_0018;
/// `StringState`'s DID (enum 0x40000001 of group 0x40000004).
const STRING_STATE: u32 = 0x5800_0000;

/// The UI string table, preloaded.
#[derive(Debug, Clone, Default)]
pub struct UiStrings {
    tables: PreloadedStringTables,
    /// The table's DID (`FileArray::GetDIDFromEnum(0x41000018, STRING_TABLE_GROUP)`).
    table: u32,
}

impl UiStrings {
    pub fn new(tables: PreloadedStringTables, table: u32) -> Self {
        Self { tables, table }
    }

    /// Reads the UI string table, the private tables it names and the string state. Reads
    /// files: call it on a worker.
    pub fn load(store: &ResourceStore) -> ResourceResult<Self> {
        Self::load_with(store, &[])
    }

    /// [`UiStrings::load`] with `extra` string tables preloaded as well (the action names of the
    /// controls screen live in the action map's table, [`UiStrings::render_in`]).
    pub fn load_with(store: &ResourceStore, extra: &[u32]) -> ResourceResult<Self> {
        let Some(table) = ac2_assets::ac2_enum_did(store, STRING_TABLE_GROUP, UI_STRING_TABLE)?
        else {
            return Ok(Self::default());
        };
        let table = table.raw();
        let mut dids = vec![table];
        dids.extend_from_slice(extra);
        Ok(Self::new(
            PreloadedStringTables::load(store, &dids, STRING_STATE),
            table,
        ))
    }

    /// `StringInfo::GetString` of string `string_id` in table `table` (no variables).
    pub fn render_in(&self, table: u32, string_id: u32) -> String {
        self.render_info(&RenderInfo {
            string_id,
            table_id: table,
            literal: None,
            variables: Vec::new(),
        })
    }

    /// `StringInfo::GetString` of string `string_id` with `variables`.
    pub fn render(&self, string_id: u32, variables: Vec<(u32, RenderVar)>) -> String {
        let info = RenderInfo {
            string_id,
            table_id: self.table,
            literal: None,
            variables,
        };
        String::from_utf16_lossy(&ac2_db::get_string(&self.tables, &info, false))
    }

    /// `StringInfo::GetString` of a whole string reference (any table the preloaded set holds).
    pub fn render_info(&self, info: &RenderInfo) -> String {
        String::from_utf16_lossy(&ac2_db::get_string(&self.tables, info, false))
    }

    /// [`UiStrings::render`] of the string named `name` (`Tokenize(name)`).
    pub fn get(&self, name: &str) -> String {
        self.render(tokenize(name), Vec::new())
    }
}

/// `StringInfo::AddVariable_String(tokenize(name), text)`.
pub(crate) fn text_var(name: &str, text: &str) -> (u32, RenderVar) {
    (
        tokenize(name),
        RenderVar::String(text.encode_utf16().collect()),
    )
}

/// `StringInfo::AddVariable_Int(tokenize(name), value)`.
pub(crate) fn int_var(name: &str, value: i32) -> (u32, RenderVar) {
    (tokenize(name), RenderVar::Int(value))
}
