//! The DLS low-frequency oscillator: a triangle wave from -1 to 1 at its frequency, silent for
//! its start delay, one per voice when a connection block routes it.
//!
//! The frequency is the DLS absolute pitch (16.16 fixed-point cents from 8.176 Hz, the pitch of
//! MIDI note 0): `8.176 * 2^(cents / 1200)`. The delay is timecents (`2^(tc / 1200)` seconds). A
//! triangle is the port's reading of the LFO shape (the DLS Level 1 text was not read).

/// The frequency of absolute pitch 0 (MIDI note 0), in Hz.
const ABSOLUTE_PITCH_ZERO_HZ: f64 = 8.175_798_915_643_707;
/// The frequency an LFO without a `CONN_DST_LFO_FREQUENCY` runs at.
pub const DEFAULT_FREQUENCY_HZ: f64 = 5.0;

/// A triangle LFO: its phase (0 to 1), running after its delay.
#[derive(Debug, Clone, PartialEq)]
pub struct Lfo {
    frequency: f64,
    delay: f64,
    phase: f64,
    elapsed: f64,
}

/// The frequency in Hz for a 16.16 absolute-pitch value in cents.
pub fn hz_from_absolute_cents(cents: f64) -> f64 {
    ABSOLUTE_PITCH_ZERO_HZ * (cents / 1200.0).exp2()
}

impl Lfo {
    /// An LFO at `frequency` Hz, starting after `delay` seconds.
    pub fn new(frequency: f64, delay: f64) -> Self {
        Self {
            frequency: if frequency > 0.0 {
                frequency
            } else {
                DEFAULT_FREQUENCY_HZ
            },
            delay: delay.max(0.0),
            phase: 0.0,
            elapsed: 0.0,
        }
    }

    /// Advances by `dt` seconds and returns the output (-1 to 1; 0 during the delay).
    pub fn tick(&mut self, dt: f64) -> f64 {
        self.elapsed += dt;
        if self.elapsed < self.delay {
            return 0.0;
        }
        self.phase = (self.phase + self.frequency * dt).fract();
        triangle(self.phase)
    }
}

/// A triangle from -1 at phase 0, through 1 at phase 0.5, back to -1 at 1.
pub fn triangle(phase: f64) -> f64 {
    if phase < 0.5 {
        4.0 * phase - 1.0
    } else {
        3.0 - 4.0 * phase
    }
}
