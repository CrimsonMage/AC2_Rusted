//! The synthesizer's controllers on the synthetic collection of `tests`: volume (CC7) scales a
//! channel's notes and only that channel's, the sustain pedal (CC64) holds a released note until
//! it goes up, and the pitch bend shifts a note by its range.

use super::tests::{expected, note_off, note_on, ramp, ramp_synth, render, Spec};
use super::Synth;
use crate::music::performance::{EventSink, MusicEvent, TimedEvent};

/// The output rate of the synthetic collection.
const RATE: f64 = 22050.0;
const PATCH: u32 = 5;

/// A controller change at `seconds` (on the timeline, just before frame `frame`'s instant).
fn control(seconds: f64, channel: u32, controller: u8, value: u8) -> TimedEvent {
    TimedEvent {
        seconds,
        event: MusicEvent::Control {
            channel,
            controller,
            value,
        },
        segment: 0,
    }
}

fn bend(seconds: f64, channel: u32, value: u16) -> TimedEvent {
    TimedEvent {
        seconds,
        event: MusicEvent::PitchBend { channel, value },
        segment: 0,
    }
}

/// The time of frame `frame`, pulled back half a frame so the event lands on that frame.
fn at_frame(frame: usize) -> f64 {
    (frame as f64 - 0.5) / RATE
}

/// A 16-frame ramp that loops over itself (a note held until its note-off).
fn looped() -> Spec {
    Spec {
        wave: ramp(16),
        loop_: Some((0, 16)),
        release_tc: None,
        connections: Vec::new(),
    }
}

#[test]
fn cc7_scales_the_channel_volume_from_its_frame() {
    let spec = looped();
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    synth.event(&control(at_frame(8), 0, 7, 50));
    let out = render(&mut synth, 16, 0.0);
    for (i, &v) in out.iter().enumerate() {
        let want = if i < 8 { 1.0 } else { 0.5 } * expected(&spec, i);
        assert!((v - want).abs() < 1e-6, "frame {i}: {v} vs {want}");
    }
    assert_eq!(synth.channel(0).volume, 50);
}

#[test]
fn cc7_on_one_channel_leaves_the_others_alone() {
    let spec = looped();
    let mut synth = ramp_synth(&spec);
    synth.bind(
        1,
        super::Source::Collection(super::tests::collection(&spec)),
    );
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    synth.event(&note_on(0.0, 1, PATCH, 60, 127));
    synth.event(&control(0.0, 1, 7, 0));
    let out = render(&mut synth, 16, 0.0);
    // Channel 0 at unity; channel 1 silent (volume 0). The sum is channel 0 alone.
    for (i, &v) in out.iter().enumerate() {
        assert!((v - expected(&spec, i)).abs() < 1e-6, "frame {i}: {v}");
    }
}

#[test]
fn the_sustain_pedal_holds_a_released_note_until_it_goes_up() {
    let spec = looped();
    let mut synth = ramp_synth(&spec);
    synth.event(&control(0.0, 0, 64, 127));
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    synth.event(&note_off(at_frame(5), 0, 60));
    let held = render(&mut synth, 32, 0.0);
    assert_eq!(synth.voices(), 1, "the pedal keeps the note");
    assert!(
        held[5..].iter().any(|v| v.abs() > 1e-3),
        "the held note keeps sounding after its key-up"
    );
    // The pedal goes up at frame 40: the note releases (no release time, so it ends there).
    synth.event(&control(at_frame(40), 0, 64, 0));
    let after = render(&mut synth, 16, 32.0 / RATE);
    assert_eq!(synth.voices(), 0, "the pedal's release ends the note");
    assert!(
        after[8..].iter().all(|&v| v == 0.0),
        "silent once the pedal went up"
    );
}

#[test]
fn without_the_pedal_a_released_note_stops_at_its_key_up() {
    let spec = looped();
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    synth.event(&note_off(at_frame(5), 0, 60));
    let out = render(&mut synth, 16, 0.0);
    assert_eq!(synth.voices(), 0);
    assert!(out[6..].iter().all(|&v| v == 0.0));
}

/// A note of the looped ramp at `key` (the unity key is 60) held for `frames`, with `controls`
/// applied first on its channel.
fn render_key(key: u8, controls: &[TimedEvent], frames: usize) -> Vec<f32> {
    let spec = looped();
    let mut synth = ramp_synth(&spec);
    for c in controls {
        synth.event(c);
    }
    synth.event(&note_on(0.0, 0, PATCH, key, 127));
    render(&mut synth, frames, 0.0)
}

