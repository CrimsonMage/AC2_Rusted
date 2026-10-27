//! The wheel accumulator: fractional pixel deltas add up to whole notches.

use super::wheel::{wheel_event_delta, WheelAccumulator, PIXELS_PER_NOTCH, WHEEL_DELTA};

#[test]
fn line_deltas_are_whole_notches_as_they_come() {
    let mut wheel = WheelAccumulator::default();
    assert_eq!(wheel.line_delta(1.0), 1);
    assert_eq!(wheel.line_delta(-1.0), -1);
    assert_eq!(wheel.line_delta(3.0), 3);
    assert_eq!(wheel.line_delta(0.0), 0);
}

#[test]
fn pixel_deltas_below_a_notch_accumulate_until_they_make_one() {
    let mut wheel = WheelAccumulator::default();
    // Three 15 pixel steps: 0.375 of a notch, then 0.75, then 1.125.
    assert_eq!(wheel.pixel_delta(15.0), 0);
    assert_eq!(wheel.pixel_delta(15.0), 0);
    assert_eq!(wheel.pixel_delta(15.0), 1);
    // The 0.125 left over waits for the next one.
    assert_eq!(wheel.pixel_delta(0.0), 0);
    assert_eq!(wheel.pixel_delta(PIXELS_PER_NOTCH * 0.875), 1);
}

#[test]
fn a_large_pixel_delta_keeps_its_fraction() {
    let mut wheel = WheelAccumulator::default();
    // 100 px is 2.5 notches: two now, the half kept.
    assert_eq!(wheel.pixel_delta(100.0), 2);
    assert_eq!(wheel.pixel_delta(20.0), 1);
}

#[test]
fn the_fraction_keeps_the_sign_of_the_motion() {
    let mut wheel = WheelAccumulator::default();
    assert_eq!(wheel.pixel_delta(-30.0), 0);
    assert_eq!(wheel.pixel_delta(-30.0), -1);
    // Reversing cancels the kept fraction instead of stepping the other way.
    let mut wheel = WheelAccumulator::default();
    assert_eq!(wheel.pixel_delta(30.0), 0);
    assert_eq!(wheel.pixel_delta(-30.0), 0);
    assert_eq!(wheel.pixel_delta(-40.0), -1);
}

#[test]
fn a_new_gesture_drops_the_leftover_fraction() {
    let mut wheel = WheelAccumulator::default();
    assert_eq!(wheel.pixel_delta(30.0), 0);
    wheel.start_gesture();
    assert_eq!(wheel.pixel_delta(15.0), 0);
    assert_eq!(wheel.pixel_delta(15.0), 0);
}

#[test]
fn non_finite_deltas_are_ignored() {
    let mut wheel = WheelAccumulator::default();
    assert_eq!(wheel.pixel_delta(f64::NAN), 0);
    assert_eq!(wheel.pixel_delta(f64::INFINITY), 0);
    assert_eq!(wheel.pixel_delta(PIXELS_PER_NOTCH), 1);
}

#[test]
fn a_notch_is_wheel_delta_units() {
    assert_eq!(wheel_event_delta(1), WHEEL_DELTA);
    assert_eq!(wheel_event_delta(-2), -2 * WHEEL_DELTA);
    assert_eq!(wheel_event_delta(0), 0);
}
