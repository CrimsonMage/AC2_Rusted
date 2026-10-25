//! The animation entry points of other bodies and the player: modes, sliders, the
//! `PhysicsDesc` animation, `RecvEvt_MoveTo` and the distance-degraded animation step.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{Ac2AnimEvent, Ac2BehaviorParams, Ac2CharTemplate};

use crate::body::{PhysicsBody, PHYSICS_QUANTUM};
use crate::moveto::{MovementParameters, PackedMovementParameters, MOVE_TO_MOVE};
use crate::physics_obj::DescSlider;
use crate::test_support::{body_animation, body_at, terrain};

const OPACITY: u32 = 0x4000_0015;
const RED: u32 = 0x4000_0016;
const MOVE: u32 = 0x4000_0001;
/// A slider the test template leaves to the client, and one the weenie drives.
const CLIENT_SLIDER: u32 = 0x4000_0030;
const WEENIE_SLIDER: u32 = 0x4000_0031;

fn remote_body() -> PhysicsBody {
    let mut body = body_at(70.0, 30.0, 50.0);
    body.set_player(false);
    let mut animation = body_animation();
    let mut template: Ac2CharTemplate = (*animation.template).clone();
    template.sliders = HashMap::from([(CLIENT_SLIDER, 0), (WEENIE_SLIDER, 1)]);
    animation.template = Arc::new(template);
    body.set_animation(animation);
    body
}

#[test]
fn bodies_are_not_the_player_until_marked() {
    let mut body = remote_body();
    assert!(!body.is_player());
    let ground = terrain(|_, _| 50.0);
    let params = MovementParameters {
        flags: MOVE_TO_MOVE,
        target_position: body.position(),
        ..MovementParameters::default()
    };
    assert!(
        !body.recv_move_to(params.clone()),
        "only the player moves to"
    );
    body.step_quantum(&ground);
    assert!(!body.is_moving_to());
    body.set_player(true);
    body.set_animation(body_animation());
    assert!(body.is_player(), "a new behaviour table keeps the flag");
    assert!(body.recv_move_to(params));
}

#[test]
fn the_player_keeps_its_client_sliders_from_the_server() {
    let mut body = remote_body();
    assert!(body.recv_do_slider(CLIENT_SLIDER, 0.5, 0.0));
    assert_eq!(body.slider_value(CLIENT_SLIDER), 0.5);
    body.set_player(true);
    assert!(!body.recv_do_slider(CLIENT_SLIDER, 0.25, 0.0));
    assert_eq!(body.slider_value(CLIENT_SLIDER), 0.5);
    assert!(body.recv_do_slider(WEENIE_SLIDER, 0.25, 0.0));
    assert_eq!(body.slider_value(WEENIE_SLIDER), 0.25);
}

#[test]
fn colour_and_opacity_sliders_set_the_animation_colour() {
    let mut body = remote_body();
    assert_eq!(body.anim_color(), [1.0; 4]);
    body.update_animation(0.5, 1.0);
    body.do_slider(0, RED, 1.5, 0.0);
    assert_eq!(body.anim_color()[0], 1.0, "capped");
    body.do_slider(0, RED, 0.25, 0.0);
    assert_eq!(body.anim_color()[0], 0.25, "at once");
    body.do_slider(7, OPACITY, 0.0, 1.0);
    assert_eq!(body.anim_opacity(), 1.0, "moves with the slider tracker");
    assert!(body
        .take_animation_events()
        .contains(&Ac2AnimEvent::SliderDone {
            context: 7,
            error: 0
        }));
    body.update_animation(0.5, 1.5);
    assert!(
        (body.anim_opacity() - 0.5).abs() < 1e-6,
        "{}",
        body.anim_opacity()
    );
}

