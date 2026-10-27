//! The impact glue: `gCollisionParameters`' minimums, `gCWPhysicsObject::SetCollisionState` (the
//! minimums reach the body) and `gCollisionProfile`'s report efuns.

use super::*;
use crate::native::{
    ntype, CollideProperty, CollisionParameters, CollisionProfile, NativeObject,
    ObjCollisionProfile,
};

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
        NativeObject::CollisionProfile(Box::default()),
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
        NativeObject::CollisionProfile(Box::new(CollisionProfile::impact([0.0, 0.0, 1.0], 4.0))),
    );
    let end = host.vm.refs.create_native(
        ntype::COLLISION_PROFILE,
        NativeObject::CollisionProfile(Box::new(CollisionProfile {
            state: 1,
            ..CollisionProfile::default()
        })),
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

/// A fresh script host and a `CollisionProfile` native holding `profile`.
fn host_with_profile(profile: CollisionProfile) -> (ScriptHost, u32) {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let report = host.vm.refs.create_native(
        ntype::COLLISION_PROFILE,
        NativeObject::CollisionProfile(Box::new(profile)),
    );
    (host, report)
}

fn prop(id: u64, collide_type: u64) -> CollideProperty {
    CollideProperty { id, collide_type }
}

/// `gCollisionProfile::CallEFun_GetObject(index, out)` copies the listed object into the out
/// native and answers 1; past the list it answers 0 and leaves the out native alone.
#[test]
fn get_object_copies_the_listed_object_or_answers_zero() {
    let profile = CollisionProfile {
        objects: vec![ObjCollisionProfile {
            id: 0x0102_0304_0506_0708,
            start_collide: true,
            is_ethereal: false,
        }],
        ..CollisionProfile::default()
    };
    let (mut host, report) = host_with_profile(profile);
    let out = host.vm.refs.create_native(
        ntype::OBJ_COLLISION_PROFILE,
        NativeObject::ObjCollisionProfile(ObjCollisionProfile::default()),
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0002, &[0, out], report, 4),
        1
    );
    let Some(NativeObject::ObjCollisionProfile(o)) = host.vm.native(out) else {
        panic!("the object out native");
    };
    assert_eq!(o.id, 0x0102_0304_0506_0708);
    assert!(o.start_collide);
    assert!(!o.is_ethereal);
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0002, &[1, out], report, 4),
        0,
        "an index past the list answers 0"
    );
}

/// The four property getters read their own lists: object (0x1B0005), environment (0x1B0007),
/// surface (0x1B0009) and water (0x1B000B), each with its own index range.
#[test]
fn the_property_getters_read_their_own_lists() {
    let profile = CollisionProfile {
        object_properties: vec![prop(1, 10)],
        env_properties: vec![prop(2, 20), prop(3, 30)],
        surface_properties: vec![prop(4, 40)],
        water_properties: vec![prop(5, 50)],
        ..CollisionProfile::default()
    };
    let (mut host, report) = host_with_profile(profile);
    let out = host.vm.refs.create_native(
        ntype::COLLIDE_PROPERTY,
        NativeObject::CollideProperty(CollideProperty::default()),
    );
    let getters = [
        (0x001B_0005u32, 0u32, (1u64, 10u64)),
        (0x001B_0007, 1, (3, 30)),
        (0x001B_0009, 0, (4, 40)),
        (0x001B_000B, 0, (5, 50)),
    ];
    for (id, index, (want_id, want_type)) in getters {
        assert_eq!(
            efun_result(&mut host.vm, id, &[index, out], report, 4),
            1,
            "efun {id:#x} index {index}"
        );
        let Some(NativeObject::CollideProperty(c)) = host.vm.native(out) else {
            panic!("the property out native");
        };
        assert_eq!((c.id, c.collide_type), (want_id, want_type), "efun {id:#x}");
    }
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0005, &[1, out], report, 4),
        0,
        "the object list has one property"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0009, &[1, out], report, 4),
        0,
        "the surface list has one property"
    );
}

/// `gCollisionProfile::CallEFun_HasContact(Plane *)` writes the contact plane for a contact
/// (state 0 or 2) and answers 1; no contact (state 3) answers 0 and leaves the plane alone.
#[test]
fn has_contact_writes_the_plane_for_a_contact() {
    let contact = CollisionProfile {
        contact_state: 0,
        contact_plane: ([0.0, 0.6, 0.8], 2.5),
        ..CollisionProfile::default()
    };
    let (mut host, report) = host_with_profile(contact);
    let plane = host
        .vm
        .refs
        .create_native(ntype::PLANE, NativeObject::Plane(([0.0; 3], 0.0)));
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_000E, &[plane], report, 3),
        1
    );
    assert_eq!(
        host.vm.native(plane),
        Some(&NativeObject::Plane(([0.0, 0.6, 0.8], 2.5)))
    );

    let (mut none, report) = host_with_profile(CollisionProfile::default());
    let plane = none
        .vm
        .refs
        .create_native(ntype::PLANE, NativeObject::Plane(([9.0; 3], 9.0)));
    assert_eq!(
        efun_result(&mut none.vm, 0x001B_000E, &[plane], report, 3),
        0
    );
    assert_eq!(
        none.vm.native(plane),
        Some(&NativeObject::Plane(([9.0; 3], 9.0))),
        "no contact leaves the plane untouched"
    );
}

/// `gCollisionProfile::CallEFun_FindObject(InstanceID, out)` copies the listed object with that
/// id (the id is two words) and answers 1; an unlisted id answers 0 and leaves the out alone.
#[test]
fn find_object_copies_the_object_with_that_id() {
    let id = 0x0000_0002_0000_0007u64;
    let profile = CollisionProfile {
        objects: vec![
            ObjCollisionProfile {
                id: 0x0000_0001_0000_0003,
                start_collide: true,
                is_ethereal: false,
            },
            ObjCollisionProfile {
                id,
                start_collide: false,
                is_ethereal: true,
            },
        ],
        ..CollisionProfile::default()
    };
    let (mut host, report) = host_with_profile(profile);
    let out = host.vm.refs.create_native(
        ntype::OBJ_COLLISION_PROFILE,
        NativeObject::ObjCollisionProfile(ObjCollisionProfile::default()),
    );
    // The id's low word is the first argument's slot (word 3), its high word the one above it.
    let (low, high) = (id as u32, (id >> 32) as u32);
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0003, &[high, low, out], report, 5),
        1
    );
    let Some(NativeObject::ObjCollisionProfile(o)) = host.vm.native(out) else {
        panic!("the object out native");
    };
    assert_eq!(o.id, id);
    assert!(o.is_ethereal && !o.start_collide);
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0003, &[0, 99, out], report, 5),
        0,
        "an unlisted id answers 0"
    );
}

/// `gCollisionProfile::CallEFun_IsEndContact` answers 1 only for contact state 1 (an end).
#[test]
fn is_end_contact_reads_the_contact_state() {
    let (mut host, end) = host_with_profile(CollisionProfile {
        contact_state: 1,
        ..CollisionProfile::default()
    });
    assert_eq!(efun_result(&mut host.vm, 0x001B_000F, &[], end, 2), 1);
    let (mut host, begin) = host_with_profile(CollisionProfile {
        contact_state: 0,
        ..CollisionProfile::default()
    });
    assert_eq!(efun_result(&mut host.vm, 0x001B_000F, &[], begin, 2), 0);
}
