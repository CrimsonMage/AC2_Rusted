//! `LookAtManager` (`portal/shared/src/physobj/lookat/lookatmanager.cpp`): the body turns its
//! head toward a target object, the weight ramping in over `ramp_time` and back out at the end
//! of the period, and gives the look up after two seconds of disinterest (the target standing
//! beyond 160 units off the body's front, or the body facing away). The body's yaw slider
//! `0x40000007` follows the target; with no target a non-player body holds its head direction
//! (`CPhysicsObj::LookAtDir`). Retail's camera-driven head for the local player is not modelled.
//!
//! Arithmetic follows the decompile's order, in `f32` (the recovery export drops the x87 terms
//! `LookAtLogic` keeps in registers; see the known gaps in `docs/retail-catalog`).

use std::f32::consts::FRAC_PI_2;

use crate::position::Position;

/// `TargetID` of the body's yaw slider (`LookAtLogic` writes it).
pub const LOOK_YAW_SLIDER: u32 = 0x4000_0007;

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
    /// `CPhysicsObj::m_fHeadX`, the head direction a server `LookAtDir` set.
    pub head_x: f32,
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
    /// `mPreviousSliderValues.x`: the yaw slider when the look began, blended away as the
    /// weight rises.
    pub previous_yaw: f32,
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
            previous_yaw: 0.0,
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
    /// `previous_yaw` is the yaw slider's value at that moment. A target equal to the body
    /// itself, or already being looked at, only keeps the look on.
    pub fn look_at(&mut self, target: u64, ramp: f32, previous_yaw: f32) {
        if target == 0 {
            return;
        }
        self.is_on = true;
        if self.target == 0 || self.parent != target {
            self.previous_yaw = previous_yaw;
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
        if me.is_player {
            // The local player's head follows the camera (`CameraManager::GetInterpViewerOffset`),
            // which the client does not model yet.
            self.time_disinterested = 0.0;
            return moves;
        }
        // A non-player holds its head direction (`LookAtManager::Update`'s no-target branch).
        moves.push(SliderMove {
            slider: LOOK_YAW_SLIDER,
            value: me.head_x,
            time: 0.5,
        });
        moves
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
        let horizontal = (local.x * local.x + local.y * local.y).sqrt();
        if local.y < 0.0 || horizontal > 10.0 {
            self.time_disinterested += quantum;
        } else if horizontal <= 160.0 {
            self.time_disinterested = 0.0;
        } else {
            self.time_disinterested = 3.0;
        }
        local.z -= me.height;
        if local.y < 0.0 {
            local.y = 0.0;
        }
        let yaw = local.y.atan2(local.x) - FRAC_PI_2;
        let blended = (1.0 - self.weight) * self.previous_yaw + self.weight * yaw;
        moves.push(SliderMove {
            slider: LOOK_YAW_SLIDER,
            value: blended,
            time: 0.0,
        });
    }
}

#[cfg(test)]
#[path = "look_at_tests.rs"]
mod tests;
