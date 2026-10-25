//! `gwpstring`: the script string type, a `PStringBase<unsigned short>` native
//! (`portal/shared/src/interp/glue/gwpstring.cpp`).
//!
//! Retail quirks kept: `length` returns `PSRefBuffer::m_len`, which counts the terminator;
//! `empty` and the comparisons write only the low byte of their result slot; `to_int64` tags
//! only the first result slot.

use super::cparse::{self, to_lower, to_upper, wcsicmp, wcsstr};
use super::format::{msvc_double, msvc_float};
use super::{Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use ac2_core::pstring;

pub(super) fn register(efuns: &mut EFuns) {
    let table: [(u32, crate::vm::EFunFn); 34] = [
        (0x01, hash),
        (0x02, empty),
        (0x03, length),
        (0x04, clear),
        (0x05, set),
        (0x06, append_string),
        (0x07, append_bool),
        (0x08, append_int32),
        (0x09, append_hex),
        (0x0A, append_uint32),
        (0x0B, append_hex),
        (0x0C, append_float),
        (0x0D, find_substring),
        (0x0E, substring),
        (0x0F, trim),
        (0x10, cmp),
        (0x11, eq),
        (0x12, ne),
        (0x13, lt),
        (0x14, gt),
        (0x15, le),
        (0x16, ge),
        (0x17, to_int32),
        (0x18, to_uint32),
        (0x19, to_int64),
        (0x1A, to_uint64),
        (0x1B, to_float),
        (0x1C, encrypt),
        (0x1D, decrypt),
        (0x1E, toupper),
        (0x1F, tolower),
        (0x20, replace),
        (0x21, append_int64),
        (0x23, append_double),
    ];
    for (n, f) in table {
        efuns.register(0x0076_0000 | n, f);
    }
}

/// Runs `f` on `this`'s characters, crashing on a null or non-string `this`.
fn with_this(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut Vec<u16>, &mut Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::WString(s)) => {
            f(s, ctx);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `this` and the string argument at `n`, copied out.
fn this_and_arg(vm: &Vm, ctx: &mut Context, n: usize) -> Option<(Vec<u16>, Vec<u16>)> {
    let call = Call::new(ctx);
    let this = vm.wstring(call.this())?.to_vec();
    let arg = vm.wstring(call.word(n)).unwrap_or(&[]).to_vec();
    Some((this, arg))
}

/// Writes the low byte of slot `n`, keeping the rest (a C++ `bool` store).
fn ret_byte(ctx: &mut Context, n: usize, value: bool) {
    let old = ctx.arg(n).unwrap_or(0);
    ctx.set_arg(n, (old & !0xFF) | u32::from(value), INT_RESULT);
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// `PStringBase<unsigned short>::compute_hash`.
pub(crate) fn compute_hash(s: &[u16]) -> u32 {
    let mut h: u32 = 0;
    for &c in s.iter().take_while(|&&c| c != 0) {
        h = h.wrapping_mul(16).wrapping_add(u32::from(c));
        if h & 0xF000_0000 != 0 {
            h = ((h & 0xF000_0000) >> 24 ^ h) & 0x0FFF_FFFF;
        }
    }
    if h == u32::MAX {
        0xFFFF_FFFE
    } else {
        h
    }
}

fn hash(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| {
        let h = compute_hash(s);
        Call::new(ctx).ret(2, h, INT_RESULT);
    })
}

fn empty(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| ret_byte(ctx, 2, s.is_empty()))
}

fn length(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| {
        Call::new(ctx).ret(2, s.len() as u32 + 1, INT_RESULT);
    })
}

fn clear(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, _| s.clear())
}

fn set(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((_, arg)) = this_and_arg(vm, ctx, 2) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    with_this(vm, ctx, |s, _| *s = arg)
}

fn append_string(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((_, arg)) = this_and_arg(vm, ctx, 2) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    with_this(vm, ctx, |s, _| s.extend(arg))
}

fn append_text(vm: &mut Vm, ctx: &mut Context, text: String) -> InterpStatus {
    with_this(vm, ctx, |s, _| s.extend(utf16(&text)))
}

/// `append_int32` of the argument's low byte ("%ld").
fn append_bool(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let byte = Call::new(ctx).word(2) & 0xFF;
    append_text(vm, ctx, byte.to_string())
}

fn append_int32(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = Call::new(ctx).word(2) as i32;
    append_text(vm, ctx, v.to_string())
}

/// `append_int32_hex` ("0x%08X"), also used for `append_uint32_hex`.
fn append_hex(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = Call::new(ctx).word(2);
    append_text(vm, ctx, format!("0x{v:08X}"))
}

fn append_uint32(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = Call::new(ctx).word(2);
    append_text(vm, ctx, v.to_string())
}

fn append_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = f32::from_bits(Call::new(ctx).word(2));
    append_text(vm, ctx, msvc_float(v))
}

fn append_int64(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = Call::new(ctx).word64(3) as i64;
    append_text(vm, ctx, v.to_string())
}

fn append_double(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let v = f64::from_bits(Call::new(ctx).word64(3));
    append_text(vm, ctx, msvc_double(v))
}

fn find_substring(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((this, arg)) = this_and_arg(vm, ctx, 2) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let at = wcsstr(&this, &arg).map_or(u32::MAX, |i| i as u32);
    Call::new(ctx).ret(3, at, INT_RESULT);
    InterpStatus::Success
}

