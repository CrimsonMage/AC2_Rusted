//! `gEnumMapper` and `gFileArray`: enum names and data-file ids
//! (`gEnumMapper.cpp`, `gFileArray.cpp`; `EnumMapper`, `EnumIDMap`, `FileArray`).
//!
//! A map id names its table through `FileArray::GetDIDFromEnum(mapid, 0x40000001)`; the table
//! is an `EnumMapper` (lookups fall through to `m_base_emp`) or an `EnumIDMap`.

use super::{narrow, Call, INT_RESULT};
use crate::data::{EnumIdMap, EnumMapper, DB_TYPE_DID_MAPPER, DB_TYPE_ENUM_MAPPER, EMAPPER_GROUP};
use crate::native::{ntype, NativeObject, ScriptList};
use crate::vm::{tag, Context, EFuns, InterpStatus, ScriptData, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0023_0001, inq_string);
    efuns.register(0x0023_0002, get_string_w);
    efuns.register(0x0023_0003, inq_enum);
    efuns.register(0x0023_0004, get_log_string_w);
    efuns.register(0x0023_0005, get_num_mappings);
    efuns.register(0x0023_0008, get_all_enum_vals);
    efuns.register(0x0023_0009, get_all_enum_names);
    efuns.register(0x0023_000A, get_enum_by_did_w);
    efuns.register(0x0027_0001, get_did_from_enum);
    efuns.register(0x0027_0002, get_did_for_dummies_w);
    efuns.register(0x0027_0003, get_did_w);
    efuns.register(0x0027_0004, get_file_name_for_dummies_w);
    efuns.register(0x1AC0_0001, get_did_from_enum_static);
    efuns.register(0x0022_000F, get_data_id_from_entity_name_w);
    // `DBCache::IsRunTime`, `IsClient`, `IsServer`: the client's cache is a run-time client.
    efuns.register(0x1AC0_0002, |_, ctx| flag(ctx, true));
    efuns.register(0x1AC0_0003, |_, ctx| flag(ctx, true));
    efuns.register(0x1AC0_0004, |_, ctx| flag(ctx, false));
}

fn flag(ctx: &mut Context, on: bool) -> InterpStatus {
    Call::new(ctx).ret(2, u32::from(on), INT_RESULT);
    InterpStatus::Success
}

/// `DBCache::GetDIDFromEnumStatic(enum_id, enum_group)`.
fn get_did_from_enum_static(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, group) = (call.word(2), call.word(3));
    let did = vm.data().enum_did(id, group).unwrap_or(0);
    Call::new(ctx).ret(4, did, INT_RESULT);
    InterpStatus::Success
}

/// The base-mapper chain depth retail could follow before a cycle would hang it.
const MAX_CHAIN: usize = 64;

/// A resolved enum table.
enum Table {
    Mapper(Box<dyn std::ops::Deref<Target = EnumMapper>>),
    IdMap(Box<dyn std::ops::Deref<Target = EnumIdMap>>),
}

/// `MasterDBMap::DivineType` then `DBObj::Get` of the right type.
fn table_by_did(data: &dyn ScriptData, did: u32) -> Option<Table> {
    match data.db_type(did)? {
        DB_TYPE_ENUM_MAPPER => data.enum_mapper(did).map(Table::Mapper),
        DB_TYPE_DID_MAPPER => data.enum_id_map(did).map(Table::IdMap),
        _ => None,
    }
}

fn table(data: &dyn ScriptData, mapid: u32) -> Option<Table> {
    let did = data.enum_did(mapid, EMAPPER_GROUP).filter(|&d| d != 0)?;
    table_by_did(data, did)
}

/// The mapper and its bases, nearest first.
fn chain(
    data: &dyn ScriptData,
    first: Box<dyn std::ops::Deref<Target = EnumMapper>>,
) -> Vec<Box<dyn std::ops::Deref<Target = EnumMapper>>> {
    let mut out = vec![first];
    while out.len() < MAX_CHAIN {
        let base = out.last().map_or(0, |m| m.base);
        match (base != 0).then(|| data.enum_mapper(base)).flatten() {
            Some(m) => out.push(m),
            None => break,
        }
    }
    out
}

