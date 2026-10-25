//! `gReservedIDs`: what an instance id says about its object (`IIDUtils::Verify*`, ac2-net's
//! `iid`). The id is the 64-bit argument (low word deeper); the `BOOL` result is below it.

use super::{Call, INT_RESULT};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0063_0001, is_player_id);
    efuns.register(0x0063_0005, is_static_id);
    efuns.register(0x0063_0006, is_dynamic_id);
}

fn verify(ctx: &mut Context, check: fn(u64) -> bool) -> InterpStatus {
    let id = Call::new(ctx).word64(3);
    Call::new(ctx).ret(4, u32::from(check(id)), INT_RESULT);
    InterpStatus::Success
}

/// `gReservedIDs::CallEFun_IsPlayerID` (`IIDUtils::VerifyPlayer`).
fn is_player_id(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    verify(ctx, ac2_net::iid::is_player)
}

/// `gReservedIDs::CallEFun_IsStaticID` (`IIDUtils::VerifyStatic`).
fn is_static_id(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    verify(ctx, ac2_net::iid::is_static)
}

/// `gReservedIDs::CallEFun_IsDynamicID` (`IIDUtils::VerifyDynamic`).
fn is_dynamic_id(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    verify(ctx, ac2_net::iid::is_dynamic)
}
