//! `CMissileManager`'s flight state machine: `Update` looks the target up, places the missile,
//! starts it moving and steers it (`MissileInfoUpdate`, `CalcAcceleration`). What retail asked of
//! the missile's `ICPhysicsObj` and of the object table (`ICPhysicsObj::GetObjectA`) goes through
//! [`MissileFlightHost`]. Errors are retail's `ulong` codes: 1 target gone, 2 bad position or
//! placement refused, 3 cannot fire (no solution or too far), 4 parameters refused, 6 target not
//! set up yet.

use crate::fpu::{ld, st, F_EPSILON};
use crate::frame::Frame;
use crate::heading::Heading;
use crate::missile::{
    targeting_bits, MissileHost, MissileManager, MissileParameters, MissileTargetingParameters,
    TargetPosition, MISSILE_FLIGHT_FX,
};
use crate::missile_gravity::chase_launch_velocity;
use crate::position::Position;
use crate::target::TargetStatus;
use crate::vector::Vector;

/// What a target or source object's `ICPhysicsObj` gives for a missile (vtable slot `0x4a0`
/// and the velocity of its second base).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTarget {
    /// The slot's result: the point was found.
    pub ok: bool,
    /// The point on the object (height adjustment, scale and offset applied).
    pub position: Position,
    /// The object's velocity.
    pub velocity: Vector,
}

/// The `ICPhysicsObj` surface the flight state machine uses beyond [`MissileHost`].
pub trait MissileFlightHost: MissileHost {
    /// The missile body's position (`0x428`).
    fn position(&self) -> Position;
    /// `0x45c`: the height above the ground, when the body has one.
    fn height_above_ground(&self) -> Option<f32>;
    /// `ICPhysicsObj::GetObjectA(id)` then `0x4a0(height_adjustment, use_scale, offset, ..)` and
    /// its velocity; `None` when there is no such object.
    fn object_target(
        &mut self,
        id: u64,
        height_adjustment: f32,
        use_scale: bool,
        offset: Vector,
    ) -> Option<ObjectTarget>;
    /// `ICPhysicsObj::GetObjectA(id)` then `0x428`.
    fn object_position(&self, id: u64) -> Option<Position>;
    /// `ICPhysicsObj::GetObjectA(id)` then `0x418` (its heading).
    fn object_heading(&self, id: u64) -> Option<Heading>;
    /// `0x4b8`: the id the body currently targets.
    fn target_id(&self) -> u64;
    /// `0x4a8(0, id, 1.0, 0)`: target `id`.
    fn set_target(&mut self, id: u64);
    /// `0x3d4` and `0x434`: the position the body starts from when it has one (a parented
    /// missile).
    fn parent_start_position(&self) -> Option<Position>;
    /// `0x52c(&pos)`: place the missile; false when refused.
    fn place_missile(&mut self, pos: &Position) -> bool;
    /// `0x1c4(1)`, called when the missile's `align_type` is 0.
    fn slot_1c4(&mut self, on: bool);
}

fn arm_from(mgr: &mut MissileManager, given: &MissileParameters, now: f64) {
    if given.missile_has_lifespan {
        mgr.expire_reported = false;
        mgr.missile_expire_time = f64::from(st(ld(mgr.info.missile_lifespan) + ld(st(now))));
    }
    if given.uses_fuel {
        mgr.fuel_reported = false;
        mgr.current_fuel = mgr.info.total_fuel;
    }
}

impl MissileManager {
    /// `CMissileManager::ClientInitMissileParameters`: merge the parameters in; error 4 if
    /// refused. A lifespan and a fuel load the new parameters name are armed from `now`.
    pub fn client_init_missile_parameters(&mut self, info: &MissileParameters, now: f64) -> u32 {
        if !info.copy_into(&mut self.info) {
            return 4;
        }
        arm_from(self, info, now);
        0
    }

