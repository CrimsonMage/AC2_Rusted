//! `CMissileManager` (`portal/shared/src/physobj/missile/missilemanager.cpp`): the flight of a
//! missile (a spell projectile) toward a target. The manager holds `MissileParameters` (the
//! script's `gMissileParameters` setters build them), the target, and the flight state; what
//! retail asked of its `ICPhysicsObj` (gravity, wind, `set_missile_velocity`, the FX and streak
//! calls, the expiry and out-of-fuel reports) goes through [`MissileHost`].
//!
//! Ported so far: the parameters, the fuel, the speed and height limits, the time and velocity
//! helpers (`GetMinTime`, `GetMaxTime`, `GetGravitationalVelocity`, `ChaseVelocity`) and the
//! per-quantum physics (`UpdateMissilePhysics`). The float math follows the x87 spills of the
//! retail code.
//!
//! The script side (`ac2-script`) binds `gMissileParameters` and `gMissileTargetingParameters`:
//! it builds a [`MissileParameters`] from its named words with [`MissileParameters::from_words`]
//! and hands it to the manager.

use crate::fpu::{epsilon_squared, ld, st, F_EPSILON};
use crate::frame::Frame;
use crate::position::Position;
use crate::vector::Vector;

/// `MissileParameters`: how a missile flies. `MissileParameters::MissileParameters` gives the
/// defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MissileParameters {
    pub context: u32,
    /// `m_bitfield`: which groups the script has set.
    pub bitfield: u32,
    pub max_velocity: f32,
    pub min_velocity: f32,
    pub max_forward_accel: f32,
    pub max_decel: f32,
    pub max_left_accel: f32,
    pub max_right_accel: f32,
    pub max_upward_accel: f32,
    pub max_downward_accel: f32,
    pub total_fuel: f32,
    pub uses_fuel: bool,
    pub report_if_no_fuel: bool,
    pub missile_lifespan: f32,
    pub missile_has_lifespan: bool,
    pub drag: f32,
    pub affected_by_gravity: bool,
    pub affected_by_wind: bool,
    /// `m_ground_follow_type`: 0 none, 1 stay at least `height_above_ground` up, 2 hold it.
    pub ground_follow_type: u32,
    pub height_above_ground: f32,
    pub tight_turn_method: u32,
    pub conserve_fuel: bool,
    pub consider_remaining_time: bool,
    pub min_catchup_speed: f32,
    pub maintain_distance: f32,
    pub use_maint_dist: bool,
    pub reaction_time_check: f32,
    pub min_reaction_time: f32,
    pub correct_for_target_velocity: bool,
    pub use_env_max_velocities: bool,
    pub align_type: i32,
    pub spin_rate: f32,
    pub sticks_upon_collide: bool,
    pub can_land: bool,
    pub detonates_on_miss: bool,
    pub detonation_proximity: f32,
    pub detonation_chance: f32,
}

impl Default for MissileParameters {
    /// `MissileParameters::MissileParameters`.
    fn default() -> Self {
        Self {
            context: 0,
            bitfield: 0,
            max_velocity: 50.0,
            min_velocity: 1.0,
            max_forward_accel: 1.0,
            max_decel: 1.0,
            max_left_accel: 1.0,
            max_right_accel: 1.0,
            max_upward_accel: 1.0,
            max_downward_accel: 1.0,
            total_fuel: 0.0,
            uses_fuel: false,
            report_if_no_fuel: true,
            missile_lifespan: 0.0,
            missile_has_lifespan: false,
            drag: 0.0,
            affected_by_gravity: false,
            affected_by_wind: false,
            ground_follow_type: 0,
            height_above_ground: 1.5,
            tight_turn_method: 2,
            conserve_fuel: false,
            consider_remaining_time: false,
            min_catchup_speed: 2.0,
            maintain_distance: 5.0,
            use_maint_dist: false,
            reaction_time_check: 0.5,
            min_reaction_time: 1.0,
            correct_for_target_velocity: true,
            use_env_max_velocities: true,
            align_type: 0,
            spin_rate: 0.0,
            sticks_upon_collide: false,
            can_land: false,
            detonates_on_miss: false,
            detonation_proximity: 0.0,
            detonation_chance: 0.0,
        }
    }
}

