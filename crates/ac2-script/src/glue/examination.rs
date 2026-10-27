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

use ac2_net::protocol::{self as net, ExaminationProfile, ExaminationRequest, SelectionInfo};

use super::{word, word64, Call, INT_RESULT};
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use crate::wire::string_info_to_net;

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
    register_profile(efuns);
}

/// `gExaminationProfile` (native 0x99, efuns 0x990001-0x990019). The ids are the ones the admin
/// dispatcher (`gmCEFunDispatcher`) and the Legions dispatcher both bind. The arguments follow
/// the admin glue's frames: the value in slot 2 (a 64-bit value takes slots 2-3), then the
/// appearance and the order.
const PROFILE_BASE: u32 = 0x0099_0000;
const ADD_EFFECT_ID: u32 = 0x19;

fn register_profile(efuns: &mut EFuns) {
    efuns.register(PROFILE_BASE | 0x01, |vm, c| compare_profile(vm, c, true));
    efuns.register(PROFILE_BASE | 0x02, |vm, c| compare_profile(vm, c, false));
    efuns.register(PROFILE_BASE | 0x03, set_request);
    efuns.register(PROFILE_BASE | 0x04, get_request);
    efuns.register(PROFILE_BASE | 0x05, add_bool);
    efuns.register(PROFILE_BASE | 0x06, add_countdown);
    efuns.register(PROFILE_BASE | 0x07, add_float);
    efuns.register(PROFILE_BASE | 0x08, add_int);
    efuns.register(PROFILE_BASE | 0x09, add_string_info);
    efuns.register(PROFILE_BASE | 0x0A, add_simple_string_info);
    efuns.register(PROFILE_BASE | 0x11, add_image);
    efuns.register(PROFILE_BASE | 0x12, add_tab);
    efuns.register(PROFILE_BASE | 0x13, add_break);
    efuns.register(PROFILE_BASE | 0x14, add_icon_desc);
    efuns.register(PROFILE_BASE | 0x15, add_long_int);
    efuns.register(PROFILE_BASE | 0x17, add_tabs);
    efuns.register(PROFILE_BASE | ADD_EFFECT_ID, add_effect_id);
}

fn profile(vm: &Vm, reference: u32) -> Option<&ExaminationProfile> {
    match vm.native(reference)? {
        NativeObject::Wire(w) => match &**w {
            net::NativeObject::ExaminationProfile(p) => Some(p),
            _ => None,
        },
        _ => None,
    }
}

fn profile_mut(vm: &mut Vm, reference: u32) -> Option<&mut ExaminationProfile> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            net::NativeObject::ExaminationProfile(p) => Some(p),
            _ => None,
        },
        _ => None,
    }
}

