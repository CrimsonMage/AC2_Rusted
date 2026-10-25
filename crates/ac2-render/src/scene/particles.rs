//! The particle systems of the resident landblocks' entities (`PSManager`): each source starts
//! a system when its block becomes resident, systems near the camera think every frame
//! (`ParticleSystem::Queue` then `Think`) and fill their particles as quads, merged per material
//! for the renderer.

use std::collections::HashMap;
use std::time::Instant;

use ac2_core::DataId;

use super::loader::{BlockKey, ParticleSource};
use crate::particles::{CameraBasis, ParticleGeometry, ParticleSystem};
use crate::Camera;

/// One material's particles of the frame, in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleBatch {
    pub material: DataId,
    pub geometry: ParticleGeometry,
}

pub(super) struct SceneParticles {
    started: Instant,
    systems: HashMap<BlockKey, Vec<ParticleSystem>>,
    /// The particles alive in the last frame's drawn systems.
    alive: usize,
}

impl SceneParticles {
    pub(super) fn new() -> Self {
        Self {
            started: Instant::now(),
            systems: HashMap::new(),
            alive: 0,
        }
    }

    /// Client time for the systems (`Timer::local_time`).
    fn now(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    /// `ParticleSystem::Create` for each startup source of a block that became resident.
    pub(super) fn add_block(&mut self, key: BlockKey, sources: &[ParticleSource]) {
        let now = self.now();
        let systems = sources
            .iter()
            .map(|entry| {
                ParticleSystem::new(
                    entry.desc.clone(),
                    entry.source.world_frame,
                    entry.source.object_scale,
                    entry.source.particle_scale,
                    entry.source.seed,
                    now,
                )
            })
            .collect();
        self.systems.insert(key, systems);
    }

    pub(super) fn remove_block(&mut self, key: &BlockKey) {
        self.systems.remove(key);
    }

    pub(super) fn alive(&self) -> usize {
        self.alive
    }

    /// Thinks the systems within their fade distance of `camera` and fills their particles.
    pub(super) fn frame(&mut self, camera: &Camera) -> Vec<ParticleBatch> {
        let Some(basis) = CameraBasis::look_at(camera.eye, camera.target) else {
            return Vec::new();
        };
        let now = self.now();
        let mut keys: Vec<BlockKey> = self.systems.keys().copied().collect();
        keys.sort_unstable();
        let mut batches: Vec<ParticleBatch> = Vec::new();
        let mut alive = 0;
        for key in keys {
            for system in self.systems.get_mut(&key).into_iter().flatten() {
                let Some(material) = system.material() else {
                    continue;
                };
                // `RenderAtPosition`: beyond the stop distance a system is not drawn and waits.
                let Some(fade) = system.fade(camera.eye) else {
                    system.set_disabled(true);
                    continue;
                };
                system.set_disabled(false);
                if system.needs_repulsors() {
                    continue;
                }
                system.think(now);
                alive += system.alive();
                let index = match batches.iter().position(|b| b.material == material) {
                    Some(index) => index,
                    None => {
                        batches.push(ParticleBatch {
                            material,
                            geometry: ParticleGeometry::default(),
                        });
                        batches.len() - 1
                    }
                };
                system.fill(&basis, now, fade, &mut batches[index].geometry);
            }
        }
        self.alive = alive;
        batches.retain(|batch| !batch.geometry.is_empty());
        batches
    }
}
