//! `SPHEREPATH`: the moving body's spheres in local, global and check space along one
//! transition.

use crate::bsp::Polygon;
use crate::fpu::{ld, st};
use crate::landdefs::{get_block_offset, is_outside};
use crate::position::Position;
use crate::vector::{Sphere, Vector};

/// `SPHEREPATH::InsertType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum InsertType {
    #[default]
    Transition,
    Placement,
    InitialPlacement,
}

/// A cell the path is in: a resident land cell id, or `None` (`CObjCell *` null).
pub(crate) type CellRef = Option<u32>;

#[derive(Debug, Clone, Default)]
pub(crate) struct SpherePath {
    pub num_sphere: usize,
    pub local_sphere: [Sphere; 2],
    pub local_low_point: Vector,
    pub global_sphere: [Sphere; 2],
    pub global_low_point: Vector,
    pub global_curr_center: [Vector; 2],
    /// The spheres in the space of the object or cell being collided with
    /// (`cache_localspace_sphere`).
    pub localspace_sphere: [Sphere; 2],
    pub localspace_low_point: Vector,
    pub localspace_curr_center: [Vector; 2],
    pub localspace_pos: Position,
    /// World up in that space.
    pub localspace_z: Vector,
    pub begin_cell: CellRef,
    pub begin_pos: Option<Position>,
    pub end_pos: Position,
    pub curr_cell: CellRef,
    pub curr_pos: Position,
    pub global_offset: Vector,
    pub step_up: bool,
    pub step_up_normal: Vector,
    pub collide: bool,
    pub check_cell: CellRef,
    pub check_pos: Position,
    pub insert_type: InsertType,
    pub step_down: bool,
    pub backup: InsertType,
    pub backup_cell: CellRef,
    pub backup_check_pos: Position,
    pub obstruction_ethereal: bool,
    pub hits_interior_cell: bool,
    pub bldg_check: bool,
    pub walkable_allowance: f32,
    pub walk_interp: f32,
    pub step_down_amt: f32,
    /// The walkable physics-BSP polygon last stepped onto (`set_walkable`), with the sphere,
    /// up vector, position and scale of the space it is in.
    pub walkable_check_pos: Sphere,
    pub walkable: Option<Polygon>,
    pub check_walkable: bool,
    pub walkable_up: Vector,
    pub walkable_pos: Position,
    pub walkable_scale: Vector,
    pub cell_array_valid: bool,
    pub neg_step_up: bool,
    pub neg_collision_normal: Vector,
    pub neg_poly_hit: bool,
    pub placement_allows_sliding: bool,
    pub placement_ethereal: bool,
}

impl SpherePath {
    /// `SPHEREPATH::init`.
    pub(crate) fn init(&mut self) {
        self.num_sphere = 0;
        self.begin_cell = None;
        self.begin_pos = None;
        self.curr_cell = None;
        self.check_cell = None;
        self.insert_type = InsertType::Transition;
        self.step_down = false;
        self.step_up = false;
        self.collide = false;
        self.hits_interior_cell = false;
        self.bldg_check = false;
        self.obstruction_ethereal = false;
        self.backup_cell = None;
        self.walkable_allowance = 0.0;
        self.walkable = None;
        self.check_walkable = false;
        self.cell_array_valid = false;
        self.neg_step_up = false;
        self.neg_poly_hit = false;
        self.placement_allows_sliding = true;
        self.placement_ethereal = false;
    }

    /// `SPHEREPATH::init_sphere`: up to two spheres, scaled; the low point is the first sphere's
    /// bottom.
    pub(crate) fn init_sphere(&mut self, spheres: &[Sphere], scale: f32) {
        self.num_sphere = spheres.len().min(2);
        let s = ld(scale);
        for (local, sphere) in self.local_sphere.iter_mut().zip(spheres) {
            local.center = Vector::new(
                st(s * ld(sphere.center.x)),
                st(s * ld(sphere.center.y)),
                st(s * ld(sphere.center.z)),
            );
            local.radius = st(s * ld(sphere.radius));
        }
        let first = self.local_sphere[0];
        self.local_low_point = Vector::new(
            first.center.x,
            first.center.y,
            st(ld(first.center.z) - ld(first.radius)),
        );
    }

