use ac2_physics::{Frame, Position, Quaternion, Vector};

use super::*;

/// A 1.8 m player of radius 0.4 standing at (70, 70, 50) facing north.
fn player() -> CameraSubject {
    CameraSubject {
        position: Position::new(
            0x7F7F_001C,
            Frame::new(
                Vector::new(70.0, 70.0, 50.0),
                Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ),
        ),
        height: 1.8,
        radius: 0.4,
        scale_z: 1.0,
        local_velocity: Vector::new(0.0, 0.0, 0.0),
        element: 0,
        no_draw: false,
    }
}

fn camera() -> CameraManager {
    let mut camera = CameraManager::new();
    camera.set_player(&player(), 0.75);
    camera
}

#[test]
fn defaults_put_the_camera_behind_the_player_at_the_default_pitch() {
    let mut camera = camera();
    assert_eq!(camera.viewer_offset, Vector::new(0.0, -1.0, 0.25));
    assert_eq!(camera.global_direction(), [-0.0, 1.0, -0.25]);
    assert_eq!(camera.target_offset, Vector::new(0.0, 0.0, 1.35));
    let p = player();
    let at = camera.update_camera(0.1, Some(&p), &p.position);
    // `RotateAngles((-0.25, pi, 0))`'s forward row, times the 4.5 m distance.
    let (s, c) = (0.25_f64.sin(), 0.25_f64.cos());
    assert!(
        (f64::from(at.frame.origin.y) - (70.0 - 4.5 * c)).abs() < 1e-5,
        "{at:?}"
    );
    assert!(
        (f64::from(at.frame.origin.z) - (50.0 + 4.5 * s + 1.8)).abs() < 1e-5,
        "{at:?}"
    );
    assert_eq!(at.frame.origin.x, 70.0);
    // The camera faces the player: its forward axis is the negated offset.
    let forward = at.frame.vector_heading();
    assert!(forward.y > 0.96 && forward.z < -0.24, "{forward:?}");
}

#[test]
fn zoom_steps_are_one_and_a_half_metres_between_the_limits() {
    let mut camera = camera();
    let p = player();
    for _ in 0..3 {
        camera.start_action(action::ZOOM_IN, 1.0, Some(&p));
    }
    // 4.5 - 3 * 1.5 floors at `0.01f + 0.1f`.
    assert_eq!(
        camera.get_camera_distance(),
        (0.01_f64 + f64::from(0.1_f32)) as f32
    );
    for _ in 0..200 {
        camera.start_action(action::ZOOM_OUT, 1.0, Some(&p));
    }
    assert_eq!(camera.get_camera_distance(), 160.0);
}

#[test]
fn the_shoulder_view_leans_the_camera_right_of_the_player() {
    let mut camera = camera();
    let p = player();
    camera.start_action(action::SHOULDER_VIEW, 1.0, Some(&p));
    assert_eq!(camera.get_camera_distance(), 1.3);
    // 70% of the radius, scaled down by 1.3 / 2.9; a jump under sqrt(0.05) m from a settled
    // offset is taken at once.
    let reach = (0.4_f64 * f64::from(0.7_f32)) as f32;
    let bias = ((1.0 - f64::from(1.3_f32) / f64::from(2.9_f32)) * f64::from(reach)) as f32;
    assert_eq!(
        camera.calc_shoulder_offset(Some(&p), 1.3),
        Vector::new(bias, 0.0, 0.0)
    );
    // Asked without a distance (`GetTargetOffset`) it is the whole reach, uneased.
    assert_eq!(
        camera.calc_shoulder_offset(Some(&p), 0.0),
        Vector::new(reach, 0.0, 0.0)
    );
    // Back out at 4.5 m the offset is 0, a small jump: taken at once.
    camera.start_action(action::SHOULDER_VIEW, 1.0, Some(&p));
    assert_eq!(
        camera.calc_shoulder_offset(Some(&p), 4.5),
        Vector::new(0.0, 0.0, 0.0)
    );
    // A wider body jumps further: the offset eases from the old one over a second.
    let wide = CameraSubject { radius: 1.0, ..p };
    camera.start_action(action::SHOULDER_VIEW, 1.0, Some(&wide));
    assert_eq!(camera.calc_shoulder_offset(Some(&wide), 1.3).x, 0.0);
    let target = camera.target_shoulder.x;
    assert!(target > 0.38, "{target}");
    camera.update_camera(0.5, Some(&wide), &wide.position);
    let eased = camera.current_shoulder.x;
    assert!(eased > 0.0 && eased < target, "{eased} {target}");
}

#[test]
fn rotating_turns_the_orbit_at_the_rotate_speed() {
    let mut camera = camera();
    let p = player();
    camera.start_action(action::ROTATE_LEFT, 1.0, Some(&p));
    camera.update_camera(0.5, Some(&p), &p.position);
    camera.stop_action(action::ROTATE_LEFT, Some(&p));
    // Half a second at one radian per second: from 180 degrees to 180 + 28.6.
    let v = camera.viewer_offset;
    let degrees = (5.0 * std::f64::consts::FRAC_PI_2 - f64::from(v.y).atan2(f64::from(v.x)))
        .to_degrees()
        .rem_euclid(360.0);
    assert!(
        (degrees - (180.0 + 0.5_f64.to_degrees())).abs() < 1e-3,
        "{degrees}"
    );
    assert!(((f64::from(v.x).powi(2) + f64::from(v.y).powi(2)).sqrt() - 1.0).abs() < 1e-6);
}

#[test]
fn first_person_sits_at_nine_tenths_of_the_height_leaning_forward() {
    let mut camera = camera();
    let p = player();
    camera.start_action(action::TOGGLE_FIRST_PERSON, 1.0, Some(&p));
    let at = camera.update_camera(0.1, Some(&p), &p.position);
    assert_eq!(at.frame.origin.x, 70.0);
    assert_eq!(at.frame.origin.y, (70.0_f64 + f64::from(0.4_f32)) as f32);
    assert_eq!(at.frame.origin.z, (1.8 * f64::from(0.9_f32) + 50.0) as f32);
    assert!(camera.first_person_ready());
}

#[test]
fn auto_distance_eases_a_close_camera_out_while_running() {
    let mut camera = camera();
    let mut p = player();
    camera.start_action(action::SHOULDER_VIEW, 1.0, Some(&p));
    p.local_velocity = Vector::new(0.0, 4.0, 0.0);
    for _ in 0..40 {
        camera.update_camera(0.1, Some(&p), &p.position);
    }
    // After 2.5 s of running, (d + 1) ^ 1.07 - 1 per update, capped at 4 m.
    let d = camera.targeting.camera_distance;
    assert!(d > 1.3 && d <= 4.0, "{d}");
    p.local_velocity = Vector::new(0.0, 0.0, 0.0);
    for _ in 0..100 {
        camera.update_camera(0.1, Some(&p), &p.position);
    }
    // At rest for over a second it eases back to the chosen distance.
    assert_eq!(camera.targeting.camera_distance, 1.3);
}
