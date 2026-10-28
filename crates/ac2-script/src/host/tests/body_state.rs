//! The `gCWPhysicsObject` body-state efuns: the freezes, the edge-slide flag, the marker toggle,
//! `IsOutside`, `GetCollisionState` and `DoBehaviorNonLatent`.

use super::*;
use crate::host::body_state::{BodySnapshot, BodyState};

fn host_with_weenie() -> ScriptHost {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    host.vm.owners.insert(9, 0x55);
    host
}

/// `SetMovementFrozen` and `SetAnimationFrozen` queue the body state and answer 1; an undefined
/// `this` crashes.
#[test]
fn the_freeze_efuns_queue_the_body_state() {
    let mut host = host_with_weenie();
    assert_eq!(efun_result(&mut host.vm, 0x0080_0054, &[1], 9, 3), 1);
    assert_eq!(efun_result(&mut host.vm, 0x0080_0056, &[0], 9, 3), 1);
    assert_eq!(
        host.take_body_states(),
        vec![
            (0x55, BodyState::MovementFrozen(true)),
            (0x55, BodyState::AnimationFrozen(false)),
        ]
    );
    assert_ne!(
        run_efun(&mut host.vm, 0x0080_0054, &[1], u32::MAX),
        InterpStatus::Success
    );
}

/// `IsOutside` is true for a landscape cell and false for an interior one or a body the host
/// has no snapshot of.
#[test]
fn is_outside_reads_the_cell() {
    let mut host = host_with_weenie();
    let snapshot = |cell| BodySnapshot {
        cell,
        collision: ac2_physics::CollisionParameters::default(),
    };
    assert_eq!(efun_result(&mut host.vm, 0x0080_0036, &[], 9, 2), 0);
    host.set_body_snapshot(0x55, Some(snapshot(0xA9B4_0021)));
    assert_eq!(efun_result(&mut host.vm, 0x0080_0036, &[], 9, 2), 1);
    host.set_body_snapshot(0x55, Some(snapshot(0xA9B4_0105)));
    assert_eq!(efun_result(&mut host.vm, 0x0080_0036, &[], 9, 2), 0);
}

/// `TogglePlaceMarkerObjects` flips the flag and answers its new value.
#[test]
fn the_marker_toggle_flips_the_flag() {
    let mut host = host_with_weenie();
    assert_eq!(efun_result(&mut host.vm, 0x0080_002B, &[], 9, 2), 1);
    assert_eq!(efun_result(&mut host.vm, 0x0080_002B, &[], 9, 2), 0);
}

/// `GetCollisionState` copies the body's physics type and placement and movement masks into the
/// parameters native; `DoBehaviorNonLatent` queues the server's `DoBehavior` and answers 0.
#[test]
fn collision_state_and_non_latent_behavior() {
    use ac2_net::protocol::{BehaviorParams, ObjectAction};
    use ac2_physics::PhysicsTypeMask;
    let mut host = host_with_weenie();
    let mut state = ac2_physics::CollisionParameters::default();
    state.set_physics_type(PhysicsTypeMask::from_bits(0x30));
    state.set_physics_placement_type(PhysicsTypeMask::from_bits(0x6));
    state.set_physics_movement_type(PhysicsTypeMask::from_bits(0x8));
    host.set_body_snapshot(
        0x55,
        Some(BodySnapshot {
            cell: 0xA9B4_0021,
            collision: state,
        }),
    );
    let native = host.vm.refs.create_native(
        crate::native::ntype::COLLISION_PARAMETERS,
        crate::native::NativeObject::CollisionParameters(Default::default()),
    );
    assert_eq!(efun_result(&mut host.vm, 0x0080_000B, &[native], 9, 3), 1);
    let Some(crate::native::NativeObject::CollisionParameters(p)) = host.vm.native(native) else {
        panic!("collision parameters");
    };
    assert_eq!(p.sets.physics_type().bits(), 0x30);
    assert_eq!(p.sets.placement_physics().bits(), 0x6);
    assert_eq!(p.sets.movement_physics().bits(), 0x8);

    let params = BehaviorParams {
        behavior: 0x41,
        ..BehaviorParams::default()
    };
    let behavior = host.vm.refs.create_native(
        0,
        crate::native::NativeObject::BehaviorParams(Box::new(params.clone())),
    );
    assert_eq!(efun_result(&mut host.vm, 0x0080_001C, &[behavior], 9, 3), 0);
    assert_eq!(
        host.take_physics(),
        vec![(0x55, ObjectAction::DoBehavior(Box::new(params)))]
    );
}
