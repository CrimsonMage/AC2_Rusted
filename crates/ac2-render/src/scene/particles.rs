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
use ac2_assets::{Ac2ParticleAnchor, Ac2ParticleSystemDesc};
use ac2_core::DataId;
use ac2_math::{Frame, Quat, Vec3};

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

/// Where an object FX's system attaches, as the FX started it: the unposed anchor
/// (`Ac2ParticleAnchor`, posed against the object's bones each frame), `m_remove_rotation`, the
/// node's water-plane flag and the water surface height found under the object when the FX
/// started (`FXManager::FireParticleFX`'s water branch; `None` without water).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleAnchor {
    pub anchor: Ac2ParticleAnchor,
    pub remove_rotation: bool,
    pub water_plane: bool,
    pub water_z: Option<f32>,
}

/// A system an object's FX started: its emitter (the object), where it attaches, and the system
/// once its description has loaded (`None` while it loads).
struct ObjectSystem {
    emitter: u64,
    anchor: ParticleAnchor,
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
    /// Where each emitter stands this frame: its frame, total scale and root bones.
    placed: HashMap<u64, PlacedObject>,
    /// The particles alive in the last frame's drawn systems.
    alive: usize,
}

/// An object's frame, scale and bones this frame (see [`ObjectPlacement`]).
struct PlacedObject {
    frame: Frame,
    scale: Vec3,
    root: Vec<Ac2Transform>,
    named: Vec<(u32, Ac2Transform)>,
}

impl PlacedObject {
    fn bones(&self) -> Bones<'_> {
        Bones {
            root: &self.root,
            named: &self.named,
        }
    }
}

/// An object's bones as its systems see them: the root Setup's bones by index (the holding
/// points), and every part group's bones by name in `CPartGroup::FindBoneType`'s search order.
#[derive(Debug, Clone, Copy)]
struct Bones<'a> {
    root: &'a [Ac2Transform],
    named: &'a [(u32, Ac2Transform)],
}

impl Bones<'_> {
    /// A root bone's model-space frame (`CPartArray::m_boneMatrices`); `None` past the Setup's
    /// bones.
    fn root(&self, index: usize) -> Option<Frame> {
        bone_frame(self.root, index)
    }

    /// The first bone with that name in the search order (`CPartGroup::FindBoneType` finds the
    /// first group whose Setup has it); `None` when no group has it.
    fn named(&self, name: u32) -> Option<Frame> {
        self.named
            .iter()
            .find(|(bone, _)| *bone == name)
            .map(|(_, bone)| Frame::new(bone.offset, bone.rotation))
    }
}

/// A bone's model-space frame.
fn bone_frame(bones: &[Ac2Transform], index: usize) -> Option<Frame> {
    bones
        .get(index)
        .map(|bone| Frame::new(bone.offset, bone.rotation))
}

/// `CPartArray::QueueParticleSystems`' connection for an attached system: its offset in the
/// bone's space, the holding point's transform in its bone's space, or the object's offset. A
/// bone that is not in the object's Setup is not an attachment, so `None` (the system is not
/// placed). A removed rotation keeps only the translation.
fn attached(anchor: &ParticleAnchor, bones: Bones) -> Option<Frame> {
    let in_object = |offset: Vec3| Frame::new(offset, Quat::IDENTITY);
    let connection = match anchor.anchor {
        Ac2ParticleAnchor::Object { offset } => in_object(offset),
        Ac2ParticleAnchor::Bone { name, offset } => {
            bones.named(name)?.compose_frame(in_object(offset))
        }
        Ac2ParticleAnchor::Holding {
            location: Some((index, transform)),
        } => {
            let location = Frame::new(transform.offset, transform.rotation);
            bones
                .root(index)
                .map_or(location, |bone| bone.compose_frame(location))
        }
        // A missing holding point attaches to the first bone at the identity (`PHYSICSOBJREP::
        // CreateParticleSystem` keeps the zero index and the identity matrix).
        Ac2ParticleAnchor::Holding { location: None } => bones.root(0).unwrap_or(Frame::IDENTITY),
    };
    Some(if anchor.remove_rotation {
        Frame::new(connection.origin, Quat::IDENTITY)
    } else {
        connection
    })
}

/// `FXManager::FireParticleFX`'s connection for a fire-and-forget system: a holding point's
/// transform alone, the posed bone (its offset ignored) or the bone offset, and for a water-plane
/// node no offset at all (the system stands at its object, on the water).
fn fired(anchor: &ParticleAnchor, bones: Bones) -> Frame {
    match anchor.anchor {
        Ac2ParticleAnchor::Holding {
            location: Some((_, transform)),
        } => Frame::new(transform.offset, transform.rotation),
        Ac2ParticleAnchor::Holding { location: None } => Frame::IDENTITY,
        Ac2ParticleAnchor::Bone { name, offset } => bones
            .named(name)
            .unwrap_or(Frame::new(offset, Quat::IDENTITY)),
        Ac2ParticleAnchor::Object { .. } if anchor.water_plane => Frame::IDENTITY,
        Ac2ParticleAnchor::Object { offset } => Frame::new(offset, Quat::IDENTITY),
    }
}

