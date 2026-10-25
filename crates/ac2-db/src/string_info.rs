//! `StringInfo`'s text (`stringinfo.cpp`): `InqStringInternal`, `InqString`, `GetString`,
//! `GetStringWithMetaTags` and `GetLogString` over a plain [`RenderInfo`], with
//! `StringInfo::SetVarInTable` turning each variable into text and `StringTable::GetString`
//! (vtable +0x80) rendering the table's entry through the meta-language.
//!
//! ac2-db sits below the crates that carry `StringInfo` (ac2-net's wire type, ac2-script's
//! native), so they convert theirs into [`RenderInfo`]; the `SID_*` kinds map one to one onto
//! [`RenderVar`]. Tables come through [`StringTables`], so a frame-thread caller can hand in
//! tables it already holds and a worker can read them from a [`crate::ResourceStore`]
//! ([`crate::StoreStringTables`]).

use std::collections::HashMap;
use std::ops::Deref;

use crate::string_meta_render::render_string;
use crate::string_meta_text::{escape_string, strip_meta_letters, unescape_string, wide};
use crate::string_state::StringState;
use crate::string_table::{StringTable, STRING_TABLE_FIRST, STRING_TABLE_LAST};

/// Nested `StringInfo` variables deeper than this fail with `gsGenericError` (retail has no
/// limit; packed data cannot nest without bound).
const MAX_DEPTH: usize = 32;

/// A string table checked out of a cache (`DBObj::Get` ... `Release`).
pub type StringTableRef<'a> = Box<dyn Deref<Target = StringTable> + 'a>;
/// The `StringState` checked out of a cache.
pub type StringStateRef<'a> = Box<dyn Deref<Target = StringState> + 'a>;

/// Where rendering gets its DAT objects. A `None` table is retail's failed `DBObj::Get`
/// (`gsMissingTable` / `gsMissingPrivateTable`); a `None` state is `StringStateInfo` without
/// its `StringState` (numbers render empty, plurals fall back to `n == 1`, meta letters to
/// the built-in English ones).
pub trait StringTables {
    /// `DBObj::Get(QualifiedDataID(did, DB_TYPE_STRING_TABLE))`.
    fn table(&self, did: u32) -> Option<StringTableRef<'_>>;
    /// `StringStateInfo::m_instance->m_state`.
    fn string_state(&self) -> Option<StringStateRef<'_>>;
}

/// One variable's value, by `StringInfoData::type` (the `SID_*` classes).
#[derive(Debug, Clone, PartialEq)]
pub enum RenderVar {
    /// 2, `SID_Int`: grouped, the default base.
    Int(i32),
    /// 3, `SID_FormattedInt`.
    FormattedInt {
        value: i32,
        grouping: i32,
        base: u16,
    },
    /// 4, `SID_Float`: rounded to a `float` first, as `SetVarInTable` passes it.
    Float { value: f64, precision: u16 },
    /// 5, `SID_FormattedFloat`.
    FormattedFloat {
        value: f64,
        precision: u16,
        grouping: i32,
        base: u16,
    },
    /// 6, `SID_UInt`.
    UInt(u32),
    /// 7, `SID_FormattedUInt`.
    FormattedUInt {
        value: u32,
        grouping: i32,
        base: u16,
    },
    /// 9, `SID_StringInfo`: a nested string, rendered with its meta letters kept. A nested
    /// reference with no table takes the outer string's (`SetTableID`), which is how
    /// `AddVariable_StringID` variables find theirs.
    Info(Box<RenderInfo>),
    /// 10, `SID_LInt`.
    LInt(i64),
    /// 11, `SID_FormattedLInt`.
    FormattedLInt {
        value: i64,
        grouping: i32,
        base: u16,
    },
    /// 12, `SID_ULInt`.
    ULInt(u64),
    /// 13, `SID_FormattedULInt`.
    FormattedULInt {
        value: u64,
        grouping: i32,
        base: u16,
    },
    /// `StringInfo::AddVariable_String(text)`: raw text, which retail stores as a nested
    /// literal escaped by `EscapeString` (a type-9 variable on the wire). A nested literal
    /// that is already escaped goes in [`RenderVar::Info`] instead.
    String(Vec<u16>),
    /// Any other type: `SetVarInTable` sets nothing, so a string using it misses a variable.
    Skipped,
}

