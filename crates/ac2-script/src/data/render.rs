//! `ScriptData::render_string` over ac2-db's string renderer (`StringTable::GetString`,
//! vtable +0x80, with `StringTableMetaLanguage::RenderString`): the script's `StringInfo` as
//! ac2-db's `RenderInfo`, and the tables and `StringState` through the script data.

use ac2_db::{RenderInfo, RenderVar, StringStateRef, StringTableRef, StringTables};

use super::DatScriptData;
use crate::native::{GetStringError, StringInfo, StringVarValue};
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
