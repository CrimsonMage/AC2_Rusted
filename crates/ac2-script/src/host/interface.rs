//! `gCWInterface` efuns on a weenie's own interface (`CWInterface`, the `this` of a weenie
//! script): which object it is and the objects it can reach. Each wrapper crashes on an
//! undefined `this` ("index != Undef_ReferenceID").

use ac2_net::protocol::ObjectAction;

use crate::glue::{word64, Call};
use crate::host::LatentPhysics;
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x007F_000A, get_internal_weenie_object);
    // `GetExternalWeenieObject` (admin 0x004BB580, unnamed in the efun table): the same code;
    // Legions binds 0x7F000A and 0x7F000B to one function (0x004B0680).
    efuns.register(0x007F_000B, get_internal_weenie_object);
    efuns.register(0x007F_0040, is_the_player);
    efuns.register(0x007F_0052, process_text);
    efuns.register(0x007F_0053, display_help_text);
    // `gCWBookKeeper::CallEFun_{Add,Remove}ObjectToBeDestroyed` (the weenie's book keeper, `this`):
    // `CWBookKeeper` -> `CBookKeep::{Add,Remove}ObjectToBeDestroyed`, which answer 1 whenever
    // `CObjectMaint` exists (it always does here). The destroy list itself is the client's object
    // maintenance, which this client does not keep (see the catalog).
    efuns.register(0x007D_001E, add_object_to_be_destroyed);
    efuns.register(0x007D_001F, remove_object_to_be_destroyed);
    efuns.register(0x0080_0057, set_no_draw_physics);
    efuns.register(0x0080_0059, get_local_visual_desc);
    efuns.register(0x0080_002A, look_at);
    efuns.register(0x0080_002C, stop_all_camera_fx);
    efuns.register(0x0080_0045, do_fx);
    efuns.register(0x0080_0047, stop_fx);
    efuns.register(0x0080_0015, do_mode);
    efuns.register(0x0080_0017, set_mode);
    efuns.register(0x0080_001B, do_behavior);
    efuns.register(0x0080_001E, release_behavior);
    efuns.register(0x0080_0027, stop_behavior);
    efuns.register(0x0080_0003, |vm, ctx| {
        physics_scale(vm, ctx, ObjectAction::SetVelocityScale)
    });
    efuns.register(0x0080_0005, |vm, ctx| {
        physics_scale(vm, ctx, ObjectAction::SetAccelerationScale)
    });
    efuns.register(0x0080_0007, |vm, ctx| {
        physics_scale(vm, ctx, ObjectAction::SetJumpScale)
    });
    efuns.register(0x0080_000A, set_collision_state);
}

