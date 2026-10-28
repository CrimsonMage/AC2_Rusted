//! `gBaseProperty`: the weenie property getters and setters a script calls on a `BaseProperty`
//! native (`portal/shared/src/interp/glue/gbaseproperty.cpp`, `BaseProperty::InqStringW`,
//! `BaseProperty::SetStringW`), over `ac2_db::BaseProperty` (one name, one type, one value).
//!
//! An `Inq*` efun writes the value into its out argument and returns 1. A property with no value,
//! or a value of another type, returns 0 and leaves the out argument alone (the value's getters
//! answer only for their own type). A `Set*` efun writes its argument when the value has that type
//! and returns the same flag. The string and vector efuns take a native (`wpstring`, `Vector`),
//! and the string output of a missing value is empty, as `InqStringW` leaves it.
//!
//! `InqStringInfo` / `SetStringInfo` copy a `StringInfo` native to and from the value
//! (`StringInfoPropertyValue::InqStringInfo` / `SetStringInfo`, the one-argument slots 0x80 and
//! 0x88). The name efuns read the `MasterProperty` names (`ac2_db::ac2_property_name`) and the
//! type names (`EnumMapper::GetString(0x40000013, type)`); `GetPropertyName`, `GetPropertyType`
//! and `GetGroupName` return the property's name, type and group words.

use ac2_db::{ac2_property_name, ac2_property_name_enum, BaseProperty, BasePropertyValue};
use ac2_math::Vec3;

use super::enums::mapper_name;
use super::{narrow, out_native, Call, INT_RESULT};
use crate::data::{render_info, script_string_info};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let function: crate::vm::EFunFn = match name {
            "gBaseProperty::CallEFun_InqBool" => inq_bool,
            "gBaseProperty::CallEFun_SetBool" => set_bool,
            "gBaseProperty::CallEFun_InqInteger" => inq_integer,
            "gBaseProperty::CallEFun_SetInteger" => set_integer,
            "gBaseProperty::CallEFun_InqLongInteger" => inq_long_integer,
            "gBaseProperty::CallEFun_InqFloat" => inq_float,
            "gBaseProperty::CallEFun_SetFloat" => set_float,
            "gBaseProperty::CallEFun_InqVector" => inq_vector,
            "gBaseProperty::CallEFun_SetVector" => set_vector,
            "gBaseProperty::CallEFun_InqStringW" => inq_string,
            "gBaseProperty::CallEFun_SetStringW" => set_string,
            "gBaseProperty::CallEFun_InqEnum" => inq_enum,
            "gBaseProperty::CallEFun_SetEnum" => set_enum,
            "gBaseProperty::CallEFun_InqDataID" => inq_data_id,
            "gBaseProperty::CallEFun_SetDataID" => set_data_id,
            "gBaseProperty::CallEFun_InqPackageID" => inq_package_id,
            "gBaseProperty::CallEFun_SetPackageID" => set_package_id,
            "gBaseProperty::CallEFun_InqStringInfo" => inq_string_info,
            "gBaseProperty::CallEFun_SetStringInfo" => set_string_info,
            "gBaseProperty::CallEFun_GetPropertyName" => get_property_name,
            "gBaseProperty::CallEFun_GetPropertyType" => get_property_type,
            "gBaseProperty::CallEFun_GetGroupName" => get_group_name,
            "gBaseProperty::CallEFun_InqPropertyNameW" => inq_property_name_w,
            "gBaseProperty::CallEFun_SetPropertyName" => set_property_name,
            "gBaseProperty::CallEFun_SetPropertyNameW" => set_property_name_w,
            "gBaseProperty::CallEFun_InqPropertyTypeW" => inq_property_type_w,
            _ => continue,
        };
        efuns.register(id, function);
    }
}

/// `this` and the argument at word 2, read before the efun touches the frame again.
fn this_and_arg(ctx: &mut Context) -> (u32, u32) {
    let call = Call::new(ctx);
    (call.this(), call.word(2))
}

/// The flag an efun returns (its return slot is word 3).
fn ret_flag(ctx: &mut Context, flag: bool) {
    Call::new(ctx).ret(3, u32::from(flag), INT_RESULT);
}

