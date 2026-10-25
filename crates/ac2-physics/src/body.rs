//! The client-facing body: one `CPhysicsObj` with its `CMotionInterp`, stepped at the retail
//! physics quantum by a `PhysicsModule` clock, and its movement state as plain data.

use std::sync::Arc;

use ac2_anim::{Ac2AnimEvent, Ac2BehaviorParams};
use ac2_world::Ac2MotionInterpDesc;

use crate::anim_rep::BodyAnimation;

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
    /// `m_tPhysicsUpdateTimePrevious`: the update time before the last quantum.
    previous_update_time: f64,
}

impl PhysicsClock {
    pub fn new(now: f64) -> Self {
        Self {
            update_time: now,
            previous_update_time: now,
        }
    }

    /// `CPhysics::UpdateRenderInterpolation`: how far `now` is past the last quantum, as a
    /// fraction of that quantum, clamped to `[0, 1]` (`CPhysics::sRenderInterpolation`).
    pub fn render_interpolation(&self, now: f64) -> f32 {
        let span = self.update_time - self.previous_update_time;
        let t = if span.abs() <= 0.0 {
            0.0
        } else {
            ((now - self.update_time) / span) as f32
        };
        if t > 0.0 {
            t.min(1.0)
        } else {
            0.0
        }
    }

    /// `PhysicsModule::UpdatePhysics`: the number of whole quanta to run at `now`. A clock
    /// running backwards, or more than a second behind, resynchronises without stepping.
    pub fn advance(&mut self, now: f64) -> u32 {
        let mut elapsed = st(now - self.update_time);
        if ld(elapsed) < 0.0 {
            self.update_time = now;
            self.previous_update_time = now;
            return 0;
        }
        if ld(elapsed) < ld(PHYSICS_QUANTUM) {
            return 0;
        }
        if ld(elapsed) > ld(MAX_PHYSICS_LAG) {
            self.update_time = now;
            self.previous_update_time = now;
            return 0;
        }
        let mut quanta = 0;
        while ld(elapsed) >= ld(PHYSICS_QUANTUM) {
            self.previous_update_time = self.update_time;
            self.update_time = ld(PHYSICS_QUANTUM) + self.update_time;
            elapsed = st(ld(elapsed) - ld(PHYSICS_QUANTUM));
            quanta += 1;
        }
        quanta
    }
}

/// `ANIM_QUANTUM`: `1.0f / 30.0f` (static initialiser).
pub const ANIM_QUANTUM: f32 = 0.033_333_335;
/// `PhysicsModule::UpdateAnimation` caps a step at this, then steps 0.05 s.
const MAX_ANIM_STEP: f32 = 0.2;
const LONG_ANIM_STEP: f32 = 0.05;

/// `PhysicsModule`'s animation clock (`m_tAnimationUpdateTime`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationClock {
    update_time: f64,
}

impl AnimationClock {
    pub fn new(now: f64) -> Self {
        Self { update_time: now }
    }