/// `EnumMapper::GetString` / `EnumIDMap::GetName`.
fn string_of(data: &dyn ScriptData, t: Table, value: u32) -> Option<Vec<u8>> {
    match t {
        Table::Mapper(m) => chain(data, m)
            .iter()
            .find_map(|m| m.name(value).map(<[u8]>::to_vec)),
        Table::IdMap(m) => m.name(value).map(<[u8]>::to_vec),
    }
}

/// `EnumMapper::GetEnum` / `EnumIDMap::NameToEnum`.
fn enum_of(data: &dyn ScriptData, t: Table, name: &[u8]) -> Option<u32> {
    match t {
        Table::Mapper(m) => chain(data, m).iter().find_map(|m| m.value(name)),
        Table::IdMap(m) => m.name_to_enum(name),
    }
}

fn widen(s: &[u8]) -> Vec<u16> {
    s.iter().map(|&b| u16::from(b)).collect()
}

/// `gEnumMapper::CallEFun_InqString(mapid, value, out wpstring)`.
fn inq_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (mapid, value, out) = (call.word(2), call.word(3), call.word(4));
    let text = table(vm.data(), mapid).and_then(|t| string_of(vm.data(), t, value));
    let target = vm.read_through(ctx, out);
    let found = text.is_some();
    if let (Some(text), Some(target)) = (text, target) {
        if let Some(NativeObject::WString(s)) = vm.native_mut(target) {
            *s = widen(&text);
        }
    }
    Call::new(ctx).ret(5, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

/// `EnumMapper::GetStringW(mapid, value)`: the name or an empty string.
fn get_string_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (mapid, value) = (call.word(2), call.word(3));
    let text = table(vm.data(), mapid)
        .and_then(|t| string_of(vm.data(), t, value))
        .unwrap_or_default();
    let r = vm.new_wstring(widen(&text));
    Call::new(ctx).ret(4, r, tag::REF);
    InterpStatus::Success
}

/// `EnumMapper::GetLogStringW`: the same lookup, through the narrow string.
fn get_log_string_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    get_string_w(vm, ctx)
}

/// `gEnumMapper::CallEFun_InqEnum(mapid, name, out value)`.
fn inq_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (mapid, name, out) = (call.word(2), call.word(3), call.word(4));
    let name = vm.wstring(name).map(narrow).unwrap_or_default();
    let value = table(vm.data(), mapid).and_then(|t| enum_of(vm.data(), t, &name));
    finish_out(vm, ctx, out, value)
}

/// `gEnumMapper::CallEFun_GetEnumByDIDW(did, name, out value)`.
fn get_enum_by_did_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (did, name, out) = (call.word(2), call.word(3), call.word(4));
    let name = vm.wstring(name).map(narrow).unwrap_or_default();
    let value = table_by_did(vm.data(), did).and_then(|t| enum_of(vm.data(), t, &name));
    finish_out(vm, ctx, out, value)
}

/// The out slot is tagged int whether or not a value is found; the value is written only
/// when found; the result is 1/0.
fn finish_out(vm: &mut Vm, ctx: &mut Context, out: u32, value: Option<u32>) -> InterpStatus {
    let current = vm.read_through(ctx, out).unwrap_or(0);
    vm.write_through(ctx, out, value.unwrap_or(current), INT_RESULT);
    Call::new(ctx).ret(5, u32::from(value.is_some()), INT_RESULT);
    InterpStatus::Success
}

/// `EnumMapper::GetNumMappings(mapid)`.
fn get_num_mappings(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mapid = Call::new(ctx).word(2);
    let n = match table(vm.data(), mapid) {
        Some(Table::Mapper(m)) => chain(vm.data(), m).iter().map(|m| m.names.len()).sum(),
        Some(Table::IdMap(m)) => m.to_id.len(),
        None => 0,
    };
    Call::new(ctx).ret(3, n as u32, INT_RESULT);
    InterpStatus::Success
}

