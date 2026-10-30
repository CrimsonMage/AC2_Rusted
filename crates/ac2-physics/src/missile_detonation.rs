//! `CMissileManager`'s collision side: whether a collision profile detonates the missile
//! (`ShouldDetonate`), what detonation does (`HandleCollision`), detonation by proximity
//! (`DetonatesOnProximityTo`) and the direction the missile would be fired in
//! (`InquireDirection`).

use crate::collision_report::{CollisionReport, RecordState};
use crate::heading::Heading;
use crate::missile::{MissileHost, MissileManager, MISSILE_FLIGHT_FX};
use crate::missile_flight::MissileFlightHost;

/// `MissileParameterInfo::DETONATES_ON_PROXIMITY`.
const DETONATES_ON_PROXIMITY: u32 = 0x2_0000;

/// What `ShouldDetonate` and `HandleCollision` read of an object the missile hit
/// (`ICPhysicsObj::GetObjectA`, `GetWeenieObj`, `is_dynamic`, `HasMissileManager`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImpactTarget {
    pub id: u64,
    /// `ICPhysicsObj` slot `0x230`, `CPhysicsObj::is_dynamic`.
    pub dynamic: bool,
    /// `ICPhysicsObj` slot `0x520`, `CPhysicsObj::HasMissileManager`.
    pub has_missile_manager: bool,
    /// The results of the object's weenie slots `0x74` and `0x94` (`None` without a weenie).
    /// Both are pure virtuals of `CWeenieObject` whose implementation is not in the client
    /// image, so what they ask is not named.
    pub weenie: Option<(bool, bool)>,
}

/// The object table as the detonation code reads it.
pub trait ImpactLookup {
    fn target(&self, id: u64) -> Option<ImpactTarget>;
}

/// The body's side of a detonation: the missile object is deleted (or sticks to what it hit).
pub trait MissileCollisionHost: MissileHost {
    /// `CPhysicsObj::DeleteSelf(fade, delay)` (slot `0x620`).
    fn delete_self(&mut self, fade: f64, delay: f64);
    /// `CPhysicsObj::StickySetParent(object)` (slot `0x3f0`).
    fn sticky_set_parent(&mut self, object: u64);
}

/// What `ShouldDetonate` decided: the object that stopped the missile, or none when it was the
/// world (the impact).
pub type Impact = Option<ImpactTarget>;

impl MissileManager {
    /// `CMissileManager::ShouldDetonate` over a collision report (`CollisionProfile`): the first
    /// object that began colliding and is solid detonates the missile. An ethereal one does when
    /// it is the target (not the source) unless the missile detonates on miss, when it does when
    /// its weenie answers either slot `0x74` or `0x94`. Otherwise an impact that is not an end
    /// detonates it on the world.
    pub fn should_detonate(
        &self,
        report: &CollisionReport,
        objects: &dyn ImpactLookup,
    ) -> Option<Impact> {
        for record in &report.objects {
            if record.state != RecordState::Begin {
                continue;
            }
            let found = objects.target(record.id);
            if !record.ethereal {
                return Some(found);
            }
            if record.id == self.target_info.source_id {
                continue;
            }
            if !self.info.detonates_on_miss {
                if record.id == self.target_info.target_id {
                    return Some(found);
                }
            } else if let Some(target) = found {
                if let Some((a, b)) = target.weenie {
                    if a || b {
                        return Some(Some(target));
                    }
                }
            }
        }
        // `IsEndCollision` is never true for a report's impact, which lists only the begin;
        // `HasCollided` is whether the report has one.
        report.impact.is_some().then_some(None)
    }

    /// `CMissileManager::HandleCollision`: a detonation stops the missile (no more updates, no
    /// flight FX or streak). A missile that does not stick is deleted after fading half a
    /// second; one that does stick to a dynamic object that is not a missile itself sticks to it.
    pub fn handle_collision(
        &mut self,
        host: &mut impl MissileCollisionHost,
        report: &CollisionReport,
        objects: &dyn ImpactLookup,
    ) {
        let Some(impact) = self.should_detonate(report, objects) else {
            return;
        };
        self.is_activated = false;
        self.is_moving = false;
        host.stop_fx(MISSILE_FLIGHT_FX);
        host.stop_streak();
        if !self.info.sticks_upon_collide {
            host.delete_self(0.5, 0.0);
        } else if let Some(target) = impact {
            if target.dynamic && !target.has_missile_manager {
                host.sticky_set_parent(target.id);
            }
        }
    }

    /// `CMissileManager::DetonatesOnProximityTo`: the missile detonates on proximity, the target
    /// asked for a chance within the missile's, and `object` is the target.
    pub fn detonates_on_proximity_to(&self, object: u64) -> bool {
        self.proximity_target()
            .is_some_and(|(target, _)| target == object)
    }

    /// The one object `detonates_on_proximity_to` answers for, and the missile's
    /// `m_detonation_proximity`: what `CPhysicsObj::FindObjCollisions` tests each obstacle against.
    pub fn proximity_target(&self) -> Option<(u64, f32)> {
        (self.info.bitfield & DETONATES_ON_PROXIMITY != 0
            && self.target_info.missile_detonation_chance <= self.info.detonation_chance)
            .then_some((self.target_info.target_id, self.info.detonation_proximity))
    }

    /// `CMissileManager::InquireDirection`: the heading and pitch of the missile's initial
    /// velocity from the body's position, and `CalcInitialVelocity`'s error. Retail computes
    /// both and then stores heading zero and pitch zero over them on every path (the stores
    /// after the `if` fall through), so the pair it returns is always (0, 0).
    pub fn inquire_direction(
        &self,
        host: &impl MissileFlightHost,
        now: f64,
        coin: impl FnMut(f32) -> bool,
    ) -> (Heading, f32, u32) {
        let start = host.position();
        let error = self
            .calc_initial_velocity(host, &start, now, coin)
            .err()
            .unwrap_or(0);
        (Heading::ZERO, 0.0, error)
    }
}

#[cfg(test)]
#[path = "missile_detonation_tests.rs"]
mod tests;
