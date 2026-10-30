//! The `TargetManager` of every object, and the routing of what the managers ask of one another.
//!
//! `ac2_physics::TargetManager` holds only its own body's state; what retail did to another
//! `CPhysicsObj` it returns as a [`TargetEffect`]. Here the object table applies them:
//! `CPhysicsObj::AddVoyeur`, `RemoveVoyeur` and `ReceiveTargetUpdate` reach the named object's
//! manager (made on first use, as `CPhysicsObj` makes its `target_manager`), and an object the
//! table does not have is ignored (`CPhysicsObj::GetObjectA` returns null). A notice for the
//! body's own target (`NoticeHandler::RecvNotice_CharacterSet`) is handed back to the caller.

use std::collections::{BTreeMap, VecDeque};

use ac2_physics::{TargetEffect, TargetInfo, TargetManager, TargetOwner};

/// What the table knows of an object's body: `None` when it is not in the world.
pub type TargetOwners<'a> = &'a dyn Fn(u64) -> Option<TargetOwner>;

/// A notice to the weenie side: `object`'s own target changed (a context id, status and the
/// last known position).
#[derive(Debug, Clone, PartialEq)]
pub struct TargetNotice {
    pub object: u64,
    pub info: Box<TargetInfo>,
}

/// Every object's `CPhysicsObj::target_manager`.
#[derive(Debug, Default)]
pub struct TargetHub {
    managers: BTreeMap<u64, TargetManager>,
}

