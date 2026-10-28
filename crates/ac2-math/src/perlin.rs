//! `PerlinNoise` (Legions `portal/shared/src/geometry/perlinnoise.cpp`): the one-dimensional
//! gradient noise that `Waveform::WAVEFORM_PERLIN` samples (the portal space's tunnel twists with
//! it). The tables are not stored data: `PerlinNoise::Init` fills them from the C runtime's
//! `rand()` after `srand(0)`, which is the MSVC linear congruential generator reproduced here.
//!
//! `Init`, from the admin 7148 disassembly: 256 gradients `g1[i] = ((rand() & 0x1FF) - 256) / 256`
//! (stored at `g1[i]`), then a Fisher-Yates shuffle of the permutation `p` with `rand() & 0xFF`
//! (`k` from 255 down to 1), then both tables are repeated at 256 and 257 for the wrap.

use std::sync::OnceLock;

/// The MSVC C runtime's `rand()`: `seed = seed * 214013 + 2531011`, the result
/// `(seed >> 16) & 0x7FFF`.
#[derive(Debug, Clone, Copy)]
pub struct MsvcRand {
    seed: u32,
}

impl MsvcRand {
    /// `srand(seed)`.
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// `rand()`, in `0..=0x7FFF`.
    pub fn next_value(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.seed >> 16) & 0x7FFF
    }
}

/// `PerlinNoise`'s tables: the gradients `g1` and the permutation `p`, each with the 2 wrap
/// entries.
#[derive(Debug, Clone, PartialEq)]
pub struct PerlinNoise {
    g1: [f32; 258],
    p: [usize; 258],
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self::init()
    }
}

impl PerlinNoise {
    /// `PerlinNoise::Init`: `srand(0)` and the tables from `rand()`.
    pub fn init() -> Self {
        let mut rand = MsvcRand::new(0);
        let mut g1 = [0.0f32; 258];
        let mut p = [0usize; 258];
        for (i, slot) in p.iter_mut().enumerate().take(256) {
            *slot = i;
            // `rand() & 0x800001FF`: the low nine bits, the value minus 256 in 1/256 steps.
            let r = (rand.next_value() & 0x1FF) as i32;
            g1[i] = ((r - 0x100) as f32) * 0.003_906_25;
        }
        for k in (1..256).rev() {
            let r = (rand.next_value() & 0xFF) as usize;
            p.swap(k, r);
        }
        for i in 0..2 {
            g1[256 + i] = g1[i];
            p[256 + i] = p[i];
        }
        Self { g1, p }
    }

    /// `PerlinNoise::Noise(arg)`: the gradient noise at `arg`, in about [-1, 1]. The integer part
    /// is truncated toward zero (`_ftol2`); the fraction, negative for negative arguments,
    /// interpolates the two lattice gradients with the smoothstep `t^2 (3 - 2t)`.
    pub fn noise(&self, arg: f64) -> f32 {
        let cell = arg.trunc() as i32;
        let t = arg - f64::from(cell);
        let i = (cell as u32 & 0xFF) as usize;
        let j = ((cell as u32 & 0xFF) as usize + 1) & 0xFF;
        let at_cell = f64::from(self.g1[self.p[i]]);
        let at_next = f64::from(self.g1[self.p[j]]);
        // `(float)(t * g1[p[i]])` is stored as float, twice.
        let lower = f64::from((t * at_cell) as f32);
        let smooth = t * t * (3.0 - (t + t));
        (smooth * ((t - 1.0) * at_next - lower) + lower) as f32
    }
}

