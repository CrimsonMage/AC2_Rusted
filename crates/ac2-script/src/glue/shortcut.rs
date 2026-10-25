//! `gShortcutInfo` (0xAC0001-0xAC000A): a `ShortcutInfo`'s fields (`_type` +4, `_data_str`
//! +0xC, `_data_iid` +0x10, `_data_enum` +0x18, `_data_dataid` +0x1C). Every wrapper crashes
//! on an undefined `this`; the id comes high word first.

use super::{word, word64, Call, INT_RESULT};
use crate::native::{NativeObject, Shortcut};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x00AC_0001, |vm, c| get(vm, c, |s| s.kind));
    efuns.register(0x00AC_0002, get_iid);
    efuns.register(0x00AC_0003, |vm, c| get(vm, c, |s| s.data_enum));
    efuns.register(0x00AC_0004, get_text);
    efuns.register(0x00AC_0005, |vm, c| {
        let v = word(c, 2);
        with(vm, c, |s| s.kind = v)
    });
    efuns.register(0x00AC_0006, |vm, c| {
        let v = word64(c, 3);
        with(vm, c, |s| s.iid = v)
    });
    efuns.register(0x00AC_0007, |vm, c| {
        let v = word(c, 2);
        with(vm, c, |s| s.data_enum = v)
    });
    efuns.register(0x00AC_0008, |vm, c| {
        let text = vm.wstring(word(c, 2)).unwrap_or(&[]).to_vec();
        with(vm, c, |s| s.text = text)
    });
    efuns.register(0x00AC_0009, |vm, c| get(vm, c, |s| s.data_id));
    efuns.register(0x00AC_000A, |vm, c| {
        let v = word(c, 2);
        with(vm, c, |s| s.data_id = v)
    });
}

fn shortcut(vm: &Vm, reference: u32) -> Option<&Shortcut> {
    match vm.native(reference)? {
        NativeObject::Shortcut(s) => Some(s),
        _ => None,
    }
}

fn with(vm: &mut Vm, ctx: &mut Context, f: impl FnOnce(&mut Shortcut)) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::Shortcut(s)) => {
            f(s);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

fn get(vm: &mut Vm, ctx: &mut Context, f: fn(&Shortcut) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = shortcut(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, v, INT_RESULT);
    InterpStatus::Success
}

/// `GetDataIID`: high word in slot 2, low word in slot 3 (tag 0x84).
fn get_iid(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(iid) = shortcut(vm, this).map(|s| s.iid) else {
        return vm.bad_this(this);
    };
    let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(3, iid as u32, tag::BOOL | tag::LONG);
    call.ret(2, (iid >> 32) as u32, high_tag);
    InterpStatus::Success
}

/// `GetDataStr`: a copy as a new `wpstring`.
fn get_text(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(text) = shortcut(vm, this).map(|s| s.text.clone()) else {
        return vm.bad_this(this);
    };
    let r = vm.new_wstring(text);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}
