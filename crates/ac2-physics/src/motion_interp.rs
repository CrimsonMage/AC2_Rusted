//! `CMotionInterp`: turns movement input (`DoMotion`, `SetOmegaVelocity`) into the local
//! acceleration and omega acceleration the physics object integrates, from the `MotionValues`
//! of the body's `MotionInterpDesc` for its element, mode and terrain.
//!
//! The animation behaviours `UpdateMovement` drives through `PHYSICSOBJREP` are in
//! `motion_interp_behaviors`.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_world::{Ac2MotionInterpDesc, Ac2MotionValues};

use crate::fpu::{ld, st, F_EPSILON};
use crate::heading::Heading;
use crate::motion_interp_behaviors::{BehaviorTimers, MotionRep};
use crate::vector::Vector;

/// `ModeStateKey` element ids.
pub(crate) const ELEMENT_GROUND: u32 = 0x4000_0001;
pub(crate) const ELEMENT_AIR: u32 = 0x4000_0002;
pub(crate) const ELEMENT_WATER: u32 = 0x4000_0003;
pub(crate) const ELEMENT_SEA_FLOOR: u32 = 0x4000_0004;
pub(crate) const ELEMENT_WALK_ON_AIR: u32 = 0x4000_0007;
/// The default (peace) mode.
pub(crate) const MODE_DEFAULT: u32 = 0x4000_0001;
/// `CMotionInterp::mTerrainType`'s constructor value.
const TERRAIN_DEFAULT: u32 = 0x8000_0001;

/// What `CMotionInterp::UpdateMovement` reads back from its object through `PHYSICSOBJREP`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MotionContext {
    /// `GetLocalVelocity` (`CPhysicsObj::get_local_velocity`).
    pub local_velocity: Vector,
    /// `GetOmegaVelocity`.
    pub omega: Vector,
    /// `GetEffectsScale` (`CPhysicsObj::m_vScale`).
    pub scale: Vector,
    /// `GetHeightAboveWalkable`.
    pub height_above_walkable: f32,
    /// `HasWater` and `GetDepth`.
    pub has_water: bool,
    pub depth: f32,
    /// `GetDesiredElement` and `GetElement`.
    pub desired_element: u32,
    pub element: u32,
    /// `IsSlidingDown`: in contact but not on walkable ground.
    pub is_sliding_down: bool,
    /// `GetConstraintFactor` (1 without a `ConstraintManager`).
    pub constraint_factor: f32,
    /// `GetVectorHeading` (`Frame::get_vector_heading`): the forward axis.
    pub vector_heading: Vector,
}

/// The heading and omega clamps `CalculateOmegaAcceleration` installs on its object
/// (`CPhysicsObj::ClampHeading`/`UnclampHeading` and `ClampOmegaVelocity`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OmegaClamp {
    pub heading: Option<Heading>,
    pub omega_z: f32,
}

#[derive(Debug, Clone)]
pub struct MotionInterp {
    desc: Option<Arc<Ac2MotionInterpDesc>>,
    mode: u32,
    acl: Vector,
    omega_acl: Vector,
    desired_omega_vel: Vector,
    local_desired_vel: Vector,
    last_do_motion: Vector,
    max_turn_vel: f32,
    turn_to_dir: Heading,
    turn_to: bool,
    vel_scale: f32,
    acl_scale: f32,
    hover_height: f32,
    terrain_type: u32,
    terrain_mods: HashMap<u32, u32>,
    global_mod: u32,
    timers: BehaviorTimers,
}

