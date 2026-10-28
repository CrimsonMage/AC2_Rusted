//! `ScriptData::render_string` over ac2-db's string renderer (`StringTable::GetString`,
//! vtable +0x80, with `StringTableMetaLanguage::RenderString`): the script's `StringInfo` as
//! ac2-db's `RenderInfo`, and the tables and `StringState` through the script data.

use ac2_db::{RenderInfo, RenderVar, StringStateRef, StringTableRef, StringTables};

use super::DatScriptData;
use crate::native::{GetStringError, StringInfo, StringVar, StringVarValue};
use crate::vm::ScriptData;

/// The script's `StringInfo` as a `RenderInfo` (variables map one to one; a `SID_String`
/// variable is `AddVariable_String`'s raw text).
pub fn render_info(info: &StringInfo) -> RenderInfo {
    RenderInfo {
        string_id: info.string_id,
        table_id: info.table_id,
        literal: info.literal.clone(),
        variables: info
            .variables
            .iter()
            .map(|v| (v.id, render_var(&v.value)))
            .collect(),
    }
}

/// A `RenderInfo` (a `StringInfo` a property holds) as the script's `StringInfo` native
/// (`StringInfo::operator=`): each variable keeps the `SID_*` type its value implies.
pub fn script_string_info(info: &RenderInfo) -> StringInfo {
    StringInfo {
        string_id: info.string_id,
        table_id: info.table_id,
        literal: info.literal.clone(),
        variables: info
            .variables
            .iter()
            .filter_map(|(id, var)| {
                let (kind, value) = script_var(var)?;
                Some(StringVar {
                    id: *id,
                    kind,
                    value,
                })
            })
            .collect(),
    }
}

/// A render variable's `SID_*` type and the script's value for it; `None` for a skipped one.
fn script_var(var: &RenderVar) -> Option<(u16, StringVarValue)> {
    use StringVarValue as V;
    Some(match var {
        RenderVar::String(s) => (1, V::String(s.clone())),
        RenderVar::Int(v) => (2, V::Int(*v)),
        RenderVar::FormattedInt {
            value,
            grouping,
            base,
        } => (
            3,
            V::FormattedInt {
                value: *value,
                grouping: *grouping,
                base: *base,
            },
        ),
        RenderVar::Float { value, precision } => (
            4,
            V::Float {
                value: *value,
                precision: *precision,
            },
        ),
        RenderVar::FormattedFloat {
            value,
            precision,
            grouping,
            base,
        } => (
            5,
            V::FormattedFloat {
                value: *value,
                precision: *precision,
                grouping: *grouping,
                base: *base,
            },
        ),
        RenderVar::UInt(v) => (6, V::UInt(*v)),
        RenderVar::FormattedUInt {
            value,
            grouping,
            base,
        } => (
            7,
            V::FormattedUInt {
                value: *value,
                grouping: *grouping,
                base: *base,
            },
        ),
        RenderVar::Info(inner) => (9, V::StringInfo(Box::new(script_string_info(inner)))),
        RenderVar::LInt(v) => (10, V::LInt(*v)),
        RenderVar::FormattedLInt {
            value,
            grouping,
            base,
        } => (
            11,
            V::FormattedLInt {
                value: *value,
                grouping: *grouping,
                base: *base,
            },
        ),
        RenderVar::ULInt(v) => (12, V::ULInt(*v)),
        RenderVar::FormattedULInt {
            value,
            grouping,
            base,
        } => (
            13,
            V::FormattedULInt {
                value: *value,
                grouping: *grouping,
                base: *base,
            },
        ),
        RenderVar::Skipped => return None,
    })
}

fn render_var(value: &StringVarValue) -> RenderVar {
    use StringVarValue as V;
    match value {
        V::String(s) => RenderVar::String(s.clone()),
        V::Int(v) => RenderVar::Int(*v),
        V::FormattedInt {
            value,
            grouping,
            base,
        } => RenderVar::FormattedInt {
            value: *value,
            grouping: *grouping,
            base: *base,
        },
        V::Float { value, precision } => RenderVar::Float {
            value: *value,
            precision: *precision,
        },
        V::FormattedFloat {
            value,
            precision,
            grouping,
            base,
        } => RenderVar::FormattedFloat {
            value: *value,
            precision: *precision,
            grouping: *grouping,
            base: *base,
        },
        V::UInt(v) => RenderVar::UInt(*v),
        V::FormattedUInt {
            value,
            grouping,
            base,
        } => RenderVar::FormattedUInt {
            value: *value,
            grouping: *grouping,
            base: *base,
        },
        V::StringInfo(inner) => RenderVar::Info(Box::new(render_info(inner))),
        V::LInt(v) => RenderVar::LInt(*v),
        V::FormattedLInt {
            value,
            grouping,
            base,
        } => RenderVar::FormattedLInt {
            value: *value,
            grouping: *grouping,
            base: *base,
        },
        V::ULInt(v) => RenderVar::ULInt(*v),
        V::FormattedULInt {
            value,
            grouping,
            base,
        } => RenderVar::FormattedULInt {
            value: *value,
            grouping: *grouping,
            base: *base,
        },
        V::Skipped => RenderVar::Skipped,
    }
}

/// ac2-db's error code as the script's (the same `GetStringErrorCode` values).
pub fn script_error(e: ac2_db::GetStringError) -> GetStringError {
    use ac2_db::GetStringError as D;
    match e {
        D::Uninitialized => GetStringError::Uninitialized,
        D::MissingString => GetStringError::MissingString,
        D::MissingTable => GetStringError::MissingTable,
        D::MissingPrivateString => GetStringError::MissingPrivateString,
        D::MissingPrivateTable => GetStringError::MissingPrivateTable,
        D::MissingVariable => GetStringError::MissingVariable,
        D::MetaLanguageError => GetStringError::MetaLanguageError,
        D::GenericError => GetStringError::GenericError,
    }
}

impl StringTables for DatScriptData {
    /// `DBObj::Get` of a string table (cache-only: a miss is requested and the efun waits).
    fn table(&self, did: u32) -> Option<StringTableRef<'_>> {
        ScriptData::string_table(self, did).map(|t| t as StringTableRef<'_>)
    }

    /// `StringStateInfo::m_instance->m_state`, held resident by `preload_tables`.
    fn string_state(&self) -> Option<StringStateRef<'_>> {
        let state = self.resident().string_state.clone()?;
        Some(Box::new(state))
    }
}