impl MissileParameters {
    /// The parameters from the script's named 32-bit words (a float's bits, an int or a bool),
    /// over the constructor's defaults. A name this does not know is ignored.
    pub fn from_words<'a>(bitfield: u32, words: impl IntoIterator<Item = (&'a str, u32)>) -> Self {
        let mut p = Self {
            bitfield,
            ..Self::default()
        };
        for (name, word) in words {
            let f = f32::from_bits(word);
            let b = word != 0;
            match name {
                "max_velocity" => p.max_velocity = f,
                "min_velocity" => p.min_velocity = f,
                "max_forward_accel" => p.max_forward_accel = f,
                "max_decel" => p.max_decel = f,
                "max_left_accel" => p.max_left_accel = f,
                "max_right_accel" => p.max_right_accel = f,
                "max_upward_accel" => p.max_upward_accel = f,
                "max_downward_accel" => p.max_downward_accel = f,
                "total_fuel" => p.total_fuel = f,
                "uses_fuel" => p.uses_fuel = b,
                "report_if_no_fuel" => p.report_if_no_fuel = b,
                "missile_lifespan" => p.missile_lifespan = f,
                "missile_has_lifespan" => p.missile_has_lifespan = b,
                "drag" => p.drag = f,
                "affected_by_gravity" => p.affected_by_gravity = b,
                "affected_by_wind" => p.affected_by_wind = b,
                "ground_follow_type" => p.ground_follow_type = word,
                "height_above_ground" => p.height_above_ground = f,
                "tight_turn_method" => p.tight_turn_method = word,
                "conserve_fuel" => p.conserve_fuel = b,
                "consider_remaining_time" => p.consider_remaining_time = b,
                "min_catchup_speed" => p.min_catchup_speed = f,
                "maintain_distance" => p.maintain_distance = f,
                "use_maint_dist" => p.use_maint_dist = b,
                "reaction_time_check" => p.reaction_time_check = f,
                "min_reaction_time" => p.min_reaction_time = f,
                "correct_for_target_velocity" => p.correct_for_target_velocity = b,
                "use_env_max_velocities" => p.use_env_max_velocities = b,
                "align_type" => p.align_type = word as i32,
                "spin_rate" => p.spin_rate = f,
                "sticks_upon_collide" => p.sticks_upon_collide = b,
                "can_land" => p.can_land = b,
                "detonates_on_miss" => p.detonates_on_miss = b,
                "detonation_proximity" => p.detonation_proximity = f,
                "detonation_chance" => p.detonation_chance = f,
                _ => {}
            }
        }
        p
    }
}

/// `MissileTargetingParameters::m_bitfield` bits the flight reads.
pub mod targeting_bits {
    /// `SetTargetID`: the missile chases an object.
    pub const TARGET_OBJECT: u32 = 0x1;
    /// `SetInitialSpeed`.
    pub const INITIAL_SPEED: u32 = 0x4;
    /// `UseSourceObjectForBasePosition`.
    pub const SOURCE_BASE_POSITION: u32 = 0x10;
    /// `SetStartTowardTarget`.
    pub const START_TOWARD_TARGET: u32 = 0x40;
    /// `SetInitialPitch`.
    pub const INITIAL_PITCH: u32 = 0x80;
    /// `SetNoFireIfTooFar`.
    pub const NO_FIRE_IF_TOO_FAR: u32 = 0x200;
    /// `SetAlwaysStartTowardTarget`.
    pub const ALWAYS_START_TOWARD_TARGET: u32 = 0x400;
}