impl MotionInterp {
    /// `CMotionInterp::CMotionInterp` over a `MotionInterpDesc` (`Init`/`SetMIDESC`); without one
    /// every lookup yields the default `MotionValues`.
    pub fn new(desc: Option<Arc<Ac2MotionInterpDesc>>) -> Self {
        Self {
            desc,
            mode: MODE_DEFAULT,
            acl: Vector::ZERO,
            omega_acl: Vector::ZERO,
            desired_omega_vel: Vector::ZERO,
            local_desired_vel: Vector::ZERO,
            last_do_motion: Vector::ZERO,
            max_turn_vel: 0.0,
            turn_to_dir: Heading::from_degrees(0.0),
            turn_to: false,
            vel_scale: 1.0,
            acl_scale: 1.0,
            hover_height: -1.0,
            terrain_type: TERRAIN_DEFAULT,
            terrain_mods: HashMap::new(),
            global_mod: 0,
            timers: BehaviorTimers::default(),
        }
    }

    pub(crate) fn mode(&self) -> u32 {
        self.mode
    }

    /// `CMotionInterp::GetTerrainType` (`mTerrainType`).
    pub(crate) fn terrain_type(&self) -> u32 {
        self.terrain_type
    }

    /// The desired mode the motion values are looked up in.
    pub(crate) fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    pub(crate) fn timers_mut(&mut self) -> &mut BehaviorTimers {
        &mut self.timers
    }

    /// `CMotionInterp::mLocalDesiredVel`.
    pub(crate) fn local_desired_velocity(&self) -> Vector {
        self.local_desired_vel
    }

    /// `CMotionInterp::mLastDoMotionValues`.
    pub(crate) fn last_do_motion(&self) -> Vector {
        self.last_do_motion
    }

    /// `CMotionInterp::GetAcceleration` (local axes).
    pub(crate) fn acceleration(&self) -> Vector {
        self.acl
    }

    /// `CMotionInterp::GetOmegaAcceleration`.
    pub(crate) fn omega_acceleration(&self) -> Vector {
        self.omega_acl
    }

    /// `CMotionInterp::SetVelocityScale` (`mVelScale`), reached from the server's
    /// `ECM_Physics::RecvEvt_SetVelocityScale` through `CPhysicsObj::SetVelocityScale` and
    /// `PHYSICSOBJREP::SetVelocityScale`.
    /// `CMotionInterp::SetMIDESC`: the body moves by `desc` from now on (the motion table the
    /// lookups read). The client loads the description on a loader thread and hands it here;
    /// `None` (the DBObj is missing) keeps the current table and returns false, as retail's
    /// `return 0` does.
    pub fn set_desc(&mut self, desc: Option<Arc<Ac2MotionInterpDesc>>) -> bool {
        match desc {
            Some(desc) => {
                self.desc = Some(desc);
                true
            }
            None => false,
        }
    }

    pub fn set_velocity_scale(&mut self, scale: f32) {
        self.vel_scale = scale;
    }

    /// `CMotionInterp::SetAccelerationScale` (`mAclScale`), from
    /// `ECM_Physics::RecvEvt_SetAccelerationScale`.
    pub fn set_acceleration_scale(&mut self, scale: f32) {
        self.acl_scale = scale;
    }

    pub(crate) fn velocity_scale(&self) -> f32 {
        self.vel_scale
    }

    pub(crate) fn acceleration_scale(&self) -> f32 {
        self.acl_scale
    }

    /// `CMotionInterp::SetHoverHeight` (`mHoverHeight`), from `ECM_Physics::RecvEvt_SetHoverHeight`
    /// through `PHYSICSOBJREP::SetHoverHeight`. The default (-1) reads the motion table's height.
    pub fn set_hover_height(&mut self, height: f32) {
        self.hover_height = height;
    }

    /// `CMotionInterp::SetGlobalModifier` (`mGlobalMod`), from
    /// `ECM_Physics::RecvEvt_SetGlobalModifier`: the motion table's lookups key on it.
    pub fn set_global_modifier(&mut self, id: u32) {
        self.global_mod = id;
    }

    /// `CMotionInterp::ResetHoverHeight` (`mHoverHeight = -1`).
    pub fn reset_hover_height(&mut self) {
        self.hover_height = -1.0;
    }

    pub(crate) fn hover_height_override(&self) -> f32 {
        self.hover_height
    }

