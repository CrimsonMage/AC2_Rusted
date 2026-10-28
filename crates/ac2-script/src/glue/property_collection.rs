//! `gPropertyCollection` and `gPropertyGroup`
//! (`portal/shared/src/interp/glue/gpropertycollection.cpp`, `gpropertygroup.cpp`): the property
//! sets a script builds (`AddOptionalProperty` has 251 call sites in the library) and walks, over
//! `ac2_db::PropertyCollection` / `PropertyGroup`.
//!
//! Every efun returns the int result of the method it calls: the adds answer 0 when the property
//! is already present, the inquiries copy the found object into their out native
//! (`BaseProperty::operator=`, `PropertyGroup::operator=`) and answer 1, or answer 0 and leave
//! the out alone. The `...W` inquiries take a `wpstring` name that `MasterProperty` must know
//! (`InqPropertyNameEnum`). An undefined `this` crashes the script.

use ac2_db::{ac2_property_name_enum, BaseProperty, PropertyCollection, PropertyGroup};

use super::enums::mapper_name;
use super::{narrow, out_native, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{Context, EFuns, InterpStatus, Vm};

/// The enum `PropertyGroup::InqPropertyGroupNameW` names groups with.
const PROPERTY_GROUP_ENUM: u32 = 0x4000_0014;

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let function: crate::vm::EFunFn = match name {
            "gPropertyCollection::CallEFun_AddProperty" => {
                |vm, ctx| collection_property(vm, ctx, PropertyCollection::add_property)
            }
            "gPropertyCollection::CallEFun_AddOptionalProperty" => {
                |vm, ctx| collection_property(vm, ctx, PropertyCollection::add_optional_property)
            }
            "gPropertyCollection::CallEFun_UpdateProperty" => {
                |vm, ctx| collection_property(vm, ctx, PropertyCollection::update_property)
            }
            "gPropertyCollection::CallEFun_ContainsGroup" => {
                |vm, ctx| collection_name(vm, ctx, PropertyCollection::contains_group)
            }
            "gPropertyCollection::CallEFun_ContainsProperty" => {
                |vm, ctx| collection_name(vm, ctx, PropertyCollection::contains_property)
            }
            "gPropertyCollection::CallEFun_InqGroup" => {
                |vm, ctx| collection_inq_group(vm, ctx, |c, name| c.get_group(name).cloned())
            }
            "gPropertyCollection::CallEFun_InqGroupAt" => {
                |vm, ctx| collection_inq_group(vm, ctx, |c, index| c.group_at(index).cloned())
            }
            "gPropertyCollection::CallEFun_InqProperty" => |vm, ctx| {
                collection_inq_property(vm, ctx, false, |c, name| c.get_property(name).cloned())
            },
            "gPropertyCollection::CallEFun_InqOptionalProperty" => |vm, ctx| {
                collection_inq_property(vm, ctx, false, |c, name| {
                    c.get_optional_property(name).cloned()
                })
            },
            "gPropertyCollection::CallEFun_InqPropertyW" => |vm, ctx| {
                collection_inq_property(vm, ctx, true, |c, name| c.get_property(name).cloned())
            },
            "gPropertyCollection::CallEFun_InqOptionalPropertyW" => |vm, ctx| {
                collection_inq_property(vm, ctx, true, |c, name| {
                    c.get_optional_property(name).cloned()
                })
            },
            "gPropertyCollection::CallEFun_GetNumGroups" => {
                |vm, ctx| collection_count(vm, ctx, |c| c.groups.len())
            }
            "gPropertyGroup::CallEFun_SetGroupName" => group_set_name,
            "gPropertyGroup::CallEFun_AddProperty" => {
                |vm, ctx| group_property(vm, ctx, PropertyGroup::add_property)
            }
            "gPropertyGroup::CallEFun_AddOptionalProperty" => {
                |vm, ctx| group_property(vm, ctx, PropertyGroup::add_optional_property)
            }
            "gPropertyGroup::CallEFun_UpdateProperty" => {
                |vm, ctx| group_property(vm, ctx, PropertyGroup::update_property)
            }
            "gPropertyGroup::CallEFun_RemoveProperty" => {
                |vm, ctx| group_name(vm, ctx, PropertyGroup::remove_property)
            }
            "gPropertyGroup::CallEFun_ContainsProperty" => {
                |vm, ctx| group_name(vm, ctx, |g, n| g.contains_property(n))
            }
            "gPropertyGroup::CallEFun_ContainsOptionalProperty" => {
                |vm, ctx| group_name(vm, ctx, |g, n| g.contains_optional_property(n))
            }
            "gPropertyGroup::CallEFun_InqProperty" => {
                |vm, ctx| group_inq(vm, ctx, |g, name| g.get_property(name).cloned())
            }
            "gPropertyGroup::CallEFun_InqOptionalProperty" => {
                |vm, ctx| group_inq(vm, ctx, |g, name| g.get_optional_property(name).cloned())
            }
            "gPropertyGroup::CallEFun_InqPropertyAt" => {
                |vm, ctx| group_inq(vm, ctx, |g, index| g.property_at(index).cloned())
            }
            "gPropertyGroup::CallEFun_InqOptionalPropertyAt" => {
                |vm, ctx| group_inq(vm, ctx, |g, index| g.optional_property_at(index).cloned())
            }
            "gPropertyGroup::CallEFun_GetNumProperties" => {
                |vm, ctx| group_count(vm, ctx, |g| g.properties.len())
            }
            "gPropertyGroup::CallEFun_GetNumOptionalProperties" => {
                |vm, ctx| group_count(vm, ctx, |g| g.optional.len())
            }
            "gPropertyGroup::CallEFun_InqPropertyGroupNameW" => group_name_w,
            _ => continue,
        };
        efuns.register(id, function);
    }
}

