//! `gStringInfo`: building and querying `StringInfo` natives
//! (`portal/shared/src/interp/glue/gStringInfo.cpp`, `StringInfo` in `stringtable/StringInfo.cpp`).
//!
//! The text (`GetString`, `InqString`, `IsValid`, ...) is in [`super::string_text`].
//! Table names and enums resolve through [`crate::vm::ScriptData`]; a table that is not a
//! string table (`DB_TYPE_STRING_TABLE`, 0x40000026) leaves the id 0, as retail does.

use super::{Call, INT_RESULT};
use crate::native::{NativeObject, StringInfo, StringVar, StringVarValue};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

/// `DB_TYPE_STRING_TABLE`.
const STRING_TABLE: u32 = 0x4000_0026;
/// The enum class `FileArray::GetDIDFromEnum` takes for string tables.
pub(super) const STRING_TABLE_ENUM: u32 = 0x4000_0004;

pub(super) fn register(efuns: &mut EFuns) {
    let table: [(u32, crate::vm::EFunFn); 27] = [
        (0x01, add_string_id_and_table_enum),
        (0x02, add_string),
        (0x03, add_int),
        (0x04, add_formatted_int),
        (0x05, add_uint),
        (0x06, add_formatted_uint),
        (0x07, add_float),
        (0x08, add_formatted_float),
        (0x09, add_string_info),
        (0x0A, set_string_id),
        (0x0B, set_table_enum),
        (0x0C, set_string_id_and_table_enum),
        (0x0D, set_table_name),
        (0x0E, set_string_id_and_table_name),
        (0x0F, set_string_id_and_table_id),
        (0x12, set_literal_value),
        (0x13, is_literal_value),
        (0x14, has_string_id),
        (0x15, has_table_id),
        (0x16, has_string_id_and_table_id),
        (0x19, reset),
        (0x1F, get_string_id),
        (0x20, get_table_id),
        (0x22, add_lint),
        (0x23, add_formatted_lint),
        (0x24, add_ulint),
        (0x25, add_formatted_ulint),
    ];
    for (n, f) in table {
        efuns.register(0x006A_0000 | n, f);
    }
}

/// Runs `f` on `this`'s StringInfo (`index != Undef_ReferenceID` otherwise).
fn with_this(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut StringInfo, &mut Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::StringInfo(info)) => {
            f(info, ctx);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `StringInfo::AddVar`: a variable replaces any with the same id.
fn add_var(info: &mut StringInfo, id: u32, kind: u16, value: StringVarValue) {
    let var = StringVar { id, kind, value };
    match info.variables.iter_mut().find(|v| v.id == id) {
        Some(slot) => *slot = var,
        None => info.variables.push(var),
    }
}

/// The table id a string-table DID gives (`~-(DivineType != STRING_TABLE) & did`).
fn table_id(vm: &Vm, did: u32) -> u32 {
    if vm.data().db_type(did) == Some(STRING_TABLE) {
        did
    } else {
        0
    }
}

/// `StringTableMetaLanguage::EscapeString`: a backslash before each of `[]!{}#\|^`.
pub(crate) fn escape(text: &[u16]) -> Vec<u16> {
    const META: &str = "[]!{}#\\|^";
    let mut out = Vec::with_capacity(text.len());
    for &c in text {
        if c != 0 && META.encode_utf16().any(|m| m == c) {
            out.push(u16::from(b'\\'));
        }
        out.push(c);
    }
    out
}

fn add_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, text) = (
        call.word(2),
        vm.wstring(call.word(3)).unwrap_or(&[]).to_vec(),
    );
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 1, StringVarValue::String(text))
    })
}

fn add_int(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, v) = (call.word(2), call.word(3) as i32);
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 2, StringVarValue::Int(v))
    })
}

fn add_uint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, v) = (call.word(2), call.word(3));
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 6, StringVarValue::UInt(v))
    })
}

/// `AddVariable_LInt(id, long)`: the long takes two slots (its low word is the lower one).
fn add_lint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, v) = (call.word(2), call.word64(4) as i64);
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 10, StringVarValue::LInt(v))
    })
}

/// `AddVariable_ULInt(id, ulong64)`.
fn add_ulint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, v) = (call.word(2), call.word64(4));
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 12, StringVarValue::ULInt(v))
    })
}

/// `AddVariable_FormattedLInt(id, long, grouping, base)`.
fn add_formatted_lint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, value) = (call.word(2), call.word64(4) as i64);
    let (grouping, base) = (call.word(5) as i32, call.word(6) as u16);
    with_this(vm, ctx, |info, _| {
        add_var(
            info,
            id,
            11,
            StringVarValue::FormattedLInt {
                value,
                grouping,
                base,
            },
        )
    })
}

/// `AddVariable_FormattedULInt(id, ulong64, grouping, base)`.
fn add_formatted_ulint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, value) = (call.word(2), call.word64(4));
    let (grouping, base) = (call.word(5) as i32, call.word(6) as u16);
    with_this(vm, ctx, |info, _| {
        add_var(
            info,
            id,
            13,
            StringVarValue::FormattedULInt {
                value,
                grouping,
                base,
            },
        )
    })
}

fn add_formatted_int(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, value, grouping, base) = (
        call.word(2),
        call.word(3) as i32,
        call.word(4) as i32,
        call.word(5) as u16,
    );
    with_this(vm, ctx, |info, _| {
        add_var(
            info,
            id,
            3,
            StringVarValue::FormattedInt {
                value,
                grouping,
                base,
            },
        )
    })
}

