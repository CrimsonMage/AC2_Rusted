use super::*;

#[derive(Default)]
struct Host {
    gravity: Vector,
    wind: Vector,
    velocity: Option<Vector>,
    expired: u32,
    out_of_fuel: u32,
    calls: Vec<String>,
}

impl MissileHost for Host {
    fn current_gravity(&self) -> Vector {
        self.gravity
    }
    fn current_wind(&self) -> Vector {
        self.wind
    }
    fn is_in_world(&self) -> bool {
        true
    }
    fn set_missile_velocity(&mut self, velocity: Vector) {
        self.velocity = Some(velocity);
    }
    fn report_out_of_fuel(&mut self) {
        self.out_of_fuel += 1;
    }
    fn report_expired(&mut self) {
        self.expired += 1;
    }
    fn stop_fx(&mut self, fx: u32) {
        self.calls.push(format!("stop_fx {fx:x}"));
    }
    fn do_fx(&mut self, fx: u32, scalar: f32) {
        self.calls.push(format!("do_fx {fx:x} {scalar}"));
    }
    fn start_streak(&mut self) {
        self.calls.push("start_streak".into());
    }
    fn stop_streak(&mut self) {
        self.calls.push("stop_streak".into());
    }
}

#[test]
fn the_defaults_are_the_constructors() {
    let p = MissileParameters::default();
    assert_eq!((p.max_velocity, p.min_velocity), (50.0, 1.0));
    assert!(p.report_if_no_fuel && p.correct_for_target_velocity);
    assert_eq!(p.tight_turn_method, 2);
    assert_eq!(p.height_above_ground, 1.5);
}

#[test]
fn script_words_set_floats_bools_and_ints() {
    let p = MissileParameters::from_words(
        0x5,
        [
            ("max_velocity", 20.0_f32.to_bits()),
            ("uses_fuel", 1),
            ("ground_follow_type", 2),
            ("unknown", 9),
        ],
    );
    assert_eq!(p.bitfield, 5);
    assert_eq!(p.max_velocity, 20.0);
    assert!(p.uses_fuel);
    assert_eq!(p.ground_follow_type, 2);
    assert_eq!(p.min_velocity, 1.0);
}

#[test]
fn min_time_raises_to_the_closing_time() {
    let mut t = 0.0002;
    assert!(get_min_time(100.0, 0.0, &mut t, 50.0));
    assert_eq!(t, 2.0);
    // A target running away as fast as the missile can never be caught.
    assert!(!get_min_time(100.0, 50.0, &mut t, 50.0));
    // Behind: the same on the negative axis.
    let mut t = 0.0002;
    assert!(get_min_time(-100.0, 0.0, &mut t, 50.0));
    assert_eq!(t, 2.0);
    assert!(!get_min_time(-100.0, -60.0, &mut t, 50.0));
    // On top of it already.
    assert!(get_min_time(0.0, 99.0, &mut t, 50.0));
}

#[test]
fn max_time_is_the_later_root() {
    // dist = (50 - 0) t + a t^2/2 with a = -10 (the discriminant 50^2 + 2 dist a):
    let mut t = 0.0;
    assert!(get_max_time(100.0, 0.0, -10.0, &mut t, 50.0));
    // roots of 100 = 50 t - 5 t^2: t = 2.764 and 7.236
    assert!((t - 7.236_068).abs() < 1e-4, "{t}");
    // No real root.
    assert!(!get_max_time(1000.0, 0.0, -10.0, &mut t, 50.0));
}

#[test]
fn gravitational_velocity_meets_the_target_in_the_time() {
    let diff = Vector::new(0.0, 10.0, 0.0);
    let gravity = Vector::new(0.0, 0.0, -10.0);
    let v = get_gravitational_velocity(diff, Vector::ZERO, gravity, 2.0).unwrap();
    // y: 10 / 2; z: -(2 * -10 * 0.5) = 10 to climb against gravity.
    assert_eq!((v.x, v.y, v.z), (0.0, 5.0, 10.0));
    assert!(get_gravitational_velocity(diff, Vector::ZERO, gravity, 0.0).is_none());
}

