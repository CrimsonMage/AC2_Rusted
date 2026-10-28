use ac2_physics::{Position, TargetOwner, TargetStatus, Vector};

use super::*;

fn owner(id: u64, x: f32) -> TargetOwner {
    let mut position = Position {
        cell_id: 0x0101_0001,
        ..Position::default()
    };
    position.frame.origin = Vector::new(x, 0.0, 0.0);
    TargetOwner {
        id,
        position,
        velocity: Vector::new(0.0, 0.0, 0.0),
        radius: 0.5,
        height: 2.0,
        scale: Vector::new(1.0, 1.0, 1.0),
    }
}

fn world<const N: usize>(x: [(u64, f32); N]) -> impl Fn(u64) -> Option<TargetOwner> {
    move |id| x.iter().find(|(i, _)| *i == id).map(|&(i, x)| owner(i, x))
}

#[test]
fn set_target_makes_the_target_tell_its_watcher_at_once() {
    let mut hub = TargetHub::new();
    let w = world([(1, 0.0), (2, 10.0)]);
    let notices = hub.set_target(1, 5, 2, 1.0, 0.5, 3.0, &w);
    assert_eq!(notices.len(), 1);
    let n = &notices[0];
    assert_eq!(n.object, 1);
    assert_eq!(n.info.status, TargetStatus::Ok);
    assert_eq!(n.info.object_id, 2);
    let t = hub.manager(2).expect("the target made a manager");
    assert_eq!(t.voyeurs().count(), 1);
    // The watcher aims at the target: straight along +x.
    let heading = hub
        .manager(1)
        .unwrap()
        .target_info()
        .unwrap()
        .interpolated_heading;
    assert_eq!((heading.x, heading.y, heading.z), (1.0, 0.0, 0.0));
}

#[test]
fn a_target_not_in_the_world_says_nothing_and_times_out_later() {
    let mut hub = TargetHub::new();
    let w = world([(1, 0.0), (9, 0.0)]);
    assert!(hub.set_target(1, 5, 2, 1.0, 0.5, 3.0, &w).is_empty());
    assert!(hub.manager(2).is_none());
    // Ten seconds of silence: the half-second sweep times the target out.
    let notices = hub.handle_targetting(14.0, &w);
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].info.status, TargetStatus::TimedOut);
}

#[test]
fn clearing_the_target_removes_the_voyeur() {
    let mut hub = TargetHub::new();
    let w = world([(1, 0.0), (2, 10.0)]);
    hub.set_target(1, 5, 2, 1.0, 0.5, 3.0, &w);
    hub.clear_target(1, 4.0, &w);
    assert_eq!(hub.manager(2).unwrap().voyeurs().count(), 0);
    assert!(hub.manager(1).unwrap().target_info().is_none());
}

#[test]
fn leaving_the_world_tells_the_voyeurs() {
    let mut hub = TargetHub::new();
    let w = world([(1, 0.0), (2, 10.0)]);
    hub.set_target(1, 5, 2, 1.0, 0.5, 3.0, &w);
    let notices = hub.leave_world(2, 4.0, &w);
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].object, 1);
    assert_eq!(notices[0].info.status, TargetStatus::ExitWorld);
}

#[test]
fn a_moved_target_updates_the_watcher_past_its_radius() {
    let mut hub = TargetHub::new();
    hub.set_target(1, 5, 2, 1.0, 0.5, 3.0, &world([(1, 0.0), (2, 10.0)]));
    let moved = world([(1, 0.0), (2, 15.0)]);
    let notices = hub.handle_targetting(4.0, &moved);
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].info.status, TargetStatus::Ok);
    assert_eq!(notices[0].info.interpolated_position.frame.origin.x, 15.0);
}
