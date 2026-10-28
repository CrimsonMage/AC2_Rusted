//! `CMissileManager`'s ballistic solvers: the launch velocity of a missile that gravity bends
//! (`CalcGravitationalVelocity`, `CalcGravitationalClosest`, `CalcGravitationalPitch`,
//! `CalcGravitationalSpeed`, `GreaterPitch`). Retail passes the interval ends in registers and
//! recurses; the argument order here and the x87 spills follow the disassembly (the decompile
//! garbles which values are passed).

use crate::fpu::{ld, st, F_EPSILON};
use crate::frame::FIVE_HALVES_PI;
use crate::heading::Heading;
use crate::missile::{
    calc_min_max_time, chase_velocity, get_gravitational_velocity, MissileManager,
};
use crate::vector::Vector;

/// `MissileTargetingParameters` bit `0x100` (tested for nothing by the solvers).
const SQUARED_CHECK_EPSILON: f32 = 0.000_2;
/// The `float` constant retail adds to an interval before halving it.
const INTERVAL_PAD: f32 = SQUARED_CHECK_EPSILON;

/// `(x*x + y*y) + z*z` as the comparisons in the solvers' base cases sum it.
fn length_squared_xyz(v: Vector) -> f64 {
    (ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y)) + ld(v.z) * ld(v.z)
}

/// `(z*z + y*y) + x*x`, the order of the distance sums in `CalcGravitationalClosest`.
fn length_squared_zyx(v: Vector) -> f64 {
    (ld(v.z) * ld(v.z) + ld(v.y) * ld(v.y)) + ld(v.x) * ld(v.x)
}

/// `(z*z + x*x) + y*y`, the order `CalcGravitationalPitch` sums a fresh velocity in.
fn length_squared_zxy(v: Vector) -> f64 {
    (ld(v.z) * ld(v.z) + ld(v.x) * ld(v.x)) + ld(v.y) * ld(v.y)
}

/// The squared distance between two velocities, the differences kept on the x87 stack.
fn distance_squared(a: Vector, b: Vector) -> f64 {
    let (dx, dy, dz) = (ld(a.x) - ld(b.x), ld(a.y) - ld(b.y), ld(a.z) - ld(b.z));
    (dz * dz + dy * dy) + dx * dx
}

/// `CMissileManager::GreaterPitch`: only a velocity shorter than `F_EPSILON` (the length is
/// `sqrt((x*x + y*y) + z*z)`) compares its z against `z_comp`; a longer one is never greater.
pub fn greater_pitch(velocity: Vector, z_comp: f32) -> bool {
    if ld(F_EPSILON) <= length_squared_xyz(velocity).sqrt() {
        return false;
    }
    ld(velocity.z) > ld(z_comp)
}

/// The velocity `Vector::FromSpherical(length, heading, 45)` gives after the missile chases the
/// target at `diff` (`ChaseVelocity`): the chase velocity's compass heading, 45 degrees up. The
/// fallback of `CalcGravitationalVelocity` and of the gravity path of `MissileInfoUpdate`.
pub(crate) fn chase_launch_velocity(diff: Vector, target_velocity: Vector, length: f32) -> Vector {
    let chase = chase_velocity(diff, target_velocity, length);
    let (x, y) = (ld(chase.x), ld(chase.y));
    let flat = (y * y + x * x).sqrt();
    let heading = if flat < ld(F_EPSILON) {
        Heading::ZERO
    } else {
        let inverse = 1.0 / flat;
        let nx = ld(st(inverse * x));
        Heading::from_radians(st(FIVE_HALVES_PI - (inverse * y).atan2(nx)))
    };
    Vector::from_spherical(length, heading.degrees(), 45.0)
}

