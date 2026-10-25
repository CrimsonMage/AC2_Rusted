//! Host glue: `gPackBuffer` (`PackBuffer`), `gCWInterface::SendServerEvt[Encrypt]` and the
//! `gCM_*::SendNotice_*` notices the UI listens to.

use std::collections::VecDeque;

use ac2_net::protocol::{self as net, PackValue};
use ac2_weenie::GameplayNotice;

use super::ServerCall;
use crate::glue::lists::{write_out, Kind};
use crate::glue::{word, word64, Call};
use crate::native::{NativeObject, PackItem};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use crate::wire;

const INT_RESULT: u8 = tag::BOOL;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0050_0001, pack_package);
    efuns.register(0x0050_0002, |vm, c| {
        pack(vm, c, |c| PackItem::Int(word(c, 2)))
    });
    efuns.register(0x0050_0003, |vm, c| {
        pack(vm, c, |c| PackItem::LongInt(word64(c, 3)))
    });
    efuns.register(0x0050_0004, |vm, c| {
        pack(vm, c, |c| PackItem::Float(word(c, 2)))
    });
    efuns.register(0x0050_0005, |vm, c| {
        pack(vm, c, |c| PackItem::LongFloat(word64(c, 3)))
    });
    efuns.register(0x0050_0006, |vm, c| unpack(vm, c, 5));
    efuns.register(0x0050_0007, |vm, c| unpack(vm, c, 1));
    efuns.register(0x0050_0008, |vm, c| unpack(vm, c, 2));
    efuns.register(0x0050_0009, |vm, c| unpack(vm, c, 3));
    efuns.register(0x0050_000A, |vm, c| unpack(vm, c, 4));
    efuns.register(0x007F_0036, |vm, c| send_server_evt(vm, c, false));
    efuns.register(0x007F_0037, |vm, c| send_server_evt(vm, c, true));
    efuns.register(0x0095_0001, get_player_id);
    efuns.register(0x0095_0002, get_player_weenie_object_reference);
    efuns.register(0x0085_0001, update_vital);
    efuns.register(0x0085_0003, selection_changed);
    efuns.register(0x0085_000B, update_selection_info);
    efuns.register(0x0085_0012, update_hero_status);
    efuns.register(0x0087_0001, display_string_info);
    efuns.register(0x0089_0001, |vm, c| examination(vm, c, false));
    efuns.register(0x0089_0002, |vm, c| examination(vm, c, true));
}

fn with_buffer<R>(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut VecDeque<PackItem>) -> R,
) -> Option<R> {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::PackBuffer(items)) => Some(f(items)),
        _ => None,
    }
}

/// `gPackBuffer::CallEFun_PackInt` / `PackLInt` / `PackFloat` / `PackLFloat`: appends a tagged
/// value (64-bit values high word first in the frame).
fn pack(vm: &mut Vm, ctx: &mut Context, item: fn(&Context) -> PackItem) -> InterpStatus {
    let value = item(ctx);
    match with_buffer(vm, ctx, |b| b.push_back(value)) {
        Some(()) => InterpStatus::Success,
        None => vm.bad_this(Call::new(ctx).this()),
    }
}

/// `PackBuffer::PackPackage(ref)`: the reference graph as it is now (`VM::PackReference`).
fn pack_package(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let reference = word(ctx, 2);
    let packed = vm.refs.pack(reference);
    match with_buffer(vm, ctx, |b| b.push_back(PackItem::Package(packed))) {
        Some(()) => InterpStatus::Success,
        None => vm.bad_this(Call::new(ctx).this()),
    }
}

/// `PackBuffer::UnPack{Int,LInt,Float,LFloat,Package}(out success)`: takes the front value
/// when its tag matches; the success out is tagged 4 and the result word(s) follow it.
fn unpack(vm: &mut Vm, ctx: &mut Context, want: u32) -> InterpStatus {
    let out = word(ctx, 2);
    let taken = with_buffer(vm, ctx, |b| {
        let matches = matches!(
            (b.front(), want),
            (Some(PackItem::Int(_)), 1)
                | (Some(PackItem::LongInt(_)), 2)
                | (Some(PackItem::Float(_)), 3)
                | (Some(PackItem::LongFloat(_)), 4)
                | (Some(PackItem::Package(_)), 5)
        );
        if matches {
            b.pop_front()
        } else {
            None
        }
    });
    let Some(taken) = taken else {
        return vm.bad_this(Call::new(ctx).this());
    };
    write_out(vm, ctx, out, Some(u64::from(taken.is_some())), Kind::A);
    let (low, high, long, reference) = match taken {
        Some(PackItem::Int(v)) | Some(PackItem::Float(v)) => (v, 0, false, false),
        Some(PackItem::LongInt(v)) | Some(PackItem::LongFloat(v)) => {
            (v as u32, (v >> 32) as u32, true, false)
        }
        Some(PackItem::Package(packed)) => (
            vm.refs.unpack(&packed, None).unwrap_or(u32::MAX),
            0,
            false,
            true,
        ),
        None if want == 5 => (u32::MAX, 0, false, true),
        None => (0, 0, want == 2 || want == 4, false),
    };
    if long {
        let high_tag = ctx.arg_tag(3).unwrap_or(tag::UNINIT);
        let mut call = Call::new(ctx);
        call.ret(4, low, tag::BOOL | tag::LONG);
        call.ret(3, high, high_tag);
    } else {
        Call::new(ctx).ret(3, low, if reference { tag::REF } else { INT_RESULT });
    }
    InterpStatus::Success
}

