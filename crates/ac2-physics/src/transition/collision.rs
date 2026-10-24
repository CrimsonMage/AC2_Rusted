//! `OBJECTINFO` (the moving object's collision traits) and `COLLISIONINFO` (what one transition
//! touched), with `OBJECTINFO::validate_walkable`, the terrain contact response.

use crate::fpu::{floor_z, is_float_zero, ld, st, F_EPSILON};
use crate::landdefs::is_outside;
use crate::transition::sphere_path::SpherePath;
use crate::transition::TransitionState;
use crate::vector::{Plane, Sphere, Vector};

/// `ObjectInfoEnum`.
pub(crate) const CONTACT_OI: u32 = 0x1;
pub(crate) const ON_WALKABLE_OI: u32 = 0x2;
pub(crate) const IS_VIEWER_OI: u32 = 0x4;
pub(crate) const PATH_CLIPPED_OI: u32 = 0x8;
pub(crate) const FREE_ROTATE_OI: u32 = 0x10;
pub(crate) const PERFECT_CLIP_OI: u32 = 0x40;
pub(crate) const EDGE_SLIDE_OI: u32 = 0x80;
pub(crate) const PREDETERMINED_TRANSITION_OI: u32 = 0x200;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ObjectInfo {
    pub state: u32,
    pub scale: f32,
    pub step_up_height: f32,
    pub step_down_height: f32,
    pub step_down: bool,
    pub max_step: f32,
    /// `CPhysicsObj::is_affected_by_gravity` of the object, read by `validate_transition`.
    pub affected_by_gravity: bool,
    /// Set by `OBJECTINFO::kill_velocity`; the object applies `set_velocity(0)` after the move.
    pub kill_velocity: bool,
    /// The moving object's `m_fPhysicsContainmentRadius` (`CPhysicsObj::CanCollide`).
    pub containment_radius: f32,
}

impl ObjectInfo {
    /// `OBJECTINFO::init`: step heights from the object; `max_step` is the larger magnitude.
    pub(crate) fn init(&mut self, state: u32, scale: f32, step_up: f32, step_down: f32) {
        self.state = state;
        self.scale = scale;
        self.step_up_height = step_up;
        self.step_down_height = step_down;
        self.step_down = ld(step_down) != 0.0;
        let down = ld(step_down).abs();
        let up = ld(step_up).abs();
        self.max_step = st(if down <= up { up } else { down });
        self.kill_velocity = false;
    }

    /// `OBJECTINFO::is_valid_walkable` (`CPhysicsObj::is_valid_walkable`).
    pub(crate) fn is_valid_walkable(normal: Vector) -> bool {
        ld(floor_z()) <= ld(normal.z)
    }

    /// `OBJECTINFO::get_walkable_z` (`CPhysicsObj::get_walkable_z`).
    pub(crate) fn walkable_z() -> f32 {
        floor_z()
    }

    pub(crate) fn has(&self, flag: u32) -> bool {
        self.state & flag != 0
    }

