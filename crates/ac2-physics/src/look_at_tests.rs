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
        head_z: -0.5,
        scale_x: 1.0,
        scale_z: 1.0,
        pitch: 0.0,
        yaw: 0.0,
        camera: None,
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
    look.look_at(OTHER, LOOK_RAMP, [0.4, 0.0, 0.6]);
    assert!(look.is_on);
    assert_eq!(look.target, OTHER);
    assert!(look.hold);
    assert_eq!(look.weight, 0.0);
    assert!((look.weight_vel - 2.0).abs() < 1e-6, "1 / ramp");
    assert_eq!(look.previous, [0.4, 0.0, 0.6]);
}

#[test]
fn looking_at_the_body_itself_does_not_start_a_look() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(ME, LOOK_RAMP, [0.0; 3]);
    assert!(!look.is_on);
    assert_eq!(look.target, 0);
}

#[test]
fn release_ramps_the_weight_out() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
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
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
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
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
    // One quantum of 0.5 s reaches full weight (`time` = ramp, weight = q * weight_vel).
    let moves = look.update(0.5, &me(false), target(0.0, 10.0));
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].slider, LOOK_YAW_SLIDER);
    // atan2(10, 0) - pi/2: straight out of the body's +y side is a yaw of 0.
    assert!(moves[0].value.abs() < 1e-5, "yaw {}", moves[0].value);
    assert_eq!(moves[0].time, 0.0);
    // The eye line (half the target's height, less the body's 1.8) over the 10 units, divided by
    // the full weight: -1.3 / 10.
    assert_eq!(moves[1].slider, LOOK_PITCH_SLIDER);
    assert!(
        (moves[1].value + 0.13).abs() < 1e-5,
        "pitch {}",
        moves[1].value
    );
    assert!((look.look_values.0 + 0.13).abs() < 1e-5);
}

#[test]
fn a_target_ahead_on_the_x_axis_gives_the_retail_bearing() {
    let mut look = LookAtManager {
        parent: ME,
        ..LookAtManager::default()
    };
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
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
        vec![
            SliderMove {
                slider: LOOK_PITCH_SLIDER,
                value: 0.25,
                time: 0.5
            },
            SliderMove {
                slider: LOOK_YAW_SLIDER,
                value: -0.5,
                time: 0.5
            }
        ]
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
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
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
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
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
    look.look_at(OTHER, LOOK_RAMP, [0.0; 3]);
    look.look_at_dir();
    assert_eq!(look.period, 1.0, "a held target keeps its period");
}

fn camera_me(offset: Vector) -> LookSelf {
    LookSelf {
        camera: Some(LookCamera {
            viewer_offset: offset,
            mouse_look: false,
        }),
        ..me(true)
    }
}

/// `Update`'s player branch: the camera behind the player leaves the head straight ahead.
#[test]
fn a_camera_behind_the_player_keeps_the_head_straight() {
    let mut look = LookAtManager::default();
    let moves = look.update(0.1, &camera_me(Vector::new(0.0, -3.5, 0.0)), None);
    assert_eq!(
        moves,
        vec![
            SliderMove {
                slider: LOOK_PITCH_SLIDER,
                value: 0.0,
                time: 0.1
            },
            SliderMove {
                slider: LOOK_YAW_SLIDER,
                value: 0.0,
                time: 0.1
            }
        ]
    );
}

/// A camera off the player's left: the left-right angle past the window is turned by pi
/// (`leftright + pi`), the step over 0.2 taking a fifth of a second; with the camera switches
/// off the head is straight.
#[test]
fn a_camera_to_the_side_turns_the_head_and_the_switches_stop_it() {
    let mut look = LookAtManager::default();
    let moves = look.update(0.1, &camera_me(Vector::new(-3.5, 0.0, 0.0)), None);
    assert_eq!(moves[1].slider, LOOK_YAW_SLIDER);
    // The weight `Update` ramps first (0.1 of the 0.5 ramp) blends the camera's share down.
    let expected = (1.0 - look.weight) * std::f32::consts::FRAC_PI_2;
    assert!(
        (moves[1].value - expected).abs() < 1e-5,
        "{} vs {expected}",
        moves[1].value
    );
    assert_eq!(moves[1].time, 0.2);
    look.camera_fk = false;
    let moves = look.update(0.1, &camera_me(Vector::new(-3.5, 0.0, 0.0)), None);
    assert!(moves.iter().all(|m| m.value == 0.0));
}
