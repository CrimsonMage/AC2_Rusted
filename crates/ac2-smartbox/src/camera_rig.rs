//! `CameraManager`'s camera rig and effects (Legions `cameramanager.cpp`): where behaviours and
//! camera effects move the camera's parent and target (`SetTargetedCamera`, `SetFreeCamera`,
//! `SetCameraAnimTarget` / `SetCameraAnimParent`, `RestorePlayerCamera`), `DoCameraFX`, and
//! `UseTime`, which runs the effects into the field of view, the screen flash, the motion blur
//! and the camera's shake.

use ac2_physics::{Position, Vector};

use crate::camera::{ld, st, CameraManager, CameraSubject};
use crate::camera_fx::{CameraFx, TrackKind};
use crate::targeting::{CameraView, CAMERA_PARENT_ID, CAMERA_TARGET_ID, INVALID_IID};

/// `CameraManager::UseTime`'s base field of view.
const BASE_FOV: f32 = 65.0;

impl CameraManager {
    /// `CameraManager::DoCameraFX(id, scalar, source)` for the loaded record: an effect for the
    /// player only does nothing from anyone else; otherwise its tracks start and it moves the
    /// camera rig (free, restore, parent and target, in that order) by its period and ramp-down
    /// time.
    pub fn do_camera_fx(&mut self, fx: &CameraFx, scalar: f32, source: u64, view: &CameraView<'_>) {
        if fx.player_only && source != self.targeting.player_id {
            return;
        }
        if fx.shake {
            self.fx.add(TrackKind::Shake, fx, source, scalar);
        }
        if fx.has_screen_flash {
            self.fx
                .add(TrackKind::ScreenFlash(fx.screen_flash), fx, source, scalar);
        }
        if fx.has_fov {
            self.fx.add(TrackKind::Fov, fx, source, scalar);
        }
        if fx.blur {
            self.fx.add(TrackKind::Blur, fx, source, scalar);
        }
        if fx.camera_distance {
            self.fx.add(TrackKind::CameraDistance, fx, source, scalar);
        }
        let (period, ramp) = (fx.period, fx.ramp_down_time);
        if fx.free_camera {
            self.set_free_camera(view, source, period, ramp);
        }
        if fx.restore_camera {
            self.restore_player_camera(view);
        }
        if fx.parent_camera {
            self.targeting
                .store_parent_pos(view, source, 0, period, ramp);
        }
        if fx.target_camera {
            self.set_target_object(view, source, period, ramp);
        }
    }

    /// `CameraManager::StopAllCameraFX`.
    pub fn stop_all_camera_fx(&mut self) {
        self.fx.stop_all();
    }

    /// `CameraManager::SetTargetObject`: the target becomes `id`. The first target the camera
    /// is given (it has no player yet) is the player: it becomes `m_playerID` and the targeting
    /// module's player, and the module takes the camera's distance. Retail also calls
    /// `SetupCameraDummyObjects` then, which makes the two dummy objects on the player's camera
    /// bones (`0x40000004`, `0x40000005`) and sets the effects' player id; here the dummies are
    /// those bones, found when asked ([`Self::dummy_exists`]), and the effects read the player
    /// from the targeting module.
    pub fn set_target_object(&mut self, view: &CameraView<'_>, id: u64, period: f32, ramp: f32) {
        self.targeting.store_target_pos(view, id, 0, period, ramp);
        if self.targeting.player_id == INVALID_IID {
            self.targeting.set_player(id);
            self.targeting.camera_distance = self.camera_distance;
        }
    }

    /// `CameraManager::SetParentObject`.
    pub fn set_parent_object(&mut self, view: &CameraView<'_>, id: u64, period: f32, ramp: f32) {
        self.targeting.store_parent_pos(view, id, 0, period, ramp);
    }

    /// `CameraManager::SetFreeCamera`: the camera stands at `root` with no target.
    pub fn set_free_camera(&mut self, view: &CameraView<'_>, root: u64, period: f32, ramp: f32) {
        self.targeting.store_parent_pos(view, root, 0, period, ramp);
        self.targeting
            .store_target_pos(view, INVALID_IID, 0, period, ramp);
    }

    /// `CameraManager::SetTargetedCamera`: the camera stands at `root`'s `parent_bone` looking at
    /// its `target_bone` (bone types; a behaviour on another object uses `0x40000002` and
    /// `0x40000001`).
    pub fn set_targeted_camera(
        &mut self,
        view: &CameraView<'_>,
        root: u64,
        (parent_bone, target_bone): (u32, u32),
        period: f32,
        ramp: f32,
    ) {
        self.targeting
            .store_parent_pos(view, root, parent_bone, period, ramp);
        self.targeting
            .store_target_pos(view, root, target_bone, period, ramp);
    }

