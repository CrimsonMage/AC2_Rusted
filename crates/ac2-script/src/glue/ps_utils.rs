//! `gPSUtils` number tests (`portal/shared/src/interp/glue/gpsutils.cpp`, `PSUtils::is_int32`,
//! `is_uint32`, `is_int64`, `is_uint64`, `is_double`): whether a string is wholly one C number.
//! Each runs the CRT converter (`wcstol`, `wcstoul`, `_wcstoint64`, `_wcstouint64` in base 0,
//! `wcstod`) and answers true when it stopped at the terminator without `ERANGE`. Text that
//! converts nothing stops where it started, so only the empty string passes then. The path
//! helpers of the same class (`get_cwdW`, `cleanup_directory_nameW`, `remove_extensionW`,
//! `strip_directoryW`) serve the build tools and are not bound.

use super::cparse::{float_end, int_prefix, IntPrefix};
use super::{Call, INT_RESULT};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x005D_0003, |vm, ctx| test(vm, ctx, Kind::I32));
    efuns.register(0x005D_0004, |vm, ctx| test(vm, ctx, Kind::U32));
    efuns.register(0x005D_0005, |vm, ctx| test(vm, ctx, Kind::I64));
    efuns.register(0x005D_0006, |vm, ctx| test(vm, ctx, Kind::U64));
    efuns.register(0x005D_0007, |vm, ctx| test(vm, ctx, Kind::Double));
}

#[derive(Clone, Copy)]
enum Kind {
    I32,
    U32,
    I64,
    U64,
    Double,
}

/// Whether `text` is wholly a number of `kind`.
fn is_number(text: &[u16], kind: Kind) -> bool {
    match kind {
        Kind::Double => {
            float_end(text).is_some_and(|(end, in_range)| end == text.len() && in_range)
        }
        _ => {
            let IntPrefix {
                end,
                negative,
                magnitude,
                ..
            } = int_prefix(text);
            if end != text.len() {
                return false;
            }
            match kind {
                Kind::I32 => magnitude <= if negative { 1 << 31 } else { (1 << 31) - 1 },
                Kind::U32 => magnitude <= u128::from(u32::MAX),
                Kind::I64 => magnitude <= if negative { 1 << 63 } else { (1 << 63) - 1 },
                _ => magnitude <= u128::from(u64::MAX),
            }
        }
    }
}

/// `gPSUtils::CallEFun_is_*(string)`: the flag in word 3.
fn test(vm: &mut Vm, ctx: &mut Context, kind: Kind) -> InterpStatus {
    let reference = Call::new(ctx).word(2);
    let Some(text) = vm.wstring(reference).map(<[u16]>::to_vec) else {
        return vm.bad_this(reference);
    };
    // The CRT sees the text up to its first NUL.
    let text = match text.iter().position(|&c| c == 0) {
        Some(n) => text[..n].to_vec(),
        None => text,
    };
    Call::new(ctx).ret(3, u32::from(is_number(&text, kind)), INT_RESULT);
    InterpStatus::Success
}