/// `MissileTargetingParameters`: where a missile starts and what it flies at.
/// `MissileTargetingParameters::MissileTargetingParameters` gives the defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct MissileTargetingParameters {
    pub bitfield: u32,
    pub target_id: u64,
    pub target_height_adjustment: f32,
    pub use_target_scale: bool,
    pub target_offset: Vector,
    pub target_pos: Position,
    pub base_pos: Position,
    pub source_id: u64,
    pub source_height_adjustment: f32,
    pub use_source_scale: bool,
    pub source_offset: Vector,
    pub initial_velocity: f32,
    pub initial_pitch: f32,
    pub missile_range: f32,
    pub missile_detonation_chance: f32,
}

impl Default for MissileTargetingParameters {
    fn default() -> Self {
        Self {
            bitfield: 0,
            target_id: 0,
            target_height_adjustment: 0.75,
            use_target_scale: false,
            target_offset: Vector::ZERO,
            target_pos: Position::default(),
            base_pos: Position::default(),
            source_id: 0,
            source_height_adjustment: 0.75,
            use_source_scale: false,
            source_offset: Vector::ZERO,
            initial_velocity: 0.0,
            initial_pitch: 0.0,
            missile_range: 0.0,
            missile_detonation_chance: 0.0,
        }
    }
}

impl MissileTargetingParameters {
    /// `CContinSoundInfo::TestBitMask` on the bitfield.
    pub fn test_bit_mask(&self, mask: u32) -> bool {
        self.bitfield & mask != 0
    }
}

/// `TargetPosition`: where the missile is headed and how the target moves.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TargetPosition {
    /// `m_target_id`: 0 is `INVALID_IID` (a fixed point).
    pub target_id: u64,
    pub offset_pos: Position,
    pub target_velocity: Vector,
    pub initialized: bool,
}

impl TargetPosition {
    /// `TargetPosition::Reset`: a fixed point at `pos`.
    pub fn reset(&mut self, pos: Position) {
        self.offset_pos = pos;
        self.target_id = 0;
        self.target_velocity = Vector::ZERO;
        self.initialized = true;
    }

    /// `TargetPosition::SetTargetInfo`: the target's latest point and velocity.
    pub fn set_target_info(&mut self, pos: Position, velocity: Vector) {
        self.initialized = true;
        self.offset_pos = pos;
        self.target_velocity = velocity;
    }
}

/// What `CMissileManager` asks of its `ICPhysicsObj` (the missile's body), by vtable slot.
pub trait MissileHost {
    /// `GetCurrentGravity` (0x450).
    fn current_gravity(&self) -> Vector;
    /// `GetCurrentWind` (0x454).
    fn current_wind(&self) -> Vector;
    /// `IsInWorld` (0x118).
    fn is_in_world(&self) -> bool;
    /// `set_missile_velocity(velocity, 1)` (0x334).
    fn set_missile_velocity(&mut self, velocity: Vector);
    /// `MissileReportOutOfFuel` (0x530).
    fn report_out_of_fuel(&mut self);
    /// `MissileReportExpired` (0x534).
    fn report_expired(&mut self);
    /// `StopFX(fx)` (0x3c0).
    fn stop_fx(&mut self, fx: u32);
    /// `DoFX(fx, scalar)` (0x3bc).
    fn do_fx(&mut self, fx: u32, scalar: f32);
    /// `StartStreak(0)` (0x3c8).
    fn start_streak(&mut self);
    /// `StopStreak` (0x3cc).
    fn stop_streak(&mut self);
}

/// The fx the missile's flight plays (`SetMoving`, `ClientActivate`).
pub const MISSILE_FLIGHT_FX: u32 = 0x4000_0001;

/// `CMissileManager`.
#[derive(Debug, Clone, PartialEq)]
pub struct MissileManager {
    pub info: MissileParameters,
    pub target_info: MissileTargetingParameters,
    pub target_pos: TargetPosition,
    pub accel_offset: Vector,
    pub desired_velocity: Vector,
    pub last_update_time: f64,
    pub current_fuel: f32,
    pub fuel_reported: bool,
    pub missile_expire_time: f64,
    pub expire_reported: bool,
    pub current_height: f32,
    pub current_acceleration: Vector,
    pub current_velocity: Vector,
    pub num_missile_steps: u32,
    pub min_time_step: f32,
    pub time_difference_limit: f32,
    pub is_activated: bool,
    pub ready_to_place: bool,
    pub ready_to_fire: bool,
    pub is_moving: bool,
}

