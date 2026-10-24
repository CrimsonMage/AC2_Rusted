//! `CTransition`: moves a body's spheres from a begin to an end position in steps no longer than
//! its radius, colliding each step with the cells it overlaps and stepping up, down and along
//! the terrain like the retail client (`find_transitional_position`, `transitional_insert`,
//! `validate_transition`).

pub(crate) mod collision;
pub(crate) mod sphere_path;
mod steps;

use crate::cells::{find_cells, find_collisions, CellSource};
use crate::fpu::{epsilon_squared, ld, st, F_EPSILON, LANDING_Z};
use crate::landdefs::{adjust_to_outside, is_outside};
use crate::position::Position;
use crate::vector::{Plane, Sphere, Vector};
use collision::{
    CollisionInfo, ObjectInfo, CONTACT_OI, FREE_ROTATE_OI, IS_VIEWER_OI, ON_WALKABLE_OI,
    PATH_CLIPPED_OI,
};
use sphere_path::{CellRef, InsertType, SpherePath};

/// `TransitionState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionState {
    Invalid,
    Ok,
    Collided,
    Adjusted,
    /// Produced by physics-BSP sphere slides, which this port does not have yet.
    #[allow(dead_code)]
    Slid,
}

/// `CELLARRAY`: the cells one check position touches, in insertion order.
#[derive(Debug, Clone, Default)]
pub(crate) struct CellArray {
    pub added_outside: bool,
    pub cells: Vec<(u32, bool)>,
}

impl CellArray {
    fn reset(&mut self) {
        self.cells.clear();
        self.added_outside = false;
    }