/// Runs `read` on the property's value at `this`: a crash for a bad `this`, `None` for no value
/// or a value of another type.
fn read_value<T>(
    vm: &mut Vm,
    this: u32,
    read: impl FnOnce(&BasePropertyValue) -> Option<T>,
) -> Result<Option<T>, InterpStatus> {
    match vm.native(this) {
        Some(NativeObject::BaseProperty(property)) => Ok(property.value.as_ref().and_then(read)),
        _ => Err(vm.bad_this(this)),
    }
}

/// An `Inq*` scalar efun: `read` gives the word the out reference (word 2) receives, `out_tag`
/// its slot tag.
fn inq_scalar(
    vm: &mut Vm,
    ctx: &mut Context,
    read: fn(&BasePropertyValue) -> Option<u32>,
    out_tag: u8,
) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let found = match read_value(vm, this, read) {
        Ok(found) => found,
        Err(status) => return status,
    };
    let Some(value) = found else {
        ret_flag(ctx, false);
        return InterpStatus::Success;
    };
    if !vm.write_through(ctx, out, value, out_tag) {
        return vm.bad_this(out);
    }
    ret_flag(ctx, true);
    InterpStatus::Success
}

/// A `Set*` scalar efun: `apply` writes the argument word when the value has the right type.
fn set_scalar(
    vm: &mut Vm,
    ctx: &mut Context,
    apply: fn(&mut BasePropertyValue, u32) -> bool,
) -> InterpStatus {
    let (this, value) = this_and_arg(ctx);
    let ok = match vm.native_mut(this) {
        Some(NativeObject::BaseProperty(property)) => property
            .value
            .as_mut()
            .is_some_and(|slot| apply(slot, value)),
        _ => return vm.bad_this(this),
    };
    ret_flag(ctx, ok);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_InqBool(bool &)`.
fn inq_bool(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    inq_scalar(vm, ctx, |v| v.inq_bool().map(u32::from), tag::BOOL)
}

/// `gBaseProperty::CallEFun_SetBool(bool)`.
fn set_bool(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_scalar(vm, ctx, |v, x| v.set_bool(x != 0))
}

/// `gBaseProperty::CallEFun_InqInteger(int &)`.
fn inq_integer(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    inq_scalar(vm, ctx, |v| v.inq_integer().map(|i| i as u32), tag::INT)
}

/// `gBaseProperty::CallEFun_SetInteger(int)`.
fn set_integer(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_scalar(vm, ctx, |v, x| v.set_integer(x as i32))
}

/// `gBaseProperty::CallEFun_InqLongInteger(long long &)`: a LongInteger value, written to the
/// 64-bit out reference (tag 0x84).
fn inq_long_integer(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let found = match read_value(vm, this, BasePropertyValue::inq_long_integer) {
        Ok(found) => found,
        Err(status) => return status,
    };
    if let Some(value) = found {
        super::lists::write_out(vm, ctx, out, Some(value as u64), super::lists::Kind::L);
    }
    ret_flag(ctx, found.is_some());
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_InqFloat(float &)`: the float's bits.
fn inq_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    inq_scalar(vm, ctx, |v| v.inq_float().map(f32::to_bits), tag::FLOAT)
}

/// `gBaseProperty::CallEFun_SetFloat(float)`.
fn set_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_scalar(vm, ctx, |v, x| v.set_float(f32::from_bits(x)))
}

/// `gBaseProperty::CallEFun_InqEnum(int &)`.
fn inq_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    inq_scalar(vm, ctx, BasePropertyValue::inq_enum, tag::INT)
}

/// `gBaseProperty::CallEFun_SetEnum(int)`.
fn set_enum(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_scalar(vm, ctx, |v, x| v.set_enum(x))
}

/// `gBaseProperty::CallEFun_InqDataID(int &)`: the DataFile value's DID word.
fn inq_data_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    inq_scalar(vm, ctx, BasePropertyValue::inq_data_file, tag::INT)
}

/// `gBaseProperty::CallEFun_SetDataID(int)`.
fn set_data_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_scalar(vm, ctx, |v, x| v.set_data_file(x))
}