#[test]
fn speed_is_held_between_the_limits() {
    let m = MissileManager {
        info: MissileParameters {
            max_velocity: 10.0,
            min_velocity: 2.0,
            ..MissileParameters::default()
        },
        ..MissileManager::default()
    };
    let mut v = Vector::new(30.0, 40.0, 0.0);
    m.adjust_speed(&mut v);
    assert!(
        (ld(v.x) - 6.0).abs() < 1e-5 && (ld(v.y) - 8.0).abs() < 1e-5,
        "{v:?}"
    );
    let mut v = Vector::new(0.6, 0.8, 0.0);
    m.adjust_speed(&mut v);
    assert!(
        (ld(v.x) - 1.2).abs() < 1e-5 && (ld(v.y) - 1.6).abs() < 1e-5,
        "{v:?}"
    );
    // Too small to aim: up the y axis at the minimum.
    let mut v = Vector::new(0.0, 0.0, 0.0);
    m.adjust_speed(&mut v);
    assert_eq!((v.x, v.y, v.z), (0.0, 2.0, 0.0));
    // In range: untouched.
    let mut v = Vector::new(3.0, 4.0, 0.0);
    m.adjust_speed(&mut v);
    assert_eq!((v.x, v.y, v.z), (3.0, 4.0, 0.0));
}

#[test]
fn ground_following_sets_or_raises_the_climb() {
    let mut m = MissileManager::default();
    m.info.height_above_ground = 3.0;
    m.current_height = 1.0;
    m.info.ground_follow_type = 2;
    let mut v = Vector::new(0.0, 5.0, 7.0);
    m.adjust_height(&mut v);
    assert_eq!(v.z, 2.0);
    m.info.ground_follow_type = 1;
    let mut v = Vector::new(0.0, 5.0, 1.0);
    m.adjust_height(&mut v);
    assert_eq!(v.z, 2.0);
    let mut v = Vector::new(0.0, 5.0, 9.0);
    m.adjust_height(&mut v);
    assert_eq!(v.z, 9.0);
    m.info.ground_follow_type = 0;
    let mut v = Vector::new(0.0, 5.0, 1.0);
    m.adjust_height(&mut v);
    assert_eq!(v.z, 1.0);
}

#[test]
fn fuel_burns_with_acceleration_and_stops_at_empty() {
    let mut m = MissileManager::default();
    m.info.uses_fuel = true;
    m.info.total_fuel = 10.0;
    m.arm(0.0);
    assert_eq!(m.current_fuel, 10.0);
    m.current_acceleration = Vector::new(3.0, 4.0, 0.0);
    m.update_fuel_use(0.5);
    assert_eq!(m.current_fuel, 7.5);
    assert!(!m.out_of_fuel());
    m.update_fuel_use(10.0);
    assert_eq!(m.current_fuel, 0.0);
    assert!(m.out_of_fuel());
    let mut host = Host::default();
    m.report_out_of_fuel(&mut host);
    assert!(m.fuel_reported);
    assert_eq!(host.out_of_fuel, 1);
}

#[test]
fn a_lifespan_arms_the_expiry_and_is_reported_once() {
    let mut m = MissileManager::default();
    m.info.missile_has_lifespan = true;
    m.info.missile_lifespan = 5.0;
    m.arm(100.0);
    assert_eq!(m.missile_expire_time, 105.0);
    m.is_moving = true;
    let mut host = Host::default();
    let mut offset = Frame::IDENTITY;
    m.update_missile_physics(&mut host, Vector::ZERO, 0.1, &mut offset, 104.0);
    assert_eq!(host.expired, 0);
    m.update_missile_physics(&mut host, Vector::ZERO, 0.1, &mut offset, 106.0);
    m.update_missile_physics(&mut host, Vector::ZERO, 0.1, &mut offset, 107.0);
    assert_eq!(host.expired, 1);
}

#[test]
fn flight_moves_the_offset_and_updates_the_velocity() {
    let mut m = MissileManager {
        is_moving: true,
        current_acceleration: Vector::new(0.0, 2.0, 0.0),
        ..MissileManager::default()
    };
    let mut host = Host::default();
    let mut offset = Frame::IDENTITY;
    m.update_missile_physics(
        &mut host,
        Vector::new(0.0, 10.0, 0.0),
        0.5,
        &mut offset,
        0.0,
    );
    // y: 10 * 0.5 + 2 * 0.5 * 0.25 = 5.25; the velocity gains 2 * 0.5.
    assert_eq!(offset.origin.y, 5.25);
    assert_eq!(host.velocity, Some(Vector::new(0.0, 11.0, 0.0)));
    assert_eq!(m.current_velocity(), Vector::new(0.0, 11.0, 0.0));
}

