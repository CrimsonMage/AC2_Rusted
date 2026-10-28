//! `CCollisionManager::ReportCollision`: the object table's begin and end, the material layers
//! and the contact plane, on the manager alone and on a mover hitting a server object.

use std::sync::Arc;

use ac2_db::{
    Ac2PropertyDesc, Ac2PropertyGroup, Ac2PropertyValue, REPORT_IGNORED, REPORT_INQUIRABLE,
    REPORT_REPORTABLE,
};

use crate::body::{MovementInput, PhysicsBody, PHYSICS_QUANTUM};
use crate::collide_property::{CollideType, PropertyDescs, TERRAIN_TYPE_PROPERTY};
use crate::collision_impact::ImpactReport;
use crate::collision_report::{
    BodyContact, CollisionManager, CollisionReport, ContactMaterials, RecordState, ReportFlags,
};
use crate::frame::Frame;
use crate::landdefs::{lcoord_to_gid, SIDE_VERTEX_COUNT};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND};
use crate::objects::{ObjectShape, StaticObject};
use crate::physics_material::PhysicsMaterial;
use crate::physics_obj::BodyShape;
use crate::position::Position;
use crate::shapes::cylsphere::CylSphere;
use crate::terrain::Terrain;
use crate::test_support::{desc, walking_values};
use crate::vector::{Plane, Quaternion, Sphere, Vector};

const OBJECT_ID: u64 = 0x5000_0077;
const BLOCK: u32 = 0x7F7F_FFFF;

fn plain() -> ReportFlags {
    ReportFlags {
        end_reporting: false,
        end_all: false,
        active: true,
    }
}

fn no_contact() -> BodyContact {
    BodyContact::default()
}

fn on_ground(material: Plane) -> BodyContact {
    BodyContact {
        plane: Some(material),
        on_landscape: true,
        ..BodyContact::default()
    }
}

fn floor() -> Plane {
    Plane {
        n: Vector::new(0.0, 0.0, 1.0),
        d: 0.0,
    }
}

fn materials(landscape: Option<u64>) -> ContactMaterials {
    ContactMaterials {
        landscape: landscape.map(|id| Arc::new(PhysicsMaterial::with_id(id))),
        ..ContactMaterials::default()
    }
}

const TERRAIN_VALUE: u32 = 0x42;
/// Two property names the synthetic table knows: one reported to the weenie, one inquirable only.
const NAME_REPORTED: u32 = 0x1000_0001;
const NAME_INQUIRABLE: u32 = 0x1000_0002;
/// A name the table does not know, or that is ignored: not inquirable.
const NAME_IGNORED: u32 = 0x1000_0003;

/// The property table of these tests: the terrain-type property reported, the names above with
/// their reports, the last one ignored.
fn synthetic_desc(name: u32) -> Option<Ac2PropertyDesc> {
    let report = match name {
        TERRAIN_TYPE_PROPERTY | NAME_REPORTED => REPORT_REPORTABLE,
        NAME_INQUIRABLE => REPORT_INQUIRABLE,
        NAME_IGNORED => REPORT_IGNORED,
        _ => return None,
    };
    Some(Ac2PropertyDesc {
        value_type: 0x4000_0002,
        group: 0,
        data: 0,
        report_to_weenie: report,
    })
}

/// A manager that reads [`synthetic_desc`] instead of the installed table.
fn manager() -> CollisionManager {
    let mut m = CollisionManager::default();
    m.descs = PropertyDescs(synthetic_desc);
    m
}

/// A material with one group of integer properties.
fn material(id: u64, properties: &[(u32, u32)]) -> Arc<PhysicsMaterial> {
    Arc::new(PhysicsMaterial {
        id,
        groups: vec![Ac2PropertyGroup {
            name: 0x4000_000A,
            properties: properties
                .iter()
                .map(|&(name, value)| Ac2PropertyValue {
                    name,
                    value_type: 0x4000_0002,
                    bytes: value.to_le_bytes().into(),
                })
                .collect(),
        }],
    })
}

