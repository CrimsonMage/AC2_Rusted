use ac2_physics::{Frame, Position, Quaternion, Vector};

use super::*;

const PLAYER: u64 = 0x5000_0001;

/// A 1.8 m player of radius 0.4 standing at (70, 70, 50) facing north.
fn player() -> CameraSubject {
    CameraSubject {
        id: PLAYER,
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

#[test]
fn the_zoom_keys_scale_the_distance_by_powers_of_the_retail_bases() {
    // `CameraZoomIn` raises 0.75 and `CameraZoomOut` 1.5 to the extent times the zoom speed.
    let mut camera = camera();
    let p = player();
    camera.start_action(action::ZOOM_SCALE_IN, 1.0, Some(&p));
    assert!(
        (camera.anim_scale - 0.75).abs() < 1e-6,
        "{}",
        camera.anim_scale
    );
    camera.stop_action(action::ZOOM_SCALE_IN, Some(&p));
    assert_eq!(camera.anim_scale, 1.0);
    camera.start_action(action::ZOOM_SCALE_OUT, 1.0, Some(&p));
    assert!(
        (camera.anim_scale - 1.5).abs() < 1e-6,
        "{}",
        camera.anim_scale
    );
    camera.stop_action(action::ZOOM_SCALE_OUT, Some(&p));
    assert_eq!(camera.anim_scale, 1.0);
    // The zoom speed preference multiplies the extent: 1.5^(0.5 * 2) = 1.5.
    let values = CameraValues {
        zoom_speed: 2.0,
        ..CameraValues::default()
    };
    camera.apply_prefs(&values);
    camera.start_action(action::ZOOM_SCALE_OUT, 0.5, Some(&p));
    assert!(
        (camera.anim_scale - 1.5).abs() < 1e-6,
        "{}",
        camera.anim_scale
    );
}

#[test]
fn the_swing_right_key_swings_left_as_legions_does() {
    let p = player();
    let mut right = camera();
    let mut left = camera();
    right.start_action(action::SWING_OUT_RIGHT, 0.0, Some(&p));
    left.start_action(0x4000_0029, 0.0, Some(&p));
    assert_eq!(right.viewer_offset, left.viewer_offset);
    // The normalised (5.9, -2.5, 4.75) is the target, so the viewer leans to +x.
    assert!(right.viewer_offset.x > 0.5, "{:?}", right.viewer_offset);
}

#[test]
fn the_fake_transition_key_turns_the_viewer_to_the_retail_direction() {
    let p = player();
    let mut camera = camera();
    camera.start_action(action::FAKE_TRANSITION, 0.0, Some(&p));
    assert!(camera.viewer_offset.x < -0.5, "{:?}", camera.viewer_offset);
    // A second press at the target returns to the default pitch behind the player.
    camera.start_action(action::FAKE_TRANSITION, 0.0, Some(&p));
    assert_eq!(camera.viewer_offset, Vector::new(0.0, -1.0, 0.25));
}

#[test]
fn look_at_self_turns_the_camera_down_then_back_to_the_front() {
    let p = player();
    let mut camera = camera();
    camera.start_action(action::LOOK_AT_SELF, 0.0, Some(&p));
    assert_eq!(camera.viewer_offset, Vector::new(0.0, 1.0, 0.0));
    camera.start_action(action::LOOK_AT_SELF, 0.0, Some(&p));
    assert_eq!(camera.viewer_offset, Vector::new(0.0, -1.0, 0.25));
}

#[test]
fn map_view_saves_the_orbit_and_restores_it() {
    let p = player();
    let mut camera = camera();
    let before = (camera.viewer_offset, camera.camera_distance);
    camera.start_action(action::MAP_VIEW, 0.0, Some(&p));
    assert!(camera.mapview);
    assert_eq!(camera.camera_distance, 160.0);
    assert_eq!(camera.viewer_offset, Vector::new(0.0, -1.0, 1.37));
    // Map view ignores the zoom keys and turns: the orbit stays where the map left it.
    camera.start_action(action::FAKE_TRANSITION, 0.0, Some(&p));
    assert_eq!(camera.viewer_offset, Vector::new(0.0, -1.0, 1.37));
    camera.start_action(action::MAP_VIEW, 0.0, Some(&p));
    assert!(!camera.mapview);
    assert_eq!((camera.viewer_offset, camera.camera_distance), before);
}

/// `CameraManager::HandleChange`: the height, transition time and auto distance go into the
/// targeting module; a changed default distance or tilt becomes the default and the camera goes
/// to it at once (`SetDefaults`); an unchanged one leaves a zoomed camera where it is.
#[test]
fn changed_preferences_apply_live() {
    let mut camera = camera();
    let values = CameraValues {
        distance: 6.0,
        tilt: 0.5,
        camera_height: 0.3,
        transition_time: 0.5,
        auto_distance: false,
        zoom_speed: 2.0,
        rotate_speed: 3.0,
        ..CameraValues::default()
    };
    camera.apply_prefs(&values);
    assert_eq!(camera.camera_distance, 6.0);
    assert_eq!(camera.viewer_offset, Vector::new(0.0, -1.0, 0.5));
    assert_eq!(camera.targeting.camera_height, 0.3);
    assert_eq!(camera.targeting.fake_trans_time, 0.5);
    assert!(!camera.targeting.auto_distance);
    assert_eq!((camera.zoom_scale, camera.rotate_scale), (2.0, 3.0));
    // The same values again do not reset the zoom.
    let p = player();
    camera.start_action(action::ZOOM_IN, 0.0, Some(&p));
    assert_eq!(camera.camera_distance, 4.5);
    camera.apply_prefs(&values);
    assert_eq!(camera.camera_distance, 4.5);
    // The transition time is how long the next change blends.
    camera.start_action(action::FAKE_TRANSITION, 0.0, Some(&p));
    assert_eq!(camera.targeting.fake_trans_time, 0.5);
}

/// The camera height lifts the camera: it is added to the viewer's z.
#[test]
fn the_camera_height_preference_lifts_the_camera() {
    let p = player();
    let mut low = camera();
    let before = low.update_camera(0.1, Some(&p), &p.position).frame.origin.z;
    let mut high = camera();
    high.apply_prefs(&CameraValues {
        camera_height: 0.5,
        ..CameraValues::default()
    });
    let after = high
        .update_camera(0.1, Some(&p), &p.position)
        .frame
        .origin
        .z;
    assert!((f64::from(after) - f64::from(before) - 0.5).abs() < 1e-5);
}

/// `SetMapView` switches `SceneTool::m_bDisableDegrades` on and restores its earlier value; the
/// things that take the camera out of the map view (`SetDefaults`, `LookAtGround`,
/// `SetMouseLookState`, `SetFirstPersonState`) do so through it.
#[test]
fn the_map_view_disables_degrades_and_the_other_camera_changes_leave_it() {
    let p = player();
    let mut camera = camera();
    assert!(!camera.disable_degrades());
    camera.start_action(action::MAP_VIEW, 0.0, Some(&p));
    assert!(camera.mapview && camera.disable_degrades());
    camera.start_action(action::MAP_VIEW, 0.0, Some(&p));
    assert!(!camera.mapview && !camera.disable_degrades());
    // A switch that was already on stays on when the map view ends.
    camera.disable_degrades = true;
    camera.set_map_view(true);
    camera.set_map_view(false);
    assert!(camera.disable_degrades());
    camera.disable_degrades = false;

    let leave: [(&str, fn(&mut CameraManager, &CameraSubject)); 5] = [
        ("reset", |c, p| c.start_action(action::RESET, 0.0, Some(p))),
        ("look at ground", |c, p| {
            c.start_action(action::LOOK_AT_GROUND, 0.0, Some(p))
        }),
        ("mouse look", |c, p| {
            c.start_action(action::TOGGLE_MOUSE_LOOK, 0.0, Some(p))
        }),
        ("first person", |c, p| {
            c.start_action(action::TOGGLE_FIRST_PERSON, 0.0, Some(p))
        }),
        ("default distance", |c, _| {
            c.apply_prefs(&CameraValues {
                distance: 7.0,
                ..CameraValues::default()
            })
        }),
    ];
    for (name, change) in leave {
        let mut fresh = self::camera();
        fresh.start_action(action::MAP_VIEW, 0.0, Some(&p));
        assert!(fresh.mapview, "{name}");
        change(&mut fresh, &p);
        assert!(!fresh.mapview, "{name}");
        assert!(!fresh.disable_degrades(), "{name}");
    }
}

/// `GetCameraDistance` adds the camera-distance effects' sum, except in first person and the map
/// view.
#[test]
fn the_camera_distance_effects_add_to_the_distance() {
    let mut camera = camera();
    camera.fx.camera_distance_sum = 2.0;
    assert_eq!(camera.get_camera_distance(), 6.5);
    camera.set_map_view(true);
    assert_eq!(camera.get_camera_distance(), 160.0);
    camera.set_map_view(false);
    camera.set_first_person(true);
    assert_eq!(camera.get_camera_distance(), FIRST_PERSON_DISTANCE);
}