impl Default for MissileManager {
    /// `CMissileManager::CMissileManager`.
    fn default() -> Self {
        Self {
            info: MissileParameters::default(),
            target_info: MissileTargetingParameters::default(),
            target_pos: TargetPosition::default(),
            accel_offset: Vector::ZERO,
            desired_velocity: Vector::ZERO,
            last_update_time: 0.0,
            current_fuel: 0.0,
            fuel_reported: false,
            missile_expire_time: 0.0,
            expire_reported: false,
            current_height: 0.0,
            current_acceleration: Vector::ZERO,
            current_velocity: Vector::ZERO,
            num_missile_steps: 5,
            min_time_step: 0.1,
            time_difference_limit: 0.001,
            is_activated: true,
            ready_to_place: false,
            ready_to_fire: false,
            is_moving: false,
        }
    }
}

impl MissileManager {
    /// `CMissileManager::Place`.
    pub fn place(&mut self) {
        self.ready_to_place = true;
    }

    /// `CMissileManager::Fire`.
    pub fn fire(&mut self) {
        self.ready_to_fire = true;
    }

    /// `CMissileManager::PlaceAndFire`.
    pub fn place_and_fire(&mut self) {
        self.ready_to_place = true;
        self.ready_to_fire = true;
    }

    /// `CMissileManager::SetActivated`.
    pub fn set_activated(&mut self, activated: bool) {
        self.is_activated = activated;
    }

    /// `CMissileManager::GetCurrentVelocity`.
    pub fn current_velocity(&self) -> Vector {
        self.current_velocity
    }

    /// The expiry and fuel the parameters arm (`ClientInitMissileParameters` and
    /// `AttemptWorldPlacement` do this after the parameters are copied in): the lifespan from
    /// `now` (`Timer::cur_time`, added as a float) and the full tank.
    pub fn arm(&mut self, now: f64) {
        if self.info.missile_has_lifespan {
            self.expire_reported = false;
            self.missile_expire_time = f64::from(st(ld(self.info.missile_lifespan) + ld(st(now))));
        }
        if self.info.uses_fuel {
            self.fuel_reported = false;
            self.current_fuel = self.info.total_fuel;
        }
    }

    /// `CMissileManager::IsInWorld`.
    pub fn is_in_world(&self, host: &impl MissileHost) -> bool {
        host.is_in_world()
    }

    /// `CMissileManager::SetMoving`: moving starts the flight FX and streak, stopping ends them.
    pub fn set_moving(&mut self, host: &mut impl MissileHost, moving: bool) {
        self.is_moving = moving;
        if moving {
            host.start_streak();
            host.do_fx(MISSILE_FLIGHT_FX, 1.0);
        } else {
            host.stop_fx(MISSILE_FLIGHT_FX);
            host.stop_streak();
        }
    }

    /// `CMissileManager::OutOfFuel`: it burns fuel and the tank is empty.
    pub fn out_of_fuel(&self) -> bool {
        self.info.uses_fuel && self.current_fuel == 0.0
    }

    /// `CMissileManager::ReportOutOfFuel`.
    pub fn report_out_of_fuel(&mut self, host: &mut impl MissileHost) {
        host.report_out_of_fuel();
        self.fuel_reported = true;
    }

    /// `CMissileManager::UpdateFuelUse`: the tank loses the acceleration's magnitude times the
    /// elapsed time (a double), and stops at empty.
    pub fn update_fuel_use(&mut self, quantum: f64) {
        let a = self.current_acceleration;
        let magnitude = ((ld(a.x) * ld(a.x) + ld(a.y) * ld(a.y)) + ld(a.z) * ld(a.z)).sqrt();
        let fuel = ld(self.current_fuel) - magnitude * quantum;
        self.current_fuel = st(fuel);
        if fuel < 0.0 {
            self.current_fuel = 0.0;
        }
    }