#[test]
fn a_full_pitch_bend_up_shifts_a_note_by_its_range() {
    // The default range is 2 semitones: 16383 is 2 * 8191 / 8192 semitones above centre.
    let bent = render_key(60, &[bend(0.0, 0, 16_383)], 64);
    let up = render_key(62, &[], 64);
    for (i, (a, b)) in bent.iter().zip(&up).enumerate() {
        assert!((a - b).abs() < 1e-2, "frame {i}: {a} vs {b}");
    }
}

#[test]
fn a_full_pitch_bend_down_shifts_a_note_the_other_way() {
    let bent = render_key(60, &[bend(0.0, 0, 0)], 64);
    let down = render_key(58, &[], 64);
    for (i, (a, b)) in bent.iter().zip(&down).enumerate() {
        assert!((a - b).abs() < 1e-2, "frame {i}: {a} vs {b}");
    }
}

#[test]
fn the_rpn_pitch_bend_range_sets_the_bend() {
    // RPN 0 (101 and 100 zero), data entry 12 semitones: a full bend is an octave.
    let controls = [
        control(0.0, 0, 101, 0),
        control(0.0, 0, 100, 0),
        control(0.0, 0, 6, 12),
        bend(0.0, 0, 16_383),
    ];
    let bent = render_key(60, &controls, 64);
    let octave = render_key(72, &[], 64);
    for (i, (a, b)) in bent.iter().zip(&octave).enumerate() {
        assert!((a - b).abs() < 2e-2, "frame {i}: {a} vs {b}");
    }
}

#[test]
fn the_channel_pressure_and_mod_wheel_are_kept_without_changing_the_sound() {
    let plain = render_key(60, &[], 32);
    let touched = render_key(
        60,
        &[
            control(0.0, 0, 1, 127),
            TimedEvent {
                seconds: 0.0,
                event: MusicEvent::ChannelPressure {
                    channel: 0,
                    value: 90,
                },
                segment: 0,
            },
        ],
        32,
    );
    assert_eq!(plain, touched, "neither changes a note without a route");
}

/// `CONN_SRC_NONE` to a fixed destination (`0` control).
const NONE: u16 = 0;
const DST_ATTENUATION: u16 = 0x0001;
const DST_PITCH: u16 = 0x0003;
const DST_PAN: u16 = 0x0004;
const DST_LFO_FREQUENCY: u16 = 0x0104;
const DST_EG1_DECAY: u16 = 0x0207;
const DST_EG1_SUSTAIN: u16 = 0x020A;
const DST_EG2_ATTACK: u16 = 0x030A;
const SRC_LFO: u16 = 0x0001;
const SRC_KEY_ON_VELOCITY: u16 = 0x0002;
const SRC_EG2: u16 = 0x0005;
const CONTROL_MOD_WHEEL: u16 = 0x0081;

/// A 16.16 scale for `value` in the destination's unit.
fn raw(value: f64) -> i32 {
    (value * 65_536.0).round() as i32
}

/// The 16-frame looped ramp with `connections` on its instrument.
fn connected(connections: Vec<(u16, u16, u16, i32)>) -> Spec {
    Spec {
        wave: ramp(16),
        loop_: Some((0, 16)),
        release_tc: None,
        connections,
    }
}

/// A note of the looped ramp at `key`, with `connections` on its instrument, rendered for `frames`.
fn render_key_with(key: u8, connections: &[(u16, u16, u16, i32)], frames: usize) -> Vec<f32> {
    let mut synth = ramp_synth(&connected(connections.to_vec()));
    synth.event(&note_on(0.0, 0, PATCH, key, 127));
    render(&mut synth, frames, 0.0)
}

/// Interleaved stereo for `frames` frames of `synth` from `start`.
fn render_stereo(synth: &mut Synth, frames: usize, start: f64) -> Vec<f32> {
    let mut out = vec![0.0f32; 2 * frames];
    synth.render(&mut out, 2, start);
    out
}

#[test]
fn a_hard_right_pan_connection_sends_the_note_to_the_right() {
    let spec = connected(vec![(NONE, 0, DST_PAN, raw(500.0))]);
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render_stereo(&mut synth, 16, 0.0);
    for i in 0..16 {
        assert!(out[2 * i].abs() < 1e-6, "left {i}: {}", out[2 * i]);
        assert!(
            (out[2 * i + 1] - expected(&spec, i)).abs() < 1e-6,
            "right {i}"
        );
    }
}

