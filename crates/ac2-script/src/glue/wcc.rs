//! `gwccinternal`: the string-building efuns the WSL compiler emits for `string + value`
//! (`WCCEfuns.cpp`: `_WCC_strcat_*`, `ToHex`, `ToLHex`, `ToIID`). Each makes a new `wpstring`.
//!
//! `_WCC_strcat_X(string, value, reverse)`: the string then the formatted value, or the value
//! first when `reverse` is set; `_WCC_strcat_str` asserts `!reverse`.

use super::format::{msvc_double, msvc_float};
use super::Call;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0074_0001, strcat_str);
    efuns.register(0x0074_0002, strcat_int);
    efuns.register(0x0074_0003, strcat_uint);
    efuns.register(0x0074_0004, strcat_float);
    efuns.register(0x0074_0005, strcat_lint);
    efuns.register(0x0074_0006, strcat_ulint);
    efuns.register(0x0074_0007, strcat_lfloat);
    efuns.register(0x0074_0008, to_hex);
    efuns.register(0x0074_0009, to_lhex);
    efuns.register(0x0074_000A, to_iid);
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// Joins the string argument (at `string_slot`) and `value`, honouring `reverse`, into a new
/// `wpstring` returned at `ret_slot`.
fn concat(
    vm: &mut Vm,
    ctx: &mut Context,
    string_slot: usize,
    value: &str,
    reverse_slot: usize,
    ret_slot: usize,
) -> InterpStatus {
    let call = Call::new(ctx);
    let base = vm.wstring(call.word(string_slot)).unwrap_or(&[]).to_vec();
    let reverse = call.word(reverse_slot) != 0;
    let value = utf16(value);
    let joined = if reverse {
        [value, base].concat()
    } else {
        [base, value].concat()
    };
    let reference = vm.new_wstring(joined);
    Call::new(ctx).ret(ret_slot, reference, tag::REF);
    InterpStatus::Success
}

/// `_WCC_strcat_str`: `a + b`.
fn strcat_str(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let a = vm.wstring(call.word(2)).unwrap_or(&[]).to_vec();
    let b = vm.wstring(call.word(3)).unwrap_or(&[]).to_vec();
    let reference = vm.new_wstring([a, b].concat());
    Call::new(ctx).ret(5, reference, tag::REF);
    InterpStatus::Success
}

/// `_WCC_strcat_int` ("%ld").
fn strcat_int(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = (Call::new(ctx).word(3) as i32).to_string();
    concat(vm, ctx, 2, &value, 4, 5)
}

/// `_WCC_strcat_uint` ("%lu").
fn strcat_uint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = Call::new(ctx).word(3).to_string();
    concat(vm, ctx, 2, &value, 4, 5)
}

/// `_WCC_strcat_float` ("%f").
fn strcat_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = msvc_float(f32::from_bits(Call::new(ctx).word(3)));
    concat(vm, ctx, 2, &value, 4, 5)
}

/// `_WCC_strcat_lint` ("%I64d"): the 64-bit value takes two slots.
fn strcat_lint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = (Call::new(ctx).word64(4) as i64).to_string();
    concat(vm, ctx, 2, &value, 5, 6)
}

/// `_WCC_strcat_ulint` ("%I64u").
fn strcat_ulint(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = Call::new(ctx).word64(4).to_string();
    concat(vm, ctx, 2, &value, 5, 6)
}

/// `_WCC_strcat_lfloat` ("%lf").
fn strcat_lfloat(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = msvc_double(f64::from_bits(Call::new(ctx).word64(4)));
    concat(vm, ctx, 2, &value, 5, 6)
}

/// `ToHex` / `ToSHex` ("0x%08X").
fn to_hex(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let text = format!("0x{:08X}", Call::new(ctx).word(2));
    let reference = vm.new_wstring(utf16(&text));
    Call::new(ctx).ret(3, reference, tag::REF);
    InterpStatus::Success
}

/// `ToLHex` ("0x%016I64X").
fn to_lhex(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let text = format!("0x{:016X}", Call::new(ctx).word64(3));
    let reference = vm.new_wstring(utf16(&text));
    Call::new(ctx).ret(4, reference, tag::REF);
    InterpStatus::Success
}

/// `ToIID` ("%04X:%04X:%04X:%04X", most significant group first).
fn to_iid(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = Call::new(ctx).word64(3);
    let text = format!(
        "{:04X}:{:04X}:{:04X}:{:04X}",
        (v >> 48) & 0xFFFF,
        (v >> 32) & 0xFFFF,
        (v >> 16) & 0xFFFF,
        v & 0xFFFF
    );
    let reference = vm.new_wstring(utf16(&text));
    Call::new(ctx).ret(4, reference, tag::REF);
    InterpStatus::Success
}