/// The system's frame in the world: the object's frame (for fire-and-forget on water, standing at
/// the water surface) composed with the connection, whose offset is scaled by the object's scale.
/// `None` when an attached system has no bone to stand on.
fn attach(
    object: Frame,
    bones: Bones,
    anchor: &ParticleAnchor,
    follow: bool,
    scale: Vec3,
) -> Option<Frame> {
    let (base, connection) = if follow {
        (object, attached(anchor, bones)?)
    } else {
        let mut base = object;
        if let (true, Ac2ParticleAnchor::Object { .. }, Some(z)) =
            (anchor.water_plane, anchor.anchor, anchor.water_z)
        {
            base.origin.z = z;
        }
        (base, fired(anchor, bones))
    };
    Some(base.compose_frame(Frame::new(
        connection.origin * scale,
        connection.orientation,
    )))
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

    /// `CPartArray::CreateParticleSystem`: a system `handle` on `emitter`, at `anchor`. Its
    /// description still has to load; `false` when `handle` is already known.
    pub(super) fn request_object(
        &mut self,
        handle: u64,
        emitter: u64,
        anchor: ParticleAnchor,
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
                anchor,
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
        self.placed.extend(placements.iter().map(|p| {
            (
                p.emitter,
                PlacedObject {
                    frame: p.frame,
                    scale: p.scale,
                    root: p.bones.clone(),
                    named: p.named_bones.clone(),
                },
            )
        }));
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
                let Some(placed) = self.placed.get(&entry.emitter) else {
                    continue;
                };
                let Some(frame) = attach(
                    placed.frame,
                    placed.bones(),
                    &entry.anchor,
                    entry.follow,
                    placed.scale,
                ) else {
                    continue;
                };
                system.place(frame, placed.scale, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bone(offset: Vec3, rotation: Quat) -> Ac2Transform {
        Ac2Transform {
            offset,
            scale: Vec3::ONE,
            rotation,
        }
    }

    fn anchor(anchor: Ac2ParticleAnchor) -> ParticleAnchor {
        ParticleAnchor {
            anchor,
            remove_rotation: false,
            water_plane: false,
            water_z: None,
        }
    }

    /// Whether two quaternions are the same rotation (`q` and `-q` alike).
    fn same_rotation(a: Quat, b: Quat) -> bool {
        a.dot(b).abs() > 1.0 - 1e-5
    }

    fn object_at(origin: Vec3) -> Frame {
        Frame::new(origin, Quat::IDENTITY)
    }

    /// `QueueParticleSystems`: an attached system stands at its offset in the bone's space, the
    /// bone posed by the object's skeleton this frame.
    #[test]
    fn an_attached_system_follows_its_posed_bone() {
        let bones = [
            bone(Vec3::ZERO, Quat::IDENTITY),
            bone(Vec3::new(0.0, 0.0, 2.0), Quat::IDENTITY),
        ];
        let named = [(7, bones[1])];
        let anchor = anchor(Ac2ParticleAnchor::Bone {
            name: 7,
            offset: Vec3::new(0.0, 1.0, 0.0),
        });
        let frame = attach(
            object_at(Vec3::new(10.0, 0.0, 0.0)),
            Bones {
                root: &bones,
                named: &named,
            },
            &anchor,
            true,
            Vec3::ONE,
        )
        .expect("the bone exists");
        assert!(frame.origin.distance(Vec3::new(10.0, 1.0, 2.0)) < 1e-5);
    }

    /// A holding point's transform sits in its bone's space; a removed rotation keeps only the
    /// translation, as `CreateParticleSystem` does with `m_remove_rotation`.
    #[test]
    fn a_holding_point_is_its_bones_space_and_rotation_can_be_removed() {
        let turned = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let bones = [bone(Vec3::new(0.0, 0.0, 1.0), Quat::IDENTITY)];
        let location = Ac2Transform {
            offset: Vec3::new(0.0, 0.0, 0.5),
            scale: Vec3::ONE,
            rotation: turned,
        };
        let holding = anchor(Ac2ParticleAnchor::Holding {
            location: Some((0, location)),
        });
        let no_names = Bones {
            root: &bones,
            named: &[],
        };
        let kept = attach(Frame::IDENTITY, no_names, &holding, true, Vec3::ONE).unwrap();
        assert!(kept.origin.distance(Vec3::new(0.0, 0.0, 1.5)) < 1e-5);
        assert!(same_rotation(kept.orientation, turned));
        let removed = ParticleAnchor {
            remove_rotation: true,
            ..holding
        };
        let frame = attach(Frame::IDENTITY, no_names, &removed, true, Vec3::ONE).unwrap();
        assert!(frame.origin.distance(Vec3::new(0.0, 0.0, 1.5)) < 1e-5);
        assert!(same_rotation(frame.orientation, Quat::IDENTITY));
    }

    /// `CreateParticleSystem` with a bone no part group has: no system at all.
    #[test]
    fn an_attached_system_without_its_bone_is_not_placed() {
        let missing = anchor(Ac2ParticleAnchor::Bone {
            name: 9,
            offset: Vec3::ONE,
        });
        let none = Bones {
            root: &[],
            named: &[],
        };
        assert!(attach(Frame::IDENTITY, none, &missing, true, Vec3::ONE).is_none());
    }

    /// `FindBoneType` searches the child part groups too: a bone only a child group's Setup has
    /// is found by name, the first group in search order winning.
    #[test]
    fn a_bone_of_a_child_group_is_found_by_name_in_search_order() {
        let root = [bone(Vec3::ZERO, Quat::IDENTITY)];
        let named = [
            (1, bone(Vec3::ZERO, Quat::IDENTITY)),
            (5, bone(Vec3::new(0.0, 3.0, 0.0), Quat::IDENTITY)),
            (5, bone(Vec3::new(0.0, 4.0, 0.0), Quat::IDENTITY)),
        ];
        let bones = Bones {
            root: &root,
            named: &named,
        };
        let child = anchor(Ac2ParticleAnchor::Bone {
            name: 5,
            offset: Vec3::ZERO,
        });
        let frame = attach(Frame::IDENTITY, bones, &child, true, Vec3::ONE).unwrap();
        assert!(frame.origin.distance(Vec3::new(0.0, 3.0, 0.0)) < 1e-5);
    }

    /// `FireParticleFX`: a holding point's transform alone, without its bone's pose.
    #[test]
    fn a_fire_and_forget_holding_point_ignores_its_bone() {
        let bones = [
            bone(Vec3::ZERO, Quat::IDENTITY),
            bone(Vec3::new(0.0, 0.0, 2.0), Quat::IDENTITY),
        ];
        let location = Ac2Transform {
            offset: Vec3::new(0.0, 0.0, 0.5),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        };
        let fired = anchor(Ac2ParticleAnchor::Holding {
            location: Some((1, location)),
        });
        let frame = attach(
            object_at(Vec3::ZERO),
            Bones {
                root: &bones,
                named: &[],
            },
            &fired,
            false,
            Vec3::ONE,
        )
        .unwrap();
        assert!(frame.origin.distance(Vec3::new(0.0, 0.0, 0.5)) < 1e-5);
    }

    /// `FireParticleFX` with a posed bone: the bone's frame, the bone offset not applied.
    #[test]
    fn a_fire_and_forget_bone_takes_the_posed_bone_not_its_offset() {
        let named = [(3, bone(Vec3::new(0.0, 0.0, 2.0), Quat::IDENTITY))];
        let fired = anchor(Ac2ParticleAnchor::Bone {
            name: 3,
            offset: Vec3::new(5.0, 5.0, 5.0),
        });
        let bones = Bones {
            root: &[],
            named: &named,
        };
        let frame = attach(object_at(Vec3::ZERO), bones, &fired, false, Vec3::ONE).unwrap();
        assert!(frame.origin.distance(Vec3::new(0.0, 0.0, 2.0)) < 1e-5);
    }

    /// `FireParticleFX`'s water branch: the system stands at the object on the water surface, and
    /// the node's offset is not applied; without water the object's own height stays.
    #[test]
    fn a_water_plane_system_stands_on_the_surface_under_its_object() {
        let water = ParticleAnchor {
            anchor: Ac2ParticleAnchor::Object {
                offset: Vec3::new(0.0, 0.0, 9.0),
            },
            remove_rotation: false,
            water_plane: true,
            water_z: Some(3.0),
        };
        let object = object_at(Vec3::new(1.0, 2.0, 7.0));
        let frame = attach(
            object,
            Bones {
                root: &[],
                named: &[],
            },
            &water,
            false,
            Vec3::ONE,
        )
        .unwrap();
        assert!(frame.origin.distance(Vec3::new(1.0, 2.0, 3.0)) < 1e-5);
        let dry = ParticleAnchor {
            water_z: None,
            ..water
        };
        let frame = attach(
            object,
            Bones {
                root: &[],
                named: &[],
            },
            &dry,
            false,
            Vec3::ONE,
        )
        .unwrap();
        assert!(frame.origin.distance(Vec3::new(1.0, 2.0, 7.0)) < 1e-5);
    }
}
