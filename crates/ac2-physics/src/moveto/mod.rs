//! The modules `MoveToManager` steers with: `MovementParameters`, `SimplePathModule` (the
//! straight-line approach to a destination), `TurnModule` (turning to a heading) and the
//! `ProgressChecker` it feeds.

mod manager;

pub(crate) use manager::MoveToManager;
pub use manager::MoveToResult;

use std::collections::VecDeque;

use crate::fpu::{ld, st, F_EPSILON};
use crate::heading::Heading;
use crate::position::Position;
use crate::vector::Vector;

/// `MovementParameters` flag bits.
pub const MOVE_TO_MOVE: u32 = 0x1;
pub const MOVE_TO_TURN: u32 = 0x2;
pub const MOVE_TO_AWAY: u32 = 0x4;
pub const MOVE_TO_PERSISTENT: u32 = 0x8;
pub const MOVE_TO_OBJECT: u32 = 0x20;
pub const MOVE_TO_USE_CYLINDERS: u32 = 0x40;

/// `MovementParameters`: what a move-to (or turn-to) aims at and how close counts.
#[derive(Debug, Clone, PartialEq)]
pub struct MovementParameters {
    pub flags: u32,
    pub desired_velocity: f32,
    pub fail_distance: f32,
    pub desired_distance: f32,
    pub max_success_distance: f32,
    pub min_success_distance: f32,
    pub target_position: Position,
    /// Waypoints taken in order before the target position.
    pub position_list: VecDeque<Position>,
    pub offset: Vector,
    pub context: u32,
}

impl Default for MovementParameters {
    /// `MovementParameters::MovementParameters`.
    fn default() -> Self {
        Self {
            flags: MOVE_TO_USE_CYLINDERS,
            desired_velocity: 1.0,
            fail_distance: f32::MAX,
            desired_distance: 1.0,
            max_success_distance: 1.0,
            min_success_distance: 100.0,
            target_position: Position::default(),
            position_list: VecDeque::new(),
            offset: Vector::ZERO,
            context: 0,
        }
    }
}

impl MovementParameters {
    pub(crate) fn has(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }
}

/// `SimplePathModule::update` results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathStatus {
    Continue,
    Success,
    Failure,
}

/// `CRITICAL_MOVE_ANGLE`: farther off than this the body turns before it walks.
const CRITICAL_MOVE_ANGLE: f32 = 90.0;
/// `TURN_EPSILON`.
const TURN_EPSILON: f32 = 10.0;
/// `MINIMUM_SPEED`.
const MINIMUM_SPEED: f32 = 0.0002;

/// `Position::cylinder_distance`: the gap between two upright cylinders (radius and height
/// from their positions' origins up), zero where they overlap on an axis.
pub(crate) fn cylinder_distance(
    radius: f32,
    height: f32,
    pos: &Position,
    other_radius: f32,
    other_height: f32,
    other: &Position,
) -> f64 {
    let off = pos.offset_to(other);
    let (x, y) = (ld(off.x), ld(off.y));
    let mut horizontal = (y * y + x * x).sqrt() - (ld(radius) + ld(other_radius));
    if !(horizontal >= 0.0) {
        horizontal = 0.0;
    }
    let (z, oz) = (ld(pos.frame.origin.z), ld(other.frame.origin.z));
    let mut vertical = if z > oz {
        z - (oz + ld(other_height))
    } else {
        oz - (z + ld(height))
    };
    if !(vertical >= 0.0) {
        vertical = 0.0;
    }
    (vertical * vertical + horizontal * horizontal).sqrt()
}

/// `SimplePathModule`.
#[derive(Debug, Clone)]
pub(crate) struct SimplePathModule {
    pub destination: Position,
    radius: f32,
    height: f32,
    target_radius: f32,
    target_height: f32,
    always_turn: bool,
    max_success_distance: f32,
    use_cylinders: bool,
    preferred_success_distance: f32,
    fail_distance: f32,
    desired_velocity: f32,
}

impl Default for SimplePathModule {
    fn default() -> Self {
        Self {
            destination: Position::default(),
            radius: 0.0,
            height: 0.0,
            target_radius: 0.0,
            target_height: 0.0,
            always_turn: false,
            max_success_distance: 5.0,
            use_cylinders: true,
            preferred_success_distance: 0.1,
            fail_distance: 100.0,
            desired_velocity: 0.0,
        }
    }
}

impl SimplePathModule {
    /// `SimplePathModule::set_parameters`, `set_always_turn` and `set_use_cylinders`.
    pub(crate) fn configure(&mut self, radius: f32, height: f32, params: &MovementParameters) {
        self.radius = radius;
        self.height = height;
        self.preferred_success_distance = params.desired_distance;
        self.max_success_distance = params.max_success_distance;
        self.fail_distance = params.fail_distance;
        self.desired_velocity = params.desired_velocity;
        self.always_turn = params.has(MOVE_TO_TURN);
        self.use_cylinders = params.has(MOVE_TO_USE_CYLINDERS);
    }

    /// `SimplePathModule::set_destination`.
    pub(crate) fn set_destination(&mut self, destination: Position, radius: f32, height: f32) {
        self.destination = destination;
        self.target_radius = radius;
        self.target_height = height;
    }

