//! The `CTransition` responses inside one step: `adjust_offset` (slide the step along the
//! contact and sliding planes), `step_down`, `check_walkable`, `edge_slide` and `cliff_slide`.

use crate::cells::CellSource;
use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::get_block_offset;
use crate::transition::collision::{EDGE_SLIDE_OI, ON_WALKABLE_OI};
use crate::transition::sphere_path::InsertType;
use crate::transition::{Transition, TransitionState};
use crate::vector::{Plane, Vector};

impl Transition {
    /// `CTransition::adjust_offset`: a step into the sliding normal follows the crease of the
    /// contact and sliding planes; a step into the contact plane loses its normal component, a
    /// step away from it is snapped onto it; a sphere sunk into the contact plane is lifted out.
    pub(crate) fn adjust_offset(&mut self, step: Vector) -> Vector {
        let (mut x, mut y, mut z) = (step.x, step.y, step.z);
        let mut sliding = false;
        if self.collision_info.sliding_normal_valid {
            let s = self.collision_info.sliding_normal;
            let into = (ld(z) * ld(s.z) + ld(y) * ld(s.y)) + ld(x) * ld(s.x);
            if into < 0.0 {
                sliding = true;
            } else {
                self.collision_info.sliding_normal_valid = false;
            }
        }
        let c = self.collision_info;
        if !c.contact_plane_valid {
            if sliding {
                let s = c.sliding_normal;
                let into = (ld(z) * ld(s.z) + ld(y) * ld(s.y)) + ld(x) * ld(s.x);
                let px = ld(st(into * ld(s.x)));
                let py = ld(st(into * ld(s.y)));
                let pz = into * ld(s.z);
                x = st(ld(x) - px);
                y = st(ld(y) - py);
                z = st(ld(z) - pz);
            }
            return Vector::new(x, y, z);
        }
        let n = c.contact_plane.n;
        let into_contact = st((ld(z) * ld(n.z) + ld(y) * ld(n.y)) + ld(x) * ld(n.x));
        if sliding {
            let s = c.sliding_normal;
            let cx = ld(n.y) * ld(s.z) - ld(n.z) * ld(s.y);
            let cy = ld(st(ld(n.z) * ld(s.x) - ld(n.x) * ld(s.z)));
            let cz = ld(n.x) * ld(s.y) - ld(n.y) * ld(s.x);
            let length = ((cz * cz + cy * cy) + cx * cx).sqrt();
            if length < ld(F_EPSILON) {
                (x, y, z) = (0.0, 0.0, 0.0);
            } else {
                let inverse = 1.0 / length;
                let ux = ld(st(cx * inverse));
                let uy = ld(st(cy * inverse));
                let uz = cz * inverse;
                let along = (ld(z) * uz + uy * ld(y)) + ux * ld(x);
                (x, y, z) = (st(ux * along), st(uy * along), st(uz * along));
            }
        } else if ld(into_contact) <= 0.0 {
            let d = ld(into_contact);
            let px = d * ld(n.x);
            let py = ld(st(d * ld(n.y)));
            let pz = ld(st(d * ld(n.z)));
            x = st(ld(x) - px);
            y = st(ld(y) - py);
            z = st(ld(z) - pz);
        } else {
            let mut offset = Vector::new(x, y, z);
            c.contact_plane.snap_to_plane(&mut offset);
            (x, y, z) = (offset.x, offset.y, offset.z);
        }
        if !c.contact_plane_is_water && c.contact_plane_cell_id != 0 {
            let plane = c.contact_plane;
            let path = &mut self.sphere_path;
            let block = get_block_offset(path.check_pos.cell_id, c.contact_plane_cell_id);
            let sphere = path.global_sphere[0];
            let dist = ((ld(sphere.center.z) - ld(block.z)) * ld(plane.n.z)
                + (ld(sphere.center.y) - ld(block.y)) * ld(plane.n.y))
                + (ld(sphere.center.x) - ld(block.x)) * ld(plane.n.x)
                + ld(plane.d);
            let radius = ld(sphere.radius);
            if dist < radius - ld(F_EPSILON) {
                let lift = (radius - dist) / ld(plane.n.z);
                if lift.abs() < radius {
                    path.add_offset_to_check_pos(Vector::new(0.0, 0.0, st(lift)));
                }
            }
        }
        Vector::new(x, y, z)
    }

    /// `CTransition::check_walkable`: without a recorded walkable polygon (the terrain records
    /// none) the position is accepted.
    pub(crate) fn check_walkable(&mut self, _z_check: f32, _cells: &dyn CellSource) -> bool {
        self.object_info.state & ON_WALKABLE_OI == 0 || self.sphere_path.check_walkables()
    }

