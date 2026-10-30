//! `MoveToManager`: the player's autonomous move-to and turn-to (walking up to a target or a
//! list of waypoints, then facing a heading), steering `CMotionInterp` each quantum.

use crate::fpu::{ld, st, F_EPSILON};
use crate::heading::Heading;
use crate::physics_obj::PhysicsObj;
use crate::position::Position;
use crate::vector::Vector;

use super::{
    MovementParameters, PathStatus, ProgressChecker, SimplePathModule, TurnModule, MOVE_TO_AWAY,
    MOVE_TO_MOVE, MOVE_TO_OBJECT, MOVE_TO_PERSISTENT, MOVE_TO_TURN,
};

/// `MoveToManager::Stop` errors.
const ERROR_INTERRUPTED: u32 = 0x36;
const ERROR_TARGET_LOST: u32 = 0x37;

/// `TargetStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetStatus {
    Undefined,
    Ok,
    /// Any other status the target manager reports (out of range, left the world, ...).
    Lost,
}

/// What a finished move-to tells its caller (`CMotionInterp::HandleMoveToCallback` with the
/// `MovementReturn`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveToResult {
    pub context: u32,
    /// 0 done, 0x36 interrupted, 0x37 target lost.
    pub error: u32,
    pub target: Position,
}

#[derive(Debug, Clone)]
pub(crate) struct MoveToManager {
    path: Option<SimplePathModule>,
    progress: ProgressChecker,
    turn: Option<TurnModule>,
    params: MovementParameters,
    target_position: Position,
    target_height: f32,
    target_radius: f32,
    target_status: TargetStatus,
    pub moving_to: bool,
    last_update_position: Option<Position>,
    results: Vec<MoveToResult>,
}

impl Default for MoveToManager {
    fn default() -> Self {
        Self {
            path: None,
            progress: ProgressChecker::default(),
            turn: None,
            params: MovementParameters::default(),
            target_position: Position::default(),
            target_height: 0.0,
            target_radius: 0.0,
            target_status: TargetStatus::Undefined,
            moving_to: false,
            last_update_position: None,
            results: Vec::new(),
        }
    }
}

impl MoveToManager {
    pub(crate) fn take_results(&mut self) -> Vec<MoveToResult> {
        std::mem::take(&mut self.results)
    }

    /// `MoveToManager::UpdateTargetInfo` (from the target manager, for an object target).
    pub(crate) fn update_target(&mut self, position: Position, radius: f32, height: f32) {
        self.target_position = position;
        self.target_radius = radius;
        self.target_height = height;
        self.target_status = TargetStatus::Ok;
    }

    /// `MoveToManager::UpdateTargetInfo`: the target manager's record (position, size, status)
    /// replaces what the move-to knows of its object target.
    pub(crate) fn update_target_info(&mut self, info: &crate::target::TargetInfo) {
        use crate::target::TargetStatus as Reported;
        self.target_position = info.target_position;
        self.target_radius = info.object_radius;
        self.target_height = info.object_height;
        self.target_status = match info.status {
            Reported::Ok => TargetStatus::Ok,
            Reported::Undef => TargetStatus::Undefined,
            _ => TargetStatus::Lost,
        };
    }

    /// `UpdateTargetInfo` with a status other than `Ok`.
    pub(crate) fn lose_target(&mut self) {
        self.target_status = TargetStatus::Lost;
    }
}

impl PhysicsObj {
    /// `CMotionInterp::MoveToEvent` -> `MoveToManager::MoveToEvent`: start a move-to (and/or
    /// turn-to); a move with neither flag finishes at once.
    pub(crate) fn move_to_event(&mut self, params: MovementParameters) -> bool {
        let mut manager = self.move_to.take().unwrap_or_default();
        if manager.moving_to {
            self.stop_move_to(&mut manager, ERROR_INTERRUPTED);
        }
        manager.params = params;
        self.translate_for_hovering(&mut manager.params);
        manager.moving_to = true;
        manager.target_status = TargetStatus::Undefined;
        let p = &manager.params;
        let started = if !p.has(MOVE_TO_MOVE) && !p.has(MOVE_TO_TURN) && !p.has(MOVE_TO_AWAY) {
            self.stop_move_to(&mut manager, 0);
            true
        } else {
            (!manager.params.has(MOVE_TO_MOVE) || self.start_move_to(&mut manager))
                && (!manager.params.has(MOVE_TO_TURN) || Self::start_turn_to(&mut manager))
        };
        self.move_to = Some(manager);
        started
    }

