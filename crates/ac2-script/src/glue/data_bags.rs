//! `gLinkRequirements`, `gMissileInfo`, `gMissileTargetingParameters` and `gFXAndBehaviorInfo`:
//! glue that stores into (or reads from) a plain native. Each efun reads `this` at word 1 and
//! its arguments from word 2 on; an undefined or foreign `this` crashes the script
//! (`index != Undef_ReferenceID`).

use super::{word, Call, INT_RESULT};
use crate::native::{LinkRequirements, MissileInfo, MissileTargeting, NativeObject, Position};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let Some((class, method)) = name.split_once("::CallEFun_") else {
            continue;
        };
        let function: crate::vm::EFunFn = match (class, method) {
            ("gLinkRequirements", "IsInputLink") => |vm, c| link_get(vm, c, |l| l.input_link),
            ("gLinkRequirements", "GetLinkType") => |vm, c| link_get(vm, c, |l| l.link_type),
            ("gLinkRequirements", "GetNumLinks") => |vm, c| link_get(vm, c, |l| l.num_links),
            ("gLinkRequirements", "IsRequired") => |vm, c| link_get(vm, c, |l| l.required),
            ("gLinkRequirements", "GetNumReqLinks") => |vm, c| link_get(vm, c, |l| l.num_req_links),
            ("gLinkRequirements", "SetInputLink") => {
                |vm, c| link_set(vm, c, |l, v| l.input_link = v)
            }
            ("gLinkRequirements", "SetLinkType") => |vm, c| link_set(vm, c, |l, v| l.link_type = v),
            ("gLinkRequirements", "SetNumLinks") => |vm, c| link_set(vm, c, |l, v| l.num_links = v),
            ("gLinkRequirements", "SetRequired") => |vm, c| link_set(vm, c, |l, v| l.required = v),
            ("gLinkRequirements", "SetNumReqLinks") => {
                |vm, c| link_set(vm, c, |l, v| l.num_req_links = v)
            }
            ("gMissileInfo", "SetEntityDID") => |vm, c| info_set(vm, c, |i, v| i.entity_did = v),
            ("gMissileInfo", "SetHoldingLocation") => {
                |vm, c| info_set(vm, c, |i, v| i.holding_location = v)
            }
            ("gMissileInfo", "SetThrown") => {
                |vm, c| info_set(vm, c, |i, v| i.thrown = v & 0xFF != 0)
            }
            ("gMissileInfo", "SetClone") => |vm, c| info_set(vm, c, |i, v| i.clone = v & 0xFF != 0),
            ("gMissileInfo", "SetMissileParameters") => info_set_missile_parameters,
            ("gMissileInfo", "SetTargetingParameters") => info_set_targeting,
            ("gMissileInfo", "SetCollisionParameters") => info_set_collision,
            ("gMissileTargetingParameters", "SetTargetID") => |vm, c| set_id(vm, c, false),
            ("gMissileTargetingParameters", "UseSourceObjectForBasePosition") => {
                |vm, c| set_id(vm, c, true)
            }
            ("gMissileTargetingParameters", "SetTargetPos") => |vm, c| set_pos(vm, c, false),
            ("gMissileTargetingParameters", "SetBasePosition") => |vm, c| set_pos(vm, c, true),
            ("gMissileTargetingParameters", "SetInitialSpeed") => |vm, c| {
                target_set(vm, c, |t, v| {
                    t.initial_velocity = f32::from_bits(v);
                    t.bitfield |= 0x4;
                })
            },
            ("gMissileTargetingParameters", "SetInitialPitch") => |vm, c| {
                target_set(vm, c, |t, v| {
                    t.initial_pitch = f32::from_bits(v);
                    t.bitfield |= 0x80;
                })
            },
            ("gMissileTargetingParameters", "SetMaxInitialTargetRange") => |vm, c| {
                target_set(vm, c, |t, v| {
                    t.missile_range = f32::from_bits(v);
                    t.bitfield |= 0x200;
                })
            },
            ("gMissileTargetingParameters", "SetMissileDetonationChance") => |vm, c| {
                target_set(vm, c, |t, v| {
                    t.missile_detonation_chance = f32::from_bits(v);
                    t.bitfield |= 0x800;
                })
            },
            ("gMissileTargetingParameters", "ClientUseParentForBasePosition") => {
                |vm, c| target_set(vm, c, |t, _| t.bitfield |= 0x20)
            }
            ("gMissileTargetingParameters", "SetStartTowardTarget") => {
                |vm, c| target_set(vm, c, |t, _| t.bitfield |= 0x40)
            }
            ("gMissileTargetingParameters", "SetNoFireIfTooFar") => {
                |vm, c| target_set(vm, c, |t, _| t.bitfield |= 0x100)
            }
            ("gMissileTargetingParameters", "SetAlwaysStartTowardTarget") => {
                |vm, c| target_set(vm, c, |t, _| t.bitfield |= 0x400)
            }
            ("gFXAndBehaviorInfo", "SetAttackerFX") => {
                |vm, c| fx_set(vm, c, |f, v| f.attacker_fx = v)
            }
            ("gFXAndBehaviorInfo", "SetTargetFX") => |vm, c| fx_set(vm, c, |f, v| f.target_fx = v),
            ("gFXAndBehaviorInfo", "SetWeaponFX") => |vm, c| fx_set(vm, c, |f, v| f.weapon_fx = v),
            ("gFXAndBehaviorInfo", "SetTargetBehavior") => |vm, c| fx_behavior(vm, c, false),
            ("gFXAndBehaviorInfo", "SetWeaponBehavior") => |vm, c| fx_behavior(vm, c, true),
            _ => continue,
        };
        efuns.register(id, function);
    }
}

