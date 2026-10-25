//! A note's resonant low-pass filter, from the DLS Level 2 destinations `CONN_DST_FILTER_CUTOFF`
//! (0x0500) and `CONN_DST_FILTER_Q` (0x0501): a 2-pole biquad in transposed direct form II, its
//! coefficients recomputed in f64 at the start of each block of [`BLOCK`] frames.
//!
//! Units (the DLS2 reading the corpus supports, see `synth::modulation`): the cutoff is in
//! absolute-pitch cents (`8.176 * 2^(cents / 1200)` Hz, the same unit as the LFO frequency, see
//! [`super::lfo`]); a cutoff at or above [`BYPASS_FRACTION`] of the output rate bypasses the
//! filter, so the corpus's fixed `0x7FFFFFFF` (32767.99998 cents, far above Nyquist) is an open
//! filter. The resonance is in centibels: `Q = 10^(cB / 200) / sqrt(2)`, so 0 cB is the
//! Butterworth response (a flat passband, -3.01 dB at the cutoff) and each 10 cB adds 1 dB at the
//! cutoff. The DLS2 text was not read: this is the SoundFont-style reading, the port's assumption.
//!
//! Transfer function: the bilinear transform of the prototype `H(s) = 1 / (s^2 + s/Q + 1)` with the
//! cutoff prewarped (`K = tan(pi fc / fs)`), so `|H(e^{j w_c})| = Q` exactly at the cutoff, and at
//! `2 fc` the gain is `1 / sqrt((1 - r^2)^2 + (r/Q)^2)` with `r = 2 / (1 - K^2)` (the tests check
//! both forms).

use std::f64::consts::SQRT_2;

use super::lfo::hz_from_absolute_cents;

/// The frames the coefficients are recomputed over.
pub const BLOCK: usize = 32;
/// A cutoff at or above this fraction of the output rate bypasses the filter.
pub const BYPASS_FRACTION: f64 = 0.45;
/// The smallest and largest resonance Q the filter runs at (keeps the coefficients well scaled).
const MIN_Q: f64 = 0.05;
const MAX_Q: f64 = 50.0;

/// The biquad's coefficients, `H(z) = (b0 + b1 z^-1 + b2 z^-2) / (1 + a1 z^-1 + a2 z^-2)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coefficients {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

impl Coefficients {
    /// The identity: the filter passes its input unchanged.
    pub const BYPASS: Self = Self {
        b0: 1.0,
        b1: 0.0,
        b2: 0.0,
        a1: 0.0,
        a2: 0.0,
    };

    /// The low-pass at `cutoff_hz` with resonance `q` for an output of `rate` frames per second.
    pub fn lowpass(cutoff_hz: f64, q: f64, rate: f64) -> Self {
        let k = (std::f64::consts::PI * cutoff_hz / rate).tan();
        let norm = 1.0 / (1.0 + k / q + k * k);
        let b0 = k * k * norm;
        Self {
            b0,
            b1: 2.0 * b0,
            b2: b0,
            a1: 2.0 * (k * k - 1.0) * norm,
            a2: (1.0 - k / q + k * k) * norm,
        }
    }

    /// The coefficients for a cutoff given in Hz: [`Self::BYPASS`] at or above the bypass limit
    /// (and for a NaN cutoff).
    pub fn for_cutoff(cutoff_hz: f64, q: f64, rate: f64) -> Self {
        if cutoff_hz < BYPASS_FRACTION * rate {
            Self::lowpass(cutoff_hz.max(1e-3), q, rate)
        } else {
            Self::BYPASS
        }
    }

    /// The magnitude of `H(e^{jw})` at `omega` radians per frame.
    #[cfg(test)]
    pub fn gain(&self, omega: f64) -> f64 {
        let (c1, s1) = (omega.cos(), omega.sin());
        let (c2, s2) = ((2.0 * omega).cos(), (2.0 * omega).sin());
        // z^-1 = cos w - j sin w, z^-2 = cos 2w - j sin 2w.
        let num = (
            self.b0 + self.b1 * c1 + self.b2 * c2,
            -(self.b1 * s1 + self.b2 * s2),
        );
        let den = (
            1.0 + self.a1 * c1 + self.a2 * c2,
            -(self.a1 * s1 + self.a2 * s2),
        );
        num.0.hypot(num.1) / den.0.hypot(den.1)
    }
}