impl RenderVar {
    /// `StringInfo::AddVariable_StringID(var, id)` (`table_id` 0: the outer string's table)
    /// and `AddVariable_StringIDandTableEnum` (the enum's table DID).
    pub fn string_id(string_id: u32, table_id: u32) -> Self {
        Self::Info(Box::new(RenderInfo {
            string_id,
            table_id,
            ..RenderInfo::default()
        }))
    }
}

/// A `StringInfo`: a string-table reference with variables, or a literal.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderInfo {
    pub string_id: u32,
    pub table_id: u32,
    /// `m_LiteralValue` when `m_IsLiteral` is set (stored escaped, meta letters allowed).
    pub literal: Option<Vec<u16>>,
    /// `(variable id, value)`; a later variable with the same id replaces an earlier one, as
    /// `StringInfo::AddVar` does.
    pub variables: Vec<(u32, RenderVar)>,
}

/// `GetStringErrorCode` (`gsSuccess` is the `Ok` side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GetStringError {
    Uninitialized = 1,
    MissingString = 2,
    MissingTable = 3,
    MissingPrivateString = 4,
    MissingPrivateTable = 5,
    MissingVariable = 6,
    MetaLanguageError = 7,
    GenericError = 8,
}

/// A failed render: the code and the text retail leaves in the result (the
/// `<string table error; ...>` line, or a partial render).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetStringFailure {
    pub code: GetStringError,
    pub text: Vec<u16>,
}

/// `StringVarTable`: variable id to its text.
pub type StringVarTable = HashMap<u32, Vec<u16>>;

/// `MasterDBMap::DivineType(did) == DB_TYPE_STRING_TABLE` (`StringInfo::SetTableID`).
fn string_table_did(did: u32) -> u32 {
    if (STRING_TABLE_FIRST..=STRING_TABLE_LAST).contains(&did) {
        did
    } else {
        0
    }
}

impl StringTable {
    /// `StringTable::GetString(result, token)` (Legions vtable +0x5C): the entry's pieces
    /// joined as they are, no variables or meta-language, from its private table when it
    /// names one.
    pub fn plain_entry(
        &self,
        tables: &dyn StringTables,
        string_id: u32,
    ) -> Result<Vec<u16>, GetStringError> {
        let entry = self.get(string_id).ok_or(GetStringError::MissingString)?;
        if entry.table == 0 {
            return Ok(entry.pieces.iter().flatten().flatten().copied().collect());
        }
        let private = tables
            .table(entry.table)
            .ok_or(GetStringError::MissingPrivateTable)?;
        let entry = private
            .get(string_id)
            .ok_or(GetStringError::MissingPrivateString)?;
        Ok(entry.pieces.iter().flatten().flatten().copied().collect())
    }