/// Runs `f` on the profile `this` names; a missing or foreign `this` crashes as retail's
/// `index != Undef_ReferenceID` assert does.
fn with_profile(vm: &mut Vm, this: u32, f: impl FnOnce(&mut ExaminationProfile)) -> InterpStatus {
    match profile_mut(vm, this) {
        Some(p) => {
            f(p);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `ExaminationProfile::operator==` (and its negation); a null `other` crashes.
fn compare_profile(vm: &mut Vm, ctx: &mut Context, eq: bool) -> InterpStatus {
    let (this, other) = (Call::new(ctx).this(), word(ctx, 2));
    let (Some(a), Some(b)) = (profile(vm, this), profile(vm, other)) else {
        return vm.bad_this(this);
    };
    let same = this == other || a == b;
    Call::new(ctx).ret(3, u32::from(same == eq), INT_RESULT);
    InterpStatus::Success
}

/// `CallEFun_SetExaminationRequest`: `ExaminationRequest::operator=` into the profile's request.
fn set_request(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let other = word(ctx, 2);
    let Some(request) = request(vm, other).copied() else {
        return vm.bad_this(other);
    };
    with_profile(vm, this, |p| p.request = request)
}

/// `CallEFun_GetExaminationRequest`: a copy of the profile's request, as a new native.
fn get_request(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(request) = profile(vm, this).map(|p| p.request) else {
        return vm.bad_this(this);
    };
    let native = NativeObject::Wire(Box::new(net::NativeObject::ExaminationRequest(request)));
    let r = vm.refs.create_native(ntype::EXAMINATION_REQUEST, native);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `AddBool(value, appearance, order)`.
fn add_bool(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (value, appearance, order) = (word(ctx, 2), word(ctx, 3), word(ctx, 4));
    with_profile(vm, this, |p| p.add_bool(value, appearance, order))
}

/// `AddCountdown(seconds, appearance, order)`: the `double` in slots 2-3.
fn add_countdown(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let seconds = f64::from_bits(word64(ctx, 3));
    let (appearance, order) = (word(ctx, 4), word(ctx, 5));
    with_profile(vm, this, |p| p.add_countdown(seconds, appearance, order))
}

/// `AddFloat(value, appearance, order)`: the `float`'s bits.
fn add_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let value = f32::from_bits(word(ctx, 2));
    let (appearance, order) = (word(ctx, 3), word(ctx, 4));
    with_profile(vm, this, |p| p.add_float(value, appearance, order))
}

/// `AddInt(value, appearance, order)`.
fn add_int(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let value = word(ctx, 2) as i32;
    let (appearance, order) = (word(ctx, 3), word(ctx, 4));
    with_profile(vm, this, |p| p.add_int(value, appearance, order))
}

/// `AddLongInt(value, appearance, order)`: the 64-bit value in slots 2-3.
fn add_long_int(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let value = word64(ctx, 3) as i64;
    let (appearance, order) = (word(ctx, 4), word(ctx, 5));
    with_profile(vm, this, |p| p.add_long_int(value, appearance, order))
}

/// `AddStringInfo(info, appearance, order)`: a copy of the script's StringInfo.
fn add_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let info_ref = word(ctx, 2);
    let Some(info) = string_info(vm, info_ref) else {
        return vm.bad_this(info_ref);
    };
    let (appearance, order) = (word(ctx, 3), word(ctx, 4));
    with_profile(vm, this, |p| p.add_string_info(info, appearance, order))
}

/// `AddSimpleStringInfo(string id, table, appearance, order)`.
fn add_simple_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (string_id, table_id) = (word(ctx, 2), word(ctx, 3));
    let (appearance, order) = (word(ctx, 4), word(ctx, 5));
    with_profile(vm, this, |p| {
        p.add_simple_string_info(string_id, table_id, appearance, order)
    })
}

/// `AddImage(did, order)`.
fn add_image(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (did, order) = (word(ctx, 2), word(ctx, 3));
    with_profile(vm, this, |p| p.add_image(did, order))
}

/// `AddTab(order)`.
fn add_tab(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let order = word(ctx, 2);
    with_profile(vm, this, |p| p.add_tab(order))
}

/// `AddBreak(order)`.
fn add_break(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let order = word(ctx, 2);
    with_profile(vm, this, |p| p.add_break(order))
}

/// `AddIconDesc(icon, order)`: the `IconDesc`'s layers (layer, image, shift colour).
fn add_icon_desc(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let icon_ref = word(ctx, 2);
    let layers = match vm.native(icon_ref) {
        Some(NativeObject::IconDesc(layers)) => layers
            .iter()
            .map(|&(layer, image, color)| net::IconLayer {
                layer,
                image,
                color,
            })
            .collect(),
        _ => return vm.bad_this(icon_ref),
    };
    let order = word(ctx, 3);
    with_profile(vm, this, |p| p.add_icon_desc(layers, order))
}

/// `AddTabs(number, order)`.
fn add_tabs(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (number, order) = (word(ctx, 2), word(ctx, 3));
    with_profile(vm, this, |p| p.add_tabs(number, order))
}

/// `AddEffectID(eid, depth, order)`: arguments in slots 2-4 (`ExaminationProfile::AddEffectID`'s
/// `value`, `depth`, `order`).
fn add_effect_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (eid, depth, order) = (word(ctx, 2), word(ctx, 3), word(ctx, 4));
    with_profile(vm, this, |p| p.add_effect_id(eid, depth, order))
}

/// The wire form of a script StringInfo reference.
fn string_info(vm: &Vm, reference: u32) -> Option<net::StringInfo> {
    match vm.native(reference)? {
        NativeObject::StringInfo(info) => Some(string_info_to_net(info)),
        _ => None,
    }
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
