use crate::missile::{targeting_bits, MissileManager, MissileParameters};
use crate::position::Position;
use crate::vector::Vector;

fn manager() -> MissileManager {
    MissileManager::default()
}

#[test]
fn the_desired_speed_is_the_top_speed_unless_catching_up_or_conserving() {
    let mut m = manager();
    let d = Vector::new(10.0, 0.0, 0.0);
    assert_eq!(m.get_desired_velocity(d, Vector::ZERO, false, 0.0), 50.0);
    assert_eq!(m.get_desired_velocity(d, Vector::ZERO, true, 0.0), 50.0);
    m.target_info.bitfield = targeting_bits::INITIAL_SPEED;
    m.target_info.initial_velocity = 12.0;
    assert_eq!(m.get_desired_velocity(d, Vector::ZERO, true, 0.0), 12.0);
    // Conserving: the target's speed away plus the catch-up margin (default 2), at least the
    // minimum speed, at most the top speed.
    m.info.conserve_fuel = true;
    let away = Vector::new(5.0, 0.0, 0.0);
    assert_eq!(m.get_desired_velocity(d, away, false, 0.0), 7.0);
    m.info.max_velocity = 6.0;
    assert_eq!(m.get_desired_velocity(d, away, false, 0.0), 6.0);
    m.info.max_velocity = 50.0;
    let toward = Vector::new(-10.0, 0.0, 0.0);
    assert_eq!(
        m.get_desired_velocity(d, toward, false, 0.0),
        1.0,
        "the minimum speed"
    );
}

#[test]
fn the_remaining_lifespan_can_raise_the_conserved_speed() {
    let mut m = manager();
    m.info.conserve_fuel = true;
    m.info.consider_remaining_time = true;
    m.missile_expire_time = 11.0;
    let d = Vector::new(100.0, 0.0, 0.0);
    // 11 - 1 (now) - 1 = 9 s left: 100 / 9 + 0 beats the margin of 2.
    let speed = m.get_desired_velocity(d, Vector::ZERO, false, 1.0);
    assert!((speed - 100.0 / 9.0).abs() < 1e-5, "{speed}");
}

#[test]
fn a_still_target_is_met_head_on_at_the_top_speed() {
    let m = manager();
    let mut out = Vector::ZERO;
    // diff is the missile's position minus the target's: 100 m behind.
    m.collide_velocity(
        Vector::new(-100.0, 0.0, 0.0),
        Vector::ZERO,
        false,
        false,
        0.0,
        &mut out,
    );
    assert_eq!((out.x, out.y, out.z), (50.0, 0.0, 0.0));
}

#[test]
fn a_moving_target_is_led() {
    let m = manager();
    let mut out = Vector::ZERO;
    // The target is 100 m ahead (+y) walking +x at 10 m/s; at 50 m/s the intercept takes
    // sqrt(100^2 / (50^2 - 10^2)) s and the missile has to cancel the target's sideways motion.
    m.collide_velocity(
        Vector::new(0.0, -100.0, 0.0),
        Vector::new(10.0, 0.0, 0.0),
        false,
        false,
        0.0,
        &mut out,
    );
    assert!((out.x - 10.0).abs() < 1e-4, "{out:?}");
    assert!(
        (out.y - (50.0_f32 * 50.0 - 100.0).sqrt()).abs() < 1e-3,
        "{out:?}"
    );
}

#[test]
fn no_intercept_leaves_the_velocity_as_it_was() {
    let m = manager();
    let mut out = Vector::new(1.0, 2.0, 3.0);
    // A target running away faster than the top speed: no non-negative root above epsilon.
    m.collide_velocity(
        Vector::new(0.0, -100.0, 0.0),
        Vector::new(0.0, 200.0, 0.0),
        false,
        true,
        0.0,
        &mut out,
    );
    assert_eq!(out, Vector::new(1.0, 2.0, 3.0));
}

