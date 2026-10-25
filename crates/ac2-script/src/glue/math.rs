//! `gMath` and `gVector` (`gMath.cpp`, `gVector.cpp`).
//!
//! A double or 64-bit argument takes two slots, high word first (the low word is deeper); a
//! 64-bit result is written the same way with tag 0x84 on its low slot only. Float results
//! are tagged 4 like ints. Retail's x87 ran at 53-bit precision, so f64 arithmetic matches it;
//! the transcendental instructions (`fsin`, `fpatan`, `f2xm1`) and the CRT's `acos`, `asin`,
//! `fmod`, `pow` are Rust's `f64` functions here.

use super::format::msvc_fixed;
use super::{Call, INT_RESULT};
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let f: Option<EFunFn> = match name.split_once("::CallEFun_") {
            Some(("gMath", m)) => math_method(m),
            Some(("gVector", m)) => vector_method(m),
            _ => None,
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

/// The 64-bit argument whose high word is slot `n`.
fn arg64(ctx: &Context, n: usize) -> u64 {
    super::word64(ctx, n + 1)
}

fn argf(ctx: &Context, n: usize) -> f64 {
    f64::from_bits(arg64(ctx, n))
}

/// A 64-bit result whose high word is slot `n`.
fn ret64(ctx: &mut Context, n: usize, v: u64) -> InterpStatus {
    let high_tag = ctx.arg_tag(n).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(n + 1, v as u32, tag::BOOL | tag::LONG);
    call.ret(n, (v >> 32) as u32, high_tag);
    InterpStatus::Success
}

fn unary(ctx: &mut Context, f: fn(f64) -> f64) -> InterpStatus {
    let x = argf(ctx, 2);
    ret64(ctx, 4, f(x).to_bits())
}

fn binary(ctx: &mut Context, f: fn(f64, f64) -> f64) -> InterpStatus {
    let (a, b) = (argf(ctx, 2), argf(ctx, 4));
    ret64(ctx, 6, f(a, b).to_bits())
}

fn ints(ctx: &mut Context, f: fn(i32, i32) -> i32) -> InterpStatus {
    let (a, b) = (super::word(ctx, 2) as i32, super::word(ctx, 3) as i32);
    Call::new(ctx).ret(4, f(a, b) as u32, INT_RESULT);
    InterpStatus::Success
}

fn floats(ctx: &mut Context, f: fn(f32, f32) -> f32) -> InterpStatus {
    let a = f32::from_bits(super::word(ctx, 2));
    let b = f32::from_bits(super::word(ctx, 3));
    Call::new(ctx).ret(4, f(a, b).to_bits(), INT_RESULT);
    InterpStatus::Success
}

fn longs(ctx: &mut Context, f: fn(u64, u64) -> u64) -> InterpStatus {
    let (a, b) = (arg64(ctx, 2), arg64(ctx, 4));
    ret64(ctx, 6, f(a, b))
}

/// `MapToUnitInterval(x, scale)`: `atan(x / scale) * 2/pi`, 0 when `scale` is 0.
fn map_to_unit_interval(_: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (x, scale) = (argf(ctx, 2), argf(ctx, 4));
    let v = if scale != 0.0 {
        ((1.0 / scale) * x).atan() * 0.636_619_772_367_584
    } else {
        0.0
    };
    ret64(ctx, 6, v.to_bits())
}

fn math_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "sin" => |_, c| unary(c, f64::sin),
        "cos" => |_, c| unary(c, f64::cos),
        "tan" => |_, c| unary(c, f64::tan),
        "acos" => |_, c| unary(c, f64::acos),
        "asin" => |_, c| unary(c, f64::asin),
        "atan" => |_, c| unary(c, f64::atan),
        "ceil" => |_, c| unary(c, f64::ceil),
        "floor" => |_, c| unary(c, f64::floor),
        "fabs" => |_, c| unary(c, f64::abs),
        "log" => |_, c| unary(c, f64::ln),
        "log10" => |_, c| unary(c, f64::log10),
        "exp" => |_, c| unary(c, f64::exp),
        "sqrt" => |_, c| unary(c, f64::sqrt),
        "pow" => |_, c| binary(c, f64::powf),
        // `fmod`: the sign of the dividend, as C's.
        "fmod" => |_, c| binary(c, |a, b| a % b),
        "__max" => |_, c| binary(c, |a, b| if a <= b { b } else { a }),
        "__min" => |_, c| binary(c, |a, b| if b <= a { b } else { a }),
        "MapToUnitInterval" => map_to_unit_interval,
        "abs" => |_, c| {
            let v = (super::word(c, 2) as i32).wrapping_abs();
            Call::new(c).ret(3, v as u32, INT_RESULT);
            InterpStatus::Success
        },
        "fake_abs64" => |_, c| {
            let v = (arg64(c, 2) as i64).wrapping_abs();
            ret64(c, 4, v as u64)
        },
        "intMax" => |_, c| ints(c, |a, b| if b < a { a } else { b }),
        "intMin" => |_, c| ints(c, |a, b| if a < b { a } else { b }),
        "floatMax" => |_, c| floats(c, |a, b| if a <= b { b } else { a }),
        "floatMin" => |_, c| floats(c, |a, b| if b <= a { b } else { a }),
        "lintMax" => |_, c| longs(c, |a, b| if a as i64 <= b as i64 { b } else { a }),
        "lintMin" => |_, c| longs(c, |a, b| if b as i64 <= a as i64 { b } else { a }),
        "ulintMax" => |_, c| longs(c, |a, b| if a <= b { b } else { a }),
        "ulintMin" => |_, c| longs(c, |a, b| if b <= a { b } else { a }),
        _ => return None,
    })
}

