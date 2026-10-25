//! Asheron's Call 2 particle systems (Legions `portal/client/src/render/particle`): the
//! `ParticleSystem` that runs a `PSDesc`'s emitters, each `ParticleEmitter` discharging particles
//! on its birth rate, thinking them through their keyframes and motion controllers, and filling
//! the billboard quads the renderer draws with the system's material.
//!
//! Positions are world axes (x east, y north, z up). A system whose `PSDesc` is not world space
//! keeps its particles relative to its own origin, in world axes: retail's `m_worldPosition`
//! stays the identity for it and `RenderAtPosition` places it by translation and scale alone. A
//! world-space system's particles are placed by the system's frame when they are discharged.
//!
//! Not modelled: repulsors (`ParticleRepulsor`, controller 3), emission by distance travelled
//! (`EmitDistanceParticles`: a placed system never moves), particle lighting
//! (`GetLighting`: particles draw unlit) and shared systems (each source runs its own).

mod controllers;
mod emitter;
mod fill;
mod keyframes;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use ac2_assets::Ac2ParticleSystemDesc;
use ac2_core::{DataId, Random};
use ac2_math::{Frame, Vec3};

pub use fill::{CameraBasis, ParticleGeometry};

use emitter::Emitter;

/// `PhysicsGlobals::gravity`.
pub(crate) const GRAVITY: Vec3 = Vec3::new(0.0, 0.0, -9.8);
/// `F_EPSILON`.
pub(crate) const EPSILON: f32 = 0.0002;

/// Everything an emitter reads from its system while it runs.
pub(crate) struct SystemContext<'a> {
    pub desc: &'a Ac2ParticleSystemDesc,
    /// `m_fStartTime`: the start time less the description's fast-forward.
    pub start_time: f64,
    /// `m_worldPosition`'s frame: the placed frame for a world-space system, else the identity.
    pub frame: Frame,
    pub rng: &'a mut Random,
}

/// A generator for the system with `seed`: the client's one stream, wound on by the seed so that
/// systems of one description do not emit in step.
fn seeded(seed: u32) -> Random {
    let mut rng = Random::default();
    for _ in 0..64 + (seed & 0x3FF) {
        rng.rand();
    }
    rng
}

/// One running particle system.
pub struct ParticleSystem {
    desc: Arc<Ac2ParticleSystemDesc>,
    emitters: Vec<Emitter>,
    /// Where the system stands, and the entity and particle scales it was placed with.
    frame: Frame,
    object_scale: Vec3,
    particle_scale: Vec3,
    start_time: f64,
    creation_time: f64,
    /// `m_bDisabled`: not drawn last frame, so not thinking until it is drawn again.
    disabled: bool,
    rng: Random,
}

impl ParticleSystem {
    /// `ParticleSystem::Create` and `Start` at client time `now`: the emitters are built and
    /// start `FastForwardTime` seconds ago, so the system opens already running.
    pub fn new(
        desc: Arc<Ac2ParticleSystemDesc>,
        frame: Frame,
        object_scale: Vec3,
        particle_scale: f32,
        seed: u32,
        now: f64,
    ) -> Self {
        let emitters = (0..desc.emitters.len())
            .map(|index| Emitter::new(&desc, index))
            .collect();
        let start_time = now - f64::from(desc.fast_forward_time);
        let mut system = Self {
            desc,
            emitters,
            frame,
            object_scale,
            particle_scale: Vec3::splat(particle_scale),
            start_time,
            creation_time: now,
            disabled: false,
            rng: seeded(seed),
        };
        let starts: Vec<f64> = system
            .desc
            .emitters
            .iter()
            .map(|emitter| f64::from(emitter.start_time) + start_time)
            .collect();
        for (emitter, start) in system.emitters.iter_mut().zip(starts) {
            emitter.set_last_discharge_time(start);
        }
        system
    }

    /// The `MaterialInstance` its particles draw with.
    pub fn material(&self) -> Option<DataId> {
        self.desc.material
    }

    /// `ParticleSystem::GetForceDraw`.
    pub fn force_draw(&self) -> bool {
        self.desc.force_draw
    }

    /// Whether any emitter responds to repulsors: those systems stay off here.
    pub fn needs_repulsors(&self) -> bool {
        self.emitters
            .iter()
            .any(|emitter| emitter.responds_to_repulsors())
    }

    /// The particles alive (`m_nParticlesAlive`).
    pub fn alive(&self) -> usize {
        self.emitters.iter().map(Emitter::len).sum()
    }

    /// `ParticleSystem::Think` at client time `now`.
    pub fn think(&mut self, now: f64) {
        if self.disabled {
            return;
        }
        let now = now.max(self.creation_time);
        let duration = self.desc.physics_duration;
        if duration != 0.0 && now - self.start_time > f64::from(duration) {
            return;
        }
        let mut context = SystemContext {
            desc: &self.desc,
            start_time: self.start_time,
            frame: self.context_frame(),
            rng: &mut self.rng,
        };
        for emitter in &mut self.emitters {
            emitter.emit_particles(&mut context, now);
            emitter.think(&mut context, now);
        }
    }

    /// `RenderAtPosition` came back without drawing: the system waits until it is drawn again.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    fn context_frame(&self) -> Frame {
        if self.desc.world_space {
            self.frame
        } else {
            Frame::IDENTITY
        }
    }

    /// The system's distance fade (`RenderAtPosition`): `None` beyond `StopFadeDistance`.
    pub fn fade(&self, camera: Vec3) -> Option<f32> {
        let distance = camera.distance(self.frame.origin);
        let (start, stop) = (self.desc.start_fade_distance, self.desc.stop_fade_distance);
        if stop < distance {
            return None;
        }
        if start < distance && stop - start > EPSILON {
            return Some(1.0 - (distance - start) / (stop - start));
        }
        Some(1.0)
    }

    /// `ParticleSystem::FillVertexBuffer` and `RenderAtPosition`'s placement: appends the
    /// system's quads and triangles, in world space, scaled by `alpha`, for a camera at `basis`.
    pub fn fill(&self, basis: &CameraBasis, now: f64, alpha: f32, out: &mut ParticleGeometry) {
        let object_scale = self.object_scale;
        let particle_scale = if self.desc.scale_type == 2 {
            object_scale * self.particle_scale
        } else {
            object_scale
        };
        // The scale `RenderAtPosition` hands the fill: the entity scale, times the visual's
        // particle scale for scale type 2.
        let scale = particle_scale;
        for emitter in &self.emitters {
            emitter.fill(
                &self.desc,
                &fill::FillContext {
                    basis,
                    now,
                    alpha,
                    scale,
                    system_start: self.start_time,
                    placement: (!self.desc.world_space).then_some(self.frame.origin),
                },
                out,
            );
        }
    }
}
