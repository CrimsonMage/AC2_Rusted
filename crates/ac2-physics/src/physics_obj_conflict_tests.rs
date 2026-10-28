use super::*;
use crate::test_support::body_at;

const ME: InstanceId = 10;
const OTHER: InstanceId = 20;

fn peer(collision: CollisionParameters, blocking: PhysicsTypeMask) -> ConflictPeer {
    ConflictPeer {
        id: OTHER,
        collision,
        blocking,
    }
}

/// A body that is touching `OTHER`, with the entry ethereal or solid.
fn touching(ethereal: bool) -> crate::body::PhysicsBody {
    let mut body = body_at(70.0, 70.0, 50.0);
    body.with_object(|o| {
        o.instance_id = ME;
        o.collision_manager.begin_check(1.0);
        o.collision_manager.touch(OTHER, ethereal, false);
    });
    body
}

/// A mover set that blocks nothing the other body is made of (its physics set shares no type
/// with the other's blocking mask and its ethereal list holds the other's id).
fn passable_for(id: InstanceId) -> CollisionParameters {
    let mut p = CollisionParameters::default();
    p.add_to_ethereal_list(id);
    p
}

#[test]
fn a_passable_pair_makes_the_entry_ethereal() {
    let mut body = touching(false);
    let other = peer(passable_for(ME), PhysicsTypeMask::default());
    body.with_object(|o| {
        o.collision = passable_for(OTHER);
        assert_eq!(o.has_conflict(OTHER, Some(&other)), None);
        assert_eq!(o.collision_manager.entry_ethereal(OTHER), Some(true));
    });
}

#[test]
fn a_pair_that_stops_being_passable_ignores_each_other() {
    let mut body = touching(true);
    // Neither side lists the other as ethereal: they block each other.
    let other = peer(CollisionParameters::default(), PhysicsTypeMask::default());
    body.with_object(|o| {
        let effect = o.has_conflict(OTHER, Some(&other));
        assert_eq!(
            effect,
            Some(ConflictEffect {
                object: OTHER,
                add: ME
            })
        );
        assert_eq!(o.collision_manager.entry_ethereal(OTHER), Some(false));
        assert!(o.collision.in_temp_ethereal_list(OTHER));
        // Already solid: nothing more happens.
        assert_eq!(o.has_conflict(OTHER, Some(&other)), None);
    });
}

#[test]
fn a_missing_object_has_no_conflict() {
    let mut body = touching(true);
    body.with_object(|o| {
        assert_eq!(o.has_conflict(OTHER, None), None);
        assert_eq!(o.collision_manager.entry_ethereal(OTHER), Some(true));
    });
}

#[test]
fn a_temp_ethereal_object_is_passable_and_marked_ignored() {
    let mut info = CollisionParameters::default();
    assert!(check_if_solid(OTHER, PhysicsTypeMask::default(), &info, false).0);
    info.add_to_temp_ethereal_list(OTHER);
    assert_eq!(
        check_if_solid(OTHER, PhysicsTypeMask::default(), &info, false),
        (false, true)
    );
}

#[test]
fn update_conflict_list_checks_every_touched_object() {
    let mut body = touching(true);
    body.with_object(|o| {
        let effects = o.update_conflict_list(&|_| {
            Some(peer(
                CollisionParameters::default(),
                PhysicsTypeMask::default(),
            ))
        });
        assert_eq!(effects.len(), 1);
    });
}
