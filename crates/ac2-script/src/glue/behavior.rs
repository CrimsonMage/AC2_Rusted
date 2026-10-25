//! `gBehaviorParams`: setters on a `BehaviorParams` native (`gBehaviorParams.cpp`). The flag
//! setters store 1; the others store their argument (word 2; 64-bit ids low word deeper).

use super::{Call, INT_RESULT};
use crate::native::{BehaviorParams, NativeObject, PlainKeys, RefKey, ScriptHash};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0017_0001, |vm, ctx| set(vm, ctx, |p, w| p.behavior = w.0));
    efuns.register(0x0017_0002, get_behavior_id);
    efuns.register(0x0017_0003, |vm, ctx| {
        set(vm, ctx, |p, w| p.time_scale = f32::from_bits(w.0))
    });
    efuns.register(0x0017_0004, |vm, ctx| set(vm, ctx, |p, w| p.mode = w.0));
    efuns.register(0x0017_0005, |vm, ctx| {
        set(vm, ctx, |p, w| p.hold_cycles = w.0)
    });
    efuns.register(0x0017_0006, |vm, ctx| {
        set(vm, ctx, |p, _| p.moveto_cancels = 1)
    });
    efuns.register(0x0017_0007, |vm, ctx| {
        set(vm, ctx, |p, _| p.early_callback = 1)
    });
    efuns.register(0x0017_0008, |vm, ctx| {
        set(vm, ctx, |p, _| p.camera_parent = 1)
    });
    efuns.register(0x0017_0009, |vm, ctx| {
        set(vm, ctx, |p, _| p.camera_target_flag = 1)
    });
    efuns.register(0x0017_000A, |vm, ctx| {
        set(vm, ctx, |p, _| p.camera_hold = 1)
    });
    efuns.register(0x0017_000B, |vm, ctx| {
        set(vm, ctx, |p, _| p.camera_restore = 1)
    });
    efuns.register(0x0017_000C, |vm, ctx| {
        set(vm, ctx, |p, w| p.camera_target = w.2)
    });
    efuns.register(0x0017_000D, |vm, ctx| {
        set(vm, ctx, |p, w| p.camera_behavior = w.0)
    });
    efuns.register(0x0017_000E, |vm, ctx| {
        set(vm, ctx, |p, _| p.fade_children = 1)
    });
    efuns.register(0x0017_000F, |vm, ctx| {
        set(vm, ctx, |p, _| p.lock_actions = 1)
    });
    efuns.register(0x0017_0010, |vm, ctx| {
        set(vm, ctx, |p, w| p.send_event = w.0)
    });
    efuns.register(0x0017_0011, |vm, ctx| {
        set(vm, ctx, |p, w| p.fx_script = w.0)
    });
    efuns.register(0x0017_0012, |vm, ctx| set(vm, ctx, |p, w| p.target = w.2));
    efuns.register(0x0017_0013, |vm, ctx| {
        set(vm, ctx, |p, w| p.vdesc_to_clone = w.0)
    });
    efuns.register(0x0017_0014, set_targeted_impulse);
    efuns.register(0x0017_0015, |vm, ctx| {
        set(vm, ctx, |p, _| p.leave_idle_alone = 1)
    });
    efuns.register(0x0017_0016, |vm, ctx| set(vm, ctx, |p, _| p.propagate = 1));
    efuns.register(0x0017_0017, |vm, ctx| set(vm, ctx, |p, w| p.fx = w.0));
    efuns.register(0x0017_0018, |vm, ctx| {
        set(vm, ctx, |p, w| p.weenie_emote = w.0)
    });
    efuns.register(0x0017_0019, |vm, ctx| {
        set(vm, ctx, |p, w| p.cloned_apr = w.0)
    });
    efuns.register(0x0017_001A, set_cloned_app_key_hash);
}

/// The argument words a setter can use: word 2, word 3, and words 2-3 as a 64-bit id (low
/// word deeper).
type Words = (u32, u32, u64);

fn set(vm: &mut Vm, ctx: &mut Context, f: fn(&mut BehaviorParams, Words)) -> InterpStatus {
    let call = Call::new(ctx);
    let words = (call.word(2), call.word(3), call.word64(3));
    let this = call.this();
    match vm.native_mut(this) {
        Some(NativeObject::BehaviorParams(p)) => {
            f(p, words);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

fn get_behavior_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let id = match vm.native(this) {
        Some(NativeObject::BehaviorParams(p)) => p.behavior,
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(2, id, INT_RESULT);
    InterpStatus::Success
}

/// `SetTargetedImpulse`: retail copies the `Vector` from the slots at fp-8, fp-4 and fp, so
/// `y` is `this`'s slot and `z` the slot above the frame; the port reads the same slots.
fn set_targeted_impulse(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let x = f32::from_bits(super::word(ctx, 2));
    let y = f32::from_bits(super::word(ctx, 1));
    let z = f32::from_bits(ctx.arg(0).unwrap_or(0));
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::BehaviorParams(p)) => {
            p.impulse = [x, y, z];
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `SetClonedAppKeyHash(AAHash)`: `gGameplayOptionsProfile::AAHashToUIElementOpacities` copies
/// the pairs (values as float bits) into a new `PHashTable<unsigned long,float>` (47 buckets,
/// first key wins), which replaces `m_clonedAppAprHash`.
fn set_cloned_app_key_hash(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, source) = (call.this(), call.word(2));
    let Some(NativeObject::Hash(src)) = vm.native(source) else {
        return vm.bad_this(source);
    };
    let mut dest = ScriptHash::default();
    for node in src.nodes() {
        dest.add(node.key, node.value, &PlainKeys);
    }
    let dest_buckets = dest.num_buckets();
    let pairs: Vec<(u32, f32)> = dest
        .nodes()
        .map(|n| match n.key {
            RefKey::Value(k) => (k as u32, f32::from_bits(n.value as u32)),
            RefKey::Ref { id, .. } => (id, f32::from_bits(n.value as u32)),
        })
        .collect();
    match vm.native_mut(this) {
        Some(NativeObject::BehaviorParams(p)) => {
            p.cloned_app_apr = pairs;
            p.cloned_app_apr_buckets = crate::native::bucket_size_index(dest_buckets) as u8;
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}
