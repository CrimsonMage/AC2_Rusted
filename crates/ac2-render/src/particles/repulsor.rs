//! Particle repulsors (`ParticleRepulsor`, `PSManager::CreateRepulsor` / `DestroyRepulsor` and the
//! manager's `s_Repulsors` list): a field an object's FX leaves on it that pushes the particles of
//! repulse-controlled emitters away (or pulls them in, for an attractor) inside its range.
//! `ParticleEmitter::Think` takes the repulsors placed this scene stamp or the one before
//! (`sceneTimestamp - m_nTimestamp < 2`) that are active, and `ParticleEmitter::DoRepulse` applies
//! them ([`super::controllers`]).

use ac2_math::Vec3;

/// What `RepulsorFX` gives `PSManager::CreateRepulsor`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepulsorSpec {
    /// `m_fAcceleration`: the push at the repulsor's centre.
    pub acceleration: f32,
    /// `m_fRange`: the push falls off linearly to nothing at this distance.
    pub range: f32,
    /// `m_fLifetime`: seconds the repulsor lives; under `F_EPSILON` it lives until it is destroyed.
    pub lifetime: f32,
    /// `m_fMinimumVelocity`: the speed its object must have for the repulsor to act (under
    /// `F_EPSILON`: none).
    pub minimum_velocity: f32,
    /// `m_bAttractor`: pulls instead of pushing.
    pub attractor: bool,
}

/// `F_EPSILON`.
const EPSILON: f32 = 0.0002;

/// One repulsor as `ParticleEmitter::Think` collects it for `DoRepulse`: where it is, its range
/// and its acceleration (`ParticleRepulsor` offsets 8, 0x64, 0x60 and 0x6c).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearbyRepulsor {
    pub position: Vec3,
    pub range: f32,
    pub acceleration: f32,
    pub attractor: bool,
}

/// `ParticleRepulsor`: the spec, the time it was made, and where its object last drew it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleRepulsor {
    spec: RepulsorSpec,
    /// `m_tTimeCreated`.
    created: f64,
    /// `m_pos`'s origin, in scene axes.
    position: Vec3,
    /// `m_fSpeed`: the object's speed when it last drew.
    speed: f32,
    /// `m_nTimestamp`: the scene stamp of that draw; `None` before the first one.
    stamp: Option<u64>,
}

impl ParticleRepulsor {
    /// `ParticleRepulsor::ParticleRepulsor` with `PSManager::CreateRepulsor`'s values, at client
    /// time `now`.
    pub fn new(spec: RepulsorSpec, now: f64) -> Self {
        Self {
            spec,
            created: now,
            position: Vec3::ZERO,
            speed: 0.0,
            stamp: None,
        }
    }

    /// `ParticleRepulsor::GetIsReadyToDie`: a repulsor with a lifetime has outlived it.
    pub fn is_ready_to_die(&self, now: f64) -> bool {
        self.spec.lifetime >= EPSILON && f64::from(self.spec.lifetime) < now - self.created
    }

    /// `ParticleRepulsor::GetIsActive`: its object is at least as fast as the minimum velocity
    /// (when there is one) and it is within its lifetime (when it has one).
    pub fn is_active(&self, now: f64) -> bool {
        (self.spec.minimum_velocity <= EPSILON || self.spec.minimum_velocity <= self.speed)
            && (self.spec.lifetime < EPSILON || now - self.created < f64::from(self.spec.lifetime))
    }

    /// `PHYSICSOBJREP::Draw`'s update of a repulsor that is not ready to die:
    /// `ParticleRepulsor::SetPosition`, the scene stamp and the object's speed.
    pub fn place(&mut self, position: Vec3, stamp: u64, speed: f32) {
        self.position = position;
        self.stamp = Some(stamp);
        self.speed = speed;
    }

    /// Where the object last drew it.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    fn nearby(&self) -> NearbyRepulsor {
        NearbyRepulsor {
            position: self.position,
            range: self.spec.range,
            acceleration: self.spec.acceleration,
            attractor: self.spec.attractor,
        }
    }
}

/// `PSManager::s_Repulsors`: the repulsors in the world, by the handle `CreateRepulsor` gave, and
/// the scene stamp they are placed under.
#[derive(Debug, Clone, Default)]
pub struct Repulsors {
    list: Vec<(u64, ParticleRepulsor)>,
    next: u64,
    /// `SceneTool::sceneTimestamp`: one more for each frame's placement.
    stamp: u64,
}

