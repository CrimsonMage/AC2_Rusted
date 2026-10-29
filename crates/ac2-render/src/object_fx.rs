//! The FX the scene draws on its objects: the particle systems, lights and streaks the objects' FX
//! start, stop and destroy, and where each drawn object stands for the systems it runs. An object
//! that hangs from an emitter follows its posed bone or holding point (`CPhysicsObj::UpdateChild`,
//! [`follow_emitters`]). The client gives each drawn object as an [`FxSlot`] and the posed frames;
//! the calls are applied to the scene in the order the FX made them.

use std::collections::HashMap;

use ac2_math::{Frame, Mat4, Vec3};
use ac2_physics::{follow_frame, FxFollow};

use crate::decal::DecalSpec;
use crate::fx_particles::{ObjectLightStart, ObjectParticleStart};
use crate::fx_streaks::ObjectStreakCall;
use crate::object::ObjectPose;
use crate::scene::{ObjectPlacement, WorldScene};

/// One drawn object as the FX placement sees it: its id, its model key and cell, and the emitter
/// and attach point of an FX object that hangs from one.
#[derive(Debug, Clone, Copy)]
pub struct FxSlot {
    pub id: u64,
    /// The model key the scene holds the object's rig under (`WorldScene::object_rig`).
    pub key: u64,
    pub cell: u32,
    pub follow: Option<FxFollow>,
}

/// An object's model-to-world matrix: its total scale, then its place.
pub fn world_matrix(scale: Vec3, at: Frame) -> Mat4 {
    Mat4::from_scale_rotation_translation(scale, at.orientation, at.origin)
}

/// The particle systems the objects' FX started (`FXManager::DoParticleFX`) and stopped
/// (`StopFX`, or their object left), in that order: the scene loads each description on its
/// loader thread and runs the system where its object stands.
pub fn apply_particles(scene: &mut WorldScene, starts: &[ObjectParticleStart], stops: &[u64]) {
    for start in starts {
        scene.start_object_particle(
            start.handle,
            start.emitter,
            start.desc,
            start.anchor,
            !start.fire_and_forget,
        );
    }
    for handle in stops {
        scene.stop_object_particle(*handle);
    }
}

/// The lights the objects' FX placed (`DoLightSourceFX`) and the objects that left the world
/// (`DestroyAllTransientFX`): the scene's point lights reach the objects it draws.
pub fn apply_lights(scene: &mut WorldScene, starts: &[ObjectLightStart], gone: &[u64]) {
    for start in starts {
        scene.start_object_light(start.emitter, start.light);
    }
    for emitter in gone {
        scene.remove_object_lights(*emitter);
    }
}

/// The streaks the objects' FX made, started, stopped and destroyed (`FXManager::DoStreakFX`,
/// `StartStreak`, `StopStreak`), in order; `visible` is `StreakEffects` (the ribbons draw only
/// while it is on).
pub fn apply_streaks(scene: &mut WorldScene, calls: &[ObjectStreakCall], visible: bool) {
    for call in calls {
        match *call {
            ObjectStreakCall::Create { emitter, spec } => {
                scene.start_object_streak(emitter, spec);
            }
            ObjectStreakCall::Activate { emitter, on } => {
                scene.set_object_streaks_active(emitter, on);
            }
            ObjectStreakCall::Gone { emitter } => scene.remove_object_streaks(emitter),
        }
    }
    scene.set_streaks_visible(visible);
}

/// The decals the objects' FX projected (`FXManager::DoDecalFX`), in order: each one takes the
/// ground under it as the scene has it when the decal starts.
pub fn apply_decals(scene: &mut WorldScene, calls: &[DecalSpec]) {
    for spec in calls {
        scene.start_object_decal(*spec);
    }
}

/// `CPhysicsObj::UpdateChild`: each FX object that hangs from an emitter stands where that
/// emitter's posed bone or holding point puts it this frame. The emitter's place is the one it
/// is drawn at; an emitter that is not drawn leaves its objects where they were.
pub fn follow_emitters(
    scene: &WorldScene,
    slots: &[FxSlot],
    frames: &mut [Frame],
    placed: &mut [Option<(u64, Mat4, Option<ObjectPose>)>],
) {
    let index: HashMap<u64, usize> = slots
        .iter()
        .enumerate()
        .map(|(i, slot)| (slot.id, i))
        .collect();
    for (i, child) in slots.iter().enumerate() {
        let Some(follow) = child.follow else {
            continue;
        };
        let Some(&emitter) = index.get(&follow.emitter) else {
            continue;
        };
        let Some((_, emitter_scale)) = scene.object_rig(slots[emitter].key) else {
            continue;
        };
        let Some(pose) = placed[emitter]
            .as_ref()
            .and_then(|placed| placed.2.as_ref())
        else {
            continue;
        };
        let Some(at) = follow_frame(
            frames[emitter],
            *emitter_scale,
            &pose.bones,
            &pose.named_bones,
            &follow.attach,
        ) else {
            continue;
        };
        frames[i] = at;
        let Some((_, scale)) = scene.object_rig(child.key) else {
            continue;
        };
        if let Some((key, _, pose)) = placed[i].take() {
            placed[i] = Some((key, world_matrix(*scale, at), pose));
        }
    }
}

/// Where each drawn object stands this frame, with its posed root bones, for the particle
/// systems it runs and the holding points they attach to.
pub fn object_placements(
    scene: &WorldScene,
    slots: &[FxSlot],
    frames: &[Frame],
    placed: &[Option<(u64, Mat4, Option<ObjectPose>)>],
) -> Vec<ObjectPlacement> {
    slots
        .iter()
        .zip(frames)
        .zip(placed)
        .filter_map(|((slot, frame), placed)| {
            let (rig, scale) = scene.object_rig(slot.key)?;
            let pose = placed.as_ref()?.2.as_ref();
            let bones = pose.map(|pose| pose.bones.clone()).unwrap_or_default();
            let named_bones = pose
                .map(|pose| pose.named_bones.clone())
                .unwrap_or_default();
            Some(ObjectPlacement {
                emitter: slot.id,
                cell: slot.cell,
                frame: *frame,
                scale: *scale,
                bones,
                named_bones,
                parents: rig.skeleton.root_parents(),
            })
        })
        .collect()
}
