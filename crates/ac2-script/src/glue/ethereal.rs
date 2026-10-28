//! `gCollisionParameters`' mask getters and raw setters, `gEtherealPhysicsType` and
//! `gEtherealWeenieType` (`portal/shared/src/interp/glue/gcollisionparameters.cpp`,
//! `getherealphysicstype.cpp`, `getherealweenietype.cpp`).
//!
//! The setters take an `EtherealPhysicsType` / `EtherealWeenieType` native and copy it into the
//! parameters' set (`CollisionParameters::SetPhysicsType` and kin also set the set's bit); the
//! getters make a new native holding a copy. The `...DWord` setters write one word of a mask and
//! set no bit. `EtherealPhysicsType::SetProperty(name, flag)` sets or clears the bit the name's
//! `MasterProperty` desc gives ([`super::collision::property_bit`]).

use ac2_physics::{PhysicsTypeMask, WeenieTypes};

use super::collision::{has_bit, property_bit};
use super::{out_native, Call, INT_RESULT};
use crate::native::{ntype, CollisionParameters, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

/// The water layer property `EtherealPhysicsType::SetEtherealToWaterLayer` clears.
const WATER_LAYER: u64 = 0x4000_001C;
/// The environment property `EtherealPhysicsType::SetSolidOnlyToEnvironment` clears.
const ENVIRONMENT: u64 = 0x4000_001B;

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let function: crate::vm::EFunFn = match name {
            "gCollisionParameters::CallEFun_SetPhysicsType" => {
                |vm, ctx| set_physics(vm, ctx, |p, mask| p.set_physics_type(mask))
            }
            "gCollisionParameters::CallEFun_SetPhysicsPlacementType" => {
                |vm, ctx| set_physics(vm, ctx, |p, mask| p.set_physics_placement_type(mask))
            }
            "gCollisionParameters::CallEFun_SetPhysicsMovementType" => {
                |vm, ctx| set_physics(vm, ctx, |p, mask| p.set_physics_movement_type(mask))
            }
            "gCollisionParameters::CallEFun_SetWeeniePlacementType" => {
                |vm, ctx| set_weenie(vm, ctx, |p, types| p.set_weenie_placement_type(types))
            }
            "gCollisionParameters::CallEFun_SetWeenieMovementType" => {
                |vm, ctx| set_weenie(vm, ctx, |p, types| p.set_weenie_movement_type(types))
            }
            "gCollisionParameters::CallEFun_GetPhysicsType" => {
                |vm, ctx| get_physics(vm, ctx, |p| p.physics_type())
            }
            "gCollisionParameters::CallEFun_GetPhysicsPlacementType" => {
                |vm, ctx| get_physics(vm, ctx, |p| p.placement_physics())
            }
            "gCollisionParameters::CallEFun_GetPhysicsMovementType" => {
                |vm, ctx| get_physics(vm, ctx, |p| p.movement_physics())
            }
            "gCollisionParameters::CallEFun_GetWeeniePlacementType" => {
                |vm, ctx| get_weenie(vm, ctx, |p| p.placement_weenie().clone())
            }
            "gCollisionParameters::CallEFun_GetWeenieMovementType" => {
                |vm, ctx| get_weenie(vm, ctx, |p| p.movement_weenie().clone())
            }
            "gCollisionParameters::CallEFun_SetPhysicsTypeLowDWord" => {
                |vm, ctx| set_word(vm, ctx, |p, w| p.physics_type_mut().set_low_dword(w))
            }
            "gCollisionParameters::CallEFun_SetPhysicsTypeHighDWord" => {
                |vm, ctx| set_word(vm, ctx, |p, w| p.physics_type_mut().set_high_dword(w))
            }
            "gCollisionParameters::CallEFun_SetMovementLowDWord" => {
                |vm, ctx| set_word(vm, ctx, |p, w| p.movement_physics_mut().set_low_dword(w))
            }
            "gCollisionParameters::CallEFun_SetMovementHighDWord" => {
                |vm, ctx| set_word(vm, ctx, |p, w| p.movement_physics_mut().set_high_dword(w))
            }
            "gCollisionParameters::CallEFun_SetPlacementLowDWord" => {
                |vm, ctx| set_word(vm, ctx, |p, w| p.placement_physics_mut().set_low_dword(w))
            }
            "gCollisionParameters::CallEFun_SetPlacementHighDWord" => {
                |vm, ctx| set_word(vm, ctx, |p, w| p.placement_physics_mut().set_high_dword(w))
            }
            "gEtherealPhysicsType::CallEFun_SetToEthereal" => {
                |vm, ctx| physics_mask(vm, ctx, |m| *m = PhysicsTypeMask::from_bits(u64::MAX))
            }
            "gEtherealPhysicsType::CallEFun_SetToSolid" => {
                |vm, ctx| physics_mask(vm, ctx, |m| *m = PhysicsTypeMask::default())
            }
            "gEtherealPhysicsType::CallEFun_SetSolidOnlyToEnvironment" => {
                |vm, ctx| physics_mask(vm, ctx, solid_only_to_environment)
            }
            "gEtherealPhysicsType::CallEFun_SetEtherealToWaterLayer" => set_ethereal_to_water,
            "gEtherealPhysicsType::CallEFun_SetProperty" => set_property,
            "gEtherealPhysicsType::CallEFun_SetPropertyEthereal" => {
                |vm, ctx| set_property_flag(vm, ctx, true)
            }
            "gEtherealPhysicsType::CallEFun_SetPropertySolid" => {
                |vm, ctx| set_property_flag(vm, ctx, false)
            }
            "gEtherealWeenieType::CallEFun_SetToEthereal" => {
                |vm, ctx| weenie_types(vm, ctx, WeenieTypes::set_to_ethereal)
            }
            "gEtherealWeenieType::CallEFun_SetToSolid" => {
                |vm, ctx| weenie_types(vm, ctx, WeenieTypes::set_to_solid)
            }
            "gEtherealWeenieType::CallEFun_AddToEtherealList" => {
                |vm, ctx| weenie_id(vm, ctx, WeenieTypes::add_to_ethereal_list)
            }
            "gEtherealWeenieType::CallEFun_AddToSolidList" => {
                |vm, ctx| weenie_id(vm, ctx, WeenieTypes::add_to_solid_list)
            }
            _ => continue,
        };
        efuns.register(id, function);
    }
}

