//! The animation side of `CMotionInterp`: the behaviours `UpdateMovement` starts and stops on
//! its object (move, falls, the run-stop skid and sliding down a slope), the mode switch for
//! diving into deep water, and the `HitGround`/`LeaveGround` landing and jump FX.

use crate::fpu::{ld, st, F_EPSILON};
use crate::motion_interp::{MotionContext, MotionInterp, ELEMENT_GROUND};
use crate::vector::Vector;

/// Behaviours `UpdateMovement` drives.
pub(crate) const BEHAVIOR_MOVE: u32 = 0x4000_0001;
/// A long fall: more than 50 m above walkable ground, or 10 m falling faster than 19 m/s.
pub(crate) const BEHAVIOR_LONG_FALL: u32 = 0x4000_0009;
/// Falling within 1.3 m of walkable ground.
pub(crate) const BEHAVIOR_NEAR_LANDING: u32 = 0x4000_000b;
/// Stopping out of a run of more than six seconds.
pub(crate) const BEHAVIOR_RUN_STOP: u32 = 0x4000_000c;
/// Sliding down a slope too steep to stand on.
pub(crate) const BEHAVIOR_SLIDE: u32 = 0x4000_000d;
/// The mode a fall into deep enough water switches to.
const MODE_DIVE: u32 = 0x4000_0002;
const MODE_SWIM: u32 = 0x4000_0003;
/// FX ids.
pub(crate) const FX_LAND: u32 = 0x4000_0002;
pub(crate) const FX_JUMP: u32 = 0x4000_0003;
pub(crate) const FX_SKID: u32 = 0x4000_0004;

/// The `PHYSICSOBJREP` animation calls `CMotionInterp` makes.
pub(crate) trait MotionRep {
    fn do_autonomous_behavior(&mut self, behavior: u32);
    fn is_doing_behavior(&self, behavior: u32) -> bool;
    fn stop_behavior(&mut self, behavior: u32);
    fn do_mode(&mut self, context: u32, mode: u32);
    /// `GetMode`: the mode tracker's desired mode.
    fn desired_mode(&self) -> u32;
    fn do_fx(&mut self, fx: u32, intensity: f32);
}

/// The running timers of the behaviour block (`mTimeRunning`, `mTimeSinceLastSkid`,
/// `mTimeSinceJump`, `mTimeSinceLand`), kept in doubles as retail does.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct BehaviorTimers {
    running: f64,
    since_skid: f64,
    since_jump: f64,
    since_land: f64,
}

impl MotionInterp {
    /// The `mTimeSinceJump`/`mTimeSinceLand` advance at the top of `UpdateMovement`.
    pub(crate) fn advance_jump_timers(&mut self, quantum: f32) {
        let timers = self.timers_mut();
        timers.since_jump += f64::from(quantum);
        timers.since_land += f64::from(quantum);
    }

    /// `CMotionInterp::HitGround`: the landing FX, its intensity the vertical speed over 6.5 m/s
    /// clamped to `[0.2, 1]`, at most every 0.1 s.
    pub(crate) fn hit_ground(&mut self, local_velocity: Vector, rep: &mut dyn MotionRep) {
        let scaled = (ld(local_velocity.z) * ld(0.153_846_16_f32)).abs();
        let mut intensity = st(scaled);
        if scaled < ld(0.2_f32) {
            intensity = 0.2;
        } else if ld(intensity) > 1.0 {
            intensity = 1.0;
        }
        if self.timers_mut().since_land > f64::from(0.1_f32) {
            rep.do_fx(FX_LAND, intensity);
            self.timers_mut().since_land = 0.0;
        }
    }

    /// `CMotionInterp::LeaveGround`: the jump FX (vertical speed over 6.28 m/s), at most every
    /// 0.1 s.
    pub(crate) fn leave_ground(&mut self, local_velocity: Vector, rep: &mut dyn MotionRep) {
        if self.timers_mut().since_jump > f64::from(0.1_f32) {
            let intensity = st((ld(local_velocity.z) * ld(0.159_235_66_f32)).abs());
            rep.do_fx(FX_JUMP, intensity);
            self.timers_mut().since_jump = 0.0;
        }
    }

