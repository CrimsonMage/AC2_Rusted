use super::*;
use crate::missile::get_gravitational_velocity;
use crate::missile_gravity::greater_pitch;

const CELL: u32 = 0x0101_0001;

fn pos(x: f32, y: f32, z: f32) -> Position {
    let mut frame = Frame::IDENTITY;
    frame.origin = Vector::new(x, y, z);
    Position::new(CELL, frame)
}

#[derive(Default)]
struct Host {
    gravity: Vector,
    position: Position,
    target: Option<ObjectTarget>,
    own_target: u64,
    placed: Option<Position>,
    velocity: Option<Vector>,
    calls: Vec<String>,
}

impl MissileHost for Host {
    fn current_gravity(&self) -> Vector {
        self.gravity
    }
    fn current_wind(&self) -> Vector {
        Vector::ZERO
    }
    fn is_in_world(&self) -> bool {
        self.placed.is_some()
    }
    fn set_missile_velocity(&mut self, velocity: Vector) {
        self.velocity = Some(velocity);
    }
    fn report_out_of_fuel(&mut self) {
        self.calls.push("out_of_fuel".into());
    }
    fn report_expired(&mut self) {}
    fn stop_fx(&mut self, _fx: u32) {}
    fn do_fx(&mut self, fx: u32, _scalar: f32) {
        self.calls.push(format!("do_fx {fx:x}"));
    }
    fn start_streak(&mut self) {
        self.calls.push("start_streak".into());
    }
    fn stop_streak(&mut self) {}
}

impl MissileFlightHost for Host {
    fn position(&self) -> Position {
        self.position
    }
    fn height_above_ground(&self) -> Option<f32> {
        None
    }
    fn object_target(&mut self, _: u64, _: f32, _: bool, _: Vector) -> Option<ObjectTarget> {
        self.target.clone()
    }
    fn object_position(&self, _: u64) -> Option<Position> {
        None
    }
    fn object_heading(&self, _: u64) -> Option<Heading> {
        None
    }
    fn target_id(&self) -> u64 {
        self.own_target
    }
    fn set_target(&mut self, id: u64) {
        self.own_target = id;
    }
    fn parent_start_position(&self) -> Option<Position> {
        None
    }
    fn place_missile(&mut self, pos: &Position) -> bool {
        self.placed = Some(*pos);
        self.position = *pos;
        true
    }
    fn slot_1c4(&mut self, _on: bool) {}
}

#[test]
fn copy_into_copies_only_the_named_groups() {
    let src = MissileParameters::from_words(1, [("max_velocity", 9.0_f32.to_bits())]);
    let mut dst = MissileParameters {
        min_velocity: 7.0,
        ..MissileParameters::default()
    };
    assert!(src.copy_into(&mut dst));
    assert_eq!(dst.max_velocity, 9.0);
    assert_eq!(dst.bitfield & 1, 1);
    // The group's other fields come along; the unnamed groups do not.
    assert_eq!(dst.min_velocity, src.min_velocity);
    assert_eq!(
        dst.max_forward_accel,
        MissileParameters::default().max_forward_accel
    );
}

#[test]
fn targeting_copy_into_swaps_the_target_kind_bits() {
    let mut dst = MissileTargetingParameters {
        bitfield: 2,
        ..Default::default()
    };
    let src = MissileTargetingParameters {
        bitfield: 1,
        target_id: 77,
        ..Default::default()
    };
    assert!(src.copy_into(&mut dst));
    assert_eq!((dst.bitfield, dst.target_id), (1, 77));
}

#[test]
fn acceleration_is_clamped_in_the_missiles_axes() {
    let mut m = MissileManager::default();
    m.info.min_reaction_time = 1.0;
    m.info.max_forward_accel = 2.0;
    m.current_velocity = Vector::new(0.0, 10.0, 0.0);
    m.desired_velocity = Vector::new(0.0, 100.0, 0.0);
    m.calc_acceleration();
    // Forward is +y; the wanted 90 is cut to the forward limit.
    assert!((m.current_acceleration.y - 2.0).abs() < 1e-5);
    assert!(m.current_acceleration.x.abs() < 1e-5);
}

