//! The camera's collision with the world: `SmartBox::UpdateCamera`'s line-of-sight pull-in and
//! the camera physics object (`SmartBox::mpCameraPhysObj`) that places an outdoor camera.
//!
//! Retail traces the camera in three steps, all `CObjCell::LOSCheck`s (not a sphere sweep;
//! `CameraManager::FakeTransition` only blends the camera between targets):
//!
//! 1. from the player's origin, lifted `2 * F_EPSILON`, by `CameraManager::GetTargetOffset` (the
//!    point the camera looks at): environment only, the water not blocking. Where it stops is
//!    the head the camera is traced from, in the cell it reached.
//! 2. from there to the desired camera position (`CameraManager::UpdateCamera`'s): the physics
//!    representation (the environment only in the map view or with a target bone,
//!    `CheckEnvOnly`), with the camera-ethereal physics set and solid weenies, so what is
//!    flagged camera-ethereal does not block. Its end point is where the camera stands.
//! 3. indoors, a short ray from that end point along the hit's normal, 0.2 m, the same blocking:
//!    the camera stands off the wall it was pulled in to (outdoors the camera object is placed
//!    instead, below).
//!
//! If every trace ran (the cells they start in are resident), `SmartBox::SetCameraPosition`
//! takes the result; otherwise the camera stays where it was.

use ac2_core::DataId;
use ac2_physics::{
    camera_ethereal_mask, landdefs, BodyShape, CollisionParameters, Frame, LosParameters,
    PhysicsBody, Position, Ray, RayIntersection, Terrain, Vector, WeenieTypes,
};

use crate::camera::{ld, st, CameraManager, CameraSubject, F_EPSILON};
use crate::scene::SceneCamera;
use crate::targeting::CameraObjects;

/// `SmartBox::Init`'s `FileArray::GetDIDFromEnum(0x4000002a, 0x40000002)`: the camera object's
/// enum value and group (a Setup with one 0.25 m sphere).
pub const CAMERA_OBJECT_ENUM: (u32, u32) = (0x4000_002A, 0x4000_0002);

/// `AFrame::operator==`: every origin and rotation component within `F_EPSILON`.
fn frame_equals(a: &Frame, b: &Frame) -> bool {
    let near = |x: f32, y: f32| (ld(x) - ld(y)).abs() < ld(F_EPSILON);
    near(a.origin.x, b.origin.x)
        && near(a.origin.y, b.origin.y)
        && near(a.origin.z, b.origin.z)
        && near(a.q.w, b.q.w)
        && near(a.q.x, b.q.x)
        && near(a.q.y, b.q.y)
        && near(a.q.z, b.q.z)
}

/// `Position::operator==` (the negation of `Position::operator!=`).
fn position_equals(a: &Position, b: &Position) -> bool {
    a.cell_id == b.cell_id && frame_equals(&a.frame, &b.frame)
}

/// The state `SmartBox::UpdateCamera` keeps for the camera's collision.
#[derive(Debug)]
pub struct CameraCollision {
    /// `mpCameraPhysObj`: the object an outdoor camera is placed with. `None` until its shape
    /// has been read (`SmartBox::Init` fails without it, so retail always has it).
    object: Option<PhysicsBody>,
    /// `last_camera_pos_before_transit` and `last_camera_pos_after_transit`: the last position
    /// the camera object was asked to place, and what it made of it.
    before_transit: Position,
    after_transit: Position,
    /// `SmartBox::m_cameraVelocity`: the camera's move this update, times the quantum.
    pub velocity: Vector,
    /// Where the head trace starts when the subject is the drawn body (an AGENTS.md
    /// exception): the subject's physics position, which the physics keeps clear of the world.
    /// The drawn origin is interpolated between two quanta and can sink a few centimetres
    /// into the ground (a crease between terrain cells), where the trace stopped at once and
    /// the camera fell onto the player for a frame.
    trace_origin: Option<Position>,
}

