//! The animation entry points of other bodies and the player: modes, sliders, the
//! `PhysicsDesc` animation, `RecvEvt_MoveTo` and the distance-degraded animation step.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{Ac2AnimEvent, Ac2BehaviorParams, Ac2CharTemplate};
use ac2_core::Random;

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
    body.update_animation(0.5, 1.0, &mut Random::default());
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
    body.update_animation(0.5, 1.5, &mut Random::default());
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
    body.update_animation(PHYSICS_QUANTUM, now, &mut Random::default());
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
        body.update_animation(0.05, now, &mut Random::default());
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
        body.update_animation(0.05, now, &mut Random::default());
    }
    assert_eq!(
        body.slider_value(OPACITY),
        before,
        "past the visible distance"
    );
    body.set_player(true);
    now += 0.05;
    body.update_animation(0.05, now, &mut Random::default());
    assert!(body.slider_value(OPACITY) == before, "still past it");
    body.set_view_distance(35.0);
    now += 0.05;
    body.update_animation(0.05, now, &mut Random::default());
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
            body.update_animation(0.05, now, &mut Random::default());
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

#[test]
fn idle_effect_triggers_restart_each_cycle_and_hooks_fire_once() {
    use ac2_anim::{
        Ac2AnimAsset, Ac2AnimInfo, Ac2AnimTrigger, Ac2Bone, Ac2DbAnimator, Ac2PlacementFrame,
        Ac2Setup, Ac2Transform, Ac2WeenieHook,
    };

    let mut body = remote_body();
    let mut animation = body_animation();
    // Triggers come from the part array's animators (`CPartArray::AddAnimation`).
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
        bounds_min: ac2_math::Vec3::ZERO,
        bounds_max: ac2_math::Vec3::ZERO,
    }));
    let info = Ac2AnimInfo {
        effects: vec![Ac2AnimTrigger {
            fx: 0x77,
            time: 0.0,
            stop: false,
            probability: 1.0,
            intensity: 0.5,
            intensity_var: 0.0,
            play: 1,
        }],
        hooks: vec![Ac2WeenieHook {
            hook_type: 1,
            hook_num: 2,
            data: 3,
            triggered: false,
            time: 0.5,
        }],
        ..Ac2AnimInfo::default()
    };
    animation.animators.insert(
        1,
        Arc::new(Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
            period: 1.0,
            bones: Vec::new(),
            info: Some(info),
        }))),
    );
    body.set_animation(animation);
    let mut random = Random::default();
    let mut now = 1.0;
    let mut events = Vec::new();
    for _ in 0..50 {
        now += 0.05;
        body.update_animation(0.05, now, &mut random);
        events.extend(body.take_animation_events());
    }
    let fx: Vec<&Ac2AnimEvent> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Ac2AnimEvent::StartFx { fx: 0x77, .. } | Ac2AnimEvent::StopFx { fx: 0x77 }
            )
        })
        .collect();
    // 2.5 s of a 1 s idle: started, then stopped and restarted at each of two wraps.
    assert_eq!(fx.len(), 5, "{fx:?}");
    assert_eq!(
        fx[0],
        &Ac2AnimEvent::StartFx {
            fx: 0x77,
            intensity: 0.5
        }
    );
    assert_eq!(fx[1], &Ac2AnimEvent::StopFx { fx: 0x77 });
    let hooks = events
        .iter()
        .filter(|e| matches!(e, Ac2AnimEvent::WeenieHook { data: 3, .. }))
        .count();
    assert_eq!(hooks, 1, "a hook fires once per track");
}

#[test]
fn a_constrained_player_stops_twenty_metres_out() {
    let ground = terrain(|_, _| 50.0);
    let walk = |constrain: bool| {
        let mut body = body_at(70.0, 30.0, 50.0);
        for _ in 0..5 {
            body.step_quantum(&ground);
        }
        let start = body.position();
        if constrain {
            body.constrain_to(start);
        }
        body.do_motion([0.0, 1.0, 0.0]);
        for _ in 0..150 {
            body.step_quantum(&ground);
        }
        body.position().distance(&start)
    };
    let free = walk(false);
    let held = walk(true);
    assert!(free > 25.0, "{free}");
    assert!((19.0..20.5).contains(&held), "{held}");
}

/// Two bone poses are the same to the float: offset, scale and the rotation up to its sign.
fn same_transform(a: &ac2_anim::Ac2Transform, b: &ac2_anim::Ac2Transform) -> bool {
    (a.offset - b.offset).length() < 1e-4
        && (a.scale - b.scale).length() < 1e-4
        && a.rotation.dot(b.rotation).abs() > 0.999_99
}

#[test]
fn the_drawn_pose_blends_from_the_previous_refresh_to_the_current() {
    let mut body = remote_body();
    // One bone posed by the walk clip, so the refreshes differ.
    let mut animation = body_animation();
    animation.setup = Some(Arc::new(ac2_anim::Ac2Setup {
        meshes: Arc::from(Vec::new()),
        bones: Arc::from(vec![ac2_anim::Ac2Bone {
            parent: None,
            name: 0,
        }]),
        placements: Arc::from(vec![ac2_anim::Ac2PlacementFrame {
            kind: 0,
            bones: Arc::from(vec![ac2_anim::Ac2Transform::IDENTITY]),
        }]),
        holding_points: Arc::from(Vec::new()),
        connection_points: Arc::from(Vec::new()),
        bounds_min: ac2_math::Vec3::ZERO,
        bounds_max: ac2_math::Vec3::ZERO,
    }));
    body.set_animation(animation);
    let ground = terrain(|_, _| 50.0);
    let mut random = Random::default();
    let step = 1.0 / 30.0;
    let mut now = 10.0;
    // Walking forward: the walk clip ramps the bone in, so some refresh pair differs.
    body.do_motion([0.0, 1.0, 0.0]);
    // The first refresh has no earlier one to blend from: one step before the pair is taken.
    now += step;
    body.update_animation(step as f32, now, &mut random);
    let (mut before, mut after) = (Vec::new(), Vec::new());
    for k in 0..60 {
        now += step;
        if k % 3 == 0 {
            body.step_quantum(&ground);
        }
        let prior = body.bone_transforms();
        body.update_animation(step as f32, now, &mut random);
        let current = body.bone_transforms();
        if prior != current {
            (before, after) = (prior, current);
            break;
        }
    }
    assert!(
        !before.is_empty(),
        "the walk changes the bones between refreshes"
    );
    let same = |a: &[ac2_anim::Ac2Transform], b: &[ac2_anim::Ac2Transform]| {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same_transform(x, y))
    };
    // Fraction 0 at the refresh is the previous refresh; one interval later it is the current.
    assert!(same(&body.blended_bone_transforms(now), &before));
    assert!(same(&body.blended_bone_transforms(now + step), &after));
    // Halfway between them is the blend of each bone.
    let half = body.blended_bone_transforms(now + step / 2.0);
    assert_eq!(half.len(), before.len());
    for ((h, b), a) in half.iter().zip(&before).zip(&after) {
        let expect = ac2_anim::blend_transform(b, a, 0.5);
        assert!(same_transform(h, &expect), "{h:?} != {expect:?}");
    }
    // Past the next refresh the blend holds at the current pose.
    assert!(same(
        &body.blended_bone_transforms(now + 5.0 * step),
        &after
    ));
}
