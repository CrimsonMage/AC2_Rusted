//! `CPhysicsObj`'s side of `CMissileManager`: the body is the manager's `ICPhysicsObj`. The
//! per-quantum `UpdateMovementInternal` runs `CMissileManager::Update` first and
//! `CMissileManager::UpdateMissilePhysics` in place of the walking integration; the `RecvEvt_*`
//! and `Missile*` entry points make the manager on first use. What the retail body answered the
//! manager through `ICPhysicsObj` vtable slots (`GetObjectA`, `0x4a0` `GetObjectOffsetPos`,
//! `0x45c` `GetHeightAboveWalkable`, `0x3d4`/`0x434`, `0x52c` `MissileInitEnd`, `0x1c4`
//! `align_to_path`) is [`Ctx`], built for the length of one call. What retail told the weenie
//! (`0x80`, `0x84`, `0x8c`) and the streak renderer comes out as [`MissileEvent`]s.

use ac2_anim::Ac2AnimEvent;

use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::is_outside;
use crate::los::{LosParameters, Ray, RayIntersection};
use crate::missile::{
    targeting_bits, MissileHost, MissileManager, MissileParameters, MissileTargetingParameters,
    TargetPosition,
};
use crate::missile_detonation::{ImpactLookup, MissileCollisionHost};
use crate::missile_flight::{MissileFlightHost, ObjectTarget};
use crate::physics_obj::PhysicsObj;
use crate::physics_obj_place::SetPositionError;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::Vector;

/// What the missile code reads of another object (`CPhysics::GetObjectA(id)` then
/// `GetTopLevelObject`, and the object's own velocity): the top-level object's position, total
/// scale and height, and the object's velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectView {
    /// `GetPosition` of the top-level object.
    pub position: Position,
    /// `GetTotalScale` of the top-level object.
    pub scale: Vector,
    /// `GetHeight` of the top-level object.
    pub height: f32,
    /// The object's velocity (the second base's slot `0xc`).
    pub velocity: Vector,
}

/// The object table as missiles read it (`ICPhysicsObj::GetObjectA`).
pub trait ObjectLookup {
    /// The object `id`, or `None` when it is not in the world.
    fn object(&self, id: u64) -> Option<ObjectView>;
}

/// The world a stepping body sees: no objects.
pub(crate) struct NoObjects;

impl ObjectLookup for NoObjects {
    fn object(&self, _id: u64) -> Option<ObjectView> {
        None
    }
}

/// What a quantum's missile code needs from outside the body: the object table and the random
/// roll (`Random::Check`) the intercept uses.
pub struct MissileEnv<'a> {
    pub objects: &'a dyn ObjectLookup,
    pub coin: &'a mut dyn FnMut(f32) -> bool,
}

/// What a missile asked of the world beyond the body: the weenie's callbacks and the streak
/// renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MissileEvent {
    /// `CPhysicsObj::StartStreak(0)` (`on`) or `StopStreak`: the object's streaks.
    Streak { on: bool },
    /// `CPhysicsObj::MissileReportOutOfFuel` (weenie slot `0x80`).
    OutOfFuel,
    /// `CPhysicsObj::MissileReportExpired` (weenie slot `0x84`).
    Expired,
    /// An error code for the weenie (slot `0x8c`): 1 target gone, 2 bad position or placement
    /// refused, 3 cannot fire, 4 parameters refused, 5 no missile manager.
    Error(u32),
    /// `CPhysicsObj::DeleteSelf(fade, delay)`: the detonated missile fades and goes.
    DeleteSelf { fade: f64, delay: f64 },
    /// `CPhysicsObj::StickySetParent(object)`: the detonated missile sticks to what it hit.
    StickTo(u64),
}

