use super::*;
use crate::test_support::{desc, desc_with_global, walking_values, RecordingRep};

fn ground_context(local_velocity: Vector) -> MotionContext {
    MotionContext {
        local_velocity,
        omega: Vector::ZERO,
        scale: Vector::new(1.0, 1.0, 1.0),
        height_above_walkable: 0.0,
        has_water: false,
        depth: 0.0,
        desired_element: ELEMENT_GROUND,
        element: ELEMENT_GROUND,
        is_sliding_down: false,
        constraint_factor: 1.0,
        vector_heading: Vector::new(0.0, 1.0, 0.0),
    }
}

fn walker() -> MotionInterp {
    MotionInterp::new(Some(desc(&[(ELEMENT_GROUND, walking_values())])))
}

#[test]
fn forward_input_accelerates_then_holds_the_forward_speed() {
    let mut interp = walker();
    interp.do_motion(Vector::new(0.0, 1.0, 0.0));
    interp.update_movement(
        0.1,
        &ground_context(Vector::ZERO),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.local_desired_vel, Vector::new(0.0, 4.0, 0.0));
    assert_eq!(interp.acceleration(), Vector::new(0.0, 10.0, 0.0));
    // One quantum short of the target the acceleration lands exactly on it.
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(0.0, 3.5, 0.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration().y, st((4.0 - 3.5) / ld(0.1)));
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(0.0, 4.0, 0.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration(), Vector::ZERO);
}

#[test]
fn releasing_input_brakes_with_the_stop_rate() {
    let mut interp = walker();
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(2.5, 3.0, 0.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration(), Vector::new(-20.0, -20.0, 0.0));
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(1.0, 1.0, 0.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration().y, st(-1.0 / ld(0.1)));
}

#[test]
fn backward_and_strafe_use_their_own_limits() {
    let mut interp = walker();
    interp.do_motion(Vector::new(1.0, -1.0, 0.0));
    interp.update_movement(
        0.1,
        &ground_context(Vector::ZERO),
        false,
        &mut RecordingRep::default(),
    );
    // The input is normalised: both axes at 1/sqrt(2).
    let half = std::f32::consts::FRAC_1_SQRT_2;
    assert!((interp.local_desired_vel.y - -2.0 * half).abs() < 1e-6);
    assert!((interp.local_desired_vel.x - 3.0 * half).abs() < 1e-6);
    // Running forward faster than 0.7 strafes at the forward speed.
    interp.do_motion(Vector::new(0.5, 0.8, 0.0));
    interp.update_movement(
        0.1,
        &ground_context(Vector::ZERO),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.local_desired_vel.x, st(ld(0.5) * 4.0));
}

#[test]
fn turning_ramps_the_turn_rate_by_speed() {
    let mut interp = walker();
    interp.update_movement(
        0.1,
        &ground_context(Vector::ZERO),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.max_turn_vel, 60.0);
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(0.0, 2.0, 0.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.max_turn_vel, 75.0);
    interp.set_omega_velocity(1.0);
    interp.set_omega_velocity(1.0);
    let clamp = interp.calculate_omega_acceleration(&ground_context(Vector::ZERO));
    assert_eq!(clamp.omega_z, 75.0);
    assert_eq!(interp.omega_acceleration().z, 360.0);
    // The desired rate is consumed: with no new input the turn brakes.
    let mut spinning = ground_context(Vector::ZERO);
    spinning.omega = Vector::new(0.0, 0.0, 40.0);
    let clamp = interp.calculate_omega_acceleration(&spinning);
    assert_eq!(clamp.omega_z, 0.0);
    assert_eq!(interp.omega_acceleration().z, -360.0);
}

#[test]
fn ground_contact_cancels_vertical_velocity() {
    let mut interp = walker();
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(0.0, 0.0, -2.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration().z, 20.0);
    // Moving across the ground the cancellation is limited to 3 m/s^2.
    interp.update_movement(
        0.1,
        &ground_context(Vector::new(0.0, 1.0, -2.0)),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration().z, 3.0);
    let mut airborne = ground_context(Vector::new(0.0, 0.0, -2.0));
    airborne.height_above_walkable = 1.5;
    interp.update_movement(0.1, &airborne, false, &mut RecordingRep::default());
    assert_eq!(interp.acceleration().z, 0.0);
}

