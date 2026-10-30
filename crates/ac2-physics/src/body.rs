//! The client-facing body: one `CPhysicsObj` with its `CMotionInterp`, stepped at the retail
//! physics quantum by a `PhysicsModule` clock, and its movement state as plain data.

use std::sync::Arc;

use ac2_anim::Ac2AnimEvent;
use ac2_world::Ac2MotionInterpDesc;

use crate::collision_report::{Collided, CollisionReport, ContactMaterials};
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
            self.update_time += ld(PHYSICS_QUANTUM);
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

/// One moving body (the player or another object): position, shape, motion interpreter,
/// animation and physics state.
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
        let mut object = PhysicsObj::new(position, shape, MotionInterp::new(motion_desc));
        object.init_object_end();
        Self { object, turn: 0.0 }
    }

    /// `CPhysicsObj::GetPosition` (`m_position`); `CPhysicsObj::GetFrame` is its `frame`. The
    /// body has no id of its own (`CPhysicsObj::GetID`): its owner, the object table or the
    /// remotes, keys it by the instance id.
    pub fn position(&self) -> Position {
        self.object.position
    }

    /// `CPhysicsObj::m_posPrevious`: where the body was before its last physics quantum
    /// (`UpdateMovement` saves it; a placement sets it to the placed position).
    pub fn previous_position(&self) -> Position {
        self.object.prev_position
    }

    /// The body's id in the world's object lists (`Terrain::insert_object`), or `None` while it
    /// is not listed. The body's own transitions skip that entry.
    pub fn set_world_id(&mut self, id: Option<u32>) {
        self.object.world_id = id;
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

    /// `CPhysicsObj::is_static` (`STATIC_PS`).
    pub fn is_static(&self) -> bool {
        self.object.state & crate::objects::STATIC_PS != 0
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

    /// [`Self::step_quantum`] in a world: a missile body reads the object table and rolls the
    /// dice from `env` (`CMissileManager::Update`).
    pub fn step_quantum_in(&mut self, terrain: &Terrain, env: &mut crate::MissileEnv<'_>) {
        if self.turn != 0.0 {
            self.object.motion.set_omega_velocity(self.turn);
        }
        self.object
            .update_movement_in(PHYSICS_QUANTUM, terrain, env);
    }

    /// Test hook: the body's object.
    #[cfg(test)]
    pub(crate) fn with_object(&mut self, f: impl FnOnce(&mut PhysicsObj)) {
        f(&mut self.object);
    }

    /// Test hook: advance the body's clock.
    #[cfg(test)]
    pub(crate) fn cur_time_add(&mut self, dt: f64) {
        self.object.cur_time += dt;
    }

    /// `CPhysicsObj::HasMissileManager`.
    pub fn has_missile_manager(&self) -> bool {
        self.object.has_missile_manager()
    }

    /// `CPhysicsObj::RecvEvt_InitializeMissile`: the server's launch velocity and target.
    pub fn recv_initialize_missile(
        &mut self,
        terrain: &Terrain,
        velocity: [f32; 3],
        target: &crate::TargetPosition,
    ) {
        let [x, y, z] = velocity;
        self.object
            .recv_initialize_missile(terrain, Vector::new(x, y, z), target);
    }

    /// `CPhysicsObj::RecvEvt_RetargetMissile`: the server's parameters and target; returns
    /// `CopyInto`'s result and the update's error.
    pub fn recv_retarget_missile(
        &mut self,
        terrain: &Terrain,
        env: &mut crate::MissileEnv<'_>,
        params: &crate::MissileParameters,
        target: &crate::TargetPosition,
    ) -> (bool, u32) {
        self.object
            .recv_retarget_missile(terrain, env.objects, params, target, &mut *env.coin)
    }

    /// `CPhysicsObj::HandleCollision` for a missile: `CMissileManager::HandleCollision` on a report
    /// of this body. A detonation queues [`crate::MissileEvent`]s.
    pub fn handle_missile_collision(
        &mut self,
        terrain: &Terrain,
        report: &CollisionReport,
        impacts: &dyn crate::ImpactLookup,
    ) {
        self.object
            .missile_handle_collision(terrain, report, impacts);
    }

    /// `CPhysicsObj::MissileInitialize`.
    pub fn missile_initialize(
        &mut self,
        terrain: &Terrain,
        objects: &dyn crate::ObjectLookup,
        info: &crate::MissileParameters,
        targeting: &crate::MissileTargetingParameters,
    ) {
        self.object
            .missile_initialize(terrain, objects, info, targeting);
    }

    /// `CPhysicsObj::MissileInitMissileParameters`.
    pub fn missile_init_parameters(&mut self, info: &crate::MissileParameters) {
        self.object.missile_init_parameters(info);
    }

    /// `CPhysicsObj::MissileInitTargetingParameters`.
    pub fn missile_init_targeting_parameters(
        &mut self,
        terrain: &Terrain,
        objects: &dyn crate::ObjectLookup,
        targeting: &crate::MissileTargetingParameters,
    ) {
        self.object
            .missile_init_targeting_parameters(terrain, objects, targeting);
    }

    /// `CPhysicsObj::MissileRetarget`.
    pub fn missile_retarget(
        &mut self,
        terrain: &Terrain,
        objects: &dyn crate::ObjectLookup,
        info: &crate::MissileParameters,
        targeting: &crate::MissileTargetingParameters,
    ) {
        self.object
            .missile_retarget(terrain, objects, info, targeting);
    }

    /// `CPhysicsObj::MissilePlaceAndFire` (`place` and `fire`), `MissilePlace` and `MissileFire`.
    pub fn missile_request(&mut self, place: bool, fire: bool) {
        self.object.missile_request(place, fire);
    }

    /// `CPhysicsObj::MissileInquireDirection`.
    pub fn missile_inquire_direction(
        &mut self,
        terrain: &Terrain,
        env: &mut crate::MissileEnv<'_>,
    ) -> Option<(Heading, f32)> {
        self.object.missile_inquire_direction(terrain, env)
    }

    /// What `CMissileManager::ShouldDetonate` reads of this body when a missile hits it.
    pub fn impact_target(&self, id: u64) -> crate::ImpactTarget {
        crate::ImpactTarget {
            id,
            dynamic: self.object.state & crate::physics_obj::STATIC_PS == 0,
            has_missile_manager: self.object.has_missile_manager(),
            weenie: None,
        }
    }

    /// `CPhysicsObj::MissileInquireCurrentVelocity`.
    pub fn missile_current_velocity(&self) -> Option<Vector> {
        self.object.missile_current_velocity()
    }

    /// What the missile code told the weenie and the streaks since the last call.
    pub fn take_missile_events(&mut self) -> Vec<crate::MissileEvent> {
        std::mem::take(&mut self.object.missile_events)
    }

    /// `m_posDrawing` while the body has a parent (`HasParent`): where a parented missile
    /// starts. The owner of the child graph sets it and clears it when the parent is let go.
    pub fn set_parent_drawing_position(&mut self, position: Option<Position>) {
        self.object.parent_drawing_position = position;
    }

    /// `PhysicsObj` view for other missiles: position, total scale, height and velocity.
    pub fn object_view(&self) -> crate::ObjectView {
        crate::ObjectView {
            position: self.object.position,
            scale: self.object.own_total_scale(),
            height: self.object.height(),
            velocity: self.object.velocity,
        }
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

    /// `CPhysicsObj::MoveTo` -> `CMotionInterp::MoveToEvent`: walk to the parameters' target (or
    /// waypoints) and/or turn to its heading; a move that started makes the body's movement
    /// active. Returns whether it started.
    pub fn move_to(&mut self, params: crate::moveto::MovementParameters) -> bool {
        let started = self.object.move_to_event(params);
        if started {
            self.object.set_movement_active(true);
        }
        started
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

    /// `CPhysicsObj::InterruptMoveTo` -> `PHYSICSOBJREP::InterruptMoveTo`: the behaviours a
    /// move-to cancels stop, then `CMotionInterp::InterruptMoveTo`.
    pub fn interrupt_move_to(&mut self) {
        self.object.stop_move_to_cancels_behaviors();
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

    /// `CPhysicsObj::IsInterpolating` (`CMotionInterp::IsInterpolating`).
    pub fn is_interpolating(&self) -> bool {
        self.object
            .interp
            .as_ref()
            .is_some_and(|interp| interp.active)
    }

    /// `CPhysicsObj::GetRemoteDoMotionValues` (`CMotionInterp::GetRemoteDoMotionValues`): the
    /// do-motion the server last sent while the body interpolates, else zero.
    pub fn remote_do_motion_values(&self) -> Vector {
        self.object
            .interp
            .as_ref()
            .map_or(Vector::ZERO, |interp| interp.last_received_motion())
    }

    /// `CPhysicsObj::JumpLogic`: when `can_jump`, `Jump` (or, standing with a behaviour table,
    /// the jump behaviour). Returns whether the jump was taken.
    pub fn jump(&mut self) -> bool {
        self.object.jump_logic()
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

    /// `CPhysicsObj::set_velocity(v, 1)` (a jump or a server velocity): a velocity that differs
    /// from the body's by `F_EPSILON` on any axis is culled to the terminal velocity and taken,
    /// and the body's movement becomes active.
    pub fn set_velocity(&mut self, velocity: [f32; 3]) {
        self.object
            .set_velocity_culled(Vector::new(velocity[0], velocity[1], velocity[2]));
    }

    /// `ECM_Physics::RecvEvt_AddImpulse` -> `CPhysicsObj::AddImpulse`: a world-axis velocity
    /// change (knockback); the body leaves contact.
    pub fn add_impulse(&mut self, direction: [f32; 3]) {
        let [x, y, z] = direction;
        self.object.add_impulse(Vector::new(x, y, z));
    }

    /// `CPhysicsObj::AddImpulseLocal`: [`Self::add_impulse`] of a vector in the body's own axes.
    pub fn add_impulse_local(&mut self, direction: [f32; 3]) {
        let [x, y, z] = direction;
        self.object.add_impulse_local(Vector::new(x, y, z));
    }

    /// `ECM_Physics::RecvEvt_AddTargetedImpulse` -> `CPhysicsObj::AddTargetedImpulse`: pushes
    /// the body away from `source` (the target's position) with `magnitude`, falling off with
    /// distance when `falloff`, and `vertical` added upward.
    pub fn add_targeted_impulse(
        &mut self,
        source: &Position,
        magnitude: f32,
        falloff: bool,
        vertical: f32,
    ) {
        self.object
            .add_targeted_impulse(source, magnitude, falloff, vertical);
    }

    /// `CPhysicsObj::RecvEvt_SetMIDESC` -> `CPhysicsObj::SetMIDESC` -> `PHYSICSOBJREP::SetMIDESC`
    /// (`SetMotionInterpreter`, and `CompareAndModify` on a changed description) ->
    /// `CMotionInterp::SetMIDESC` with a description the client already loaded (`None` keeps
    /// the table).
    pub fn set_motion_desc(&mut self, desc: Option<Arc<Ac2MotionInterpDesc>>) -> bool {
        self.object.motion.set_desc(desc)
    }

    /// `ECM_Physics::RecvEvt_SetVelocityScale` -> `CPhysicsObj::SetVelocityScale` ->
    /// `CMotionInterp::SetVelocityScale`: scales the walking and running speeds.
    pub fn set_velocity_scale(&mut self, scale: f32) {
        self.object.motion.set_velocity_scale(scale);
        let standard = self.object.motion.standard_motion_values();
        self.object
            .impact_minimums
            .set_velocity_scale(scale, standard.max_forward_velocity);
    }

    /// The last impact speed an environment contact recorded above the collision minimum
    /// (`CCollisionManager::SetCollisionNormal`), `None` until one does.
    pub fn last_impact_speed(&self) -> Option<f32> {
        self.object.impact_speed
    }

    /// `CPhysicsObj::ReportCollisions`, which is all of `CPhysicsObj::UpdateMisc` once the
    /// detection manager (never made in 7160) is out: the collision report at the body's clock
    /// (`CCollisionManager::ReportCollision`). `force_end` ends every reported record. Returns
    /// the report only when it lists something (`CollisionProfile::NonEmpty`).
    pub fn report_collisions(
        &mut self,
        materials: &ContactMaterials,
        force_end: bool,
    ) -> Option<CollisionReport> {
        self.report(materials, force_end, false)
    }

    /// The materials of the body's current contact, read from the resident world: the landscape
    /// vertex under the body when it is on the landscape (`CPhysicsObj::SetContactInfo`), the
    /// block's water when it is in water (`CLandBlock::GetWaterPhysicsMaterial`), and the interior
    /// cell's material when it is in one (`CPhysicsObj::UpdateCellMaterial`).
    pub fn contact_materials(&self, terrain: &Terrain) -> ContactMaterials {
        let position = self.object.position;
        let state = self.object.transient_state;
        let on = |flag: u32| state & flag != 0;
        ContactMaterials {
            landscape: on(crate::physics_obj::LANDSCAPE_CONTACT_TS)
                .then(|| terrain.land_physics_material(&position))
                .flatten(),
            water: on(crate::physics_obj_contact::WATER_CONTACT_TS)
                .then(|| terrain.water_physics_material(position.cell_id))
                .flatten(),
            environment: on(crate::physics_obj_contact::ENVIRONMENT_CONTACT_TS)
                .then(|| terrain.env_physics_material(position.cell_id))
                .flatten(),
        }
    }

    /// The motion terrain the body's motion interpreter looks its values up under
    /// (`CMotionInterp::GetTerrainType`): the sentinel until a landscape or water property sets it.
    pub fn motion_terrain_type(&self) -> u32 {
        self.object.motion.terrain_type()
    }

    /// `CPhysicsObj::EndAllCollisions`: ends every record the body has reported and drops its
    /// object table (`CCollisionManager::ReportCollision` with both end flags set).
    pub fn end_all_collisions(&mut self, materials: &ContactMaterials) -> Option<CollisionReport> {
        self.report(materials, true, true)
    }

    fn report(
        &mut self,
        materials: &ContactMaterials,
        end_reporting: bool,
        end_all: bool,
    ) -> Option<CollisionReport> {
        let contact = crate::collision_report::BodyContact {
            plane: self
                .object
                .has_contact()
                .then_some(self.object.contact_plane),
            in_water: self.object.transient_state & crate::physics_obj_contact::WATER_CONTACT_TS
                != 0,
            on_landscape: self.object.transient_state & crate::physics_obj::LANDSCAPE_CONTACT_TS
                != 0,
            in_environment: self.object.transient_state
                & crate::physics_obj_contact::ENVIRONMENT_CONTACT_TS
                != 0,
        };
        let flags = crate::collision_report::ReportFlags {
            end_reporting,
            end_all,
            // `ICPhysicsObj` slot `0x248` is `CPhysicsObj::reports_collisions`: a body that does not
            // report begins nothing.
            active: self.reports_collisions(),
        };
        let now = self.object.cur_time;
        let mut report = self
            .object
            .collision_manager
            .report(now, contact, materials, flags);
        // An object erased from the table also leaves the temporary ethereal list
        // (`CollisionParameters::RemoveFromTempEtherealList`) and is for the owner to tell
        // (`take_dropped`).
        for id in std::mem::take(&mut report.dropped) {
            self.object.collision.remove_from_temp_ethereal_list(id);
            self.object.dropped.push(id);
        }
        // `CCollisionManager::SetTerrainType` hands the motion terrain to the body
        // (`CPhysicsObj::SetTerrainType`), whether or not the report lists anything.
        if let Some(terrain) = self.object.collision_manager.take_terrain() {
            self.object.motion.set_terrain_type(terrain as u32);
        }
        (!report.is_empty()).then_some(report)
    }

    /// The objects the reports erased from the touched table since the last call (expired,
    /// removed, or ended with everything). Retail tells each one's body to drop this body too
    /// (`ICPhysicsObj` slot `0x1f8`, `CPhysicsObj::CollisionRemoveObject(this id)`): the owner
    /// of the other bodies calls [`Self::collision_remove_object`] on them.
    pub fn take_dropped(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.object.dropped)
    }

    /// `CPhysicsObj::set_report_collisions` with its `ReportCollisions` calls (the bit alone is
    /// `set_report_collisions`): turning reporting off ends every record the body
    /// reported (`ReportCollisions(1)`, returned) and clears `REPORT_COLLISIONS_PS`; turning it
    /// on sets the bit and runs a plain report (`ReportCollisions(0)`). No change when the body
    /// is already in that mode.
    pub fn set_report_collisions(
        &mut self,
        on: bool,
        materials: &ContactMaterials,
    ) -> Option<CollisionReport> {
        const REPORT_COLLISIONS_PS: u32 = 0x4;
        let reporting = self.reports_collisions();
        if on && !reporting {
            self.object.state |= REPORT_COLLISIONS_PS;
            return self.report(materials, false, false);
        }
        if !on && reporting {
            let report = self.report(materials, true, false);
            self.object.state &= !REPORT_COLLISIONS_PS;
            return report;
        }
        None
    }

    /// The server objects the last move collided with, taken once, for the owner to tell their
    /// bodies ([`Self::touched_by`]); empty for a body no script gave a collision state
    /// (`handle_all_collisions` returns before it for a body without a manager).
    pub fn take_collided(&mut self) -> Vec<Collided> {
        let collided = std::mem::take(&mut self.object.collided);
        if self.object.has_collision_manager {
            collided
        } else {
            Vec::new()
        }
    }

    /// `CPhysicsObj::handle_all_collisions` on the object the mover hit: the mover (`mover`,
    /// whose `ignore_collisions` is `mover_ignores`) joins this body's touched table, and the
    /// contact normal it recorded against this body (`impact`, from [`Collided::impact`]) is
    /// checked against this body's vertical minimum. A body without a collision manager hears
    /// nothing.
    pub fn touched_by(
        &mut self,
        mover: &crate::ConflictPeer,
        mover_ignores: bool,
        impact: Option<(crate::vector::Vector, crate::vector::Vector)>,
    ) {
        self.object.touched_by(mover, mover_ignores, impact);
    }

    /// `CPhysicsObj::CollisionRemoveObject`: the object `id` left the world; this body's entry
    /// for it ends at the next report, and the properties of the object end at once: that
    /// report is returned when it lists something, for the script (`HandleCollision`).
    pub fn collision_remove_object(&mut self, id: u64) -> Option<CollisionReport> {
        self.object.collision_remove_object(id)
    }

    /// `CPhysicsObj::UpdateCollidePhysicsMask`.
    pub fn update_collide_physics_mask(&mut self, mask: crate::PhysicsTypeMask) -> bool {
        self.object.update_collide_physics_mask(mask)
    }

    /// Whether a script gave the body a collision state, which is what makes
    /// `CPhysicsObj::m_pCollisionManager` (`SetCollisionState` makes it the first time).
    pub fn has_collision_manager(&self) -> bool {
        self.object.has_collision_manager
    }

    /// `CPhysicsObj::GetCollisionStatePtr`: the manager's current collision parameters, `None`
    /// for a body without a manager (`m_current_info`, here the body's `collision`).
    pub fn collision_state_ptr(&self) -> Option<&crate::collision_types::CollisionParameters> {
        self.object
            .has_collision_manager
            .then_some(&self.object.collision)
    }

    /// `CCollisionManager::IsEmpty` (and `CPhysicsObj::misc_can_be_inactive`'s test): nothing is
    /// touched. True for a body without a manager.
    pub fn collisions_empty(&self) -> bool {
        self.object.collision_manager.is_empty()
    }

    /// `CCollisionManager::AddPusherProperties`: the touched object `key` pushes the body with
    /// the vector properties of its physics material (`properties`, the object's property list),
    /// turned by `object`'s frame when the material is local-space. Returns whether it joined.
    pub fn add_pusher_properties(
        &mut self,
        key: u64,
        object: Option<&crate::position::Position>,
        properties: &[ac2_db::Ac2PropertyValue],
    ) -> bool {
        self.object
            .collision_manager
            .pushers
            .add_properties(key, object, properties)
    }

    /// `CCollisionManager::RemovePropertiesAndErase`'s pusher part: the touched object `key` no
    /// longer pushes the body. Returns whether it did.
    pub fn remove_pusher(&mut self, key: u64) -> bool {
        self.object.collision_manager.pushers.remove(key)
    }

    /// `CCollisionManager::SetCollisionState` on a script's `CollisionParameters`: a non-zero
    /// `m_min_reported_impactXY` or `Z` replaces the body's minimum; zero keeps it.
    pub fn set_collision_minimums(&mut self, xy: f32, z: f32) {
        if xy != 0.0 {
            self.object.impact_minimums.xy = xy;
        }
        if z != 0.0 {
            self.object.impact_minimums.z = z;
        }
    }

    /// The body's collision parameters (`CPhysicsObj::SetCollisionState`), for a script or the
    /// client to set its ethereal and physics-type sets.
    pub fn collision_mut(&mut self) -> &mut crate::collision_types::CollisionParameters {
        &mut self.object.collision
    }

    /// `CPhysicsObj::m_id`, which the conflict checks name this body by.
    pub fn set_instance_id(&mut self, id: u64) {
        self.object.instance_id = id;
    }

    /// `CPhysicsObj::SetCollisionState` -> `CCollisionManager::SetCollisionState`: `info` merged
    /// into the body's sets, then each touched object is checked again for a conflict
    /// (`peer` answers `GetObjectA`). The effects are for the other bodies:
    /// [`Self::add_to_temp_ethereal_list`] on `object`.
    pub fn set_collision_state(
        &mut self,
        info: &crate::collision_types::CollisionParameters,
        peer: &dyn Fn(u64) -> Option<crate::ConflictPeer>,
    ) -> Vec<crate::ConflictEffect> {
        self.object.set_collision_state(info, peer)
    }

    /// `CPhysicsObj::AddToTempEtherealList`.
    pub fn add_to_temp_ethereal_list(&mut self, id: u64) -> bool {
        self.object.add_to_temp_ethereal_list(id)
    }

    /// What another body's conflict check reads of this one.
    pub fn conflict_peer(&self) -> crate::ConflictPeer {
        self.object.conflict_peer()
    }

    /// `CPhysicsObj::ignore_collisions`.
    pub fn ignores_collisions(&self) -> bool {
        self.object.state & crate::physics_obj_conflict::IGNORE_COLLISIONS_PS != 0
    }

    /// `CPhysicsObj::ApplyEtherealProperty`: sets the body's blocking bits (`EtherealPhysicsType::
    /// SetBit` on `m_blocking_mask`) named by `bits`; bits the body already has stay set.
    pub fn apply_blocking_bits(&mut self, bits: u64) {
        let merged = self.object.blocking.bits() | bits;
        self.object.blocking = crate::collision_types::PhysicsTypeMask::from_bits(merged);
    }

    /// `CPhysicsObj::set_movement_frozen`: sets or clears `MOVEMENT_FROZEN_PS`.
    pub fn set_movement_frozen(&mut self, frozen: bool) {
        self.object
            .set_state_bit(crate::physics_obj::MOVEMENT_FROZEN_PS, frozen);
    }

    /// `CPhysicsObj::set_animation_frozen`: sets or clears `ANIMATION_FROZEN_PS`.
    pub fn set_animation_frozen(&mut self, frozen: bool) {
        self.object
            .set_state_bit(crate::physics_obj::ANIMATION_FROZEN_PS, frozen);
    }

    /// `CPhysicsObj::set_edge_slide`: sets or clears `EDGE_SLIDE_PS`.
    pub fn set_edge_slide(&mut self, slide: bool) {
        self.object
            .set_state_bit(crate::physics_obj::EDGE_SLIDE_PS, slide);
    }

    /// `CPhysicsObj::GetCollisionState`: the state a script reads (`gCWPhysicsObject::
    /// GetCollisionState`): the blocking mask as the physics type (bit 0x10), then the
    /// collision manager's placement mask and movement mask.
    pub fn script_collision_state(&self) -> crate::collision_types::CollisionParameters {
        let collision = &self.object.collision;
        let mut state = collision.collision_state(collision.movement_physics());
        state.set_physics_type(self.object.blocking);
        state
    }

    /// The body's blocking physics-type bits (`CPhysicsObj::m_blocking_mask`).
    pub fn blocking_bits(&self) -> u64 {
        self.object.blocking.bits()
    }

    /// The collision impact minimums as the body holds them (`m_min_reported_impactXY`, `Z`).
    pub fn impact_minimums(&self) -> (f32, f32) {
        let m = self.object.impact_minimums;
        (m.xy, m.z)
    }

    /// `ECM_Physics::RecvEvt_SetAccelerationScale` -> `CMotionInterp::SetAccelerationScale`.
    pub fn set_acceleration_scale(&mut self, scale: f32) {
        self.object.motion.set_acceleration_scale(scale);
    }

    /// `ECM_Physics::RecvEvt_SetJumpScale` -> `CPhysicsObj::SetJumpScale`: a negative scale is
    /// refused (`m_fJumpScale` keeps its value).
    pub fn set_jump_scale(&mut self, scale: f32) {
        if scale >= 0.0 {
            self.object.jump_scale = scale;
            let standard = self.object.motion.standard_motion_values();
            self.object
                .impact_minimums
                .set_jump_scale(scale, standard.jump_impulse);
        }
    }

    /// `ECM_Physics::RecvEvt_SetHoverHeight` -> `CPhysicsObj::SetHoverHeight`.
    pub fn set_hover_height(&mut self, height: f32) {
        self.object.motion.set_hover_height(height);
    }

    /// `ECM_Physics::RecvEvt_SetGlobalModifier` -> `CPhysicsObj::SetGlobalModifier` ->
    /// `CMotionInterp::SetGlobalModifier`: the motion table lookups key on `id`.
    pub fn set_global_modifier(&mut self, id: u32) {
        self.object.motion.set_global_modifier(id);
    }

    /// `ECM_Physics::RecvEvt_AddTerrainModifier` -> `CPhysicsObj::AddTerrainModifier` ->
    /// `CMotionInterp::AddTerrainModifier`.
    pub fn add_terrain_modifier(&mut self, terrain: u32, modifier: u32) {
        self.object.motion.add_terrain_modifier(terrain, modifier);
    }

    /// `ECM_Physics::RecvEvt_ClearTerrainModifiers` -> `CPhysicsObj::ClearTerrainModifiers` ->
    /// `CMotionInterp::ClearTerrainModifiers`.
    pub fn clear_terrain_modifiers(&mut self) {
        self.object.motion.clear_terrain_modifiers();
    }

    /// `CPhysicsObj::ResetHoverHeight`: the motion table's hover height again.
    pub fn reset_hover_height(&mut self) {
        self.object.motion.reset_hover_height();
    }

    /// `CPhysicsObj::RecvEvt_DoFX` -> `CPhysicsObj::DoFX` -> `PHYSICSOBJREP::DoFX` ->
    /// `FXManager::DoFX`: the effect starts at `scalar`; its sounds are the client's
    /// (`StartFx` on the animation events, as an animation's own FX starts).
    pub fn do_fx(&mut self, fx: u32, scalar: f32) -> bool {
        self.object.anim.push_events([Ac2AnimEvent::StartFx {
            fx,
            intensity: scalar,
        }]);
        true
    }

    /// `CPhysicsObj::RecvEvt_StopFX` -> `PHYSICSOBJREP::StopFX` -> `FXManager::StopFX`.
    pub fn stop_fx(&mut self, fx: u32) -> bool {
        self.object.anim.push_events([Ac2AnimEvent::StopFx { fx }]);
        true
    }

    /// The server-tuned scales and hover height the body moves with.
    pub fn tuning(&self) -> Tuning {
        Tuning {
            velocity_scale: self.object.motion.velocity_scale(),
            acceleration_scale: self.object.motion.acceleration_scale(),
            jump_scale: self.object.jump_scale,
            hover_height: self.object.motion.hover_height_override(),
        }
    }
}

/// The per-body scales a server event sets (`CPhysicsObj::Get*Scale`, `GetHoverHeight` before
/// the table's default). `hover_height` is -1 while the motion table's height applies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tuning {
    pub velocity_scale: f32,
    pub acceleration_scale: f32,
    pub jump_scale: f32,
    pub hover_height: f32,
}

#[path = "body_access.rs"]
mod access;
pub use access::is_moving_velocity;

#[path = "body_anim.rs"]
mod anim;

#[path = "body_parts.rs"]
mod parts;

#[path = "body_profile.rs"]
mod profile;
pub use profile::PhysObjProfile;

#[path = "physics_obj_desc.rs"]
mod desc;
pub use desc::DescPhysics;

#[path = "body_drawn.rs"]
mod drawn;

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