    /// `CMissileManager::ClientInitTargetingParameters`: take the targeting parameters whole
    /// (`MissileTargetingParameters::operator=`) and set the target up.
    pub fn client_init_targeting_parameters(
        &mut self,
        host: &mut impl MissileFlightHost,
        params: &MissileTargetingParameters,
    ) -> u32 {
        self.target_info = params.clone();
        self.setup_target(host)
    }

    /// `CMissileManager::Initialize`: merge both parameter sets in and set the target up.
    pub fn initialize(
        &mut self,
        host: &mut impl MissileFlightHost,
        info: &MissileParameters,
        target_info: &MissileTargetingParameters,
    ) -> u32 {
        let a = info.copy_into(&mut self.info);
        let b = target_info.copy_into(&mut self.target_info);
        if !(a && b) {
            return 4;
        }
        self.setup_target(host)
    }

    /// `CMissileManager::Retarget`: the missile keeps its velocity, loses its acceleration, takes
    /// the new parameters and target, and re-arms the lifespan and fuel the new parameters name.
    pub fn retarget(
        &mut self,
        host: &mut impl MissileFlightHost,
        info: &MissileParameters,
        target_info: &MissileTargetingParameters,
        now: f64,
    ) -> u32 {
        self.desired_velocity = self.current_velocity;
        self.current_acceleration = Vector::ZERO;
        let a = info.copy_into(&mut self.info);
        let b = target_info.copy_into(&mut self.target_info);
        if !(a && b) {
            return 4;
        }
        let error = self.setup_target(host);
        if error == 0 {
            if info.missile_has_lifespan {
                self.expire_reported = false;
                self.missile_expire_time =
                    f64::from(st(ld(self.info.missile_lifespan) + ld(st(now))));
            }
            if info.uses_fuel {
                self.current_fuel = self.info.total_fuel;
                self.fuel_reported = false;
            }
        }
        error
    }

    /// `CMissileManager::ClientRetarget`: new parameters and the server's target, then an
    /// `Update`. Returns `CopyInto`'s result and the update's error.
    pub fn client_retarget(
        &mut self,
        host: &mut impl MissileFlightHost,
        param: &MissileParameters,
        target: &TargetPosition,
        now: f64,
        coin: impl FnMut(f32) -> bool,
    ) -> (bool, u32) {
        let copied = param.copy_into(&mut self.info);
        self.target_pos = target.clone();
        self.last_update_time = now;
        let error = self.update(host, now, coin);
        (copied, error)
    }

    /// `CMissileManager::ClientActivate`: the server's target and velocity; the missile is
    /// moving at once.
    pub fn client_activate(
        &mut self,
        host: &mut impl MissileFlightHost,
        missile_velocity: Vector,
        target_desc: &TargetPosition,
        now: f64,
    ) {
        self.target_pos = target_desc.clone();
        self.current_velocity = missile_velocity;
        host.set_missile_velocity(missile_velocity);
        self.desired_velocity = missile_velocity;
        self.last_update_time = now;
        self.is_moving = true;
        host.start_streak();
        host.do_fx(MISSILE_FLIGHT_FX, 1.0);
    }

    /// `CMissileManager::SetupTarget`: an object target waits for its first sighting; otherwise
    /// the target is the fixed point it already holds. Then `TargetUpdate`.
    pub fn setup_target(&mut self, host: &mut impl MissileFlightHost) -> u32 {
        if self
            .target_info
            .test_bit_mask(targeting_bits::TARGET_OBJECT)
        {
            self.target_pos.target_id = self.target_info.target_id;
            self.target_pos.initialized = false;
        } else {
            let pos = self.target_pos.offset_pos;
            self.target_pos.reset(pos);
        }
        self.target_status = TargetStatus::Undef;
        self.target_update(host)
    }