#[test]
fn nan_or_oversized_input_is_sanitised() {
    let mut interp = walker();
    interp.do_motion(Vector::new(f32::NAN, 1.0, 0.0));
    assert_eq!(interp.last_do_motion(), Vector::ZERO);
    interp.do_motion(Vector::new(0.0, 3.0, 4.0));
    let motion = interp.last_do_motion();
    assert!((motion.y - 0.6).abs() < 1e-7 && (motion.z - 0.8).abs() < 1e-7);
}

#[test]
fn hovering_bodies_brake_onto_their_hover_height() {
    let mut values = walking_values();
    values.gravity = false;
    values.hover_height = 2.0;
    values.vertical_acceleration = 5.0;
    values.max_vertical_velocity = 3.0;
    let mut interp = MotionInterp::new(Some(desc(&[(ELEMENT_GROUND, values)])));
    let mut low = ground_context(Vector::ZERO);
    low.height_above_walkable = 1.0;
    interp.update_movement(0.1, &low, false, &mut RecordingRep::default());
    assert_eq!(interp.acceleration().z, 5.0);
    let mut rising = ground_context(Vector::new(0.0, 0.0, 3.0));
    rising.height_above_walkable = 1.9;
    interp.update_movement(0.1, &rising, false, &mut RecordingRep::default());
    // Would overshoot: decelerate at v^2 / 2d, limited to the vertical acceleration.
    assert_eq!(interp.acceleration().z, -5.0);
}

#[test]
fn missing_desc_moves_nothing() {
    let mut interp = MotionInterp::new(None);
    interp.do_motion(Vector::new(0.0, 1.0, 0.0));
    interp.update_movement(
        0.1,
        &ground_context(Vector::ZERO),
        false,
        &mut RecordingRep::default(),
    );
    assert_eq!(interp.acceleration(), Vector::ZERO);
    assert!(interp.motion_values(ELEMENT_GROUND).gravity);
}

#[test]
fn turn_to_spins_the_short_way_and_clamps_on_arrival() {
    let mut interp = walker();
    interp.update_movement(
        0.1,
        &ground_context(Vector::ZERO),
        false,
        &mut RecordingRep::default(),
    );
    // Facing north (heading 0); turn to west (270): counter-clockwise, positive yaw rate.
    interp.turn_to(Heading::from_degrees(270.0));
    let clamp = interp.calculate_omega_acceleration(&ground_context(Vector::ZERO));
    assert_eq!(clamp.omega_z, 60.0);
    assert!(clamp.heading.is_some());
    // East (90): clockwise.
    interp.turn_to(Heading::from_degrees(90.0));
    let clamp = interp.calculate_omega_acceleration(&ground_context(Vector::ZERO));
    assert_eq!(clamp.omega_z, -60.0);
    // Already facing it and still: the turn ends.
    interp.turn_to(Heading::from_degrees(0.0));
    let clamp = interp.calculate_omega_acceleration(&ground_context(Vector::ZERO));
    assert_eq!((clamp.omega_z, clamp.heading), (0.0, None));
}

/// `CMotionInterp::SetGlobalModifier`: the motion values lookup keys on `mGlobalMod`, so the
/// global modifier's values replace the default ones once it is set.
#[test]
fn a_global_modifier_selects_its_motion_values() {
    let default = walking_values();
    let global = Ac2MotionValues {
        forward_acceleration: 2.0,
        ..walking_values()
    };
    let mut interp = MotionInterp::new(Some(desc_with_global(&default, 7, &global)));
    assert_eq!(
        interp.motion_values(ELEMENT_GROUND).forward_acceleration,
        10.0
    );
    interp.set_global_modifier(7);
    assert_eq!(
        interp.motion_values(ELEMENT_GROUND).forward_acceleration,
        2.0
    );
    interp.set_global_modifier(0);
    assert_eq!(
        interp.motion_values(ELEMENT_GROUND).forward_acceleration,
        10.0
    );
}
