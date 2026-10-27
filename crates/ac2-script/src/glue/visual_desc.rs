//! `gVisualDesc`, `gAppliedAppearanceKey` (`portal/shared/src/interp/glue/gvisualdesc.cpp.c`,
//! `gappliedappearancekey.cpp.c`): the efuns a `CharacterGenSystem` script edits its appearance
//! with. The description is ac2-weenie's [`Appearance`] (the algebra is its `appearance_ops`);
//! the parent a description names is read through the data source (`visual_desc`), so a
//! parent that is not resident is a miss, as the entity efuns treat one.

use std::sync::Arc;

use ac2_weenie::Appearance;

use super::{out_native, word, Call, INT_RESULT};
use crate::native::{ntype, AppliedAppearanceKey, NativeObject, RefKey};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};

/// `VisualDesc::GetVisualDescFromDB`'s check: the DB type of a visual description.
const DB_TYPE_VISUAL_DESC: u32 = 0x4000_0021;

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let f = match name.split_once("::CallEFun_") {
            Some(("gVisualDesc", m)) => visual_method(m),
            Some(("gAppliedAppearanceKey", m)) => key_method(m),
            _ => None,
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

fn visual_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "Clear" => |vm, c| edit_visual(vm, c, |a, _| a.clear()),
        "Empty" => |vm, c| {
            let this = Call::new(c).this();
            match vm.native(this) {
                Some(NativeObject::VisualDesc(a)) => {
                    let empty = a.is_empty();
                    Call::new(c).ret(2, u32::from(empty), INT_RESULT);
                    InterpStatus::Success
                }
                _ => vm.bad_this(this),
            }
        },
        "SetScale" => |vm, c| {
            let scale = match vm.native(word(c, 2)) {
                Some(NativeObject::Vector(v)) => *v,
                _ => return vm.bad_this(word(c, 2)),
            };
            edit_visual(vm, c, |a, _| a.scale = scale)
        },
        "GetScale" => |vm, c| {
            let this = Call::new(c).this();
            let scale = match vm.native(this) {
                Some(NativeObject::VisualDesc(a)) => a.scale_vector(),
                _ => return vm.bad_this(this),
            };
            let vector = vm
                .refs
                .create_native(ntype::VECTOR, NativeObject::Vector(scale));
            Call::new(c).ret(2, vector, tag::REF);
            InterpStatus::Success
        },
        "AddAppearanceKey" => |vm, c| {
            let (pg, table) = (word(c, 2), word(c, 3));
            let applied = match vm.native(word(c, 4)) {
                Some(NativeObject::AppliedKey(k)) => *k,
                _ => return vm.bad_this(word(c, 4)),
            };
            let parent = parent_of(vm, Call::new(c).this());
            let mut added = false;
            let status = edit_visual(vm, c, |a, _| {
                added = a.add_appearance_key(
                    pg,
                    table,
                    &[(applied.key, applied.scalar)],
                    parent.as_deref(),
                );
            });
            Call::new(c).ret(5, u32::from(added), INT_RESULT);
            status
        },
        "RemoveAppearanceKey" => |vm, c| {
            let (pg, table) = (word(c, 2), word(c, 3));
            let applied = match vm.native(word(c, 4)) {
                Some(NativeObject::AppliedKey(k)) => *k,
                _ => return vm.bad_this(word(c, 4)),
            };
            let parent = parent_of(vm, Call::new(c).this());
            let mut removed = false;
            let status = edit_visual(vm, c, |a, _| {
                removed = a.remove_appearance_key(pg, table, applied.key, parent.as_deref());
            });
            Call::new(c).ret(5, u32::from(removed), INT_RESULT);
            status
        },
        "AddAppInfoHashToPartGroup" => |vm, c| {
            let (pg, hash) = (word(c, 2), word(c, 3));
            let infos = match vm.native(hash) {
                Some(NativeObject::Hash(h)) => app_infos_of(vm, h),
                _ => return vm.bad_this(hash),
            };
            let parent = parent_of(vm, Call::new(c).this());
            let mut added = false;
            let status = edit_visual(vm, c, |a, _| {
                added = a.add_app_info(pg, &infos, parent.as_deref());
            });
            Call::new(c).ret(4, u32::from(added), INT_RESULT);
            status
        },
        "InqAppInfoHashForPartGroup" => |vm, c| {
            let (pg, hash) = (word(c, 2), word(c, 3));
            let this = Call::new(c).this();
            let infos = match vm.native(this) {
                Some(NativeObject::VisualDesc(a)) => a.app_info_of(pg),
                _ => return vm.bad_this(this),
            };
            let found = infos.is_some();
            if let Some(infos) = infos {
                // `HashTable::operator=`: the out hash takes the group's tables.
                let refs: Vec<(u32, u32)> = infos
                    .into_iter()
                    .map(|(table, keys)| {
                        let info = vm.refs.create_native(
                            ntype::APPEARANCE_INFO,
                            NativeObject::AppearanceInfo(keys),
                        );
                        (table, info)
                    })
                    .collect();
                if let Some(NativeObject::Hash(h)) = vm.native_mut(hash) {
                    h.flush();
                    for (table, info) in refs {
                        h.set(
                            RefKey::Value(u64::from(table)),
                            u64::from(info),
                            &crate::native::PlainKeys,
                        );
                    }
                }
            }
            Call::new(c).ret(4, u32::from(found), INT_RESULT);
            InterpStatus::Success
        },
        "ApplyAdditiveVDesc" => |vm, c| {
            let Some(incoming) = incoming_of(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            let mut ok = false;
            let status = edit_visual(vm, c, |a, _| ok = a.apply_additive_appearance(&incoming));
            Call::new(c).ret(3, u32::from(ok), INT_RESULT);
            status
        },
        "ApplySubtractiveVDesc" => |vm, c| {
            let Some(incoming) = incoming_of(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            let parent = parent_of(vm, Call::new(c).this());
            let mut ok = false;
            let status = edit_visual(vm, c, |a, _| {
                ok = a.apply_subtractive(&incoming, parent.as_deref());
            });
            Call::new(c).ret(3, u32::from(ok), INT_RESULT);
            status
        },
        "ReplaceVisualDesc" => |vm, c| {
            let Some(incoming) = incoming_of(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            let mut ok = false;
            let status = edit_visual(vm, c, |a, _| ok = a.replace_with(&incoming));
            Call::new(c).ret(3, u32::from(ok), INT_RESULT);
            status
        },
        "GetVisualDescFromDB" => |vm, c| {
            let (did, out) = (word(c, 2), word(c, 3));
            // `VisualDesc::GetVisualDescFromDB`: a visual description record, copied into `out`.
            let record = if vm.data().db_type(did) == Some(DB_TYPE_VISUAL_DESC) {
                vm.data().visual_desc(did)
            } else {
                None
            };
            let found = record.is_some();
            if let Some(record) = record {
                match vm.native_mut(out) {
                    Some(NativeObject::VisualDesc(a)) => a.load_from(&record),
                    _ => return vm.bad_this(out),
                }
            }
            Call::new(c).ret(4, u32::from(found), INT_RESULT);
            InterpStatus::Success
        },
        "GetTotalIconDesc" => |vm, c| {
            let this = Call::new(c).this();
            let icon = out_native(vm, c, word(c, 2));
            let layers = match vm.native(this) {
                Some(NativeObject::VisualDesc(a)) => {
                    let parent = parent_of(vm, this);
                    a.total_icon_layers(parent.as_deref())
                }
                _ => return vm.bad_this(this),
            };
            let found = !layers.is_empty();
            match vm.native_mut(icon) {
                Some(NativeObject::IconDesc(out)) => *out = layers,
                _ => return vm.bad_this(icon),
            }
            Call::new(c).ret(3, u32::from(found), INT_RESULT);
            InterpStatus::Success
        },
        _ => return None,
    })
}

fn key_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "SetAppKey" => |vm, c| edit_key(vm, c, |k, v| k.key = v),
        "SetMod" => |vm, c| edit_key(vm, c, |k, v| k.scalar = f32::from_bits(v)),
        "GetAppKey" => |vm, c| get_key(vm, c, |k| k.key),
        "GetMod" => |vm, c| get_key(vm, c, |k| k.scalar.to_bits()),
        _ => return None,
    })
}

