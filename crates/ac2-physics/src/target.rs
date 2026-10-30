//! `TargetManager` (`portal/shared/src/physobj/target/targetmanager.cpp`): a body names one
//! object as its target and asks it for position updates (a *voyeur* of the target); each body
//! in turn keeps the voyeurs watching it and tells them when it has moved by more than the
//! radius they asked for. The manager holds only its own state: whatever retail did to another
//! `CPhysicsObj` (`GetObjectA`, `AddVoyeur`, `RemoveVoyeur`, `ReceiveTargetUpdate`) or to the
//! notice handler (`RecvNotice_CharacterSet`) comes back as a [`TargetEffect`] for the owner of
//! the object table to apply.

use std::collections::BTreeMap;

use crate::fpu::{ld, st, F_EPSILON};
use crate::position::Position;
use crate::vector::Vector;

/// `TargetStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetStatus {
    #[default]
    Undef = 0,
    Ok = 1,
    ExitWorld = 2,
    Teleported = 3,
    Contained = 4,
    Parented = 5,
    TimedOut = 6,
}

/// `TargetInfo`, the record a target sends its voyeurs and the manager keeps for its own
/// target. `TargetInfo::TargetInfo` makes it `INVALID_IID`, zeroed, `Undef`, scale zero.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TargetInfo {
    pub context_id: u32,
    pub object_id: u64,
    pub radius: f32,
    pub quantum: f64,
    pub target_position: Position,
    pub interpolated_position: Position,
    pub interpolated_heading: Vector,
    pub velocity: Vector,
    pub status: TargetStatus,
    pub last_update_time: f64,
    pub object_radius: f32,
    pub object_height: f32,
    pub scale: Vector,
}

/// `TargetVoyeur`: an object watching this one.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetVoyeur {
    pub object_id: u64,
    pub quantum: f64,
    pub radius: f32,
    pub last_sent_position: Position,
}

/// What the manager reads of the body it belongs to (`physobj`: its id, `GetPosition`, the
/// velocity, `GetRadius`, `GetHeight` and the scale vector).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetOwner {
    pub id: u64,
    pub position: Position,
    pub velocity: Vector,
    pub radius: f32,
    pub height: f32,
    pub scale: Vector,
}

/// What retail did to something outside the manager.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetEffect {
    /// `CPhysicsObj::RemoveVoyeur(voyeur)` on `target`.
    RemoveVoyeur { target: u64, voyeur: u64 },
    /// `CPhysicsObj::AddVoyeur(voyeur, radius, quantum)` on `target`.
    AddVoyeur {
        target: u64,
        voyeur: u64,
        radius: f32,
        quantum: f64,
    },
    /// `CPhysicsObj::ReceiveTargetUpdate(info)` on `to`.
    Update { to: u64, info: Box<TargetInfo> },
    /// `NoticeHandler::RecvNotice_CharacterSet(physobj, target_info)`: the body's own target
    /// changed state (`weenie` target callback).
    Notice(Box<TargetInfo>),
}

/// A `CPhysicsObj::SetTarget(0, object_id, radius, quantum)` a body asked for: it is the owner
/// of the target managers that applies it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetRequest {
    pub object_id: u64,
    pub radius: f32,
    pub quantum: f64,
}

/// `TargetManager`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TargetManager {
    target_info: Option<TargetInfo>,
    voyeurs: BTreeMap<u64, TargetVoyeur>,
    last_update_time: f64,
}