    /// `CMissileManager::TargetUpdate`: a fixed point is Ok once it has a cell; an object is
    /// looked up for its point and velocity, and the body's own target is pointed at it. Error 1
    /// for a fixed point with no cell.
    pub fn target_update(&mut self, host: &mut impl MissileFlightHost) -> u32 {
        let id = self.target_pos.target_id;
        if id == 0 {
            if self.target_pos.offset_pos.cell_id != 0 {
                self.target_status = TargetStatus::Ok;
                return 0;
            }
            return 1;
        }
        let ti = &self.target_info;
        if let Some(found) = host.object_target(
            id,
            ti.target_height_adjustment,
            ti.use_target_scale,
            ti.target_offset,
        ) {
            if found.ok {
                self.target_pos
                    .set_target_info(found.position, found.velocity);
                self.target_status = TargetStatus::Ok;
            }
        }
        if host.target_id() != id {
            host.set_target(id);
        }
        0
    }

    /// `CMissileManager::GetStartPos`: the base position the targeting parameters name.
    pub fn get_start_pos(&self) -> Position {
        self.target_info.base_pos
    }

    /// `CMissileManager::InitPosition`: where the missile starts. A parented missile starts where
    /// its body says when that is valid; else at the source object's point (bit `0x10`) or the
    /// base position. Error 2 when there is none.
    pub fn init_position(&self, host: &mut impl MissileFlightHost) -> Result<Position, u32> {
        if let Some(pos) = host.parent_start_position() {
            if pos.is_valid() {
                return Ok(pos);
            }
        }
        let pos = if !self
            .target_info
            .test_bit_mask(targeting_bits::SOURCE_BASE_POSITION)
        {
            self.get_start_pos()
        } else {
            let ti = &self.target_info;
            let Some(found) = host.object_target(
                ti.source_id,
                ti.source_height_adjustment,
                ti.use_source_scale,
                ti.source_offset,
            ) else {
                return Err(2);
            };
            if !found.ok {
                return Err(2);
            }
            found.position
        };
        if pos.is_valid() {
            Ok(pos)
        } else {
            Err(2)
        }
    }

    /// `CMissileManager::AttemptWorldPlacement`: find the start, arm the lifespan and fuel, and
    /// ask the body to place the missile there (error 2 when it refuses).
    pub fn attempt_world_placement(&mut self, host: &mut impl MissileFlightHost, now: f64) -> u32 {
        let pos = match self.init_position(host) {
            Ok(pos) => pos,
            Err(error) => return error,
        };
        let info = self.info;
        arm_from(self, &info, now);
        if host.place_missile(&pos) {
            0
        } else {
            2
        }
    }

    /// `CMissileManager::InitHeadingTowardTarget`: whether the missile starts toward its target:
    /// never without the start-toward bit (`0x40`), always with the always bit (`0x400`), else
    /// when the target is within 45 degrees of the source's (or missile's) heading.
    pub fn init_heading_toward_target(
        &self,
        host: &impl MissileFlightHost,
        missile_pos: &Position,
        target_pos: &Position,
    ) -> bool {
        if !self
            .target_info
            .test_bit_mask(targeting_bits::START_TOWARD_TARGET)
        {
            return false;
        }
        if self
            .target_info
            .test_bit_mask(targeting_bits::ALWAYS_START_TOWARD_TARGET)
        {
            return true;
        }
        let source = host
            .object_position(self.target_info.source_id)
            .unwrap_or(*missile_pos);
        source
            .local_heading(target_pos)
            .is_in_range(Heading::ZERO, Heading::from_degrees(45.0))
    }