/// `gLinkRequirements` getters: the field in slot 2.
fn link_get(vm: &mut Vm, ctx: &mut Context, read: fn(&LinkRequirements) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let value = match vm.native(this) {
        Some(NativeObject::LinkRequirements(l)) => read(l),
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(2, value, INT_RESULT);
    InterpStatus::Success
}

/// `gLinkRequirements` setters: the argument word into the field.
fn link_set(vm: &mut Vm, ctx: &mut Context, write: fn(&mut LinkRequirements, u32)) -> InterpStatus {
    let (this, value) = (Call::new(ctx).this(), word(ctx, 2));
    match vm.native_mut(this) {
        Some(NativeObject::LinkRequirements(l)) => write(l, value),
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

fn info_set(vm: &mut Vm, ctx: &mut Context, write: fn(&mut MissileInfo, u32)) -> InterpStatus {
    let (this, value) = (Call::new(ctx).this(), word(ctx, 2));
    match vm.native_mut(this) {
        Some(NativeObject::MissileInfo(i)) => write(i, value),
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

/// `MissileInfo::SetMissileParameters(params)`: a copy of the parameters native.
fn info_set_missile_parameters(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(NativeObject::MissileParameters(p)) = vm.native(arg).cloned() else {
        return vm.bad_this(arg);
    };
    match vm.native_mut(this) {
        Some(NativeObject::MissileInfo(i)) => i.missile_params = Some(p),
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

/// `MissileInfo::SetTargetingParameters(params)`.
fn info_set_targeting(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(NativeObject::MissileTargeting(p)) = vm.native(arg).cloned() else {
        return vm.bad_this(arg);
    };
    match vm.native_mut(this) {
        Some(NativeObject::MissileInfo(i)) => i.targeting_params = *p,
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

/// `MissileInfo::SetCollisionParameters(params)`.
fn info_set_collision(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, arg) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(NativeObject::CollisionParameters(p)) = vm.native(arg).cloned() else {
        return vm.bad_this(arg);
    };
    match vm.native_mut(this) {
        Some(NativeObject::MissileInfo(i)) => i.collision_params = p,
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

fn target_set(
    vm: &mut Vm,
    ctx: &mut Context,
    write: fn(&mut MissileTargeting, u32),
) -> InterpStatus {
    let (this, value) = (Call::new(ctx).this(), word(ctx, 2));
    match vm.native_mut(this) {
        Some(NativeObject::MissileTargeting(t)) => write(t, value),
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

/// `MissileTargetingParameters::SetTargetID` (`source` false: group bit 1) and
/// `UseSourceObjectForBasePosition` (group bit 0x10): the 64-bit id (words 3 and 2), the height
/// adjustment, the use-scale flag and the offset vector.
fn set_id(vm: &mut Vm, ctx: &mut Context, source: bool) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, id) = (call.this(), call.word64(3));
    let (height, use_scale, offset) = (f32::from_bits(call.word(4)), call.word(5), call.word(6));
    let Some(NativeObject::Vector(offset)) = vm.native(offset).cloned() else {
        return vm.bad_this(offset);
    };
    match vm.native_mut(this) {
        Some(NativeObject::MissileTargeting(t)) if source => {
            t.source_id = id;
            t.source_height_adjustment = height;
            t.use_source_scale = use_scale;
            t.source_offset = offset;
            t.bitfield |= 0x10;
        }
        Some(NativeObject::MissileTargeting(t)) => {
            t.target_id = id;
            t.target_height_adjustment = height;
            t.use_target_scale = use_scale;
            t.target_offset = offset;
            t.bitfield |= 0x1;
        }
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

/// `SetTargetPos` (group bit 2) and `SetBasePosition` (group bit 8): a copy of the position.
fn set_pos(vm: &mut Vm, ctx: &mut Context, base: bool) -> InterpStatus {
    let (this, arg) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(NativeObject::Position(pos)) = vm.native(arg).cloned() else {
        return vm.bad_this(arg);
    };
    let pos: Position = pos;
    match vm.native_mut(this) {
        Some(NativeObject::MissileTargeting(t)) if base => {
            t.base_pos = pos;
            t.bitfield |= 0x8;
        }
        Some(NativeObject::MissileTargeting(t)) => {
            t.target_pos = pos;
            t.bitfield |= 0x2;
        }
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

fn fx_set(
    vm: &mut Vm,
    ctx: &mut Context,
    write: fn(&mut crate::native::FxAndBehavior, u32),
) -> InterpStatus {
    let (this, value) = (Call::new(ctx).this(), word(ctx, 2));
    match vm.native_mut(this) {
        Some(NativeObject::FxAndBehavior(f)) => write(f, value),
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}

/// `FXAndBehaviorInfo` behaviour setters: a copy of the `BehaviorParams`.
fn fx_behavior(vm: &mut Vm, ctx: &mut Context, weapon: bool) -> InterpStatus {
    let (this, arg) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(NativeObject::BehaviorParams(b)) = vm.native(arg).cloned() else {
        return vm.bad_this(arg);
    };
    match vm.native_mut(this) {
        Some(NativeObject::FxAndBehavior(f)) if weapon => f.weapon_behavior = Some(b),
        Some(NativeObject::FxAndBehavior(f)) => f.target_behavior = Some(b),
        _ => return vm.bad_this(this),
    }
    InterpStatus::Success
}
