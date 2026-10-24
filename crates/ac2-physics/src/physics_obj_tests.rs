//! Whole-step behaviour on synthetic terrain (no game data).

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::frame::Frame;
use crate::landdefs::SIDE_VERTEX_COUNT;
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::physics_obj::BodyShape;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::test_support::{desc, walking_values};
use crate::vector::{Quaternion, Sphere, Vector};

const BLOCK: u32 = 0x7F7F_FFFF;

/// A block whose vertex heights are `height(x, y)` (vertex steps of 10 m).
fn terrain(height: impl Fn(usize, usize) -> f32) -> Terrain {
    let heights: Vec<f32> = (0..SIDE_VERTEX_COUNT)
        .flat_map(|x| (0..SIDE_VERTEX_COUNT).map(move |y| (x, y)))
        .map(|(x, y)| height(x, y))
        .collect();
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK, &heights, |_, _| false);
    terrain
}

/// Two stacked 0.4 m spheres, 1.8 m tall, 0.5 m steps.
fn shape() -> BodyShape {
    BodyShape {
        spheres: vec![
            Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4),
            Sphere::new(Vector::new(0.0, 0.0, 1.2), 0.4),
        ],
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    }
}

fn body_at(x: f32, y: f32, z: f32) -> PhysicsBody {
    let cell =
        crate::landdefs::lcoord_to_gid(0x7F * 8 + (x / 20.0) as i32, 0x7F * 8 + (y / 20.0) as i32);
    let mut values_air = walking_values();
    values_air.forward_acceleration = 0.0;
    values_air.backward_acceleration = 0.0;
    values_air.strafe_acceleration = 0.0;
    let desc = desc(&[
        (ELEMENT_GROUND, walking_values()),
        (ELEMENT_AIR, values_air),
    ]);
    PhysicsBody::new(
        Position::new(cell, Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY)),
        shape(),
        Some(desc),
    )
}

fn run(body: &mut PhysicsBody, terrain: &Terrain, seconds: f32) {
    let quanta = (seconds / PHYSICS_QUANTUM).round() as usize;
    for _ in 0..quanta {
        body.step_quantum(terrain);
    }
}

#[test]
fn a_dropped_body_falls_and_settles_on_flat_ground() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 53.0);
    run(&mut body, &ground, 0.3);
    let state = body.movement_state();
    assert!(
        !state.contact && state.velocity[2] < -2.0,
        "falling: {state:?}"
    );
    run(&mut body, &ground, 2.0);
    let state = body.movement_state();
    assert!(state.contact && state.on_walkable, "{state:?}");
    assert!((state.origin[2] - 50.0).abs() < 0.01, "{state:?}");
    assert!(state.velocity[2].abs() < 0.5, "{state:?}");
    assert_eq!(state.element, ELEMENT_GROUND);
}

#[test]
fn walking_forward_reaches_and_holds_the_forward_speed() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    run(&mut body, &ground, 1.0);
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    let start = body.position().frame.origin;
    run(&mut body, &ground, 3.0);
    let state = body.movement_state();
    assert!((state.velocity[1] - 4.0).abs() < 1e-3, "{state:?}");
    assert!(state.velocity[0].abs() < 1e-4);
    let travelled = body.position().frame.origin.y - start.y
        + 160.0 * ((state.cell_id >> 16) & 0xFF) as f32
        - 160.0 * 0x7F as f32;
    // 0.4 s to accelerate at 10 m/s^2 loses 0.8 m against holding 4 m/s for 3 s.
    assert!((travelled - 11.2).abs() < 0.3, "{travelled}");
    assert!(state.contact && state.on_walkable);
    assert!((state.origin[2] - 50.0).abs() < 0.01);
}

#[test]
fn walking_up_a_walkable_slope_keeps_ground_contact() {
    // Rises 3 m per 10 m north (about 17 degrees).
    let slope = terrain(|_, y| 50.0 + 3.0 * y as f32);
    let mut body = body_at(70.0, 30.0, 59.5);
    run(&mut body, &slope, 1.5);
    assert!(body.movement_state().contact, "{:?}", body.movement_state());
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    run(&mut body, &slope, 2.0);
    let state = body.movement_state();
    assert!(state.contact && state.on_walkable, "{state:?}");
    let ground = 50.0 + 0.3 * state.origin[1];
    assert!(
        (state.origin[2] - ground).abs() < 0.05,
        "{state:?} vs {ground}"
    );
    assert!(state.origin[1] > 35.0, "{state:?}");
}

#[test]
fn a_too_steep_slope_is_not_walkable() {
    // Rises 20 m per 10 m north (63 degrees): steeper than floor_z.
    let cliff = terrain(|_, y| 50.0 + if y >= 4 { 20.0 * (y - 4) as f32 } else { 0.0 });
    let mut body = body_at(70.0, 35.0, 50.0);
    run(&mut body, &cliff, 1.0);
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    run(&mut body, &cliff, 3.0);
    let state = body.movement_state();
    // The body cannot climb the face: it stays at its foot (y = 40).
    assert!(state.origin[1] < 41.0, "{state:?}");
    assert!(state.origin[2] < 52.0, "{state:?}");
}

