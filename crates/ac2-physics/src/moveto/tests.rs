//! The player's move-to and turn-to on synthetic terrain (no game data).

use super::*;
use crate::body::{PhysicsBody, PHYSICS_QUANTUM};
use crate::frame::Frame;
use crate::terrain::Terrain;
use crate::test_support::{body_at, terrain};
use crate::vector::Quaternion;

fn at(body: &PhysicsBody, x: f32, y: f32, heading: f32) -> Position {
    let mut pos = body.position();
    let mut frame = Frame::new(Vector::new(x, y, 50.0), Quaternion::IDENTITY);
    frame.set_heading(Heading::from_degrees(heading));
    pos.frame = frame;
    pos
}

fn run(body: &mut PhysicsBody, ground: &Terrain, seconds: f32) {
    for _ in 0..(seconds / PHYSICS_QUANTUM).round() as usize {
        body.step_quantum(ground);
    }
}

fn walk_to(target: Position) -> MovementParameters {
    MovementParameters {
        flags: MOVE_TO_MOVE | MOVE_TO_USE_CYLINDERS,
        desired_distance: 0.5,
        max_success_distance: 2.0,
        target_position: target,
        context: 9,
        ..MovementParameters::default()
    }
}

#[test]
fn a_move_to_walks_up_to_the_target_and_stops_short_of_it() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    body.set_radius(0.4);
    run(&mut body, &ground, 0.5);
    let target = at(&body, 70.0, 40.0, 0.0);
    assert!(body.move_to(walk_to(target)));
    assert!(body.is_moving_to());
    run(&mut body, &ground, 6.0);
    let state = body.movement_state();
    let gap = 40.0 - state.origin[1] - 0.4;
    assert!((0.0..0.6).contains(&gap), "{state:?}");
    assert_eq!(state.do_motion, [0.0, 0.0, 0.0]);
    // Retail never ends a reached move-to by itself; the caller interrupts it.
    assert!(body.is_moving_to());
    body.interrupt_move_to();
    let results = body.take_move_to_results();
    assert_eq!(results.len(), 1);
    assert_eq!((results[0].context, results[0].error), (9, 0x36));
}

#[test]
fn a_turn_to_faces_the_target_heading() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    run(&mut body, &ground, 0.5);
    let params = MovementParameters {
        flags: MOVE_TO_TURN,
        target_position: at(&body, 70.0, 30.0, 90.0),
        ..MovementParameters::default()
    };
    assert!(body.move_to(params));
    run(&mut body, &ground, 3.0);
    let heading = body.movement_state().heading_degrees;
    assert!((heading - 90.0).abs() < 10.0, "{heading}");
}

#[test]
fn waypoints_are_walked_in_order() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    run(&mut body, &ground, 0.5);
    // With a list the target position is not used: each reached waypoint pops the next.
    let mut params = walk_to(at(&body, 70.0, 99.0, 0.0));
    params.position_list = [at(&body, 70.0, 36.0, 0.0), at(&body, 70.0, 42.0, 0.0)]
        .into_iter()
        .collect();
    assert!(body.move_to(params));
    run(&mut body, &ground, 2.0);
    let y = body.movement_state().origin[1];
    assert!(y < 36.5, "first the waypoint: {y}");
    run(&mut body, &ground, 4.0);
    let y = body.movement_state().origin[1];
    assert!(y > 40.5, "then the target: {y}");
}

#[test]
fn a_move_to_without_move_or_turn_ends_at_once() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    run(&mut body, &ground, 0.5);
    let params = MovementParameters {
        flags: 0,
        context: 4,
        ..MovementParameters::default()
    };
    assert!(body.move_to(params));
    assert!(!body.is_moving_to());
    let results = body.take_move_to_results();
    assert_eq!((results[0].context, results[0].error), (4, 0));
}

fn reported(position: Position, status: crate::target::TargetStatus) -> crate::target::TargetInfo {
    crate::target::TargetInfo {
        object_id: 7,
        target_position: position,
        object_radius: 0.4,
        object_height: 1.8,
        status,
        ..Default::default()
    }
}

fn walk_to_object() -> MovementParameters {
    MovementParameters {
        flags: MOVE_TO_MOVE | MOVE_TO_USE_CYLINDERS | MOVE_TO_OBJECT,
        target_object: Some(7),
        ..walk_to(Position::default())
    }
}

#[test]
fn a_move_to_an_object_waits_for_the_target_managers_first_word() {
    use crate::target::TargetStatus;
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    body.set_radius(0.4);
    run(&mut body, &ground, 0.5);
    let target = at(&body, 70.0, 40.0, 0.0);
    assert!(body.move_to(walk_to_object()));
    // `MoveToManager::UpdateMoveTo`: status undefined, so it stands still.
    run(&mut body, &ground, 1.0);
    assert!((body.movement_state().origin[1] - 30.0).abs() < 0.01);
    assert!(body.is_moving_to());
    // The voyeur update arrives (`UpdateTargetInfo`) and it walks.
    body.update_target_info(&reported(target, TargetStatus::Ok));
    run(&mut body, &ground, 2.0);
    assert!(body.movement_state().origin[1] > 32.0);
}

#[test]
fn a_target_manager_status_other_than_ok_ends_the_move_to_with_error_0x37() {
    use crate::target::TargetStatus;
    for status in [
        TargetStatus::ExitWorld,
        TargetStatus::Teleported,
        TargetStatus::TimedOut,
    ] {
        let ground = terrain(|_, _| 50.0);
        let mut body = body_at(70.0, 30.0, 50.0);
        run(&mut body, &ground, 0.5);
        let target = at(&body, 70.0, 60.0, 0.0);
        assert!(body.move_to(walk_to_object()));
        body.update_target_info(&reported(target, TargetStatus::Ok));
        run(&mut body, &ground, 0.5);
        body.update_target_info(&reported(target, status));
        run(&mut body, &ground, 0.2);
        assert!(!body.is_moving_to(), "{status:?}");
        let results = body.take_move_to_results();
        assert_eq!((results[0].context, results[0].error), (9, 0x37));
    }
}