    fn distance(&self, pos: &Position) -> f64 {
        if self.use_cylinders {
            cylinder_distance(
                self.radius,
                self.height,
                pos,
                self.target_radius,
                self.target_height,
                &self.destination,
            )
        } else {
            pos.distance(&self.destination)
        }
    }

    /// `SimplePathModule::CalculateXYDistanceToDestination`: the distance at the
    /// destination's height.
    fn xy_distance(&self, pos: &Position) -> f64 {
        let mut level = *pos;
        level.frame.origin.z = self.destination.frame.origin.z;
        self.distance(&level)
    }

    /// `SimplePathModule::update`: the heading to the destination and, facing within 90
    /// degrees of it, the walk toward it (slowing over the last half of the success distance,
    /// at least 0.3), every axis at most the desired velocity.
    pub(crate) fn update(&self, current: &Position) -> (Vector, Heading, PathStatus) {
        let d = st(self.distance(current));
        let heading = current.heading_to(&self.destination);
        let facing = current.frame.heading();
        let preferred = ld(self.preferred_success_distance);
        let mut motion = Vector::ZERO;
        if ld(d) > preferred
            && facing.is_in_range(heading, Heading::from_degrees(CRITICAL_MOVE_ANGLE))
        {
            let off = current.offset_to(&self.destination);
            let local = current.frame.global_to_local_vec(off);
            let (x, y) = (ld(local.x), ld(local.y));
            let length = (x * x + y * y).sqrt();
            motion = if length < ld(F_EPSILON) {
                Vector::ZERO
            } else {
                let inverse = 1.0 / length;
                Vector::new(st(inverse * x), st(inverse * y), st(inverse * 0.0))
            };
            let half = st(ld(self.max_success_distance) * 0.5);
            if ld(d) < ld(half) && ld(d) > preferred {
                let mut f = (ld(d) - preferred) / (ld(half) - preferred);
                if f < ld(0.3_f32) {
                    f = ld(0.3_f32);
                }
                motion = Vector::new(
                    st(f * ld(motion.x)),
                    st(f * ld(motion.y)),
                    st(f * ld(motion.z)),
                );
            }
        }
        let cap = |v: f32| {
            if ld(v) < ld(self.desired_velocity) {
                v
            } else {
                self.desired_velocity
            }
        };
        motion = Vector::new(cap(motion.x), cap(motion.y), cap(motion.z));
        let status = if !(ld(d) < ld(self.fail_distance)) {
            PathStatus::Failure
        } else if ld(d) <= preferred {
            PathStatus::Success
        } else if !(self.xy_distance(current) < preferred * ld(0.2_f32)) {
            PathStatus::Continue
        } else if ld(d) > ld(self.max_success_distance) {
            PathStatus::Failure
        } else {
            PathStatus::Continue
        };
        (motion, heading, status)
    }
}

/// `TurnModule`.
#[derive(Debug, Clone, Default)]
pub(crate) struct TurnModule {
    pub desired_heading: Heading,
}

impl TurnModule {
    /// `TurnModule::update`: done within 10 degrees of the desired heading.
    pub(crate) fn update(&self, current: Heading) -> (Heading, PathStatus) {
        let off = Heading::difference(current, self.desired_heading).abs();
        let status = if off.is_in_range(Heading::ZERO, Heading::from_degrees(TURN_EPSILON)) {
            PathStatus::Success
        } else {
            PathStatus::Continue
        };
        (self.desired_heading, status)
    }
}

/// `ProgressChecker`: the speeds made toward a goal against the speeds asked for over the last
/// second.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProgressChecker {
    history: VecDeque<(f64, f32)>,
    desired: VecDeque<(f64, f32)>,
}

impl ProgressChecker {
    /// `ProgressChecker::reset`.
    pub(crate) fn reset(&mut self) {
        self.history.clear();
        self.desired.clear();
    }

    /// `ProgressChecker::push`: a speed made.
    pub(crate) fn push(&mut self, now: f64, speed: f32) {
        self.history.push_back((now, speed));
    }

    /// `ProgressChecker::check_progress`: false once the last second's speed made falls under a
    /// tenth of the speed asked for (both over more than four samples).
    pub(crate) fn check_progress(&mut self, now: f64, desired_speed: f32) -> bool {
        if desired_speed < MINIMUM_SPEED {
            return true;
        }
        let cutoff = now - 1.0;
        while self.history.front().is_some_and(|&(time, _)| time < cutoff) {
            self.history.pop_front();
        }
        while self.desired.front().is_some_and(|&(time, _)| time < cutoff) {
            self.desired.pop_front();
        }
        self.desired.push_back((now, desired_speed));
        if self.history.len() > 4 && self.desired.len() > 4 {
            let made: f64 = self.history.iter().map(|&(_, s)| ld(s)).sum();
            let asked: f64 = self.desired.iter().map(|&(_, s)| ld(s)).sum();
            let average_made = made / self.history.len() as f64;
            let threshold = asked / self.desired.len() as f64 * ld(0.1_f32);
            if !(threshold <= average_made) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
