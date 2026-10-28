use std::collections::HashMap;

use super::*;
use crate::collision_impact::ImpactReport;
use crate::collision_report::{CollisionReport, ObjectRecord, RecordState};
use crate::vector::Vector;

#[derive(Default)]
struct Host {
    calls: Vec<String>,
}

impl MissileHost for Host {
    fn current_gravity(&self) -> Vector {
        Vector::ZERO
    }
    fn current_wind(&self) -> Vector {
        Vector::ZERO
    }
    fn is_in_world(&self) -> bool {
        true
    }
    fn set_missile_velocity(&mut self, _: Vector) {}
    fn report_out_of_fuel(&mut self) {}
    fn report_expired(&mut self) {}
    fn stop_fx(&mut self, fx: u32) {
        self.calls.push(format!("stop_fx {fx:x}"));
    }
    fn do_fx(&mut self, _: u32, _: f32) {}
    fn start_streak(&mut self) {}
    fn stop_streak(&mut self) {
        self.calls.push("stop_streak".into());
    }
}

impl MissileCollisionHost for Host {
    fn delete_self(&mut self, fade: f64, delay: f64) {
        self.calls.push(format!("delete {fade} {delay}"));
    }
    fn sticky_set_parent(&mut self, object: u64) {
        self.calls.push(format!("stick {object}"));
    }
}

#[derive(Default)]
struct Table(HashMap<u64, ImpactTarget>);

impl ImpactLookup for Table {
    fn target(&self, id: u64) -> Option<ImpactTarget> {
        self.0.get(&id).copied()
    }
}

fn target(id: u64, dynamic: bool, weenie: Option<(bool, bool)>) -> ImpactTarget {
    ImpactTarget {
        id,
        dynamic,
        has_missile_manager: false,
        weenie,
    }
}

fn record(id: u64, state: RecordState, ethereal: bool) -> ObjectRecord {
    ObjectRecord {
        id,
        state,
        ethereal,
    }
}

fn report(objects: Vec<ObjectRecord>, impact: bool) -> CollisionReport {
    CollisionReport {
        objects,
        impact: impact.then_some(ImpactReport {
            normal: Vector::new(0.0, 0.0, 1.0),
            speed: 3.0,
        }),
        ..CollisionReport::default()
    }
}

fn manager() -> MissileManager {
    let mut m = MissileManager::default();
    m.target_info.source_id = 1;
    m.target_info.target_id = 2;
    m
}

#[test]
fn a_solid_object_that_began_colliding_detonates() {
    let m = manager();
    let table = Table(HashMap::from([(9, target(9, true, None))]));
    let r = report(vec![record(9, RecordState::Begin, false)], false);
    assert_eq!(
        m.should_detonate(&r, &table),
        Some(Some(target(9, true, None)))
    );
    // An object that ended, or none at all, does not.
    let ended = report(vec![record(9, RecordState::End, false)], false);
    assert_eq!(m.should_detonate(&ended, &table), None);
}

#[test]
fn an_ethereal_object_detonates_only_the_target_unless_it_detonates_on_miss() {
    let mut m = manager();
    let table = Table(HashMap::from([
        (1, target(1, true, None)),
        (2, target(2, true, None)),
        (3, target(3, true, Some((false, true)))),
        (4, target(4, true, Some((false, false)))),
    ]));
    let ethereal = |id| report(vec![record(id, RecordState::Begin, true)], false);
    assert!(m.should_detonate(&ethereal(1), &table).is_none(), "source");
    assert!(m.should_detonate(&ethereal(2), &table).is_some(), "target");
    assert!(
        m.should_detonate(&ethereal(3), &table).is_none(),
        "bystander"
    );
    m.info.detonates_on_miss = true;
    assert!(
        m.should_detonate(&ethereal(2), &table).is_none(),
        "target alone is not enough"
    );
    assert!(
        m.should_detonate(&ethereal(3), &table).is_some(),
        "weenie slot 0x94"
    );
    assert!(m.should_detonate(&ethereal(4), &table).is_none());
    assert!(
        m.should_detonate(&ethereal(1), &table).is_none(),
        "source never"
    );
}

#[test]
fn an_impact_detonates_on_the_world() {
    let m = manager();
    let r = report(Vec::new(), true);
    assert_eq!(m.should_detonate(&r, &Table::default()), Some(None));
    assert_eq!(
        m.should_detonate(&report(Vec::new(), false), &Table::default()),
        None
    );
}

#[test]
fn a_detonation_stops_the_missile_and_deletes_it() {
    let mut m = manager();
    m.is_moving = true;
    let mut host = Host::default();
    let r = report(Vec::new(), true);
    m.handle_collision(&mut host, &r, &Table::default());
    assert!(!m.is_activated && !m.is_moving);
    assert_eq!(
        host.calls,
        ["stop_fx 40000001", "stop_streak", "delete 0.5 0"]
    );
}

#[test]
fn a_sticky_missile_sticks_to_a_dynamic_object_without_a_missile_manager() {
    let mut m = manager();
    m.info.sticks_upon_collide = true;
    let table = Table(HashMap::from([
        (9, target(9, true, None)),
        (
            8,
            ImpactTarget {
                has_missile_manager: true,
                ..target(8, true, None)
            },
        ),
        (7, target(7, false, None)),
    ]));
    for (id, sticks) in [(9, true), (8, false), (7, false)] {
        let mut host = Host::default();
        let mut missile = m.clone();
        missile.handle_collision(
            &mut host,
            &report(vec![record(id, RecordState::Begin, false)], false),
            &table,
        );
        assert_eq!(
            host.calls.iter().any(|c| c.starts_with("stick")),
            sticks,
            "{id}"
        );
        assert!(!host.calls.iter().any(|c| c.starts_with("delete")));
    }
}

#[test]
fn proximity_needs_the_bit_the_chance_and_the_target() {
    let mut m = manager();
    assert!(!m.detonates_on_proximity_to(2));
    m.info.bitfield = 0x2_0000;
    m.info.detonation_chance = 0.5;
    m.target_info.missile_detonation_chance = 0.5;
    assert!(m.detonates_on_proximity_to(2));
    assert!(!m.detonates_on_proximity_to(3));
    m.target_info.missile_detonation_chance = 0.75;
    assert!(!m.detonates_on_proximity_to(2));
}
