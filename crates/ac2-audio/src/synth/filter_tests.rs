//! The low-pass filter and the vibrato LFO on synthetic collections (no game data): a sine wave
//! looped at unity, so its frequency in the output is the wave's own, and each test's expected
//! value is worked from the filter's transfer function or the LFO's model.

use std::f64::consts::PI;

use super::filter::{resonance_q, Coefficients};
use super::tests::{note_on, ramp_synth, render, Spec};
use super::Synth;
use crate::music::dls::{Articulation, Connection};
use crate::music::performance::{EventSink, MusicEvent, TimedEvent};

/// The output rate of the synthetic collection (the `tests` rate).
const RATE: f64 = 22050.0;
const PATCH: u32 = 5;
/// Samples in the looped sine: 0.1 s, a whole number of cycles of 1000 and 2000 Hz.
const LOOP: usize = 2205;

const NONE: u16 = 0x0000;
const SRC_EG2: u16 = 0x0005;
const SRC_VIBRATO: u16 = 0x0009;
const CONTROL_MOD_WHEEL: u16 = 0x0081;
const CONTROL_PRESSURE: u16 = 0x0008;
const DST_PITCH: u16 = 0x0003;
const DST_EG2_ATTACK: u16 = 0x030A;
const DST_VIB_FREQUENCY: u16 = 0x0114;
const DST_VIB_DELAY: u16 = 0x0115;
const DST_FILTER_CUTOFF: u16 = 0x0500;
const DST_FILTER_Q: u16 = 0x0501;

/// The absolute-pitch cents of `hz` (the DLS2 unit, from 8.176 Hz).
fn cents_of(hz: f64) -> f64 {
    1200.0 * (hz / 8.175_798_915_643_707).log2()
}

/// A 16.16 fixed-point raw value.
fn raw(value: f64) -> i32 {
    (value * 65_536.0).round() as i32
}

/// The sine of `freq` Hz at `amplitude`, one loop of [`LOOP`] samples at [`RATE`].
fn sine(freq: f64, amplitude: f64) -> Vec<i16> {
    (0..LOOP)
        .map(|i| (amplitude * (2.0 * PI * freq * i as f64 / RATE).sin()).round() as i16)
        .collect()
}

/// The looped sine with `connections` on its instrument.
fn sine_spec(freq: f64, connections: Vec<(u16, u16, u16, i32)>) -> Spec {
    Spec {
        wave: sine(freq, 16_000.0),
        loop_: Some((0, LOOP as u32)),
        release_tc: None,
        connections,
    }
}