/// `CPhysicsObj::GetObjectOffsetPos`: the point `height` (a fraction of the object's height)
/// plus `offset` (scaled by the object's scale when `use_scale`, turned by its frame) from the
/// object. A line of sight from the object's position to that point (against the environment
/// only) ends where it is blocked. The result is the point and whether the trace succeeded.
pub(crate) fn object_offset_pos(
    view: &ObjectView,
    terrain: &Terrain,
    height: f32,
    use_scale: bool,
    offset: Vector,
) -> (bool, Position) {
    let mut pos = view.position;
    let mut scaled = offset;
    if use_scale {
        scaled.x = st(ld(scaled.x) * ld(view.scale.x));
        scaled.y = st(ld(scaled.y) * ld(view.scale.y));
        scaled.z = st(ld(scaled.z) * ld(view.scale.z));
    }
    let lift = ld(view.height) * ld(height) + ld(scaled.z);
    let m = pos.frame.m.map(ld);
    let (sx, sy) = (ld(scaled.x), ld(scaled.y));
    let out = Vector::new(
        st((lift * m[6] + sx * m[0]) + sy * m[3]),
        st((lift * m[7] + sx * m[1]) + sy * m[4]),
        st((lift * m[8] + sx * m[2]) + sy * m[5]),
    );
    let small = |v: f32| ld(v).abs() < ld(F_EPSILON);
    if small(out.x) && small(out.y) && small(out.z) {
        return (true, pos);
    }
    if small(offset.x)
        && small(offset.y)
        && small(offset.z)
        && (0.0..1.0).contains(&height)
        && small(out.x)
        && small(out.y)
        && is_outside(pos.cell_id)
    {
        pos.frame.origin.z = st(ld(out.z) + ld(pos.frame.origin.z));
        return (true, pos);
    }
    let mut los = LosParameters::new();
    los.set_ray(Ray::from_offset(pos.frame.origin, out), pos.cell_id);
    los.intersect = RayIntersection::EnvironmentOnly;
    let ok = terrain.los_check(&mut los);
    pos.frame.origin = los.end_point();
    pos.cell_id = los.final_cell_id();
    (ok, pos)
}

/// The body as the manager's `ICPhysicsObj` for one call.
struct Ctx<'a> {
    obj: &'a mut PhysicsObj,
    terrain: &'a Terrain,
    objects: &'a dyn ObjectLookup,
}

impl MissileHost for Ctx<'_> {
    fn current_gravity(&self) -> Vector {
        self.obj.local_gravity
    }
    fn current_wind(&self) -> Vector {
        self.obj.wind
    }
    fn is_in_world(&self) -> bool {
        self.obj.position.cell_id != 0
    }
    fn set_missile_velocity(&mut self, velocity: Vector) {
        self.obj.set_velocity_culled(velocity);
    }
    fn report_out_of_fuel(&mut self) {
        self.obj.missile_events.push(MissileEvent::OutOfFuel);
    }
    fn report_expired(&mut self) {
        self.obj.missile_events.push(MissileEvent::Expired);
    }
    fn stop_fx(&mut self, fx: u32) {
        self.obj.anim.push_events([Ac2AnimEvent::StopFx { fx }]);
    }
    fn do_fx(&mut self, fx: u32, scalar: f32) {
        self.obj.anim.push_events([Ac2AnimEvent::StartFx {
            fx,
            intensity: scalar,
            target: 0,
        }]);
    }
    fn start_streak(&mut self) {
        self.obj
            .missile_events
            .push(MissileEvent::Streak { on: true });
    }
    fn stop_streak(&mut self) {
        self.obj
            .missile_events
            .push(MissileEvent::Streak { on: false });
    }
}

impl MissileCollisionHost for Ctx<'_> {
    fn delete_self(&mut self, fade: f64, delay: f64) {
        self.obj
            .missile_events
            .push(MissileEvent::DeleteSelf { fade, delay });
    }
    fn sticky_set_parent(&mut self, object: u64) {
        self.obj.missile_events.push(MissileEvent::StickTo(object));
    }
}

