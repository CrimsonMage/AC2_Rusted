//! Whole-step behaviour on synthetic terrain (no game data).

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::terrain::Terrain;
use crate::test_support::{animate, body_at, terrain};

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
        animate(body, PHYSICS_QUANTUM);
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
    animate(&mut body, 0.2);
    assert_eq!(body.current_element(), ELEMENT_GROUND);
    animate(&mut body, 0.2);
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

#[test]
fn the_posed_bone_follows_the_playing_behaviours() {
    use ac2_anim::{Ac2Bone, Ac2PlacementFrame, Ac2Setup, Ac2Transform};
    use std::sync::Arc;

    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    let mut animation = crate::test_support::body_animation();
    animation.setup = Some(Arc::new(Ac2Setup {
        meshes: Arc::from(Vec::new()),
        bones: Arc::from(vec![Ac2Bone {
            parent: None,
            name: 0,
        }]),
        placements: Arc::from(vec![Ac2PlacementFrame {
            kind: 0,
            bones: Arc::from(vec![Ac2Transform::IDENTITY]),
        }]),
        holding_points: Arc::from(Vec::new()),
        connection_points: Arc::from(Vec::new()),
        degrade: Default::default(),
        bounds_min: ac2_math::Vec3::ZERO,
        bounds_max: ac2_math::Vec3::ZERO,
    }));
    body.set_animation(animation);
    assert_eq!(
        body.posed_parts()[0].offset.x,
        0.0,
        "rest pose before animating"
    );
    run_animated(&mut body, &ground, 0.5);
    assert_eq!(body.posed_parts()[0].offset.x, 0.25, "idle clip");
    body.do_motion([0.0, 1.0, 0.0]);
    run_animated(&mut body, &ground, 1.0);
    // The move layer (1) sits above idle (0) and takes the bone once fully ramped in.
    assert_eq!(body.posed_parts()[0].offset.x, 0.5);
}

#[test]
fn the_drawn_position_blends_the_last_quantum() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    run(&mut body, &ground, 1.0);
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    run(&mut body, &ground, 1.0);
    let before = body.position();
    body.step_quantum(&ground);
    let (previous, current) = (body.previous_position(), body.position());
    assert_eq!(previous, before);
    assert!(current.frame.origin.y > previous.frame.origin.y);
    assert_eq!(
        body.drawing_position(0.0).frame.origin,
        previous.frame.origin
    );
    assert_eq!(
        body.drawing_position(1.0).frame.origin,
        current.frame.origin
    );
    let half = body.drawing_position(0.5).frame.origin.y;
    let mid = (previous.frame.origin.y + current.frame.origin.y) * 0.5;
    assert!((half - mid).abs() < 1e-5, "{half} {mid}");
    // A placement does not blend: the previous position is the placed one.
    let mut there = current;
    there.frame.origin.x += 3.0;
    let placed = body.place(there, 0, &ground);
    assert_eq!(placed, crate::physics_obj_place::SetPositionError::Ok);
    assert_eq!(body.previous_position(), body.position());
}

#[test]
fn an_immobile_body_is_forced_into_its_cell_and_stays_put() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    body.set_mobile(false);
    assert!(!body.is_mobile());
    // Below the ground: a placement check would fail or lift it; ForceIntoCell takes it as is.
    let mut sunk = body.position();
    sunk.frame.origin.z = 40.0;
    let placed = body.place(sunk, 0, &ground);
    assert_eq!(placed, crate::physics_obj_place::SetPositionError::Ok);
    assert_eq!(body.position().frame.origin.z, 40.0);
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    run(&mut body, &ground, 1.0);
    assert_eq!(body.position(), sunk);
}

/// `CPhysicsObj::SetVelocityScale` -> `CMotionInterp::SetVelocityScale`: the forward speed
/// limit (`max_forward_velocity` times the scale) becomes the held speed.
#[test]
fn a_velocity_scale_from_the_server_sets_the_walking_speed() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 30.0, 50.0);
    body.set_velocity_scale(0.5);
    assert_eq!(body.tuning().velocity_scale, 0.5);
    run(&mut body, &ground, 1.0);
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        ..MovementInput::default()
    });
    run(&mut body, &ground, 3.0);
    let state = body.movement_state();
    assert!((state.velocity[1] - 2.0).abs() < 1e-3, "{state:?}");
}

/// `CPhysicsObj::SetJumpScale`: the jump impulse is `m_fJumpScale` times the table's; a negative
/// scale is refused and keeps the last one.
#[test]
fn a_jump_scale_from_the_server_scales_the_jump_and_refuses_negatives() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_jump_scale(2.0);
    assert_eq!(body.tuning().jump_scale, 2.0);
    body.set_jump_scale(-1.0);
    assert_eq!(body.tuning().jump_scale, 2.0);
    run(&mut body, &ground, 0.3);
    run(&mut body, &ground, 0.5);
    assert!(body.jump());
    assert_eq!(body.movement_state().velocity[2], 10.0);
}

/// `CPhysicsObj::SetHoverHeight` stores the height; `ResetHoverHeight` returns to the table's
/// (-1 here).
#[test]
fn a_hover_height_from_the_server_is_kept_until_reset() {
    let mut body = body_at(70.0, 70.0, 50.0);
    assert_eq!(body.tuning().hover_height, -1.0);
    body.set_hover_height(1.5);
    assert_eq!(body.tuning().hover_height, 1.5);
    body.reset_hover_height();
    assert_eq!(body.tuning().hover_height, -1.0);
    // The acceleration scale stores as given.
    body.set_acceleration_scale(0.25);
    assert_eq!(body.tuning().acceleration_scale, 0.25);
}