impl PerlinNoise {
    /// `PerlinNoise::fBm1(x, H, lacunarity, octaves)` (`0x0062B180`): octave `i` is noise at
    /// `x * lacunarity^i` weighted by `pow(lacunarity^i, -H)`. The loop runs while `i < octaves`
    /// (six octaves at 6.0); a fractional octave count adds the next octave scaled by its
    /// fraction. The octaves sum in extended precision (the weights are doubles, the accumulator
    /// a double) and the result rounds to float once, at the return.
    pub fn fbm1(&self, x: f64, h: f64, lacunarity: f64, octaves: f64) -> f32 {
        let mut sum = 0.0f64;
        let mut arg = x;
        let mut power = 1.0f64; // lacunarity^i
        let mut i = 0usize;
        if octaves > 0.0 {
            loop {
                sum += f64::from(self.noise(arg)) * power.powf(-h);
                arg *= lacunarity;
                power *= lacunarity;
                i += 1;
                if (i as f64) >= octaves {
                    break;
                }
            }
        }
        let remainder = octaves - octaves.trunc();
        if remainder != 0.0 {
            sum += f64::from(self.noise(arg)) * power.powf(-h) * remainder;
        }
        sum as f32
    }
}

/// The client's shared `PerlinNoise` (`perlinNoise`, initialised on first use as `Noise` does).
pub fn shared() -> &'static PerlinNoise {
    static SHARED: OnceLock<PerlinNoise> = OnceLock::new();
    SHARED.get_or_init(PerlinNoise::init)
}

#[cfg(test)]
mod tests {
    use super::{MsvcRand, PerlinNoise};

    #[test]
    fn the_c_runtime_rand_from_seed_zero() {
        // 0 * 214013 + 2531011 = 2531011; (2531011 >> 16) & 0x7FFF = 38.
        let mut rand = MsvcRand::new(0);
        assert_eq!(rand.next_value(), 38);
        // The second: (2531011 * 214013 + 2531011) mod 2^32, shifted: 7719.
        assert_eq!(rand.next_value(), 7_719);
    }

    #[test]
    fn the_first_gradient_is_the_first_rand_scaled() {
        let perlin = PerlinNoise::init();
        // (38 & 0x1FF) - 256 = -218, in 1/256 steps.
        assert_eq!(perlin.g1[0], -218.0 / 256.0);
        // The noise is zero at the lattice points (t = 0) and continuous across them.
        assert_eq!(perlin.noise(0.0), 0.0);
        assert!((perlin.noise(1.0 - 1e-9) - perlin.noise(1.0)).abs() < 1e-4);
    }

    #[test]
    fn fbm1_with_one_octave_is_the_noise_and_with_no_weight_decay_it_sums_the_octaves() {
        let perlin = PerlinNoise::init();
        let x = 2.375;
        assert_eq!(perlin.fbm1(x, 0.7, 2.0, 1.0), perlin.noise(x));
        // H = 0: every weight is 1; the octaves are at x, 2x, 4x, ...
        let expected: f32 = (0..6).map(|i| perlin.noise(x * f64::from(1 << i))).sum();
        assert!((perlin.fbm1(x, 0.0, 2.0, 6.0) - expected).abs() < 1e-4);
    }

    #[test]
    fn the_waveform_noise_is_the_client_dice_within_its_amplitude() {
        use crate::waveform::{kind, Waveform};
        let noise = Waveform {
            kind: kind::NOISE,
            base: 1.0,
            amplitude: 0.5,
            ..Waveform::default()
        };
        let mut random = ac2_core::Random::default();
        for _ in 0..200 {
            let v = noise.value_for_phase_with(0.0, 0.0, &mut random);
            assert!((0.5..=1.5).contains(&v), "{v}");
        }
        // Without the dice the noise is its base.
        assert_eq!(noise.value(3.0), 1.0);
    }

    #[test]
    fn the_tables_wrap_and_stay_a_permutation() {
        let perlin = PerlinNoise::init();
        let mut seen = [false; 256];
        for &v in &perlin.p[..256] {
            assert!(!seen[v], "the permutation repeats {v}");
            seen[v] = true;
        }
        assert_eq!(perlin.p[256], perlin.p[0]);
        assert_eq!(perlin.g1[257], perlin.g1[1]);
    }
}