fn landscape(material: Arc<PhysicsMaterial>) -> ContactMaterials {
    ContactMaterials {
        landscape: Some(material),
        ..ContactMaterials::default()
    }
}

/// The listed properties, as `(name, collide type)`.
fn listed(props: &[crate::collide_property::CollideProperty]) -> Vec<(u32, CollideType)> {
    props.iter().map(|p| (p.name(), p.collide_type)).collect()
}

/// The objects a report lists, as `(id, began)`.
fn objects_of(report: &CollisionReport) -> Vec<(u64, bool)> {
    report
        .objects
        .iter()
        .map(|o| (o.id, o.state == RecordState::Begin))
        .collect()
}

#[test]
fn a_touched_object_begins_then_ends_once_it_is_not_touched_for_a_second() {
    let mut m = CollisionManager::default();
    m.begin_check(10.0);
    m.touch(OBJECT_ID, false, false);
    let began = m.report(10.0, no_contact(), &ContactMaterials::default(), plain());
    assert_eq!(objects_of(&began), vec![(OBJECT_ID, true)]);

    // Still touched in the check at 10.5: nothing is listed.
    m.begin_check(10.5);
    m.touch(OBJECT_ID, false, false);
    let held = m.report(10.5, no_contact(), &ContactMaterials::default(), plain());
    assert!(held.objects.is_empty(), "{held:?}");

    // Not touched in the check at 12, and over a second since the last touch: it ends.
    m.begin_check(12.0);
    let ended = m.report(12.0, no_contact(), &ContactMaterials::default(), plain());
    assert_eq!(objects_of(&ended), vec![(OBJECT_ID, false)]);

    // Dropped from the table: a later report lists nothing for it.
    m.begin_check(12.5);
    let after = m.report(12.5, no_contact(), &ContactMaterials::default(), plain());
    assert!(after.objects.is_empty(), "{after:?}");
}

#[test]
fn an_object_within_a_second_of_its_touch_is_not_ended_yet() {
    let mut m = CollisionManager::default();
    m.begin_check(0.0);
    m.touch(OBJECT_ID, true, false);
    m.report(0.0, no_contact(), &ContactMaterials::default(), plain());
    m.begin_check(0.5);
    let quiet = m.report(0.5, no_contact(), &ContactMaterials::default(), plain());
    assert!(quiet.objects.is_empty(), "{quiet:?}");
}

#[test]
fn end_all_ends_every_reported_object_and_drops_the_table() {
    let mut m = CollisionManager::default();
    m.begin_check(3.0);
    m.touch(OBJECT_ID, true, false);
    m.report(3.0, no_contact(), &ContactMaterials::default(), plain());
    let ended = m.report(
        3.0,
        no_contact(),
        &ContactMaterials::default(),
        ReportFlags {
            end_reporting: true,
            end_all: true,
            active: true,
        },
    );
    assert_eq!(objects_of(&ended), vec![(OBJECT_ID, false)]);
    assert!(
        ended.objects[0].ethereal,
        "keeps the ethereal flag it was reported with"
    );
    m.begin_check(3.1);
    let again = m.report(3.1, no_contact(), &ContactMaterials::default(), plain());
    assert!(again.objects.is_empty());
}

#[test]
fn an_inactive_body_begins_nothing_and_discards_its_impact() {
    let mut m = CollisionManager::default();
    m.begin_check(1.0);
    m.touch(OBJECT_ID, false, false);
    m.set_impact(ImpactReport {
        normal: Vector::new(0.0, 0.0, 1.0),
        speed: 4.0,
    });
    let inactive = m.report(
        1.0,
        on_ground(floor()),
        &materials(Some(9)),
        ReportFlags {
            active: false,
            ..plain()
        },
    );
    assert!(inactive.is_empty(), "{inactive:?}");
    // The impact was taken by the inactive report: a later active one has none.
    let later = m.report(1.0, no_contact(), &ContactMaterials::default(), plain());
    assert!(later.impact.is_none());
}