/// The wire form of a buffer's values; `None` names a native the wire cannot carry.
fn to_wire(items: &VecDeque<PackItem>) -> Result<Vec<PackValue>, u32> {
    items
        .iter()
        .map(|item| {
            Ok(match item {
                PackItem::Int(v) => PackValue::Int(*v),
                PackItem::LongInt(v) => PackValue::LongInt(*v),
                PackItem::Float(v) => PackValue::Float(f32::from_bits(*v)),
                PackItem::LongFloat(v) => PackValue::LongFloat(f64::from_bits(*v)),
                PackItem::Package(p) => PackValue::Reference(wire::to_net(p).map_err(|e| e.0)?),
            })
        })
        .collect()
}

/// `gCWInterface::CallEFun_SendServerEvt[Encrypt](function, buffer)`: queues the call for the
/// session (`ECM_Interp::SendEvt_InterpSEvent`); returns 1.
fn send_server_evt(vm: &mut Vm, ctx: &mut Context, encrypt: bool) -> InterpStatus {
    let (function, buffer) = (word(ctx, 2), word(ctx, 3));
    let args = match vm.native(buffer) {
        Some(NativeObject::PackBuffer(items)) => to_wire(items),
        _ => return vm.bad_this(buffer),
    };
    match args {
        Ok(args) => vm.outbox.server_calls.push(ServerCall {
            function,
            args,
            encrypt,
        }),
        Err(t) => vm.outbox.dropped.push(format!(
            "server call {function:#010X}: native type {t:#X} has no wire form"
        )),
    }
    Call::new(ctx).ret(4, 1, INT_RESULT);
    InterpStatus::Success
}

/// The weenie object a notice is raised on (`obj`, a script instance its process owns).
fn owner(vm: &Vm, reference: u32) -> u64 {
    vm.owners.get(&reference).copied().unwrap_or(0)
}

fn wire_native(vm: &Vm, reference: u32) -> Option<net::NativeObject> {
    let object = vm.refs.heap(reference).ok()?;
    wire::native_to_net(object.native_type, object.native.as_ref()?).ok()
}

fn notice(
    vm: &mut Vm,
    ctx: &mut Context,
    ret: usize,
    n: Option<GameplayNotice>,
    what: &str,
) -> InterpStatus {
    match n {
        Some(n) => vm.outbox.notices.push(n),
        None => vm
            .outbox
            .dropped
            .push(format!("{what}: the native has no wire form")),
    }
    Call::new(ctx).ret(ret, 1, INT_RESULT);
    InterpStatus::Success
}

/// `CM_Character::SendNotice_SelectionChanged(obj, newSelection)`.
fn selection_changed(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = word64(ctx, 4);
    notice(
        vm,
        ctx,
        5,
        Some(GameplayNotice::SelectionChanged(id)),
        "SelectionChanged",
    )
}

/// `CM_Character::SendNotice_UpdateSelectionInfo(obj, info)`: `id` is the weenie the notice
/// is raised on.
fn update_selection_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (obj, info) = (word(ctx, 2), word(ctx, 3));
    let n = match wire_native(vm, info) {
        Some(net::NativeObject::SelectionInfo(info)) => Some(GameplayNotice::UpdateSelectionInfo {
            id: owner(vm, obj),
            info,
        }),
        _ => None,
    };
    notice(vm, ctx, 4, n, "UpdateSelectionInfo")
}

/// `CM_Communication::SendNotice_DisplayStringInfo(obj, type, msg)`.
fn display_string_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (kind, msg) = (word(ctx, 3), word(ctx, 4));
    let n = match wire_native(vm, msg) {
        Some(net::NativeObject::StringInfo(text)) => {
            Some(GameplayNotice::DisplayStringInfo { kind, text })
        }
        _ => None,
    };
    notice(vm, ctx, 5, n, "DisplayStringInfo")
}

/// `CM_Examination::SendNotice_ExaminationRequestChanged` / `UpdateExaminationProfile`.
fn examination(vm: &mut Vm, ctx: &mut Context, request: bool) -> InterpStatus {
    let n = match (wire_native(vm, word(ctx, 3)), request) {
        (Some(net::NativeObject::ExaminationRequest(r)), true) => {
            Some(GameplayNotice::ExaminationRequestChanged(r))
        }
        (Some(net::NativeObject::ExaminationProfile(p)), false) => {
            Some(GameplayNotice::UpdateExaminationProfile(p))
        }
        _ => None,
    };
    notice(vm, ctx, 4, n, "examination notice")
}

/// `CM_Character::SendNotice_UpdateVital(obj, vital, current, max)` (64-bit values high word
/// first).
fn update_vital(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (vital, current, max) = (word(ctx, 3), word64(ctx, 5), word64(ctx, 7));
    let n = GameplayNotice::UpdateVital {
        vital,
        current,
        max,
    };
    notice(vm, ctx, 8, Some(n), "UpdateVital")
}

/// `CM_Character::SendNotice_UpdateHeroStatus(obj, isHero, curFocus, maxFocus)`.
fn update_hero_status(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (hero, current, max) = (word(ctx, 3) != 0, word64(ctx, 5), word64(ctx, 7));
    let n = GameplayNotice::UpdateHeroStatus { hero, current, max };
    notice(vm, ctx, 8, Some(n), "UpdateHeroStatus")
}

/// `CObjectMaint::GetPlayerID`: 64-bit, low word in slot 3 (tag 0x84).
fn get_player_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = vm.player;
    let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(3, id as u32, tag::BOOL | tag::LONG);
    call.ret(2, (id >> 32) as u32, high_tag);
    InterpStatus::Success
}

/// `CObjectMaint::GetPlayerWeenieObjectReference`: the player's script instance.
fn get_player_weenie_object_reference(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let player = vm.player;
    let r = vm
        .owners
        .iter()
        .find(|(_, &id)| id == player && player != 0)
        .map_or(u32::MAX, |(&r, _)| r);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}