impl MissileFlightHost for Ctx<'_> {
    fn position(&self) -> Position {
        self.obj.position
    }
    fn height_above_ground(&self) -> Option<f32> {
        Some(self.obj.height_above_walkable(self.terrain))
    }
    fn object_target(
        &mut self,
        id: u64,
        height_adjustment: f32,
        use_scale: bool,
        offset: Vector,
    ) -> Option<ObjectTarget> {
        let view = self.objects.object(id)?;
        let (ok, position) =
            object_offset_pos(&view, self.terrain, height_adjustment, use_scale, offset);
        Some(ObjectTarget {
            ok,
            position,
            velocity: view.velocity,
        })
    }
    fn object_position(&self, id: u64) -> Option<Position> {
        self.objects.object(id).map(|view| view.position)
    }
    fn object_heading(&self, id: u64) -> Option<crate::heading::Heading> {
        self.objects
            .object(id)
            .map(|view| view.position.frame.heading())
    }
    fn target_id(&self) -> u64 {
        self.obj.target_id
    }
    fn set_target(&mut self, id: u64) {
        self.obj.target_id = id;
        // `CMissileManager`'s call: `SetTarget(0, id, 1.0, 0.0)` (radius one metre, no quantum).
        self.obj.target_requests.push(crate::TargetRequest {
            object_id: id,
            radius: 1.0,
            quantum: 0.0,
        });
    }
    fn parent_start_position(&self) -> Option<Position> {
        self.obj.parent_drawing_position
    }
    fn place_missile(&mut self, pos: &Position) -> bool {
        // `CPhysicsObj::MissileInitEnd`: leave the parent, then `EnterWorldSimple(pos, 0)`
        // (`SetPosition` with the placement flag only).
        self.obj.parent_drawing_position = None;
        self.obj.set_position_checked(*pos, 1, self.terrain) == SetPositionError::Ok
    }
    fn slot_1c4(&mut self, on: bool) {
        // `CPhysicsObj::align_to_path`.
        if on {
            self.obj.state |= crate::physics_obj::PATH_ALIGNED_PS;
        } else {
            self.obj.state &= !crate::physics_obj::PATH_ALIGNED_PS;
        }
    }
}

impl PhysicsObj {
    /// `CPhysicsObj::HasMissileManager`.
    pub(crate) fn has_missile_manager(&self) -> bool {
        self.missile.is_some()
    }

    /// The target and proximity a mover's transition tests obstacles against
    /// (`CMissileManager::DetonatesOnProximityTo` in `CPhysicsObj::FindObjCollisions`).
    pub(crate) fn missile_proximity(&self) -> Option<(u64, f32)> {
        self.missile.as_ref()?.proximity_target()
    }

    /// The missile branch of `CPhysicsObj::set_description`: the manager made when the body has
    /// none, then `SetTargetPos`, `SetActivated`, `SetMoving`, `SetTargetID` when the description
    /// names a target, and the described velocity as a missile's.
    pub(crate) fn describe_missile(
        &mut self,
        missile: &crate::body::DescMissile,
        velocity: [f32; 3],
    ) {
        self.ensure_missile_manager();
        let empty = Terrain::new();
        self.with_missile(&empty, &NoObjects, |manager, ctx, _| {
            let info = &mut manager.target_info;
            info.target_pos = missile.target_pos;
            info.bitfield |= 2;
            manager.set_activated(missile.activated);
            manager.set_moving(ctx, missile.moving);
            if let Some((id, offset, use_scale)) = missile.target {
                let info = &mut manager.target_info;
                info.target_id = id;
                info.target_height_adjustment = 0.0;
                info.use_target_scale = use_scale;
                info.target_offset = offset;
                info.bitfield |= targeting_bits::TARGET_OBJECT;
            }
        });
        let [x, y, z] = velocity;
        self.set_velocity_culled(Vector::new(x, y, z));
    }

    /// The manager made on first use: `IntrusiveSmartPointer::create`, the manager bound to the
    /// body, and `set_movement_active(1)`.
    fn ensure_missile_manager(&mut self) {
        if self.missile.is_none() {
            self.missile = Some(Box::default());
            self.set_movement_active(true);
        }
    }