    /// `CMotionInterp::InterruptMoveTo` -> `MoveToManager::Interrupt`.
    pub(crate) fn interrupt_move_to(&mut self) {
        let Some(mut manager) = self.move_to.take() else {
            return;
        };
        if manager.moving_to {
            self.stop_move_to(&mut manager, ERROR_INTERRUPTED);
        }
        self.move_to = Some(manager);
    }

    /// `MoveToManager::TranslateForHovering`: a position target of a body without gravity
    /// is raised by its default hover height.
    fn translate_for_hovering(&self, params: &mut MovementParameters) {
        if params.has(MOVE_TO_OBJECT) {
            return;
        }
        let values = self.motion.motion_values(self.desired_element());
        if !values.gravity {
            params.offset.z = st(ld(params.offset.z) + ld(values.hover_height));
        }
    }

    /// `MoveToManager::StartMoveTo`: aim the path module at the next waypoint (or the target
    /// position) plus the offset.
    fn start_move_to(&mut self, manager: &mut MoveToManager) -> bool {
        manager.progress.reset();
        let radius = self.radius();
        let height = self.height();
        let path = manager.path.get_or_insert_with(SimplePathModule::default);
        path.configure(radius, height, &manager.params);
        if !manager.params.has(MOVE_TO_OBJECT) {
            let mut destination = match manager.params.position_list.pop_front() {
                Some(waypoint) => waypoint,
                None => manager.params.target_position,
            };
            destination.add_local_offset(manager.params.offset);
            path.set_destination(destination, 0.0, 0.0);
        }
        true
    }

    /// `MoveToManager::StartTurnTo`: aim the turn module at the target position's heading.
    fn start_turn_to(manager: &mut MoveToManager) -> bool {
        let turn = manager.turn.get_or_insert_with(TurnModule::default);
        if !manager.params.has(MOVE_TO_OBJECT) {
            turn.desired_heading = manager.params.target_position.frame.heading();
        }
        true
    }

    /// `MoveToManager::Stop`: stand still, stop turning and report to the caller.
    fn stop_move_to(&mut self, manager: &mut MoveToManager, error: u32) {
        if !manager.moving_to {
            return;
        }
        manager.moving_to = false;
        manager.progress.reset();
        self.motion.do_motion(Vector::ZERO);
        let heading = self.position.frame.heading();
        self.motion.stop_turn_to(heading);
        manager.results.push(MoveToResult {
            context: manager.params.context,
            error,
            target: manager.target_position,
        });
        manager.params.context = 0;
        manager.target_status = TargetStatus::Undefined;
        manager.target_radius = 0.0;
        manager.target_height = 0.0;
    }

    /// `MoveToManager::Update`: the move, then the turn; with both done, the next waypoint.
    pub(crate) fn move_to_update(&mut self, quantum: f32) -> Option<(Vector, Heading)> {
        let mut manager = self.move_to.take()?;
        if !manager.moving_to {
            self.move_to = Some(manager);
            return None;
        }
        let mut motion = self.motion.last_do_motion();
        let mut heading = self.motion.turn_to_direction();
        if self.update_move_to(&mut manager, quantum, &mut motion, &mut heading)
            && self.update_turn_to(&mut manager, &mut heading)
            && !manager.params.position_list.is_empty()
        {
            self.start_move_to(&mut manager);
        }
        self.move_to = Some(manager);
        Some((motion, heading))
    }