    /// `CTransition::step_down`: lower the check position by `height` (unless stepping up) and
    /// insert; it holds when the insert lands on a contact plane at least `z_val` upright and a
    /// placement insert there succeeds.
    pub(crate) fn step_down(&mut self, height: f32, z_val: f32, cells: &dyn CellSource) -> bool {
        let path = &mut self.sphere_path;
        path.neg_poly_hit = false;
        path.step_down = true;
        path.step_down_amt = height;
        path.walk_interp = 1.0;
        if !path.step_up {
            path.cell_array_valid = false;
            let drop = st(-ld(height));
            path.check_pos.frame.origin.z = st(ld(drop) + ld(path.check_pos.frame.origin.z));
            path.cache_global_sphere(Some(Vector::new(0.0, 0.0, drop)));
        }
        let ts = self.transitional_insert(5, cells);
        self.sphere_path.step_down = false;
        if ts != TransitionState::Ok
            || !self.collision_info.contact_plane_valid
            || ld(self.collision_info.contact_plane.n.z) < ld(z_val)
        {
            return false;
        }
        if self.object_info.state & EDGE_SLIDE_OI != 0
            && !self.sphere_path.step_up
            && !self.check_walkable(z_val, cells)
        {
            return false;
        }
        self.sphere_path.backup = self.sphere_path.insert_type;
        self.sphere_path.insert_type = InsertType::Placement;
        let placed = self.transitional_insert(1, cells);
        self.sphere_path.insert_type = self.sphere_path.backup;
        placed == TransitionState::Ok
    }

    /// `CTransition::edge_slide`: a walker that cannot step down keeps to the edge. Without
    /// edge sliding it walks off; on a too-steep contact it slides along the cliff
    /// (`cliff_slide`); with no ground at all it stays where it was. Returns whether the
    /// insertion is finished and the state it leaves.
    pub(crate) fn edge_slide(
        &mut self,
        height: f32,
        z_val: f32,
        cells: &dyn CellSource,
    ) -> (bool, TransitionState) {
        let state = self.object_info.state;
        if state & ON_WALKABLE_OI == 0 || state & EDGE_SLIDE_OI == 0 {
            self.sphere_path.walkable = false;
            let (backup, cell) = (
                self.sphere_path.backup_check_pos,
                self.sphere_path.backup_cell,
            );
            self.sphere_path.set_check_pos(backup, cell);
            self.collision_info.clear_contact_plane();
            self.sphere_path.cell_array_valid = true;
            return (true, TransitionState::Ok);
        }
        let contact_valid = self.collision_info.contact_plane_valid;
        if contact_valid && ld(self.collision_info.contact_plane.n.z) < ld(z_val) {
            self.sphere_path.walkable = false;
            self.sphere_path.restore_check_pos();
            let plane = self.collision_info.contact_plane;
            let ts = self.cliff_slide(plane);
            self.collision_info.clear_contact_plane();
            return (false, ts);
        }
        if contact_valid {
            self.sphere_path.walkable = false;
            self.sphere_path.restore_check_pos();
            self.sphere_path.cell_array_valid = true;
            self.collision_info.clear_contact_plane();
            return (true, TransitionState::Ok);
        }
        // `SPHEREPATH::walkable` would select `precipice_slide`; terrain never records one.
        let back = self
            .sphere_path
            .check_pos
            .offset_to(&self.sphere_path.curr_pos);
        self.sphere_path.add_offset_to_check_pos(back);
        self.step_down(height, z_val, cells);
        self.collision_info.clear_contact_plane();
        self.sphere_path.restore_check_pos();
        self.sphere_path.walkable = false;
        self.sphere_path.cell_array_valid = true;
        (true, TransitionState::Collided)
    }

    /// `CTransition::cliff_slide`: slide along the crease between the too-steep contact plane
    /// and the last known one, pushing the check position back to the crease.
    fn cliff_slide(&mut self, contact: Plane) -> TransitionState {
        let p = contact.n;
        let l = self.collision_info.last_known_contact_plane.n;
        let a = ld(p.y) * ld(l.z) - ld(p.z) * ld(l.y);
        let b = ld(l.x) * ld(p.z) - ld(l.z) * ld(p.x);
        let c = (ld(p.x) * ld(l.y) - ld(p.y) * ld(l.x)) * 0.0;
        let vx = st(c - b);
        let vy = st(a - c);
        let vz = b * 0.0 - a * 0.0;
        let length = ((vz * vz + ld(vy) * ld(vy)) + ld(vx) * ld(vx)).sqrt();
        if length < ld(F_EPSILON) {
            return TransitionState::Ok;
        }
        let inverse = 1.0 / length;
        let normal = Vector::new(st(ld(vx) * inverse), st(ld(vy) * inverse), st(vz * inverse));
        let path = &self.sphere_path;
        let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
        let sphere = path.global_sphere[0].center;
        let curr = path.global_curr_center[0];
        let dx = ld(sphere.x) - ld(curr.x);
        let dy = ld(sphere.y) - ld(curr.y);
        let dz = ld(st(ld(sphere.z) - ld(curr.z)));
        let mx = ld(st(ld(block.x) + dx));
        let my = ld(block.y) + dy;
        let dot = ((ld(block.z) + dz) * ld(normal.z) + my * ld(normal.y)) + mx * ld(normal.x);
        let (scale, collision_normal) = if dot > 0.0 {
            let back = -dot;
            (
                back,
                Vector::new(st(-ld(normal.x)), st(-ld(normal.y)), st(-ld(normal.z))),
            )
        } else {
            (dot, normal)
        };
        let offset = Vector::new(
            st(ld(normal.x) * scale),
            st(ld(normal.y) * scale),
            st(ld(normal.z) * scale),
        );
        self.sphere_path.add_offset_to_check_pos(offset);
        self.collision_info.set_collision_normal(collision_normal);
        TransitionState::Adjusted
    }
}