    /// The `mCanAnimate` block of `CMotionInterp::UpdateMovement`.
    pub(crate) fn update_behaviors(
        &mut self,
        quantum: f32,
        ctx: &MotionContext,
        rep: &mut dyn MotionRep,
    ) {
        let v = ctx.local_velocity;
        let speed2 = st((ld(v.z) * ld(v.z) + ld(v.x) * ld(v.x)) + ld(v.y) * ld(v.y));
        let height = ld(ctx.height_above_walkable);
        let w = ctx.omega;
        let omega2 = (ld(w.z) * ld(w.z) + ld(w.y) * ld(w.y)) + ld(w.x) * ld(w.x);
        if ld(speed2) > ld(F_EPSILON) || omega2 > 0.0 {
            rep.do_autonomous_behavior(BEHAVIOR_MOVE);
        } else if rep.is_doing_behavior(BEHAVIOR_MOVE) {
            rep.stop_behavior(BEHAVIOR_MOVE);
        }
        if height != 0.0 {
            self.falling_behaviors(ctx, height, rep);
        }
        let q = f64::from(quantum);
        let mut skidding = false;
        if ctx.is_sliding_down && ld(speed2) > 2.0 && ctx.desired_element == ELEMENT_GROUND {
            skidding = true;
            let since = self.timers_mut().since_skid;
            if since > 1.0 || since == 0.0 {
                rep.do_fx(FX_SKID, speed2);
                self.timers_mut().since_skid = 0.0;
            }
            self.timers_mut().since_skid += q;
            rep.do_autonomous_behavior(BEHAVIOR_SLIDE);
        } else if rep.is_doing_behavior(BEHAVIOR_SLIDE) {
            rep.stop_behavior(BEHAVIOR_SLIDE);
        }
        if ld(speed2) > 2.0 {
            self.timers_mut().running += q;
        } else {
            self.timers_mut().running = 0.0;
        }
        let forward = ld(v.y);
        let desired_forward = ld(self.local_desired_velocity().y);
        let ran_long = self.timers_mut().running > 6.0;
        if forward > 1.5 && desired_forward <= 0.0 && ran_long && height == 0.0 {
            rep.do_autonomous_behavior(BEHAVIOR_RUN_STOP);
            let since = self.timers_mut().since_skid;
            if since > 1.0 || (since == 0.0 && forward > ld(2.5_f32)) {
                let intensity = st(forward * ld(0.181_818_19_f32) - ld(0.2_f32));
                rep.do_fx(FX_SKID, intensity);
                self.timers_mut().since_skid = 0.0;
            }
            self.timers_mut().since_skid += q;
        } else {
            if rep.is_doing_behavior(BEHAVIOR_RUN_STOP) {
                rep.stop_behavior(BEHAVIOR_RUN_STOP);
            }
            if !skidding {
                self.timers_mut().since_skid = 0.0;
            }
        }
    }

    /// Falling above walkable ground: the near-landing behaviour under 1.3 m, the dive mode over
    /// deep enough water, else the long-fall behaviour.
    fn falling_behaviors(&mut self, ctx: &MotionContext, height: f64, rep: &mut dyn MotionRep) {
        let above_water = -ld(ctx.depth);
        let vz = ld(ctx.local_velocity.z);
        if !(vz < -2.0) {
            return;
        }
        if height < 1.3 {
            rep.do_autonomous_behavior(BEHAVIOR_NEAR_LANDING);
            return;
        }
        if !(above_water > 2.0) {
            return;
        }
        if above_water < height && ctx.has_water {
            if rep.desired_mode() != MODE_SWIM && self.motion_values(ctx.desired_element).gravity {
                rep.do_mode(0, MODE_DIVE);
            }
            return;
        }
        if height > 50.0 || (height > 10.0 && vz < -19.0) {
            rep.do_autonomous_behavior(BEHAVIOR_LONG_FALL);
        }
    }
}