/// `this` and the word at 2.
fn this_and_arg(ctx: &mut Context) -> (u32, u32) {
    let call = Call::new(ctx);
    (call.this(), call.word(2))
}

fn ret(ctx: &mut Context, word: usize, value: u32) {
    Call::new(ctx).ret(word, value, INT_RESULT);
}

fn collection_mut(vm: &mut Vm, this: u32) -> Option<&mut PropertyCollection> {
    match vm.native_mut(this) {
        Some(NativeObject::PropertyCollection(c)) => Some(c),
        _ => None,
    }
}

fn group_mut(vm: &mut Vm, this: u32) -> Option<&mut PropertyGroup> {
    match vm.native_mut(this) {
        Some(NativeObject::PropertyGroup(g)) => Some(g),
        _ => None,
    }
}

/// The `BaseProperty` a property argument names.
fn property_arg(vm: &mut Vm, ctx: &Context, arg: u32) -> Result<BaseProperty, InterpStatus> {
    let reference = out_native(vm, ctx, arg);
    match vm.native(reference) {
        Some(NativeObject::BaseProperty(p)) => Ok((**p).clone()),
        _ => Err(vm.bad_this(reference)),
    }
}

/// Copies `property` into the `BaseProperty` out native (`BaseProperty::operator=`).
fn write_property(
    vm: &mut Vm,
    ctx: &Context,
    out: u32,
    property: BaseProperty,
) -> Result<(), InterpStatus> {
    let target = out_native(vm, ctx, out);
    match vm.native_mut(target) {
        Some(NativeObject::BaseProperty(slot)) => {
            **slot = property;
            Ok(())
        }
        _ => Err(vm.bad_this(target)),
    }
}

/// `Add/Update(property)` on the collection: the int result in word 3.
fn collection_property(
    vm: &mut Vm,
    ctx: &mut Context,
    op: fn(&mut PropertyCollection, &BaseProperty) -> bool,
) -> InterpStatus {
    let (this, arg) = this_and_arg(ctx);
    if collection_mut(vm, this).is_none() {
        return vm.bad_this(this);
    }
    let property = match property_arg(vm, ctx, arg) {
        Ok(p) => p,
        Err(status) => return status,
    };
    let ok = collection_mut(vm, this).is_some_and(|c| op(c, &property));
    ret(ctx, 3, u32::from(ok));
    InterpStatus::Success
}

/// A name query on the collection (`ContainsGroup`, `ContainsProperty`): result in word 3.
fn collection_name(
    vm: &mut Vm,
    ctx: &mut Context,
    query: fn(&PropertyCollection, u32) -> bool,
) -> InterpStatus {
    let (this, name) = this_and_arg(ctx);
    let Some(collection) = collection_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let found = query(collection, name);
    ret(ctx, 3, u32::from(found));
    InterpStatus::Success
}

/// `GetNumGroups`: the group count in word 2.
fn collection_count(
    vm: &mut Vm,
    ctx: &mut Context,
    count: fn(&PropertyCollection) -> usize,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(collection) = collection_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let n = count(collection) as u32;
    ret(ctx, 2, n);
    InterpStatus::Success
}

/// `InqGroup(name, out)` / `InqGroupAt(index, out)`: the group copied into the out native.
fn collection_inq_group(
    vm: &mut Vm,
    ctx: &mut Context,
    find: fn(&PropertyCollection, u32) -> Option<PropertyGroup>,
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, key, out) = (call.this(), call.word(2), call.word(3));
    let Some(collection) = collection_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let found = find(collection, key);
    let flag = found.is_some();
    if let Some(group) = found {
        let target = out_native(vm, ctx, out);
        match vm.native_mut(target) {
            Some(NativeObject::PropertyGroup(slot)) => **slot = group,
            _ => return vm.bad_this(target),
        }
    }
    ret(ctx, 4, u32::from(flag));
    InterpStatus::Success
}