    /// `CMissileManager::GetInitialGravitationalFlight`: the launch velocity of a ballistic
    /// missile at `target_desc`. A start that is not toward the target turns the horizontal speed
    /// to the source's heading.
    pub fn get_initial_gravitational_flight(
        &self,
        host: &impl MissileFlightHost,
        missile_pos: &Position,
        target_desc: &TargetPosition,
    ) -> Vector {
        let target_velocity = target_desc.target_velocity;
        let diff = missile_pos.offset_to(&target_desc.offset_pos);
        let (input, by_pitch) = if self
            .target_info
            .test_bit_mask(targeting_bits::INITIAL_PITCH)
        {
            (self.target_info.initial_pitch, true)
        } else if self
            .target_info
            .test_bit_mask(targeting_bits::INITIAL_SPEED)
        {
            (self.target_info.initial_velocity, false)
        } else {
            (self.info.max_velocity, false)
        };
        let mut speed = self.calc_gravitational_velocity(
            host.current_gravity(),
            input,
            diff,
            target_velocity,
            by_pitch,
        );
        if !self.init_heading_toward_target(host, missile_pos, &target_desc.offset_pos) {
            let heading = host
                .object_heading(self.target_info.source_id)
                .unwrap_or_else(|| missile_pos.frame.heading());
            let flat = st((ld(speed.y) * ld(speed.y) + ld(speed.x) * ld(speed.x)).sqrt());
            let radians = heading.radians();
            speed.x = st(radians.sin() * ld(flat));
            speed.y = st(radians.cos() * ld(flat));
        }
        speed
    }

    /// `CMissileManager::CalcInitialVelocity`: the velocity the missile leaves `start_pos` with,
    /// or the error (2 bad start, 6 no target yet, 3 no solution or beyond `m_missile_range`).
    pub fn calc_initial_velocity(
        &self,
        host: &impl MissileFlightHost,
        start_pos: &Position,
        now: f64,
        mut coin: impl FnMut(f32) -> bool,
    ) -> Result<Vector, u32> {
        if !start_pos.is_valid() {
            return Err(2);
        }
        if !self.target_pos.initialized {
            return Err(6);
        }
        let velocity;
        if !self.info.affected_by_gravity || self.info.use_maint_dist {
            if !self.init_heading_toward_target(host, start_pos, &self.target_pos.offset_pos) {
                let mut speed = self.info.max_velocity;
                if self
                    .target_info
                    .test_bit_mask(targeting_bits::INITIAL_SPEED)
                {
                    speed = self.target_info.initial_velocity;
                }
                let f = start_pos.forward_vector();
                velocity = Vector::new(
                    st(ld(speed) * ld(f.x)),
                    st(ld(speed) * ld(f.y)),
                    st(ld(speed) * ld(f.z)),
                );
            } else {
                let mut out = Vector::ZERO;
                self.intercept_velocity(start_pos, true, true, now, &mut coin, &mut out);
                velocity = out;
            }
        } else {
            velocity = self.get_initial_gravitational_flight(host, start_pos, &self.target_pos);
        }
        if self
            .target_info
            .test_bit_mask(targeting_bits::NO_FIRE_IF_TOO_FAR)
        {
            let d = start_pos.offset_to(&self.target_pos.offset_pos);
            let range = ld(self.target_info.missile_range);
            if range * range < (ld(d.x) * ld(d.x) + ld(d.y) * ld(d.y)) + ld(d.z) * ld(d.z) {
                return Err(3);
            }
        }
        Ok(velocity)
    }

    /// `CMissileManager::StartMovement`: launch from the body's position at the initial velocity.
    pub fn start_movement(
        &mut self,
        host: &mut impl MissileFlightHost,
        now: f64,
        coin: impl FnMut(f32) -> bool,
    ) -> u32 {
        let start = host.position();
        let velocity = match self.calc_initial_velocity(host, &start, now, coin) {
            Ok(v) => v,
            Err(error) => return error,
        };
        if self.info.align_type == 0 {
            host.slot_1c4(true);
        }
        self.current_velocity = velocity;
        host.set_missile_velocity(velocity);
        self.desired_velocity = velocity;
        self.last_update_time = now;
        self.is_moving = true;
        host.start_streak();
        host.do_fx(MISSILE_FLIGHT_FX, 1.0);
        0
    }

