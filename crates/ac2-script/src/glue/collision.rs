//! `gCollisionParameters` and `gCollisionProfile`: the impact minimums a script sets, and the
//! collision report it reads (`CollisionProfile::HasCollided`, `IsEndCollision`).

use super::{out_native, Call};
use crate::native::{CollisionParameters, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let function: crate::vm::EFunFn = match name {
            "gCollisionParameters::CallEFun_SetMinReportedImpactXY" => set_min_xy,
            "gCollisionParameters::CallEFun_SetMinReportedImpactZ" => set_min_z,
            "gCollisionProfile::CallEFun_HasCollided" => has_collided,
            "gCollisionProfile::CallEFun_IsEndCollision" => is_end_collision,
            _ => continue,
        };
        efuns.register(id, function);
    }
}

/// `gCollisionParameters::CallEFun_SetMinReportedImpactXY(float)`: writes the native's XY
/// minimum (retail's +0xA8). No result. An undefined `this` crashes the script.
fn set_min_xy(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_minimum(vm, ctx, |p, value| p.min_reported_impact_xy = value)
}

/// `gCollisionParameters::CallEFun_SetMinReportedImpactZ(float)`: the Z minimum (+0xAC).
fn set_min_z(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    set_minimum(vm, ctx, |p, value| p.min_reported_impact_z = value)
}

fn set_minimum(
    vm: &mut Vm,
    ctx: &mut Context,
    write: fn(&mut CollisionParameters, f32),
) -> InterpStatus {
    let call = Call::new(ctx);
    let this = call.this();
    let value = f32::from_bits(call.word(2));
    match vm.native_mut(this) {
        Some(NativeObject::CollisionParameters(p)) => {
            write(p, value);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `gCollisionProfile::CallEFun_HasCollided(Vector *normal, float *speed)`: when the report has
/// a collision (`CollisionProfile::HasCollided`), the normal and the speed are written through
/// the out parameters; the result is 1, else 0 and the outs are untouched.
fn has_collided(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, normal_ref, speed_ref) = (call.this(), call.word(2), call.word(3));
    let report = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => p.has_collided(),
        _ => return vm.bad_this(this),
    };
    let found = report.is_some();
    if let Some((normal, speed)) = report {
        let vector = out_native(vm, ctx, normal_ref);
        if let Some(NativeObject::Vector(v)) = vm.native_mut(vector) {
            *v = normal;
        }
        vm.write_through(ctx, speed_ref, speed.to_bits(), tag::INT);
    }
    Call::new(ctx).ret(4, u32::from(found), tag::BOOL);
    InterpStatus::Success
}

/// `gCollisionProfile::CallEFun_IsEndCollision()`: state 1.
fn is_end_collision(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let end = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => p.is_end_collision(),
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(2, u32::from(end), tag::BOOL);
    InterpStatus::Success
}
