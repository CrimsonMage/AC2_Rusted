//! Retail `Heading`: a compass angle in degrees, `[0, 360)`, clockwise from +y (north).

use crate::fpu::{ld, st};

/// `Heading::get_radians` multiplies by this `float` (`0x3C8EFA35`).
const DEGREES_TO_RADIANS: f32 = 0.017_453_292;
/// `Heading::set` multiplies radians by this `float` (`0x42652EE1`).
const RADIANS_TO_DEGREES: f32 = 57.295_78;
/// `1 / 360` and `1 / 255` as the `float` constants `ToUInt8`/`FromUInt8` load.
const INVERSE_360: f32 = 0.002_777_777_8;
const INVERSE_255: f32 = 0.003_921_568_9;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Heading {
    degrees: f32,
}

impl Heading {
    pub const ZERO: Self = Self { degrees: 0.0 };

    /// `Heading::Heading(float, DEGREES)`.
    pub fn from_degrees(degrees: f32) -> Self {
        Self {
            degrees: normalize(ld(degrees)),
        }
    }

    /// `Heading::Heading(float, RADIANS)`: the radians are scaled and stored as `float` before
    /// the wrap.
    pub fn from_radians(radians: f32) -> Self {
        Self::from_degrees(st(ld(RADIANS_TO_DEGREES) * ld(radians)))
    }

    pub fn degrees(self) -> f32 {
        self.degrees
    }

    /// `Heading::get_radians` (an x87 result, not stored).
    pub fn radians(self) -> f64 {
        ld(DEGREES_TO_RADIANS) * ld(self.degrees)
    }

    /// `Heading::get_degrees_0_centered`: `(-180, 180]`.
    pub fn degrees_0_centered(self) -> f64 {
        let degrees = ld(self.degrees);
        if degrees > 180.0 {
            degrees - 360.0
        } else {
            degrees
        }
    }

    /// `Heading::ToUInt8`: truncates `degrees / 360 * 255` (`_ftol2`); the low byte is kept.
    pub(crate) fn to_u8(self) -> u8 {
        let scaled = ld(self.degrees) * ld(INVERSE_360) * 255.0;
        (scaled.trunc() as i64) as u8
    }

    /// `Heading::FromUInt8`.
    pub(crate) fn from_u8(value: u8) -> Self {
        Self {
            degrees: normalize(f64::from(value) * ld(INVERSE_255) * 360.0),
        }
    }

    /// `operator-(Heading, Heading)`.
    pub(crate) fn difference(lhs: Self, rhs: Self) -> Self {
        Self {
            degrees: normalize(ld(lhs.degrees) - ld(rhs.degrees)),
        }
    }

    /// `Heading::abs`: the size of the turn, `[0, 180]`.
    pub(crate) fn abs(self) -> Self {
        let mut v = ld(self.degrees);
        if v > 180.0 {
            v -= 360.0;
        }
        Self {
            degrees: normalize(v.abs()),
        }
    }

    /// `Heading::is_in_range`: within `range` of `center` either way.
    pub fn is_in_range(self, center: Self, range: Self) -> bool {
        ld(Self::difference(self, center).abs().degrees) < ld(range.degrees)
    }
}

/// The wrap `Heading::set`, `FromUInt8` and `operator-` share: `fmod(value, 360)`, stored; a
/// negative result gains 360 (stored again) and a result that still compares `>= 360` becomes 0.
/// The compares use the unrounded x87 value.
fn normalize(value: f64) -> f32 {
    let wrapped = value % 360.0;
    if wrapped >= 0.0 {
        return st(wrapped);
    }
    let raised = wrapped + 360.0;
    if raised < 360.0 {
        st(raised)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_wrap_into_one_turn() {
        assert_eq!(Heading::from_degrees(370.0).degrees(), 10.0);
        assert_eq!(Heading::from_degrees(-90.0).degrees(), 270.0);
        // A tiny negative heading reaches 360 at 53-bit precision and wraps to zero.
        assert_eq!(Heading::from_degrees(-1e-30).degrees(), 0.0);
        // One just under the double rounding stays below 360 and stores as 360.0f.
        assert_eq!(Heading::from_degrees(-1e-5).degrees(), 360.0);
        assert_eq!(Heading::from_radians(std::f32::consts::PI).degrees(), 180.0);
    }

    #[test]
    fn byte_quantisation_round_trips_its_own_steps() {
        for byte in [0_u8, 1, 64, 127, 200, 254] {
            let heading = Heading::from_u8(byte);
            let back = heading.to_u8();
            assert!(back == byte || back + 1 == byte, "{byte} -> {back}");
        }
        assert_eq!(Heading::from_degrees(359.9).to_u8(), 254);
    }

    #[test]
    fn differences_and_centering() {
        let a = Heading::from_degrees(10.0);
        let b = Heading::from_degrees(350.0);
        assert_eq!(Heading::difference(a, b).degrees(), 20.0);
        assert_eq!(Heading::difference(b, a).degrees_0_centered(), -20.0);
    }
}