#[test]
fn a_centre_pan_leaves_both_sides_at_unity() {
    let spec = connected(Vec::new());
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render_stereo(&mut synth, 16, 0.0);
    for i in 0..16 {
        assert!((out[2 * i] - expected(&spec, i)).abs() < 1e-6);
        assert!((out[2 * i + 1] - expected(&spec, i)).abs() < 1e-6);
    }
}

#[test]
fn the_channels_pan_controller_splits_the_note_left_and_right() {
    let spec = connected(Vec::new());
    let mut synth = ramp_synth(&spec);
    synth.event(&control(0.0, 0, 10, 0));
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render_stereo(&mut synth, 16, 0.0);
    for i in 0..16 {
        assert!((out[2 * i] - expected(&spec, i)).abs() < 1e-6, "left {i}");
        assert!(out[2 * i + 1].abs() < 1e-6, "right {i}");
    }
}

#[test]
fn velocity_attenuates_through_its_connection() {
    // 96 dB at velocity 0, none at 127: the attenuation at velocity 64 is 96 * (63 / 127) dB.
    let spec = connected(vec![(SRC_KEY_ON_VELOCITY, 0, DST_ATTENUATION, raw(96.0))]);
    let mut loud = ramp_synth(&spec);
    loud.event(&note_on(0.0, 0, PATCH, 60, 127));
    let loud = render(&mut loud, 16, 0.0);
    let mut soft = ramp_synth(&spec);
    soft.event(&note_on(0.0, 0, PATCH, 60, 64));
    let soft = render(&mut soft, 16, 0.0);
    let db = 96.0 * 63.0 / 127.0;
    let ratio = (64.0 / 127.0) * 10f64.powf(-db / 20.0);
    for i in 0..16 {
        let want = f64::from(loud[i]) * ratio;
        assert!(
            (f64::from(soft[i]) - want).abs() < 1e-6,
            "frame {i}: {} vs {want}",
            soft[i]
        );
    }
}

/// A constant half-scale wave under an EG1 decay of 0.1 s (timecents -3986) to `sustain` (0.1
/// percent units), rendered for `frames`.
fn decaying(sustain: i32, frames: usize) -> Vec<f32> {
    let spec = Spec {
        wave: vec![16384; RATE as usize],
        loop_: None,
        release_tc: None,
        connections: vec![
            (NONE, 0, DST_EG1_DECAY, raw(-3986.0)),
            (NONE, 0, DST_EG1_SUSTAIN, raw(f64::from(sustain))),
        ],
    };
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    render(&mut synth, frames, 0.0)
}

#[test]
fn the_volume_envelope_falls_100_db_per_decay_time_in_db() {
    // A sustain of zero: the decay runs its full 100 dB, so after 0.02 s (a fifth of the decay
    // time) the level is 20 dB down. A linear decay would be at 0.4 of half scale instead.
    let out = decaying(0, 2400);
    let decay = (-3986.0 / 1200.0f64).exp2();
    // Frame 440 is 441 ticks in (0.02 s, the attack's tick included).
    let t = 441.0 / RATE;
    let want = 0.5 * 10f64.powf(-(100.0 * t / decay) / 20.0);
    assert!(
        (f64::from(out[440]) - want).abs() < 1e-6,
        "{} vs {want}",
        out[440]
    );
    assert!(
        out[2300..].iter().all(|&v| v == 0.0),
        "silent past the decay time"
    );
}

#[test]
fn the_volume_envelope_stops_its_decay_at_the_sustain_in_db() {
    // Half sustain is 6.02 dB down, reached 0.06 of the decay time in (about frame 133), and held.
    let out = decaying(500, 6000);
    let want = 0.5 * 10f64.powf(-6.0206 / 20.0);
    assert!((f64::from(out[200]) - want).abs() < 1e-4, "{}", out[200]);
    assert!(
        (f64::from(out[5999]) - want).abs() < 1e-4,
        "held at {}",
        out[5999]
    );
}

#[test]
fn the_pitch_envelope_bends_the_note_an_octave_as_it_attacks() {
    // EG2 attack 0.1 s (linear), routed to pitch at 1200 cents: after the attack the note plays an
    // octave up, so the 16-frame loop repeats every 8 output frames.
    let spec = connected(vec![
        (NONE, 0, DST_EG2_ATTACK, raw(-3986.0)),
        (SRC_EG2, 0, DST_PITCH, raw(1200.0)),
    ]);
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render(&mut synth, 3000, 0.0);
    for i in 2400..2990 {
        assert!(
            (out[i] - out[i + 8]).abs() < 1e-4,
            "frame {i}: an octave up repeats every 8 frames"
        );
    }
    // Without the route the loop repeats every 16 frames, and not every 8.
    let mut plain = ramp_synth(&connected(Vec::new()));
    plain.event(&note_on(0.0, 0, PATCH, 60, 127));
    let plain = render(&mut plain, 3000, 0.0);
    assert!((2400..2990).any(|i| (plain[i] - plain[i + 8]).abs() > 1e-2));
}

