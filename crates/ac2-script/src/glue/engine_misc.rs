//! Small engine glue: `gTimer`, `gdebug::PrintString`, `gCWDistributor`'s process queries and
//! the chat shell registration of `gCWInterface` (`AddShellCmd`, `AddShellHelp` and their
//! string-id forms).

use super::string_info::STRING_TABLE_ENUM;
use super::{word, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x006C_0002, get_time);
    efuns.register(0x001C_0005, print_string);
    efuns.register(0x007E_0001, get_process_id);
    efuns.register(0x007E_0002, get_my_package_id);
    efuns.register(0x007F_003D, |vm, c| add_shell(vm, c, false, false));
    efuns.register(0x007F_003C, |vm, c| add_shell(vm, c, false, true));
    efuns.register(0x007F_003F, |vm, c| add_shell(vm, c, true, false));
    efuns.register(0x007F_003E, |vm, c| add_shell(vm, c, true, true));
}

/// `gTimer::CallEFun_get_time`: `Timer::cur_time`, a double in two slots (low word in slot 3).
fn get_time(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let bits = vm.cur_time.to_bits();
    let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(3, bits as u32, tag::BOOL | tag::LONG);
    call.ret(2, (bits >> 32) as u32, high_tag);
    InterpStatus::Success
}

/// `gdebug::CallEFun_PrintString(wpstring)` (`PrintString`): retail prints `interp:  <text>` on
/// the debug console; here it goes to the log.
fn print_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let text = vm.wstring(word(ctx, 2)).map(<[u16]>::to_vec);
    match text {
        Some(text) => {
            log::info!("interp:  {}", String::from_utf16_lossy(&text));
            InterpStatus::Success
        }
        None => vm.bad_this(word(ctx, 2)),
    }
}

/// `gCWDistributor::CallEFun_GetProcessID` (`CWDistributor::GetProcessID`): the object's process
/// id. An instance without a weenie process answers 0.
fn get_process_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    if this == u32::MAX {
        return vm.bad_this(this);
    }
    let id = vm.process_ids.get(&this).copied().unwrap_or(0);
    Call::new(ctx).ret(2, id, INT_RESULT);
    InterpStatus::Success
}

/// `gCWDistributor::CallEFun_GetMyPackageID`: the instance's package id (vtable slot 1).
fn get_my_package_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    if this == u32::MAX {
        return vm.bad_this(this);
    }
    let pid = vm.package_id(this).unwrap_or(0);
    Call::new(ctx).ret(2, pid, INT_RESULT);
    InterpStatus::Success
}

/// `gCWInterface::CallEFun_AddShellCmd(cmd, function, final)` and `AddShellHelp(cmd,
/// function)`, and the `ByStringID(token, table enum, function[, final])` forms that read the
/// command text from a string table (`CWInterface::AddShellCmdByStringID`). The chat shell exists
/// in the client, so the answer is 1 when every word registered.
fn add_shell(vm: &mut Vm, ctx: &mut Context, help: bool, by_id: bool) -> InterpStatus {
    let call = Call::new(ctx);
    let this = call.this();
    if this == u32::MAX {
        return vm.bad_this(this);
    }
    let (text, function, result_slot) = if by_id {
        let (token, table, function) = (call.word(2), call.word(3), call.word(4));
        let did = vm.data().enum_did(table, STRING_TABLE_ENUM).unwrap_or(0);
        let info = crate::native::StringInfo {
            string_id: token,
            table_id: did,
            ..Default::default()
        };
        let (text, _) = super::string_text::inq_string_internal(vm.data(), &info, false);
        (text, function, if help { 5 } else { 6 })
    } else {
        let (cmd, function) = (call.word(2), call.word(3));
        match vm.native(cmd) {
            Some(NativeObject::WString(text)) => (text.clone(), function, if help { 4 } else { 5 }),
            _ => return vm.bad_this(cmd),
        }
    };
    let words = text
        .split(|&c| matches!(c, 0x3B | 0x3A | 0x2C))
        .filter(|w| !w.is_empty())
        .count();
    let added = vm.shell.add(help, &text, function);
    Call::new(ctx).ret(result_slot, u32::from(added == words), INT_RESULT);
    InterpStatus::Success
}
