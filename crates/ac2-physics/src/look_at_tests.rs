//! `LookAtManager`: the look's state machine, the ramp, the aim and the disinterest release.

use super::*;
use crate::frame::Frame;
use crate::vector::Vector;

const ME: u64 = 1;
const OTHER: u64 = 7;

fn at(x: f32, y: f32, z: f32) -> Position {
    let mut frame = Frame::IDENTITY;
    frame.origin = Vector::new(x, y, z);
    Position::new(0x0001_0001, frame)
}

fn me(is_player: bool) -> LookSelf {
    LookSelf {
        position: at(0.0, 0.0, 0.0),
        height: 1.8,
        is_player,
        head_x: 0.25,
    }
}

fn target(x: f32, y: f32) -> Option<LookTarget> {
    Some(LookTarget {
        position: at(x, y, 0.0),
        height: 1.0,
        mobile: false,
    })
}

#[test]
fn a_new_look_turns_on_and_ramps_from_zero() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.4);
    assert!(look.is_on);
    assert_eq!(look.target, OTHER);
    assert!(look.hold);
    assert_eq!(look.weight, 0.0);
    assert!((look.weight_vel - 2.0).abs() < 1e-6, "1 / ramp");
    assert_eq!(look.previous_yaw, 0.4);
}

#[test]
fn looking_at_the_body_itself_does_not_start_a_look() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(ME, LOOK_RAMP, 0.0);
    assert!(!look.is_on);
    assert_eq!(look.target, 0);
}

#[test]
fn release_ramps_the_weight_out() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    look.release(0.5);
    assert_eq!(look.target, 0);
    assert!(!look.is_on && !look.hold);
    assert!((look.weight_vel - 2.0).abs() < 1e-6);
    look.release(0.0);
    assert_eq!(look.weight, 1.0, "a zero ramp ends at once");
    assert_eq!(look.weight_vel, 0.0);
}

#[test]
fn the_weight_follows_the_ramp_in() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    look.update(0.25, &me(true), target(0.0, 10.0));
    assert!(
        (look.weight - 0.5).abs() < 1e-6,
        "a quarter of the ramp is half weight"
    );
    look.update(0.25, &me(true), target(0.0, 10.0));
    assert!(
        (look.weight - 1.0).abs() < 1e-6,
        "the ramp ends at full weight"
    );
}

#[test]
fn the_head_aims_at_a_target_off_to_one_side() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    // One quantum of 0.5 s reaches full weight (`time` = ramp, weight = q * weight_vel).
    let moves = look.update(0.5, &me(false), target(0.0, 10.0));
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].slider, LOOK_YAW_SLIDER);
    // atan2(10, 0) - pi/2: straight out of the body's +y side is a yaw of 0.
    assert!(moves[0].value.abs() < 1e-5, "yaw {}", moves[0].value);
    assert_eq!(moves[0].time, 0.0);
}

#[test]
fn a_target_ahead_on_the_x_axis_gives_the_retail_bearing() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    let moves = look.update(0.5, &me(false), target(10.0, 0.0));
    let expected = -std::f32::consts::FRAC_PI_2;
    assert!(
        (moves[0].value - expected).abs() < 1e-5,
        "{}",
        moves[0].value
    );
}

#[test]
fn a_non_player_holds_its_head_direction_when_turned_on() {
    let mut look = LookAtManager::default();
    assert!(
        look.update(0.1, &me(false), None).is_empty(),
        "a body with no look does nothing"
    );
    look.look_at_dir();
    let moves = look.update(0.1, &me(false), None);
    assert_eq!(
        moves,
        vec![SliderMove {
            slider: LOOK_YAW_SLIDER,
            value: 0.25,
            time: 0.5
        }]
    );
}

#[test]
fn a_head_direction_without_a_target_runs_for_half_a_second() {
    let mut look = LookAtManager::default();
    look.look_at_dir();
    assert_eq!(look.period, 0.5);
    assert!(look.is_on);
    // The period ends (its last ramp is 0.5 s too), so the direction stops holding.
    for _ in 0..10 {
        look.update(0.1, &me(false), None);
    }
    assert!(!look.is_on, "the head direction runs out");
}

#[test]
fn a_player_stops_looking_after_two_seconds_of_disinterest() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    // A target behind the player (local y below zero) is disinterest.
    for _ in 0..40 {
        look.update(0.1, &me(true), target(0.0, -10.0));
    }
    assert_eq!(look.target, 0, "the look is released");
    assert!(!look.is_on);
}

#[test]
fn a_target_in_front_keeps_the_look() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    for _ in 0..40 {
        look.update(0.1, &me(true), target(0.0, 10.0));
    }
    assert_eq!(look.target, OTHER);
    assert_eq!(look.time_disinterested, 0.0);
}

#[test]
fn look_at_dir_leaves_the_target_alone() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, 0.0);
    look.look_at_dir();
    assert_eq!(look.period, 1.0, "a held target keeps its period");
}