    /// `MoveToManager::UpdateMoveTo`: lift the interpolation caps, record the progress made
    /// toward the destination, follow an object target and step the path module. True once
    /// the destination is reached.
    fn update_move_to(
        &mut self,
        manager: &mut MoveToManager,
        quantum: f32,
        motion: &mut Vector,
        heading: &mut Heading,
    ) -> bool {
        if !manager.params.has(MOVE_TO_MOVE) {
            return true;
        }
        if manager.path.is_none() {
            return false;
        }
        self.distance_cap_2d = -1.0;
        self.velocity_rail = None;
        let current = self.position;
        if let Some(last) = manager.last_update_position {
            let destination = if manager.params.has(MOVE_TO_OBJECT) {
                manager.target_position
            } else {
                manager
                    .path
                    .as_ref()
                    .map_or(current, |path| path.destination)
            };
            let off = current.offset_to(&destination);
            let (x, y, z) = (ld(off.x), ld(off.y), ld(off.z));
            let length = ((z * z + y * y) + x * x).sqrt();
            let direction = if length < ld(F_EPSILON) {
                Vector::ZERO
            } else {
                let inverse = 1.0 / length;
                Vector::new(st(x * inverse), st(y * inverse), st(z * inverse))
            };
            let moved = last.offset_to(&current);
            let mut made = (ld(moved.z) * ld(direction.z) + ld(moved.y) * ld(direction.y))
                + ld(moved.x) * ld(direction.x);
            if !(made >= 0.0) {
                made = 0.0;
            }
            if ld(quantum) > 0.0 {
                manager.progress.push(self.cur_time, st(made / ld(quantum)));
            }
        }
        manager.last_update_position = Some(current);
        if manager.params.has(MOVE_TO_OBJECT) {
            match manager.target_status {
                TargetStatus::Ok => {}
                TargetStatus::Undefined => return false,
                TargetStatus::Lost => {
                    self.stop_move_to(manager, ERROR_TARGET_LOST);
                    return false;
                }
            }
            let mut destination = manager.target_position;
            destination.add_local_offset(manager.params.offset);
            let (radius, height) = (manager.target_radius, manager.target_height);
            if let Some(path) = manager.path.as_mut() {
                path.set_destination(destination, radius, height);
            }
        }
        let Some(path) = manager.path.as_ref() else {
            return false;
        };
        let (step, toward, status) = path.update(&current);
        *motion = step;
        *heading = toward;
        match status {
            PathStatus::Success => true,
            PathStatus::Failure => false,
            PathStatus::Continue => {
                let values = self.motion.motion_values(self.desired_element());
                let v = self
                    .motion
                    .local_desired_for(step, &values, self.scale, 1.0);
                let v = Vector::new(v.x, v.y, st(ld(step.z) * ld(values.max_forward_velocity)));
                let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
                let speed = st(((z * z + y * y) + x * x).sqrt());
                // Retail ignores the verdict; only the desired speed is recorded.
                manager.progress.check_progress(self.cur_time, speed);
                false
            }
        }
    }

    /// `MoveToManager::UpdateTurnTo`: face the target (an object target's direction) through
    /// the turn module. True once facing it.
    fn update_turn_to(&mut self, manager: &mut MoveToManager, heading: &mut Heading) -> bool {
        if !manager.params.has(MOVE_TO_TURN) {
            return true;
        }
        if manager.turn.is_none() {
            return false;
        }
        let current = self.position;
        if manager.params.has(MOVE_TO_OBJECT) {
            match manager.target_status {
                TargetStatus::Ok => {}
                TargetStatus::Undefined => return false,
                TargetStatus::Lost => {
                    self.stop_move_to(manager, ERROR_TARGET_LOST);
                    return false;
                }
            }
            manager.target_radius = 1.0;
            manager.target_height = 1.0;
            let toward = if current.distance(&manager.target_position) >= ld(F_EPSILON) {
                current.heading_to(&manager.target_position)
            } else {
                current.frame.heading()
            };
            if let Some(turn) = manager.turn.as_mut() {
                turn.desired_heading = toward;
            }
        }
        let Some(turn) = manager.turn.as_ref() else {
            return false;
        };
        let (desired, status) = turn.update(current.frame.heading());
        *heading = desired;
        match status {
            PathStatus::Continue => false,
            PathStatus::Success => true,
            PathStatus::Failure => {
                if !manager.params.has(MOVE_TO_PERSISTENT) {
                    let mut frame = self.position.frame;
                    frame.set_heading(Heading::from_u8(desired.to_u8()));
                    self.position.frame = frame;
                    self.omega = Vector::ZERO;
                }
                true
            }
        }
    }
}