/// `gBaseProperty::CallEFun_InqPackageID(int &)`.
fn inq_package_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    inq_scalar(vm, ctx, BasePropertyValue::inq_package_id, tag::INT)
}

/// `gBaseProperty::CallEFun_SetPackageID(int)`.
fn set_package_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_scalar(vm, ctx, |v, x| v.set_package_id(x))
}

/// `gBaseProperty::CallEFun_InqVector(Vector &)`: writes the vector native the argument names.
fn inq_vector(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let found = match read_value(vm, this, BasePropertyValue::inq_vector) {
        Ok(found) => found,
        Err(status) => return status,
    };
    if let Some(value) = found {
        let target = out_native(vm, ctx, out);
        match vm.native_mut(target) {
            Some(NativeObject::Vector(out_vector)) => *out_vector = value.to_array(),
            _ => return vm.bad_this(target),
        }
    }
    ret_flag(ctx, found.is_some());
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_SetVector(Vector)`: the argument's components.
fn set_vector(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = this_and_arg(ctx);
    let source = out_native(vm, ctx, arg);
    let input = match vm.native(source) {
        Some(NativeObject::Vector(components)) => Vec3::from_array(*components),
        _ => return vm.bad_this(source),
    };
    let ok = match vm.native_mut(this) {
        Some(NativeObject::BaseProperty(property)) => property
            .value
            .as_mut()
            .is_some_and(|slot| slot.set_vector(input)),
        _ => return vm.bad_this(this),
    };
    ret_flag(ctx, ok);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_InqStringW(wpstring &)`: the string value, widened. Without a string
/// value the output is empty and the flag is 0.
fn inq_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let found = match read_value(vm, this, |v| v.inq_string().map(<[u8]>::to_vec)) {
        Ok(found) => found,
        Err(status) => return status,
    };
    let ok = found.is_some();
    let text: Vec<u16> = found
        .unwrap_or_default()
        .into_iter()
        .map(u16::from)
        .collect();
    let target = out_native(vm, ctx, out);
    match vm.native_mut(target) {
        Some(NativeObject::WString(out_text)) => *out_text = text,
        _ => return vm.bad_this(target),
    }
    ret_flag(ctx, ok);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_SetStringW(wpstring)`: the string is narrowed to bytes.
fn set_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = this_and_arg(ctx);
    let source = out_native(vm, ctx, arg);
    let bytes = match vm.wstring(source) {
        Some(text) => narrow(text),
        None => return vm.bad_this(source),
    };
    let ok = match vm.native_mut(this) {
        Some(NativeObject::BaseProperty(property)) => property
            .value
            .as_mut()
            .is_some_and(|slot| slot.set_string(&bytes)),
        _ => return vm.bad_this(this),
    };
    ret_flag(ctx, ok);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_InqStringInfo(StringInfo &)`: copies the value's `StringInfo` into the
/// out native. A property without a `StringInfo` value answers 0 and leaves the out alone.
fn inq_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let found = match read_value(vm, this, |v| v.inq_string_info().cloned()) {
        Ok(found) => found,
        Err(status) => return status,
    };
    let Some(info) = found else {
        ret_flag(ctx, false);
        return InterpStatus::Success;
    };
    let target = out_native(vm, ctx, out);
    match vm.native_mut(target) {
        Some(NativeObject::StringInfo(out_info)) => *out_info = script_string_info(&info),
        _ => return vm.bad_this(target),
    }
    ret_flag(ctx, true);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_SetStringInfo(StringInfo)`: the argument's `StringInfo` becomes the
/// value, when the value is a `StringInfo`.
fn set_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = this_and_arg(ctx);
    let source = out_native(vm, ctx, arg);
    let info = match vm.native(source) {
        Some(NativeObject::StringInfo(info)) => render_info(info),
        _ => return vm.bad_this(source),
    };
    let ok = match vm.native_mut(this) {
        Some(NativeObject::BaseProperty(property)) => property
            .value
            .as_mut()
            .is_some_and(|slot| slot.set_string_info(info)),
        _ => return vm.bad_this(this),
    };
    ret_flag(ctx, ok);
    InterpStatus::Success
}

/// The `BaseProperty` `this` names, read as one word: the out word of the 0-argument getters.
fn property_word(vm: &mut Vm, ctx: &mut Context, read: fn(&BaseProperty) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let word = match vm.native(this) {
        Some(NativeObject::BaseProperty(property)) => read(property),
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(2, word, INT_RESULT);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_GetPropertyName()`: `BaseProperty::m_propertyName`.
fn get_property_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_word(vm, ctx, |p| p.name)
}