#[test]
fn an_lfo_is_gated_by_the_mod_wheel_and_moves_the_pitch_when_it_is_up() {
    // The LFO at 5 Hz (absolute pitch cents from 8.176 Hz), routed to pitch at 100 cents by the mod
    // wheel (CC1).
    let lfo_cents = 1200.0 * (5.0 / 8.175_798_915_643_707f64).log2();
    let spec = connected(vec![
        (NONE, 0, DST_LFO_FREQUENCY, raw(lfo_cents)),
        (SRC_LFO, CONTROL_MOD_WHEEL, DST_PITCH, raw(100.0)),
    ]);
    let mut quiet = ramp_synth(&connected(Vec::new()));
    quiet.event(&note_on(0.0, 0, PATCH, 60, 127));
    let plain = render(&mut quiet, 3000, 0.0);
    let mut still = ramp_synth(&spec);
    still.event(&note_on(0.0, 0, PATCH, 60, 127));
    let still = render(&mut still, 3000, 0.0);
    assert_eq!(still, plain, "the wheel at zero leaves the pitch alone");
    let mut wheel = ramp_synth(&spec);
    wheel.event(&control(0.0, 0, 1, 127));
    wheel.event(&note_on(0.0, 0, PATCH, 60, 127));
    let wheel = render(&mut wheel, 3000, 0.0);
    assert!(
        wheel.iter().zip(&plain).any(|(a, b)| (a - b).abs() > 1e-3),
        "the wheel up vibrates the pitch"
    );
}

#[test]
fn an_lfo_is_a_triangle_at_its_frequency_after_its_delay() {
    use super::lfo::Lfo;
    let mut lfo = Lfo::new(5.0, 0.0);
    let dt = 1e-4;
    let at = |frames: usize, lfo: &mut Lfo| {
        let mut v = 0.0;
        for _ in 0..frames {
            v = lfo.tick(dt);
        }
        v
    };
    // Phase 0.5 (a peak of +1) at 0.1 s, phase 0 (a trough of -1) at 0.2 s.
    assert!((at(1000, &mut lfo) - 1.0).abs() < 1e-6);
    assert!((at(1000, &mut lfo) + 1.0).abs() < 1e-6);
    let mut delayed = Lfo::new(5.0, 0.05);
    assert_eq!(at(400, &mut delayed), 0.0, "silent through its delay");
    assert!(at(200, &mut delayed).abs() > 0.0, "running after it");
}

const SRC_KEY_NUMBER: u16 = 0x0003;

#[test]
fn the_key_number_to_pitch_route_tracks_the_key_from_middle_c() {
    // 100 cents per key (scale in cents): key 62 is 200 cents up, so it sounds as key 64 does.
    let tracked = render_key_with(62, &[(SRC_KEY_NUMBER, 0, DST_PITCH, raw(100.0))], 64);
    let plain = render_key_with(64, &[], 64);
    for (i, (a, b)) in tracked.iter().zip(&plain).enumerate() {
        assert!((a - b).abs() < 1e-2, "frame {i}: {a} vs {b}");
    }
}

#[test]
fn the_lfo_to_attenuation_route_moves_the_level_with_the_lfo() {
    // A constant half-scale wave under a 5 Hz LFO that attenuates by 6 dB at its peak (phase 0.5,
    // 0.1 s in). Without the route the level is flat at half scale.
    let lfo_cents = 1200.0 * (5.0 / 8.175_798_915_643_707f64).log2();
    let spec = Spec {
        wave: vec![16384; RATE as usize],
        loop_: None,
        release_tc: None,
        connections: vec![
            (NONE, 0, DST_LFO_FREQUENCY, raw(lfo_cents)),
            (SRC_LFO, 0, DST_ATTENUATION, raw(6.0)),
        ],
    };
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render(&mut synth, 2300, 0.0);
    // At the trough (0 s, LFO -1) the attenuation is -6 dB: a boost to just over half scale's
    // double; at the peak (0.1 s) it is 6 dB: the level falls to about 0.25.
    assert!(
        f64::from(out[2205]) < 0.27,
        "the peak attenuates: {}",
        out[2205]
    );
    assert!(out[0] > 0.9, "the trough boosts: {}", out[0]);
}