impl Default for CameraCollision {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraCollision {
    /// Without the camera object yet.
    pub fn new() -> Self {
        Self {
            object: None,
            before_transit: Position::default(),
            after_transit: Position::default(),
            velocity: Vector::new(0.0, 0.0, 0.0),
            trace_origin: None,
        }
    }

    /// The physics position to start the head trace from for the next [`Self::trace`] (the
    /// subject's position is then its drawn one); `None` starts it at the subject, as retail.
    pub fn set_trace_origin(&mut self, origin: Option<Position>) {
        self.trace_origin = origin;
    }

    /// `SmartBox::Init`'s camera object: a body of the camera Setup's `shape`, its placement and
    /// movement collisions the camera-ethereal physics set against solid weenies.
    pub fn set_object(&mut self, shape: BodyShape) {
        let mut body = PhysicsBody::new(Position::default(), shape, None);
        let ethereal = camera_ethereal_mask();
        let mut collide = CollisionParameters::default();
        collide.set_physics_placement_type(ethereal);
        collide.set_weenie_placement_type(&WeenieTypes::default());
        collide.set_physics_movement_type(ethereal);
        collide.set_weenie_movement_type(&WeenieTypes::default());
        body.set_collision_state(&collide, &|_| None);
        self.object = Some(body);
    }

    /// Whether the camera object exists.
    pub fn has_object(&self) -> bool {
        self.object.is_some()
    }

    /// `SmartBox::UpdateCamera`'s traces for the camera wanted at `desired`, with the player at
    /// `subject` and the scene camera at `current`. The position to set the camera to, or `None`
    /// where a trace's start cell is not resident (retail then leaves the camera).
    pub fn trace(
        &mut self,
        camera: &mut CameraManager,
        subject: &CameraSubject,
        desired: &Position,
        current: &Position,
        quantum: f32,
        terrain: &Terrain,
    ) -> Option<Position> {
        // The line to the point the camera looks at, from just above the player's origin.
        let offset = camera.get_target_offset(Some(subject));
        let lift = 2.0 * ld(F_EPSILON);
        let from = self.trace_origin.unwrap_or(subject.position);
        let origin = from.frame.origin;
        let start = Vector::new(origin.x, origin.y, st(ld(origin.z) + lift));
        // To the drawn subject's target point: the same line as retail's when the two agree.
        let offset = if self.trace_origin.is_some() {
            let drawn = subject.position.frame.origin;
            let lifted = Vector::new(drawn.x, drawn.y, st(ld(drawn.z) + lift));
            let mut begin = from;
            begin.frame.origin = start;
            let mut end = subject.position;
            end.frame.origin = lifted;
            let to_drawn = begin.offset_to(&end);
            Vector::new(
                st(ld(to_drawn.x) + ld(offset.x)),
                st(ld(to_drawn.y) + ld(offset.y)),
                st(ld(to_drawn.z) + ld(offset.z)),
            )
        } else {
            offset
        };
        let mut head = LosParameters::new();
        head.intersect = RayIntersection::EnvironmentOnly;
        head.set_blocked_by_water(false);
        head.set_ray(Ray::from_offset(start, offset), from.cell_id);
        if !terrain.los_check(&mut head) {
            return None;
        }
        let mut head_position = Position::new(head.final_cell_id(), subject.position.frame);
        head_position.frame.origin = head.end_point();

        // From there to the wanted camera position.
        let mut los = LosParameters::new();
        los.set_from_begin_and_end_position(&head_position, desired);
        los.intersect = if camera.check_env_only() {
            RayIntersection::EnvironmentOnly
        } else {
            RayIntersection::PhysicsRep
        };
        let ethereal = camera_ethereal_mask();
        let solid = WeenieTypes::default();
        los.set_los_params(&solid, ethereal);
        // The player the camera follows never blocks it (an AGENTS.md exception): the ray
        // starts inside the player's upper sphere.
        los.ignore_instance(subject.id);
        let mut clear = terrain.los_check(&mut los);
        let end = los.end_point();
        let mut position = Position::new(los.final_cell_id(), desired.frame);
        position.frame.origin = end;

        if landdefs::is_outside(los.final_cell_id()) {
            position = self.place_outdoors(position, terrain);
        } else {
            // Stand off the wall: 0.2 m along the hit's normal.
            let normal = los.impact_normal();
            let stand_off = 2.0 * ld(0.1_f32);
            let push = Vector::new(
                st(ld(normal.x) * stand_off),
                st(ld(normal.y) * stand_off),
                st(ld(normal.z) * stand_off),
            );
            let mut back = LosParameters::new();
            back.set_ray(Ray::from_offset(end, push), los.final_cell_id());
            back.intersect = RayIntersection::PhysicsRep;
            back.set_los_params(&solid, ethereal);
            back.ignore_instance(subject.id);
            clear &= terrain.los_check(&mut back);
            position.frame.origin = back.end_point();
            position.cell_id = back.final_cell_id();
        }
        if !clear {
            return None;
        }
        let moved = position.offset_to(current);
        self.velocity = Vector::new(
            st(ld(moved.x) * ld(quantum)),
            st(ld(moved.y) * ld(quantum)),
            st(ld(moved.z) * ld(quantum)),
        );
        Some(position)
    }

