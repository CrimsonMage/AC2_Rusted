//! What the remote bodies' collisions hand to the other bodies and to the scripts in one physics
//! quantum: `CPhysicsObj::handle_all_collisions` adds the mover to the manager of each object it
//! hit, `CCollisionManager::ReportCollision` erases an expired object and tells it to drop the
//! reporting body too (`CPhysicsObj::CollisionRemoveObject`), and `CPhysicsObj::ReportCollisions`
//! hands the report to the weenie's `HandleCollision`. A remote's side is done inside
//! `Remotes::step_quantum`; what concerns the player's body, which the remotes do not hold, waits
//! here for the client.

use ac2_physics::{CollisionReport, ConflictPeer, Vector};

/// A mover that hit `target`: the target's body hears of the mover (`PhysicsBody::touched_by`).
#[derive(Debug, Clone, PartialEq)]
pub struct Touch {
    /// The `InstanceID` of the object that was hit.
    pub target: u64,
    /// The mover, as the target's conflict check reads it.
    pub mover: ConflictPeer,
    /// The mover's `ignore_collisions`.
    pub ignores: bool,
    /// The contact normal and the mover's velocity, when `target` was the last object that
    /// stopped the mover.
    pub impact: Option<(Vector, Vector)>,
}

/// The collision traffic a step left for its owner to apply.
#[derive(Debug, Clone, Default)]
pub(crate) struct CollisionTraffic {
    /// The reports of remotes that have a collision manager, for each one's script.
    pub reports: Vec<(u64, CollisionReport)>,
    /// `(owner, other)`: `owner`'s report erased `other`, which should drop `owner`.
    pub drops: Vec<(u64, u64)>,
    /// Touches whose target is not a remote body (the player's).
    pub touches: Vec<Touch>,
}

#[cfg(test)]
mod tests {
    use ac2_core::Random;
    use ac2_physics::{
        BodyShape, CollisionParameters, Frame, PhysicsBody, Position, RecordState, Terrain, Vector,
    };

    use crate::remotes::{Remote, Remotes};

    const CELL: u32 = 0x7F7F_001C;
    const BLOCK: u32 = 0x7F7F_FFFF;
    const WATCHER: u64 = 0x5000_0001;
    const VISITOR: u64 = 0x5000_0002;

    fn remote(managed: bool) -> Remote {
        let shape = BodyShape {
            spheres: vec![ac2_physics::Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4)],
            height: 0.8,
            step_up_height: 0.1,
            step_down_height: 0.1,
            scale: 1.0,
            allow_free_heading: false,
        };
        let at = Position::new(
            CELL,
            Frame::new(
                Vector::new(70.0, 70.0, 50.0),
                ac2_physics::Quaternion::IDENTITY,
            ),
        );
        let mut body = PhysicsBody::new(at, shape, None);
        if managed {
            body.set_instance_id(WATCHER);
            body.set_collision_state(&CollisionParameters::default(), &|_| None);
        }
        Remote {
            visual: None,
            body: Some(body),
            failed: false,
            stored: None,
            in_world: true,
            lost_cell: None,
            active: true,
            player_distance: 0.0,
            object: None,
            shadow: None,
            ethereal: false,
            alpha_moves: None,
            behavior: 0,
            fx_follow: None,
            listed_cell: None,
        }
    }

    #[test]
    fn a_remote_with_a_collision_state_reports_what_touched_it_to_its_script() {
        let mut terrain = Terrain::new();
        terrain.insert_heights(BLOCK, &[50.0; 17 * 17], |_, _| false);
        let mut remotes = Remotes::<()>::default();
        remotes.insert(WATCHER, remote(true));
        let mover = remote(false);
        let peer = ac2_physics::ConflictPeer {
            id: VISITOR,
            ..mover.body().expect("body").conflict_peer()
        };
        remotes
            .body_mut(WATCHER)
            .expect("body")
            .touched_by(&peer, false, None);
        let mut random = Random::default();
        remotes.step_quantum(&mut terrain, 1.0, &mut random, &[]);
        let reports = remotes.take_collision_reports();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].0, WATCHER);
        assert_eq!(reports[0].1.objects.len(), 1);
        assert_eq!(reports[0].1.objects[0].id, VISITOR);
        assert_eq!(reports[0].1.objects[0].state, RecordState::Begin);
        assert!(remotes.take_collision_reports().is_empty());
    }

    #[test]
    fn a_remote_without_a_collision_state_reports_nothing() {
        let mut terrain = Terrain::new();
        terrain.insert_heights(BLOCK, &[50.0; 17 * 17], |_, _| false);
        let mut remotes = Remotes::<()>::default();
        remotes.insert(WATCHER, remote(false));
        let mut random = Random::default();
        remotes.step_quantum(&mut terrain, 1.0, &mut random, &[]);
        assert!(remotes.take_collision_reports().is_empty());
        assert!(remotes.take_collision_drops().is_empty());
        assert!(remotes.take_collision_touches().is_empty());
    }

    #[test]
    fn an_object_destroyed_in_the_world_logs_its_leave_then_its_end() {
        use crate::remotes::TargetLog;
        let mut terrain = Terrain::new();
        let mut remotes = Remotes::<()>::default();
        remotes.insert(WATCHER, remote(false));
        remotes.removed(WATCHER, &mut terrain);
        let log = remotes.take_target_log();
        assert_eq!(log.len(), 2);
        match &log[0] {
            TargetLog::Left(left) => {
                assert_eq!(left.owner.id, WATCHER);
                assert_eq!(left.owner.position.frame.origin.x, 70.0);
            }
            other => panic!("expected the leave, got {other:?}"),
        }
        assert_eq!(log[1], TargetLog::Gone(WATCHER));
        assert!(remotes.take_target_log().is_empty());
    }

    #[test]
    fn an_object_out_of_the_world_logs_no_leave() {
        use crate::remotes::TargetLog;
        let mut terrain = Terrain::new();
        let mut remotes = Remotes::<()>::default();
        let mut out = remote(false);
        out.in_world = false;
        remotes.insert(WATCHER, out);
        remotes.removed(WATCHER, &mut terrain);
        assert_eq!(remotes.take_target_log(), vec![TargetLog::Gone(WATCHER)]);
    }

    #[test]
    fn ending_all_collisions_reports_the_end_of_what_touched_the_object() {
        let mut terrain = Terrain::new();
        terrain.insert_heights(BLOCK, &[50.0; 17 * 17], |_, _| false);
        let mut remotes = Remotes::<()>::default();
        remotes.insert(WATCHER, remote(true));
        let peer = ac2_physics::ConflictPeer {
            id: VISITOR,
            ..remote(false).body().expect("body").conflict_peer()
        };
        remotes
            .body_mut(WATCHER)
            .expect("body")
            .touched_by(&peer, false, None);
        let mut random = Random::default();
        remotes.step_quantum(&mut terrain, 1.0, &mut random, &[]);
        remotes.take_collision_reports();
        // `CPhysicsObj::EndAllCollisions` (the body leaving the world): the visitor's record ends.
        remotes.end_all_collisions(WATCHER, &terrain);
        let reports = remotes.take_collision_reports();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].0, WATCHER);
        assert_eq!(reports[0].1.objects[0].id, VISITOR);
        assert_eq!(reports[0].1.objects[0].state, RecordState::End);
    }
}