    /// `CMissileManager::Update`: the per-quantum driver. Look the target up; place the missile
    /// when asked (clearing the request), start it when asked, then steer it and report running
    /// out of fuel once.
    pub fn update(
        &mut self,
        host: &mut impl MissileFlightHost,
        now: f64,
        mut coin: impl FnMut(f32) -> bool,
    ) -> u32 {
        if !self.is_activated {
            return 0;
        }
        let error = self.target_update(host);
        if error != 0 || self.target_status == TargetStatus::Undef {
            return error;
        }
        if self.target_status != TargetStatus::Ok {
            return 1;
        }
        if !self.target_pos.initialized {
            return 0;
        }
        if self.ready_to_place {
            let error = self.attempt_world_placement(host, now);
            if error != 0 {
                return error;
            }
            self.ready_to_place = false;
        }
        if !self.is_in_world(host) {
            return 0;
        }
        if self.ready_to_fire {
            let error = self.start_movement(host, now, &mut coin);
            if error != 0 {
                return error;
            }
            self.ready_to_fire = false;
        }
        if self.is_moving {
            let error = self.missile_info_update(host, now, &mut coin);
            if error != 0 {
                return error;
            }
            if self.info.uses_fuel
                && self.out_of_fuel()
                && self.info.report_if_no_fuel
                && !self.fuel_reported
            {
                self.report_out_of_fuel(host);
            }
        }
        0
    }

    /// `CMissileManager::MissileInfoUpdate`: once `reaction_time_check` seconds have passed,
    /// burn fuel, note the height and choose the desired velocity: a ballistic missile solves
    /// the gravity flight (or heads 45 degrees up toward the target when it cannot), any other
    /// intercepts. Then `CalcAcceleration`. Error 6 when the target is not set up.
    pub fn missile_info_update(
        &mut self,
        host: &mut impl MissileFlightHost,
        now: f64,
        coin: impl FnMut(f32) -> bool,
    ) -> u32 {
        let elapsed = st(now - self.last_update_time);
        if !(ld(self.info.reaction_time_check) < ld(elapsed)) {
            return 0;
        }
        self.last_update_time = now;
        if !self.target_pos.initialized {
            return 6;
        }
        self.update_fuel_use(ld(elapsed));
        if let Some(height) = host.height_above_ground() {
            self.current_height = height;
        }
        let position = host.position();
        if self.info.affected_by_gravity && !self.info.use_maint_dist {
            let diff = position.offset_to(&self.target_pos.offset_pos);
            let target_velocity = self.target_pos.target_velocity;
            let gravity = host.current_gravity();
            let (possible, t_min, t_max) = crate::missile::calc_min_max_time(
                diff,
                target_velocity,
                gravity,
                self.info.max_velocity,
            );
            self.desired_velocity = if possible {
                let v_min = crate::missile::get_gravitational_velocity(
                    diff,
                    target_velocity,
                    gravity,
                    t_min,
                )
                .unwrap_or_default();
                let v_max = crate::missile::get_gravitational_velocity(
                    diff,
                    target_velocity,
                    gravity,
                    t_max,
                )
                .unwrap_or_default();
                self.calc_gravitational_closest(
                    t_min,
                    t_max,
                    diff,
                    target_velocity,
                    gravity,
                    self.current_velocity,
                    v_min,
                    v_max,
                )
            } else {
                chase_launch_velocity(diff, target_velocity, self.info.max_velocity)
            };
        } else {
            let mut out = self.desired_velocity;
            self.intercept_velocity(&position, false, false, now, coin, &mut out);
            self.desired_velocity = out;
        }
        self.calc_acceleration();
        0
    }