/// The resonance Q for a DLS resonance in centibels.
pub fn resonance_q(centibels: f64) -> f64 {
    (10f64.powf(centibels / 200.0) / SQRT_2).clamp(MIN_Q, MAX_Q)
}

/// The biquad's state, in transposed direct form II.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Biquad {
    c: Coefficients,
    s1: f64,
    s2: f64,
}

impl Biquad {
    /// Replaces the coefficients. A bypass clears the state, so a later cutoff starts clean.
    fn set(&mut self, c: Coefficients) {
        if c == Coefficients::BYPASS {
            self.s1 = 0.0;
            self.s2 = 0.0;
        }
        self.c = c;
    }

    fn process(&mut self, x: f64) -> f64 {
        let c = self.c;
        let y = c.b0 * x + self.s1;
        self.s1 = c.b1 * x - c.a1 * y + self.s2;
        self.s2 = c.b2 * x - c.a2 * y;
        y
    }
}

/// One voice's filter: the cutoff in absolute cents and the resonance Q, with the cutoff's route
/// offsets added at each block's start.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    /// The cutoff before the routes, in absolute-pitch cents.
    cents: f64,
    q: f64,
    rate: f64,
    state: Biquad,
    /// Frames until the coefficients are next recomputed.
    left: usize,
}

impl Filter {
    /// A filter at `cents` (absolute) with resonance `q`, for an output of `rate` frames per second.
    pub fn new(cents: f64, q: f64, rate: u32) -> Self {
        Self {
            cents,
            q,
            rate: f64::from(rate.max(1)),
            state: Biquad {
                c: Coefficients::BYPASS,
                s1: 0.0,
                s2: 0.0,
            },
            left: 0,
        }
    }

    /// Filters one frame `x`. The coefficients are recomputed at each block's first frame from the
    /// cutoff plus `offset_cents` (the routes' cents at that frame).
    pub fn process(&mut self, x: f64, offset_cents: f64) -> f64 {
        if self.left == 0 {
            self.left = BLOCK;
            let hz = hz_from_absolute_cents(self.cents + offset_cents);
            self.state
                .set(Coefficients::for_cutoff(hz, self.q, self.rate));
        }
        self.left -= 1;
        self.state.process(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_resonance_is_butterworth_at_zero_centibels_and_adds_a_decibel_per_ten() {
        assert!((resonance_q(0.0) - 1.0 / SQRT_2).abs() < 1e-12);
        let boosted = resonance_q(60.0);
        assert!((20.0 * (boosted * SQRT_2).log10() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn the_gain_at_the_cutoff_is_q_because_the_cutoff_is_prewarped() {
        for cb in [0.0, 30.0, 60.0, 128.0] {
            let q = resonance_q(cb);
            let fc = 1000.0;
            let c = Coefficients::lowpass(fc, q, 22050.0);
            let omega = 2.0 * std::f64::consts::PI * fc / 22050.0;
            assert!((c.gain(omega) - q).abs() < 1e-9, "Q {q}: {}", c.gain(omega));
        }
    }

    #[test]
    fn the_dc_gain_is_unity() {
        let c = Coefficients::lowpass(3000.0, 2.0, 44100.0);
        assert!((c.gain(0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_cutoff_at_the_bypass_limit_or_above_is_the_identity() {
        assert_eq!(
            Coefficients::for_cutoff(0.45 * 44100.0, 1.0, 44100.0),
            Coefficients::BYPASS
        );
        assert_eq!(
            Coefficients::for_cutoff(1.0e9, 1.0, 44100.0),
            Coefficients::BYPASS
        );
        assert_eq!(
            Coefficients::for_cutoff(f64::NAN, 1.0, 44100.0),
            Coefficients::BYPASS
        );
        let mut f = Filter::new(scaled_open(), 0.7, 44100);
        for i in 0..200 {
            let x = (i as f64 * 0.37).sin();
            assert_eq!(f.process(x, 0.0), x, "the open filter passes its input");
        }
    }

    /// The fixed open cutoff of the corpus: `0x7FFFFFFF` in 16.16 absolute cents.
    fn scaled_open() -> f64 {
        f64::from(i32::MAX) / 65_536.0
    }
}
