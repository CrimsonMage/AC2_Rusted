//! `LookAtManager` (`portal/shared/src/physobj/lookat/lookatmanager.cpp`): the body turns its
//! head toward a target object, the weight ramping in over `ramp_time` and back out at the end
//! of the period, and gives the look up after two seconds of disinterest (the target standing
//! beyond 10 units off the body's front, or behind it). The head turns by two sliders the
//! `AMLook` animator reads: `0x40000007` about the body's x axis (up and down) and `0x40000009`
//! about its z axis (left and right).
//!
//! `LookAtLogic` aims both at the target, blended from the values the look began with by the
//! weight, and keeps the aim (`mLookAtValues`). With no target a body other than the player holds
//! its head direction (`GetLookAtX` / `GetLookAtZ`, the server's `LookAtDir`); the player's head
//! follows the camera (`CameraManager::GetInterpViewerOffset`), as the look-at-camera and
//! camera-FK switches allow, blended toward the last aim by the weight that ramps out.
//!
//! Admin 7148 addresses: `LookAt` 0x004FD780, `LookAtLogic` 0x004FD930 and `Update` 0x004FDB70
//! (both read from the disassembly; the decompile loses their slider arguments). Arithmetic is
//! x87: f64 intermediates, `f32` where retail stores a `float`.

use std::f32::consts::FRAC_PI_2;

use crate::fpu::{ld, st};
use crate::heading::Heading;
use crate::position::Position;

/// The head's turn about the body's x axis (up and down).
pub const LOOK_PITCH_SLIDER: u32 = 0x4000_0007;
/// The head's turn about the y axis (`LookAt` keeps its value; nothing here moves it).
pub const LOOK_ROLL_SLIDER: u32 = 0x4000_0008;
/// The head's turn about the body's z axis (left and right).
pub const LOOK_YAW_SLIDER: u32 = 0x4000_0009;

/// `M_PI_2_LOWER` / `M_PI_2_UPPER` (set by the static initializers `$E14` / `$E16`).
const PI_2_LOWER: f32 = 1.470_796_3;
const PI_2_UPPER: f32 = 1.670_796_4;
/// `2 / pi` and `pi / 2` as `Update` loads them.
const TWO_OVER_PI: f32 = 0.636_619_75;
const HALF_PI: f32 = 1.570_796_4;

/// The ramp `CPhysicsObj::LookAt` uses for both a new target and a release.
pub const LOOK_RAMP: f32 = 0.5;

/// What the look logic reads of the target (`GetObjectA(mTargetID)`: `GetPosition`,
/// `GetHeight` and `is_mobile`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LookTarget {
    pub position: Position,
    pub height: f32,
    pub mobile: bool,
}

/// What the look logic reads of the body itself.
#[derive(Debug, Clone, Copy)]
pub struct LookSelf {
    pub position: Position,
    /// `GetHeight` of the body (the eye line is this height).
    pub height: f32,
    /// `IsThePlayer`.
    pub is_player: bool,
    /// `CPhysicsObj::GetLookAtX` / `GetLookAtZ`: the head direction a server `LookAtDir` set.
    pub head_x: f32,
    pub head_z: f32,
    /// `GetTotalScale`'s x and z.
    pub scale_x: f32,
    pub scale_z: f32,
    /// `GetSliderCurrentValue` of the pitch and yaw sliders.
    pub pitch: f32,
    pub yaw: f32,
    /// The camera, for the player (`CameraManager::GetCameraManager`; `None` when there is none).
    pub camera: Option<LookCamera>,
}

/// What `Update` reads of the camera for the player's head.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LookCamera {
    /// `CameraManager::GetInterpViewerOffset`.
    pub viewer_offset: crate::vector::Vector,
    /// `CameraManager::m_mouseLook`: the offset is taken into the body's frame.
    pub mouse_look: bool,
}

/// A slider the manager wants moved: `DoSlider(0, slider, value, time)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderMove {
    pub slider: u32,
    pub value: f32,
    pub time: f32,
}

/// The manager's state (`LookAtManager`'s fields). A target id of 0 is `INVALID_IID`.
#[derive(Debug, Clone, PartialEq)]
pub struct LookAtManager {
    /// `mParentID`: the body's own id.
    pub parent: u64,
    /// `mTargetID`.
    pub target: u64,
    /// `mRampTime`.
    pub ramp_time: f32,
    /// `mPeriod`.
    pub period: f32,
    /// `mTime`.
    pub time: f32,
    /// `mWeight`.
    pub weight: f32,
    /// `mWeightVel`.
    pub weight_vel: f32,
    /// `mTimeDisinterested`.
    pub time_disinterested: f32,
    /// `m_bIsOn`.
    pub is_on: bool,
    /// `m_bHold`: a target holds the look for the whole period.
    pub hold: bool,
    /// `mPreviousSliderValues`: the three look sliders when the look began, blended away as the
    /// weight rises.
    pub previous: [f32; 3],
    /// `mLookAtValues.x` and `.z`: the last aim `LookAtLogic` took (up-down, left-right).
    pub look_values: (f32, f32),
    /// `m_bLookAtCamera` and `m_bCameraFK` (both on from the constructor): the player's head
    /// turns toward the camera, and follows it at all.
    pub look_at_camera: bool,
    pub camera_fk: bool,
}