#[test]
fn an_impact_is_listed_once() {
    let mut m = CollisionManager::default();
    m.set_impact(ImpactReport {
        normal: Vector::new(0.0, 0.0, 1.0),
        speed: 2.5,
    });
    let first = m.report(0.0, no_contact(), &ContactMaterials::default(), plain());
    assert_eq!(first.impact.map(|i| i.speed), Some(2.5));
    let second = m.report(0.0, no_contact(), &ContactMaterials::default(), plain());
    assert!(second.impact.is_none());
}

#[test]
fn the_landscape_layer_begins_changes_and_ends_its_properties() {
    let mut m = manager();
    let a = material(9, &[(NAME_REPORTED, 1), (NAME_INQUIRABLE, 2)]);
    let first = m.report(0.0, on_ground(floor()), &landscape(a.clone()), plain());
    assert_eq!(
        listed(&first.surface),
        vec![(NAME_REPORTED, CollideType::Begin)],
        "only the reported property is listed; the inquirable one is kept, not listed"
    );
    // The same material is not read again.
    let same = m.report(0.1, on_ground(floor()), &landscape(a), plain());
    assert!(same.surface.is_empty());
    // A new material: its properties the layer already holds are the same, the others end.
    let b = material(10, &[(NAME_REPORTED, 1)]);
    let changed = m.report(0.2, on_ground(floor()), &landscape(b), plain());
    assert!(changed.surface.is_empty(), "{changed:?}");
    let c = material(11, &[(NAME_REPORTED, 1)]);
    let swapped = m.report(0.3, on_ground(floor()), &landscape(c), plain());
    assert!(swapped.surface.is_empty());
    // Leaving the landscape ends every reported property it held.
    let left = m.report(0.4, no_contact(), &ContactMaterials::default(), plain());
    assert_eq!(
        listed(&left.surface),
        vec![(NAME_REPORTED, CollideType::End)]
    );
}

#[test]
fn a_material_property_that_is_gone_ends_when_the_material_changes() {
    let mut m = manager();
    let a = material(9, &[(NAME_REPORTED, 1)]);
    m.report(0.0, on_ground(floor()), &landscape(a), plain());
    let b = material(10, &[]);
    let changed = m.report(0.1, on_ground(floor()), &landscape(b), plain());
    assert_eq!(
        listed(&changed.surface),
        vec![(NAME_REPORTED, CollideType::End)]
    );
}

#[test]
fn a_layer_without_a_material_lists_nothing() {
    let mut m = manager();
    let report = m.report(0.0, on_ground(floor()), &materials(None), plain());
    assert!(report.surface.is_empty(), "{report:?}");
}

#[test]
fn the_terrain_type_property_sets_the_motion_terrain_and_clears_it_when_left() {
    let mut m = manager();
    let a = material(9, &[(TERRAIN_TYPE_PROPERTY, TERRAIN_VALUE)]);
    let on = m.report(0.0, on_ground(floor()), &landscape(a), plain());
    assert_eq!(
        listed(&on.surface),
        vec![(TERRAIN_TYPE_PROPERTY, CollideType::Begin)]
    );
    assert_eq!(m.take_terrain(), Some(u64::from(TERRAIN_VALUE)));
    assert_eq!(m.take_terrain(), None, "taken once");
    // Leaving: the land terrain ends, the sentinel is the motion terrain again.
    m.report(0.1, no_contact(), &ContactMaterials::default(), plain());
    assert_eq!(m.take_terrain(), Some(0x8000_0001));
}

