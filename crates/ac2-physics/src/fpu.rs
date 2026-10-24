//! The retail floating-point model.
//!
//! The client ran the x87 FPU at 53-bit precision (`_setdefaultprecision`, and
//! `D3DCREATE_FPU_PRESERVE` on every device), so a chain of x87 operations rounds each step to a
//! double mantissa and rounds to `f32` only where the code stores into `float` memory. Physics
//! code here evaluates expressions in `f64` in the retail operation order and calls [`st`] exactly
//! where the retail code stores a `float` (`fstp DWORD PTR`); the call sites cite the function.

/// Rounds an x87 intermediate to `float` storage (`fstp DWORD PTR`).
#[inline(always)]
pub(crate) fn st(value: f64) -> f32 {
    value as f32
}

/// Loads a `float` into the x87 stack (`fld DWORD PTR`).
#[inline(always)]
pub(crate) fn ld(value: f32) -> f64 {
    f64::from(value)
}

/// `F_EPSILON` (`0x3951B717`), the tolerance of nearly every physics comparison.
pub(crate) const F_EPSILON: f32 = 0.0002;

/// `F_EPSILON * F_EPSILON` as the x87 forms it before a squared-length compare.
#[inline(always)]
pub(crate) fn epsilon_squared() -> f64 {
    ld(F_EPSILON) * ld(F_EPSILON)
}

/// `PhysicsGlobals::floor_z`: `cos(45.5 degrees)`, the steepest walkable normal (static
/// initialiser `fcos` of the double `0.7941248096574199`, identical in Legions).
pub(crate) fn floor_z() -> f32 {
    st(0.794_124_809_657_419_9_f64.cos())
}

/// `PhysicsGlobals::landing_z` (`0xba0c44`): `cos(85 degrees)`.
pub(crate) const LANDING_Z: f32 = 0.087_155_744;

/// `IsFloatZero`: `|value| < epsilon` with the x87 compare order (`fld epsilon; fcompp`).
#[inline(always)]
pub(crate) fn is_float_zero(value: f32, epsilon: f32) -> bool {
    ld(value).abs() < ld(epsilon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retail_constants_have_their_float_bits() {
        assert_eq!(F_EPSILON.to_bits(), 0x3951_B717);
        assert_eq!(floor_z(), 0.700_909_26);
        assert_eq!(st(1.483_529_864_195_180_2_f64.cos()), LANDING_Z);
    }

    #[test]
    fn stores_round_to_nearest_float() {
        let third = ld(1.0) / ld(3.0);
        assert_eq!(st(third), 1.0_f32 / 3.0);
        assert!(is_float_zero(-0.0001, F_EPSILON));
        assert!(!is_float_zero(F_EPSILON, F_EPSILON));
    }
}
