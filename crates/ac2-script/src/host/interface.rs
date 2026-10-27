//! `gCWInterface` efuns on a weenie's own interface (`CWInterface`, the `this` of a weenie
//! script): which object it is and the objects it can reach. Each wrapper crashes on an
//! undefined `this` ("index != Undef_ReferenceID").

use crate::glue::{word64, Call};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x007F_000A, get_internal_weenie_object);
    // `GetExternalWeenieObject` (admin 0x004BB580, unnamed in the efun table): the same code;
    // Legions binds 0x7F000A and 0x7F000B to one function (0x004B0680).
    efuns.register(0x007F_000B, get_internal_weenie_object);
    efuns.register(0x007F_0040, is_the_player);
    efuns.register(0x007F_0052, process_text);
    // `gCWBookKeeper::CallEFun_{Add,Remove}ObjectToBeDestroyed` (the weenie's book keeper, `this`):
    // `CWBookKeeper` -> `CBookKeep::{Add,Remove}ObjectToBeDestroyed`, which answer 1 whenever
    // `CObjectMaint` exists (it always does here). The destroy list itself is the client's object
    // maintenance, which this client does not keep (see the catalog).
    efuns.register(0x007D_001E, add_object_to_be_destroyed);
    efuns.register(0x007D_001F, remove_object_to_be_destroyed);
    efuns.register(0x0080_0057, set_no_draw_physics);
    efuns.register(0x0080_002C, stop_all_camera_fx);
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