    /// Runs `f` on the manager with the body as its host (the manager is out of the body while
    /// it runs).
    fn with_missile<R>(
        &mut self,
        terrain: &Terrain,
        objects: &dyn ObjectLookup,
        f: impl FnOnce(&mut MissileManager, &mut Ctx<'_>, f64) -> R,
    ) -> Option<R> {
        let mut manager = self.missile.take()?;
        let now = self.cur_time;
        let result = {
            let mut ctx = Ctx {
                obj: self,
                terrain,
                objects,
            };
            f(&mut manager, &mut ctx, now)
        };
        self.missile = Some(manager);
        Some(result)
    }

    /// The error code for the weenie (`m_pWeenieObj` slot `0x8c`) unless it is 0 or, for the
    /// manager's own results, 6 (the target is not set up yet).
    fn report_missile_error(&mut self, error: u32, ignore_not_ready: bool) {
        if error != 0 && !(ignore_not_ready && error == 6) {
            self.missile_events.push(MissileEvent::Error(error));
        }
    }

    /// The head of `CPhysicsObj::UpdateMovementInternal(float)`: `CMissileManager::Update`, its
    /// error (other than 6) reported to the weenie.
    pub(crate) fn missile_update(&mut self, terrain: &Terrain, env: &mut MissileEnv<'_>) {
        let objects = env.objects;
        let coin = &mut *env.coin;
        let error = self.with_missile(terrain, objects, |manager, ctx, now| {
            manager.update(ctx, now, coin)
        });
        if let Some(error) = error {
            self.report_missile_error(error, true);
        }
    }

    /// `CPhysicsObj::RecvEvt_InitializeMissile`: the manager takes the server's velocity and
    /// target, and the missile is moving at once.
    pub(crate) fn recv_initialize_missile(
        &mut self,
        terrain: &Terrain,
        velocity: Vector,
        target: &TargetPosition,
    ) {
        self.ensure_missile_manager();
        self.with_missile(terrain, &NoObjects, |manager, ctx, now| {
            manager.client_activate(ctx, velocity, target, now);
        });
    }

    /// `CPhysicsObj::RecvEvt_RetargetMissile`: the server's parameters and target, then an
    /// update. Returns `CopyInto`'s result and the update's error.
    pub(crate) fn recv_retarget_missile(
        &mut self,
        terrain: &Terrain,
        objects: &dyn ObjectLookup,
        params: &MissileParameters,
        target: &TargetPosition,
        coin: &mut dyn FnMut(f32) -> bool,
    ) -> (bool, u32) {
        self.ensure_missile_manager();
        self.with_missile(terrain, objects, |manager, ctx, now| {
            manager.client_retarget(ctx, params, target, now, coin)
        })
        .unwrap_or((false, 0))
    }

    /// `CPhysicsObj::MissileInitialize`: both parameter sets in; an error other than 6 goes to
    /// the weenie.
    pub(crate) fn missile_initialize(
        &mut self,
        terrain: &Terrain,
        objects: &dyn ObjectLookup,
        info: &MissileParameters,
        targeting: &MissileTargetingParameters,
    ) {
        self.ensure_missile_manager();
        let error = self.with_missile(terrain, objects, |manager, ctx, _| {
            manager.initialize(ctx, info, targeting)
        });
        if let Some(error) = error {
            self.report_missile_error(error, true);
        }
    }

    /// `CPhysicsObj::MissileInitMissileParameters`: the parameters in (the error is dropped).
    pub(crate) fn missile_init_parameters(&mut self, info: &MissileParameters) {
        self.ensure_missile_manager();
        let now = self.cur_time;
        if let Some(manager) = self.missile.as_mut() {
            manager.client_init_missile_parameters(info, now);
        }
    }

    /// `CPhysicsObj::MissileInitTargetingParameters`: the targeting parameters in, the target set
    /// up (the error is dropped).
    pub(crate) fn missile_init_targeting_parameters(
        &mut self,
        terrain: &Terrain,
        objects: &dyn ObjectLookup,
        targeting: &MissileTargetingParameters,
    ) {
        self.ensure_missile_manager();
        self.with_missile(terrain, objects, |manager, ctx, _| {
            manager.client_init_targeting_parameters(ctx, targeting)
        });
    }

    /// `CPhysicsObj::MissilePlaceAndFire`, `MissilePlace` and `MissileFire`: ask the manager
    /// (`place`, `fire`); without one the weenie hears error 5.
    pub(crate) fn missile_request(&mut self, place: bool, fire: bool) {
        match self.missile.as_mut() {
            Some(manager) => {
                manager.ready_to_place |= place;
                manager.ready_to_fire |= fire;
            }
            None => self.missile_events.push(MissileEvent::Error(5)),
        }
    }

    /// `CPhysicsObj::MissileRetarget`: new parameters and targeting; without a manager the
    /// weenie hears error 5, else the manager's error.
    pub(crate) fn missile_retarget(
        &mut self,
        terrain: &Terrain,
        objects: &dyn ObjectLookup,
        info: &MissileParameters,
        targeting: &MissileTargetingParameters,
    ) {
        let error = self.with_missile(terrain, objects, |manager, ctx, now| {
            manager.retarget(ctx, info, targeting, now)
        });
        match error {
            Some(error) => self.report_missile_error(error, false),
            None => self.missile_events.push(MissileEvent::Error(5)),
        }
    }

    /// `CPhysicsObj::HandleCollision` for the missile: the report to `CMissileManager::HandleCollision`.
    pub(crate) fn missile_handle_collision(
        &mut self,
        terrain: &Terrain,
        report: &crate::collision_report::CollisionReport,
        impacts: &dyn ImpactLookup,
    ) {
        self.with_missile(terrain, &NoObjects, |manager, ctx, _| {
            manager.handle_collision(ctx, report, impacts);
        });
    }

    /// `CPhysicsObj::MissileInquireDirection`: the manager's heading and pitch, or `None` with the
    /// weenie told (error 5 without a manager, else the manager's error).
    pub(crate) fn missile_inquire_direction(
        &mut self,
        terrain: &Terrain,
        env: &mut MissileEnv<'_>,
    ) -> Option<(crate::heading::Heading, f32)> {
        let coin = &mut *env.coin;
        let result = self.with_missile(terrain, env.objects, |manager, ctx, now| {
            manager.inquire_direction(ctx, now, coin)
        });
        match result {
            None => {
                self.missile_events.push(MissileEvent::Error(5));
                None
            }
            Some((heading, pitch, error)) => {
                self.report_missile_error(error, false);
                Some((heading, pitch))
            }
        }
    }

    /// `CPhysicsObj::MissileInquireCurrentVelocity`: the manager's velocity, or `None` without
    /// one.
    pub(crate) fn missile_current_velocity(&self) -> Option<Vector> {
        self.missile
            .as_ref()
            .map(|manager| manager.current_velocity)
    }

    /// `CPhysicsObj::set_velocity(v, 1)` as `set_missile_velocity` reaches it: a velocity that
    /// differs from the body's by `F_EPSILON` on any axis is culled to the terminal velocity
    /// (`CullVelocity`) and taken, and the body's movement becomes active.
    pub(crate) fn set_velocity_culled(&mut self, velocity: Vector) {
        let far = |a: f32, b: f32| ld(F_EPSILON) <= (ld(a) - ld(b)).abs();
        let v = self.velocity;
        if far(velocity.x, v.x) || far(velocity.y, v.y) || far(velocity.z, v.z) {
            self.velocity = self.culled_velocity(velocity);
        }
        self.set_movement_active(true);
    }

    /// `CPhysicsObj::UpdateMovementInternal(float, Frame&)`'s missile branch: the body's velocity,
    /// quantum and offset go to `CMissileManager::UpdateMissilePhysics` and nothing else moves
    /// the body. False when the body has no manager.
    pub(crate) fn missile_physics(
        &mut self,
        terrain: &Terrain,
        quantum: f32,
        offset: &mut crate::frame::Frame,
    ) -> bool {
        let velocity = self.velocity;
        self.with_missile(terrain, &NoObjects, |manager, ctx, now| {
            manager.update_missile_physics(ctx, velocity, quantum, offset, now);
        })
        .is_some()
    }
}

#[cfg(test)]
#[path = "physics_obj_missile_tests.rs"]
mod tests;
