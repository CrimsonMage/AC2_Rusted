//! The tunnel's clouds (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`,
//! `PortalSpace::Effect::Render`, and `ParticleEmitter::DischargeParticle`): the particle system
//! `Initialize` makes from PSDesc `0x40000015`. While the tunnel is more than 0.9 visible, each of
//! the portals 40 to 49 ahead of the last one passed is discharged once: half the time a particle
//! is born at a random point of its ring (within 0.8 of its radius), and the portal is marked. The
//! system then draws in the identity frame at unit scale.

use std::fmt;
use std::sync::Arc;

use ac2_assets::Ac2ParticleSystemDesc;
use ac2_core::{DataId, Random};
use ac2_math::matrix::Matrix4;
use ac2_math::{Frame, Vec3};

use super::tunnel::{make_safe_index, Tunnel};
use crate::particles::{CameraBasis, ParticleGeometry, ParticleSystem};

/// The portal offsets from the last one passed that `Render` discharges at (`0x28` to `0x31`).
const DISCHARGE_FIRST: usize = 40;
const DISCHARGE_END: usize = 50;
/// The visibility `Render` discharges above (`0.9 < m_fTunnelVisibility`).
const DISCHARGE_VISIBILITY: f64 = 0.9;
/// The spread of a discharge, as a fraction of the portal's radius either side of its centre.
const SPREAD: f64 = 0.8;

/// The tunnel clouds: the particle system (`m_pTunnelCloudsParticleSystem`), or none when the DAT
/// has no description.
#[derive(Default)]
pub struct PortalClouds {
    system: Option<ParticleSystem>,
    material: Option<DataId>,
}

impl fmt::Debug for PortalClouds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PortalClouds")
            .field("material", &self.material)
            .field("running", &self.system.is_some())
            .finish()
    }
}

impl PortalClouds {
    /// `PSManager::CreateParticleSystem(PSDesc, 0)` at client time `now`: the system is placed at
    /// the identity frame and runs at unit scale.
    pub fn new(desc: Option<Arc<Ac2ParticleSystemDesc>>, now: f64) -> Self {
        let material = desc.as_ref().and_then(|desc| desc.material);
        let system =
            desc.map(|desc| ParticleSystem::new(desc, Frame::IDENTITY, Vec3::ONE, 1.0, 0, now));
        Self { system, material }
    }

    /// The clouds' material (the description's `MaterialInstance`), drawn by the object pass.
    pub fn material(&self) -> Option<DataId> {
        self.material
    }

    /// `Render`'s discharge loop: while `visibility` is above 0.9, each portal in the window that
    /// has no particle yet may discharge one at a random point of its ring, then is marked.
    pub fn discharge(
        &mut self,
        tunnel: &mut Tunnel,
        last: usize,
        visibility: f32,
        now: f64,
        random: &mut Random,
    ) {
        if f64::from(visibility) <= DISCHARGE_VISIBILITY {
            return;
        }
        let Some(system) = self.system.as_mut() else {
            return;
        };
        for step in DISCHARGE_FIRST..DISCHARGE_END {
            let index = make_safe_index((last + step) as i64);
            let portal = &mut tunnel.portals[index];
            if portal.has_particle {
                continue;
            }
            // `fVar13 < 0.5 != (fVar13 == 0.5)`: a roll of at most one half.
            if random.roll_dice_f32(0.0, 1.0) <= 0.5 {
                let radius = f64::from(portal.radius);
                // The first roll is the point's z, the second its x (`fVar14`, `fVar15`).
                let (lower, upper) = ((radius * -SPREAD) as f32, (radius * SPREAD) as f32);
                let z = random.roll_dice_f32(lower, upper);
                let x = random.roll_dice_f32(lower, upper);
                system.discharge_offset(now, ring_point(&portal.world, x, z));
            }
            portal.has_particle = true;
        }
    }

    /// `ParticleSystem::Queue` and `Render`: the system thinks at `now` and fills its quads for
    /// the camera at `basis`. Empty when there is no system.
    pub fn geometry(&mut self, now: f64, basis: &CameraBasis) -> ParticleGeometry {
        let mut out = ParticleGeometry::default();
        if let Some(system) = self.system.as_mut() {
            system.think(now);
            system.fill(basis, now, 1.0, &mut out);
        }
        out
    }
}

/// A point of a ring's plane `(x, 0, z)` in the world: the ring's matrix, divided by its `w`. The
/// divide is taken in extended precision and the coordinates round to float at the store.
fn ring_point(world: &Matrix4, x: f32, z: f32) -> Vec3 {
    let [px, py, pz, w] = world.transform_f64(f64::from(x), 0.0, f64::from(z));
    let inverse = 1.0 / w;
    Vec3::new(
        (px * inverse) as f32,
        (py * inverse) as f32,
        (pz * inverse) as f32,
    )
}

#[cfg(test)]
mod tests {
    use ac2_core::Random;
    use ac2_math::matrix::Matrix4;

    use super::{ring_point, PortalClouds};
    use crate::portal::tunnel::{Portal, Tunnel};

    #[test]
    fn a_ring_point_on_the_identity_is_its_plane_coordinates() {
        let p = ring_point(&Matrix4::identity(), 0.5, -2.0);
        assert_eq!((p.x, p.y, p.z), (0.5, 0.0, -2.0));
    }

    #[test]
    fn nothing_discharges_below_the_visibility_threshold() {
        let mut tunnel = Tunnel::new();
        tunnel.portals[45] = Portal {
            radius: 10.0,
            ..Portal::default()
        };
        let mut clouds = PortalClouds::default();
        let mut random = Random::default();
        clouds.discharge(&mut tunnel, 0, 0.9, 1.0, &mut random);
        assert!(!tunnel.portals[45].has_particle);
        assert_eq!(clouds.material(), None);
    }
}