#[test]
fn a_terrain_property_the_table_does_not_report_still_sets_the_terrain() {
    // The terrain-type property is inquirable only: the script does not see it, the body does.
    fn inquirable_terrain(name: u32) -> Option<Ac2PropertyDesc> {
        match name {
            TERRAIN_TYPE_PROPERTY => Some(Ac2PropertyDesc {
                value_type: 0x4000_0002,
                group: 0,
                data: 0,
                report_to_weenie: REPORT_INQUIRABLE,
            }),
            _ => synthetic_desc(name),
        }
    }
    let mut m = CollisionManager::default();
    m.descs = PropertyDescs(inquirable_terrain);
    let a = material(9, &[(TERRAIN_TYPE_PROPERTY, TERRAIN_VALUE)]);
    let report = m.report(0.0, on_ground(floor()), &landscape(a), plain());
    assert!(report.surface.is_empty(), "{report:?}");
    assert_eq!(m.take_terrain(), Some(u64::from(TERRAIN_VALUE)));
}

#[test]
fn the_water_terrain_wins_over_the_landscape_terrain() {
    let mut m = manager();
    let ground = material(9, &[(TERRAIN_TYPE_PROPERTY, 5)]);
    let wet = material(20, &[(TERRAIN_TYPE_PROPERTY, 7)]);
    let mats = ContactMaterials {
        landscape: Some(ground),
        water: Some(wet),
        ..ContactMaterials::default()
    };
    let contact = BodyContact {
        in_water: true,
        on_landscape: true,
        ..on_ground(floor())
    };
    let report = m.report(0.0, contact, &mats, plain());
    assert_eq!(
        listed(&report.water),
        vec![(TERRAIN_TYPE_PROPERTY, CollideType::Begin)]
    );
    assert_eq!(m.take_terrain(), Some(7));
    // Leaving the water, the land terrain applies again.
    let out = m.report(
        0.1,
        on_ground(floor()),
        &landscape(material(9, &[(TERRAIN_TYPE_PROPERTY, 5)])),
        plain(),
    );
    assert_eq!(
        listed(&out.water),
        vec![(TERRAIN_TYPE_PROPERTY, CollideType::End)]
    );
    assert_eq!(m.take_terrain(), Some(5));
}

#[test]
fn an_ignored_property_is_not_kept() {
    let mut m = manager();
    let a = material(9, &[(NAME_IGNORED, 1)]);
    let report = m.report(0.0, on_ground(floor()), &landscape(a), plain());
    assert!(report.surface.is_empty());
    assert!(!report.is_empty() || report.surface.is_empty());
}

#[test]
fn the_contact_plane_begins_with_contact_and_ends_with_the_plane_it_listed() {
    let mut m = CollisionManager::default();
    let plane = Plane {
        n: Vector::new(0.0, 0.0, 1.0),
        d: -2.0,
    };
    let began = m.report(0.0, on_ground(plane), &ContactMaterials::default(), plain());
    let c = began.contact.expect("contact begins");
    assert_eq!((c.plane.d, c.state), (-2.0, RecordState::Begin));
    let held = m.report(0.1, on_ground(plane), &ContactMaterials::default(), plain());
    assert!(held.contact.is_none());
    let ended = m.report(0.2, no_contact(), &ContactMaterials::default(), plain());
    let c = ended.contact.expect("contact ends");
    assert_eq!((c.plane.d, c.state), (-2.0, RecordState::End));
}

#[test]
fn a_report_with_nothing_to_say_is_empty() {
    let mut m = CollisionManager::default();
    assert!(m
        .report(0.0, no_contact(), &ContactMaterials::default(), plain())
        .is_empty());
}

// --- A mover against a server object (`CPhysicsObj` through a `CTransition`).

const BLOCK_CELL_STEP: f32 = 20.0;

fn cell_at(x: f32, y: f32) -> u32 {
    lcoord_to_gid(
        0x7F * 8 + (x / BLOCK_CELL_STEP) as i32,
        0x7F * 8 + (y / BLOCK_CELL_STEP) as i32,
    )
}

fn position(x: f32, y: f32, z: f32) -> Position {
    Position::new(
        cell_at(x, y),
        Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY),
    )
}

fn flat() -> Terrain {
    let heights = vec![0.0; SIDE_VERTEX_COUNT * SIDE_VERTEX_COUNT];
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK, &heights, |_, _| false);
    terrain
}