/// The `EtherealPhysicsType` a native argument names.
fn physics_arg(vm: &mut Vm, ctx: &Context, arg: u32) -> Result<PhysicsTypeMask, InterpStatus> {
    let reference = out_native(vm, ctx, arg);
    match vm.native(reference) {
        Some(NativeObject::EtherealPhysicsType(mask)) => Ok(*mask),
        _ => Err(vm.bad_this(reference)),
    }
}

fn parameters(vm: &mut Vm, this: u32) -> Option<&mut CollisionParameters> {
    match vm.native_mut(this) {
        Some(NativeObject::CollisionParameters(p)) => Some(p),
        _ => None,
    }
}

/// `Set*PhysicsType(EtherealPhysicsType)`: the argument's mask goes into the set.
fn set_physics(
    vm: &mut Vm,
    ctx: &mut Context,
    set: fn(&mut ac2_physics::CollisionParameters, PhysicsTypeMask),
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, arg) = (call.this(), call.word(2));
    if parameters(vm, this).is_none() {
        return vm.bad_this(this);
    }
    let mask = match physics_arg(vm, ctx, arg) {
        Ok(mask) => mask,
        Err(status) => return status,
    };
    if let Some(p) = parameters(vm, this) {
        set(&mut p.sets, mask);
    }
    InterpStatus::Success
}

/// `SetWeenie*Type(EtherealWeenieType)`: `EtherealWeenieType::Update` into the set.
fn set_weenie(
    vm: &mut Vm,
    ctx: &mut Context,
    set: fn(&mut ac2_physics::CollisionParameters, &WeenieTypes),
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, arg) = (call.this(), call.word(2));
    if parameters(vm, this).is_none() {
        return vm.bad_this(this);
    }
    let source = out_native(vm, ctx, arg);
    let types = match vm.native(source) {
        Some(NativeObject::EtherealWeenieType(types)) => (**types).clone(),
        _ => return vm.bad_this(source),
    };
    if let Some(p) = parameters(vm, this) {
        set(&mut p.sets, &types);
    }
    InterpStatus::Success
}

/// `Get*PhysicsType()`: a new `EtherealPhysicsType` native holding a copy (word 2).
fn get_physics(
    vm: &mut Vm,
    ctx: &mut Context,
    read: fn(&ac2_physics::CollisionParameters) -> PhysicsTypeMask,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(p) = parameters(vm, this) else {
        return vm.bad_this(this);
    };
    let mask = read(&p.sets);
    let object = vm.refs.create_native(
        ntype::ETHEREAL_PHYSICS_TYPE,
        NativeObject::EtherealPhysicsType(mask),
    );
    Call::new(ctx).ret(2, object, tag::REF);
    InterpStatus::Success
}

