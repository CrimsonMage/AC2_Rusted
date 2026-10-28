//! `CCollisionManager::HasConflict` and `UpdateConflictList`: when the body's collision state
//! changes, each object it is touching is asked again whether the two can pass through one
//! another (`ICPhysicsObj` slot `0x20c`, `CPhysicsObj::Interpermiable`). An object that stops
//! being passable is put on both bodies' temporary ethereal lists, so the two are not stuck inside
//! one another (`ICPhysicsObj` slot `0x204`, `CPhysicsObj::AddToTempEtherealList`).

use crate::collision_types::{CollisionParameters, InstanceId, PhysicsTypeMask};
use crate::physics_obj::PhysicsObj;

/// `CPhysicsObj::ignore_collisions`' state bit.
pub(crate) const IGNORE_COLLISIONS_PS: u32 = 0x8;

/// What the conflict check reads of another object (`ICPhysicsObj::GetObjectA`, then its
/// `GetCollisionStatePtr` and `m_blocking_mask`).
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictPeer {
    pub id: InstanceId,
    pub collision: CollisionParameters,
    pub blocking: PhysicsTypeMask,
}

/// What a conflict asked of the other object: `AddToTempEtherealList(add)` on `object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictEffect {
    pub object: InstanceId,
    pub add: InstanceId,
}

/// `CPhysicsObj::CheckIfSolid`: whether an object with `blocking` and `id` blocks the mover with
/// `info` (its placement or movement set). An object on the mover's temporary ethereal list does
/// not block, and the mover is told it was ignored (`*_is_in_temp_ignore_list`).
fn check_if_solid(
    id: InstanceId,
    blocking: PhysicsTypeMask,
    info: &CollisionParameters,
    placement: bool,
) -> (bool, bool) {
    let mut solid = info.blocks(id, blocking, placement);
    let mut ignored = false;
    if solid && id != 0 && info.in_temp_ethereal_list(id) {
        solid = false;
        ignored = true;
    }
    (solid, ignored)
}

impl PhysicsObj {
    /// `CPhysicsObj::Interpermiable(moving, 0, ...)`: the two pass through one another when this
    /// body does not block the mover's set, or the mover does not block this body's.
    fn interpermiable(&self, moving: &ConflictPeer) -> bool {
        if !check_if_solid(self.instance_id, self.blocking, &moving.collision, false).0 {
            return true;
        }
        !check_if_solid(moving.id, moving.blocking, &self.collision, false).0
    }

    /// `CCollisionManager::HasConflict` for the touched object `id` (`other` is
    /// `ICPhysicsObj::GetObjectA(id)`): passable, the entry becomes ethereal; no longer
    /// passable after having been, the entry turns solid and each body ignores the other from
    /// now on. The effect is for the other body.
    pub(crate) fn has_conflict(
        &mut self,
        id: InstanceId,
        other: Option<&ConflictPeer>,
    ) -> Option<ConflictEffect> {
        let other = other?;
        if id == 0 {
            return None;
        }
        if self.interpermiable(other) {
            self.collision_manager.set_entry_ethereal(id, true);
            return None;
        }
        if self.collision_manager.entry_ethereal(id) == Some(true) {
            self.collision_manager.set_entry_ethereal(id, false);
            self.collision.add_to_temp_ethereal_list(id);
            return Some(ConflictEffect {
                object: id,
                add: self.instance_id,
            });
        }
        None
    }

    /// `CCollisionManager::UpdateConflictList`: the current physics mask gains the desired one
    /// (`UpdateCurrentPhysicsMask`), then every touched object is checked.
    pub(crate) fn update_conflict_list(
        &mut self,
        peer: &dyn Fn(InstanceId) -> Option<ConflictPeer>,
    ) -> Vec<ConflictEffect> {
        let desired = self.desired_collision_mask;
        self.collision.update_current_physics_mask(desired);
        let mut effects = Vec::new();
        for id in self.collision_manager.touched_ids() {
            effects.extend(self.has_conflict(id, peer(id).as_ref()));
        }
        effects
    }

    /// `CCollisionManager::SetCollisionState`: `info` merged in, the desired movement mask
    /// kept when it names one, then `UpdateConflictList`.
    pub(crate) fn set_collision_state(
        &mut self,
        info: &CollisionParameters,
        peer: &dyn Fn(InstanceId) -> Option<ConflictPeer>,
    ) -> Vec<ConflictEffect> {
        self.collision.merge_set_state(info);
        if info.has_movement_physics() {
            self.desired_collision_mask = info.movement_physics();
        }
        self.update_conflict_list(peer)
    }

    /// `CPhysicsObj::AddToTempEtherealList`.
    pub(crate) fn add_to_temp_ethereal_list(&mut self, id: InstanceId) -> bool {
        self.collision.add_to_temp_ethereal_list(id)
    }

    /// What other bodies' conflict checks read of this one.
    pub(crate) fn conflict_peer(&self) -> ConflictPeer {
        ConflictPeer {
            id: self.instance_id,
            collision: self.collision.clone(),
            blocking: self.blocking,
        }
    }
}

#[cfg(test)]
#[path = "physics_obj_conflict_tests.rs"]
mod tests;
