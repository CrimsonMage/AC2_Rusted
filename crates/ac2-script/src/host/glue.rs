//! Host glue: `gPackBuffer` (`PackBuffer`), `gCWInterface::SendServerEvt[Encrypt]` and
//! `ggmCObjMaint`'s player lookups.

use std::collections::VecDeque;

use ac2_net::protocol::PackValue;

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