    /// `CMotionInterp::FindMotionValues`: the desc's values for (element, mode) with the global
    /// modifier and the terrain modifier registered for the current terrain.
    fn find_motion_values(&self, element: u32, mode: u32) -> Option<Ac2MotionValues> {
        let desc = self.desc.as_ref()?;
        let modifier = self.terrain_mods.get(&self.terrain_type).copied();
        desc.motion_values(element, mode, self.global_mod, self.terrain_type, modifier)
            .copied()
    }

    /// `CMotionInterp::GetMotionValues` for the desired mode state: no fallback to the default
    /// mode, the static default `MotionValues` when the desc has none.
    pub(crate) fn motion_values(&self, desired_element: u32) -> Ac2MotionValues {
        self.find_motion_values(desired_element, self.mode)
            .unwrap_or_default()
    }

    /// `PHYSICSOBJREP::GetStandardMotionValues`: the peace-on-earth values (element and mode
    /// `0x40000001`), else the static default.
    pub(crate) fn standard_motion_values(&self) -> Ac2MotionValues {
        self.find_motion_values(ELEMENT_GROUND, MODE_DEFAULT)
            .unwrap_or_default()
    }

    /// `CMotionInterp::GetMotionValuesForElement`: `element` in the desired mode, else the
    /// static default.
    pub(crate) fn motion_values_for_element(&self, element: u32) -> Ac2MotionValues {
        self.find_motion_values(element, self.mode)
            .unwrap_or_default()
    }

    /// `CMotionInterp::GetDesiredMotionValues`: the mode's values, else the element's default
    /// mode, else the static default.
    pub(crate) fn desired_motion_values(&self, element: u32, mode: u32) -> Ac2MotionValues {
        self.find_motion_values(element, mode)
            .or_else(|| self.find_motion_values(element, MODE_DEFAULT))
            .unwrap_or_default()
    }

    /// `CMotionInterp::GetWaterMultiplier` (0 without a desc).
    pub(crate) fn water_multiplier(&self) -> f32 {
        self.desc
            .as_ref()
            .map_or(0.0, |desc| desc.water_multiplier())
    }

    /// `CMotionInterp::GetSeaFloorMultiplier` (0 without a desc).
    pub(crate) fn sea_floor_multiplier(&self) -> f32 {
        self.desc
            .as_ref()
            .map_or(0.0, |desc| desc.sea_floor_multiplier())
    }

    /// `CMotionInterp::GetHoverHeight`.
    pub(crate) fn hover_height(&self, desired_element: u32) -> f32 {
        if self.hover_height == -1.0 {
            self.motion_values(desired_element).hover_height
        } else {
            self.hover_height
        }
    }

    /// `CMotionInterp::DoMotion`: a NaN component zeroes the input; a vector longer than 1 is
    /// normalised (`D3DXVec3Normalize`).
    pub(crate) fn do_motion(&mut self, motion: Vector) {
        let mut motion = motion;
        if motion.x.is_nan() || motion.y.is_nan() || motion.z.is_nan() {
            motion = Vector::ZERO;
        }
        // `(z*z + y*y) + x*x`, as `DoMotion` accumulates it.
        let length2 = (ld(motion.z) * ld(motion.z) + ld(motion.y) * ld(motion.y))
            + ld(motion.x) * ld(motion.x);
        if length2 > 1.0 {
            motion = motion.d3dx_normalize();
        }
        self.last_do_motion = motion;
    }

    /// `CMotionInterp::TurnTo`: turn toward `heading` (quantised to the byte steps
    /// `Heading::ToUInt8` keeps).
    pub(crate) fn turn_to(&mut self, heading: Heading) {
        self.turn_to_dir = Heading::from_u8(heading.to_u8());
        self.turn_to = true;
    }

    /// What `InterpolationManager::Update` / `MoveToManager::Update` write through their
    /// pointers into the interpreter: the do-motion and the turn target, with the turn on and
    /// not instant.
    pub(crate) fn steer(&mut self, motion: Vector, heading: Heading) {
        self.last_do_motion = motion;
        self.turn_to_dir = heading;
        self.turn_to = true;
    }

