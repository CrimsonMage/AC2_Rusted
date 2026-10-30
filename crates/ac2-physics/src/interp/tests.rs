//! Remote-object interpolation on synthetic terrain (no game data).

use crate::body::{PhysicsBody, PHYSICS_QUANTUM};
use crate::frame::Frame;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::test_support::{body_at, terrain};
use crate::vector::{Quaternion, Vector};

/// A position on the test block at `(x, y)` on the 50 m plateau.
fn at(body: &PhysicsBody, x: f32, y: f32) -> Position {
    let mut pos = body.position();
    pos.frame = Frame::new(Vector::new(x, y, 50.0), Quaternion::IDENTITY);
    pos
}

/// Quanta at the physics rate with the interpolation clock following them from `start`.
fn run(body: &mut PhysicsBody, ground: &Terrain, start: f64, seconds: f32) -> f64 {
    let quanta = (seconds / PHYSICS_QUANTUM).round() as usize;
    let mut now = start;
    for _ in 0..quanta {
        now += f64::from(PHYSICS_QUANTUM);
        body.set_time(now);
        body.step_quantum(ground);
    }
    now
}

#[test]
fn a_remote_body_follows_the_positions_it_is_sent() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    let mut now = run(&mut body, &ground, 0.0, 1.0);
    // The sender walks north at 4 m/s; positions arrive every half second.
    for step in 1..=6 {
        let target = at(&body, 70.0, 30.0 + 2.0 * step as f32);
        body.interpolate_to(now, target, 0.0, [0.0, 1.0, 0.0], true, &ground);
        now = run(&mut body, &ground, now, 0.5);
    }
    assert!(body.is_interpolating());
    // The last position says the sender stopped.
    let last = at(&body, 70.0, 42.0);
    body.interpolate_to(now, last, 90.0, [0.0, 0.0, 0.0], true, &ground);
    run(&mut body, &ground, now, 3.0);
    let state = body.movement_state();
    let origin = body.position().frame.origin;
    assert!((origin.y - 42.0).abs() < 0.3, "{state:?}");
    assert!((origin.x - 70.0).abs() < 0.3, "{state:?}");
    assert!(state.contact, "{state:?}");
}

#[test]
fn a_position_beyond_fifteen_metres_is_jumped_to() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    let now = run(&mut body, &ground, 0.0, 1.0);
    let far = at(&body, 70.0, 60.0);
    body.interpolate_to(now, far, 0.0, [0.0, 0.0, 0.0], true, &ground);
    assert!(!body.is_interpolating());
    assert!((body.position().frame.origin.y - 60.0).abs() < 0.01);
}

#[test]
fn a_body_that_cannot_reach_the_path_blips_to_the_newest_position() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    let mut now = run(&mut body, &ground, 0.0, 1.0);
    // Frozen in place (movement frozen): no progress along the path for two seconds.
    body.set_physics_state(crate::physics_obj::DEFAULT_STATE | 0x2_0000);
    let target = at(&body, 70.0, 36.0);
    body.interpolate_to(now, target, 0.0, [0.0, 1.0, 0.0], true, &ground);
    now = run(&mut body, &ground, now, 1.0);
    assert!((body.position().frame.origin.y - 30.0).abs() < 0.01);
    body.set_physics_state(crate::physics_obj::DEFAULT_STATE);
    // Unfrozen two seconds later: no node passed in two seconds blips to the newest position,
    // which then keeps the sender's motion.
    now += 2.0;
    let mut quanta = 0;
    while body.is_interpolating() && quanta < 40 {
        now = run(&mut body, &ground, now, PHYSICS_QUANTUM);
        quanta += 1;
    }
    assert!(!body.is_interpolating());
    let state = body.movement_state();
    assert!((state.origin[1] - 36.0).abs() < 0.5, "{state:?}");
    assert_eq!(state.do_motion, [0.0, 1.0, 0.0]);
}

#[test]
fn a_queued_jump_is_taken_where_the_sender_jumped() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    let now = run(&mut body, &ground, 0.0, 1.0);
    let here = at(&body, 70.0, 30.0);
    body.interpolate_and_impulse(now, here, 0.0, [0.0, 0.0, 0.0], [0.0, 0.0, 5.0], true);
    run(&mut body, &ground, now, 0.3);
    assert_eq!(body.movement_state().times_jumped, 1);
}

#[test]
fn the_last_received_do_motion_is_the_newest_queued_one_while_interpolating() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    assert_eq!(body.remote_do_motion_values(), Vector::ZERO);
    let now = run(&mut body, &ground, 0.0, 0.5);
    let target = at(&body, 70.0, 32.0);
    body.interpolate_to(now, target, 0.0, [0.0, 1.0, 0.0], true, &ground);
    assert_eq!(body.remote_do_motion_values(), Vector::new(0.0, 1.0, 0.0));
}

#[test]
fn leaving_the_world_stops_interpolating_and_drops_contact() {
    // `CPhysicsObj::LeaveWorld` -> `CMotionInterp::HandleLeaveWorld`, `clear_transient_states`.
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    let now = run(&mut body, &ground, 0.0, 0.5);
    let target = at(&body, 70.0, 32.0);
    body.interpolate_to(now, target, 0.0, [0.0, 1.0, 0.0], true, &ground);
    assert!(body.is_interpolating());
    body.leave_world();
    assert!(!body.is_interpolating());
    assert!(!body.has_contact());
}