    /// `CameraManager::RestorePlayerCamera`: back to the player with no parent, at once.
    pub fn restore_player_camera(&mut self, view: &CameraView<'_>) {
        self.targeting
            .store_parent_pos(view, INVALID_IID, 0, 1.0, 0.0);
        let player = self.targeting.player_id;
        self.targeting.store_target_pos(view, player, 0, 1.0, 0.0);
    }

    /// `CameraManager::SetCameraAnimTarget`: the target becomes the camera's target dummy (on
    /// the player's `CAMERA_TARGET_BONE`), when the player has that bone.
    pub fn set_camera_anim_target(&mut self, view: &CameraView<'_>, period: f32, ramp: f32) {
        if self.dummy_exists(view, CAMERA_TARGET_ID) {
            self.targeting
                .store_target_pos(view, CAMERA_TARGET_ID, 0, period, ramp);
        }
    }

    /// `CameraManager::SetCameraAnimParent`: as [`Self::set_camera_anim_target`] for the parent
    /// dummy.
    pub fn set_camera_anim_parent(&mut self, view: &CameraView<'_>, period: f32, ramp: f32) {
        if self.dummy_exists(view, CAMERA_PARENT_ID) {
            self.targeting
                .store_parent_pos(view, CAMERA_PARENT_ID, 0, period, ramp);
        }
    }

    /// `ICPhysicsObj::GetObjectA` of a camera dummy (`SetupCameraDummyObjects` made one for each
    /// of the player's camera bones).
    fn dummy_exists(&self, view: &CameraView<'_>, id: u64) -> bool {
        let bone = if id == CAMERA_PARENT_ID {
            crate::targeting::CAMERA_PARENT_BONE
        } else {
            crate::targeting::CAMERA_TARGET_BONE
        };
        let zero = Vector::new(0.0, 0.0, 0.0);
        view.objects
            .bone_position(self.targeting.player_id, bone, zero)
            .is_some()
    }

    /// `CameraManager::CheckEnvOnly`: in the map view, or on a target bone, the camera's line
    /// of sight checks only the environment.
    pub fn check_env_only(&self) -> bool {
        self.mapview || self.targeting.target_bone != 0
    }

    /// `CameraManager::GetTargetOffset`: the look-at offset plus the shoulder offset (asked
    /// without easing).
    pub fn get_target_offset(&mut self, player: Option<&CameraSubject>) -> Vector {
        let shoulder = self.calc_shoulder_offset(player, 0.0);
        let t = self.target_offset;
        Vector::new(t.x + shoulder.x, t.y + shoulder.y, t.z + shoulder.z)
    }

    /// `CameraManager::UseTime(res)` at local time `now`, with the player present: the effects
    /// advance by the time since the last call, the field of view becomes 65 plus their sum, and
    /// their shake moves `current` (the scene camera's current position). `distance(source)` is
    /// a source's distance from the player; `roll` is `Random::RollDice`.
    pub fn use_time(
        &mut self,
        now: f64,
        current: &mut Position,
        distance: &mut dyn FnMut(u64) -> Option<f64>,
        roll: &mut dyn FnMut(f32, f32) -> f32,
    ) {
        let last = *self.last_use_time.get_or_insert(now);
        self.fx.use_time((now - last) as f32);
        self.last_use_time = Some(now);
        self.fov = st(ld(self.fx.fov_sum) + ld(BASE_FOV));
        let shake = self.fx.distortion(distance, roll);
        let o = &mut current.frame.origin;
        o.x = st(ld(o.x) + ld(shake.x));
        o.y = st(ld(shake.y) + ld(o.y));
        o.z = st(ld(shake.z) + ld(o.z));
    }

    /// `SceneTool::m_cFlashColor` after `use_time`: the screen flash (r, g, b, a).
    pub fn flash_colour(&self) -> [f32; 4] {
        self.fx.flash_sum
    }

    /// `SceneTool::m_fBlurAmount` after `use_time`: the motion blur, 0 to 1.
    pub fn blur_amount(&self) -> f32 {
        self.fx.blur_sum
    }
}

#[cfg(test)]
#[path = "camera_rig_tests.rs"]
mod tests;