/// `gCWPhysicsObject::CallEFun_LookAt(iid)` -> `CWPhysicsObject::LookAt` -> `CPhysicsObj::LookAt`:
/// the weenie's body turns its head to the object `iid` (`INVALID_IID` ends the look). Queued as
/// the server's `LookAt` event for the object; answers 1 for a weenie with an owner.
fn look_at(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let target = Call::new(ctx).word64(3);
    if id != 0 {
        vm.physics.push((id, ObjectAction::LookAt(target)));
    }
    Call::new(ctx).ret(4, u32::from(id != 0), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_StopAllCameraFX` (`CWPhysicsObject::StopAllCameraFX` ->
/// `CPhysicsObj`'s camera effects): the weenie's physics object's camera effects end. The client
/// has no camera effect manager yet, so the stop has nothing to end here (a known gap); the call
/// only checks `this` and returns, as retail's does.
fn stop_all_camera_fx(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    match this_weenie(vm, ctx) {
        Ok(_) => InterpStatus::Success,
        Err(status) => status,
    }
}

/// `gCWPhysicsObject::CallEFun_DoFX(fx, scalar)` -> `CWPhysicsObject::DoFX` -> `CPhysicsObj::DoFX`
/// -> `FXManager::DoFX`: the weenie's physics object starts the effect (its sounds, which the
/// client's FX manager plays). The call is queued as the server's `DoFX` event for the object;
/// the efun answers 1 (started) for a weenie with an owner. Equipping a weapon calls it
/// (`CPlayer` 0xC8's `EquipItem_Done`).
fn do_fx(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let fx = Call::new(ctx).word(2);
    let scalar = f32::from_bits(Call::new(ctx).word(3));
    vm.physics.push((id, ObjectAction::DoFx { fx, scalar }));
    Call::new(ctx).ret(4, 1, tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_StopFX(fx)` -> `CWPhysicsObject::StopFX` -> `CPhysicsObj::StopFX`:
/// the effect's sounds stop. Answers 1 for a weenie with an owner.
fn stop_fx(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let fx = Call::new(ctx).word(2);
    vm.physics.push((id, ObjectAction::StopFx(fx)));
    Call::new(ctx).ret(3, 1, tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_DoMode(mode)` -> `CWPhysicsObject::DoMode` -> `CPhysicsObj::DoMode(
/// cid, mode)`: the weenie's body animates toward the mode. The start is queued for the body with
/// the context (`Vm::latent_physics`); the efun answers 1 for a weenie with an owner and the
/// context stops until the mode is reached (`ModeDone`, `ScriptHost::resume_latent`).
fn do_mode(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let mode = Call::new(ctx).word(2);
    if id == 0 {
        Call::new(ctx).ret(4, 0, tag::INT);
        return InterpStatus::Success;
    }
    vm.latent_physics.push(LatentPhysics {
        object: id,
        context: ctx.cid,
        action: ObjectAction::DoMode(mode),
    });
    Call::new(ctx).ret(4, 1, tag::INT);
    InterpStatus::Interrupted
}

/// `gCWPhysicsObject::CallEFun_SetMode(mode)` -> `CWPhysicsObject::SetMode` -> `CPhysicsObj::SetMode`
/// (`ModeTracker::SetMode`, switching at once): queued as the server's `SetMode`. Answers 1 for a
/// weenie with an owner.
fn set_mode(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let mode = Call::new(ctx).word(2);
    if id != 0 {
        vm.physics.push((id, ObjectAction::SetMode(mode)));
    }
    Call::new(ctx).ret(3, u32::from(id != 0), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_DoBehavior(params)` -> `CWPhysicsObject::DoBehavior` ->
/// `CPhysicsObj::DoBehavior(params)`: the `BehaviorParams` native in argument 2 starts the
/// behaviour on the body with the context (`Vm::latent_physics`). Retail answers 0 for a weenie
/// with a physics object (8 without one), and the context stops while the behaviour runs
/// (`BehaviorDone`, `ScriptHost::resume_latent`). An argument that is not a `BehaviorParams`
/// crashes the script.
fn do_behavior(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let reference = Call::new(ctx).word(2);
    let params = match vm.native(reference) {
        Some(NativeObject::BehaviorParams(p)) => p.clone(),
        _ => return vm.bad_this(reference),
    };
    if id == 0 {
        Call::new(ctx).ret(4, 8, tag::INT);
        return InterpStatus::Success;
    }
    vm.latent_physics.push(LatentPhysics {
        object: id,
        context: ctx.cid,
        action: ObjectAction::DoBehavior(params),
    });
    Call::new(ctx).ret(4, 0, tag::INT);
    InterpStatus::Interrupted
}

/// `gCWPhysicsObject::CallEFun_ReleaseBehavior(behavior, params)` -> `CWPhysicsObject::
/// ReleaseBehavior` -> `CPhysicsObj::ReleaseBehavior`: queued as the server's `ReleaseBehavior`
/// with the behaviour id (argument 2) and the `BehaviorParams` native (argument 3). Answers 0 for
/// a weenie with an owner (8 without one).
fn release_behavior(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let behavior = Call::new(ctx).word(2);
    let reference = Call::new(ctx).word(3);
    let params = match vm.native(reference) {
        Some(NativeObject::BehaviorParams(p)) => p.clone(),
        _ => return vm.bad_this(reference),
    };
    if id != 0 {
        vm.physics
            .push((id, ObjectAction::ReleaseBehavior { behavior, params }));
    }
    Call::new(ctx).ret(5, if id != 0 { 0 } else { 8 }, tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_StopBehavior(behavior)` -> `CWPhysicsObject::StopBehavior` ->
/// `CPhysicsObj::StopBehavior` (which answers 1): queued as the server's `StopBehavior`. Answers 1
/// for a weenie with an owner (8 without one, as `CWPhysicsObject::StopBehavior` does).
fn stop_behavior(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let behavior = Call::new(ctx).word(2);
    if id != 0 {
        vm.physics.push((id, ObjectAction::StopBehavior(behavior)));
    }
    Call::new(ctx).ret(3, if id != 0 { 1 } else { 8 }, tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_SetVelocityScale`, `SetAccelerationScale` and `SetJumpScale` (a
/// float in argument 2, no result): `CWPhysicsObject::Set*Scale` on the weenie's physics object,
/// queued as the server's event for it.
fn physics_scale(vm: &mut Vm, ctx: &mut Context, action: fn(f32) -> ObjectAction) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let scale = f32::from_bits(Call::new(ctx).word(2));
    if id != 0 {
        vm.physics.push((id, action(scale)));
    }
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_SetCollisionState(CollisionParameters *)` ->
/// `CWPhysicsObject::SetCollisionState` -> `CPhysicsObj::SetCollisionState` ->
/// `CCollisionManager::SetCollisionState`: the parameters' non-zero impact minimums replace the
/// weenie's body's (the body keeps the zero ones). The parameters must be a native (an
/// argument that is not one crashes the script). The efun answers 0, as retail does.
fn set_collision_state(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let params = Call::new(ctx).word(2);
    let (xy, z, sets) = match vm.native(params) {
        Some(NativeObject::CollisionParameters(p)) => (
            p.min_reported_impact_xy,
            p.min_reported_impact_z,
            (*p.sets).clone(),
        ),
        _ => return vm.bad_this(params),
    };
    if id != 0 {
        vm.collision_minimums.push((id, xy, z));
        vm.collision_sets.push((id, sets));
    }
    Call::new(ctx).ret(3, 0, tag::BOOL);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_SetNoDrawPhysics(flag)` -> `CWPhysicsObject::SetNoDrawPhysics` ->
/// `CPhysicsObj::set_nodraw` (the state's 0x10 bit set or cleared): the weenie's physics object
/// is the caller's, which the client owns, so the change is queued for it. The efun returns 1 (an
/// int; retail returns 0 for a weenie without a physics object, which the host cannot be).
fn set_no_draw_physics(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let flag = Call::new(ctx).word(2) != 0;
    vm.no_draw.push((id, flag));
    Call::new(ctx).ret(3, 1, tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_GetLocalVisualDesc` -> `CWPhysicsObject::GetLocalVisualDesc`
/// (`VisualDesc::VisualDesc` copy into a new native of type 0x71): the weenie's local description
/// as the client holds it, returned as a new reference in the return slot (word 2, tagged REF).
/// Retail reads the physics object's `m_visual`; the client's object table keeps that copy and
/// the host shares it through [`crate::host::ScriptHost::set_appearance`]. A weenie the host
/// never shared has no description, so the copy is the default (empty) one.
fn get_local_visual_desc(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let appearance = vm.appearances.get(&id).cloned().unwrap_or_default();
    let reference = vm.refs.create_native(
        crate::native::ntype::VISUAL_DESC,
        NativeObject::VisualDesc(Box::new(appearance)),
    );
    Call::new(ctx).ret(2, reference, tag::REF);
    InterpStatus::Success
}

/// `CWInterface::ProcessText(text)` (`gCWInterface::CallEFun_ProcessText`, the text a `wpstring`
/// in argument 2, the status in argument 3): the line goes to the chat shell, which the client
/// runs (see `Vm::shell_lines`). The status the shell gives is not traced here, so it returns 0.
fn process_text(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    if let Err(status) = this_weenie(vm, ctx) {
        return status;
    }
    let reference = Call::new(ctx).word(2);
    let Some(text) = vm.wstring(reference).map(String::from_utf16_lossy) else {
        return vm.bad_this(reference);
    };
    vm.shell_lines.push(text);
    Call::new(ctx).ret(3, 0, tag::INT);
    InterpStatus::Success
}

/// `gCWInterface::CallEFun_DisplayHelpText(cmd)` -> `CWInterface::DisplayHelpText`: the player
/// shell's help for `cmd` (an empty one asks for all) is shown in the chat windows. The request is
/// queued; the client's shell answers it (`ScriptPort::display_help`).
fn display_help_text(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    if let Err(status) = this_weenie(vm, ctx) {
        return status;
    }
    let reference = Call::new(ctx).word(2);
    let Some(text) = vm.wstring(reference).map(String::from_utf16_lossy) else {
        return vm.bad_this(reference);
    };
    vm.help_requests.push(text);
    InterpStatus::Success
}

fn add_object_to_be_destroyed(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    book_keeper_answer(vm, ctx)
}

fn remove_object_to_be_destroyed(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    book_keeper_answer(vm, ctx)
}

/// The book keeper's answer: 1 for a defined `this`, else 0 (the efun's return slot).
fn book_keeper_answer(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let defined = Call::new(ctx).this() != u32::MAX;
    Call::new(ctx).ret(2, u32::from(defined), tag::BOOL);
    InterpStatus::Success
}

/// The weenie `this` is the interface of, or `None` (after crashing) for an undefined `this`.
pub(super) fn this_weenie(vm: &mut Vm, ctx: &mut Context) -> Result<u64, InterpStatus> {
    let this = Call::new(ctx).this();
    if this == u32::MAX {
        return Err(vm.bad_this(this));
    }
    Ok(vm.owners.get(&this).copied().unwrap_or(0))
}

/// `CWInterface::IsThePlayer`: whether this weenie is `CObjectMaint::GetPlayerID`.
fn is_the_player(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let is = u32::from(id != 0 && id == vm.player);
    Call::new(ctx).ret(2, is, tag::BOOL);
    InterpStatus::Success
}

/// `CWInterface::GetInternalWeenieObject(iid)`: the script instance of the weenie `iid`
/// (`CObjectMaint::GetWeenieObject`), or the undefined reference when it has none.
fn get_internal_weenie_object(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    if let Err(status) = this_weenie(vm, ctx) {
        return status;
    }
    let id = word64(ctx, 3);
    let r = vm
        .owners
        .iter()
        .find(|(_, &owner)| owner == id && id != 0)
        .map_or(u32::MAX, |(&r, _)| r);
    Call::new(ctx).ret(4, r, tag::REF);
    InterpStatus::Success
}