    /// `CMissileManager::AdjustHeight`: ground-following sets the climb (`follow` 2) or raises
    /// it to the wanted rate (`follow` 1) from the height left to the wanted one.
    pub fn adjust_height(&self, velocity: &mut Vector) {
        let wanted = ld(self.info.height_above_ground) - ld(self.current_height);
        match self.info.ground_follow_type {
            2 => velocity.z = st(wanted),
            1 if ld(velocity.z) < wanted => velocity.z = st(wanted),
            _ => {}
        }
    }

    /// `CMissileManager::AdjustSpeed`: the velocity's squared length (kept as a float) is held
    /// between the speed limits; a velocity too small to aim is sent up the y axis.
    pub fn adjust_speed(&self, velocity: &mut Vector) {
        let (min, max) = (self.info.min_velocity, self.info.max_velocity);
        let v = *velocity;
        let length2 = st((ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y)) + ld(v.z) * ld(v.z));
        let tiny = epsilon_squared();
        if ld(length2) < ld(min) * ld(min) {
            if tiny <= ld(length2) {
                let k = ld(min) / ld(length2).sqrt();
                *velocity = Vector::new(st(k * ld(v.x)), st(k * ld(v.y)), st(k * ld(v.z)));
            } else {
                *velocity = Vector::new(0.0, min, 0.0);
            }
        }
        if ld(max) * ld(max) < ld(length2) {
            if tiny <= ld(length2) {
                let v = *velocity;
                let sum = (ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y)) + ld(v.z) * ld(v.z);
                let k = ld(max) / sum.sqrt();
                *velocity = Vector::new(st(k * ld(v.x)), st(k * ld(v.y)), st(k * ld(v.z)));
            } else {
                *velocity = Vector::new(0.0, max, 0.0);
            }
        }
    }

    /// `CMissileManager::UpdateMissilePhysics`: one quantum of flight. Not moving, the offset is
    /// the identity frame. Moving, the acceleration gains gravity and loses drag, the wind adds
    /// to the velocity, and `offset` gains `v q + a q^2 / 2`; the velocity gains `a q`, becomes
    /// the current velocity and the body's missile velocity. A lifespan that has run out
    /// (`now` past the expiry) is reported once. `velocity` is the body's velocity.
    pub fn update_missile_physics(
        &mut self,
        host: &mut impl MissileHost,
        velocity: Vector,
        quantum: f32,
        offset: &mut Frame,
        now: f64,
    ) {
        if !self.is_moving {
            *offset = Frame::IDENTITY;
            return;
        }
        let mut v = velocity;
        let mut a = self.current_acceleration;
        if self.info.affected_by_gravity {
            let g = host.current_gravity();
            a = Vector::new(
                st(ld(a.x) + ld(g.x)),
                st(ld(a.y) + ld(g.y)),
                st(ld(a.z) + ld(g.z)),
            );
        }
        if self.info.drag != 0.0 {
            let d = ld(self.info.drag);
            a = Vector::new(
                st(ld(a.x) - ld(v.x) * d),
                st(ld(a.y) - ld(st(ld(v.y) * d))),
                st(ld(a.z) - ld(st(ld(v.z) * d))),
            );
        }
        if self.info.affected_by_wind {
            let w = host.current_wind();
            v = Vector::new(
                st(ld(v.x) + ld(w.x)),
                st(ld(v.y) + ld(w.y)),
                st(ld(v.z) + ld(w.z)),
            );
        }
        let q = ld(quantum);
        let half = ld(0.5_f32);
        let x_term = st(ld(st(ld(a.x) * half * q)) * q);
        let y_term = st(ld(a.y) * half * q * q);
        let z_term = ld(a.z) * half * q * q;
        let vz_step = st(ld(v.z) * q);
        let dx = st(ld(v.x) * q + ld(x_term));
        let dy = ld(v.y) * q + ld(y_term);
        let dz = st(ld(vz_step) + z_term);
        let o = offset.origin;
        offset.origin = Vector::new(st(ld(dx) + ld(o.x)), st(dy + ld(o.y)), st(ld(dz) + ld(o.z)));
        let az_step = st(ld(a.z) * q);
        let v = Vector::new(
            st(ld(a.x) * q + ld(v.x)),
            st(ld(a.y) * q + ld(v.y)),
            st(ld(az_step) + ld(v.z)),
        );
        self.current_velocity = v;
        host.set_missile_velocity(v);
        if self.info.missile_has_lifespan && self.missile_expire_time < now && !self.expire_reported
        {
            host.report_expired();
            self.expire_reported = true;
        }
    }
}

