//! The client-facing body: one `CPhysicsObj` with its `CMotionInterp`, stepped at the retail
//! physics quantum by a `PhysicsModule` clock, and its movement state as plain data.

use std::sync::Arc;

use ac2_world::Ac2MotionInterpDesc;

use crate::fpu::{ld, st};
use crate::heading::Heading;
use crate::motion_interp::MotionInterp;
use crate::physics_obj::{BodyShape, PhysicsObj};
use crate::physics_obj_place::SetPositionError;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::Vector;

/// `PHYSICS_QUANTUM`: `1.0f / 10.0f` (static initialiser, identical in Legions).
pub const PHYSICS_QUANTUM: f32 = 0.1;
/// `PhysicsModule::UpdatePhysics` drops a backlog longer than this (seconds).
const MAX_PHYSICS_LAG: f32 = 1.0;

/// `PhysicsModule`'s fixed-step clock (`m_tPhysicsUpdateTime`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsClock {
    update_time: f64,
}

impl PhysicsClock {
    pub fn new(now: f64) -> Self {
        Self { update_time: now }
    }

    /// `PhysicsModule::UpdatePhysics`: the number of whole quanta to run at `now`. A clock
    /// running backwards, or more than a second behind, resynchronises without stepping.
    pub fn advance(&mut self, now: f64) -> u32 {
        let mut elapsed = st(now - self.update_time);
        if ld(elapsed) < 0.0 {
            self.update_time = now;
            return 0;
        }
        if ld(elapsed) < ld(PHYSICS_QUANTUM) {
            return 0;
        }
        if ld(elapsed) > ld(MAX_PHYSICS_LAG) {
            self.update_time = now;
            return 0;
        }
        let mut quanta = 0;
        while ld(elapsed) >= ld(PHYSICS_QUANTUM) {
            self.update_time = ld(PHYSICS_QUANTUM) + self.update_time;
            elapsed = st(ld(elapsed) - ld(PHYSICS_QUANTUM));
            quanta += 1;
        }
        quanta
    }
}

/// Movement input as retail physics takes it (`CPhysicsObj::DoMotion`): a do-motion vector
/// (strafe right, forward, up; each in `[-1, 1]`, normalised when longer than 1) and the
/// heading to turn to; or a normalised turn rate (`CMotionInterp::SetOmegaVelocity`) for
/// turning without a target heading.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MovementInput {
    pub motion: [f32; 3],
    /// Compass degrees to turn to (`CMotionInterp::TurnTo`).
    pub heading: Option<f32>,
    /// Normalised turn rate in `[-1, 1]`, positive counter-clockwise.
    pub turn: f32,
}

/// The body's movement state each quantum, as plain data for movement reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementState {
    /// `Position::objcell_id`.
    pub cell_id: u32,
    /// `Position::frame` origin, AC axes relative to the cell's landblock.
    pub origin: [f32; 3],
    /// `Position::frame` rotation, retail order `(w, x, y, z)`.
    pub rotation: [f32; 4],
    /// `CPhysicsObj::m_vVelocity`, world axes, m/s.
    pub velocity: [f32; 3],
    /// `CPhysicsObj::m_vOmegaVelocity`, degrees/s.
    pub omega: [f32; 3],
    /// `CMotionInterp::mLastDoMotionValues`.
    pub do_motion: [f32; 3],
    /// The turn rate applied each quantum (`set_turn_rate`).
    pub turn: f32,
    /// `ModeStateKey` element and mode.
    pub element: u32,
    pub mode: u32,
    /// `CONTACT_TS` and `ON_WALKABLE_TS`.
    pub contact: bool,
    pub on_walkable: bool,
    /// `CPhysicsObj::get_heading` (`Frame::get_heading`): compass degrees, `[0, 360)`,
    /// clockwise from north.
    pub heading_degrees: f32,
    /// `CPhysicsObj::m_nTimesJumped`: incremented by every `Jump`.
    pub times_jumped: u32,
    /// `CMotionInterp::IsTurningTo`: turning toward `turn_to_degrees`.
    pub turning_to: bool,
    /// `CMotionInterp::GetTurnToDirection`: the turn target, compass degrees `[0, 360)` (the
    /// current heading after `StopTurnTo`).
    pub turn_to_degrees: f32,
}

/// One moving body (the player): position, shape, motion interpreter and physics state.
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    object: PhysicsObj,
    /// The turn rate `SetOmegaVelocity` gets each quantum.
    turn: f32,
}

impl PhysicsBody {
    /// A body at `position` with its Setup shape and `MotionInterpDesc` (`None` moves nothing).
    pub fn new(
        position: Position,
        shape: BodyShape,
        motion_desc: Option<Arc<Ac2MotionInterpDesc>>,
    ) -> Self {
        Self {
            object: PhysicsObj::new(position, shape, MotionInterp::new(motion_desc)),
            turn: 0.0,
        }
    }

    pub fn position(&self) -> Position {
        self.object.position
    }

    /// Places the body (a server position update): velocity and contact are kept.
    pub fn set_position(&mut self, position: Position) {
        self.object.position = position;
        self.object.prev_position = position;
        self.object.anim_position = position;
    }