#[test]
fn the_turn_direction_follows_the_method() {
    let mut m = manager();
    let mut flipped = Vector::new(1.0, 2.0, 3.0);
    m.info.tight_turn_method = 0;
    m.perp_adjust(&mut flipped, true, |_| false);
    assert_eq!(flipped, Vector::new(-1.0, -2.0, -3.0));
    m.info.tight_turn_method = 1;
    let mut kept = Vector::new(1.0, 2.0, 3.0);
    m.perp_adjust(&mut kept, true, |_| true);
    assert_eq!(kept, Vector::new(1.0, 2.0, 3.0));
    // Method 2: the first turn flips on the coin.
    m.info.tight_turn_method = 2;
    let mut coin = Vector::new(1.0, 0.0, 0.0);
    m.perp_adjust(&mut coin, true, |chance| {
        assert_eq!(chance, 0.5);
        true
    });
    assert_eq!(coin.x, -1.0);
    // Later turns flip against the current velocity.
    m.current_velocity = Vector::new(1.0, 0.0, 0.0);
    let mut against = Vector::new(-1.0, 0.0, 0.0);
    m.perp_adjust(&mut against, false, |_| false);
    assert_eq!(against.x, 1.0);
    let mut along = Vector::new(1.0, 0.0, 0.0);
    m.perp_adjust(&mut along, false, |_| false);
    assert_eq!(along.x, 1.0);
    // No minimum speed: straight on.
    m.info.min_velocity = 0.0;
    let mut zeroed = Vector::new(1.0, 2.0, 3.0);
    m.perp_adjust(&mut zeroed, true, |_| true);
    assert_eq!((zeroed.x, zeroed.y, zeroed.z), (0.0, 0.0, 0.0));
}

#[test]
fn circling_at_the_maintain_distance_runs_along_the_ring() {
    let m = manager();
    let mut out = Vector::ZERO;
    // 5 m out (the default maintain distance): the whole velocity is the perpendicular.
    m.maint_dist_velocity(
        Vector::new(3.0, 4.0, 0.0),
        Vector::ZERO,
        true,
        true,
        |_| false,
        &mut out,
    );
    assert!(
        (out.x + 4.0).abs() < 1e-5 && (out.y - 3.0).abs() < 1e-5,
        "{out:?}"
    );
    // The coin turns it the other way.
    m.maint_dist_velocity(
        Vector::new(3.0, 4.0, 0.0),
        Vector::ZERO,
        true,
        true,
        |_| true,
        &mut out,
    );
    assert!(
        (out.x - 4.0).abs() < 1e-5 && (out.y + 3.0).abs() < 1e-5,
        "{out:?}"
    );
    // Far out the ring is closed in on (distance 10 -> 5 m to cover radially).
    m.maint_dist_velocity(
        Vector::new(0.0, 10.0, 0.0),
        Vector::ZERO,
        true,
        true,
        |_| false,
        &mut out,
    );
    assert!(out.y < 0.0, "{out:?}");
}

#[test]
fn intercepting_uses_the_offset_from_the_target() {
    let mut m = manager();
    let cell = 0x0101_0001;
    let mut target = Position {
        cell_id: cell,
        ..Position::default()
    };
    target.frame.origin = Vector::new(100.0, 0.0, 0.0);
    m.target_pos.set_target_info(target, Vector::ZERO);
    let missile = Position {
        cell_id: cell,
        ..Position::default()
    };
    let mut out = Vector::ZERO;
    m.intercept_velocity(&missile, true, true, 0.0, |_| false, &mut out);
    assert_eq!((out.x, out.y, out.z), (50.0, 0.0, 0.0));
    // With the maintain-distance mode it closes in on the ring (speed-limited), a little sideways.
    m.info = MissileParameters {
        use_maint_dist: true,
        ..m.info
    };
    m.intercept_velocity(&missile, true, true, 0.0, |_| false, &mut out);
    assert!(out.x > 40.0 && out.y < -1.0, "{out:?}");
}