    /// `CMotionInterp::IsTurningTo`.
    pub(crate) fn is_turning_to(&self) -> bool {
        self.turn_to
    }

    /// `CMotionInterp::GetTurnToDirection`.
    pub(crate) fn turn_to_direction(&self) -> Heading {
        self.turn_to_dir
    }

    /// `CMotionInterp::StopTurnTo`: the turn target becomes the current heading.
    pub(crate) fn stop_turn_to(&mut self, current: Heading) {
        self.turn_to_dir = current;
        self.turn_to = false;
    }

    /// `CMotionInterp::SetOmegaVelocity`: adds `normalized * mMaxTurnVel` to the desired yaw
    /// rate, clamped to `[-mMaxTurnVel, mMaxTurnVel]`. The desired rate is cleared by every
    /// `CalculateOmegaAcceleration`, so input sets it each quantum.
    pub(crate) fn set_omega_velocity(&mut self, normalized: f32) {
        let max = self.max_turn_vel;
        self.desired_omega_vel.z = st(ld(normalized) * ld(max) + ld(self.desired_omega_vel.z));
        let limited = if ld(self.desired_omega_vel.z) < ld(max) {
            self.desired_omega_vel.z
        } else {
            max
        };
        let floor = st(-ld(max));
        self.desired_omega_vel.z = if ld(floor) < ld(limited) {
            limited
        } else {
            floor
        };
    }

    /// `CMotionInterp::ConvertDoMotionToLocalDesired` with the vertical constraint the update
    /// path passes: strafe uses the forward speed while moving forward faster than 0.7.
    fn convert_do_motion_to_local_desired(
        &mut self,
        values: &Ac2MotionValues,
        ctx: &MotionContext,
        constrain: bool,
    ) {
        let factor = if constrain {
            ld(ctx.constraint_factor)
        } else {
            1.0
        };
        let motion = self.last_do_motion;
        self.local_desired_vel = self.local_desired_for(motion, values, ctx.scale, factor);
    }

    /// The local velocity `ConvertDoMotionToLocalDesired(motion, factor, out, 1)` gives a
    /// do-motion vector: forward or backward speed on y, strafe (forward over 0.7 forward) on
    /// x, the vertical speed on z, at the velocity scale and the body's effects scale.
    pub(crate) fn local_desired_for(
        &self,
        motion: Vector,
        values: &Ac2MotionValues,
        scale: Vector,
        factor: f64,
    ) -> Vector {
        let forward = factor * ld(values.max_forward_velocity);
        let backward = factor * ld(values.max_backward_velocity);
        let strafe = ld(st(factor * ld(values.max_strafe_velocity)));
        let vel = ld(self.vel_scale);
        let speed = if ld(motion.y) > 0.0 {
            forward
        } else {
            backward
        };
        let y = st(vel * ld(motion.y) * ld(scale.y) * speed);
        let side = if ld(motion.y) > ld(0.7_f32) {
            forward
        } else {
            strafe
        };
        let x = st(ld(motion.x) * vel * ld(scale.x) * side);
        let z = st(ld(values.max_vertical_velocity) * ld(motion.z));
        Vector::new(x, y, z)
    }