#[test]
fn gravity_drag_and_wind_shape_the_flight() {
    let mut m = MissileManager {
        is_moving: true,
        ..MissileManager::default()
    };
    m.info.affected_by_gravity = true;
    m.info.affected_by_wind = true;
    m.info.drag = 0.5;
    let mut host = Host {
        gravity: Vector::new(0.0, 0.0, -10.0),
        wind: Vector::new(1.0, 0.0, 0.0),
        ..Host::default()
    };
    let mut offset = Frame::IDENTITY;
    m.update_missile_physics(&mut host, Vector::new(0.0, 4.0, 0.0), 1.0, &mut offset, 0.0);
    // accel = gravity - drag * velocity = (0, -2, -10); the wind joins the velocity: (1, 4, 0).
    let v = host.velocity.unwrap();
    assert_eq!((v.x, v.y, v.z), (1.0, 2.0, -10.0));
    assert_eq!(offset.origin.x, 1.0);
    assert_eq!(offset.origin.y, 4.0 - 1.0);
    assert_eq!(offset.origin.z, -5.0);
}

#[test]
fn a_missile_that_is_not_moving_leaves_no_offset() {
    let mut m = MissileManager::default();
    let mut host = Host::default();
    let mut offset = Frame::IDENTITY;
    offset.origin = Vector::new(1.0, 2.0, 3.0);
    m.update_missile_physics(&mut host, Vector::new(0.0, 1.0, 0.0), 0.5, &mut offset, 0.0);
    assert_eq!(offset, Frame::IDENTITY);
    assert!(host.velocity.is_none());
}

#[test]
fn moving_starts_and_stops_the_flight_effects() {
    let mut m = MissileManager::default();
    let mut host = Host::default();
    m.set_moving(&mut host, true);
    m.set_moving(&mut host, false);
    assert_eq!(
        host.calls,
        [
            "start_streak",
            "do_fx 40000001 1",
            "stop_fx 40000001",
            "stop_streak"
        ]
    );
}

#[test]
fn chasing_heads_along_both_directions_at_the_length() {
    // A target going +x and a missile going +y: the chase is the diagonal.
    let v = chase_velocity(Vector::new(5.0, 0.0, 0.0), Vector::new(0.0, 8.0, 0.0), 10.0);
    assert!(
        (v.x - 7.071_068).abs() < 1e-4 && (v.y - 7.071_068).abs() < 1e-4,
        "{v:?}"
    );
    // A still target: the missile's own velocity.
    assert_eq!(
        chase_velocity(Vector::ZERO, Vector::new(1.0, 2.0, 3.0), 9.0),
        Vector::new(1.0, 2.0, 3.0)
    );
    // A still missile: along the target.
    let v = chase_velocity(Vector::new(0.0, 0.0, 4.0), Vector::ZERO, 2.0);
    assert!((v.z - 1.0).abs() < 1e-6 && v.x == 0.0, "{v:?}");
}

#[test]
fn a_target_position_resets_to_a_fixed_point() {
    let mut t = TargetPosition {
        target_id: 5,
        ..TargetPosition::default()
    };
    t.reset(Position::default());
    assert_eq!(t.target_id, 0);
    assert!(t.initialized);
    t.set_target_info(Position::default(), Vector::new(1.0, 0.0, 0.0));
    assert_eq!(t.target_velocity.x, 1.0);
}

#[test]
fn the_time_window_spans_the_axes_that_do_not_accelerate() {
    // No acceleration: only the earliest time is solved, and the latest is twice it.
    let (possible, t_min, t_max) = calc_min_max_time(
        Vector::new(0.0, 100.0, 0.0),
        Vector::ZERO,
        Vector::ZERO,
        50.0,
    );
    assert!(possible);
    assert_eq!((t_min, t_max), (2.0, 4.0));
    // An accelerating z axis (gravity) sets the latest time from its root.
    let (possible, t_min, t_max) = calc_min_max_time(
        Vector::new(0.0, 100.0, 0.0),
        Vector::ZERO,
        Vector::new(0.0, 0.0, -10.0),
        50.0,
    );
    assert!(possible);
    assert_eq!(t_min, 2.0);
    // z: dist 0, accel -10: roots of 0 = 50 t - 5 t^2 -> 10
    assert!((t_max - 10.0).abs() < 1e-4, "{t_max}");
    // A target out-running the missile on a free axis is impossible.
    let (possible, ..) = calc_min_max_time(
        Vector::new(0.0, 100.0, 0.0),
        Vector::new(0.0, 60.0, 0.0),
        Vector::ZERO,
        50.0,
    );
    assert!(!possible);
}
