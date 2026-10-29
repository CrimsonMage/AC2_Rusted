//! The camera's subject and its update (`SmartBox::UpdateCamera`): the player's physics object as
//! the camera manager reads it, at its current quantum or where it is drawn this frame. And the
//! camera's reach into the world (`CameraManager`'s `ICPhysicsObj::GetObjectA` lookups): the
//! objects and bones behaviours and camera effects point the camera at, the player's behaviour
//! cameras (`PHYSICSOBJREP::SetCameraAnimTarget`, `SetCameraAnimParent`, `RestoreCamera`,
//! `SetTargetedCamera`), the FX nodes' camera effects (`FXManager::DoCameraFX`, read on a loader
//! thread), and `CameraManager::UseTime`'s step after the scene camera's slerp.

use ac2_anim::Ac2AnimEvent;
use ac2_math::{Mat4, Vec3};
use ac2_physics::{Frame, PhysicsBody, Position, Quaternion, Vector};
use ac2_smartbox::camera_fx::CameraFx;
use ac2_smartbox::targeting::{CameraObjects, CameraView};
use ac2_smartbox::CameraSubject;

use crate::client::{Client, Player};
use crate::loader::Request;
use crate::remote::Remotes;

/// The mode in which `AvatarVisibleInFirstPerson` 2 hides the avatar (`GetMode() == 0x40000001`).
const AVATAR_HIDDEN_MODE: u32 = 0x4000_0001;

/// The camera's lookup over the player's and the other objects' bodies.
pub(crate) struct CameraWorld<'a> {
    pub(crate) player: Option<&'a Player>,
    pub(crate) remotes: &'a Remotes,
}

impl CameraWorld<'_> {
    fn body(&self, id: u64) -> Option<&PhysicsBody> {
        match self.player.filter(|p| p.id == id) {
            Some(player) => player.body.as_ref().map(|body| &body.physics),
            None => self.remotes.get(id)?.body(),
        }
    }
}

impl CameraObjects for CameraWorld<'_> {
    fn subject(&self, id: u64) -> Option<CameraSubject> {
        let body = self.remotes.get(id)?.body()?;
        let position = body.position();
        let state = body.movement_state();
        let [vx, vy, vz] = state.velocity;
        Some(CameraSubject {
            id,
            position,
            height: f64::from(body.height()),
            radius: f64::from(body.radius()),
            scale_z: body.own_total_scale().z,
            local_velocity: position.frame.global_to_local_vec(Vector::new(vx, vy, vz)),
            element: state.element,
            no_draw: false,
        })
    }

    /// `CPhysicsObj::GetBonePosition`: the bone type's matrix (`GetBoneTypeMatrix`) after the
    /// offset, combined into the object's frame at its total scale (`Frame::combine`).
    fn bone_position(&self, id: u64, bone: u32, offset: Vector) -> Option<Position> {
        let body = self.body(id)?;
        let to_bone = body.bone_type_matrix(bone)?;
        let relative = to_bone * Mat4::from_translation(Vec3::new(offset.x, offset.y, offset.z));
        let (_, rotation, translation) = relative.to_scale_rotation_translation();
        let position = body.position();
        let frame = Frame::combine_scaled(
            &position.frame,
            Vector::new(translation.x, translation.y, translation.z),
            Quaternion::new(rotation.w, rotation.x, rotation.y, rotation.z),
            body.own_total_scale(),
        );
        Some(Position::new(position.cell_id, frame))
    }
}

/// `AvatarVisibleInFirstPerson` `level` for the player's mode: 0 never, 1 always, 2 unless the
/// player is in mode `0x40000001`.
fn avatar_shown(level: u8, mode: Option<u32>) -> bool {
    match level {
        0 => false,
        2 => mode != Some(AVATAR_HIDDEN_MODE),
        _ => true,
    }
}

impl Client {
    /// What the camera reads from the player's physics object.
    pub(crate) fn camera_subject(&self, require_in_world: bool) -> Option<CameraSubject> {
        self.camera_subject_at(require_in_world, false)
    }