fn vector(vm: &Vm, r: u32) -> Option<[f32; 3]> {
    match vm.native(r)? {
        NativeObject::Vector(v) => Some(*v),
        _ => None,
    }
}

/// Runs `f` on `this` as a `Vector`.
fn with_vector(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut [f32; 3], &Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match vm.native_mut(this) {
        Some(NativeObject::Vector(v)) => {
            f(v, ctx);
            InterpStatus::Success
        }
        _ => vm.bad_this(this),
    }
}

/// `this` and the vector in word 2.
fn pair(vm: &mut Vm, ctx: &mut Context, f: fn(&mut [f32; 3], [f32; 3])) -> InterpStatus {
    let other = Call::new(ctx).word(2);
    let Some(o) = vector(vm, other) else {
        return vm.bad_this(other);
    };
    with_vector(vm, ctx, |v, _| f(v, o))
}

/// A float result in word `n` from `this`.
fn query(
    vm: &mut Vm,
    ctx: &mut Context,
    n: usize,
    f: impl FnOnce(&[f32; 3], &Vm, &Context) -> Option<u32>,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = vector(vm, this) else {
        return vm.bad_this(this);
    };
    match f(&v, vm, ctx) {
        Some(bits) => {
            Call::new(ctx).ret(n, bits, INT_RESULT);
            InterpStatus::Success
        }
        None => vm.bad_this(Call::new(ctx).word(2)),
    }
}

fn mul(a: f32, b: f32) -> f64 {
    f64::from(a) * f64::from(b)
}

fn vector_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "Set" => |vm, c| {
            with_vector(vm, c, |v, c| {
                let w = |n| f32::from_bits(super::word(c, n));
                *v = [w(2), w(3), w(4)];
            })
        },
        "Add" => |vm, c| pair(vm, c, |v, o| *v = [o[0] + v[0], o[1] + v[1], o[2] + v[2]]),
        "Subtract" => |vm, c| pair(vm, c, |v, o| *v = [v[0] - o[0], v[1] - o[1], v[2] - o[2]]),
        "Copy" => |vm, c| pair(vm, c, |v, o| *v = o),
        "Scale" => |vm, c| {
            with_vector(vm, c, |v, c| {
                let s = f32::from_bits(super::word(c, 2));
                *v = [s * v[0], s * v[1], s * v[2]];
            })
        },
        // `D3DXVec3Normalize`: a zero vector stays zero.
        "Normalize" => |vm, c| {
            with_vector(vm, c, |v, _| {
                let len = (mul(v[0], v[0]) + mul(v[1], v[1]) + mul(v[2], v[2])).sqrt();
                *v = if len == 0.0 {
                    [0.0; 3]
                } else {
                    v.map(|x| (f64::from(x) / len) as f32)
                };
            })
        },
        // Retail's MagnitudeSquared returns without writing a result.
        "MagnitudeSquared" => |vm, c| with_vector(vm, c, |_, _| {}),
        "Magnitude" => |vm, c| {
            query(vm, c, 2, |v, _, _| {
                let m = (mul(v[2], v[2]) + mul(v[1], v[1]) + mul(v[0], v[0])).sqrt();
                Some((m as f32).to_bits())
            })
        },
        "Dot" => |vm, c| {
            query(vm, c, 3, |v, vm, c| {
                let o = vector(vm, super::word(c, 2))?;
                Some(((mul(o[0], v[0]) + mul(o[1], v[1]) + mul(o[2], v[2])) as f32).to_bits())
            })
        },
        "Equals" => |vm, c| {
            query(vm, c, 3, |v, vm, c| {
                let o = vector(vm, super::word(c, 2))?;
                let near = (0..3).all(|i| (f64::from(v[i]) - f64::from(o[i])).abs() < 0.0002);
                Some(u32::from(near))
            })
        },
        "GetX" => |vm, c| query(vm, c, 2, |v, _, _| Some(v[0].to_bits())),
        "GetY" => |vm, c| query(vm, c, 2, |v, _, _| Some(v[1].to_bits())),
        "GetZ" => |vm, c| query(vm, c, 2, |v, _, _| Some(v[2].to_bits())),
        "Cross" => cross,
        "GetStringW" => get_string_w,
        _ => return None,
    })
}

/// `Cross(other)`: a new `Vector` (`this x other`, each component one f32 store).
fn cross(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, other) = (call.this(), call.word(2));
    let (Some(a), Some(b)) = (vector(vm, this), vector(vm, other)) else {
        return vm.bad_this(this);
    };
    let c = [
        (mul(a[1], b[2]) - mul(a[2], b[1])) as f32,
        (mul(b[0], a[2]) - mul(b[2], a[0])) as f32,
        (mul(a[0], b[1]) - mul(b[0], a[1])) as f32,
    ];
    let r = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector(c));
    Call::new(ctx).ret(3, r, tag::REF);
    InterpStatus::Success
}

/// `Vector::GetStringW` into the referenced string: `(%.3f, %.3f, %.3f)`.
fn get_string_w(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, out) = (call.this(), call.word(2));
    let Some(v) = vector(vm, this) else {
        return vm.bad_this(this);
    };
    let text = format!(
        "({}, {}, {})",
        msvc_fixed(f64::from(v[0]), 3),
        msvc_fixed(f64::from(v[1]), 3),
        msvc_fixed(f64::from(v[2]), 3)
    );
    if let Some(target) = vm.read_through(ctx, out) {
        if let Some(NativeObject::WString(s)) = vm.native_mut(target) {
            *s = text.encode_utf16().collect();
        }
    }
    InterpStatus::Success
}
