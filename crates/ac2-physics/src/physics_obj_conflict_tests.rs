use super::*;
use crate::test_support::body_at;

const ME: InstanceId = 10;
const OTHER: InstanceId = 20;

fn peer(collision: CollisionParameters, blocking: PhysicsTypeMask) -> ConflictPeer {
    ConflictPeer {
        id: OTHER,
        collision,
        blocking,
        position: None,
        properties: Vec::new(),
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

#[test]
fn a_collision_state_with_a_physics_type_sets_the_blocking_mask() {
    // `CCollisionManager::SetCollisionState` -> `CPhysicsObj::SetPhysicsCollideMask`.
    let mut body = body_at(70.0, 70.0, 50.0);
    assert_eq!(body.blocking_bits(), 0);
    let mut info = CollisionParameters::default();
    info.set_physics_type(PhysicsTypeMask::from_bits(0b1010));
    body.set_collision_state(&info, &|_| None);
    assert_eq!(body.blocking_bits(), 0b1010);
    assert!(body.has_collision_manager());
    // A state that does not name the physics type leaves it alone.
    body.set_collision_state(&CollisionParameters::default(), &|_| None);
    assert_eq!(body.blocking_bits(), 0b1010);
}

#[test]
fn a_scenery_obstacle_the_mover_passes_adds_its_mask_to_the_collide_mask() {
    // `AddObjectCollision` for an object with no `InstanceID`: when it is not solid to the mover
    // its blocking mask joins the manager's current collide mask; the result is 1 with an object.
    use crate::collision_types::{CollisionParameters, PhysicsTypeMask};
    use crate::transition::collision::CollisionInfo;
    let mut body = body_at(70.0, 70.0, 50.0);
    let mut info = CollisionParameters::default();
    info.set_physics_movement_type(PhysicsTypeMask::from_bits(0b0001));
    body.set_collision_state(&info, &|_| None);
    let collisions = CollisionInfo {
        collide_objects: vec![3, 4],
        // Shares bit 0 with the mover's movement set: ethereal to it. Bit 5 it does not share.
        scenery: vec![
            PhysicsTypeMask::from_bits(0b0011),
            PhysicsTypeMask::from_bits(0b10_0000),
        ],
        ..CollisionInfo::default()
    };
    let mut added = false;
    body.with_object(|o| added = o.handle_all_collisions(&collisions, true));
    assert!(added);
    // Only the passable one joined: the movement set is the desired mask and its bits.
    assert_eq!(body.collision_mut().movement_physics().bits(), 0b0011);
}