    /// `CELLARRAY::add_cell`: each cell id once.
    pub(crate) fn add_cell(&mut self, cell_id: u32, resident: bool) {
        if !self.cells.iter().any(|&(id, _)| id == cell_id) {
            self.cells.push((cell_id, resident));
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Transition {
    pub object_info: ObjectInfo,
    pub sphere_path: SpherePath,
    pub collision_info: CollisionInfo,
    pub cell_array: CellArray,
}

/// The object traits `CPhysicsObj::transition` hands `CTransition` (`get_object_info`,
/// `init_sphere`, step heights).
#[derive(Debug, Clone, Copy)]
pub(crate) struct TransitionObject<'a> {
    pub state: u32,
    pub spheres: &'a [Sphere],
    pub scale: f32,
    pub step_up_height: f32,
    pub step_down_height: f32,
    pub affected_by_gravity: bool,
    /// `CTransition::init_contact_plane` from the object's contact, when in contact.
    pub contact: Option<(u32, Plane, bool, bool, bool)>,
    pub sliding_normal: Option<Vector>,
    pub frames_stationary_fall: i32,
}

impl Transition {
    /// `CTransition::makeTransition` + `init_object` + `init_sphere` + `init_path` as
    /// `CPhysicsObj::transition` sequences them.
    pub(crate) fn new(
        object: &TransitionObject<'_>,
        begin_cell: CellRef,
        begin: Option<Position>,
        end: Position,
    ) -> Self {
        let mut transition = Self::default();
        transition.sphere_path.init();
        transition.collision_info.init();
        transition.object_info.init(
            object.state,
            object.scale,
            object.step_up_height,
            object.step_down_height,
        );
        transition.object_info.affected_by_gravity = object.affected_by_gravity;
        if let Some((cell_id, plane, is_water, is_landscape, is_environment)) = object.contact {
            transition.init_contact_plane(cell_id, plane, is_water, is_landscape, is_environment);
        }
        if let Some(normal) = object.sliding_normal {
            transition.collision_info.set_sliding_normal(normal);
        }
        transition
            .sphere_path
            .init_sphere(object.spheres, object.scale);
        transition.sphere_path.init_path(begin_cell, begin, end);
        transition.collision_info.frames_stationary_fall = object.frames_stationary_fall;
        transition
    }

    /// `CTransition::init_contact_plane`: the object's contact as both the last known and the
    /// current contact plane.
    fn init_contact_plane(
        &mut self,
        cell_id: u32,
        plane: Plane,
        is_water: bool,
        is_landscape: bool,
        is_environment: bool,
    ) {
        let c = &mut self.collision_info;
        c.last_known_contact_plane_valid = true;
        c.last_known_contact_plane = plane;
        c.last_known_contact_plane_is_water = is_water;
        c.last_known_contact_plane_is_landscape = is_landscape;
        c.last_known_contact_plane_is_environment = is_environment;
        c.last_known_contact_plane_cell_id = cell_id;
        c.set_contact_plane(plane, is_water, is_landscape, is_environment);
        c.contact_plane_cell_id = cell_id;
    }

    /// `CTransition::find_valid_position`.
    pub(crate) fn find_valid_position(&mut self, cells: &dyn CellSource) -> bool {
        match self.sphere_path.insert_type {
            InsertType::Transition => self.find_transitional_position(cells),
            // `find_placement_position` is not part of this port.
            InsertType::Placement => false,
        }
    }

    /// `CTransition::calc_num_steps`: steps of at most 0.9 radius (a viewer steps a whole radius
    /// and takes one step more).
    fn calc_num_steps(&self) -> (Vector, Vector, u32) {
        let path = &self.sphere_path;
        let Some(begin) = path.begin_pos else {
            return (Vector::ZERO, Vector::ZERO, 1);
        };
        let offset = begin.offset_to(&path.end_pos);
        let radius = ld(path.local_sphere[0].radius);
        let (x, y, z) = (ld(offset.x), ld(offset.y), ld(offset.z));
        let dist = ((x * x + y * y) + z * z).sqrt();
        if self.object_info.state & IS_VIEWER_OI != 0 {
            if dist <= ld(F_EPSILON) {
                return (offset, Vector::ZERO, 0);
            }
            let steps = dist / radius;
            let per_step = scale_offset(offset, 1.0 / steps);
            return (offset, per_step, steps.floor() as i64 as u32 + 1);
        }
        let steps = dist / (ld(0.9_f32) * radius);
        if steps > 1.0 {
            let whole = steps.ceil();
            let per_step = scale_offset(offset, 1.0 / whole);
            return (offset, per_step, whole as i64 as u32);
        }
        if offset == Vector::ZERO {
            return (offset, Vector::ZERO, 0);
        }
        (offset, offset, 1)
    }

    /// `CTransition::find_transitional_position`.
    fn find_transitional_position(&mut self, cells: &dyn CellSource) -> bool {
        if self.sphere_path.begin_cell.is_none() {
            return false;
        }
        let mut ts = TransitionState::Ok;
        let (offset, mut per_step, num_steps) = self.calc_num_steps();
        let free_rotate = self.object_info.state & FREE_ROTATE_OI != 0;
        if free_rotate {
            let q = self.sphere_path.end_pos.frame.q;
            self.sphere_path.curr_pos.frame.set_rotate(q);
        }
        let path = &mut self.sphere_path;
        path.check_pos = path.curr_pos;
        path.check_cell = path.curr_cell;
        path.cell_array_valid = false;
        path.cache_global_sphere(None);
        if num_steps == 0 {
            if !free_rotate {
                let q = self.sphere_path.end_pos.frame.q;
                self.sphere_path.curr_pos.frame.set_rotate(q);
            }
            let (curr, cell) = (self.sphere_path.curr_pos, self.sphere_path.curr_cell);
            self.sphere_path.set_check_pos(curr, cell);
            self.build_cell_array(cells);
            return true;
        }
        let mut step = 0;
        loop {
            let viewer = self.object_info.state & IS_VIEWER_OI != 0;
            if viewer && step == num_steps - 1 {
                let dist = offset.mag2_xyz().sqrt();
                if dist > ld(F_EPSILON) {
                    let done = f64::from(num_steps - 1);
                    let radius = ld(self.sphere_path.local_sphere[0].radius);
                    let scale = (dist - done * radius) / dist;
                    per_step = Vector::new(
                        st(ld(offset.x) * scale),
                        st(ld(offset.y) * scale),
                        st(ld(offset.z) * scale),
                    );
                }
            }
            let adjusted = self.adjust_offset(per_step);
            self.sphere_path.global_offset = adjusted;
            if self.object_info.state & IS_VIEWER_OI == 0 && adjusted.mag2_xyz() < epsilon_squared()
            {
                return step != 0 && ts == TransitionState::Ok;
            }
            if !free_rotate {
                let t = st(f64::from(step + 1) / f64::from(num_steps));
                let (from, to) = (
                    self.sphere_path
                        .begin_pos
                        .map(|p| p.frame)
                        .unwrap_or_default(),
                    self.sphere_path.end_pos.frame,
                );
                self.sphere_path
                    .check_pos
                    .frame
                    .slerp_rotation(&from, &to, t);
            }
            self.collision_info.sliding_normal_valid = false;
            self.collision_info.clear_contact_plane();
            if self.sphere_path.insert_type == InsertType::Transition {
                let path = &mut self.sphere_path;
                path.cell_array_valid = false;
                let origin = &mut path.check_pos.frame.origin;
                origin.x = st(ld(origin.x) + ld(adjusted.x));
                origin.y = st(ld(adjusted.y) + ld(origin.y));
                origin.z = st(ld(adjusted.z) + ld(origin.z));
                path.cache_global_sphere(Some(adjusted));
                let inserted = self.transitional_insert(3, cells);
                ts = self.validate_transition(inserted, cells);
                if self.collision_info.frames_stationary_fall != 0 {
                    break;
                }
            } else {
                // Placement inserts are not part of this port.
                return false;
            }
            if (self.collision_info.collision_normal_valid
                && self.object_info.state & PATH_CLIPPED_OI != 0)
                || {
                    step += 1;
                    step >= num_steps
                }
            {
                break;
            }
        }
        ts == TransitionState::Ok
    }

    /// `CTransition::build_cell_array`.
    fn build_cell_array(&mut self, cells: &dyn CellSource) -> Option<u32> {
        self.cell_array.reset();
        self.sphere_path.cell_array_valid = true;
        self.sphere_path.hits_interior_cell = false;
        find_cells(self, cells)
    }

    /// `CTransition::transitional_insert`: up to `attempts` collision passes of the check
    /// position against its cell and then the other cells it touches, stepping down onto
    /// walkable ground after an unobstructed pass.
    pub(crate) fn transitional_insert(
        &mut self,
        attempts: i32,
        cells: &dyn CellSource,
    ) -> TransitionState {
        if self.sphere_path.check_cell.is_none() {
            return TransitionState::Ok;
        }
        let mut result = TransitionState::Invalid;
        for _ in 0..attempts {
            let cell = self
                .sphere_path
                .check_cell
                .and_then(|id| cells.land_cell(id));
            let mut ts = match cell {
                Some(cell) => find_collisions(self, cell),
                None => TransitionState::Collided,
            };
            match ts {
                TransitionState::Ok => {
                    ts = self.check_other_cells(self.sphere_path.check_cell, cells);
                    if ts != TransitionState::Ok {
                        self.sphere_path.neg_poly_hit = false;
                    }
                    if ts == TransitionState::Collided {
                        return ts;
                    }
                }
                TransitionState::Collided => {
                    self.sphere_path.neg_poly_hit = false;
                    return ts;
                }
                TransitionState::Slid => {
                    self.collision_info.clear_contact_plane();
                    self.sphere_path.neg_poly_hit = false;
                }
                TransitionState::Adjusted => self.sphere_path.neg_poly_hit = false,
                TransitionState::Invalid => {}
            }
            result = ts;
            if ts != TransitionState::Ok {
                continue;
            }
            if self.sphere_path.collide {
                return self.insert_after_collide(cells);
            }
            let path = &self.sphere_path;
            if path.neg_poly_hit && !path.step_down && !path.step_up {
                // Physics-BSP negative polygon hits (`neg_step_up`, `CSphere::slide_sphere`)
                // are not produced by terrain.
                self.sphere_path.neg_poly_hit = false;
                continue;
            }
            let state = self.object_info.state;
            if self.collision_info.contact_plane_valid
                || state & CONTACT_OI == 0
                || self.sphere_path.step_down
                || self.sphere_path.check_cell.is_none()
                || !self.object_info.step_down
            {
                return TransitionState::Ok;
            }
            match self.step_down_after_insert(cells) {
                Ok(ts) => return ts,
                Err(ts) => result = ts,
            }
        }
        result
    }

    /// The `collide` branch of `transitional_insert`: a placement insert on a walkable contact,
    /// else back to the saved position with the step-up normal as the collision.
    fn insert_after_collide(&mut self, cells: &dyn CellSource) -> TransitionState {
        self.sphere_path.collide = false;
        let mut blocked = true;
        if self.collision_info.contact_plane_valid && self.check_walkable(LANDING_Z, cells) {
            self.sphere_path.backup = self.sphere_path.insert_type;
            self.sphere_path.insert_type = InsertType::Placement;
            let placed = self.transitional_insert(1, cells);
            self.sphere_path.insert_type = self.sphere_path.backup;
            blocked = placed != TransitionState::Ok;
        }
        self.sphere_path.walkable = false;
        if !blocked {
            return TransitionState::Ok;
        }
        self.sphere_path.restore_check_pos();
        self.collision_info.clear_contact_plane();
        if !self.collision_info.last_known_contact_plane_valid {
            let normal = self.sphere_path.step_up_normal;
            self.collision_info.set_collision_normal(normal);
            return TransitionState::Collided;
        }
        self.object_info.kill_velocity = true;
        self.collision_info.last_known_contact_plane_valid = false;
        TransitionState::Collided
    }

    /// The step-down tail of `transitional_insert`: an object in contact that found no contact
    /// plane steps down (half the radius at most for one sphere), then edge-slides.
    /// `Ok` returns from `transitional_insert`; `Err` carries an edge slide that did not finish,
    /// whose state the next insertion attempt starts from.
    fn step_down_after_insert(
        &mut self,
        cells: &dyn CellSource,
    ) -> Result<TransitionState, TransitionState> {
        let (mut height, z) = if self.object_info.state & ON_WALKABLE_OI != 0 {
            (self.object_info.step_down_height, ObjectInfo::walkable_z())
        } else {
            (0.04, LANDING_Z)
        };
        self.sphere_path.walkable_allowance = z;
        self.sphere_path.save_check_pos();
        let radius = ld(self.sphere_path.global_sphere[0].radius);
        if self.sphere_path.num_sphere < 2 && radius + radius < ld(height) {
            height = st(radius * 0.5);
        }
        let radius = ld(self.sphere_path.global_sphere[0].radius);
        let stepped = if ld(height) <= radius + radius {
            self.step_down(height, z, cells)
        } else {
            height = st(ld(height) * 0.5);
            self.step_down(height, z, cells) || self.step_down(height, z, cells)
        };
        if stepped {
            self.sphere_path.walkable = false;
            return Ok(TransitionState::Ok);
        }
        let (finished, ts) = self.edge_slide(height, z, cells);
        if finished {
            Ok(ts)
        } else {
            Err(ts)
        }
    }

    /// `CTransition::validate_transition`: accept the check position (or fall back to the
    /// current one with the last known contact plane), keep the sliding normal and count frames
    /// spent falling in place.
    pub(crate) fn validate_transition(
        &mut self,
        ts: TransitionState,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let mut ts = ts;
        let mut on_ground = true;
        let unchanged = ts == TransitionState::Ok
            && self.sphere_path.check_pos.cell_id == self.sphere_path.curr_pos.cell_id
            && self
                .sphere_path
                .check_pos
                .frame
                .approx_eq(&self.sphere_path.curr_pos.frame);
        if ts == TransitionState::Ok && !unchanged {
            let path = &mut self.sphere_path;
            path.curr_pos = path.check_pos;
            path.curr_cell = path.check_cell;
            path.cache_global_curr_center();
            path.check_pos = path.curr_pos;
            path.check_cell = path.curr_cell;
            path.cell_array_valid = false;
            path.cache_global_sphere(None);
        } else {
            on_ground = false;
            if ts != TransitionState::Ok {
                self.revert_to_current(&mut on_ground, cells);
                ts = TransitionState::Ok;
            }
        }
        if self.collision_info.collision_normal_valid {
            let normal = self.collision_info.collision_normal;
            self.collision_info.set_sliding_normal(normal);
        }
        if self.object_info.state & IS_VIEWER_OI == 0 && self.object_info.affected_by_gravity {
            self.count_stationary_frames(on_ground);
        }
        let c = &mut self.collision_info;
        c.last_known_contact_plane_valid = c.contact_plane_valid;
        if c.contact_plane_valid {
            c.last_known_contact_plane = c.contact_plane;
            c.last_known_contact_plane_cell_id = c.contact_plane_cell_id;
            c.last_known_contact_plane_is_water = c.contact_plane_is_water;
            c.last_known_contact_plane_is_landscape = c.contact_plane_is_landscape;
            c.last_known_contact_plane_is_environment = c.contact_plane_is_environment;
            self.object_info.state |= CONTACT_OI;
            if ObjectInfo::is_valid_walkable(c.contact_plane.n) {
                self.object_info.state |= ON_WALKABLE_OI;
            } else {
                self.object_info.state &= !ON_WALKABLE_OI;
            }
        } else {
            self.object_info.state &= !(CONTACT_OI | ON_WALKABLE_OI);
        }
        ts
    }

    /// The failed-insert branch of `validate_transition`: within reach of the last known contact
    /// plane it becomes the contact again; the check position returns to the current one.
    fn revert_to_current(&mut self, on_ground: &mut bool, cells: &dyn CellSource) {
        let c = &self.collision_info;
        if c.last_known_contact_plane_valid {
            let center = self.sphere_path.global_curr_center[0];
            let plane = c.last_known_contact_plane;
            let n = plane.n;
            let dist = ((ld(center.z) * ld(n.z) + ld(center.y) * ld(n.y)) + ld(center.x) * ld(n.x))
                + ld(plane.d);
            let reach = ld(F_EPSILON) + ld(self.sphere_path.global_sphere[0].radius);
            if dist.abs() < reach {
                let (is_water, is_landscape) = (
                    c.last_known_contact_plane_is_water,
                    c.last_known_contact_plane_is_landscape,
                );
                let cell_id = c.last_known_contact_plane_cell_id;
                self.collision_info
                    .set_contact_plane(plane, is_water, is_landscape, false);
                self.collision_info.contact_plane_cell_id = cell_id;
                if self.object_info.state & ON_WALKABLE_OI != 0 {
                    *on_ground = true;
                }
            }
        }
        if !self.collision_info.collision_normal_valid {
            self.collision_info
                .set_collision_normal(Vector::new(0.0, 0.0, 1.0));
        }
        let (curr, cell) = (self.sphere_path.curr_pos, self.sphere_path.curr_cell);
        self.sphere_path.set_check_pos(curr, cell);
        self.build_cell_array(cells);
    }

    /// `validate_transition`'s `frames_stationary_fall`: three frames without ground under a
    /// falling body give it a flat contact plane at its feet.
    fn count_stationary_frames(&mut self, on_ground: bool) {
        if on_ground {
            self.collision_info.frames_stationary_fall = 0;
            return;
        }
        match self.collision_info.frames_stationary_fall {
            0 => self.collision_info.frames_stationary_fall = 1,
            1 => self.collision_info.frames_stationary_fall = 2,
            _ => {
                let sphere = self.sphere_path.global_sphere[0];
                self.collision_info.frames_stationary_fall = 3;
                let c = sphere.center;
                let d = st(ld(sphere.radius) - ((ld(c.y) + ld(c.x)) * 0.0 + ld(c.z)));
                let plane = Plane::new(Vector::new(0.0, 0.0, 1.0), d);
                self.collision_info
                    .set_contact_plane(plane, false, false, false);
                self.collision_info.contact_plane_cell_id = self.sphere_path.check_pos.cell_id;
                if self.object_info.state & CONTACT_OI == 0 {
                    self.collision_info.set_collision_normal(plane.n);
                    self.collision_info.collided_with_environment = true;
                }
            }
        }
    }

    /// `CTransition::check_other_cells`: collide with every other cell the check position
    /// touches, then move the check position into the cell holding it.
    fn check_other_cells(&mut self, current: CellRef, cells: &dyn CellSource) -> TransitionState {
        self.cell_array.reset();
        self.sphere_path.cell_array_valid = true;
        self.sphere_path.hits_interior_cell = false;
        let new_cell = find_cells(self, cells);
        let mut ts = TransitionState::Ok;
        let touched = self.cell_array.cells.clone();
        for (cell_id, resident) in touched {
            if !resident || Some(cell_id) == current {
                continue;
            }
            let Some(cell) = cells.land_cell(cell_id) else {
                continue;
            };
            ts = find_collisions(self, cell);
            match ts {
                TransitionState::Collided | TransitionState::Adjusted => return ts,
                TransitionState::Slid => {
                    self.collision_info.clear_contact_plane();
                    return ts;
                }
                _ => {}
            }
        }
        self.sphere_path.check_cell = new_cell;
        if let Some(cell_id) = new_cell {
            self.sphere_path.adjust_check_pos(cell_id);
            return ts;
        }
        if self.sphere_path.step_down {
            return TransitionState::Collided;
        }
        let mut position = self.sphere_path.check_pos;
        if is_outside(position.cell_id) {
            adjust_to_outside(&mut position.cell_id, &mut position.frame.origin);
        }
        if position.cell_id == 0 {
            return TransitionState::Collided;
        }
        self.sphere_path.adjust_check_pos(position.cell_id);
        self.sphere_path.check_pos = position;
        self.sphere_path.check_cell = None;
        self.sphere_path.cell_array_valid = false;
        self.sphere_path.cache_global_sphere(None);
        self.sphere_path.cell_array_valid = true;
        ts
    }
}

/// Scales an offset by an inverse step count: the x term uses the unrounded inverse, the y and
/// z terms the stored one (`calc_num_steps`'s `fst`).
fn scale_offset(offset: Vector, inverse: f64) -> Vector {
    let stored = ld(st(inverse));
    Vector::new(
        st(inverse * ld(offset.x)),
        st(stored * ld(offset.y)),
        st(stored * ld(offset.z)),
    )
}