impl MissileManager {
    /// `CMissileManager::CalcGravitationalClosest`: the launch velocity among those that meet
    /// the target between `t_min` and `t_max` whose time-of-flight velocity is closest to
    /// `desired` (the missile's current velocity). `v_min` and `v_max` are the velocities at the
    /// interval ends. Steps through the interval, then narrows onto the step where the distance
    /// stopped falling; the interval shorter than `m_time_difference_limit` picks an end by
    /// comparing the squared lengths of `desired` and `v_min`.
    #[allow(clippy::too_many_arguments)]
    pub fn calc_gravitational_closest(
        &self,
        t_min: f32,
        t_max: f32,
        diff: Vector,
        target_velocity: Vector,
        accel: Vector,
        desired: Vector,
        v_min: Vector,
        v_max: Vector,
    ) -> Vector {
        let span = ld(t_max) - ld(t_min);
        if span < ld(self.time_difference_limit) {
            return if length_squared_xyz(desired) <= length_squared_xyz(v_min) {
                v_max
            } else {
                v_min
            };
        }
        let steps = f64::from(self.num_missile_steps);
        let step_extended = (span + ld(INTERVAL_PAD)) / steps;
        let step = st(step_extended);
        let mut best = st(distance_squared(v_min, desired));
        let (mut older, mut last) = (t_min, t_min);
        let (mut v_older, mut v_last) = (v_min, v_min);
        let mut time = st(step_extended + ld(t_min));
        loop {
            if ld(t_max) <= ld(time) {
                // Ran off the end: narrow on whichever of the last step and the top end is better.
                let d_max = distance_squared(v_max, desired);
                return if ld(best) <= d_max {
                    self.calc_gravitational_closest(
                        older,
                        t_max,
                        diff,
                        target_velocity,
                        accel,
                        desired,
                        v_older,
                        v_max,
                    )
                } else {
                    self.calc_gravitational_closest(
                        last,
                        t_max,
                        diff,
                        target_velocity,
                        accel,
                        desired,
                        v_last,
                        v_max,
                    )
                };
            }
            let current =
                get_gravitational_velocity(diff, target_velocity, accel, time).unwrap_or_default();
            let d = distance_squared(current, desired);
            let keep_going = d <= ld(best);
            best = st(d);
            if !keep_going {
                return self.calc_gravitational_closest(
                    older,
                    time,
                    diff,
                    target_velocity,
                    accel,
                    desired,
                    v_older,
                    current,
                );
            }
            older = last;
            v_older = v_last;
            last = time;
            v_last = current;
            time = st(ld(time) + ld(step));
        }
    }

    /// `CMissileManager::CalcGravitationalPitch`: the launch velocity whose speed is `speed`,
    /// found between `t_min` and `t_max` by stepping `step` and halving onto the step where the
    /// speed crosses `speed`. `v_min` and `v_max` are the velocities at the ends. False when the
    /// speed never crosses.
    #[allow(clippy::too_many_arguments)]
    pub fn calc_gravitational_pitch(
        &self,
        t_min: f32,
        t_max: f32,
        step: f32,
        diff: Vector,
        target_velocity: Vector,
        accel: Vector,
        speed: f32,
        v_min: Vector,
        v_max: Vector,
        out: &mut Vector,
    ) -> bool {
        let speed_squared = st(ld(speed) * ld(speed));
        if ld(t_max) - ld(t_min) < ld(self.time_difference_limit) {
            *out = if length_squared_xyz(v_min) < ld(speed_squared) {
                v_min
            } else {
                v_max
            };
            return true;
        }
        let below = length_squared_zxy(v_min) < ld(speed_squared);
        let narrow =
            |me: &Self, from: f32, to: f32, v_from: Vector, v_to: Vector, out: &mut Vector| {
                let half = st((ld(to) - ld(from) + ld(INTERVAL_PAD)) * ld(0.5_f32));
                me.calc_gravitational_pitch(
                    from,
                    to,
                    half,
                    diff,
                    target_velocity,
                    accel,
                    speed,
                    v_from,
                    v_to,
                    out,
                )
            };
        let (mut previous, mut v_previous) = (t_min, v_min);
        let mut time = st(ld(t_min) + ld(step));
        while ld(time) < ld(t_max) {
            let current =
                get_gravitational_velocity(diff, target_velocity, accel, time).unwrap_or(v_min);
            let length2 = length_squared_zxy(current);
            let crossed = if below {
                length2 > ld(speed_squared)
            } else {
                length2 < ld(speed_squared)
            };
            if crossed {
                return narrow(self, previous, time, v_previous, current, out);
            }
            previous = time;
            v_previous = current;
            time = st(ld(time) + ld(step));
        }
        let length2 = length_squared_zyx_hi(v_max);
        let crosses = if below {
            length2 > ld(speed_squared)
        } else {
            length2 < ld(speed_squared)
        };
        if !crosses {
            return false;
        }
        narrow(self, previous, t_max, v_previous, v_max, out)
    }

