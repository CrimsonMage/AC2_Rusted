//! `gCollisionParameters` and `gCollisionProfile`: the impact minimums a script sets, and the
//! collision report it reads (`CollisionProfile::HasCollided`, `IsEndCollision`, the object and
//! property lists, `HasContact`).

use super::{out_native, Call};
use crate::native::{CollideProperty, CollisionParameters, CollisionProfile, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let function: crate::vm::EFunFn = match name {
            "gCollisionParameters::CallEFun_SetMinReportedImpactXY" => set_min_xy,
            "gCollisionParameters::CallEFun_SetMinReportedImpactZ" => set_min_z,
            "gCollisionProfile::CallEFun_HasCollided" => has_collided,
            "gCollisionProfile::CallEFun_IsEndCollision" => is_end_collision,
            "gCollisionProfile::CallEFun_GetObject" => get_object,
            "gCollisionProfile::CallEFun_GetObjectProperty" => get_object_property,
            "gCollisionProfile::CallEFun_GetEnvProperty" => get_env_property,
            "gCollisionProfile::CallEFun_GetSurfaceProperty" => get_surface_property,
            "gCollisionProfile::CallEFun_GetWaterProperty" => get_water_property,
            "gCollisionProfile::CallEFun_HasContact" => has_contact,
            "gCollisionProfile::CallEFun_IsEndContact" => is_end_contact,
            "gCollisionProfile::CallEFun_FindObject" => find_object,
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

/// `gCollisionProfile::CallEFun_GetObject(int index, ObjCollisionProfile *)`: the object at
/// `index` (`CollisionProfile::GetObjectA`) is copied into the out native; the result is 1, or 0
/// with the out untouched when `index` is past the list.
fn get_object(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, index, out) = (call.this(), call.word(2) as usize, call.word(3));
    let found = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => p.objects.get(index).copied(),
        _ => return vm.bad_this(this),
    };
    let hit = found.is_some();
    if let Some(object) = found {
        let target = out_native(vm, ctx, out);
        if let Some(NativeObject::ObjCollisionProfile(o)) = vm.native_mut(target) {
            *o = object;
        }
    }
    Call::new(ctx).ret(4, u32::from(hit), tag::BOOL);
    InterpStatus::Success
}

/// `gCollisionProfile::CallEFun_GetObjectProperty(int index, CollideProperty *)`.
fn get_object_property(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_at(vm, ctx, |p| &p.object_properties)
}

/// `gCollisionProfile::CallEFun_GetEnvProperty(int index, CollideProperty *)`.
fn get_env_property(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_at(vm, ctx, |p| &p.env_properties)
}

/// `gCollisionProfile::CallEFun_GetSurfaceProperty(int index, CollideProperty *)`.
fn get_surface_property(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_at(vm, ctx, |p| &p.surface_properties)
}

/// `gCollisionProfile::CallEFun_GetWaterProperty(int index, CollideProperty *)`.
fn get_water_property(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    property_at(vm, ctx, |p| &p.water_properties)
}

/// The shared body of the four property getters (`CollisionProfile::GetObjectProperty`,
/// `GetEnvProperty`, `GetSurfaceProperty`, `GetWaterProperty`): the property at `index` of the
/// list `list` picks is copied into the out `CollideProperty`; the result is 1, or 0 past the end.
fn property_at(
    vm: &mut Vm,
    ctx: &mut Context,
    list: fn(&CollisionProfile) -> &Vec<CollideProperty>,
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, index, out) = (call.this(), call.word(2) as usize, call.word(3));
    let found = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => list(p).get(index).copied(),
        _ => return vm.bad_this(this),
    };
    let hit = found.is_some();
    if let Some(property) = found {
        let target = out_native(vm, ctx, out);
        if let Some(NativeObject::CollideProperty(c)) = vm.native_mut(target) {
            *c = property;
        }
    }
    Call::new(ctx).ret(4, u32::from(hit), tag::BOOL);
    InterpStatus::Success
}

/// `gCollisionProfile::CallEFun_HasContact(Plane *)`: when the report has a contact
/// (`CollisionProfile::HasContact`, state 0 or 2), the plane is written; the result is 1, else 0
/// with the plane untouched.
fn has_contact(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, plane_ref) = (call.this(), call.word(2));
    let contact = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => p.has_contact(),
        _ => return vm.bad_this(this),
    };
    let found = contact.is_some();
    if let Some(plane) = contact {
        let target = out_native(vm, ctx, plane_ref);
        if let Some(NativeObject::Plane(p)) = vm.native_mut(target) {
            *p = plane;
        }
    }
    Call::new(ctx).ret(3, u32::from(found), tag::BOOL);
    InterpStatus::Success
}

/// `gCollisionProfile::CallEFun_IsEndContact()`: the contact state is 1 (an end).
fn is_end_contact(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let end = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => p.is_end_contact(),
        _ => return vm.bad_this(this),
    };
    Call::new(ctx).ret(2, u32::from(end), tag::BOOL);
    InterpStatus::Success
}

/// `gCollisionProfile::CallEFun_FindObject(InstanceID, ObjCollisionProfile *)`: the first listed
/// object with that id (`CollisionProfile::FindObject`) is copied into the out native; the
/// result is 1, or 0 with the out untouched. The id is an 8-byte argument (two words).
fn find_object(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, id, out) = (call.this(), call.word64(3), call.word(4));
    let found = match vm.native(this) {
        Some(NativeObject::CollisionProfile(p)) => p.objects.iter().copied().find(|o| o.id == id),
        _ => return vm.bad_this(this),
    };
    let hit = found.is_some();
    if let Some(object) = found {
        let target = out_native(vm, ctx, out);
        if let Some(NativeObject::ObjCollisionProfile(o)) = vm.native_mut(target) {
            *o = object;
        }
    }
    Call::new(ctx).ret(5, u32::from(hit), tag::BOOL);
    InterpStatus::Success
}