    /// `CMissileManager::CalcAcceleration`: the acceleration toward the desired velocity. The
    /// wanted change over `min_reaction_time` (plus the offset) is cut to the maximum
    /// accelerations in the missile's own axes (forward is the current velocity; up is the most
    /// vertical perpendicular). Nothing changes with an empty tank.
    pub fn calc_acceleration(&mut self) {
        if self.info.uses_fuel && self.current_fuel == 0.0 {
            return;
        }
        let cur = self.current_velocity;
        let want = self.desired_velocity;
        let dx = st(ld(want.x) - ld(cur.x));
        let dy = st(ld(want.y) - ld(cur.y));
        let dz = st(ld(want.z) - ld(cur.z));
        let inverse = st(1.0 / ld(self.info.min_reaction_time));
        let off = self.accel_offset;
        let mut base = Vector::new(
            st(ld(dx) * ld(inverse) + ld(off.x)),
            st(ld(inverse) * ld(dy) + ld(off.y)),
            st(ld(inverse) * ld(dz) + ld(off.z)),
        );
        if self.info.conserve_fuel && self.info.drag != 0.0 {
            let drag = ld(self.info.drag);
            if ld(cur.x) * ld(dx) < 0.0 && ld(base.x) < ld(cur.x) * drag * 1.5 {
                base.x = 0.0;
            }
            if ld(cur.y) * ld(dy) < 0.0 && ld(base.y) < ld(cur.y) * drag * 1.5 {
                base.y = 0.0;
            }
            if ld(cur.z) * ld(dz) < 0.0 && ld(base.z) < ld(cur.z) * drag * 1.5 {
                base.z = 0.0;
            }
        }
        let length = (ld(cur.x) * ld(cur.x) + ld(cur.y) * ld(cur.y) + ld(cur.z) * ld(cur.z)).sqrt();
        let forward = if length < ld(F_EPSILON) {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            let k = 1.0 / length;
            Vector::new(st(ld(cur.x) * k), st(ld(cur.y) * k), st(k * ld(cur.z)))
        };
        let flat = ld(forward.x) * ld(forward.x) + ld(forward.y) * ld(forward.y);
        let (forward, up) = if ld(F_EPSILON) <= flat {
            let up = if forward.z == 0.0 {
                Vector::new(0.0, 0.0, 1.0)
            } else {
                let ux = st(-ld(forward.x));
                let uy = st(-ld(forward.y));
                let uz = st(flat / ld(forward.z));
                let l = (ld(ux) * ld(ux) + ld(uy) * ld(uy) + ld(uz) * ld(uz)).sqrt();
                if ld(F_EPSILON) <= l {
                    let k = 1.0 / l;
                    Vector::new(st(k * ld(ux)), st(ld(uy) * k), st(ld(uz) * k))
                } else {
                    Vector::new(ux, uy, uz)
                }
            };
            (forward, up)
        } else {
            (Vector::new(0.0, 0.0, 1.0), Vector::new(0.0, 1.0, 0.0))
        };
        let mut frame = Frame::IDENTITY;
        frame.orient_frame(up, forward);
        let m = frame.m.map(ld);
        let b = (ld(base.x), ld(base.y), ld(base.z));
        let local_x = st(m[0] * b.0 + m[1] * b.1 + m[2] * b.2);
        let local_y = st(m[3] * b.0 + m[4] * b.1 + m[5] * b.2);
        let local_z = st(m[6] * b.0 + m[7] * b.1 + m[8] * b.2);
        let i = &self.info;
        let mut clamped = false;
        let mut lx = local_x;
        if i.max_right_accel < lx {
            lx = i.max_right_accel;
            clamped = true;
        }
        if lx < -i.max_left_accel {
            lx = -i.max_left_accel;
            clamped = true;
        }
        let mut ly = local_y;
        if i.max_forward_accel < ly {
            ly = i.max_forward_accel;
            clamped = true;
        }
        if ly < -i.max_decel {
            ly = -i.max_decel;
            clamped = true;
        }
        let mut lz = local_z;
        if i.max_upward_accel < lz {
            lz = i.max_upward_accel;
            clamped = true;
        }
        if lz < -i.max_downward_accel {
            lz = -i.max_downward_accel;
            clamped = true;
        }
        if clamped {
            base = frame.local_to_global_vec(Vector::new(lx, ly, lz));
        }
        self.current_acceleration = base;
    }
}

#[cfg(test)]
#[path = "missile_flight_tests.rs"]
mod tests;
