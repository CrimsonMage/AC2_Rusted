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
            sets: Default::default(),
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
    CollideProperty {
        id,
        collide_type,
        ..CollideProperty::default()
    }
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

/// `gCollisionParameters::CallEFun_AddToEtherealList(InstanceID)` adds the id to the native's
/// ethereal lists: the body then passes through it, and answers 1.
#[test]
fn add_to_ethereal_list_makes_the_id_ethereal_on_the_native() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let native = host.vm.refs.create_native(
        ntype::COLLISION_PARAMETERS,
        NativeObject::CollisionParameters(CollisionParameters::default()),
    );
    // The 64-bit id 0x1_0000_0042: word 2 holds the high half, word 3 the low one.
    assert_eq!(
        efun_result(&mut host.vm, 0x001A_0001, &[1, 0x42], native, 4),
        1,
        "AddToEtherealList answers 1"
    );
    let Some(NativeObject::CollisionParameters(p)) = host.vm.native(native) else {
        panic!("the native is a CollisionParameters");
    };
    let none = ac2_physics::PhysicsTypeMask::default();
    assert!(!p.sets.blocks(0x1_0000_0042, none, false), "passed through");
    assert!(
        !p.sets.blocks(0x1_0000_0042, none, true),
        "and on placement"
    );
    assert!(
        p.sets.blocks(0x1_0000_0043, none, false),
        "other objects still block"
    );
}

/// `UpdateMovementEthereal` and the other mask efuns answer 0 for a name the master property
/// table lacks, and set no bit (`EtherealPhysicsType::SetProperty`). The name is outside
/// the property range, so no installed table holds it.
#[test]
fn a_mask_efun_with_an_unknown_property_changes_nothing() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let native = host.vm.refs.create_native(
        ntype::COLLISION_PARAMETERS,
        NativeObject::CollisionParameters(CollisionParameters::default()),
    );
    let unknown = [0, 0x7EAD_BEEF];
    for id in [
        0x001A_000B,
        0x001A_000C,
        0x001A_0010,
        0x001A_0011,
        0x001A_0007,
        0x001A_0008,
    ] {
        assert_eq!(
            efun_result(&mut host.vm, id, &unknown, native, 4),
            0,
            "{id:#010x}"
        );
    }
    let Some(NativeObject::CollisionParameters(p)) = host.vm.native(native) else {
        panic!("the native is a CollisionParameters");
    };
    // Retail sets the set's bitfield flag and clears its mask before the name is checked, so
    // only the observable state is compared: no physics-type bits, and the body still solid.
    let none = ac2_physics::PhysicsTypeMask::default();
    assert_eq!(p.sets.physics_type(), none);
    assert!(p.sets.blocks(7, none, false));
    assert!(p.sets.blocks(7, none, true));
}

/// With the DATs installed, a mask efun answers whether the master property table names the
/// property, as `MasterProperty::GetPropertyDesc` does. Skipped without `AC2_DAT_DIR`.
#[test]
fn a_mask_efun_answers_the_master_property_table() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(std::sync::Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let native = host.vm.refs.create_native(
        ntype::COLLISION_PARAMETERS,
        NativeObject::CollisionParameters(CollisionParameters::default()),
    );
    for name in [0x4000_0038_u32, 0x4000_001B, 0x4000_0026, 0x4000_001C] {
        let known = ac2_db::ac2_property_type(name).is_some();
        let answer = efun_result(&mut host.vm, 0x001A_000B, &[0, name], native, 4);
        assert_eq!(
            answer,
            u32::from(known),
            "UpdateMovementEthereal({name:#010x})"
        );
    }
}