/// `EnumMapper::GetAllEnumVals(mapid)` into a new `AList` (mapper entries in id order, then the
/// base's; an `EnumIDMap` gives its `m_EnumToID` values).
fn get_all_enum_vals(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mapid = Call::new(ctx).word(2);
    let items: Vec<u64> = match table(vm.data(), mapid) {
        Some(Table::Mapper(m)) => chain(vm.data(), m)
            .iter()
            .flat_map(|m| {
                m.names
                    .iter()
                    .map(|(id, _)| u64::from(*id))
                    .collect::<Vec<_>>()
            })
            .collect(),
        Some(Table::IdMap(m)) => m.to_id.iter().map(|(_, did)| u64::from(*did)).collect(),
        None => Vec::new(),
    };
    let list = NativeObject::List(ScriptList { wide: false, items });
    let r = vm.refs.create_native(ntype::A_LIST, list);
    Call::new(ctx).ret(3, r, tag::REF);
    InterpStatus::Success
}

/// `EnumMapper::GetAllEnumNames(mapid)` into a new `RList` of `wpstring`s.
fn get_all_enum_names(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mapid = Call::new(ctx).word(2);
    let names: Vec<Vec<u8>> = match table(vm.data(), mapid) {
        Some(Table::Mapper(m)) => chain(vm.data(), m)
            .iter()
            .flat_map(|m| m.names.iter().map(|(_, n)| n.clone()).collect::<Vec<_>>())
            .collect(),
        Some(Table::IdMap(m)) => m.to_name.iter().map(|(_, n)| n.clone()).collect(),
        None => Vec::new(),
    };
    let items = names
        .iter()
        .map(|n| u64::from(vm.new_wstring(widen(n))))
        .collect();
    let list = NativeObject::List(ScriptList { wide: false, items });
    let r = vm.refs.create_native(ntype::R_LIST, list);
    Call::new(ctx).ret(3, r, tag::REF);
    InterpStatus::Success
}

/// `FileArray::GetDIDFromEnum(enum_id, enum_group)`.
fn get_did_from_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (id, group) = (call.word(2), call.word(3));
    let did = vm.data().enum_did(id, group).unwrap_or(0);
    Call::new(ctx).ret(4, did, INT_RESULT);
    InterpStatus::Success
}

/// `FileArray::GetDIDForDummiesW(name)`: the name's own type, then `GetDID`.
fn get_did_for_dummies_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let name = vm
        .wstring(Call::new(ctx).word(2))
        .map(narrow)
        .unwrap_or_default();
    let did = vm.data().file_did(&name, 0).unwrap_or(0);
    Call::new(ctx).ret(3, did, INT_RESULT);
    InterpStatus::Success
}

/// `FileArray::GetDIDW(name, type)`.
fn get_did_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (name, kind) = (call.word(2), call.word(3));
    let name = vm.wstring(name).map(narrow).unwrap_or_default();
    let did = vm.data().file_did(&name, kind).unwrap_or(0);
    Call::new(ctx).ret(4, did, INT_RESULT);
    InterpStatus::Success
}

/// `FileArray::GetFileNameForDummiesW(did)`.
fn get_file_name_for_dummies_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let did = Call::new(ctx).word(2);
    let name = vm.data().file_name(did).unwrap_or_default();
    let r = vm.new_wstring(widen(&name));
    Call::new(ctx).ret(3, r, tag::REF);
    InterpStatus::Success
}

/// `EntityDesc::GetDataIDFromEntityNameW(name)`: `FileArray::GetDID(name, 0x40000049)`.
fn get_data_id_from_entity_name_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let name = vm
        .wstring(Call::new(ctx).word(2))
        .map(narrow)
        .unwrap_or_default();
    let did = vm.data().file_did(&name, 0x4000_0049).unwrap_or(0);
    Call::new(ctx).ret(3, did, INT_RESULT);
    InterpStatus::Success
}