impl TargetManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// `CPhysicsObj::GetTargetInfo`: the record of the current target.
    pub fn target_info(&self) -> Option<&TargetInfo> {
        self.target_info.as_ref()
    }

    /// The voyeurs watching the body.
    pub fn voyeurs(&self) -> impl Iterator<Item = &TargetVoyeur> {
        self.voyeurs.values()
    }

    /// `TargetManager::GetInterpolatedPosition`: where the body will be `quantum` seconds on,
    /// its position moved by its velocity.
    pub fn interpolated_position(owner: &TargetOwner, quantum: f64) -> Position {
        let mut p = owner.position;
        let v = owner.velocity;
        p.frame.origin = Vector::new(
            st(ld(v.x) * quantum + ld(p.frame.origin.x)),
            st(ld(v.y) * quantum + ld(p.frame.origin.y)),
            st(ld(v.z) * quantum + ld(p.frame.origin.z)),
        );
        p
    }

    /// `TargetManager::ClearTarget`: lets go of the current target (it stops watching us).
    pub fn clear_target(&mut self, owner: u64) -> Vec<TargetEffect> {
        let mut effects = Vec::new();
        if let Some(info) = self.target_info.take() {
            effects.push(TargetEffect::RemoveVoyeur {
                target: info.object_id,
                voyeur: owner,
            });
        }
        effects
    }

    /// `TargetManager::SetTargetQuantum`: a new quantum, passed on to the target as a fresh
    /// `AddVoyeur`.
    pub fn set_target_quantum(&mut self, owner: u64, quantum: f64) -> Vec<TargetEffect> {
        let Some(info) = self.target_info.as_mut() else {
            return Vec::new();
        };
        info.quantum = quantum;
        vec![TargetEffect::AddVoyeur {
            target: info.object_id,
            voyeur: owner,
            radius: info.radius,
            quantum,
        }]
    }

    /// `TargetManager::SetTarget`: drops the old target, then watches `object_id` (0 is
    /// `INVALID_IID`: a `TimedOut` notice for `context_id` and no target). `now` is
    /// `Timer::cur_time`.
    pub fn set_target(
        &mut self,
        owner: u64,
        context_id: u32,
        object_id: u64,
        radius: f32,
        quantum: f64,
        now: f64,
    ) -> Vec<TargetEffect> {
        let mut effects = self.clear_target(owner);
        if object_id == 0 {
            let failed = TargetInfo {
                context_id,
                status: TargetStatus::TimedOut,
                ..TargetInfo::default()
            };
            effects.push(TargetEffect::Notice(Box::new(failed)));
            return effects;
        }
        self.target_info = Some(TargetInfo {
            context_id,
            object_id,
            radius,
            quantum,
            last_update_time: now,
            ..TargetInfo::default()
        });
        effects.push(TargetEffect::AddVoyeur {
            target: object_id,
            voyeur: owner,
            radius,
            quantum,
        });
        effects
    }

    /// `TargetManager::ReceiveUpdate`: an update about the current target replaces the record
    /// (only when it names that target), stamps it with `now`, aims `interpolated_heading` from
    /// the body to the target's interpolated position (unit length, straight up when nearer
    /// than `F_EPSILON`) and tells the notice handler.
    pub fn receive_update(
        &mut self,
        owner: &TargetOwner,
        update: &TargetInfo,
        now: f64,
    ) -> Vec<TargetEffect> {
        let Some(info) = self.target_info.as_mut() else {
            return Vec::new();
        };
        if update.object_id != info.object_id {
            return Vec::new();
        }
        info.object_id = update.object_id;
        info.radius = update.radius;
        info.quantum = update.quantum;
        info.target_position = update.target_position;
        info.interpolated_position = update.interpolated_position;
        info.velocity = update.velocity;
        info.status = update.status;
        info.last_update_time = now;
        info.object_radius = update.object_radius;
        info.object_height = update.object_height;
        info.scale = update.scale;
        let o = owner.position.offset_to(&update.interpolated_position);
        let length = (ld(o.z) * ld(o.z) + ld(o.y) * ld(o.y) + ld(o.x) * ld(o.x)).sqrt();
        info.interpolated_heading = if length < ld(F_EPSILON) {
            Vector::new(0.0, 0.0, 1.0)
        } else {
            let inverse = 1.0 / length;
            Vector::new(
                st(inverse * ld(o.x)),
                st(inverse * ld(o.y)),
                st(inverse * ld(o.z)),
            )
        };
        vec![TargetEffect::Notice(Box::new(info.clone()))]
    }

    /// `TargetManager::SendVoyeurUpdate`: records `p` as the voyeur's last sent position and
    /// builds the `TargetInfo` the voyeur receives.
    fn send_voyeur_update(
        owner: &TargetOwner,
        voyeur: &mut TargetVoyeur,
        p: Position,
        status: TargetStatus,
    ) -> TargetEffect {
        voyeur.last_sent_position = p;
        let info = TargetInfo {
            context_id: 0,
            object_id: owner.id,
            radius: voyeur.radius,
            quantum: voyeur.quantum,
            target_position: owner.position,
            interpolated_position: p,
            velocity: owner.velocity,
            status,
            object_radius: owner.radius,
            object_height: owner.height,
            scale: owner.scale,
            ..TargetInfo::default()
        };
        TargetEffect::Update {
            to: voyeur.object_id,
            info: Box::new(info),
        }
    }

    /// `TargetManager::CheckAndUpdateVoyeur`: an update (`Ok`) once the body's interpolated
    /// position is farther than the voyeur's radius from the last one sent.
    fn check_and_update_voyeur(
        owner: &TargetOwner,
        voyeur: &mut TargetVoyeur,
    ) -> Option<TargetEffect> {
        let p = Self::interpolated_position(owner, voyeur.quantum);
        let distance = p.distance(&voyeur.last_sent_position);
        (ld(voyeur.radius) < distance)
            .then(|| Self::send_voyeur_update(owner, voyeur, p, TargetStatus::Ok))
    }

    /// `TargetManager::HandleTargetting`: at most every half second, times out a target that
    /// has said nothing for ten seconds and updates the voyeurs that need it.
    pub fn handle_targetting(&mut self, owner: &TargetOwner, now: f64) -> Vec<TargetEffect> {
        let mut effects = Vec::new();
        if now - self.last_update_time < 0.5 {
            return effects;
        }
        if let Some(info) = self.target_info.as_mut() {
            if info.status == TargetStatus::Undef && info.last_update_time + 10.0 < now {
                info.status = TargetStatus::TimedOut;
                effects.push(TargetEffect::Notice(Box::new(info.clone())));
            }
        }
        for voyeur in self.voyeurs.values_mut() {
            effects.extend(Self::check_and_update_voyeur(owner, voyeur));
        }
        self.last_update_time = now;
        effects
    }

    /// `TargetManager::NotifyVoyeurOfEvent`: every voyeur is told `event` at the body's
    /// position.
    pub fn notify_voyeurs_of_event(
        &mut self,
        owner: &TargetOwner,
        event: TargetStatus,
    ) -> Vec<TargetEffect> {
        self.voyeurs
            .values_mut()
            .map(|voyeur| Self::send_voyeur_update(owner, voyeur, owner.position, event))
            .collect()
    }

    /// `TargetManager::HandleLeaveWorld`: lets go of the target and tells the voyeurs
    /// `ExitWorld`.
    pub fn handle_leave_world(&mut self, owner: &TargetOwner) -> Vec<TargetEffect> {
        let mut effects = self.clear_target(owner.id);
        effects.extend(self.notify_voyeurs_of_event(owner, TargetStatus::ExitWorld));
        effects
    }

    /// `TargetManager::RemoveVoyeur`: true when the object was a voyeur.
    pub fn remove_voyeur(&mut self, object_id: u64) -> bool {
        self.voyeurs.remove(&object_id).is_some()
    }

    /// `TargetManager::AddVoyeur`: an existing voyeur gets the new radius and quantum; a new
    /// one is told the body's position at once (`Ok`).
    pub fn add_voyeur(
        &mut self,
        owner: &TargetOwner,
        object_id: u64,
        radius: f32,
        quantum: f64,
    ) -> Vec<TargetEffect> {
        if let Some(voyeur) = self.voyeurs.get_mut(&object_id) {
            voyeur.radius = radius;
            voyeur.quantum = quantum;
            return Vec::new();
        }
        let mut voyeur = TargetVoyeur {
            object_id,
            quantum,
            radius,
            last_sent_position: Position::default(),
        };
        let effect = Self::send_voyeur_update(owner, &mut voyeur, owner.position, TargetStatus::Ok);
        self.voyeurs.insert(object_id, voyeur);
        vec![effect]
    }
}

#[cfg(test)]
#[path = "target_tests.rs"]
mod tests;
