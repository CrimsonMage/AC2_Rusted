//! The DLS2 EG delay and hold: the connection blocks set them (`CONN_DST_EG1_DELAYTIME` 0x020B,
//! `CONN_DST_EG1_HOLDTIME` 0x020C, `CONN_DST_EG2_DELAYTIME` 0x030F, `CONN_DST_EG2_HOLDTIME`
//! 0x0310), and the envelope stays silent through the delay and at full scale through the hold.
//! Synthetic: no corpus block sets them.

use super::envelope::{Envelope, EnvelopeParams, Shape};
use super::modulation::resolve;
use crate::music::dls::{Articulation, Connection};

/// `CONN_SRC_NONE` with no control: a fixed value of the destination, in timecents or per cent.
fn fixed(destination: u16, scale: i32) -> Connection {
    Connection {
        source: 0,
        control: 0,
        destination,
        transform: 0,
        scale,
    }
}

/// A 16.16 value.
fn raw(value: f64) -> i32 {
    (value * 65_536.0) as i32
}

/// The timecent value of `seconds` (`1200 * log2(seconds)`), as the blocks store it.
fn timecents(seconds: f64) -> i32 {
    raw(1200.0 * seconds.log2())
}

fn level2(connections: Vec<Connection>) -> Articulation {
    Articulation {
        connections,
        level2: true,
    }
}

fn params(delay: f64, attack: f64, hold: f64, decay: f64, shape: Shape) -> EnvelopeParams {
    EnvelopeParams {
        delay,
        attack,
        hold,
        decay,
        sustain: 0.5,
        release: 0.1,
        shape,
    }
}

/// Runs `envelope` for `seconds` in 1 ms steps, returning its level at each step.
fn run(envelope: &mut Envelope, seconds: f64) -> Vec<f64> {
    let steps = (seconds / 0.001).round() as usize;
    (0..steps).map(|_| envelope.tick(0.001)).collect()
}

#[test]
fn the_delay_is_silent_before_the_attack_begins() {
    let mut envelope = Envelope::new(params(0.2, 0.05, 0.0, 0.0, Shape::Db));
    let levels = run(&mut envelope, 0.3);
    assert!(
        levels[..190].iter().all(|&l| l == 0.0),
        "silent through the delay"
    );
    assert!(
        levels[195..].iter().any(|&l| l > 0.0),
        "the attack begins after it"
    );
}

#[test]
fn the_hold_keeps_the_linear_pitch_envelope_at_full_scale_before_its_decay() {
    // EG2 shape: an instant attack, a 0.3 s hold, then a decay to the sustain of 0.5 in 1 s.
    let mut envelope = Envelope::new(params(0.0, 0.0, 0.3, 1.0, Shape::Linear));
    let levels = run(&mut envelope, 0.6);
    assert!(
        levels[..290].iter().all(|&l| (l - 1.0).abs() < 1e-9),
        "full through the hold"
    );
    assert!(levels[599] < 1.0 - 0.05, "decays after it: {}", levels[599]);
}

#[test]
fn the_hold_keeps_the_db_volume_envelope_at_full_scale_before_its_decay() {
    let mut envelope = Envelope::new(params(0.0, 0.0, 0.3, 1.0, Shape::Db));
    let levels = run(&mut envelope, 0.6);
    assert!(
        levels[..290].iter().all(|&l| (l - 1.0).abs() < 1e-9),
        "full through the hold"
    );
    assert!(levels[599] < 0.99, "the decay follows: {}", levels[599]);
}

#[test]
fn a_release_during_the_hold_starts_from_full_scale() {
    let mut envelope = Envelope::new(params(0.0, 0.0, 1.0, 1.0, Shape::Db));
    run(&mut envelope, 0.1);
    envelope.release();
    let levels = run(&mut envelope, 0.05);
    // 100 dB over the 0.1 s release: 1 dB in the first millisecond, from full scale.
    assert!(
        levels[0] > 0.88,
        "the release starts at full scale: {}",
        levels[0]
    );
    assert!(levels[49] < levels[0], "and falls");
}

#[test]
fn a_release_during_the_delay_is_silent_at_once() {
    let mut envelope = Envelope::new(params(0.5, 0.1, 0.0, 0.0, Shape::Db));
    run(&mut envelope, 0.1);
    envelope.release();
    run(&mut envelope, 0.2);
    assert!(envelope.finished(), "a note off during the delay ends it");
}

#[test]
fn without_a_delay_or_hold_the_envelope_is_unchanged() {
    let mut plain = Envelope::new(params(0.0, 0.05, 0.0, 0.2, Shape::Db));
    let levels = run(&mut plain, 0.3);
    // Linear in amplitude: 0.9 at 45 ms; the peak at 50 ms, where the decay (0.5 dB) starts.
    assert!(
        levels[44] < 0.92 && levels[49] > 0.92,
        "attack to full at 50 ms"
    );
    assert!(levels[60] < levels[49], "the decay starts at once");
}

#[test]
fn the_connection_blocks_set_the_eg1_delay_and_hold() {
    let art = level2(vec![
        fixed(0x020B, timecents(0.25)),
        fixed(0x020C, timecents(0.5)),
    ]);
    let eg = resolve([&art], 60, 100).eg1;
    assert!((eg.delay - 0.25).abs() < 1e-3, "delay {}", eg.delay);
    assert!((eg.hold - 0.5).abs() < 1e-3, "hold {}", eg.hold);
}

#[test]
fn the_connection_blocks_set_the_eg2_delay_and_hold() {
    let art = level2(vec![
        fixed(0x030F, timecents(0.1)),
        fixed(0x0310, timecents(0.2)),
    ]);
    let eg = resolve([&art], 60, 100).eg2;
    assert!((eg.delay - 0.1).abs() < 1e-3, "delay {}", eg.delay);
    assert!((eg.hold - 0.2).abs() < 1e-3, "hold {}", eg.hold);
}

#[test]
fn no_block_leaves_the_delay_and_hold_at_zero() {
    let eg = resolve([&Articulation::default()], 60, 100);
    assert_eq!(
        (eg.eg1.delay, eg.eg1.hold, eg.eg2.delay, eg.eg2.hold),
        (0.0, 0.0, 0.0, 0.0)
    );
}
