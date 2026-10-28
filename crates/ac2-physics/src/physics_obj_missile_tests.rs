//! The body as a missile's `ICPhysicsObj`, on synthetic terrain.

use super::*;
use crate::body::PhysicsBody;
use crate::frame::Frame;
use crate::missile::MISSILE_FLIGHT_FX;
use crate::test_support::{body_at, terrain};

fn target_at(body: &PhysicsBody, dy: f32) -> TargetPosition {
    let mut position = body.position();
    position.frame.origin.y += dy;
    TargetPosition {
        target_id: 0,
        offset_pos: position,
        target_velocity: Vector::ZERO,
        initialized: true,
    }
}

fn step(body: &mut PhysicsBody, ground: &Terrain, quanta: usize) {
    let mut coin = |_: f32| false;
    let mut env = MissileEnv {
        objects: &NoObjects,
        coin: &mut coin,
    };
    for _ in 0..quanta {
        body.cur_time_add(0.1);
        body.step_quantum_in(ground, &mut env);
    }
}

#[test]
fn initialize_missile_makes_the_manager_and_starts_the_flight() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 80.0);
    assert!(!body.has_missile_manager());
    let target = target_at(&body, 60.0);
    body.recv_initialize_missile(&ground, [0.0, 20.0, 0.0], &target);
    assert!(body.has_missile_manager());
    assert_eq!(
        body.missile_current_velocity(),
        Some(Vector::new(0.0, 20.0, 0.0))
    );
    let events = body.take_missile_events();
    assert!(events.contains(&MissileEvent::Streak { on: true }));
    assert!(body.take_missile_events().is_empty());
    let _ = MISSILE_FLIGHT_FX;
}

#[test]
fn a_missile_flies_toward_its_target_each_quantum() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 80.0);
    let start = body.position();
    let target = target_at(&body, 60.0);
    body.recv_initialize_missile(&ground, [0.0, 20.0, 0.0], &target);
    step(&mut body, &ground, 10);
    let moved = body.position().offset_to(&start);
    assert!(moved.y.abs() > 10.0, "{moved:?}");
    assert!(moved.x.abs() < 0.5 && moved.z.abs() < 0.5, "{moved:?}");
    assert!(body.velocity().y > 5.0);
}

#[test]
fn a_missile_without_a_manager_is_refused_with_error_five() {
    let mut obj = body_at(70.0, 70.0, 80.0);
    let mut events = Vec::new();
    obj.with_object(|o| {
        o.missile_request(true, true);
        events = std::mem::take(&mut o.missile_events);
    });
    assert_eq!(events, vec![MissileEvent::Error(5)]);
}

#[test]
fn set_velocity_culled_ignores_a_change_under_epsilon() {
    let mut body = body_at(70.0, 70.0, 80.0);
    body.with_object(|o| {
        o.velocity = Vector::new(1.0, 0.0, 0.0);
        o.set_velocity_culled(Vector::new(1.0 + 1e-5, 0.0, 0.0));
        assert_eq!(o.velocity.x, 1.0);
        o.set_velocity_culled(Vector::new(2.0, 0.0, 0.0));
        assert_eq!(o.velocity.x, 2.0);
    });
}

#[test]
fn offset_pos_with_no_offset_is_the_object_position() {
    let ground = terrain(|_, _| 50.0);
    let body = body_at(70.0, 70.0, 80.0);
    let mut view = body.object_view();
    view.height = 0.0;
    let (ok, pos) = object_offset_pos(&view, &ground, 0.75, false, Vector::ZERO);
    assert!(ok);
    assert_eq!(pos, view.position);
}

#[test]
fn offset_pos_lifts_by_the_height_fraction() {
    let ground = terrain(|_, _| 50.0);
    let body = body_at(70.0, 70.0, 80.0);
    let mut view = body.object_view();
    view.height = 2.0;
    view.position.frame = Frame::new(
        view.position.frame.origin,
        crate::vector::Quaternion::IDENTITY,
    );
    let (_, pos) = object_offset_pos(&view, &ground, 0.5, false, Vector::ZERO);
    assert!(
        (pos.frame.origin.z - (view.position.frame.origin.z + 1.0)).abs() < 1e-3,
        "{pos:?}"
    );
}

#[test]
fn inquire_direction_reports_zero_heading_and_pitch_as_retail_stores_them() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 80.0);
    let target = target_at(&body, 60.0);
    body.recv_initialize_missile(&ground, [0.0, 20.0, 0.0], &target);
    body.take_missile_events();
    let mut coin = |_: f32| false;
    let mut env = MissileEnv {
        objects: &NoObjects,
        coin: &mut coin,
    };
    let (heading, pitch) = body
        .missile_inquire_direction(&ground, &mut env)
        .expect("manager");
    assert_eq!((heading.degrees(), pitch), (0.0, 0.0));
}

#[test]
fn a_detonation_through_the_body_queues_the_delete() {
    use crate::collision_report::CollisionReport;
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 80.0);
    let target = target_at(&body, 60.0);
    body.recv_initialize_missile(&ground, [0.0, 20.0, 0.0], &target);
    body.take_missile_events();
    struct None_;
    impl crate::ImpactLookup for None_ {
        fn target(&self, _: u64) -> Option<crate::ImpactTarget> {
            None
        }
    }
    let report = CollisionReport {
        impact: Some(crate::collision_impact::ImpactReport {
            normal: Vector::new(0.0, 0.0, 1.0),
            speed: 5.0,
        }),
        ..CollisionReport::default()
    };
    body.handle_missile_collision(&ground, &report, &None_);
    let events = body.take_missile_events();
    assert!(events.contains(&MissileEvent::Streak { on: false }));
    assert!(events.contains(&MissileEvent::DeleteSelf {
        fade: 0.5,
        delay: 0.0
    }));
}
