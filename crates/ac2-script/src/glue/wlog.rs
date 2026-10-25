//! `gWLogSystem`: the script log masks (`WLogSystem`) and `DLogMsg`.
//!
//! `DLogMsg` (`PSUtils::make_log_string`, `ArgumentParser::FinishOutputText`) is retail's
//! debug output; the port sends it to the `log` crate at debug level, target `wsl`. The seven
//! `DLogMsg` efun ids are one glue function reading the string in word 2.

use super::{Call, INT_RESULT};
use crate::vm::{Context, EFuns, InterpStatus, LogTypes, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0075_0001, is_file_logging);
    efuns.register(0x0075_0002, is_print_logging);
    efuns.register(0x0075_0003, |vm, ctx| set_mask(vm, ctx, |m, v| m.file |= v));
    efuns.register(0x0075_0004, |vm, ctx| {
        set_mask(vm, ctx, |m, v| m.print |= v)
    });
    efuns.register(0x0075_0005, |vm, ctx| set_mask(vm, ctx, |m, v| m.file = v));
    efuns.register(0x0075_0006, |vm, ctx| set_mask(vm, ctx, |m, v| m.print = v));
    efuns.register(0x0075_0007, |vm, ctx| {
        set_mask(vm, ctx, |m, v| m.file &= !v)
    });
    efuns.register(0x0075_0008, |vm, ctx| {
        set_mask(vm, ctx, |m, v| m.print &= !v)
    });
    efuns.register(0x0075_0009, |vm, ctx| get_mask(vm, ctx, vm.log_types.file));
    efuns.register(0x0075_000A, |vm, ctx| get_mask(vm, ctx, vm.log_types.print));
    for n in 0x0B..=0x11 {
        efuns.register(0x0075_0000 | n, dlog_msg);
    }
}

/// `WLogSystem::AddFileLog`, `SetPrintLog`, ...: the mask in word 2.
fn set_mask(vm: &mut Vm, ctx: &mut Context, f: fn(&mut LogTypes, u32)) -> InterpStatus {
    let value = Call::new(ctx).word(2);
    f(&mut vm.log_types, value);
    InterpStatus::Success
}

fn get_mask(_vm: &mut Vm, ctx: &mut Context, value: u32) -> InterpStatus {
    Call::new(ctx).ret(2, value, INT_RESULT);
    InterpStatus::Success
}

/// `WLogSystem::IsFileLogging`: type 4 always logs to file.
fn is_file_logging(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let t = Call::new(ctx).word(2);
    let on = t == 4 || vm.log_types.file & t == t;
    Call::new(ctx).ret(3, u32::from(on), INT_RESULT);
    InterpStatus::Success
}

/// `WLogSystem::IsPrintLogging`.
fn is_print_logging(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let t = Call::new(ctx).word(2);
    let on = vm.log_types.print & t == t;
    Call::new(ctx).ret(3, u32::from(on), INT_RESULT);
    InterpStatus::Success
}

/// `gWLogSystem::CallEFun_DLogMsg(wpstring)`.
fn dlog_msg(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let text = vm
        .wstring(Call::new(ctx).word(2))
        .map(String::from_utf16_lossy);
    log::debug!(target: "wsl", "{}", text.unwrap_or_default());
    InterpStatus::Success
}
