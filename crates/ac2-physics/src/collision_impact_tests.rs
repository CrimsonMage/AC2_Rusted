//! The impact minimums and the impact report of a body (`CPhysicsObj::InitObjectEnd`,
//! `CPhysicsObj::SetJumpScale`, `CCollisionManager::SetCollisionState`, and the
//! `CCollisionManager::ReportCollision` take).

use crate::body::{MovementInput, PHYSICS_QUANTUM};
use crate::test_support::{body_at, terrain};

/// `CPhysicsObj::InitObjectEnd` with the standard walking values (hover 0, forward speed 4, jump
/// impulse 5): XY is `0 * 4 + 0.1`, Z is `5 * 1 + 0.1`.
#[test]
fn a_new_body_starts_with_the_init_object_end_minimums() {
    let body = body_at(70.0, 70.0, 50.0);
    let (xy, z) = body.impact_minimums();
    assert!((xy - 0.1).abs() < 1e-6, "{xy}");
    assert!((z - 5.1).abs() < 1e-6, "{z}");
}

/// `CPhysicsObj::SetVelocityScale` and `SetJumpScale` rebuild the minimums from the scales.
#[test]
fn the_scales_rebuild_the_minimums() {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_velocity_scale(0.5);
    body.set_jump_scale(2.0);
    let (xy, z) = body.impact_minimums();
    assert!((xy - 2.1).abs() < 1e-6, "{xy}");
    assert!((z - 10.1).abs() < 1e-6, "{z}");
}

/// `CCollisionManager::SetCollisionState` copies only the non-zero minimums of the script's
/// `CollisionParameters`; a zero keeps the body's value.
#[test]
fn a_collision_state_replaces_only_the_non_zero_minimums() {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.set_collision_minimums(0.0, 7.0);
    let (xy, z) = body.impact_minimums();
    assert!((xy - 0.1).abs() < 1e-6, "{xy}");
    assert_eq!(z, 7.0);
    body.set_collision_minimums(3.0, 0.0);
    let (xy, z) = body.impact_minimums();
    assert_eq!(xy, 3.0);
    assert_eq!(z, 7.0);
}

/// A body that lands from a fall above its Z minimum records one impact (its normal and speed),
/// and the collision report takes it once.
#[test]
fn a_landing_impact_is_taken_once_by_the_report() {
    let ground = terrain(|_, _| 50.0);
    let mut body = body_at(70.0, 70.0, 60.0);
    body.set_input(MovementInput::default());
    for _ in 0..(2.0 / PHYSICS_QUANTUM).round() as usize {
        body.step_quantum(&ground);
    }
    let report = body.take_impact_report().expect("the landing is an impact");
    assert!(report.speed > 0.0, "{report:?}");
    assert!((report.normal.z - 1.0).abs() < 1e-3, "{report:?}");
    assert!(body.last_impact_speed().is_some());
    assert_eq!(body.take_impact_report(), None);
}