    /// The camera's subject at the body's current quantum, or with `drawn` where the body is
    /// drawn this frame (`PhysicsBody::drawing_position` at the render interpolation; the
    /// mouse-look camera's, `AGENTS.md`).
    fn camera_subject_at(&self, require_in_world: bool, drawn: bool) -> Option<CameraSubject> {
        let player = self.player.as_ref()?;
        if require_in_world && !player.in_world {
            return None;
        }
        let body = player.body.as_ref()?;
        let position = if drawn {
            body.physics.drawing_position(self.render_interpolation)
        } else {
            body.physics.position()
        };
        let state = body.physics.movement_state();
        let [vx, vy, vz] = state.velocity;
        Some(CameraSubject {
            id: player.id,
            position,
            height: body.height,
            radius: body.radius,
            scale_z: body.scale,
            local_velocity: position.frame.global_to_local_vec(Vector::new(vx, vy, vz)),
            element: state.element,
            no_draw: self.session.player().is_some_and(|id| self.is_no_draw(id)),
        })
    }

    /// `AvatarVisibleInFirstPerson` (`SceneOptions::avatar_first_person`).
    pub fn set_avatar_first_person(&mut self, level: u8) {
        self.avatar_first_person = level;
    }

    /// Whether the avatar is shown in first person: the preference is not "never", and at 2 the
    /// player is not in mode `0x40000001` (`CPhysicsObj`'s draw, `CameraManager::UpdateCamera`).
    fn avatar_shown(&self) -> bool {
        let mode = self
            .player
            .as_ref()
            .and_then(|p| p.body.as_ref())
            .map(|b| b.physics.mode());
        avatar_shown(self.avatar_first_person, mode)
    }

    /// `CPhysicsObj`'s draw: with the camera in first person (`GetFirstPersonIsReady`) and the
    /// avatar not shown, the player and what it holds are not drawn.
    pub(crate) fn avatar_hidden(&self) -> bool {
        self.camera.first_person_ready() && !self.avatar_shown()
    }

    /// `SmartBox::UpdateCamera`. Without `interpolate` (the mouse-look camera, every frame) the
    /// subject is where the body is drawn, so camera and body move together between quanta. The
    /// camera can follow the other objects too ([`CameraWorld`]).
    pub(crate) fn update_camera(&mut self, quantum: f32, interpolate: bool) {
        let Some(subject) = self.camera_subject_at(true, !interpolate) else {
            return;
        };
        // `UpdateCamera`'s dynamic first-person test: the avatar shown and a behaviour flagged
        // `DYNAMIC_FP_CAMERA` playing on the player.
        let dynamic = self
            .player
            .as_ref()
            .and_then(|p| p.body.as_ref())
            .is_some_and(|b| b.physics.check_for_dynamic_fp_camera());
        self.camera.dynamic_first_person = dynamic && self.avatar_shown();
        let world = CameraWorld {
            player: self.player.as_ref(),
            remotes: &self.remotes,
        };
        self.scene
            .update_with(&mut self.camera, &subject, &world, quantum, interpolate);
    }