fn pillar() -> Arc<ObjectShape> {
    Arc::new(ObjectShape::new(
        Vec::new(),
        vec![CylSphere {
            low: Vector::ZERO,
            height: 3.0,
            radius: 0.5,
        }],
        Vec::new(),
    ))
}

/// A walker: a body of two spheres, walking on the ground and in the air.
fn walker(x: f32, y: f32) -> PhysicsBody {
    let mut air = walking_values();
    air.forward_acceleration = 0.0;
    air.backward_acceleration = 0.0;
    air.strafe_acceleration = 0.0;
    let shape = BodyShape {
        spheres: vec![
            Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4),
            Sphere::new(Vector::new(0.0, 0.0, 1.2), 0.4),
        ],
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let desc = desc(&[(ELEMENT_GROUND, walking_values()), (ELEMENT_AIR, air)]);
    PhysicsBody::new(position(x, y, 0.0), shape, Some(desc))
}

/// One quantum at `now`, then the report for the step.
fn step(
    body: &mut PhysicsBody,
    world: &Terrain,
    now: f64,
    mats: &ContactMaterials,
) -> Option<CollisionReport> {
    body.set_time(now);
    body.step_quantum(world);
    body.report_collisions(mats, false)
}

#[test]
fn a_mover_hitting_a_server_object_lists_it_begin_then_end_and_its_landscape_under_it() {
    let mut world = flat();
    let at = position(80.2, 84.0, 0.0);
    world.insert_object(
        StaticObject::dynamic(at, Vector::new(1.0, 1.0, 1.0), pillar(), true)
            .with_instance_id(OBJECT_ID),
    );
    let mut body = walker(80.0, 80.0);
    let mats = ContactMaterials {
        landscape: Some(Arc::new(PhysicsMaterial::with_id(9))),
        ..ContactMaterials::default()
    };

    // Settle on the ground, then walk north into the object. The mover catches its edge and
    // slides around it, so the object is touched for a while and then left behind.
    let mut now = 0.0;
    let mut reports = Vec::new();
    for _ in 0..5 {
        now += PHYSICS_QUANTUM as f64;
        reports.extend(step(&mut body, &world, now, &mats));
    }
    body.set_input(MovementInput {
        motion: [0.0, 1.0, 0.0],
        heading: Some(0.0),
        turn: 0.0,
    });
    for _ in 0..(3.0 / PHYSICS_QUANTUM).round() as usize {
        now += PHYSICS_QUANTUM as f64;
        reports.extend(step(&mut body, &world, now, &mats));
    }

    let objects: Vec<_> = reports
        .iter()
        .flat_map(|r| r.objects.iter().copied())
        .filter(|o| o.id == OBJECT_ID)
        .collect();
    let states: Vec<_> = objects.iter().map(|o| o.state).collect();
    assert_eq!(
        states,
        vec![RecordState::Begin, RecordState::End],
        "one begin when it is hit, one end once it is left behind: {objects:?}"
    );
    assert!(!objects[0].ethereal && !objects[1].ethereal);

    // The landscape material has no property the unit tests' table names: nothing is listed for
    // it. The contact plane begins with the landing.
    assert!(reports.iter().all(|r| r.surface.is_empty()));
    assert!(reports
        .iter()
        .any(|r| r.contact.is_some_and(|c| c.state == RecordState::Begin)));
}

#[test]
fn a_changed_ignore_flag_marks_a_reported_entry() {
    let mut m = CollisionManager::default();
    m.begin_check(10.0);
    m.touch(OBJECT_ID, false, false);
    // Not reported yet: the flag changes without marking.
    m.touch(OBJECT_ID, false, true);
    assert_eq!(m.entry_ignore(OBJECT_ID), Some((true, false)));
    m.report(10.0, no_contact(), &ContactMaterials::default(), plain());
    m.begin_check(10.5);
    m.touch(OBJECT_ID, false, false);
    assert_eq!(m.entry_ignore(OBJECT_ID), Some((false, true)));
}