/// `CMissileManager::GetMinTime`: the least time to cover `dist` against a target moving at
/// `target_velocity` along the axis, at `max_speed`, raising `min_time` to it. False when the
/// missile can never close the distance.
pub fn get_min_time(dist: f32, target_velocity: f32, min_time: &mut f32, max_speed: f32) -> bool {
    let mut speed = ld(max_speed);
    if ld(dist) <= 0.0 {
        if 0.0 <= ld(dist) {
            return true;
        }
        speed = -speed;
        if ld(target_velocity) <= speed {
            return false;
        }
    } else if speed <= ld(target_velocity) {
        return false;
    }
    let t = ld(dist) / (speed - ld(target_velocity));
    if t > ld(*min_time) {
        *min_time = st(t);
    }
    true
}

/// `CMissileManager::GetMaxTime`: the later root of `dist = (max_speed - target_velocity) t +
/// accel t^2 / 2` (the speed negated for an accelerating target), stored as a float; false when
/// there is no real root or it is negative.
pub fn get_max_time(
    dist: f32,
    target_velocity: f32,
    accel: f32,
    max_time: &mut f32,
    max_speed: f32,
) -> bool {
    let mut speed = ld(max_speed);
    if 0.0 < ld(accel) {
        speed *= -1.0;
    }
    let d = speed - ld(target_velocity);
    let discriminant = d * d + (ld(dist) * ld(accel) + ld(dist) * ld(accel));
    if discriminant < 0.0 {
        return false;
    }
    let root = discriminant.sqrt();
    let inverse = 1.0 / ld(accel);
    let first = st((root - d) * inverse);
    let second = (-d - root) * inverse;
    *max_time = if ld(first) < second {
        st(second)
    } else {
        first
    };
    0.0 <= ld(*max_time)
}

/// `CMissileManager::CalcMinMaxTime`: the earliest and latest times a missile of
/// `max_desired_speed` can meet a target `diff` away moving at `target_velocity` under
/// `accel`. The earliest starts at `F_EPSILON` and rises over each axis with no acceleration
/// (`GetMinTime`); the latest starts at twice that and is the later root on the first axis (z,
/// then y, then x) that accelerates (`GetMaxTime`). Returns `(possible, t_min, t_max)`;
/// impossible when an axis can never be closed or the latest is before the earliest.
pub fn calc_min_max_time(
    diff: Vector,
    target_velocity: Vector,
    accel: Vector,
    max_desired_speed: f32,
) -> (bool, f32, f32) {
    let eps = ld(F_EPSILON);
    let mut t_min = F_EPSILON;
    let mut possible = true;
    if ld(accel.z).abs() < eps {
        possible &= get_min_time(diff.z, target_velocity.z, &mut t_min, max_desired_speed);
    }
    if ld(accel.y).abs() < eps {
        possible &= get_min_time(diff.y, target_velocity.y, &mut t_min, max_desired_speed);
    }
    if ld(accel.x).abs() < eps {
        possible &= get_min_time(diff.x, target_velocity.x, &mut t_min, max_desired_speed);
    }
    let mut t_max = st(ld(t_min) + ld(t_min));
    let axis = if ld(accel.z).abs() > eps {
        Some((diff.z, target_velocity.z, accel.z))
    } else if ld(accel.y).abs() > eps {
        Some((diff.y, target_velocity.y, accel.y))
    } else if ld(accel.x).abs() > eps {
        Some((diff.x, target_velocity.x, accel.x))
    } else {
        None
    };
    if let Some((dist, velocity, a)) = axis {
        possible &= get_max_time(dist, velocity, a, &mut t_max, max_desired_speed);
    }
    if t_max < t_min {
        possible = false;
    }
    (possible, t_min, t_max)
}