impl TargetHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// The manager of `object`, when it has one.
    pub fn manager(&self, object: u64) -> Option<&TargetManager> {
        self.managers.get(&object)
    }

    /// The object is destroyed: its manager goes with it.
    pub fn remove_object(&mut self, object: u64) {
        self.managers.remove(&object);
    }

    /// `CPhysicsObj::GetTargetID`: the id of `object`'s target (0, `INVALID_IID`, when it has no
    /// manager or no target).
    pub fn target_id(&self, object: u64) -> u64 {
        self.target_info(object).map_or(0, |info| info.object_id)
    }

    /// `CPhysicsObj::GetTargetQuantum`: the quantum `object` asked its target for (0 without one).
    pub fn target_quantum(&self, object: u64) -> f64 {
        self.target_info(object).map_or(0.0, |info| info.quantum)
    }

    /// `CPhysicsObj::GetTargetInfo`: a copy of `object`'s target record, when it has one.
    pub fn target_info(&self, object: u64) -> Option<&TargetInfo> {
        self.managers.get(&object)?.target_info()
    }

    /// `CPhysicsObj::AddVoyeur`: `voyeur` starts watching `object` (its manager is made on first
    /// use); a new voyeur is told the object's position at once.
    pub fn add_voyeur(
        &mut self,
        object: u64,
        voyeur: u64,
        radius: f32,
        quantum: f64,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let Some(body) = owners(object) else {
            return Vec::new();
        };
        let effects = self
            .managers
            .entry(object)
            .or_default()
            .add_voyeur(&body, voyeur, radius, quantum);
        self.apply(object, effects, now, owners)
    }

    /// `CPhysicsObj::RemoveVoyeur`: true when `voyeur` was watching `object`.
    pub fn remove_voyeur(&mut self, object: u64, voyeur: u64) -> bool {
        self.managers
            .get_mut(&object)
            .is_some_and(|manager| manager.remove_voyeur(voyeur))
    }

    /// `CPhysicsObj::ReceiveTargetUpdate`: `object`'s target sent `info` (ignored when `object`
    /// has no manager).
    pub fn receive_target_update(
        &mut self,
        object: u64,
        info: &TargetInfo,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let Some(body) = owners(object) else {
            return Vec::new();
        };
        let Some(manager) = self.managers.get_mut(&object) else {
            return Vec::new();
        };
        let effects = manager.receive_update(&body, info, now);
        self.apply(object, effects, now, owners)
    }

    /// `CPhysicsObj::SetTarget`.
    #[allow(clippy::too_many_arguments)]
    pub fn set_target(
        &mut self,
        owner: u64,
        context_id: u32,
        object_id: u64,
        radius: f32,
        quantum: f64,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let effects = self
            .managers
            .entry(owner)
            .or_default()
            .set_target(owner, context_id, object_id, radius, quantum, now);
        self.apply(owner, effects, now, owners)
    }

    /// `CPhysicsObj::ClearTarget`.
    pub fn clear_target(
        &mut self,
        owner: u64,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let Some(manager) = self.managers.get_mut(&owner) else {
            return Vec::new();
        };
        let effects = manager.clear_target(owner);
        self.apply(owner, effects, now, owners)
    }

    /// `CPhysicsObj::SetTargetQuantum`.
    pub fn set_target_quantum(
        &mut self,
        owner: u64,
        quantum: f64,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let Some(manager) = self.managers.get_mut(&owner) else {
            return Vec::new();
        };
        let effects = manager.set_target_quantum(owner, quantum);
        self.apply(owner, effects, now, owners)
    }

    /// `CPhysicsObj::HandleLeaveWorld`'s target part: the object lets go of its target and its
    /// voyeurs are told `ExitWorld`.
    pub fn leave_world(
        &mut self,
        owner: u64,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let Some(body) = owners(owner) else {
            return Vec::new();
        };
        let Some(manager) = self.managers.get_mut(&owner) else {
            return Vec::new();
        };
        let effects = manager.handle_leave_world(&body);
        self.apply(owner, effects, now, owners)
    }

    /// `CPhysicsObj::HandleTargetting` for every object in the world: each manager checks its
    /// target's timeout and updates its voyeurs.
    pub fn handle_targetting(&mut self, now: f64, owners: TargetOwners<'_>) -> Vec<TargetNotice> {
        let ids: Vec<u64> = self.managers.keys().copied().collect();
        let mut notices = Vec::new();
        for id in ids {
            let Some(body) = owners(id) else { continue };
            let Some(manager) = self.managers.get_mut(&id) else {
                continue;
            };
            let effects = manager.handle_targetting(&body, now);
            notices.extend(self.apply(id, effects, now, owners));
        }
        notices
    }

    /// Applies `effects` the manager of `from` asked for, and whatever those cause in turn.
    fn apply(
        &mut self,
        from: u64,
        effects: Vec<TargetEffect>,
        now: f64,
        owners: TargetOwners<'_>,
    ) -> Vec<TargetNotice> {
        let mut notices = Vec::new();
        let mut queue: VecDeque<(u64, TargetEffect)> =
            effects.into_iter().map(|e| (from, e)).collect();
        while let Some((source, effect)) = queue.pop_front() {
            match effect {
                TargetEffect::Notice(info) => notices.push(TargetNotice {
                    object: source,
                    info,
                }),
                TargetEffect::RemoveVoyeur { target, voyeur } => {
                    if owners(target).is_some() {
                        if let Some(manager) = self.managers.get_mut(&target) {
                            manager.remove_voyeur(voyeur);
                        }
                    }
                }
                TargetEffect::AddVoyeur {
                    target,
                    voyeur,
                    radius,
                    quantum,
                } => {
                    let Some(body) = owners(target) else { continue };
                    let manager = self.managers.entry(target).or_default();
                    let more = manager.add_voyeur(&body, voyeur, radius, quantum);
                    queue.extend(more.into_iter().map(|e| (target, e)));
                }
                TargetEffect::Update { to, info } => {
                    let Some(body) = owners(to) else { continue };
                    let manager = self.managers.entry(to).or_default();
                    let more = manager.receive_update(&body, &info, now);
                    queue.extend(more.into_iter().map(|e| (to, e)));
                }
            }
        }
        notices
    }
}

#[cfg(test)]
#[path = "target_hub_tests.rs"]
mod tests;