    /// `CMotionInterp::ClampVelocity`: only upper limits. x is first held to the forward
    /// speed (retail compares it with the y-scaled forward limit), y to the forward or backward
    /// limit, then x to the forward or strafe limit by how much of the y limit y uses, z to the
    /// vertical speed.
    pub(crate) fn clamp_velocity(
        &self,
        v: Vector,
        values: &Ac2MotionValues,
        scale: Vector,
    ) -> Vector {
        let vel = ld(self.vel_scale);
        let forward = ld(values.max_forward_velocity);
        let below = |limit: f64, value: f32| if limit < ld(value) { st(limit) } else { value };
        let mut out = v;
        out.x = below(ld(scale.y) * vel * forward, out.x);
        let scaled_y = ld(scale.y) * vel;
        let limit_y = if ld(out.y) > 0.0 {
            scaled_y * forward
        } else {
            scaled_y * ld(values.max_backward_velocity)
        };
        out.y = below(limit_y, out.y);
        let stored_y = ld(st(limit_y));
        let ratio = if !(ld(F_EPSILON) > stored_y.abs()) {
            ld(out.y) / stored_y
        } else {
            0.0
        };
        let scaled_x = ld(scale.x) * vel;
        let limit_x = if ratio > ld(0.7_f32) {
            scaled_x * forward
        } else {
            scaled_x * ld(values.max_strafe_velocity)
        };
        out.x = below(limit_x, out.x);
        out.z = below(ld(values.max_vertical_velocity), out.z);
        out
    }

    /// `CMotionInterp::UpdateMovement` (without the interpolation and move-to managers): the
    /// behaviours on `rep`, the local acceleration toward `mLocalDesiredVel` for each axis, the
    /// hover/ground vertical acceleration, then `CalculateOmegaAcceleration`.
    pub(crate) fn update_movement(
        &mut self,
        quantum: f32,
        ctx: &MotionContext,
        managed: bool,
        rep: &mut dyn MotionRep,
    ) -> OmegaClamp {
        self.advance_jump_timers(quantum);
        let motion = &mut self.last_do_motion;
        if motion.mag2_xyz() > 1.0 {
            for component in [&mut motion.x, &mut motion.y, &mut motion.z] {
                if 1.0 < ld(*component) {
                    *component = 1.0;
                }
                if ld(*component) < -1.0 {
                    *component = -1.0;
                }
            }
        }
        self.turn_to_dir = Heading::from_u8(self.turn_to_dir.to_u8());
        let values = self.motion_values(ctx.desired_element);
        // A managed body (interpolating or moving to) skips the constraint factor.
        self.convert_do_motion_to_local_desired(&values, ctx, !managed);
        self.acl = Vector::ZERO;
        self.omega_acl = Vector::ZERO;
        self.update_behaviors(quantum, ctx, rep);
        self.update_linear_acceleration(quantum, &values, ctx);
        self.calculate_omega_acceleration(ctx)
    }

    fn update_linear_acceleration(
        &mut self,
        quantum: f32,
        values: &Ac2MotionValues,
        ctx: &MotionContext,
    ) {
        let q = ld(quantum);
        let v = ctx.local_velocity;
        let scale = ctx.scale;
        let vel = ld(self.vel_scale);
        let acl = ld(self.acl_scale);
        let max_strafe = ld(st(ld(values.max_strafe_velocity) * vel * ld(scale.x)));
        let max_forward = ld(st(vel * ld(values.max_forward_velocity) * ld(scale.y)));
        let max_backward = ld(st(-(ld(values.max_backward_velocity) * vel * ld(scale.y))));
        self.max_turn_vel = if ld(values.max_forward_velocity).abs() > ld(F_EPSILON) {
            let speed = ld(v.z) * ld(v.z) + ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y);
            let t = speed.sqrt() / ld(values.max_forward_velocity);
            if t <= 1.0 {
                st((1.0 - t) * ld(values.still_turn_velocity)
                    + t * ld(values.running_turn_velocity))
            } else {
                values.running_turn_velocity
            }
        } else {
            values.still_turn_velocity
        };
        let x_scaled = ld(scale.x) > 1.0;
        let x_factor = if x_scaled { ld(scale.x) } else { 1.0 };
        let strafe_acl = ld(st(ld(values.strafe_acceleration) * acl * x_factor));
        let strafe_brake = ld(st(ld(values.strafe_brake) * acl * x_factor));
        let y_factor = if ld(scale.y) > 1.0 { ld(scale.y) } else { 1.0 };
        // The forward acceleration stays on the x87 stack; the others are stored.
        let forward_acl = ld(values.forward_acceleration) * acl * y_factor;
        let forward_brake = ld(st(ld(values.forward_brake) * acl * y_factor));
        let backward_acl = ld(st(-(ld(values.backward_acceleration) * acl * y_factor)));
        let backward_brake = ld(st(-(ld(values.backward_brake) * acl * y_factor)));

