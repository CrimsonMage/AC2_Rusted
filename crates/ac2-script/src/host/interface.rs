//! `gCWInterface` efuns on a weenie's own interface (`CWInterface`, the `this` of a weenie
//! script): which object it is and the objects it can reach. Each wrapper crashes on an
//! undefined `this` ("index != Undef_ReferenceID").

use crate::glue::{word64, Call};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x007F_000A, get_internal_weenie_object);
    efuns.register(0x007F_0040, is_the_player);
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
