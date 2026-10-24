//! AC2 `Waveform`: an animated scalar evaluated against client time (`Timer::local_time`, f64
//! seconds). `Waveform::GetValueForPhase` keeps each time-scaled term on the x87 stack and stores
//! amplitude, phase and frequency to `float` before combining them; this follows the same order.

/// `WaveformType` values.
pub mod kind {
    pub const INVALID: u32 = 0;
    pub const NONE: u32 = 1;
    pub const SPEED: u32 = 2;
    pub const NOISE: u32 = 3;
    pub const SINE: u32 = 4;
    pub const SQUARE: u32 = 5;
    pub const BOUNCE: u32 = 6;
    pub const PERLIN: u32 = 7;
    pub const FRACTAL: u32 = 8;
    pub const FRAMELOOP: u32 = 9;
}

/// The packed `Waveform` record: type plus base/amplitude/phase/frequency/roughness and their
/// per-second velocities.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Waveform {
    pub kind: u32,
    pub base: f32,
    pub base_velocity: f32,
    pub amplitude: f32,
    pub amplitude_velocity: f32,
    pub phase: f32,
    pub phase_velocity: f32,
    pub frequency: f32,
    pub frequency_velocity: f32,
    pub roughness: f32,
    pub roughness_velocity: f32,
}

impl Waveform {
    /// `Waveform::Waveform(float)`: a constant.
    pub fn constant(value: f32) -> Self {
        Self {
            kind: kind::NONE,
            base: value,
            ..Self::default()
        }
    }

    /// `Waveform::GetValue`: the value at client time `time`.
    pub fn value(&self, time: f64) -> f32 {
        self.value_for_phase(0.0, time)
    }

    /// `Waveform::GetValueForPhase`. `NOISE` rolls the client's global dice and `PERLIN`/`FRACTAL`
    /// sample its global `PerlinNoise`; neither source is reproduced, so those return the base.
    pub fn value_for_phase(&self, phase: f32, time: f64) -> f32 {
        let term = |velocity: f32, value: f32| f64::from(velocity) * time + f64::from(value);
        if self.kind == kind::NONE {
            return term(self.base_velocity, self.base) as f32;
        }
        let amplitude = term(self.amplitude_velocity, self.amplitude) as f32;
        let phase = ((f64::from(phase) + f64::from(self.phase)) * std::f64::consts::PI
            + f64::from(self.phase_velocity) * time) as f32;
        let frequency = term(self.frequency_velocity, self.frequency) as f32;
        let base = term(self.base_velocity, self.base);
        let wave = || (f64::from(frequency) * time + f64::from(phase)).sin();
        let value = match self.kind {
            kind::SPEED => f64::from(frequency) * time + f64::from(phase) + base,
            kind::SINE => wave() * f64::from(amplitude) + base,
            kind::SQUARE if wave() < 0.0 => -f64::from(amplitude) + base,
            kind::SQUARE => f64::from(amplitude) + base,
            kind::BOUNCE => wave().abs() * f64::from(amplitude) + base,
            kind::NOISE | kind::PERLIN | kind::FRACTAL => f64::from(base as f32),
            kind::FRAMELOOP => frame_loop(amplitude, frequency, phase, time) + base,
            _ => base,
        };
        value as f32
    }
}

/// The `WAVEFORM_FRAMELOOP` case: `amplitude` frames at `frequency` frames per second, each frame
/// stepping the value by `phase`. `_ftol2` truncates.
fn frame_loop(amplitude: f32, frequency: f32, phase: f32, time: f64) -> f64 {
    let frames = amplitude as i32;
    let rate = frequency as i32;
    if frames < 1 || rate < 1 {
        return 0.0;
    }
    let period = (1.0 / f64::from(rate)) * f64::from(frames);
    let loops = (time / period) as i32;
    let frame = ((time - f64::from(loops) * period) / period * f64::from(frames)) as i32;
    f64::from(frame) * f64::from(phase)
}

#[cfg(test)]
mod tests {
    use super::{kind, Waveform};

    #[test]
    fn constant_and_linear_waveforms() {
        assert_eq!(Waveform::constant(0.75).value(123.0), 0.75);
        let speed = Waveform {
            kind: kind::NONE,
            base: 1.0,
            base_velocity: 0.5,
            ..Waveform::default()
        };
        assert_eq!(speed.value(4.0), 3.0);
    }

    #[test]
    fn periodic_waveforms_follow_their_phase_in_half_turns() {
        let sine = Waveform {
            kind: kind::SINE,
            base: 1.0,
            amplitude: 2.0,
            phase: 0.5,
            ..Waveform::default()
        };
        // Phase 0.5 is a quarter turn: sin(pi/2) = 1.
        assert!((sine.value(0.0) - 3.0).abs() < 1e-6);
        let square = Waveform {
            kind: kind::SQUARE,
            amplitude: 1.0,
            frequency: 1.0,
            ..Waveform::default()
        };
        assert_eq!(square.value(4.0), -1.0);
        assert_eq!(square.value(1.0), 1.0);
    }

    #[test]
    fn frame_loop_steps_once_per_frame() {
        let frames = Waveform {
            kind: kind::FRAMELOOP,
            amplitude: 4.0,
            frequency: 2.0,
            phase: 1.0 / std::f32::consts::PI,
            ..Waveform::default()
        };
        // Four frames at two per second: t = 1.6 s is frame 3 of the loop.
        assert!((frames.value(1.6) - 3.0).abs() < 1e-5);
        assert!((frames.value(2.1) - 0.0).abs() < 1e-5);
    }
}
