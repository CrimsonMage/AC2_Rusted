//! `gEffectUINode` (`geffectuinode.cpp`): the status bar's effect node `CEffectSystem` 0x7 fills
//! for `gmCWInterface::InqEffectList`, over ac2-net's `EffectUiNode` (`EffectUINode::StreamPack`:
//! eid, flags, name, description, icon, duration, time cast).
//!
//! Layout (`EffectUINode`): vtable 0x0, `m_eid` 0x4, `m_flags` 0x8, `m_enabled` 0xC, `m_name`
//! 0x10, `m_desc` 0x14, `m_icon` 0x18, `m_duration` 0x20, `m_timeCast` 0x28.

use ac2_net::protocol::{EffectUiNode, NativeObject as Wire};

use super::{word, word64, Call};
use crate::native::NativeObject;
use crate::vm::{Context, EFunFn, EFuns, InterpStatus, Vm};

/// `EffectUINode::SetEnabled` / `SetToggled` / `SetPulsed` set these bits in `m_flags`.
const FLAG_ENABLED: u32 = 1;
const FLAG_TOGGLED: u32 = 2;
const FLAG_PULSED: u32 = 4;

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        if let Some(("gEffectUINode", m)) = name.split_once("::CallEFun_") {
            if let Some(f) = method(m) {
                efuns.register(id, f);
            }
        }
    }
}

fn node_mut(vm: &mut Vm, r: u32) -> Option<&mut EffectUiNode> {
    match vm.native_mut(r)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::EffectUiNode(n) => Some(&mut **n),
            _ => None,
        },
        _ => None,
    }
}

fn node(vm: &Vm, r: u32) -> Option<EffectUiNode> {
    match vm.native(r)? {
        NativeObject::Wire(w) => match &**w {
            Wire::EffectUiNode(n) => Some((**n).clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Runs `f` on `this`'s node; the crash for a null or foreign `this` otherwise.
fn edit(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut EffectUiNode, &Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match node_mut(vm, this) {
        Some(n) => {
            f(n, ctx);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `SetName` / `SetDescription`: the argument is a `wpstring` whose characters are copied in
/// (`EffectUINode::SetName` keeps the buffer; the wire form keeps the text).
fn set_text(vm: &mut Vm, ctx: &mut Context, desc: bool) -> InterpStatus {
    let source = word(ctx, 2);
    let Some(text) = vm.wstring(source).map(String::from_utf16_lossy) else {
        return vm.bad_this(source);
    };
    edit(vm, ctx, |n, _| {
        if desc {
            n.desc = text;
        } else {
            n.name = text;
        }
    })
}

/// `SetDurationInfo(timeCast, duration)`: two doubles, each two words (high word first), and
/// `EffectUINode` stores them at 0x28 and 0x20.
fn set_duration_info(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let time_cast = f64::from_bits(word64(ctx, 3));
    let duration = f64::from_bits(word64(ctx, 5));
    edit(vm, ctx, |n, _| {
        n.time_cast = time_cast;
        n.duration = duration;
    })
}

/// `SetEnabled` / `SetDisabled` / `SetToggled` / `SetPulsed`: one bit of `m_flags`.
fn set_flag(vm: &mut Vm, ctx: &mut Context, bit: u32, on: bool) -> InterpStatus {
    edit(vm, ctx, |n, _| {
        if on {
            n.flags |= bit;
        } else {
            n.flags &= !bit;
        }
    })
}

fn method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "SetEID" => |vm, c| edit(vm, c, |n, c| n.eid = word(c, 2)),
        "SetIcon" => |vm, c| edit(vm, c, |n, c| n.icon = word(c, 2)),
        "SetName" => |vm, c| set_text(vm, c, false),
        "SetDescription" => |vm, c| set_text(vm, c, true),
        "SetEnabled" => |vm, c| set_flag(vm, c, FLAG_ENABLED, true),
        "SetDisabled" => |vm, c| set_flag(vm, c, FLAG_ENABLED, false),
        "SetToggled" => |vm, c| set_flag(vm, c, FLAG_TOGGLED, true),
        "SetPulsed" => |vm, c| set_flag(vm, c, FLAG_PULSED, true),
        "SetDurationInfo" => set_duration_info,
        // `EffectUINode::operator=` (the flags, names, icon and times; `m_enabled` is not
        // copied, nor is it read by the wire form).
        "Set" => |vm, c| {
            let other = word(c, 2);
            let Some(rhs) = node(vm, other) else {
                return vm.bad_this(other);
            };
            edit(vm, c, |n, _| *n = rhs)
        },
        _ => return None,
    })
}
