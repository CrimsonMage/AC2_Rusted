//! The particle systems of the resident landblocks' entities (`PSManager`): each source starts
//! a system when its block becomes resident, systems near the camera think every frame
//! (`ParticleSystem::Queue` then `Think`) and fill their particles as quads, merged per material
//! for the renderer. An object's FX systems (`CPartArray::CreateParticleSystem`) follow the
//! object: placed each frame where the object stands, until they are released by a stop, then
//! they keep their last place until they are ready to die (`PSManager::RegisterAutonomousSystem`).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use ac2_anim::Ac2Transform;
use ac2_assets::Ac2ParticleSystemDesc;
use ac2_core::DataId;
use ac2_math::{Frame, Vec3};

use super::loader::{BlockKey, ParticleSource};
use super::ObjectPlacement;
use crate::particles::{CameraBasis, ParticleGeometry, ParticleSystem};
use crate::Camera;

/// One material's particles of the frame, in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleBatch {
    pub material: DataId,
    pub geometry: ParticleGeometry,
}

/// A system an object's FX started: its emitter (the object), where it attaches, and the system
/// once its description has loaded (`None` while it loads).
struct ObjectSystem {
    emitter: u64,
    placement: Ac2Transform,
    /// Kept on its object every frame; a fire-and-forget system is placed once, where its object
    /// stood when it started (`FXManager::FireParticleFX`).
    follow: bool,
    /// The client time the system was started at (`ParticleSystem::Start`).
    started: f64,
    /// Released by a stop: it keeps its last place and runs out its particles.
    stopped: bool,
    system: Option<ParticleSystem>,
}

pub(super) struct SceneParticles {
    started: Instant,
    /// The client's time, when the frame pins it (`set_clock`); the wall clock otherwise.
    clock: Option<f64>,
    systems: HashMap<BlockKey, Vec<ParticleSystem>>,
    objects: HashMap<u64, ObjectSystem>,
    /// Where each emitter stands this frame: its frame and total scale.
    placed: HashMap<u64, (Frame, Vec3)>,
    /// The particles alive in the last frame's drawn systems.
    alive: usize,
}

/// `CPartArray::QueueParticleSystems`' placement: the system's frame is the object's frame with
/// the attachment's offset in the object's space, scaled by the object's scale; a removed
/// rotation keeps only the translation (already done by the attachment).
fn attach(object: Frame, placement: Ac2Transform, scale: Vec3) -> Frame {
    object.compose_frame(Frame::new(placement.offset * scale, placement.rotation))
}

impl SceneParticles {
    pub(super) fn new() -> Self {
        Self {
            started: Instant::now(),
            clock: None,
            systems: HashMap::new(),
            objects: HashMap::new(),
            placed: HashMap::new(),
            alive: 0,
        }
    }

    /// Client time for the systems (`Timer::local_time`).
    pub(super) fn now(&self) -> f64 {
        self.clock
            .unwrap_or_else(|| self.started.elapsed().as_secs_f64())
    }

    pub(super) fn set_clock(&mut self, now: f64) {
        self.clock = Some(now);
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

    /// `CPartArray::CreateParticleSystem`: a system `handle` on `emitter`, at `placement`. Its
    /// description still has to load; `false` when `handle` is already known.
    pub(super) fn request_object(
        &mut self,
        handle: u64,
        emitter: u64,
        placement: Ac2Transform,
        follow: bool,
    ) -> bool {
        if self.objects.contains_key(&handle) {
            return false;
        }
        let started = self.now();
        self.objects.insert(
            handle,
            ObjectSystem {
                emitter,
                placement,
                follow,
                started,
                stopped: false,
                system: None,
            },
        );
        true
    }

    /// A description loaded for `handle`: the system starts at the time it was requested. A
    /// system stopped or dropped meanwhile ignores it; a description that draws nothing drops it.
    pub(super) fn object_loaded(&mut self, handle: u64, desc: Option<Arc<Ac2ParticleSystemDesc>>) {
        let Some(entry) = self.objects.get_mut(&handle) else {
            return;
        };
        match desc {
            Some(desc) => {
                entry.system = Some(ParticleSystem::new(
                    desc,
                    Frame::IDENTITY,
                    Vec3::ONE,
                    1.0,
                    handle as u32,
                    entry.started,
                ));
            }
            None => {
                self.objects.remove(&handle);
            }
        }
    }

    /// `CPartArray::DestroyParticleSystem`: a system never drawn is destroyed; one that was
    /// placed becomes autonomous at its last place.
    pub(super) fn stop_object(&mut self, handle: u64) {
        let placed = self
            .objects
            .get(&handle)
            .is_some_and(|entry| entry.system.as_ref().is_some_and(ParticleSystem::placed));
        if placed {
            if let Some(entry) = self.objects.get_mut(&handle) {
                entry.stopped = true;
            }
        } else {
            self.objects.remove(&handle);
        }
    }

    /// How many object systems the scene holds (loading, running or running out).
    pub(super) fn object_systems(&self) -> usize {
        self.objects.len()
    }

    /// The frame's object placements (`CPartArray::QueueParticleSystems`' object frames).
    pub(super) fn place_objects(&mut self, placements: &[ObjectPlacement]) {
        self.placed.clear();
        self.placed
            .extend(placements.iter().map(|p| (p.emitter, (p.frame, p.scale))));
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
        let mut batches: Vec<ParticleBatch> = Vec::new();
        let mut alive = 0;
        let mut keys: Vec<BlockKey> = self.systems.keys().copied().collect();
        keys.sort_unstable();
        for key in keys {
            for system in self.systems.get_mut(&key).into_iter().flatten() {
                draw(system, camera, &basis, now, &mut batches, &mut alive);
            }
        }
        let mut handles: Vec<u64> = self.objects.keys().copied().collect();
        handles.sort_unstable();
        let mut retired = Vec::new();
        for handle in handles {
            let Some(entry) = self.objects.get_mut(&handle) else {
                continue;
            };
            let Some(system) = entry.system.as_mut() else {
                continue;
            };
            if !entry.stopped && (entry.follow || !system.placed()) {
                let Some((frame, scale)) = self.placed.get(&entry.emitter) else {
                    continue;
                };
                system.place(attach(*frame, entry.placement, *scale), *scale, 1.0);
            }
            draw(system, camera, &basis, now, &mut batches, &mut alive);
            if system.ready_to_die(now) {
                retired.push(handle);
            }
        }
        for handle in retired {
            self.objects.remove(&handle);
        }
        self.alive = alive;
        batches.retain(|batch| !batch.geometry.is_empty());
        batches
    }
}

/// `ParticleSystem::Render`'s path for one system this frame: its distance fade, its thinking,
/// and its quads merged into the batch of its material.
fn draw(
    system: &mut ParticleSystem,
    camera: &Camera,
    basis: &CameraBasis,
    now: f64,
    batches: &mut Vec<ParticleBatch>,
    alive: &mut usize,
) {
    let Some(material) = system.material() else {
        return;
    };
    // `RenderAtPosition`: beyond the stop distance a system is not drawn and waits.
    let Some(fade) = system.fade(camera.eye) else {
        system.set_disabled(true);
        return;
    };
    system.set_disabled(false);
    if system.needs_repulsors() {
        return;
    }
    system.think(now);
    *alive += system.alive();
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
    system.fill(basis, now, fade, &mut batches[index].geometry);
}