    /// `OBJECTINFO::validate_walkable`: the response of `sphere` (the global check sphere moved
    /// into the polygon's block) to a walkable plane. A viewer is pushed back along its path out
    /// of the plane; a body sinking below it is lifted (or, while stepping down, interpolated)
    /// and touching it records the contact plane.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn validate_walkable(
        &self,
        sphere: Sphere,
        plane: Plane,
        is_water: bool,
        water_depth: f32,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        land_cell_id: u32,
        is_landscape: bool,
    ) -> TransitionState {
        let n = plane.n;
        let c = sphere.center;
        if self.has(IS_VIEWER_OI) {
            let dist = ((ld(c.y) * ld(n.y) + ld(c.z) * ld(n.z)) + ld(c.x) * ld(n.x)) + ld(plane.d)
                - ld(sphere.radius);
            if dist > -ld(F_EPSILON) {
                return TransitionState::Ok;
            }
            // The distance is stored before the divide.
            let dist = st(dist);
            let curr = path.global_curr_center[0];
            let ox = ld(c.x) - ld(curr.x);
            let oy = ld(st(ld(c.y) - ld(curr.y)));
            let oz = ld(c.z) - ld(curr.z);
            let angle = ld(dist) / ((oz * ld(n.z) + ox * ld(n.x)) + oy * ld(n.y));
            let within = angle > 0.0 && angle <= 1.0;
            let outside_start = path.begin_pos.is_none_or(|begin| is_outside(begin.cell_id));
            if within || outside_start {
                let back = -angle;
                path.add_offset_to_check_pos(Vector::new(
                    st(ox * back),
                    st(oy * back),
                    st(oz * back),
                ));
                collisions.set_collision_normal(n);
                collisions.collided_with_environment = true;
                return TransitionState::Adjusted;
            }
            return TransitionState::Ok;
        }
        let dist = (((ld(c.z) - ld(sphere.radius)) * ld(n.z) + ld(c.x) * ld(n.x))
            + ld(c.y) * ld(n.y))
            + ld(plane.d)
            + ld(water_depth);
        if dist < -ld(F_EPSILON) {
            if path.check_walkable {
                return TransitionState::Collided;
            }
            let z_dist = st(dist / ld(n.z));
            let valid = Self::is_valid_walkable(n);
            let mut ts = TransitionState::Ok;
            if path.step_down || !self.has(ON_WALKABLE_OI) || valid {
                collisions.set_contact_plane(plane, is_water, is_landscape, false);
                collisions.contact_plane_cell_id = land_cell_id;
                if path.step_down {
                    let amount = st(ld(path.step_down_amt) * ld(path.walk_interp));
                    if !is_float_zero(amount, F_EPSILON) {
                        let interp =
                            (1.0 - (-1.0 / ld(amount)) * ld(z_dist)) * ld(path.walk_interp);
                        if !(interp < ld(path.walk_interp)) || interp < -0.1 {
                            return TransitionState::Collided;
                        }
                        path.walk_interp = st(interp);
                    }
                }
                path.add_offset_to_check_pos(Vector::new(0.0, 0.0, st(-ld(z_dist))));
                ts = TransitionState::Adjusted;
            }
            if !self.has(CONTACT_OI) && !path.step_down {
                collisions.set_collision_normal(n);
                collisions.collided_with_environment = true;
            }
            return ts;
        }
        if dist > ld(F_EPSILON) {
            return TransitionState::Ok;
        }
        let valid = Self::is_valid_walkable(n);
        if path.step_down || !self.has(ON_WALKABLE_OI) || valid {
            collisions.set_contact_plane(plane, is_water, is_landscape, false);
            collisions.contact_plane_cell_id = land_cell_id;
        }
        if !self.has(CONTACT_OI) {
            collisions.set_collision_normal(n);
            collisions.collided_with_environment = true;
        }
        TransitionState::Ok
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CollisionInfo {
    pub last_known_contact_plane_valid: bool,
    pub last_known_contact_plane: Plane,
    pub last_known_contact_plane_is_water: bool,
    pub last_known_contact_plane_is_landscape: bool,
    pub last_known_contact_plane_is_environment: bool,
    pub last_known_contact_plane_cell_id: u32,
    pub contact_plane_valid: bool,
    pub contact_plane: Plane,
    pub contact_plane_cell_id: u32,
    pub contact_plane_is_water: bool,
    pub contact_plane_is_landscape: bool,
    pub contact_plane_is_environment: bool,
    pub sliding_normal_valid: bool,
    pub sliding_normal: Vector,
    pub collision_normal_valid: bool,
    pub collision_normal: Vector,
    /// `collide_object`: the static objects the transition was stopped by, once each.
    pub collide_objects: Vec<u32>,
    pub collided_with_environment: bool,
    pub frames_stationary_fall: i32,
}

impl CollisionInfo {
    /// `COLLISIONINFO::init`.
    pub(crate) fn init(&mut self) {
        self.last_known_contact_plane_valid = false;
        self.contact_plane_valid = false;
        self.sliding_normal_valid = false;
        self.collision_normal_valid = false;
        self.collided_with_environment = false;
        self.contact_plane_cell_id = 0;
        self.frames_stationary_fall = 0;
        self.contact_plane_is_water = false;
        self.contact_plane_is_landscape = false;
        self.contact_plane_is_environment = false;
    }

    /// `COLLISIONINFO::AddObject` (static objects are never ethereal, so each is also the last
    /// collided object).
    pub(crate) fn add_object(&mut self, id: u32) {
        if !self.collide_objects.contains(&id) {
            self.collide_objects.push(id);
        }
    }

    /// `COLLISIONINFO::set_contact_plane`.
    pub(crate) fn set_contact_plane(
        &mut self,
        plane: Plane,
        is_water: bool,
        is_landscape: bool,
        is_environment: bool,
    ) {
        self.contact_plane_valid = true;
        self.contact_plane = plane;
        self.contact_plane_is_water = is_water;
        self.contact_plane_is_landscape = is_landscape;
        self.contact_plane_is_environment = is_environment;
    }

    /// The four stores retail repeats to drop the contact plane.
    pub(crate) fn clear_contact_plane(&mut self) {
        self.contact_plane_valid = false;
        self.contact_plane_is_water = false;
        self.contact_plane_is_landscape = false;
        self.contact_plane_is_environment = false;
    }

    /// `COLLISIONINFO::set_collision_normal`: stored normalised, zero when tiny.
    pub(crate) fn set_collision_normal(&mut self, normal: Vector) {
        self.collision_normal_valid = true;
        self.collision_normal = normal;
        self.collision_normal.normalize_check_small();
    }

    /// `COLLISIONINFO::set_sliding_normal`: the horizontal part, normalised.
    pub(crate) fn set_sliding_normal(&mut self, normal: Vector) {
        self.sliding_normal_valid = true;
        self.sliding_normal = Vector::new(normal.x, normal.y, 0.0);
        self.sliding_normal.normalize_check_small();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;
    use crate::position::Position;
    use crate::vector::Quaternion;

    fn path_with_sphere(center: Vector) -> SpherePath {
        let mut path = SpherePath::default();
        path.init();
        path.init_sphere(&[Sphere::new(Vector::new(0.0, 0.0, 0.5), 0.5)], 1.0);
        let origin = Vector::new(center.x, center.y, center.z - 0.5);
        let position = Position::new(0x7F7F_0001, Frame::new(origin, Quaternion::IDENTITY));
        path.init_path(Some(position.cell_id), Some(position), position);
        path.set_check_pos(position, Some(position.cell_id));
        path
    }

    fn walker() -> ObjectInfo {
        let mut info = ObjectInfo::default();
        info.init(CONTACT_OI | ON_WALKABLE_OI, 1.0, 0.5, 0.5);
        info
    }

    #[test]
    fn a_body_sunk_into_flat_ground_is_lifted_onto_it() {
        let ground = Plane::new(Vector::new(0.0, 0.0, 1.0), -10.0);
        let mut path = path_with_sphere(Vector::new(5.0, 5.0, 10.3));
        let mut collisions = CollisionInfo::default();
        let sphere = path.global_sphere[0];
        let ts = walker().validate_walkable(
            sphere,
            ground,
            false,
            0.0,
            &mut path,
            &mut collisions,
            0x7F7F_0001,
            true,
        );
        assert_eq!(ts, TransitionState::Adjusted);
        assert!((path.check_pos.frame.origin.z - 10.0).abs() < 1e-6);
        assert!(collisions.contact_plane_valid && collisions.contact_plane_is_landscape);
        assert!(
            !collisions.collision_normal_valid,
            "in contact: no collision normal"
        );
    }

    #[test]
    fn touching_and_clear_of_the_ground() {
        let ground = Plane::new(Vector::new(0.0, 0.0, 1.0), -10.0);
        let mut collisions = CollisionInfo::default();
        let mut path = path_with_sphere(Vector::new(5.0, 5.0, 10.5));
        let sphere = path.global_sphere[0];
        let mut airborne = ObjectInfo::default();
        airborne.init(0, 1.0, 0.5, 0.5);
        let ts = airborne.validate_walkable(
            sphere,
            ground,
            false,
            0.0,
            &mut path,
            &mut collisions,
            0x7F7F_0001,
            true,
        );
        assert_eq!(ts, TransitionState::Ok);
        assert!(collisions.contact_plane_valid);
        assert!(collisions.collision_normal_valid && collisions.collided_with_environment);
        let mut collisions = CollisionInfo::default();
        let mut path = path_with_sphere(Vector::new(5.0, 5.0, 11.0));
        let sphere = path.global_sphere[0];
        let ts = airborne.validate_walkable(
            sphere,
            ground,
            false,
            0.0,
            &mut path,
            &mut collisions,
            0x7F7F_0001,
            true,
        );
        assert_eq!(ts, TransitionState::Ok);
        assert!(!collisions.contact_plane_valid);
    }

    #[test]
    fn a_walker_does_not_take_a_steep_contact() {
        // 60 degrees: steeper than floor_z.
        let steep = Plane::new(Vector::new(0.866_025_4, 0.0, 0.5), -5.0);
        let mut path = path_with_sphere(Vector::new(0.0, 0.0, 9.0));
        let mut collisions = CollisionInfo::default();
        let sphere = path.global_sphere[0];
        let ts = walker().validate_walkable(
            sphere,
            steep,
            false,
            0.0,
            &mut path,
            &mut collisions,
            0x7F7F_0001,
            true,
        );
        assert_eq!(ts, TransitionState::Ok);
        assert!(!collisions.contact_plane_valid);
    }

    #[test]
    fn normals_store_normalised() {
        let mut collisions = CollisionInfo::default();
        collisions.set_sliding_normal(Vector::new(3.0, 4.0, 9.0));
        assert_eq!(collisions.sliding_normal, Vector::new(0.6, 0.8, 0.0));
        collisions.set_collision_normal(Vector::new(0.0, 0.0, 2.0));
        assert_eq!(collisions.collision_normal, Vector::new(0.0, 0.0, 1.0));
    }
}