    /// The player's behaviour cameras among this step's animation events
    /// (`BehaviorTracker::ProcessParamFlags` sends them for the player only). A targeted camera
    /// first has its target play the behaviour (`CPhysicsObj::DoBehaviorOnObject`:
    /// `GetObjectA(target)->DoBehavior(behaviorID)`, default `BehaviorParams` for the id); only
    /// when that starts does the camera stand at the target's bone type `0x40000002` looking at its
    /// `0x40000001`, ramping in over half a second.
    pub(crate) fn camera_animation_events(
        &mut self,
        events: &[(u64, Option<u32>, Vec<Ac2AnimEvent>)],
    ) {
        let Some(player_id) = self.player.as_ref().map(|p| p.id) else {
            return;
        };
        let player_events: Vec<Ac2AnimEvent> = events
            .iter()
            .filter(|(id, _, _)| *id == player_id)
            .flat_map(|(_, _, events)| events.iter().copied())
            .collect();
        for event in player_events {
            if let Ac2AnimEvent::TargetedCamera {
                target, behavior, ..
            } = event
            {
                let params = ac2_anim::Ac2BehaviorParams::new(behavior);
                let took = self
                    .script_body_mut(target)
                    .is_some_and(|body| body.do_behavior(&params));
                if !took {
                    continue;
                }
            }
            let Some(subject) = self.camera_subject(false) else {
                return;
            };
            let world = CameraWorld {
                player: self.player.as_ref(),
                remotes: &self.remotes,
            };
            let view = CameraView {
                player: Some(&subject),
                objects: &world,
            };
            let camera = &mut self.camera;
            match event {
                Ac2AnimEvent::CameraAnimParent { period, ramp_time } => {
                    camera.set_camera_anim_parent(&view, period, ramp_time)
                }
                Ac2AnimEvent::CameraAnimTarget { period, ramp_time } => {
                    camera.set_camera_anim_target(&view, period, ramp_time)
                }
                Ac2AnimEvent::RestoreCamera => camera.restore_player_camera(&view),
                Ac2AnimEvent::TargetedCamera { target, period, .. } => {
                    let bones = (0x4000_0002, 0x4000_0001);
                    camera.set_targeted_camera(&view, target, bones, period, 0.5);
                }
                _ => {}
            }
        }
    }

    /// `FXManager::DoCameraFX(id, scalar, source)`: the record is read on a loader thread and
    /// played when it arrives ([`Self::camera_fx_loaded`]).
    pub(crate) fn request_camera_fx(&mut self, did: u32, scalar: f32, source: u64) {
        if did != 0 {
            self.loader
                .request(Request::CameraFx(did, scalar.to_bits(), source));
        }
    }

    /// A camera effect's record arrived: `CameraManager::DoCameraFX` plays it. A missing record
    /// does nothing (`IError::ReportMissingDataFrom`).
    pub(crate) fn camera_fx_loaded(
        &mut self,
        source: u64,
        scalar: f32,
        fx: Result<Option<CameraFx>, String>,
    ) {
        let Ok(Some(fx)) = fx else {
            return;
        };
        let Some(subject) = self.camera_subject(false) else {
            return;
        };
        let world = CameraWorld {
            player: self.player.as_ref(),
            remotes: &self.remotes,
        };
        let view = CameraView {
            player: Some(&subject),
            objects: &world,
        };
        self.camera.do_camera_fx(&fx, scalar, source, &view);
    }

    /// `SmartBox::UseTime`'s `CameraManager::UseTime(res)` after the slerp, with the player
    /// present: the effects run, the field of view follows them, and their shake moves the
    /// scene camera's current position.
    pub(crate) fn camera_use_time(&mut self) {
        let Some(player) = self.player.as_ref().filter(|p| p.body.is_some()) else {
            return;
        };
        let now = self.session.local_time();
        let at = player.position();
        let world = CameraWorld {
            player: self.player.as_ref(),
            remotes: &self.remotes,
        };
        let mut distance = |source: u64| -> Option<f64> {
            let from = match world.body(source) {
                Some(body) => body.position(),
                None => return None,
            };
            Some(at.distance(&from))
        };
        let random = &mut self.random;
        let mut roll = |lo: f32, hi: f32| random.roll_dice_f32(lo, hi);
        self.camera
            .use_time(now, &mut self.scene.current, &mut distance, &mut roll);
        self.scene.fov = self.camera.fov();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_person_avatar_shows_by_the_preference_and_the_players_mode() {
        let other = Some(0x4000_0002);
        assert!(!avatar_shown(0, other));
        assert!(avatar_shown(1, Some(AVATAR_HIDDEN_MODE)));
        assert!(!avatar_shown(2, Some(AVATAR_HIDDEN_MODE)));
        assert!(avatar_shown(2, other));
        assert!(avatar_shown(2, None));
    }
}
