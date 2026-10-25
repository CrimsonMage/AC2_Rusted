//! `gVisualDesc` and `gAppliedAppearanceKey` (`portal/shared/src/interp/glue/gvisualdesc.cpp.c`,
//! `gappliedappearancekey.cpp.c`): the efuns a `CharacterGenSystem` script edits its appearance
//! with. The description is ac2-weenie's [`Appearance`]; the efuns that need a parent's DAT
//! description (`MakeLocalPGDDFromParent`) answer as the empty start does.

use ac2_weenie::Appearance;

use super::{word, Call, INT_RESULT};
use crate::native::{AppliedAppearanceKey, NativeObject};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};

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
        "Clear" => |vm, c| edit_visual(vm, c, |a, _| *a = Appearance::default()),
        "SetScale" => |vm, c| {
            let scale = match vm.native(word(c, 2)) {
                Some(NativeObject::Vector(v)) => *v,
                _ => return vm.bad_this(word(c, 2)),
            };
            edit_visual(vm, c, |a, _| a.scale = scale)
        },
        "AddAppearanceKey" => |vm, c| {
            let (pg, table) = (word(c, 2), word(c, 3));
            let applied = match vm.native(word(c, 4)) {
                Some(NativeObject::AppliedKey(k)) => *k,
                _ => return vm.bad_this(word(c, 4)),
            };
            let mut added = false;
            let status = edit_visual(vm, c, |a, _| {
                added = a.add_appearance_key(pg, table, &[(applied.key, applied.scalar)]);
            });
            Call::new(c).ret(5, u32::from(added), INT_RESULT);
            status
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