#[test]
fn turning_to_a_heading_arrives_and_stops() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    run(&mut body, &ground, 0.5);
    body.set_input(MovementInput {
        heading: Some(90.0),
        ..MovementInput::default()
    });
    run(&mut body, &ground, 4.0);
    let state = body.movement_state();
    assert!(!state.turning_to, "the turn finished: {state:?}");
    let heading = state.heading_degrees;
    assert!((heading - 90.0).abs() < 1.5, "{heading}");
    assert!(body.movement_state().omega[2].abs() < 1e-3);
}

#[test]
fn a_turn_to_survives_input_without_a_heading_until_stopped() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    run(&mut body, &ground, 0.5);
    body.turn_to(180.0);
    run(&mut body, &ground, 0.3);
    body.set_input(MovementInput {
        motion: [0.0, 0.0, 0.0],
        ..MovementInput::default()
    });
    assert!(body.movement_state().turning_to);
    run(&mut body, &ground, 0.3);
    assert!(body.movement_state().turning_to);
    body.stop_turn_to();
    let state = body.movement_state();
    assert!(!state.turning_to);
    assert_eq!(state.turn_to_degrees, state.heading_degrees);
}

#[test]
fn set_heading_snaps_to_the_byte_steps() {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_heading(100.0);
    // 100 degrees is 70.8 of 255 steps: step 70 is 98.82 degrees.
    let heading = body.movement_state().heading_degrees;
    assert!((heading - 70.0 * 360.0 / 255.0).abs() < 1e-3, "{heading}");
}

#[test]
fn a_grounded_body_jumps_with_the_jump_impulse_and_lands_again() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    run(&mut body, &ground, 0.3);
    assert!(!body.jump(), "too soon after landing");
    run(&mut body, &ground, 0.5);
    assert!(body.jump());
    let state = body.movement_state();
    assert_eq!(state.velocity[2], 5.0);
    assert_eq!(state.times_jumped, 1);
    assert_eq!(state.element, ELEMENT_AIR);
    run(&mut body, &ground, 0.3);
    assert!(
        body.position().frame.origin.z > 50.8,
        "{:?}",
        body.movement_state()
    );
    run(&mut body, &ground, 1.5);
    let state = body.movement_state();
    assert!(
        state.contact && (state.origin[2] - 50.0).abs() < 0.01,
        "{state:?}"
    );
}

fn animated_body_at(x: f32, y: f32, z: f32) -> PhysicsBody {
    let mut body = body_at(x, y, z);
    body.set_animation(crate::test_support::body_animation());
    body
}

/// Physics quanta with an animation step after each, as the client frame runs them.
fn run_animated(body: &mut PhysicsBody, terrain: &Terrain, seconds: f32) {
    let quanta = (seconds / PHYSICS_QUANTUM).round() as usize;
    for _ in 0..quanta {
        body.step_quantum(terrain);
        body.update_animation(PHYSICS_QUANTUM);
    }
}

#[test]
fn an_animated_jump_waits_for_the_air_transition() {
    let ground = terrain(|_, _| 50.0);
    let mut body = animated_body_at(70.0, 70.0, 50.0);
    run_animated(&mut body, &ground, 0.8);
    body.take_animation_events();
    assert!(body.jump());
    assert_eq!(body.movement_state().element, ELEMENT_AIR);
    assert_eq!(body.current_element(), ELEMENT_GROUND, "still animating");
    body.update_animation(0.2);
    assert_eq!(body.current_element(), ELEMENT_GROUND);
    body.update_animation(0.2);
    assert_eq!(body.current_element(), ELEMENT_AIR);
    run_animated(&mut body, &ground, 2.0);
    let state = body.movement_state();
    assert!(
        state.contact && state.element == ELEMENT_GROUND,
        "{state:?}"
    );
    let fx: Vec<u32> = body
        .take_animation_events()
        .into_iter()
        .filter_map(|event| match event {
            ac2_anim::Ac2AnimEvent::StartFx { fx, .. } => Some(fx),
            _ => None,
        })
        .collect();
    assert_eq!(fx, vec![0x4000_0003, 0x4000_0002], "jump then land FX");
}

#[test]
fn walking_plays_the_move_behaviour_and_standing_stops_it() {
    let ground = terrain(|_, _| 50.0);
    let mut body = animated_body_at(70.0, 30.0, 50.0);
    run_animated(&mut body, &ground, 0.8);
    assert!(body.is_doing_behavior(0x4000_0002), "idle");
    assert!(!body.is_doing_behavior(0x4000_0001));
    body.do_motion([0.0, 1.0, 0.0]);
    run_animated(&mut body, &ground, 0.5);
    assert!(body.is_doing_behavior(0x4000_0001));
    body.do_motion([0.0, 0.0, 0.0]);
    run_animated(&mut body, &ground, 1.5);
    assert!(
        !body.is_doing_behavior(0x4000_0001),
        "{:?}",
        body.movement_state()
    );
}