#[test]
fn an_empty_tank_leaves_the_acceleration() {
    let mut m = MissileManager::default();
    m.info.uses_fuel = true;
    m.current_fuel = 0.0;
    m.current_acceleration = Vector::new(1.0, 2.0, 3.0);
    m.desired_velocity = Vector::new(5.0, 5.0, 5.0);
    m.calc_acceleration();
    assert_eq!(m.current_acceleration, Vector::new(1.0, 2.0, 3.0));
}

#[test]
fn update_places_fires_and_steers_toward_a_fixed_target() {
    let mut host = Host {
        position: pos(0.0, 0.0, 0.0),
        ..Host::default()
    };
    let mut m = MissileManager::default();
    m.target_info.base_pos = pos(0.0, 0.0, 0.0);
    let target_pos = pos(0.0, 30.0, 0.0);
    m.target_pos.reset(target_pos);
    assert_eq!(m.setup_target(&mut host), 0);
    assert_eq!(m.target_status, TargetStatus::Ok);
    m.place_and_fire();
    assert_eq!(m.update(&mut host, 10.0, |_| false), 0);
    assert!(host.placed.is_some());
    assert!(m.is_moving && !m.ready_to_fire);
    let v = host.velocity.expect("launched");
    assert!(v.y > 0.0, "{v:?}");
    // A later update past the reaction time sets an acceleration toward the target.
    assert_eq!(m.update(&mut host, 11.0, |_| false), 0);
    assert!(m.desired_velocity.y > 0.0);
}

#[test]
fn an_object_target_is_looked_up_and_targeted() {
    let mut host = Host {
        target: Some(ObjectTarget {
            ok: true,
            position: pos(5.0, 5.0, 0.0),
            velocity: Vector::new(1.0, 0.0, 0.0),
        }),
        ..Host::default()
    };
    let mut m = MissileManager::default();
    m.target_info.bitfield = targeting_bits::TARGET_OBJECT;
    m.target_info.target_id = 42;
    assert_eq!(m.setup_target(&mut host), 0);
    assert_eq!(m.target_pos.offset_pos, pos(5.0, 5.0, 0.0));
    assert_eq!(m.target_pos.target_velocity, Vector::new(1.0, 0.0, 0.0));
    assert_eq!(host.own_target, 42);
}

#[test]
fn a_fixed_point_with_no_cell_is_error_one() {
    let mut host = Host::default();
    let mut m = MissileManager::default();
    m.target_pos.reset(Position::default());
    assert_eq!(m.target_update(&mut host), 1);
}

#[test]
fn greater_pitch_needs_a_vanishing_velocity() {
    assert!(!greater_pitch(Vector::new(0.0, 0.0, 5.0), 0.5));
    assert!(greater_pitch(Vector::new(0.0, 0.0, 0.00001), -1.0));
}

#[test]
fn the_ballistic_solver_lands_on_a_still_target() {
    let m = MissileManager::default();
    let gravity = Vector::new(0.0, 0.0, -9.8);
    let diff = Vector::new(0.0, 20.0, 0.0);
    let v = m.calc_gravitational_velocity(gravity, 30.0, diff, Vector::ZERO, false);
    // Flying v under gravity for some time t reaches the target.
    let speed = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    assert!((speed - 30.0).abs() < 1.0, "speed {speed} v {v:?}");
    let t = diff.y / v.y;
    let z = v.z * t - 0.5 * 9.8 * t * t;
    assert!(z.abs() < 0.5, "z {z}");
    // The solver's own time-of-flight velocity for that time matches.
    let check = get_gravitational_velocity(diff, Vector::ZERO, gravity, t).unwrap();
    assert!((check.z - v.z).abs() < 1.0);
}

#[test]
fn the_closest_search_picks_an_end_when_the_window_is_small() {
    let m = MissileManager::default();
    let out = m.calc_gravitational_closest(
        1.0,
        1.0001,
        Vector::new(0.0, 10.0, 0.0),
        Vector::ZERO,
        Vector::new(0.0, 0.0, -9.8),
        Vector::new(0.0, 5.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 9.0, 0.0),
    );
    // |desired|^2 = 25 > |v_min|^2 = 1, so the low end.
    assert_eq!(out, Vector::new(0.0, 1.0, 0.0));
}