/// `PStringBase<unsigned short>::substring(start, end)`: characters `start..=end`, `end`
/// clamped to the last character; empty when `start` is past the end.
fn substring(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (start, end) = (call.word(2) as usize, call.word(3) as usize);
    let Some(this) = vm.wstring(call.this()).map(<[u16]>::to_vec) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let n = this.len();
    let piece = match start < n {
        true if end.min(n - 1) >= start => this[start..=end.min(n - 1)].to_vec(),
        _ => Vec::new(),
    };
    let reference = vm.new_wstring(piece);
    Call::new(ctx).ret(4, reference, tag::REF);
    InterpStatus::Success
}

/// `trim(true, true, whitespace_string)`: " \n\r\t\f" from both ends.
fn trim(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    const WS: [u16; 5] = [0x20, 0x0A, 0x0D, 0x09, 0x0C];
    with_this(vm, ctx, |s, _| {
        let start = s.iter().position(|c| !WS.contains(c)).unwrap_or(s.len());
        let end = s
            .iter()
            .rposition(|c| !WS.contains(c))
            .map_or(start, |e| e + 1);
        *s = s[start..end.max(start)].to_vec();
    })
}

/// The comparisons: the flag (argument 2's low byte) selects case-sensitive `wcscmp` over
/// `_wcsicmp`.
fn compare(vm: &mut Vm, ctx: &mut Context, test: fn(i32) -> bool) -> InterpStatus {
    let Some((this, arg)) = this_and_arg(vm, ctx, 2) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let sensitive = Call::new(ctx).word(3) & 0xFF != 0;
    let order = if sensitive {
        cparse::wcscmp(&this, &arg)
    } else {
        wcsicmp(&this, &arg)
    };
    ret_byte(ctx, 4, test(order));
    InterpStatus::Success
}

fn cmp(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some((this, arg)) = this_and_arg(vm, ctx, 2) else {
        return vm.bad_this(Call::new(ctx).this());
    };
    let sensitive = Call::new(ctx).word(3) & 0xFF != 0;
    let order = if sensitive {
        cparse::wcscmp(&this, &arg)
    } else {
        wcsicmp(&this, &arg)
    };
    Call::new(ctx).ret(4, order as u32, INT_RESULT);
    InterpStatus::Success
}

fn eq(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, |o| o == 0)
}
fn ne(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, |o| o != 0)
}
fn lt(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, |o| o < 0)
}
fn gt(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, |o| o > 0)
}
fn le(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, |o| o <= 0)
}
fn ge(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    compare(vm, ctx, |o| o >= 0)
}

fn to_int32(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| {
        let v = cparse::wcstol(s);
        Call::new(ctx).ret(2, v as u32, INT_RESULT);
    })
}

fn to_uint32(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| {
        let v = cparse::wcstoul(s);
        Call::new(ctx).ret(2, v, INT_RESULT);
    })
}

/// 64-bit results land at `fp - 3` (low) and `fp - 2`; only the low slot is tagged.
fn ret_long(ctx: &mut Context, v: u64) {
    let high_tag = ctx.fp.checked_sub(2).map_or(0, |i| ctx.tags[i]);
    let mut call = Call::new(ctx);
    call.ret(3, v as u32, INT_RESULT | tag::LONG);
    call.ret(2, (v >> 32) as u32, high_tag);
}

fn to_int64(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| ret_long(ctx, cparse::wcstoi64(s) as u64))
}

fn to_uint64(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| ret_long(ctx, cparse::wcstoui64(s)))
}

/// `wcstod`, stored as a `float`.
fn to_float(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, ctx| {
        let v = cparse::wcstod(s) as f32;
        Call::new(ctx).ret(2, v.to_bits(), INT_RESULT);
    })
}

fn bytes_of(s: &[u16]) -> Vec<u8> {
    s.iter().flat_map(|c| c.to_le_bytes()).collect()
}

fn from_bytes(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect()
}

/// `PStringBase<unsigned short>::encrypt`: nibble swap, then the word XOR.
fn encrypt(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, _| {
        let mut b = bytes_of(s);
        pstring::munge(&mut b);
        *s = from_bytes(&b);
    })
}

/// `PStringBase<unsigned short>::decrypt`: the word XOR, then nibble swap.
fn decrypt(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, _| {
        let mut b = bytes_of(s);
        pstring::unmunge(&mut b);
        *s = from_bytes(&b);
    })
}

fn toupper(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, _| s.iter_mut().for_each(|c| *c = to_upper(*c)))
}

fn tolower(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_this(vm, ctx, |s, _| s.iter_mut().for_each(|c| *c = to_lower(*c)))
}

/// `PStringBase<unsigned short>::replace(search, with)`: every non-overlapping occurrence,
/// returning the count. (An empty search string would never terminate in retail; here it
/// replaces nothing.)
fn replace(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let Some(this) = vm.wstring(call.this()).map(<[u16]>::to_vec) else {
        return vm.bad_this(call.this());
    };
    let search = vm.wstring(call.word(2)).unwrap_or(&[]).to_vec();
    let with = vm.wstring(call.word(3)).unwrap_or(&[]).to_vec();
    let mut out = Vec::with_capacity(this.len());
    let mut count = 0u32;
    let mut i = 0;
    while !search.is_empty() && i + search.len() <= this.len() {
        if this[i..i + search.len()] == search[..] {
            out.extend_from_slice(&with);
            i += search.len();
            count += 1;
        } else {
            out.push(this[i]);
            i += 1;
        }
    }
    out.extend_from_slice(&this[i.min(this.len())..]);
    if count > 0 {
        if let Some(NativeObject::WString(s)) = vm.native_mut(Call::new(ctx).this()) {
            *s = out;
        }
    }
    Call::new(ctx).ret(4, count, INT_RESULT);
    InterpStatus::Success
}