    /// `PhysicsModule::UpdateAnimation`: the animation step to run at `now`, once at least
    /// `ANIM_QUANTUM` has passed (0.05 s after a stall over 0.2 s). A clock running backwards
    /// resynchronises.
    pub fn advance(&mut self, now: f64) -> Option<f32> {
        let elapsed = now - self.update_time;
        let step = st(elapsed);
        if elapsed < 0.0 {
            self.update_time = now;
            return None;
        }
        if !(ld(ANIM_QUANTUM) <= ld(step)) {
            return None;
        }
        self.update_time = now;
        Some(if ld(step) > ld(MAX_ANIM_STEP) {
            LONG_ANIM_STEP
        } else {
            step
        })
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
    /// The desired element and mode (`ModeTracker::GetDesiredModeState`).
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

    /// `CPhysicsObj::m_posPrevious`: where the body was before its last physics quantum
    /// (`UpdateMovement` saves it; a placement sets it to the placed position).
    pub fn previous_position(&self) -> Position {
        self.object.prev_position
    }

    /// The drawn position (`CPhysicsObj::PrepareObjectForDrawing`, and
    /// `TargetingModule::GetPosition` for the camera's target): `Position::slerp` from the
    /// previous to the current position by `t`, the clock's render interpolation
    /// (`CPhysics::sRenderInterpolation`, `PhysicsClock::render_interpolation`). A static body
    /// draws where it is.
    pub fn drawing_position(&self, t: f32) -> Position {
        let o = &self.object;
        if o.state & crate::objects::STATIC_PS != 0 {
            return o.position;
        }
        let mut drawn = o.position;
        drawn.slerp(&o.prev_position, &o.position, t);
        drawn
    }

    /// `CPhysicsObj::set_mobile` (`ICPhysicsObj::makeObject`'s `mobile`, from
    /// `CWeenieObject::IsMobile`): an immobile body (an item, a door) does not move under
    /// physics and its placements go straight into the cell (`ForceIntoCell`). Bodies start
    /// mobile.
    pub fn set_mobile(&mut self, mobile: bool) {
        let o = &mut self.object;
        if mobile {
            o.state |= crate::physics_obj::MOBILE_PS;
        } else {
            o.state &= !crate::physics_obj::MOBILE_PS;
        }
    }

    /// `CPhysicsObj::is_mobile`.
    pub fn is_mobile(&self) -> bool {
        self.object.is_mobile()
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
        self.set_time(now);
        let quanta = clock.advance(now);
        for _ in 0..quanta {
            self.step_quantum(terrain);
        }
        quanta
    }

    /// `Timer::cur_time` for interpolation: the clock server positions are stamped on
    /// (`update` sets it to its `now`).
    pub fn set_time(&mut self, now: f64) {
        self.object.cur_time = now;
    }

    /// `CPhysicsObj::InterpolateTo`, for a server-driven body: steer toward `position` (its
    /// heading, the sender's do-motion and contact flag) along the queue of positions, or jump
    /// there when it is more than 15 m away. `time_sent` orders the positions.
    pub fn interpolate_to(
        &mut self,
        time_sent: f64,
        position: Position,
        heading_degrees: f32,
        motion: [f32; 3],
        has_contact: bool,
        terrain: &Terrain,
    ) {
        let [x, y, z] = motion;
        self.object.interpolate_to(
            time_sent,
            &position,
            Vector::new(x, y, z),
            Heading::from_degrees(heading_degrees),
            has_contact,
            terrain,
        );
    }

    /// `CPhysicsObj::InterpolateAndImpulse`: a queued position where the sender jumped (or was
    /// pushed) with `velocity`; the body takes the impulse on reaching it.
    pub fn interpolate_and_impulse(
        &mut self,
        time_sent: f64,
        position: Position,
        heading_degrees: f32,
        motion: [f32; 3],
        velocity: [f32; 3],
        jump: bool,
    ) {
        let [x, y, z] = motion;
        let [vx, vy, vz] = velocity;
        self.object.interpolate_and_impulse(
            time_sent,
            &position,
            Vector::new(x, y, z),
            Heading::from_degrees(heading_degrees),
            Vector::new(vx, vy, vz),
            jump,
        );
    }

    /// `CSetup::radius` of the body's Setup (`Ac2SetupPhysics::radius`), unscaled: the size
    /// move-to distances measure from.
    pub fn set_radius(&mut self, radius: f32) {
        self.object.setup_radius = radius;
    }

    /// `CMotionInterp::MoveToEvent`: walk to the parameters' target (or waypoints) and/or turn
    /// to its heading. Returns whether it started.
    pub fn move_to(&mut self, params: crate::moveto::MovementParameters) -> bool {
        self.object.move_to_event(params)
    }

    /// `CMotionInterp::UpdateTargetInfo`: where an object target is now, and its size.
    pub fn update_move_to_target(&mut self, position: Position, radius: f32, height: f32) {
        if let Some(manager) = self.object.move_to.as_mut() {
            manager.update_target(position, radius, height);
        }
    }

    /// The object target can no longer be tracked: the move-to ends with 0x37.
    pub fn lose_move_to_target(&mut self) {
        if let Some(manager) = self.object.move_to.as_mut() {
            manager.lose_target();
        }
    }

    /// `CMotionInterp::InterruptMoveTo`.
    pub fn interrupt_move_to(&mut self) {
        self.object.interrupt_move_to();
    }

    /// `CMotionInterp::IsMovingTo`.
    pub fn is_moving_to(&self) -> bool {
        self.object
            .move_to
            .as_ref()
            .is_some_and(|manager| manager.moving_to)
    }

    /// The move-tos that ended since the last call (`HandleMoveToCallback`).
    pub fn take_move_to_results(&mut self) -> Vec<crate::moveto::MoveToResult> {
        self.object
            .move_to
            .as_mut()
            .map(|manager| manager.take_results())
            .unwrap_or_default()
    }

    /// `CPhysicsObj::StopInterpolating`.
    pub fn stop_interpolating(&mut self) {
        if let Some(interp) = self.object.interp.as_mut() {
            interp.stop();
        }
    }

    /// `CPhysicsObj::IsInterpolating`.
    pub fn is_interpolating(&self) -> bool {
        self.object
            .interp
            .as_ref()
            .is_some_and(|interp| interp.active)
    }

    /// `CPhysicsObj::JumpLogic`: when `can_jump`, `Jump` (or, standing with a behaviour table,
    /// the jump behaviour). Returns whether the jump was taken.
    pub fn jump(&mut self) -> bool {
        self.object.jump_logic()
    }

    /// Animate the body with its behaviour table (`PHYSICSOBJREP::SetupModeTracker`): element
    /// and mode changes then wait for the animation's mode transitions, and `update_movement`
    /// starts and stops the movement behaviours.
    pub fn set_animation(&mut self, animation: BodyAnimation) {
        self.object.set_animation(animation, true);
    }

    /// One animation step (`CPhysicsObj::UpdateAnimation`), at the steps an
    /// [`AnimationClock`] gives.
    pub fn update_animation(&mut self, quantum: f32) {
        self.object.update_animation(quantum);
    }

    /// `CPhysicsObj::DoBehavior` (a weenie or server behaviour).
    pub fn do_behavior(&mut self, params: &Ac2BehaviorParams) -> bool {
        self.object
            .anim
            .with_modes(|modes, source| modes.do_behavior(params, source))
    }

    /// `CPhysicsObj::StopBehavior`.
    pub fn stop_behavior(&mut self, behavior: u32) {
        self.object
            .anim
            .with_modes(|modes, source| modes.stop_behavior(behavior, source));
    }

    /// `PHYSICSOBJREP::GetElement`: the element the animation has reached.
    pub fn current_element(&self) -> u32 {
        self.object.current_element()
    }

    /// `PHYSICSOBJREP::IsDoingBehavior`.
    pub fn is_doing_behavior(&self, behavior: u32) -> bool {
        self.object.anim.modes.is_doing_behavior(behavior)
    }

    /// The body's bones posed by the last animation step, in model space (relative to the
    /// body's frame, AC axes); the Setup rest pose before the first step, empty without a
    /// Setup. Part `i` of the Setup is drawn with bone `i`'s transform.
    pub fn posed_parts(&self) -> Vec<ac2_anim::Ac2Transform> {
        self.object.posed_parts()
    }

    /// The body's bones relative to their parents (`CPartArray::m_boneTransforms`), what the
    /// renderer builds the bone and deform matrices from (`CPartArray::UpdateBones`); empty
    /// without a Setup.
    pub fn bone_transforms(&self) -> Vec<ac2_anim::Ac2Transform> {
        self.object.bone_transforms()
    }

    /// The behaviour callbacks and FX the animation raised since the last call, in order.
    pub fn take_animation_events(&mut self) -> Vec<Ac2AnimEvent> {
        self.object.take_animation_events()
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
            element: o.desired_element(),
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
