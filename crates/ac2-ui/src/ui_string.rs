//! A string for the client to render (`StringInfo` as the UI frameworks build it): a string table
//! entry by table enum (`SetStringIDandTableEnum`) with variables, which may themselves be strings
//! (`AddVariable_StringInfo`), the UI's own or the server's.

use crate::skill_tree::SkillVar;

/// The UI's string table (`0x41000018`).
pub const UI_TABLE: u32 = 0x4100_0018;

/// One `StringInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct UiString {
    pub string_id: u32,
    /// The table's enum (`StringInfo::SetTableEnum`).
    pub table_enum: u32,
    /// A literal instead of the entry (`SetLiteralValue`).
    pub literal: Option<String>,
    pub vars: Vec<(u32, SkillVar)>,
}

impl UiString {
    /// An entry of `table_enum` with these variables.
    pub fn new(string_id: u32, table_enum: u32, vars: Vec<(u32, SkillVar)>) -> Self {
        Self {
            string_id,
            table_enum,
            literal: None,
            vars,
        }
    }

    /// An entry of the UI table.
    pub fn ui(string_id: u32, vars: Vec<(u32, SkillVar)>) -> Self {
        Self::new(string_id, UI_TABLE, vars)
    }
}

/// Renders the UI's strings.
pub trait UiStrings {
    fn render_string(&self, string: &UiString) -> String;
}