/// `Inq[Optional]Property[W](name, out)`: the property copied into the out native. The `W` forms
/// take a `wpstring` and resolve it through `MasterProperty`; an unknown name answers 0.
fn collection_inq_property(
    vm: &mut Vm,
    ctx: &mut Context,
    wide: bool,
    find: fn(&PropertyCollection, u32) -> Option<BaseProperty>,
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, key, out) = (call.this(), call.word(2), call.word(3));
    if collection_mut(vm, this).is_none() {
        return vm.bad_this(this);
    }
    let name = if wide {
        let source = out_native(vm, ctx, key);
        let bytes = match vm.wstring(source) {
            Some(text) => narrow(text),
            None => return vm.bad_this(source),
        };
        ac2_property_name_enum(&bytes)
    } else {
        Some(key)
    };
    let found = name.and_then(|n| collection_mut(vm, this).and_then(|c| find(c, n)));
    let flag = found.is_some();
    if let Some(property) = found {
        if let Err(status) = write_property(vm, ctx, out, property) {
            return status;
        }
    }
    ret(ctx, 4, u32::from(flag));
    InterpStatus::Success
}

/// `SetGroupName(name)` (`BehaviorTracker::SetAnimationObj` on the group): stores the word, 1.
fn group_set_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, name) = this_and_arg(ctx);
    let Some(group) = group_mut(vm, this) else {
        return vm.bad_this(this);
    };
    group.name = name;
    ret(ctx, 3, 1);
    InterpStatus::Success
}

/// `Add/Update(property)` on the group: the int result in word 3.
fn group_property(
    vm: &mut Vm,
    ctx: &mut Context,
    op: fn(&mut PropertyGroup, &BaseProperty) -> bool,
) -> InterpStatus {
    let (this, arg) = this_and_arg(ctx);
    if group_mut(vm, this).is_none() {
        return vm.bad_this(this);
    }
    let property = match property_arg(vm, ctx, arg) {
        Ok(p) => p,
        Err(status) => return status,
    };
    let ok = group_mut(vm, this).is_some_and(|g| op(g, &property));
    ret(ctx, 3, u32::from(ok));
    InterpStatus::Success
}

/// A name operation on the group (`RemoveProperty`, `Contains...`): result in word 3.
fn group_name(
    vm: &mut Vm,
    ctx: &mut Context,
    op: fn(&mut PropertyGroup, u32) -> bool,
) -> InterpStatus {
    let (this, name) = this_and_arg(ctx);
    let Some(group) = group_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let ok = op(group, name);
    ret(ctx, 3, u32::from(ok));
    InterpStatus::Success
}

/// The property counts: the int in word 2.
fn group_count(vm: &mut Vm, ctx: &mut Context, count: fn(&PropertyGroup) -> usize) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(group) = group_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let n = count(group) as u32;
    ret(ctx, 2, n);
    InterpStatus::Success
}

/// `Inq[Optional]Property(name, out)` / `...At(index, out)` on the group.
fn group_inq(
    vm: &mut Vm,
    ctx: &mut Context,
    find: fn(&PropertyGroup, u32) -> Option<BaseProperty>,
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, key, out) = (call.this(), call.word(2), call.word(3));
    let Some(group) = group_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let found = find(group, key);
    let flag = found.is_some();
    if let Some(property) = found {
        if let Err(status) = write_property(vm, ctx, out, property) {
            return status;
        }
    }
    ret(ctx, 4, u32::from(flag));
    InterpStatus::Success
}

/// `InqPropertyGroupNameW(wpstring &)`: the group's name in the `0x40000014` enum
/// (`EnumMapper::GetString`), widened; the flag is the mapper's result and the string is empty
/// when it has none.
fn group_name_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, out) = this_and_arg(ctx);
    let name = match group_mut(vm, this) {
        Some(group) => group.name,
        None => return vm.bad_this(this),
    };
    let text = mapper_name(vm.data(), PROPERTY_GROUP_ENUM, name);
    let found = text.is_some();
    let wide: Vec<u16> = text
        .unwrap_or_default()
        .iter()
        .map(|&c| u16::from(c))
        .collect();
    let target = out_native(vm, ctx, out);
    match vm.native_mut(target) {
        Some(NativeObject::WString(out_text)) => *out_text = wide,
        _ => return vm.bad_this(target),
    }
    ret(ctx, 3, u32::from(found));
    InterpStatus::Success
}