    /// The outdoor branch: the camera object is placed at `position` (`LeaveWorld`,
    /// `SyncPositionSimple(pos, sliding)`, which makes it enter the world there, and
    /// `LeaveWorld` again) unless the position is the last one asked, whose result is reused.
    /// The object's position, where it has a cell, is the camera's.
    fn place_outdoors(&mut self, position: Position, terrain: &Terrain) -> Position {
        if position_equals(&position, &self.before_transit) {
            return self.after_transit;
        }
        self.before_transit = position;
        let mut result = position;
        if let Some(object) = &mut self.object {
            object.leave_world();
            // `set_cell_id(0)`: the object has no cell until it is placed.
            let mut unplaced = object.position();
            unplaced.cell_id = 0;
            object.set_position(unplaced);
            // `CPhysicsObj::EnterWorldSimple(pos, 1)`: a placement that may slide.
            let _ = object.place(position, crate::remotes::ENTER_WORLD_FLAGS, terrain);
            object.set_placement_position(&position);
            let placed = object.position();
            if placed.cell_id != 0 {
                result = placed;
            }
            object.leave_world();
        }
        self.after_transit = result;
        result
    }
}

/// The camera object's Setup, from the DAT: `FileArray::GetDIDFromEnum(0x4000002a, 0x40000002)`.
pub fn camera_object_did(
    store: &ac2_db::ResourceStore,
) -> Result<Option<DataId>, ac2_db::ResourceError> {
    ac2_assets::ac2_enum_did(store, CAMERA_OBJECT_ENUM.1, CAMERA_OBJECT_ENUM.0)
}

impl SceneCamera {
    /// [`Self::update_with`] with `SmartBox::UpdateCamera`'s collision: the camera manager's
    /// desired position is pulled in to what the line of sight from the player's head to it
    /// meets ([`CameraCollision::trace`]). With no ray trace (first person, or a target other
    /// than the player) the desired position stands, as before.
    #[allow(clippy::too_many_arguments)]
    pub fn update_collided(
        &mut self,
        camera: &mut CameraManager,
        subject: &CameraSubject,
        objects: &dyn CameraObjects,
        collision: &mut CameraCollision,
        terrain: &Terrain,
        quantum: f32,
        interpolate: bool,
    ) {
        let current = self.current;
        let desired = camera.update_camera_with(quantum, Some(subject), objects, &current);
        self.fov = camera.fov();
        self.previous = self.current;
        if camera.no_ray_trace() {
            if crate::scene::frame_is_valid(&desired.frame) {
                self.current = desired;
            }
            return;
        }
        if let Some(position) =
            collision.trace(camera, subject, &desired, &current, quantum, terrain)
        {
            self.set_position(position, !interpolate);
        }
    }
}

#[cfg(test)]
#[path = "camera_collision_tests.rs"]
mod tests;