    /// `SPHEREPATH::init_path`: a transition from `begin` to `end`, or a placement at `end`
    /// when there is no begin position.
    pub(crate) fn init_path(
        &mut self,
        begin_cell: CellRef,
        begin: Option<Position>,
        end: Position,
    ) {
        self.begin_cell = begin_cell;
        self.begin_pos = begin;
        self.end_pos = end;
        self.curr_pos = begin.unwrap_or(end);
        self.curr_cell = begin_cell;
        self.cache_global_curr_center();
        self.insert_type = if begin.is_some() {
            InsertType::Transition
        } else {
            InsertType::Placement
        };
        self.placement_ethereal = begin.is_none();
    }

    /// `SPHEREPATH::cache_global_curr_center`: the local sphere centres in `curr_pos`.
    pub(crate) fn cache_global_curr_center(&mut self) {
        let m = self.curr_pos.frame.m.map(ld);
        let o = self.curr_pos.frame.origin;
        for (center, sphere) in self
            .global_curr_center
            .iter_mut()
            .zip(&self.local_sphere)
            .take(self.num_sphere)
        {
            let (x, y, z) = (
                ld(sphere.center.x),
                ld(sphere.center.y),
                ld(sphere.center.z),
            );
            *center = Vector::new(
                st(((x * m[0] + y * m[3]) + z * m[6]) + ld(o.x)),
                st(((m[7] * z + m[4] * y) + m[1] * x) + ld(o.y)),
                st(((m[5] * y + x * m[2]) + z * m[8]) + ld(o.z)),
            );
        }
    }

    /// `SPHEREPATH::cache_global_sphere`: with an offset the cached global spheres and low point
    /// move by it; without one they are recomputed from the local spheres in `check_pos`.
    pub(crate) fn cache_global_sphere(&mut self, offset: Option<Vector>) {
        if let Some(offset) = offset {
            for sphere in self.global_sphere.iter_mut().take(self.num_sphere) {
                sphere.center.x = st(ld(sphere.center.x) + ld(offset.x));
                sphere.center.y = st(ld(offset.y) + ld(sphere.center.y));
                sphere.center.z = st(ld(offset.z) + ld(sphere.center.z));
            }
            self.global_low_point.x = st(ld(offset.x) + ld(self.global_low_point.x));
            self.global_low_point.y = st(ld(offset.y) + ld(self.global_low_point.y));
            self.global_low_point.z = st(ld(offset.z) + ld(self.global_low_point.z));
            return;
        }
        let m = self.check_pos.frame.m.map(ld);
        let o = self.check_pos.frame.origin;
        for (global, local) in self
            .global_sphere
            .iter_mut()
            .zip(&self.local_sphere)
            .take(self.num_sphere)
        {
            global.radius = local.radius;
            let (x, y, z) = (ld(local.center.x), ld(local.center.y), ld(local.center.z));
            global.center = Vector::new(
                st(((m[6] * z + y * m[3]) + m[0] * x) + ld(o.x)),
                st(((m[1] * x + y * m[4]) + m[7] * z) + ld(o.y)),
                st(((m[8] * z + x * m[2]) + m[5] * y) + ld(o.z)),
            );
        }
        let low = self.local_low_point;
        let (x, y, z) = (ld(low.x), ld(low.y), ld(low.z));
        self.global_low_point = Vector::new(
            st(((m[6] * z + m[0] * x) + m[3] * y) + ld(o.x)),
            st(((m[7] * z + m[1] * x) + m[4] * y) + ld(o.y)),
            st(((m[8] * z + m[2] * x) + m[5] * y) + ld(o.z)),
        );
    }