/// `GetWeenie*Type()`: a new `EtherealWeenieType` native holding a copy.
fn get_weenie(
    vm: &mut Vm,
    ctx: &mut Context,
    read: fn(&ac2_physics::CollisionParameters) -> WeenieTypes,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(p) = parameters(vm, this) else {
        return vm.bad_this(this);
    };
    let types = read(&p.sets);
    let object = vm.refs.create_native(
        ntype::ETHEREAL_WEENIE_TYPE,
        NativeObject::EtherealWeenieType(Box::new(types)),
    );
    Call::new(ctx).ret(2, object, tag::REF);
    InterpStatus::Success
}

/// `Set*DWord(int)`: one word of a mask, no bit set.
fn set_word(
    vm: &mut Vm,
    ctx: &mut Context,
    write: fn(&mut ac2_physics::CollisionParameters, u32),
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, word) = (call.this(), call.word(2));
    let Some(p) = parameters(vm, this) else {
        return vm.bad_this(this);
    };
    write(&mut p.sets, word);
    InterpStatus::Success
}

/// Runs `edit` on the `EtherealPhysicsType` at `this`.
fn physics_mask(vm: &mut Vm, ctx: &mut Context, edit: fn(&mut PhysicsTypeMask)) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::EtherealPhysicsType(mask)) => {
            edit(mask);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `EtherealPhysicsType::SetProperty(name, flag)`: sets or clears the bit the name's desc gives;
/// 0 for a name that has none.
fn set_property_bit(mask: &mut PhysicsTypeMask, name: u64, flag: bool) -> bool {
    let bit = property_bit(name);
    mask.set_property(bit, flag, has_bit)
}

/// `EtherealPhysicsType::SetSolidOnlyToEnvironment`: every type blocks except the environment's.
fn solid_only_to_environment(mask: &mut PhysicsTypeMask) {
    *mask = PhysicsTypeMask::from_bits(u64::MAX);
    set_property_bit(mask, ENVIRONMENT, false);
}

/// `SetEtherealToWaterLayer(int)`: a non-zero flag first sets the water layer's bit, and the
/// bit is then cleared in either case.
fn set_ethereal_to_water(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, ethereal) = (call.this(), call.word(2) != 0);
    match vm.native_mut(this) {
        Some(NativeObject::EtherealPhysicsType(mask)) => {
            if ethereal {
                set_property_bit(mask, WATER_LAYER, true);
            }
            set_property_bit(mask, WATER_LAYER, false);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `SetProperty(name, flag)`: the name at word 2, the flag at word 3, the result at word 4.
fn set_property(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, name, flag) = (call.this(), call.word(2), call.word(3) != 0);
    property_result(vm, ctx, this, name, flag, 4)
}

/// `SetPropertyEthereal(name)` / `SetPropertySolid(name)`: the result at word 3.
fn set_property_flag(vm: &mut Vm, ctx: &mut Context, flag: bool) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, name) = (call.this(), call.word(2));
    property_result(vm, ctx, this, name, flag, 3)
}

fn property_result(
    vm: &mut Vm,
    ctx: &mut Context,
    this: u32,
    name: u32,
    flag: bool,
    result_slot: usize,
) -> InterpStatus {
    let ok = match vm.native_mut(this) {
        Some(NativeObject::EtherealPhysicsType(mask)) => {
            set_property_bit(mask, u64::from(name), flag)
        }
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(result_slot, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// Runs `edit` on the `EtherealWeenieType` at `this`.
fn weenie_types(vm: &mut Vm, ctx: &mut Context, edit: fn(&mut WeenieTypes)) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::EtherealWeenieType(types)) => {
            edit(types);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `AddToEtherealList(InstanceID)` / `AddToSolidList(InstanceID)`: the id (high word at word 2,
/// low at word 3) joins the list; the result is word 4.
fn weenie_id(
    vm: &mut Vm,
    ctx: &mut Context,
    add: fn(&mut WeenieTypes, u64) -> bool,
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, id) = (call.this(), call.word64(3));
    let ok = match vm.native_mut(this) {
        Some(NativeObject::EtherealWeenieType(types)) => add(types, id),
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(4, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}