    /// `CMissileManager::CalcGravitationalSpeed`: as `CalcGravitationalPitch`, but the launch
    /// velocity is the one whose `GreaterPitch` against `z_comp` (the sine of the wanted pitch)
    /// changes; it hands the crossing to `CalcGravitationalPitch`.
    #[allow(clippy::too_many_arguments)]
    pub fn calc_gravitational_speed(
        &self,
        t_min: f32,
        t_max: f32,
        step: f32,
        diff: Vector,
        target_velocity: Vector,
        accel: Vector,
        z_comp: f32,
        v_min: Vector,
        v_max: Vector,
        out: &mut Vector,
    ) -> bool {
        if ld(t_max) - ld(t_min) < ld(self.time_difference_limit) {
            *out = if length_squared_xyz(v_min) < length_squared_xyz(v_max) {
                v_min
            } else {
                v_max
            };
            return true;
        }
        let greater = greater_pitch(v_min, z_comp);
        let (mut previous, mut v_previous) = (t_min, v_min);
        let mut time = st(ld(t_min) + ld(step));
        while ld(time) < ld(t_max) {
            let current =
                get_gravitational_velocity(diff, target_velocity, accel, time).unwrap_or(v_min);
            if greater_pitch(current, z_comp) != greater {
                return self.pitch_between(
                    previous,
                    time,
                    diff,
                    target_velocity,
                    accel,
                    z_comp,
                    v_previous,
                    current,
                    out,
                );
            }
            previous = time;
            v_previous = current;
            time = st(ld(time) + ld(step));
        }
        if greater_pitch(v_max, z_comp) == greater {
            return false;
        }
        self.pitch_between(
            previous,
            t_max,
            diff,
            target_velocity,
            accel,
            z_comp,
            v_previous,
            v_max,
            out,
        )
    }

    /// The `CalcGravitationalPitch` call `CalcGravitationalSpeed` makes on the crossing: the step
    /// is half the interval plus the pad.
    #[allow(clippy::too_many_arguments)]
    fn pitch_between(
        &self,
        from: f32,
        to: f32,
        diff: Vector,
        target_velocity: Vector,
        accel: Vector,
        z_comp: f32,
        v_from: Vector,
        v_to: Vector,
        out: &mut Vector,
    ) -> bool {
        let half = st((ld(to) - ld(from) + ld(INTERVAL_PAD)) * ld(0.5_f32));
        self.calc_gravitational_pitch(
            from,
            to,
            half,
            diff,
            target_velocity,
            accel,
            z_comp,
            v_from,
            v_to,
            out,
        )
    }

    /// `CMissileManager::CalcGravitationalVelocity`: the launch velocity toward a target `diff`
    /// away moving at `target_velocity`, under the body's gravity. With `by_pitch` false
    /// `input` is the speed (`CalcGravitationalPitch` finds the pitch); with it true `input` is
    /// the pitch in degrees and `m_max_velocity` the speed limit (`CalcGravitationalSpeed` takes
    /// its sine). When the time window is empty or the solver finds nothing, the missile heads
    /// 45 degrees up toward `ChaseVelocity`. Always succeeds (retail returns 1).
    pub fn calc_gravitational_velocity(
        &self,
        gravity: Vector,
        input: f32,
        diff: Vector,
        target_velocity: Vector,
        by_pitch: bool,
    ) -> Vector {
        let v_max = self.info.max_velocity;
        let max_desired_speed = if by_pitch { v_max } else { input };
        let (possible, t_min, t_max) =
            calc_min_max_time(diff, target_velocity, gravity, max_desired_speed);
        if possible {
            let v_min = get_gravitational_velocity(diff, target_velocity, gravity, t_min)
                .unwrap_or_default();
            let v_top = get_gravitational_velocity(diff, target_velocity, gravity, t_max)
                .unwrap_or_default();
            let mut step =
                st((ld(t_max) - ld(t_min) + ld(INTERVAL_PAD)) / f64::from(self.num_missile_steps));
            if !(ld(step) >= ld(self.min_time_step)) {
                step = self.min_time_step;
            }
            let mut out = Vector::ZERO;
            let found = if by_pitch {
                let z_comp = st((ld(input) * 0.017_453_292_519_943_295).sin());
                self.calc_gravitational_speed(
                    t_min,
                    t_max,
                    step,
                    diff,
                    target_velocity,
                    gravity,
                    z_comp,
                    v_min,
                    v_top,
                    &mut out,
                )
            } else {
                self.calc_gravitational_pitch(
                    t_min,
                    t_max,
                    step,
                    diff,
                    target_velocity,
                    gravity,
                    input,
                    v_min,
                    v_top,
                    &mut out,
                )
            };
            if found {
                return out;
            }
        }
        chase_launch_velocity(diff, target_velocity, max_desired_speed)
    }
}

/// `(z*z + y*y) + x*x`, the order `CalcGravitationalPitch` sums the top end's velocity in.
fn length_squared_zyx_hi(v: Vector) -> f64 {
    length_squared_zyx(v)
}