/// The sine at `freq` rendered at unity for `frames`, with `connections` and `controls`.
fn render_sine(
    freq: f64,
    connections: Vec<(u16, u16, u16, i32)>,
    controls: &[TimedEvent],
    frames: usize,
) -> Vec<f32> {
    let mut synth = ramp_synth(&sine_spec(freq, connections));
    for c in controls {
        synth.event(c);
    }
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    render(&mut synth, frames, 0.0)
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

/// The closed form of the 2-pole gain at `2 fc`: `r = 2 / (1 - K^2)` with `K = tan(pi fc / fs)`.
fn closed_gain_at_twice_cutoff(fc: f64, q: f64) -> f64 {
    let k = (PI * fc / RATE).tan();
    let r = 2.0 / (1.0 - k * k);
    1.0 / ((1.0 - r * r).powi(2) + (r / q).powi(2)).sqrt()
}

#[test]
fn a_two_pole_low_pass_attenuates_twice_its_cutoff_as_its_transfer_function_gives() {
    let fc = 1000.0;
    let frames = 13_230;
    let window = 4_410..frames;
    let plain = render_sine(2.0 * fc, vec![], &[], frames);
    let filtered = render_sine(
        2.0 * fc,
        vec![(NONE, 0, DST_FILTER_CUTOFF, raw(cents_of(fc)))],
        &[],
        frames,
    );
    let measured = rms(&filtered[window.clone()]) / rms(&plain[window]);
    let q = resonance_q(0.0);
    let omega = 2.0 * PI * 2.0 * fc / RATE;
    let transfer = Coefficients::lowpass(fc, q, RATE).gain(omega);
    assert!(
        (measured - transfer).abs() / transfer < 0.005,
        "measured {measured} vs the transfer function {transfer}"
    );
    assert!(
        (transfer - closed_gain_at_twice_cutoff(fc, q)).abs() < 1e-9,
        "the coefficients match the closed form"
    );
    // Butterworth (Q = 1/sqrt 2): 1 / sqrt(9 + 8) = 0.2425 analog, -12.3 dB; the prewarped digital
    // filter at 1 kHz is -12.65 dB.
    let db = 20.0 * transfer.log10();
    assert!(
        (-13.0..-12.0).contains(&db),
        "about -12 dB at twice the cutoff: {db}"
    );
}

#[test]
fn the_resonance_peaks_at_the_cutoff_by_its_q() {
    let fc = 1000.0;
    let frames = 13_230;
    let window = 4_410..frames;
    let plain = render_sine(fc, vec![], &[], frames);
    let gain_at = |cb: f64| {
        let mut connections = vec![(NONE, 0, DST_FILTER_CUTOFF, raw(cents_of(fc)))];
        if cb != 0.0 {
            connections.push((NONE, 0, DST_FILTER_Q, raw(cb)));
        }
        let out = render_sine(fc, connections, &[], frames);
        rms(&out[window.clone()]) / rms(&plain[window.clone()])
    };
    let flat = gain_at(0.0);
    let peaked = gain_at(60.0);
    let q = resonance_q(60.0);
    assert!(
        (peaked - q).abs() / q < 0.005,
        "the gain at the cutoff is Q: {peaked} vs {q}"
    );
    assert!(
        (flat - resonance_q(0.0)).abs() / flat < 0.005,
        "Butterworth at 0 cB: {flat}"
    );
    assert!(peaked > flat * 1.9, "60 cB peaks at the cutoff above 0 cB");
}

#[test]
fn the_cutoff_follows_eg2_over_time() {
    // The base cutoff 500 Hz; EG2 (linear, a 1 s attack) raises it by 2400 cents, to 2 kHz.
    let base = 500.0;
    let connections = vec![
        (NONE, 0, DST_FILTER_CUTOFF, raw(cents_of(base))),
        (NONE, 0, DST_EG2_ATTACK, raw(0.0)),
        (SRC_EG2, 0, DST_FILTER_CUTOFF, raw(2400.0)),
    ];
    let frames = 28_665;
    let plain = render_sine(2000.0, vec![], &[], frames);
    let filtered = render_sine(2000.0, connections, &[], frames);
    let q = resonance_q(0.0);
    let span = 441;
    let mut previous = 0.0;
    for t in [0.1, 0.4, 0.9, 1.2] {
        let start = (t * RATE) as usize - span / 2;
        let window = start..start + span;
        let centre = (start + span / 2) as f64 / RATE;
        // The envelope is at its level at the window's centre (linear in time over 1 s, then held).
        let level = centre.min(1.0);
        let cutoff = cents_of(base) + 2400.0 * level;
        let expected =
            Coefficients::lowpass(8.175_798_915_643_707 * (cutoff / 1200.0).exp2(), q, RATE)
                .gain(2.0 * PI * 2000.0 / RATE);
        let measured = rms(&filtered[window.clone()]) / rms(&plain[window]);
        assert!(
            (measured - expected).abs() / expected < 0.03,
            "t {t}: measured {measured} vs {expected} at level {level}"
        );
        assert!(
            measured > previous,
            "the output brightens as EG2 opens the cutoff"
        );
        previous = measured;
    }
}

/// The upward zero crossings of `x` (sample `i` at `i / RATE`), linearly interpolated.
fn upward_crossings(x: &[f32]) -> Vec<f64> {
    (0..x.len() - 1)
        .filter(|&i| f64::from(x[i]) < 0.0 && f64::from(x[i + 1]) >= 0.0)
        .map(|i| {
            let (a, b) = (f64::from(x[i]), f64::from(x[i + 1]));
            (i as f64 + (-a) / (b - a)) / RATE
        })
        .collect()
}

/// The triangle from -1 at phase 0 to +1 at 0.5, as the vibrato LFO's model draws it.
fn triangle(phase: f64) -> f64 {
    if phase < 0.5 {
        4.0 * phase - 1.0
    } else {
        3.0 - 4.0 * phase
    }
}

#[test]
fn the_vibrato_moves_the_pitch_by_its_depth_at_its_frequency_after_its_delay() {
    // A 1 kHz sine with vibrato to pitch: 50 cents at 5 Hz, starting after 0.2 s.
    let (fc_vib, delay): (f64, f64) = (5.0, 0.2);
    let connections = vec![
        (NONE, 0, DST_VIB_FREQUENCY, raw(cents_of(fc_vib))),
        (NONE, 0, DST_VIB_DELAY, raw(1200.0 * delay.log2())),
        (SRC_VIBRATO, 0, DST_PITCH, raw(50.0)),
    ];
    let frames = 33_075;
    let out = render_sine(1000.0, connections, &[], frames);
    let crossings = upward_crossings(&out);
    let mut deviations = Vec::new();
    for pair in crossings.windows(2) {
        let (t0, t1) = (pair[0], pair[1]);
        let cents = 1200.0 * ((1.0 / (t1 - t0)) / 1000.0).log2();
        deviations.push(((t0 + t1) / 2.0, cents));
    }
    assert!(
        deviations.len() > 1000,
        "about a thousand cycles: {}",
        deviations.len()
    );
    let mut largest: f64 = 0.0;
    for (mid, cents) in deviations {
        if mid < delay - 0.005 {
            assert!(
                cents.abs() < 0.2,
                "silent before the delay at {mid}: {cents}"
            );
        } else if mid > delay + 0.02 {
            let expected = 50.0 * triangle((fc_vib * (mid - delay)).fract());
            assert!(
                (cents - expected).abs() < 1.5,
                "at {mid}: {cents} cents vs the model's {expected}"
            );
            largest = largest.max(cents.abs());
        }
    }
    assert!(largest > 45.0, "the depth is reached: {largest}");
}

#[test]
fn the_vibrato_is_gated_by_the_mod_wheel_and_by_the_channel_pressure() {
    let frames = 13_230;
    let vib = |control: u16| {
        vec![
            (NONE, 0, DST_VIB_FREQUENCY, raw(cents_of(5.0))),
            (SRC_VIBRATO, control, DST_PITCH, raw(50.0)),
        ]
    };
    let plain = render_sine(1000.0, vec![], &[], frames);
    let wheel_at_zero = render_sine(1000.0, vib(CONTROL_MOD_WHEEL), &[], frames);
    assert_eq!(
        wheel_at_zero, plain,
        "the mod wheel at zero leaves the pitch alone"
    );
    let wheel = TimedEvent {
        seconds: 0.0,
        event: MusicEvent::Control {
            channel: 0,
            controller: 1,
            value: 127,
        },
        segment: 0,
    };
    let wheel_up = render_sine(1000.0, vib(CONTROL_MOD_WHEEL), &[wheel], frames);
    assert_ne!(wheel_up, plain, "the mod wheel up vibrates the pitch");
    let pressure_at_zero = render_sine(1000.0, vib(CONTROL_PRESSURE), &[], frames);
    assert_eq!(
        pressure_at_zero, plain,
        "no pressure leaves the pitch alone"
    );
    let pressure = TimedEvent {
        seconds: 0.0,
        event: MusicEvent::ChannelPressure {
            channel: 0,
            value: 127,
        },
        segment: 0,
    };
    let pressed = render_sine(1000.0, vib(CONTROL_PRESSURE), &[pressure], frames);
    assert_ne!(pressed, plain, "full pressure vibrates the pitch");
}

#[test]
fn an_open_cutoff_is_bypassed_bit_for_bit() {
    // The corpus's fixed maximum (0x7FFFFFFF) is an open filter, so the note is unchanged.
    let frames = 6_615;
    let plain = render_sine(1000.0, vec![], &[], frames);
    let open = render_sine(
        1000.0,
        vec![(NONE, 0, DST_FILTER_CUTOFF, i32::MAX)],
        &[],
        frames,
    );
    assert_eq!(open, plain);
}

#[test]
fn a_note_with_no_cutoff_block_has_no_filter() {
    use super::modulation::resolve;
    let m = resolve(std::iter::empty(), 60, 100);
    assert!(m.filter.is_none());
    assert!(m.vibrato.is_none());
}

#[test]
fn a_level_two_cutoff_overrides_a_level_one_cutoff_for_the_same_note() {
    use super::modulation::resolve;
    let block = |destination: u16, scale: f64| Connection {
        source: NONE,
        control: 0,
        destination,
        transform: 0,
        scale: raw(scale),
    };
    let level_one = Articulation {
        connections: vec![block(DST_FILTER_CUTOFF, f64::from(i32::MAX) / 65_536.0)],
        level2: false,
    };
    let level_two = Articulation {
        connections: vec![block(DST_FILTER_CUTOFF, cents_of(2000.0))],
        level2: true,
    };
    // The level-1 region's open maximum sits before the level-2 instrument's cutoff, yet the
    // level-2 block wins.
    let m = resolve([&level_one, &level_two], 60, 100);
    let f = m.filter.expect("a cutoff is set");
    assert!(
        (f.cutoff_cents - cents_of(2000.0)).abs() < 1e-3,
        "{}",
        f.cutoff_cents
    );
    let only_one = resolve([&level_one], 60, 100);
    assert!(
        only_one.filter.expect("set").cutoff_cents > 30_000.0,
        "open alone"
    );
}

#[test]
fn a_synth_with_no_connection_blocks_plays_the_plain_sine() {
    // The filter and vibrato leave a plain note as it was: the synth renders the sine as before.
    let mut synth = Synth::new(RATE as u32);
    synth.bind(
        0,
        super::Source::Collection(super::tests::collection(&sine_spec(1000.0, vec![]))),
    );
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render(&mut synth, 1000, 0.0);
    assert!(out.iter().any(|v| v.abs() > 0.1));
}