#[test]
fn far_bodies_refresh_their_sliders_less_often() {
    let mut body = remote_body();
    body.set_min_visible_distance(40.0);
    // A quarter of 40 m is 10 m; at 35 m the step is (35 - 10) / (40 - 10) * 0.25 s.
    body.set_view_distance(35.0);
    body.do_slider(0, OPACITY, 0.0, 10.0);
    let mut now = 5.0;
    body.update_animation(PHYSICS_QUANTUM, now);
    assert_eq!(
        body.slider_value(OPACITY),
        0.5,
        "the first update moves 5 s worth"
    );
    // From 0.5 to 1 over 10 s: 0.05 per second.
    body.do_slider(0, OPACITY, 1.0, 10.0);
    let mut values = Vec::new();
    for _ in 0..12 {
        now += 0.05;
        body.update_animation(0.05, now);
        values.push(body.slider_value(OPACITY));
    }
    // Updates land after 0.25 s (the first frame at least 0.2083 s on) and 0.5 s.
    let changes: Vec<usize> = (1..values.len())
        .filter(|&i| values[i] != values[i - 1])
        .collect();
    assert_eq!(changes, vec![4, 9], "{values:?}");
    assert!((values[4] - 0.5125).abs() < 1e-5, "{values:?}");
    body.set_view_distance(50.0);
    let before = body.slider_value(OPACITY);
    for _ in 0..10 {
        now += 0.05;
        body.update_animation(0.05, now);
    }
    assert_eq!(
        body.slider_value(OPACITY),
        before,
        "past the visible distance"
    );
    body.set_player(true);
    now += 0.05;
    body.update_animation(0.05, now);
    assert!(body.slider_value(OPACITY) == before, "still past it");
    body.set_view_distance(35.0);
    now += 0.05;
    body.update_animation(0.05, now);
    assert!(
        body.slider_value(OPACITY) > before,
        "the player updates every step"
    );
}

#[test]
fn a_description_sets_mode_behaviours_and_sliders() {
    let mut body = remote_body();
    body.do_slider(0, WEENIE_SLIDER, 0.7, 0.0);
    let mut now = 1.0;
    let mut animate = |body: &mut PhysicsBody, seconds: f32| {
        for _ in 0..(seconds / 0.05).round() as usize {
            now += 0.05;
            body.update_animation(0.05, now);
        }
    };
    animate(&mut body, 0.05);
    let walk = Ac2BehaviorParams::new(MOVE);
    let sliders = [DescSlider {
        slider: OPACITY,
        value: 0.0,
        velocity: 0.5,
    }];
    body.set_description_animation(crate::motion_interp::MODE_DEFAULT, &[walk], &sliders);
    assert!(body.is_doing_behavior(MOVE));
    assert_eq!(body.slider_desired_value(OPACITY), 0.0);
    assert_eq!(body.slider_desired_value(WEENIE_SLIDER), 0.0, "undescribed");
    assert_eq!(
        body.slider_value(WEENIE_SLIDER),
        0.0,
        "without velocity: at once"
    );
    animate(&mut body, 1.0);
    assert!(
        (body.slider_value(OPACITY) - 0.5).abs() < 1e-5,
        "1 to 0 over |1 - 0| / 0.5 = 2 s"
    );
    body.set_description_animation(crate::motion_interp::MODE_DEFAULT, &[], &sliders);
    animate(&mut body, 1.0);
    assert!(!body.is_doing_behavior(MOVE), "no longer described");
}

#[test]
fn a_server_move_to_unpacks_over_the_defaults() {
    let target = body_at(80.0, 30.0, 50.0).position();
    let params = MovementParameters::unpack(PackedMovementParameters {
        flags: MOVE_TO_MOVE,
        desired_velocity: Some(0.5),
        fail_distance: Some(20.0),
        desired_distance: Some(0.25),
        max_success_distance: Some(3.0),
        target_position: Some(target),
        positions: vec![target],
        context: 3,
        ..PackedMovementParameters::default()
    });
    assert_eq!(
        params.flags, MOVE_TO_MOVE,
        "the default use-cylinders flag is replaced"
    );
    assert_eq!(params.min_success_distance, 100.0, "not sent: the default");
    assert_eq!(
        (params.desired_velocity, params.max_success_distance),
        (0.5, 3.0)
    );
    assert_eq!(params.target_object, None);
    assert_eq!(params.position_list.len(), 1);
    let mut body = body_at(70.0, 30.0, 50.0);
    assert!(body.recv_move_to(params));
    assert!(body.is_moving_to());
}