    /// `StringTable::GetString(result, token, varTable, 1)` (vtable +0x80): the entry (from
    /// its private table when it names one), its variables' texts, rendered by
    /// `StringTableMetaLanguage::RenderString`. `out` keeps the caller's text on failure
    /// unless the render got far enough to replace it.
    pub fn render_entry(
        &self,
        tables: &dyn StringTables,
        string_id: u32,
        vars: &StringVarTable,
        out: &mut Vec<u16>,
    ) -> Result<(), GetStringError> {
        let entry = self.get(string_id).ok_or(GetStringError::MissingString)?;
        let private;
        let entry = if entry.table != 0 {
            private = tables
                .table(entry.table)
                .ok_or(GetStringError::MissingPrivateTable)?;
            private
                .get(string_id)
                .ok_or(GetStringError::MissingPrivateString)?
        } else {
            entry
        };
        let mut values = Vec::with_capacity(entry.variables.len());
        let mut missing = false;
        for id in &entry.variables {
            match vars.get(id) {
                Some(value) => values.push(value.clone()),
                None => {
                    missing = true;
                    values.push(Vec::new());
                }
            }
        }
        if missing {
            return Err(GetStringError::MissingVariable);
        }
        let state = tables.string_state();
        render_string(
            &entry.pieces,
            &values,
            &entry.variable_names,
            state.as_deref().map(|s| &**s),
            out,
        )
        .map(|_| ())
        .map_err(|err| {
            log::debug!(
                "StringTableMetaLanguage syntax error: {} (token 0x{string_id:08X}, pos {})",
                err.message,
                err.pos
            );
            GetStringError::MetaLanguageError
        })
    }
}

/// The text `InqStringInternal` writes before rendering.
fn error_text(string_id: u32, table_id: u32) -> Vec<u16> {
    wide(&format!(
        "<string table error; token: 0x{string_id:08X} tableDID: 0x{table_id:08X}>"
    ))
}

/// `StringInfo::SetVarInTable` for every variable: numbers through the table's
/// `*ToString` (empty when that fails), nested strings through `InqStringInternal` with meta
/// letters kept (their error text when that fails). `table_id` is the outer string's table,
/// which a nested reference without one takes.
fn var_table(
    tables: &dyn StringTables,
    variables: &[(u32, RenderVar)],
    table_id: u32,
    table: &StringTable,
    depth: usize,
) -> StringVarTable {
    let state = tables.string_state();
    let state = state.as_deref().map(|s| &**s);
    let mut vars = StringVarTable::new();
    for (id, var) in variables {
        let text = match var {
            RenderVar::Int(v) => table.int_to_string(state, *v, 1, -1),
            RenderVar::FormattedInt {
                value,
                grouping,
                base,
            } => table.int_to_string(state, *value, *grouping, *base as i16),
            RenderVar::Float { value, precision } => {
                table.float_to_string(state, f64::from(*value as f32), *precision, 1, -1)
            }
            RenderVar::FormattedFloat {
                value,
                precision,
                grouping,
                base,
            } => table.float_to_string(
                state,
                f64::from(*value as f32),
                *precision,
                *grouping,
                *base as i16,
            ),
            RenderVar::UInt(v) => table.uint_to_string(state, *v, 1, -1),
            RenderVar::FormattedUInt {
                value,
                grouping,
                base,
            } => table.uint_to_string(state, *value, *grouping, *base as i16),
            RenderVar::LInt(v) => table.lint_to_string(state, *v, 1, -1),
            RenderVar::FormattedLInt {
                value,
                grouping,
                base,
            } => table.lint_to_string(state, *value, *grouping, *base as i16),
            RenderVar::ULInt(v) => table.ulint_to_string(state, *v, 1, -1),
            RenderVar::FormattedULInt {
                value,
                grouping,
                base,
            } => table.ulint_to_string(state, *value, *grouping, *base as i16),
            RenderVar::Info(nested) => {
                let inherit =
                    nested.literal.is_none() && nested.string_id != 0 && nested.table_id == 0;
                let nested_table = if inherit {
                    string_table_did(table_id)
                } else {
                    nested.table_id
                };
                let text = match inq_internal(tables, nested, nested_table, true, depth + 1) {
                    Ok(text) => text,
                    Err(failure) => failure.text,
                };
                Some(text)
            }
            RenderVar::String(text) => Some(escape_string(text)),
            RenderVar::Skipped => continue,
        };
        vars.insert(*id, text.unwrap_or_default());
    }
    vars
}

