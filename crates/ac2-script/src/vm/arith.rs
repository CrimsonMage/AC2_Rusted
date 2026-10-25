//! The arithmetic handlers (`interp_binop.cpp`, `interp_unaryop.cpp`).
//!
//! 32-bit integer results are tagged `INT`, 64-bit ones `LONG_INT` in both slots; float results
//! `FLOAT`, double results `DOUBLE`; comparisons push one `INT` 0/1. Retail evaluated floats on
//! the x87 at 53-bit precision and rounded to `float` on the store, so `f32` arithmetic is done
//! in `f64` and rounded once.

use super::exec::Step;
use super::{tag, Context, Fault, InterpStatus, Vm};

/// `_ftol2`: truncation toward zero; out-of-range values give the "integer indefinite" pattern.
fn ftol(value: f64) -> i32 {
    if value.is_nan() || value >= 2_147_483_648.0 || value < -2_147_483_648.0 {
        i32::MIN
    } else {
        value as i32
    }
}

/// `_ftol2` with a 64-bit result (edx:eax).
fn ftol64(value: f64) -> i64 {
    if value.is_nan() || value >= 9.223_372_036_854_776e18 || value < -9.223_372_036_854_776e18 {
        i64::MIN
    } else {
        value as i64
    }
}

/// `_allshl` / `_aullshr`: the count is the low byte; 64 or more gives 0.
fn shift64(value: u64, count: u64, left: bool) -> u64 {
    let n = (count & 0xFF) as u32;
    if n >= 64 {
        0
    } else if left {
        value << n
    } else {
        value >> n
    }
}

impl Vm {
    pub(super) fn arith(&mut self, code: u32, ctx: &mut Context) -> Step {
        let r = match code {
            0x0D..=0x11 | 0x43..=0x47 | 0x61 | 0x2D..=0x38 | 0x48 | 0x49 => {
                self.int_binop(code, ctx)
            }
            0x8D..=0x91 | 0xC3..=0xC7 | 0xE1 => self.long_binop(code & 0x7F, ctx),
            0xAD..=0xB8 => self.long_compare(code & 0x7F, ctx),
            0x15..=0x18 => self.float_binop(code, ctx),
            0x95..=0x98 => self.double_binop(code & 0x7F, ctx),
            0x3A..=0x41 => self.float_compare(code, ctx),
            0xBA..=0xC1 => self.double_compare(code & 0x7F, ctx),
            0x12..=0x14 | 0x42 | 0x2C | 0x1C | 0x62 | 0x1E | 0x60 => self.int_unary(code, ctx),
            0x92..=0x94 | 0x9C | 0x9E | 0xAC | 0xC2 | 0xE0 | 0xE2 => {
                self.long_unary(code & 0x7F, ctx)
            }
            0x19..=0x1B | 0x1D | 0x1F | 0x20 | 0x39 => self.float_unary(code, ctx),
            0x99..=0x9B | 0x9D | 0x9F | 0xA0 | 0xB9 => self.double_unary(code & 0x7F, ctx),
            _ => return Err(self.crash(Fault::UnknownOpcode(code))),
        };
        r.map_err(|f| self.crash(f))
    }

    fn int_binop(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = ctx.pop()?;
        let a = ctx.pop()?;
        let (sa, sb) = (a as i32, b as i32);
        let result = match code {
            0x0D => sa.wrapping_add(sb) as u32,
            0x0E => sa.wrapping_sub(sb) as u32,
            0x0F => sa.wrapping_mul(sb) as u32,
            0x10 | 0x11 => {
                if sb == 0 || (sa == i32::MIN && sb == -1) {
                    return Err(Fault::DivideByZero);
                }
                (if code == 0x10 { sa / sb } else { sa % sb }) as u32
            }
            0x43 => a | b,
            0x44 => a ^ b,
            0x45 => a & b,
            0x46 => (sa << (b & 0x1F)) as u32,
            0x47 => (sa >> (b & 0x1F)) as u32,
            0x61 => a >> (b & 0x1F),
            0x2D => u32::from(a != 0 || b != 0),
            0x2E => u32::from(a != 0 && b != 0),
            0x2F | 0x48 => u32::from(a == b),
            0x30 | 0x49 => u32::from(a != b),
            0x31 => u32::from(sa < sb),
            0x32 => u32::from(sa > sb),
            0x33 => u32::from(sa <= sb),
            0x34 => u32::from(sa >= sb),
            0x35 => u32::from(a < b),
            0x36 => u32::from(a > b),
            0x37 => u32::from(a <= b),
            _ => u32::from(a >= b),
        };
        Ok(ctx.push(result, tag::INT)?)
    }

    fn long_binop(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = ctx.pop64()?;
        let a = ctx.pop64()?;
        let (sa, sb) = (a as i64, b as i64);
        let result = match code {
            0x0D => a.wrapping_add(b),
            0x0E => a.wrapping_sub(b),
            0x0F => a.wrapping_mul(b),
            // `_alldiv` / `_allrem`.
            0x10 | 0x11 => {
                if sb == 0 {
                    return Err(Fault::DivideByZero);
                }
                (if code == 0x10 {
                    sa.wrapping_div(sb)
                } else {
                    sa.wrapping_rem(sb)
                }) as u64
            }
            0x43 => a | b,
            0x44 => a ^ b,
            0x45 => a & b,
            0x46 => shift64(a, b, true),
            // Both 64-bit right shifts call `_aullshr`: logical, even the signed one.
            _ => shift64(a, b, false),
        };
        Ok(ctx.push64(result, tag::LONG_INT)?)
    }