/// `gBaseProperty::CallEFun_GetPropertyType()`: `BaseProperty::m_propertyType`.
fn get_property_type(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_word(vm, ctx, |p| p.value_type)
}

/// `gBaseProperty::CallEFun_GetGroupName()`: `BaseProperty::m_propertyGroup` (the group word the
/// retail efun returns).
fn get_group_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_word(vm, ctx, |p| p.group)
}

/// Widens a name for a `wpstring` out argument.
fn widen(text: &[u8]) -> Vec<u16> {
    text.iter().map(|&c| u16::from(c)).collect()
}

/// Writes `text` into the `wpstring` out argument `out` and returns the flag.
fn write_wstring(
    vm: &mut Vm,
    ctx: &mut Context,
    out: u32,
    text: Vec<u16>,
    found: bool,
) -> InterpStatus {
    let target = out_native(vm, ctx, out);
    match vm.native_mut(target) {
        Some(NativeObject::WString(out_text)) => *out_text = text,
        _ => return vm.bad_this(target),
    }
    ret_flag(ctx, found);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_InqPropertyNameW(wpstring &)`: the property's name; the flag is 0 and
/// the string empty when the `MasterProperty` names no such property.
fn inq_property_name_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let name = match vm.native(this) {
        Some(NativeObject::BaseProperty(property)) => property.name,
        _ => return vm.bad_this(this),
    };
    let text = ac2_property_name(name);
    let found = text.is_some();
    write_wstring(vm, ctx, out, widen(text.unwrap_or_default()), found)
}

/// `gBaseProperty::CallEFun_InqPropertyTypeW(wpstring &)`: the name of the property's type in the
/// `0x40000013` enum (`EnumMapper::GetString`); empty and 0 when it has none.
fn inq_property_type_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let value_type = match vm.native(this) {
        Some(NativeObject::BaseProperty(property)) => property.value_type,
        _ => return vm.bad_this(this),
    };
    let text = mapper_name(vm.data(), PROPERTY_TYPE_ENUM, value_type);
    let found = text.is_some();
    write_wstring(vm, ctx, out, widen(&text.unwrap_or_default()), found)
}

/// The enum `BaseProperty::InqPropertyTypeW` names property types with.
const PROPERTY_TYPE_ENUM: u32 = 0x4000_0013;

/// `gBaseProperty::CallEFun_SetPropertyName(int)`: `BaseProperty::SetPropertyName(name)`.
fn set_property_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, name) = this_and_arg(ctx);
    let ok = match vm.native_mut(this) {
        Some(NativeObject::BaseProperty(property)) => property.set_property_name(name),
        _ => return vm.bad_this(this),
    };
    ret_flag(ctx, ok);
    InterpStatus::Success
}

/// `gBaseProperty::CallEFun_SetPropertyNameW(wpstring)`: the name is looked up in the
/// `MasterProperty` names (`InqPropertyNameEnum`) and set; an unknown name changes nothing.
fn set_property_name_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = this_and_arg(ctx);
    if !matches!(vm.native(this), Some(NativeObject::BaseProperty(_))) {
        return vm.bad_this(this);
    }
    let source = out_native(vm, ctx, arg);
    let bytes = match vm.wstring(source) {
        Some(text) => narrow(text),
        None => return vm.bad_this(source),
    };
    let ok = match ac2_property_name_enum(&bytes) {
        Some(name) => match vm.native_mut(this) {
            Some(NativeObject::BaseProperty(property)) => property.set_property_name(name),
            _ => false,
        },
        None => false,
    };
    ret_flag(ctx, ok);
    InterpStatus::Success
}
