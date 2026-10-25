//! `gExaminationRequest` and `gSelectionInfo`.
//!
//! `gExaminationRequest` (efuns 0x9A0001-0x9A0014 as the Legions efun switch binds them):
//! the fields of an `ExaminationRequest` (Legions layout: +4 type, +8 `_data_enum`, +0xC
//! `_data_DataID`, +0x10 `_admin`, +0x18 `_data_iid`, and the Legions words +0x20, +0x24,
//! +0x28 with their own getters and setters, 0x9A000F-0x9A0014). Every wrapper crashes on an
//! undefined `this`; results are tagged 4, the 64-bit id 0x84 on its low word.
//!
//! `gSelectionInfo` (0xAB0001-0xAB0010): `SelectionInfo`'s seven words (`m_type`,
//! `m_curHealth`, `m_PKDamage`, `m_maxHealth`, `m_curVigor`, `m_PKVigorloss`, `m_maxVigor`);
//! 0xAB0008 is `SetMaxHealth` (unnamed in the admin efun table, bound in Legions).

use ac2_net::protocol::{self as net, ExaminationRequest, SelectionInfo};

use super::{word, word64, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

const BASE: u32 = 0x009A_0000;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(BASE | 0x01, |vm, c| compare(vm, c, true));
    efuns.register(BASE | 0x02, |vm, c| compare(vm, c, false));
    efuns.register(BASE | 0x03, |vm, c| get(vm, c, |r| r.kind));
    efuns.register(BASE | 0x04, |vm, c| get(vm, c, |r| r.data_enum));
    efuns.register(BASE | 0x05, get_iid);
    efuns.register(BASE | 0x06, |vm, c| set(vm, c, |r, v| r.kind = v));
    efuns.register(BASE | 0x07, |vm, c| set(vm, c, |r, v| r.data_enum = v));
    efuns.register(BASE | 0x08, set_iid);
    efuns.register(BASE | 0x09, |vm, c| get(vm, c, |r| r.admin));
    efuns.register(BASE | 0x0A, |vm, c| set(vm, c, |r, v| r.admin = v));
    efuns.register(BASE | 0x0D, |vm, c| get(vm, c, |r| r.data_id));
    efuns.register(BASE | 0x0E, |vm, c| set(vm, c, |r, v| r.data_id = v));
    efuns.register(BASE | 0x0F, |vm, c| set(vm, c, |r, v| r.extra[0] = v));
    efuns.register(BASE | 0x10, |vm, c| get(vm, c, |r| r.extra[0]));
    efuns.register(BASE | 0x11, |vm, c| set(vm, c, |r, v| r.extra[1] = v));
    efuns.register(BASE | 0x12, |vm, c| get(vm, c, |r| r.extra[1]));
    efuns.register(BASE | 0x13, |vm, c| set(vm, c, |r, v| r.extra[2] = v));
    efuns.register(BASE | 0x14, |vm, c| get(vm, c, |r| r.extra[2]));
    register_selection_info(efuns);
}

const SELECTION: u32 = 0x00AB_0000;

/// The `SelectionInfo` field each get/set efun pair reads and writes.
const SELECTION_FIELDS: [(u32, u32, fn(&mut SelectionInfo) -> &mut u32); 7] = [
    (0x03, 0x04, |s| &mut s.kind),
    (0x05, 0x06, |s| &mut s.cur_health),
    (0x07, 0x08, |s| &mut s.max_health),
    (0x09, 0x0A, |s| &mut s.cur_vigor),
    (0x0B, 0x0C, |s| &mut s.max_vigor),
    (0x0D, 0x0E, |s| &mut s.pk_damage),
    (0x0F, 0x10, |s| &mut s.pk_vigor_loss),
];

fn register_selection_info(efuns: &mut EFuns) {
    efuns.register(SELECTION | 0x01, |vm, c| compare_selection(vm, c, true));
    efuns.register(SELECTION | 0x02, |vm, c| compare_selection(vm, c, false));
    for (get, set, _) in SELECTION_FIELDS {
        efuns.register(SELECTION | get, selection_field);
        efuns.register(SELECTION | set, selection_field);
    }
}

fn selection(vm: &mut Vm, reference: u32) -> Option<&mut SelectionInfo> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            net::NativeObject::SelectionInfo(s) => Some(s),
            _ => None,
        },
        _ => None,
    }
}

/// A `SelectionInfo` getter (result in slot 2) or setter (value in slot 2).
fn selection_field(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = vm.current_efun() & 0xFFFF;
    let (get, _, field) = SELECTION_FIELDS
        .iter()
        .copied()
        .find(|&(g, s, _)| g == id || s == id)
        .expect("registered SelectionInfo efun");
    let this = Call::new(ctx).this();
    let value = word(ctx, 2);
    let Some(s) = selection(vm, this) else {
        return vm.bad_this(this);
    };
    if id == get {
        let v = *field(s);
        Call::new(ctx).ret(2, v, INT_RESULT);
    } else {
        *field(s) = value;
    }
    InterpStatus::Success
}

/// `SelectionInfo::operator==` (every word) and its negation.
fn compare_selection(vm: &mut Vm, ctx: &mut Context, eq: bool) -> InterpStatus {
    let (this, other) = (Call::new(ctx).this(), word(ctx, 2));
    let a = selection(vm, this).copied();
    let b = selection(vm, other).copied();
    let (Some(a), Some(b)) = (a, b) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(3, u32::from((a == b) == eq), INT_RESULT);
    InterpStatus::Success
}

fn request(vm: &Vm, reference: u32) -> Option<&ExaminationRequest> {
    match vm.native(reference)? {
        NativeObject::Wire(w) => match &**w {
            net::NativeObject::ExaminationRequest(r) => Some(r),
            _ => None,
        },
        _ => None,
    }
}

fn request_mut(vm: &mut Vm, reference: u32) -> Option<&mut ExaminationRequest> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            net::NativeObject::ExaminationRequest(r) => Some(r),
            _ => None,
        },
        _ => None,
    }
}

fn get(vm: &mut Vm, ctx: &mut Context, f: fn(&ExaminationRequest) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(value) = request(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, value, INT_RESULT);
    InterpStatus::Success
}

fn set(vm: &mut Vm, ctx: &mut Context, f: fn(&mut ExaminationRequest, u32)) -> InterpStatus {
    let this = Call::new(ctx).this();
    let value = word(ctx, 2);
    match request_mut(vm, this) {
        Some(r) => {
            f(r, value);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `GetDataIID`: high word in slot 2, low word in slot 3 (tag 0x84).
fn get_iid(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(iid) = request(vm, this).map(|r| r.iid) else {
        return vm.bad_this(this);
    };
    let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(3, iid as u32, tag::BOOL | tag::LONG);
    call.ret(2, (iid >> 32) as u32, high_tag);
    InterpStatus::Success
}

/// `SetDataIID(iid)` (high word first).
fn set_iid(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let iid = word64(ctx, 3);
    match request_mut(vm, this) {
        Some(r) => {
            r.iid = iid;
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `ExaminationRequest::operator==` (Legions 0x005C90C0: every field, the extra words too) and
/// its negation; a null `other` crashes as retail's dereference would.
fn compare(vm: &mut Vm, ctx: &mut Context, eq: bool) -> InterpStatus {
    let (this, other) = (Call::new(ctx).this(), word(ctx, 2));
    let (Some(a), Some(b)) = (request(vm, this), request(vm, other)) else {
        return vm.bad_this(this);
    };
    let same = this == other || a == b;
    Call::new(ctx).ret(3, u32::from(same == eq), INT_RESULT);
    InterpStatus::Success
}