/// A physics report fills the profile the efuns read: the impact (`HasCollided`), the contact
/// plane (`HasContact`), an object that began (`GetObject`) and a surface material
/// (`GetSurfaceProperty`). An end report answers `IsEndCollision`.
#[test]
fn a_physics_report_fills_the_profile_the_efuns_read() {
    use ac2_physics::{
        CollideProperty as PhysicsProperty, CollideType, CollisionReport, ContactRecord,
        ImpactReport, ObjectRecord, Plane, RecordState, Vector,
    };
    let report = CollisionReport {
        impact: Some(ImpactReport {
            normal: Vector::new(0.0, 0.0, 1.0),
            speed: 3.0,
        }),
        contact: Some(ContactRecord {
            plane: Plane {
                n: Vector::new(0.0, 0.6, 0.8),
                d: 2.5,
            },
            state: RecordState::Begin,
        }),
        objects: vec![ObjectRecord {
            id: 0x77,
            state: RecordState::Begin,
            ethereal: true,
        }],
        surface: vec![PhysicsProperty {
            id: 9,
            collide_type: CollideType::Begin,
            property: ac2_db::Ac2PropertyValue {
                name: 0x4000_0036,
                value_type: 0x4000_0007,
                bytes: 5u32.to_le_bytes().into(),
            },
        }],
        ..CollisionReport::default()
    };
    let (mut host, handle) = host_with_profile(CollisionProfile::from_report(&report));

    let normal = host
        .vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([0.0, 0.0, 0.0]));
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_000C, &[normal, 0], handle, 4),
        1
    );
    assert_eq!(
        host.vm.native(normal),
        Some(&NativeObject::Vector([0.0, 0.0, 1.0]))
    );

    let plane = host
        .vm
        .refs
        .create_native(ntype::PLANE, NativeObject::Plane(([0.0; 3], 0.0)));
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_000E, &[plane], handle, 3),
        1
    );
    assert_eq!(
        host.vm.native(plane),
        Some(&NativeObject::Plane(([0.0, 0.6, 0.8], 2.5)))
    );

    let out = host.vm.refs.create_native(
        ntype::OBJ_COLLISION_PROFILE,
        NativeObject::ObjCollisionProfile(ObjCollisionProfile::default()),
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0002, &[0, out], handle, 4),
        1
    );
    let Some(NativeObject::ObjCollisionProfile(o)) = host.vm.native(out) else {
        panic!("the object out native");
    };
    assert_eq!((o.id, o.start_collide, o.is_ethereal), (0x77, true, true));

    let surface = host.vm.refs.create_native(
        ntype::COLLIDE_PROPERTY,
        NativeObject::CollideProperty(CollideProperty::default()),
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x001B_0009, &[0, surface], handle, 4),
        1
    );
    let Some(NativeObject::CollideProperty(c)) = host.vm.native(surface) else {
        panic!("the surface out native");
    };
    assert_eq!(c.id, 9);

    // An end report: the contact ends and no impact is listed.
    let end = CollisionReport {
        contact: Some(ContactRecord {
            plane: Plane {
                n: Vector::new(0.0, 0.0, 1.0),
                d: 0.0,
            },
            state: RecordState::End,
        }),
        ..CollisionReport::default()
    };
    let (mut ended, end_handle) = host_with_profile(CollisionProfile::from_report(&end));
    assert_eq!(
        efun_result(&mut ended.vm, 0x001B_000F, &[], end_handle, 2),
        1,
        "IsEndContact"
    );
    assert_eq!(
        efun_result(&mut ended.vm, 0x001B_000D, &[], end_handle, 2),
        0
    );
    assert_eq!(
        efun_result(&mut ended.vm, 0x001B_000C, &[normal, 0], end_handle, 4),
        0
    );
}

/// The `gCollideProperty` efuns read the record a report listed: the master id (`GetID`) and the
/// begin, end and changed flags of its collide type. A surface record that changed reads as
/// changed; the id is the one the list gave it.
#[test]
fn the_collide_property_efuns_read_the_record_the_report_listed() {
    use ac2_physics::{CollideProperty as PhysicsProperty, CollideType, CollisionReport};
    let property = |id, collide_type| PhysicsProperty {
        id,
        collide_type,
        property: ac2_db::Ac2PropertyValue {
            name: 0x4000_0036,
            value_type: 0x4000_0007,
            bytes: 5u32.to_le_bytes().into(),
        },
    };
    let report = CollisionReport {
        surface: vec![
            property(7, CollideType::Begin),
            property(8, CollideType::Change),
            property(9, CollideType::End),
        ],
        ..CollisionReport::default()
    };
    let (mut host, handle) = host_with_profile(CollisionProfile::from_report(&report));
    let out = host.vm.refs.create_native(
        ntype::COLLIDE_PROPERTY,
        NativeObject::CollideProperty(CollideProperty::default()),
    );
    // (id, collide type) per surface index, with the flags' answers (begin, end, changed).
    let expected = [
        (7u64, 0u64, [1, 0, 0]),
        (8, 2, [0, 0, 1]),
        (9, 1, [0, 1, 0]),
    ];
    for (index, (id, collide_type, flags)) in expected.into_iter().enumerate() {
        assert_eq!(
            efun_result(&mut host.vm, 0x001B_0009, &[index as u32, out], handle, 4),
            1
        );
        let Some(NativeObject::CollideProperty(c)) = host.vm.native(out) else {
            panic!("the surface out native");
        };
        assert_eq!((c.id, c.collide_type), (id, collide_type));
        assert_eq!(
            efun_result(&mut host.vm, 0x0019_0001, &[], out, 2),
            id as u32,
            "GetID"
        );
        assert_eq!(
            efun_result(&mut host.vm, 0x0019_0004, &[], out, 2),
            flags[0],
            "IsBegin"
        );
        assert_eq!(
            efun_result(&mut host.vm, 0x0019_0005, &[], out, 2),
            flags[1],
            "IsEnd"
        );
        assert_eq!(
            efun_result(&mut host.vm, 0x0019_0006, &[], out, 2),
            flags[2],
            "IsChanged"
        );
    }
}