    /// `SPHEREPATH::add_offset_to_check_pos`.
    pub(crate) fn add_offset_to_check_pos(&mut self, offset: Vector) {
        self.cell_array_valid = false;
        let origin = &mut self.check_pos.frame.origin;
        origin.x = st(ld(origin.x) + ld(offset.x));
        origin.y = st(ld(offset.y) + ld(origin.y));
        origin.z = st(ld(offset.z) + ld(origin.z));
        self.cache_global_sphere(Some(offset));
    }

    /// `SPHEREPATH::set_check_pos`.
    pub(crate) fn set_check_pos(&mut self, position: Position, cell: CellRef) {
        self.check_pos = position;
        self.check_cell = cell;
        self.cell_array_valid = false;
        self.cache_global_sphere(None);
    }

    /// `SPHEREPATH::save_check_pos`.
    pub(crate) fn save_check_pos(&mut self) {
        self.backup_cell = self.check_cell;
        self.backup_check_pos = self.check_pos;
    }

    /// `SPHEREPATH::restore_check_pos`.
    pub(crate) fn restore_check_pos(&mut self) {
        self.set_check_pos(self.backup_check_pos, self.backup_cell);
    }

    /// `SPHEREPATH::adjust_check_pos`: rebases `check_pos` onto `cell_id`'s landblock.
    pub(crate) fn adjust_check_pos(&mut self, cell_id: u32) {
        if is_outside(cell_id) {
            let offset = get_block_offset(cell_id, self.check_pos.cell_id);
            self.cache_global_sphere(Some(offset));
            let origin = &mut self.check_pos.frame.origin;
            origin.x = st(ld(offset.x) + ld(origin.x));
            origin.y = st(ld(offset.y) + ld(origin.y));
            origin.z = st(ld(offset.z) + ld(origin.z));
        }
        self.check_pos.cell_id = cell_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;
    use crate::vector::Quaternion;

    fn path_at(origin: Vector) -> SpherePath {
        let mut path = SpherePath::default();
        path.init();
        path.init_sphere(&[Sphere::new(Vector::new(0.0, 0.0, 0.5), 0.5)], 1.0);
        let position = Position::new(0x7F7F_0001, Frame::new(origin, Quaternion::IDENTITY));
        path.init_path(Some(position.cell_id), Some(position), position);
        path.set_check_pos(position, Some(position.cell_id));
        path
    }

    #[test]
    fn global_spheres_follow_the_check_position() {
        let mut path = path_at(Vector::new(10.0, 20.0, 5.0));
        assert_eq!(path.global_sphere[0].center, Vector::new(10.0, 20.0, 5.5));
        assert_eq!(path.global_low_point, Vector::new(10.0, 20.0, 5.0));
        assert_eq!(path.global_curr_center[0], Vector::new(10.0, 20.0, 5.5));
        path.add_offset_to_check_pos(Vector::new(1.0, 0.0, -0.25));
        assert_eq!(path.check_pos.frame.origin, Vector::new(11.0, 20.0, 4.75));
        assert_eq!(path.global_low_point, Vector::new(11.0, 20.0, 4.75));
        path.save_check_pos();
        path.add_offset_to_check_pos(Vector::new(0.0, 3.0, 0.0));
        path.restore_check_pos();
        assert_eq!(path.global_sphere[0].center, Vector::new(11.0, 20.0, 5.25));
    }

    #[test]
    fn adjust_check_pos_rebases_across_blocks() {
        let mut path = path_at(Vector::new(161.0, 20.0, 5.0));
        path.adjust_check_pos(0x807F_0001);
        assert_eq!(path.check_pos.cell_id, 0x807F_0001);
        assert_eq!(path.check_pos.frame.origin.x, 1.0);
        assert_eq!(path.global_sphere[0].center.x, 1.0);
    }
}