    /// `CPhysicsObj::SetPosition`: place the body at `position` through a placement check
    /// (`SLIDE_SPF` lets it move to a free spot nearby, `FORCE_SPF` skips the check). On
    /// failure the body stays where it was.
    pub fn place(&mut self, position: Position, flags: u32, world: &Terrain) -> SetPositionError {
        self.object.set_position_checked(position, flags, world)
    }

    /// `CPhysicsObj::DoMotion` with `TurnTo` when a heading is given, and the turn rate. Input
    /// without a heading leaves a running turn-to alone: only `stop_turn_to` or a new turn
    /// command ends it.
    pub fn set_input(&mut self, input: MovementInput) {
        self.do_motion(input.motion);
        if let Some(degrees) = input.heading {
            self.turn_to(degrees);
        }
        self.set_turn_rate(input.turn);
    }

    /// `CMotionInterp::DoMotion` (through `CPhysicsObj::DoMotion`): the do-motion vector,
    /// strafe right, forward, up.
    pub fn do_motion(&mut self, motion: [f32; 3]) {
        let [x, y, z] = motion;
        self.object.motion.do_motion(Vector::new(x, y, z));
    }

    /// `CMotionInterp::TurnTo`: turn toward compass `degrees` until reached or stopped.
    pub fn turn_to(&mut self, degrees: f32) {
        self.object.motion.turn_to(Heading::from_degrees(degrees));
    }

    /// `CMotionInterp::StopTurnTo`: the turn target becomes the current heading.
    pub fn stop_turn_to(&mut self) {
        let current = self.object.position.frame.heading();
        self.object.motion.stop_turn_to(current);
    }

    /// The normalised turn rate in `[-1, 1]` (positive counter-clockwise) passed to
    /// `CMotionInterp::SetOmegaVelocity` every quantum until changed; 0 stops turning.
    pub fn set_turn_rate(&mut self, normalized: f32) {
        self.turn = normalized;
    }

    /// `CPhysicsObj::set_heading`: the heading quantised to `Heading::ToUInt8` steps and set
    /// on the current frame (`Frame::set_heading`, then `set_frame`).
    pub fn set_heading(&mut self, degrees: f32) {
        let heading = Heading::from_u8(Heading::from_degrees(degrees).to_u8());
        let mut frame = self.object.position.frame;
        frame.set_heading(heading);
        self.object.position.frame = frame;
    }

    /// One physics quantum (`CPhysicsObj::UpdateMovement(PHYSICS_QUANTUM)`).
    pub fn step_quantum(&mut self, terrain: &Terrain) {
        if self.turn != 0.0 {
            self.object.motion.set_omega_velocity(self.turn);
        }
        self.object.update_movement(PHYSICS_QUANTUM, terrain);
    }

    /// Runs the quanta the clock owes at `now`; returns how many ran.
    pub fn update(&mut self, clock: &mut PhysicsClock, now: f64, terrain: &Terrain) -> u32 {
        let quanta = clock.advance(now);
        for _ in 0..quanta {
            self.step_quantum(terrain);
        }
        quanta
    }

    /// `CPhysicsObj::JumpLogic` without the jump behaviour: when `can_jump`, `Jump`. Returns
    /// whether the body jumped.
    pub fn jump(&mut self) -> bool {
        if !self.object.can_jump() {
            return false;
        }
        self.object.jump();
        true
    }

    pub fn movement_state(&self) -> MovementState {
        let o = &self.object;
        let frame = o.position.frame;
        let motion = o.motion.last_do_motion();
        MovementState {
            cell_id: o.position.cell_id,
            origin: [frame.origin.x, frame.origin.y, frame.origin.z],
            rotation: [frame.q.w, frame.q.x, frame.q.y, frame.q.z],
            velocity: [o.velocity.x, o.velocity.y, o.velocity.z],
            omega: [o.omega.x, o.omega.y, o.omega.z],
            do_motion: [motion.x, motion.y, motion.z],
            turn: self.turn,
            element: o.element,
            mode: o.motion.mode(),
            contact: o.has_contact(),
            on_walkable: o.on_walkable(),
            heading_degrees: frame.heading().degrees(),
            times_jumped: o.times_jumped,
            turning_to: o.motion.is_turning_to(),
            turn_to_degrees: o.motion.turn_to_direction().degrees(),
        }
    }

    /// `CPhysicsObj::m_state` (`PhysicsState` bits, as the server sends them).
    pub fn set_physics_state(&mut self, state: u32) {
        self.object.state = state;
    }

    /// `CPhysicsObj::set_velocity` (a jump or a server velocity).
    pub fn set_velocity(&mut self, velocity: [f32; 3]) {
        self.object.velocity = Vector::new(velocity[0], velocity[1], velocity[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_steps_whole_quanta_and_drops_long_stalls() {
        let mut clock = PhysicsClock::new(10.0);
        assert_eq!(clock.advance(10.05), 0);
        assert_eq!(clock.advance(10.35), 3);
        assert_eq!(clock.advance(10.36), 0);
        assert_eq!(clock.advance(13.0), 0);
        assert_eq!(clock.advance(13.1), 1);
        assert_eq!(clock.advance(12.0), 0);
        assert_eq!(clock.advance(12.25), 2);
    }
}
