//! `gVM`: the VM queries scripts make (`portal/shared/src/interp/glue/gVM.cpp`).
//!
//! The collector and the admin query/trace efuns are not ported: the GC controls do nothing,
//! `DeleteAllUnUsedReferences` reports 0 and the queries return empty results.

use super::{narrow, Call, INT_RESULT};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0072_0001, get_context_id);
    efuns.register(0x0072_0002, get_singleton_package_w);
    efuns.register(0x0072_0003, get_singleton_package_by_id);
    efuns.register(0x0072_0004, get_singleton_package_by_id);
    efuns.register(0x0072_0005, get_package_id);
    efuns.register(0x0072_0006, get_package_id_by_ref);
    efuns.register(0x0072_0007, is_derived_from);
    efuns.register(0x0072_000D, delete_all_unused_references);
    efuns.register(0x0072_000E, get_package_name_by_pkg_id_w);
    efuns.register(0x0072_000F, get_stack_trace);
    efuns.register(0x0072_0010, no_op);
    efuns.register(0x0072_0011, no_op);
    efuns.register(0x0072_0012, no_op);
    efuns.register(0x0072_0014, get_instance_reference);
    efuns.register(0x0072_0017, is_instance_of);
    efuns.register(0x0072_0018, get_package_reference);
    efuns.register(0x0072_0019, get_data_id);
}

/// `GarbageCollect`, `SetActiveGC`, `SetAggressiveGCDeletion`.
fn no_op(_vm: &mut Vm, _ctx: &mut Context) -> InterpStatus {
    InterpStatus::Success
}

/// `gVM::CallEFun_GetContextID`.
fn get_context_id(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let cid = ctx.cid;
    Call::new(ctx).ret(2, cid, INT_RESULT);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetSingletonPackageByID` (`VM::GetSingletonPackageByID`).
fn get_singleton_package_by_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let pid = call.word(2);
    let reference = if pid == u32::MAX {
        u32::MAX
    } else {
        vm.singleton_package(pid)
    };
    call.ret(3, reference, tag::REF);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetSingletonPackageW` (`VM::GetSingletonPackageW`).
fn get_singleton_package_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let name = vm.wstring(call.word(2)).map(narrow).unwrap_or_default();
    let reference = vm
        .program
        .package_id(&String::from_utf8_lossy(&name))
        .filter(|&pid| pid != u32::MAX)
        .map_or(u32::MAX, |pid| vm.singleton_package(pid));
    call.ret(3, reference, tag::REF);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetPackageID` (`VM::GetPackageID(wide name)`).
fn get_package_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let name = vm.wstring(call.word(2)).map(narrow).unwrap_or_default();
    let pid = vm
        .program
        .package_id(&String::from_utf8_lossy(&name))
        .unwrap_or(u32::MAX);
    call.ret(3, pid, INT_RESULT);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetPackageIDByRef` (`ReferenceTable::GetPackageID`).
fn get_package_id_by_ref(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let reference = call.word(2);
    match vm.package_id(reference) {
        Ok(pid) => {
            call.ret(3, pid, INT_RESULT);
            InterpStatus::Success
        }
        Err(fault) => {
            vm.last_fault = Some(fault);
            InterpStatus::Crash
        }
    }
}

/// `gVM::CallEFun_IsDerivedFrom` (`ByteStream::IsDerivedFrom(pkg, base)`).
fn is_derived_from(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let (pkg, base) = (call.word(2), call.word(3));
    let derived = vm.program.is_derived_from(pkg, base);
    call.ret(4, u32::from(derived), INT_RESULT);
    InterpStatus::Success
}

/// `gVM::CallEFun_IsInstanceOf` (`VM::IsInstanceOf`).
fn is_instance_of(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let (reference, pkg) = (call.word(2), call.word(3));
    let result = reference != u32::MAX
        && vm
            .package_id(reference)
            .is_ok_and(|own| vm.program.is_derived_from(own, pkg));
    call.ret(4, u32::from(result), INT_RESULT);
    InterpStatus::Success
}

/// `gVM::CallEFun_DeleteAllUnUsedReferences`: no collector, nothing deleted.
fn delete_all_unused_references(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    Call::new(ctx).ret(2, 0, INT_RESULT);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetPackageNameByPkgIDW` (`VM::GetPackageNameByPkgIDW`, via the resolver's
/// reverse map).
fn get_package_name_by_pkg_id_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let pid = Call::new(ctx).word(2);
    let name: Vec<u16> = vm
        .program
        .package_names
        .iter()
        .find(|(_, id)| *id == pid)
        .map(|(n, _)| n.iter().map(|&b| u16::from(b)).collect())
        .unwrap_or_default();
    let reference = vm.new_wstring(name);
    Call::new(ctx).ret(3, reference, tag::REF);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetStackTrace`: the debugger's trace text is not ported; an empty string.
fn get_stack_trace(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let reference = vm.new_wstring(Vec::new());
    Call::new(ctx).ret(2, reference, tag::REF);
    InterpStatus::Success
}

/// `gVM::CallEFun_WSL_EFun_GetInstanceReference`: the running process's instance state.
fn get_instance_reference(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let instance = ctx.instance.unwrap_or(u32::MAX);
    Call::new(ctx).ret(2, instance, tag::REF);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetPackageReference` (`VM::GetPackageReference`): the singleton of WState
/// `did` if that state's package derives from `base`.
fn get_package_reference(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let (did, base) = (call.word(2), call.word(3));
    let mut result = u32::MAX;
    if did != 0 && base != u32::MAX {
        let pid = vm.state_package(did);
        if pid.is_some_and(|pid| vm.program.is_derived_from(pid, base)) {
            result = vm.refs.singleton(did);
        }
    }
    call.ret(4, result, tag::REF);
    InterpStatus::Success
}

/// `gVM::CallEFun_GetDataID`: a heap object's `m_pkgStateID`, else 0.
fn get_data_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let mut call = Call::new(ctx);
    let reference = call.word(2);
    let did = vm.refs.heap(reference).map_or(0, |o| o.pkg_state_id);
    call.ret(3, did, INT_RESULT);
    InterpStatus::Success
}