/// `CMissileManager::GetGravitationalVelocity`: the velocity that meets a target `diff` away
/// moving at `target_velocity` after `time` seconds under `gravity`. Nothing for a time that
/// is not positive.
pub fn get_gravitational_velocity(
    diff: Vector,
    target_velocity: Vector,
    gravity: Vector,
    time: f32,
) -> Option<Vector> {
    if !(0.0 < ld(time)) {
        return None;
    }
    let t = ld(time);
    let half = ld(0.5_f32);
    let a_x = st(t * ld(gravity.x) * half);
    let a_y = t * ld(gravity.y) * half;
    let a_z = st(ld(st(t * ld(gravity.z))) * half);
    let inverse = 1.0 / t;
    let b_x = inverse * ld(diff.x) + ld(target_velocity.x);
    let b_y = st(ld(st(inverse * ld(diff.y))) + ld(target_velocity.y));
    let b_z = st(ld(st(inverse * ld(diff.z))) + ld(target_velocity.z));
    Some(Vector::new(
        st(b_x - ld(a_x)),
        st(ld(b_y) - a_y),
        st(ld(b_z) - ld(a_z)),
    ))
}

/// The direction `D3DXVec3Normalize` gives (the d3dx8 DLL; float math here).
fn d3dx_normalize(v: Vector) -> Vector {
    let length = (ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y) + ld(v.z) * ld(v.z)).sqrt();
    if length == 0.0 {
        return Vector::ZERO;
    }
    let inverse = st(1.0 / length);
    Vector::new(
        st(ld(v.x) * ld(inverse)),
        st(ld(v.y) * ld(inverse)),
        st(ld(v.z) * ld(inverse)),
    )
}

/// `CMissileManager::ChaseVelocity`: the velocity of length `length` that heads along the sum of
/// the target's and the missile's directions (the target's own direction when the missile's
/// speed is nearly zero; `speed` itself when the target's is).
pub fn chase_velocity(target_velocity: Vector, speed: Vector, length: f32) -> Vector {
    let t = target_velocity;
    let eps = ld(F_EPSILON);
    let target_length = ((ld(t.z) * ld(t.z) + ld(t.y) * ld(t.y)) + ld(t.x) * ld(t.x)).sqrt();
    if target_length < eps {
        return speed;
    }
    let inverse_target = st(1.0 / target_length);
    let it = ld(inverse_target);
    let target_x = ld(t.x) * it;
    let target_y = st(ld(t.y) * it);
    let target_z = st(ld(t.z) * it);
    let s = speed;
    let speed_length = ((ld(s.z) * ld(s.z) + ld(s.y) * ld(s.y)) + ld(s.x) * ld(s.x)).sqrt();
    if speed_length < eps {
        return Vector::new(st(target_x), target_y, target_z);
    }
    let inverse_speed = 1.0 / speed_length;
    let n_x = st(inverse_speed * ld(s.x));
    let n_y = st(ld(s.y) * inverse_speed);
    let n_z = st(ld(s.z) * inverse_speed);
    let sum = Vector::new(
        st(target_x + ld(n_x)),
        st(ld(n_y) + ld(target_y)),
        st(ld(n_z) + ld(target_z)),
    );
    let d = d3dx_normalize(sum);
    let l = ld(length);
    Vector::new(st(ld(d.x) * l), st(ld(d.y) * l), st(ld(d.z) * l))
}

#[cfg(test)]
#[path = "missile_tests.rs"]
mod tests;