/// The parent description of `this` (read through the data source), when it names one that is
/// resident. A description without a parent has none.
fn parent_of(vm: &Vm, this: u32) -> Option<Arc<Appearance>> {
    let did = match vm.native(this)? {
        NativeObject::VisualDesc(a) => a.parent,
        _ => return None,
    };
    if did == 0 {
        return None;
    }
    vm.data().visual_desc(did)
}

/// A copy of the description `reference` names, for the merges that read the other one.
fn incoming_of(vm: &Vm, reference: u32) -> Option<Appearance> {
    match vm.native(reference)? {
        NativeObject::VisualDesc(a) => Some((**a).clone()),
        _ => None,
    }
}

/// The tables of an `AppearanceInfo` hash: each key (a table DID) with its applied keys.
fn app_infos_of(vm: &Vm, hash: &crate::native::ScriptHash) -> Vec<(u32, Vec<(u32, f32)>)> {
    hash.nodes()
        .filter_map(|node| {
            let RefKey::Value(table) = node.key else {
                return None;
            };
            let keys = match vm.native(node.value as u32)? {
                NativeObject::AppearanceInfo(keys) => keys.clone(),
                _ => return None,
            };
            Some((table as u32, keys))
        })
        .collect()
}

/// Edits the description `this` names.
fn edit_visual(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut Appearance, &Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::VisualDesc(a)) => {
            f(a, ctx);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// Sets a word of the applied key `this` names, from its first argument.
fn edit_key(vm: &mut Vm, ctx: &mut Context, f: fn(&mut AppliedAppearanceKey, u32)) -> InterpStatus {
    let this = Call::new(ctx).this();
    let value = word(ctx, 2);
    match vm.native_mut(this) {
        Some(NativeObject::AppliedKey(k)) => {
            f(k, value);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// Returns a word of the applied key `this` names in the return slot (2).
fn get_key(vm: &mut Vm, ctx: &mut Context, f: fn(&AppliedAppearanceKey) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native(this) {
        Some(NativeObject::AppliedKey(k)) => {
            let value = f(k);
            Call::new(ctx).ret(2, value, tag::BOOL);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

#[cfg(test)]
#[path = "visual_desc_tests.rs"]
mod tests;