impl Default for LookAtManager {
    /// `LookAtManager::LookAtManager`: ramp 0.5, period 1.5, weight velocity 2, off.
    fn default() -> Self {
        Self {
            parent: 0,
            target: 0,
            ramp_time: 0.5,
            period: 1.5,
            time: 0.0,
            weight: 0.0,
            weight_vel: 2.0,
            time_disinterested: 0.0,
            is_on: false,
            hold: false,
            previous: [0.0; 3],
            look_values: (0.0, 0.0),
            look_at_camera: true,
            camera_fk: true,
        }
    }
}

impl LookAtManager {
    /// `LookAtManager::Release(ramp)`: the look ends; the weight ramps back to 0 over `ramp`
    /// (at once when `ramp` is 0).
    pub fn release(&mut self, ramp: f32) {
        self.target = 0;
        self.ramp_time = ramp;
        self.is_on = false;
        self.hold = false;
        if ramp == 0.0 {
            self.weight = 1.0;
            self.weight_vel = 0.0;
        } else {
            self.weight = 0.0;
            self.weight_vel = 1.0 / ramp;
        }
    }

    /// `LookAtManager::LookAt(target, ramp)`: a new target starts the look (and turns it on);
    /// `previous` is the three look sliders at that moment. A target equal to the body itself
    /// is no look (`CPhysicsObj::LookAt` releases instead); a target already being looked at
    /// keeps the look on without restarting its ramp.
    pub fn look_at(&mut self, target: u64, ramp: f32, previous: [f32; 3]) {
        if target == 0 || target == self.parent {
            return;
        }
        self.is_on = true;
        if self.target == 0 || self.parent != target {
            self.previous = previous;
            self.target = target;
            self.ramp_time = ramp;
            self.time = 0.0;
            self.period = 1.0;
            self.hold = true;
            if ramp == 0.0 {
                self.weight = 1.0;
                self.weight_vel = 0.0;
            } else {
                self.weight = 0.0;
                self.weight_vel = 1.0 / ramp;
            }
        }
    }

    /// `LookAtManager::LookAtDir`: a head direction turns the look on for half a second when no
    /// target holds it.
    pub fn look_at_dir(&mut self) {
        self.is_on = true;
        if self.target == 0 {
            self.time = 0.0;
            self.period = 0.5;
        }
    }

    /// `LookAtManager::Update(quantum)`: advances the ramp and the period, then aims the head.
    /// Returns the slider moves to make (in order).
    pub fn update(
        &mut self,
        quantum: f32,
        me: &LookSelf,
        target: Option<LookTarget>,
    ) -> Vec<SliderMove> {
        let mut moves = Vec::new();
        if !me.is_player && !self.is_on {
            return moves;
        }
        self.time += quantum;
        let ends = self.period - self.ramp_time;
        if ends < self.time && self.is_on {
            if self.hold {
                self.time = ends;
            } else {
                self.is_on = false;
                self.target = 0;
            }
        }
        self.weight = self.ramp_weight(quantum);
        self.weight = self.weight.clamp(0.0, 1.0);

        if let Some(target) = target.filter(|_| self.target != self.parent) {
            self.look_logic(quantum, me, &target, &mut moves);
            if !me.is_player || self.time_disinterested <= 2.0 {
                return moves;
            }
            // `CPhysicsObj::LookAt(INVALID_IID)`: the disinterested player stops looking.
            self.release(LOOK_RAMP);
            return moves;
        }
        if !me.is_player {
            // A non-player holds its head direction over half a second.
            moves.push(SliderMove {
                slider: LOOK_PITCH_SLIDER,
                value: me.head_x,
                time: 0.5,
            });
            moves.push(SliderMove {
                slider: LOOK_YAW_SLIDER,
                value: me.head_z,
                time: 0.5,
            });
            return moves;
        }
        self.time_disinterested = 0.0;
        if let Some(camera) = me.camera {
            self.follow_camera(me, &camera, &mut moves);
        }
        moves
    }

