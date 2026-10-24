//! `Random`, the client's shared generator (`portal/shared/src/random`). `Random::rand` is
//! L'Ecuyer's two-stream generator with a 32-entry Bays-Durham shuffle (Numerical Recipes'
//! `ran2` without its warm-up): its state is static data (`_seed` 1, `_idum2` 123456789, `_iy` and
//! `_iv` zero) that nothing reseeds, so the stream is the same on every run of the client.

/// `_seed` / `_idum2` moduli and multipliers (`IM1`, `IA1`, `IQ1`; `IM2`, `IA2`, `IQ2`).
const IM1: i32 = 0x7FFF_FFAB;
const IA1: i32 = 0x9C4E;
const IQ1: i32 = 0xD1A4;
const IM2: i32 = 0x7FFF_FF07;
const IA2: i32 = 0x9EF4;
const IQ2: i32 = 0xCE26;
/// Shuffle-table divisor `1 + (IM1 - 1) / 32`.
const NDIV: i32 = 0x03FF_FFFE;
/// `1 / IM1` and `1 - EPS` as the doubles `Random::rand` loads.
const AM: f64 = 4.656_613_057_391_769e-10;
const RNMX: f64 = 0.999_999_88;

/// The `Random::rand` state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Random {
    seed: i32,
    idum2: i32,
    iy: i32,
    iv: [i32; 32],
}

impl Default for Random {
    /// The state the client starts with.
    fn default() -> Self {
        Self {
            seed: 1,
            idum2: 123_456_789,
            iy: 0,
            iv: [0; 32],
        }
    }
}

impl Random {
    /// `Random::rand`: the next value in [0, 0.99999988].
    pub fn rand(&mut self) -> f64 {
        // `seed * IA1 - (seed / IQ1) * IM1` in 32-bit wrapping arithmetic, as compiled.
        self.seed = self
            .seed
            .wrapping_mul(IA1)
            .wrapping_sub((self.seed / IQ1).wrapping_mul(IM1));
        if self.seed < 0 {
            self.seed = self.seed.wrapping_add(IM1);
        }
        self.idum2 = self
            .idum2
            .wrapping_mul(IA2)
            .wrapping_sub((self.idum2 / IQ2).wrapping_mul(IM2));
        if self.idum2 < 0 {
            self.idum2 = self.idum2.wrapping_add(IM2);
        }
        let slot = (self.iy / NDIV) as usize;
        self.iy = self.iv[slot].wrapping_sub(self.idum2);
        self.iv[slot] = self.seed;
        if self.iy < 1 {
            self.iy = self.iy.wrapping_add(IM1 - 1);
        }
        (f64::from(self.iy) * AM).min(RNMX)
    }

    /// `Random::RollDice(long, long)`: an integer in [lower, upper] (bounds in either order),
    /// `_ftol2` of the span times `rand()` plus the lower bound.
    pub fn roll_dice(&mut self, lower: i32, upper: i32) -> i32 {
        if lower == upper {
            return lower;
        }
        let (lo, hi) = (lower.min(upper), lower.max(upper));
        let span = f64::from(hi.wrapping_sub(lo).wrapping_add(1) as u32);
        // x87 extended product, truncated; for spans below 2^21 the f64 product is exact.
        ((span * self.rand()) as i64 as i32).wrapping_add(lo)
    }

    /// `Random::RollDice(float, float)`: `(upper - lower) * (float)rand() + lower`.
    pub fn roll_dice_f32(&mut self, lower: f32, upper: f32) -> f32 {
        if lower == upper {
            return lower;
        }
        let (lo, hi) = (lower.min(upper), lower.max(upper));
        let r = self.rand() as f32;
        (f64::from(hi - lo) * f64::from(r) + f64::from(lo)) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::Random;

    #[test]
    fn first_values_follow_the_static_state() {
        let mut random = Random::default();
        // seed 1 -> 40014; idum2 123456789 -> 123456789 * 40692 mod IM2; iy = 0 - idum2 + IM1 - 1.
        let first = random.rand();
        let idum2 = (123_456_789_i64 * 40692) % 2_147_483_399;
        let iy = 2_147_483_562 - idum2;
        assert_eq!(first, iy as f64 * 4.656_613_057_391_769e-10);
        assert_eq!(random.iv[0], 40014);
        for _ in 0..1000 {
            let value = random.rand();
            assert!((0.0..=0.999_999_88).contains(&value));
        }
    }

    #[test]
    fn dice_stay_in_bounds() {
        let mut random = Random::default();
        for _ in 0..1000 {
            assert!((0..=1).contains(&random.roll_dice(1, 0)));
            assert!((-3.0..=2.0).contains(&random.roll_dice_f32(2.0, -3.0)));
        }
        assert_eq!(random.roll_dice(5, 5), 5);
    }
}