        let desired = self.local_desired_vel;
        let (dx, vx) = (ld(desired.x), ld(v.x));
        if dx > vx {
            let braking = dx == 0.0 || (vx < 0.0 && dx > 0.0);
            self.acl.x = st(if braking { strafe_brake } else { strafe_acl });
            let next = q * ld(self.acl.x) + vx;
            if next > max_strafe || next > dx {
                self.acl.x = st((dx - vx) / q);
            }
        }
        if dx < vx {
            let braking = dx == 0.0 || (vx > 0.0 && dx < 0.0);
            self.acl.x = st(-(if braking { strafe_brake } else { strafe_acl }));
            let next = q * ld(self.acl.x) + vx;
            if next < -max_strafe || next < dx {
                self.acl.x = st((dx - vx) / q);
            }
        }
        let (dy, vy) = (ld(desired.y), ld(v.y));
        if dy > vy {
            let braking = dy == 0.0 || (vy < 0.0 && dy > 0.0);
            self.acl.y = st(if braking { forward_brake } else { forward_acl });
            let next = q * ld(self.acl.y) + vy;
            if next > max_forward || next > dy {
                self.acl.y = st((dy - vy) / q);
            }
        }
        if dy < vy {
            let braking = dy == 0.0 || (vy > 0.0 && dy < 0.0);
            self.acl.y = st(if braking {
                backward_brake
            } else {
                backward_acl
            });
            let next = q * ld(self.acl.y) + vy;
            if next < max_backward || next < dy {
                self.acl.y = st((dy - vy) / q);
            }
        }
        if values.gravity {
            self.ground_vertical_acceleration(q, v, ctx);
        } else {
            self.hover_vertical_acceleration(q, values, ctx);
        }
    }

    /// The `mGravity` branch of `UpdateMovement`: on a walkable (height 0) in neither the air
    /// nor desiring it, a vertical velocity is cancelled in one quantum, limited to 3 m/s^2
    /// while moving across the ground faster than 0.1 m/s.
    fn ground_vertical_acceleration(&mut self, q: f64, v: Vector, ctx: &MotionContext) {
        if ld(ctx.height_above_walkable) != 0.0
            || ctx.desired_element == ELEMENT_AIR
            || ctx.element == ELEMENT_AIR
            || ld(v.z) == 0.0
        {
            return;
        }
        let cancel = -(ld(v.z) / q);
        self.acl.z = st(cancel);
        if ld(v.x).abs() > 0.1 || ld(v.y).abs() > 0.1 {
            if cancel > 3.0 {
                self.acl.z = 3.0;
            }
            if ld(self.acl.z) < -3.0 {
                self.acl.z = -3.0;
            }
        }
    }

    /// The hover branch of `UpdateMovement` (no gravity): accelerate toward the hover height,
    /// limited by the vertical speed, and brake to land exactly on it when this quantum would
    /// cross it.
    fn hover_vertical_acceleration(
        &mut self,
        q: f64,
        values: &Ac2MotionValues,
        ctx: &MotionContext,
    ) {
        let v = ctx.local_velocity;
        let mut height = ctx.height_above_walkable;
        if ctx.has_water {
            let above_water = -ld(ctx.depth);
            if above_water < ld(height) {
                height = st(above_water);
            }
        }
        let hover = ld(if self.hover_height == -1.0 {
            values.hover_height
        } else {
            self.hover_height
        });
        let h = ld(height);
        let vertical = ld(values.vertical_acceleration) * ld(self.acl_scale);
        self.acl.z = st(if h < hover { vertical } else { -vertical });
        let next = (ld(self.acl.z) + ld(values.lift)) * q + ld(v.z);
        let max_down = -(ld(values.max_vertical_velocity) * ld(self.vel_scale));
        if next < max_down {
            self.acl.z = st((max_down - ld(v.z)) / q);
        }
        let max_up = ld(st(ld(values.max_vertical_velocity) * ld(self.vel_scale)));
        if next > max_up {
            self.acl.z = st((max_up - ld(v.z)) / q);
        }
        let step = (ld(self.acl.z) + ld(values.lift)) * q;
        let predicted = ((ld(v.z) + step) + step * 0.5) * q + h;
        let crosses = (h < hover && predicted > hover) || (h > hover && predicted < hover);
        if !crosses {
            return;
        }
        let gap = hover - h;
        if gap == 0.0 {
            return;
        }
        let brake = -((ld(v.z) * ld(v.z)) / (gap + gap));
        self.acl.z = st(brake);
        let limit = ld(st(ld(values.vertical_acceleration) * ld(self.acl_scale)));
        if brake > limit {
            self.acl.z = st(limit);
        }
        let floor = -(ld(values.vertical_acceleration) * ld(self.acl_scale));
        if floor > ld(self.acl.z) {
            self.acl.z = st(floor);
        }
    }

    /// `CMotionInterp::CalculateOmegaAcceleration`: while turning to a heading the desired yaw
    /// rate is `mMaxTurnVel` toward it until within 0.001 degrees and still; the yaw rate then
    /// accelerates toward the desired one (`mTurnAcl`) or brakes to zero (`mTurnSB`), and the
    /// object's omega is clamped to the desired rate, which is consumed.
    pub(crate) fn calculate_omega_acceleration(&mut self, ctx: &MotionContext) -> OmegaClamp {
        let values = self.motion_values(ctx.desired_element);
        if self.turn_to {
            self.desired_omega_vel = Vector::ZERO;
            let (x, y) = (ld(ctx.vector_heading.x), ld(ctx.vector_heading.y));
            let length = (x * x + y * y).sqrt();
            let current = if length < ld(F_EPSILON) {
                Heading::ZERO
            } else {
                let inverse = 1.0 / length;
                let xr = ld(st(x * inverse));
                let angle = (y * inverse).atan2(xr);
                Heading::from_radians(st(crate::frame::FIVE_HALVES_PI - angle))
            };
            let delta = Heading::difference(current, self.turn_to_dir).degrees_0_centered();
            if delta.abs() >= ld(0.001_f32) || ld(ctx.omega.z).abs() >= ld(F_EPSILON) {
                let sign = if delta > ld(0.001_f32) {
                    1.0
                } else if -ld(0.001_f32) > delta {
                    -1.0
                } else {
                    0.0
                };
                self.desired_omega_vel.z =
                    st(sign * ld(self.max_turn_vel) + ld(self.desired_omega_vel.z));
            } else {
                self.turn_to = false;
            }
        }
        let omega_z = ld(ctx.omega.z);
        let desired = ld(self.desired_omega_vel.z);
        if desired.abs() < ld(F_EPSILON) {
            if omega_z < 0.0 {
                self.omega_acl.z = values.turn_brake;
            } else if omega_z > 0.0 {
                self.omega_acl.z = st(-ld(values.turn_brake));
            }
        } else if omega_z < desired {
            self.omega_acl.z = values.turn_acceleration;
        } else if omega_z > desired {
            self.omega_acl.z = st(-ld(values.turn_acceleration));
        }
        let heading = (self.turn_to && !(ld(self.omega_acl.z) == 0.0 && omega_z == 0.0))
            .then_some(self.turn_to_dir);
        let clamp = OmegaClamp {
            heading,
            omega_z: self.desired_omega_vel.z,
        };
        self.desired_omega_vel = Vector::ZERO;
        clamp
    }
}

#[cfg(test)]
#[path = "motion_interp_tests.rs"]
mod tests;