/// `CPhysicsObj::DoFX` and `StopFX` queue the FX manager's start and stop (its sound part is the
/// client's): the events come out of the body's animation events in order.
#[test]
fn a_server_fx_starts_and_stops_through_the_animation_events() {
    use ac2_anim::Ac2AnimEvent;
    let mut body = body_at(70.0, 70.0, 50.0);
    assert!(body.do_fx(0x4000_0010, 0.5));
    assert!(body.stop_fx(0x4000_0010));
    let fx: Vec<Ac2AnimEvent> = body
        .take_animation_events()
        .into_iter()
        .filter(|e| {
            matches!(
                e,
                Ac2AnimEvent::StartFx { .. } | Ac2AnimEvent::StopFx { .. }
            )
        })
        .collect();
    assert_eq!(
        fx,
        vec![
            Ac2AnimEvent::StartFx {
                fx: 0x4000_0010,
                intensity: 0.5
            },
            Ac2AnimEvent::StopFx { fx: 0x4000_0010 },
        ]
    );
}

/// A position in the body's cell at `origin`, upright.
fn place_like(body: &PhysicsBody, origin: [f32; 3]) -> crate::position::Position {
    crate::position::Position::new(
        body.position().cell_id,
        crate::frame::Frame::new(
            crate::vector::Vector::new(origin[0], origin[1], origin[2]),
            crate::vector::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    )
}

/// `CPhysicsObj::AddImpulse`: the world direction adds to the velocity, and the body leaves
/// contact so the next quantum falls from it.
#[test]
fn an_impulse_adds_to_the_velocity_and_leaves_contact() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    run(&mut body, &ground, 0.5);
    assert!(body.movement_state().contact);
    body.set_velocity([1.0, 0.0, 0.0]);
    body.add_impulse([0.5, -2.0, 4.0]);
    let state = body.movement_state();
    assert_eq!(state.velocity, [1.5, -2.0, 4.0]);
    assert!(!state.contact, "{state:?}");
}

/// `CPhysicsObj::AddTargetedImpulse`: the push points from the source to the body, `magnitude`
/// long, with `vertical` added upward.
#[test]
fn a_targeted_impulse_pushes_away_from_its_source() {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_velocity([0.0; 3]);
    let source = place_like(&body, [67.0, 70.0, 50.0]);
    body.add_targeted_impulse(&source, 5.0, false, 1.0);
    let v = body.movement_state().velocity;
    assert!((v[0] - 5.0).abs() < 1e-4, "{v:?}");
    assert!(v[1].abs() < 1e-4 && (v[2] - 1.0).abs() < 1e-4, "{v:?}");
}

/// With falloff the magnitude is divided by the distance again: 5 at 3 m reads 5 / 3.
#[test]
fn a_targeted_impulse_with_falloff_weakens_with_distance() {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_velocity([0.0; 3]);
    let source = place_like(&body, [67.0, 70.0, 50.0]);
    body.add_targeted_impulse(&source, 5.0, true, 0.0);
    let v = body.movement_state().velocity;
    assert!((v[0] - 5.0 / 3.0).abs() < 1e-4, "{v:?}");
}

/// A source on the body (within `F_EPSILON`) has no direction: only the vertical part applies.
#[test]
fn a_targeted_impulse_from_the_body_itself_is_vertical_only() {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_velocity([0.0; 3]);
    let source = place_like(&body, [70.0, 70.0, 50.0]);
    body.add_targeted_impulse(&source, 5.0, false, 2.5);
    assert_eq!(body.movement_state().velocity, [0.0, 0.0, 2.5]);
}

#[test]
fn ethereal_bits_accumulate_on_the_body_and_none_clears_them() {
    let mut body = body_at(70.0, 70.0, 53.0);
    assert_eq!(body.blocking_bits(), 0);
    body.apply_blocking_bits(1 << 4);
    body.apply_blocking_bits(1 << 5);
    assert_eq!(body.blocking_bits(), (1 << 4) | (1 << 5));
    body.apply_blocking_bits(0);
    assert_eq!(body.blocking_bits(), (1 << 4) | (1 << 5));
}

fn pusher_vector(name: u32, v: [f32; 3]) -> ac2_db::Ac2PropertyValue {
    let mut bytes = Vec::new();
    for c in v {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    ac2_db::Ac2PropertyValue {
        name,
        value_type: 0x4000_0004,
        bytes: bytes.into_boxed_slice(),
    }
}

#[test]
fn a_general_velocity_pusher_carries_a_grounded_body() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    run(&mut body, &ground, 1.0);
    let before = body.position().frame.origin;
    // `CollideProperty` 0x40000042: a general velocity of 1 m/s along +x.
    assert!(body.add_pusher_properties(9, None, &[pusher_vector(0x4000_0042, [1.0, 0.0, 0.0])]));
    run(&mut body, &ground, 1.0);
    let moved = body.position().frame.origin.x - before.x;
    assert!((moved - 1.0).abs() < 0.05, "moved {moved}");
    assert!(body.movement_state().contact, "still on the ground");
    // Once the object is gone the push stops.
    assert!(body.remove_pusher(9));
    let at = body.position().frame.origin.x;
    run(&mut body, &ground, 0.5);
    assert!((body.position().frame.origin.x - at).abs() < 1e-3);
}

#[test]
fn an_upward_pusher_lifts_a_body_off_the_ground_without_gravity() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 50.0);
    run(&mut body, &ground, 1.0);
    // `CollideProperty` 0x40000042 with +z: it is lifting (above F_EPSILON).
    body.add_pusher_properties(3, None, &[pusher_vector(0x4000_0042, [0.0, 0.0, 2.0])]);
    run(&mut body, &ground, 1.0);
    let state = body.movement_state();
    assert!(!state.contact, "{state:?}");
    assert!(state.origin[2] > 51.0, "{state:?}");
}
