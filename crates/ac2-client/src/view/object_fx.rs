//! The FX the objects make drawn in the scene: the particle systems, lights and streaks their FX
//! start and stop (`FXManager::DoParticleFX`, `DoLightSourceFX`, `DoStreakFX`), and where each
//! drawn object stands for the systems it runs.
use ac2_math::{Frame, Mat4};
use ac2_render::object::ObjectPose;
use ac2_render::scene::ObjectPlacement;

use super::{ClientView, Drawn};
use crate::fx_particles::{ObjectLightStart, ObjectParticleStart};
use crate::fx_streaks::ObjectStreakCall;

impl ClientView {
    /// The particle systems the objects' FX started (`FXManager::DoParticleFX`) and stopped
    /// (`StopFX`, or their object left), in that order: the scene loads each description on its
    /// loader thread and runs the system where its object stands.
    pub fn set_object_particles(&mut self, starts: &[ObjectParticleStart], stops: &[u64]) {
        for start in starts {
            self.scene.start_object_particle(
                start.handle,
                start.emitter,
                start.desc,
                start.anchor,
                !start.fire_and_forget,
            );
        }
        for handle in stops {
            self.scene.stop_object_particle(*handle);
        }
    }

    /// The lights the objects' FX placed (`DoLightSourceFX`) and the objects that left the world
    /// (`DestroyAllTransientFX`): the scene's point lights reach the objects it draws.
    pub fn set_object_lights(&mut self, starts: &[ObjectLightStart], gone: &[u64]) {
        for start in starts {
            self.scene.start_object_light(start.emitter, start.light);
        }
        for emitter in gone {
            self.scene.remove_object_lights(*emitter);
        }
    }

    /// The streaks the objects' FX made, started, stopped and destroyed (`FXManager::DoStreakFX`,
    /// `StartStreak`, `StopStreak`), in order; `visible` is `StreakEffects` (the ribbons draw only
    /// while it is on).
    pub fn set_object_streaks(&mut self, calls: &[ObjectStreakCall], visible: bool) {
        for call in calls {
            match *call {
                ObjectStreakCall::Create { emitter, spec } => {
                    self.scene.start_object_streak(emitter, spec);
                }
                ObjectStreakCall::Activate { emitter, on } => {
                    self.scene.set_object_streaks_active(emitter, on);
                }
                ObjectStreakCall::Gone { emitter } => self.scene.remove_object_streaks(emitter),
            }
        }
        self.scene.set_streaks_visible(visible);
    }

    /// Where each drawn object stands this frame, with its posed root bones, for the particle
    /// systems it runs and the holding points they attach to.
    pub(super) fn object_placements(
        &self,
        drawn: &[Drawn<'_>],
        frames: &[Frame],
        placed: &[Option<(u64, Mat4, Option<ObjectPose>)>],
    ) -> Vec<ObjectPlacement> {
        drawn
            .iter()
            .zip(frames)
            .zip(placed)
            .filter_map(|((drawn, frame), placed)| {
                let (_, scale) = self.scene.object_rig(drawn.key)?;
                let pose = placed.as_ref()?.2.as_ref();
                let bones = pose.map(|pose| pose.bones.clone()).unwrap_or_default();
                let named_bones = pose
                    .map(|pose| pose.named_bones.clone())
                    .unwrap_or_default();
                Some(ObjectPlacement {
                    emitter: drawn.id,
                    cell: drawn.cell,
                    frame: *frame,
                    scale: *scale,
                    bones,
                    named_bones,
                })
            })
            .collect()
    }
}