fn inq_internal(
    tables: &dyn StringTables,
    info: &RenderInfo,
    table_id: u32,
    retain_metas: bool,
    depth: usize,
) -> Result<Vec<u16>, GetStringFailure> {
    let strip = |text: Vec<u16>| {
        if retain_metas {
            text
        } else {
            strip_meta_letters(&text)
        }
    };
    if let Some(literal) = &info.literal {
        return Ok(strip(literal.clone()));
    }
    let mut out = error_text(info.string_id, table_id);
    let fail = |code, text| Err(GetStringFailure { code, text });
    if info.string_id == 0 || table_id == 0 {
        return fail(GetStringError::Uninitialized, out);
    }
    if depth > MAX_DEPTH {
        return fail(GetStringError::GenericError, out);
    }
    let Some(table) = tables.table(table_id) else {
        return fail(GetStringError::MissingTable, out);
    };
    let vars = var_table(tables, &info.variables, table_id, &table, depth);
    let result = table.render_entry(tables, info.string_id, &vars, &mut out);
    let out = strip(out);
    match result {
        Ok(()) => Ok(out),
        Err(code) => fail(code, out),
    }
}

/// `StringInfo::InqStringInternal(result, bRetainMetas)`: a literal as it is, or the table's
/// entry rendered with the variables; meta letters stripped unless retained. Not unescaped.
pub fn inq_string_internal(
    tables: &dyn StringTables,
    info: &RenderInfo,
    retain_metas: bool,
) -> Result<Vec<u16>, GetStringFailure> {
    inq_internal(tables, info, info.table_id, retain_metas, 0)
}

/// `StringInfo::InqString(result, bRetainMetas)`: [`inq_string_internal`], unescaped on
/// success.
pub fn inq_string(
    tables: &dyn StringTables,
    info: &RenderInfo,
    retain_metas: bool,
) -> Result<Vec<u16>, GetStringFailure> {
    inq_string_internal(tables, info, retain_metas).map(|text| unescape_string(&text))
}

/// `StringInfo::GetString(bRetainMetas)`: `InqString`'s text, or the failure's text.
pub fn get_string(tables: &dyn StringTables, info: &RenderInfo, retain_metas: bool) -> Vec<u16> {
    inq_string(tables, info, retain_metas).unwrap_or_else(|failure| failure.text)
}

/// `StringInfo::GetStringWithMetaTags`: `GetString(true)`.
pub fn get_string_with_meta_tags(tables: &dyn StringTables, info: &RenderInfo) -> Vec<u16> {
    get_string(tables, info, true)
}

/// `StringInfo::GetLogString(bRetainMetaTags)`: `(text)` (not unescaped), or
/// `<could not render string: table 0x%08X token 0x%08X. Reason = %d>`.
pub fn get_log_string(
    tables: &dyn StringTables,
    info: &RenderInfo,
    retain_metas: bool,
) -> Vec<u16> {
    match inq_string_internal(tables, info, retain_metas) {
        Ok(text) => {
            let mut out = wide("(");
            out.extend(text);
            out.push(u16::from(b')'));
            out
        }
        Err(failure) => wide(&format!(
            "<could not render string: table 0x{:08X} token 0x{:08X}. Reason = {}>",
            info.table_id, info.string_id, failure.code as u32
        )),
    }
}

/// The body of `InqStringInternal` after the table lookup: `info`'s variables set into a
/// `StringVarTable` and `table`'s entry rendered (`StringTable::GetString`, vtable +0x80),
/// meta letters kept, not unescaped. This is ac2-script's `ScriptData::render_string`.
pub fn render_with_table(
    tables: &dyn StringTables,
    table: &StringTable,
    info: &RenderInfo,
) -> Result<Vec<u16>, GetStringError> {
    let vars = var_table(tables, &info.variables, info.table_id, table, 0);
    let mut out = Vec::new();
    table.render_entry(tables, info.string_id, &vars, &mut out)?;
    Ok(out)
}