impl Repulsors {
    /// `PSManager::CreateRepulsor`: a repulsor made at client time `now`; its handle.
    pub fn create(&mut self, spec: RepulsorSpec, now: f64) -> u64 {
        self.next += 1;
        self.list
            .push((self.next, ParticleRepulsor::new(spec, now)));
        self.next
    }

    /// `PSManager::DestroyRepulsor`.
    pub fn destroy(&mut self, handle: u64) {
        self.list.retain(|(id, _)| *id != handle);
    }

    /// A repulsor by handle.
    pub fn get(&self, handle: u64) -> Option<&ParticleRepulsor> {
        self.list
            .iter()
            .find(|(id, _)| *id == handle)
            .map(|(_, repulsor)| repulsor)
    }

    /// A repulsor by handle, to place it.
    pub fn get_mut(&mut self, handle: u64) -> Option<&mut ParticleRepulsor> {
        self.list
            .iter_mut()
            .find(|(id, _)| *id == handle)
            .map(|(_, repulsor)| repulsor)
    }

    /// The scene stamp advances for a new frame's placements.
    pub fn next_stamp(&mut self) -> u64 {
        self.stamp += 1;
        self.stamp
    }

    /// The scene stamp repulsors are placed under now.
    pub fn stamp(&self) -> u64 {
        self.stamp
    }

    /// The repulsors `ParticleEmitter::Think` hands `DoRepulse` at client time `now`: placed this
    /// stamp or the last, and active.
    pub fn nearby(&self, now: f64) -> Vec<NearbyRepulsor> {
        self.list
            .iter()
            .filter(|(_, r)| {
                r.stamp.is_some_and(|s| self.stamp.wrapping_sub(s) < 2) && r.is_active(now)
            })
            .map(|(_, r)| r.nearby())
            .collect()
    }

    /// How many repulsors exist.
    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(lifetime: f32, minimum_velocity: f32) -> RepulsorSpec {
        RepulsorSpec {
            acceleration: 10.0,
            range: 5.0,
            lifetime,
            minimum_velocity,
            attractor: false,
        }
    }

    #[test]
    fn a_lifetime_ends_the_repulsor_and_zero_lives_on() {
        let mut list = Repulsors::default();
        let timed = list.create(spec(2.0, 0.0), 10.0);
        let endless = list.create(spec(0.0, 0.0), 10.0);
        let stamp = list.next_stamp();
        for handle in [timed, endless] {
            list.get_mut(handle).unwrap().place(Vec3::ZERO, stamp, 0.0);
        }
        assert_eq!(list.nearby(11.0).len(), 2);
        assert_eq!(
            list.nearby(12.5).len(),
            1,
            "the timed one is no longer active"
        );
        assert!(list.get(timed).unwrap().is_ready_to_die(12.5));
        assert!(!list.get(endless).unwrap().is_ready_to_die(1.0e6));
    }

    #[test]
    fn the_minimum_velocity_gates_the_repulsor_on_its_objects_speed() {
        let mut list = Repulsors::default();
        let handle = list.create(spec(0.0, 3.0), 0.0);
        let stamp = list.next_stamp();
        list.get_mut(handle).unwrap().place(Vec3::ZERO, stamp, 2.0);
        assert!(list.nearby(1.0).is_empty());
        list.get_mut(handle).unwrap().place(Vec3::ZERO, stamp, 3.0);
        assert_eq!(list.nearby(1.0).len(), 1);
    }

    #[test]
    fn a_repulsor_not_placed_for_two_stamps_is_not_nearby() {
        let mut list = Repulsors::default();
        let handle = list.create(spec(0.0, 0.0), 0.0);
        assert!(list.nearby(0.0).is_empty(), "never placed");
        let stamp = list.next_stamp();
        list.get_mut(handle).unwrap().place(Vec3::X, stamp, 0.0);
        assert_eq!(list.nearby(0.0).len(), 1);
        list.next_stamp();
        assert_eq!(list.nearby(0.0).len(), 1, "one frame stale still counts");
        list.next_stamp();
        assert!(list.nearby(0.0).is_empty(), "two frames stale does not");
        list.destroy(handle);
        assert!(list.is_empty());
    }
}