    fn long_compare(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = ctx.pop64()?;
        let a = ctx.pop64()?;
        let (sa, sb) = (a as i64, b as i64);
        let result = match code {
            0x2D => a != 0 || b != 0,
            0x2E => a != 0 && b != 0,
            0x2F => a == b,
            0x30 => a != b,
            0x31 => sa < sb,
            0x32 => sa > sb,
            0x33 => sa <= sb,
            0x34 => sa >= sb,
            0x35 => a < b,
            0x36 => a > b,
            0x37 => a <= b,
            _ => a >= b,
        };
        Ok(ctx.push(u32::from(result), tag::INT)?)
    }

    fn float_binop(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = f64::from(f32::from_bits(ctx.pop()?));
        let a = f64::from(f32::from_bits(ctx.pop()?));
        let result = match code {
            0x15 => a + b,
            0x16 => a - b,
            0x17 => a * b,
            _ => a / b,
        };
        Ok(ctx.push_float(result as f32)?)
    }

    fn double_binop(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = f64::from_bits(ctx.pop64()?);
        let a = f64::from_bits(ctx.pop64()?);
        let result = match code {
            0x15 => a + b,
            0x16 => a - b,
            0x17 => a * b,
            _ => a / b,
        };
        Ok(ctx.push_double(result)?)
    }

    fn compare(code: u32, a: f64, b: f64) -> bool {
        match code & 0x7F {
            0x3A => a != 0.0 || b != 0.0,
            0x3B => a != 0.0 && b != 0.0,
            0x3C => a == b,
            0x3D => a != b,
            0x3E => a < b,
            0x3F => a > b,
            // `(a < b) != (a == b)`: false for unordered operands.
            0x40 => a <= b,
            _ => a >= b,
        }
    }

    fn float_compare(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = f64::from(f32::from_bits(ctx.pop()?));
        let a = f64::from(f32::from_bits(ctx.pop()?));
        Ok(ctx.push(u32::from(Self::compare(code, a, b)), tag::INT)?)
    }

    fn double_compare(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let b = f64::from_bits(ctx.pop64()?);
        let a = f64::from_bits(ctx.pop64()?);
        Ok(ctx.push(u32::from(Self::compare(code, a, b)), tag::INT)?)
    }

    fn int_unary(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let v = ctx.pop()?;
        let s = v as i32;
        match code {
            0x12 => ctx.push(s.wrapping_neg() as u32, tag::INT)?,
            0x13 => ctx.push(s.wrapping_add(1) as u32, tag::INT)?,
            0x14 => ctx.push(s.wrapping_sub(1) as u32, tag::INT)?,
            0x42 => ctx.push(!v, tag::INT)?,
            0x2C => ctx.push(u32::from(v == 0), tag::INT)?,
            // I2F / U2F: one rounding of the exact integer to float.
            0x1C => ctx.push_float(s as f32)?,
            0x62 => ctx.push_float(v as f32)?,
            // I2I / U2U: widen to 64 bits.
            0x1E => ctx.push64(i64::from(s) as u64, tag::LONG_INT)?,
            _ => ctx.push64(u64::from(v), tag::LONG_INT)?,
        }
        Ok(())
    }

    /// `Interp::HandleUnaryOp_Long`.
    fn long_unary(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let v = ctx.pop64()?;
        match code {
            0x12 => ctx.push64(v.wrapping_neg(), tag::LONG_INT)?,
            0x13 => ctx.push64(v.wrapping_add(1), tag::LONG_INT)?,
            0x14 => ctx.push64(v.wrapping_sub(1), tag::LONG_INT)?,
            0x42 => ctx.push64(!v, tag::LONG_INT)?,
            0x2C => ctx.push64(u64::from(v == 0), tag::LONG_INT)?,
            0x1C => ctx.push_double(v as i64 as f64)?,
            // 2^63 + (v & 0x7FFF...) rounded once: the exact unsigned value to double.
            0x62 => ctx.push_double(v as f64)?,
            // L2I / UL2U: the low word.
            _ => ctx.push(v as u32, tag::INT)?,
        }
        Ok(())
    }

    /// `Interp::HandleFUnaryOp`.
    fn float_unary(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let v = f64::from(f32::from_bits(ctx.pop()?));
        match code {
            0x19 => ctx.push_float((v * -1.0) as f32)?,
            0x1A => ctx.push_float((v + 1.0) as f32)?,
            0x1B => ctx.push_float((v - 1.0) as f32)?,
            0x1D => ctx.push_int(ftol(v))?,
            0x1F => ctx.push_double(v)?,
            0x20 => ctx.push_time(v)?,
            _ => ctx.push(u32::from(v == 0.0), tag::INT)?,
        }
        Ok(())
    }

    /// `Interp::HandleFUnaryOp_Long`.
    fn double_unary(&mut self, code: u32, ctx: &mut Context) -> Result<(), Fault> {
        let v = f64::from_bits(ctx.pop64()?);
        match code {
            0x19 => ctx.push_double(v * -1.0)?,
            0x1A => ctx.push_double(v + 1.0)?,
            0x1B => ctx.push_double(v - 1.0)?,
            0x1D => ctx.push_long(ftol64(v))?,
            0x1F => ctx.push_float(v as f32)?,
            0x20 => ctx.push_time(v)?,
            _ => ctx.push(u32::from(v == 0.0), tag::INT)?,
        }
        Ok(())
    }
}

impl From<Fault> for InterpStatus {
    fn from(_: Fault) -> Self {
        InterpStatus::Crash
    }
}
