//! What the remote bodies ask of the target managers and of the collision reports when they
//! leave the world or are destroyed: `CPhysicsObj::HandleLeaveWorld`'s `TargetManager` and
//! `EndAllCollisions` calls, and a missile's `CPhysicsObj::SetTarget`. The client owns the
//! [`crate::TargetHub`] and applies the log in the order it was made.

use ac2_physics::CollisionReport;

use super::*;

/// One request for the target hub, in the order the remotes made it.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetLog {
    /// The object left the world (`CPhysicsObj::HandleLeaveWorld`): its manager lets go of its
    /// target and tells its voyeurs `ExitWorld`. The body is as it left.
    Left(LeftBody),
    /// The object is destroyed (`CPhysicsObj::~CPhysicsObj`): its manager goes with it.
    Gone(u64),
    /// `CPhysicsObj::SetTarget(0, object_id, radius, quantum)` on `owner` (a missile's target).
    SetTarget {
        owner: u64,
        object_id: u64,
        radius: f32,
        quantum: f64,
    },
}

/// A body as it left the world: what `TargetManager` reads of it, with the size left for
/// [`Remotes::left_owner`] to fill in from the Setup (the object may be gone by then).
#[derive(Debug, Clone, PartialEq)]
pub struct LeftBody {
    pub owner: TargetOwner,
    pub visual: Option<u32>,
}

impl<B> Remotes<B> {
    /// `HandleLeaveWorld`'s record for an object that is still in the world: called before its
    /// flag goes down.
    pub(crate) fn note_left(&mut self, id: u64) {
        let Some(remote) = self.objects.get(&id) else {
            return;
        };
        let Some(body) = remote.body.as_ref() else {
            return;
        };
        let v = body.movement_state().velocity;
        let scale = remote
            .object
            .as_ref()
            .map_or(Vector::new(1.0, 1.0, 1.0), |(_, s)| *s);
        let owner = TargetOwner {
            id,
            position: body.position(),
            velocity: Vector::new(v[0], v[1], v[2]),
            radius: 0.0,
            height: 0.0,
            scale,
        };
        let visual = remote.visual;
        self.target_log
            .push(TargetLog::Left(LeftBody { owner, visual }));
    }
}

impl<B> Remotes<B> {
    /// The target hub requests made since the last call.
    pub fn take_target_log(&mut self) -> Vec<TargetLog> {
        std::mem::take(&mut self.target_log)
    }

    /// `CPhysicsObj::EndAllCollisions` for an object that left the world: every record it
    /// reported ends (a missile's `CMissileManager::HandleCollision` on that all-`End` report
    /// detonates nothing, so it is not run). A body with a collision manager hands the report to its script
    /// (`take_collision_reports`).
    pub fn end_all_collisions(&mut self, id: u64, terrain: &Terrain) {
        let Some(body) = self.objects.get_mut(&id).and_then(|r| r.body.as_mut()) else {
            return;
        };
        let manager = body.has_collision_manager();
        let materials = body.contact_materials(terrain);
        let report: Option<CollisionReport> = body.end_all_collisions(&materials);
        if let (Some(report), true) = (report, manager) {
            self.traffic.reports.push((id, report));
        }
        // The objects the report erased drop this one too (`CollisionRemoveObject`).
        let dropped = body.take_dropped();
        for other in dropped {
            match self.body_mut(other) {
                Some(body) => {
                    if let Some(report) = body.collision_remove_object(id) {
                        self.traffic.reports.push((other, report));
                    }
                }
                None => self.traffic.drops.push((id, other)),
            }
        }
    }
}

impl<B: AsRef<BodyData>> Remotes<B> {
    /// The owner record of a body that left, with `GetRadius` and `GetHeight` of its Setup.
    pub fn left_owner(&self, left: &LeftBody) -> TargetOwner {
        let mut owner = left.owner;
        if let Some((radius, height)) = self.sizes_of_visual(left.visual) {
            owner.radius = radius;
            owner.height = height;
        }
        owner
    }

    /// What `TargetManager` reads of the object's body, in the world or not (the body as it
    /// leaves the world still has its position): `None` without a body.
    pub fn target_owner_any(&self, id: u64) -> Option<TargetOwner> {
        let remote = self.objects.get(&id)?;
        let body = remote.body.as_ref()?;
        let (radius, height) = self.sizes_of(remote)?;
        let v = body.movement_state().velocity;
        let scale = remote
            .object
            .as_ref()
            .map_or(Vector::new(1.0, 1.0, 1.0), |(_, s)| *s);
        Some(TargetOwner {
            id,
            position: body.position(),
            velocity: Vector::new(v[0], v[1], v[2]),
            radius,
            height,
            scale,
        })
    }
}