    /// `Update`'s player branch: the head follows the camera. The offset from the viewer (in the
    /// body's frame under mouse look) gives the left-right angle by its heading and the up-down by
    /// its height; the switches fold or clear them; each is scaled by the body, blended toward
    /// the last aim by the weight and moved over a tenth of a second (a fifth for a step over
    /// 0.2).
    fn follow_camera(&self, me: &LookSelf, camera: &LookCamera, moves: &mut Vec<SliderMove>) {
        let offset = if camera.mouse_look {
            me.position.frame.global_to_local_vec(camera.viewer_offset)
        } else {
            camera.viewer_offset
        };
        let dir_x = -offset.x;
        let dir_y = -ld(offset.y);
        let dir_z = -offset.z;
        let length = (ld(dir_x) * ld(dir_x) + dir_y * dir_y).sqrt();
        let heading = if length < f64::from(0.0002_f32) {
            Heading::ZERO
        } else {
            let inverse = 1.0 / length;
            let x = st(ld(dir_x) * inverse);
            let angle = (inverse * dir_y).atan2(ld(x));
            Heading::from_radians(st(7.853_981_633_974_483 - angle))
        };
        let mut left_right = st(-heading.radians_0_centered());
        let mut up_down = dir_z;
        let in_window = |a: f32| PI_2_LOWER < a && a < PI_2_UPPER;
        let fold = |a: f32, by: f32| st((ld(a) * ld(TWO_OVER_PI) - ld(HALF_PI)) * ld(by));
        if !self.camera_fk {
            up_down = 0.0;
            left_right = 0.0;
        } else if !self.look_at_camera {
            if !(-HALF_PI <= left_right && left_right <= HALF_PI) {
                left_right = 0.0;
            }
        } else {
            if in_window(left_right) {
                up_down = fold(left_right, dir_z);
            } else if PI_2_UPPER < left_right || left_right < -PI_2_LOWER {
                up_down = st(-ld(dir_z));
            }
            if in_window(left_right) {
                left_right = fold(left_right, left_right);
            } else if PI_2_UPPER < left_right {
                left_right = st(-(ld(std::f32::consts::PI) - ld(left_right)));
            } else if left_right < -PI_2_LOWER {
                left_right = st(ld(left_right) + ld(std::f32::consts::PI));
            }
        }
        let rest = 1.0 - ld(self.weight);
        let w = ld(self.weight);
        let pitch = st(rest / ld(me.scale_z) * ld(up_down) + ld(self.look_values.0) * w);
        let yaw = st(rest / ld(me.scale_x) * ld(left_right) + ld(self.look_values.1) * w);
        let time = |current: f32, value: f32| {
            if f64::from(0.2_f32) < (ld(current) - ld(value)).abs() {
                0.2
            } else {
                0.1
            }
        };
        moves.push(SliderMove {
            slider: LOOK_PITCH_SLIDER,
            value: pitch,
            time: time(me.pitch, pitch),
        });
        moves.push(SliderMove {
            slider: LOOK_YAW_SLIDER,
            value: yaw,
            time: time(me.yaw, yaw),
        });
    }

    /// The weight `Update` sets from the time: ramping in before `ramp_time`, full between the
    /// ramps, ramping out over the last `ramp_time` of the period.
    fn ramp_weight(&self, quantum: f32) -> f32 {
        let ramp = self.ramp_time;
        if ramp <= self.time {
            if self.time <= ramp || self.period - ramp <= self.time {
                if self.time <= self.period - ramp {
                    quantum * self.weight_vel + self.weight
                } else {
                    (self.period - self.time) / ramp
                }
            } else {
                1.0
            }
        } else {
            self.time / ramp
        }
    }

    /// `LookAtManager::LookAtLogic`: aims the head at the target. The eye line is the target's
    /// height (half of it, three quarters for a mobile target) above its position, less the
    /// body's own height; the yaw is the bearing of the target in the body's frame, blended
    /// from the yaw the look began with by the weight. A target beyond 10 units to the side or
    /// behind the body, or past 160 units, counts toward disinterest.
    fn look_logic(
        &mut self,
        quantum: f32,
        me: &LookSelf,
        target: &LookTarget,
        moves: &mut Vec<SliderMove>,
    ) {
        let mut dir = me.position.offset_to(&target.position);
        let eye = if target.mobile { 0.75 } else { 0.5 };
        dir.z += target.height * eye;
        let mut local = me.position.frame.global_to_local_vec(dir);
        let horizontal = st((ld(local.x) * ld(local.x) + ld(local.y) * ld(local.y)).sqrt());
        if local.y < 0.0 || horizontal > 10.0 {
            self.time_disinterested += quantum;
        } else if horizontal <= 160.0 {
            self.time_disinterested = 0.0;
        } else {
            self.time_disinterested = 3.0;
        }
        local.z = st(ld(local.z) - ld(me.height));
        if local.y < 0.0 {
            local.y = 0.0;
        }
        let yaw = st(ld(local.y).atan2(ld(local.x)) - ld(FRAC_PI_2));
        let w = ld(self.weight);
        let rest = 1.0 - w;
        moves.push(SliderMove {
            slider: LOOK_YAW_SLIDER,
            value: st(rest * ld(self.previous[2]) + ld(yaw) * w),
            time: 0.0,
        });
        // The up-down share: the eye line's rise over the distance, divided by the weight.
        let rise = if horizontal != 0.0 && self.weight != 0.0 {
            ld(local.z) / (ld(horizontal) * w)
        } else {
            0.0
        };
        moves.push(SliderMove {
            slider: LOOK_PITCH_SLIDER,
            value: st(rest * ld(self.previous[0]) + rise),
            time: 0.0,
        });
        let pitch = if horizontal == 0.0 {
            0.0
        } else {
            st(ld(local.z) / ld(horizontal))
        };
        self.look_values = (pitch, yaw);
    }
}

#[cfg(test)]
#[path = "look_at_tests.rs"]
mod tests;
