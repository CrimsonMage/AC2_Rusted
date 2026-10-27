//! The impact glue: `gCollisionParameters`' minimums, `gCWPhysicsObject::SetCollisionState` (the
//! minimums reach the body) and `gCollisionProfile`'s report efuns.

use super::*;
use crate::native::{ntype, CollisionParameters, CollisionProfile, NativeObject};

/// `gCollisionParameters::CallEFun_SetMinReportedImpactXY` and `Z` write the native's two
/// minimums (retail's +0xA8 and +0xAC); no result.
#[test]
fn the_minimum_efuns_write_the_parameters_native() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let params = host.vm.refs.create_native(
        ntype::COLLISION_PARAMETERS,
        NativeObject::CollisionParameters(CollisionParameters::default()),
    );
    assert_eq!(
        run_efun(&mut host.vm, 0x001A_0002, &[2.5f32.to_bits()], params),
        InterpStatus::Success
    );
    assert_eq!(
        run_efun(&mut host.vm, 0x001A_0003, &[7.0f32.to_bits()], params),
        InterpStatus::Success
    );
    let Some(NativeObject::CollisionParameters(p)) = host.vm.native(params) else {
        panic!("the parameters native");
    };
    assert_eq!(p.min_reported_impact_xy, 2.5);
    assert_eq!(p.min_reported_impact_z, 7.0);
}

/// An undefined or foreign `this` crashes the minimum efuns, as retail's `index !=
/// Undef_ReferenceID` assert does.
#[test]
fn the_minimum_efuns_crash_on_a_foreign_this() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let other = host.vm.refs.create_native(
        ntype::COLLISION_PROFILE,
        NativeObject::CollisionProfile(CollisionProfile::default()),
    );
    assert_ne!(
        run_efun(&mut host.vm, 0x001A_0002, &[1.0f32.to_bits()], other),
        InterpStatus::Success
    );
}

/// `gCWPhysicsObject::CallEFun_SetCollisionState` queues the parameters' minimums for the
/// weenie's body, and answers 0.
#[test]
fn set_collision_state_queues_the_parameters_minimums_for_the_weenie() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    host.vm.owners.insert(9, 0x55);
    let params = host.vm.refs.create_native(
        ntype::COLLISION_PARAMETERS,
        NativeObject::CollisionParameters(CollisionParameters {
            min_reported_impact_xy: 2.5,
            min_reported_impact_z: 0.0,
        }),
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_000A, &[params], 9, 3),
        0,
        "SetCollisionState answers 0"
    );
    assert_eq!(host.take_collision_minimums(), vec![(0x55, 2.5, 0.0)]);
    assert!(host.take_collision_minimums().is_empty());
}

/// `gCollisionProfile::CallEFun_HasCollided` writes an impact's normal into the out vector and
/// answers 1; `IsEndCollision` answers 0 for an impact and 1 for an end.
#[test]
fn the_collision_report_efuns_read_an_impact() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let impact = host.vm.refs.create_native(
        ntype::COLLISION_PROFILE,
        NativeObject::CollisionProfile(CollisionProfile::impact([0.0, 0.0, 1.0], 4.0)),
    );
    let end = host.vm.refs.create_native(
        ntype::COLLISION_PROFILE,
        NativeObject::CollisionProfile(CollisionProfile {
            state: 1,
            ..CollisionProfile::default()
        }),
    );
    let normal = host
        .vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([0.0, 0.0, 0.0]));
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_000C, &[normal, 0], impact, 4),
        1,
        "HasCollided answers 1 for an impact"
    );
    assert_eq!(
        host.vm.native(normal),
        Some(&NativeObject::Vector([0.0, 0.0, 1.0]))
    );
    assert_eq!(efun_result(&mut host.vm, 0x001B_000D, &[], impact, 2), 0);
    assert_eq!(efun_result(&mut host.vm, 0x001B_000D, &[], end, 2), 1);
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_000C, &[normal, 0], end, 4),
        0,
        "HasCollided answers 0 for an end"
    );
}