fn add_formatted_uint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, value, grouping, base) = (
        call.word(2),
        call.word(3),
        call.word(4) as i32,
        call.word(5) as u16,
    );
    with_this(vm, ctx, |info, _| {
        add_var(
            info,
            id,
            7,
            StringVarValue::FormattedUInt {
                value,
                grouping,
                base,
            },
        )
    })
}

/// `AddVariable_Float(id, float, precision)`: the float widens to the stored double.
fn add_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let id = call.word(2);
    let value = f64::from(f32::from_bits(call.word(3)));
    let precision = call.word(4) as u16;
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 4, StringVarValue::Float { value, precision })
    })
}

/// `AddVariable_FormattedFloat(id, double, precision, grouping, base)`.
fn add_formatted_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let id = call.word(2);
    let value = f64::from_bits(call.word64(4));
    let (precision, grouping, base) = (
        call.word(5) as u16,
        call.word(6) as i32,
        call.word(7) as u16,
    );
    with_this(vm, ctx, |info, _| {
        add_var(
            info,
            id,
            5,
            StringVarValue::FormattedFloat {
                value,
                precision,
                grouping,
                base,
            },
        )
    })
}

/// `AddVariable_StringInfo(id, other)`: a copy of the other StringInfo.
fn add_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let id = call.word(2);
    let other = match vm.native(call.word(3)) {
        Some(NativeObject::StringInfo(o)) => o.clone(),
        _ => StringInfo::default(),
    };
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 9, StringVarValue::StringInfo(Box::new(other)))
    })
}

/// `AddVariable_StringIDandTableEnum(id, string id, table enum)`: a nested StringInfo.
fn add_string_id_and_table_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, string_id, table_enum) = (call.word(2), call.word(3), call.word(4));
    let did = vm
        .data()
        .enum_did(table_enum, STRING_TABLE_ENUM)
        .unwrap_or(0);
    let nested = StringInfo {
        string_id,
        table_id: table_id(vm, did),
        ..StringInfo::default()
    };
    with_this(vm, ctx, |info, _| {
        add_var(info, id, 9, StringVarValue::StringInfo(Box::new(nested)))
    })
}

fn set_string_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = Call::new(ctx).word(2);
    with_this(vm, ctx, |info, _| info.string_id = id)
}

/// `StringInfo::SetTableEnum`.
fn set_table_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let table_enum = Call::new(ctx).word(2);
    let did = vm
        .data()
        .enum_did(table_enum, STRING_TABLE_ENUM)
        .unwrap_or(0);
    let table = table_id(vm, did);
    with_this(vm, ctx, |info, _| info.table_id = table)
}

/// `StringInfo::SetStringIDandTableEnum`.
fn set_string_id_and_table_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, table_enum) = (call.word(2), call.word(3));
    let did = vm
        .data()
        .enum_did(table_enum, STRING_TABLE_ENUM)
        .unwrap_or(0);
    let table = table_id(vm, did);
    with_this(vm, ctx, |info, _| {
        info.string_id = id;
        info.table_id = table;
    })
}

/// `StringInfo::SetTableName`: a missing file leaves the table id as it was.
fn table_by_name(vm: &Vm, name_ref: u32) -> Option<u32> {
    let name = super::narrow(vm.wstring(name_ref)?);
    let did = vm.data().file_did(&name, STRING_TABLE)?;
    (did != 0).then(|| table_id(vm, did))
}

fn set_table_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let table = table_by_name(vm, Call::new(ctx).word(2));
    with_this(vm, ctx, |info, _| {
        if let Some(t) = table {
            info.table_id = t;
        }
    })
}

/// `StringInfo::SetStringIDandTableName`.
fn set_string_id_and_table_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let id = call.word(2);
    let table = table_by_name(vm, call.word(3));
    with_this(vm, ctx, |info, _| {
        info.string_id = id;
        if let Some(t) = table {
            info.table_id = t;
        }
    })
}

fn set_string_id_and_table_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, did) = (call.word(2), call.word(3));
    let table = table_id(vm, did);
    with_this(vm, ctx, |info, _| {
        info.string_id = id;
        info.table_id = table;
    })
}

/// `StringInfo::SetLiteralValue(value, escape = true)`.
fn set_literal_value(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = escape(vm.wstring(Call::new(ctx).word(2)).unwrap_or(&[]));
    with_this(vm, ctx, |info, _| info.literal = Some(value))
}

fn ret_flag(vm: &mut Vm, ctx: &mut Context, f: fn(&StringInfo) -> u32) -> InterpStatus {
    with_this(vm, ctx, |info, ctx| {
        let v = f(info);
        Call::new(ctx).ret(2, v, INT_RESULT);
    })
}

fn is_literal_value(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_flag(vm, ctx, |i| u32::from(i.literal.is_some()))
}

fn has_string_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_flag(vm, ctx, |i| u32::from(i.string_id != 0))
}

fn has_table_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_flag(vm, ctx, |i| u32::from(i.table_id != 0))
}

fn has_string_id_and_table_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_flag(vm, ctx, |i| u32::from(i.string_id != 0 && i.table_id != 0))
}

fn get_string_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_flag(vm, ctx, |i| i.string_id)
}

fn get_table_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    ret_flag(vm, ctx, |i| i.table_id)
}

/// `StringInfo::Reset`.
fn reset(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |info, _| *info = StringInfo::default())
}
