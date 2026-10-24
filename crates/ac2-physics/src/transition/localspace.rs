//! The `SPHEREPATH` state the physics BSPs and Setup shapes write: the spheres cached in an
//! object's space (`cache_localspace_sphere`), the collide / walkable / negative-polygon
//! records, and the slides that resolve them (`step_up_slide`, `precipice_slide`).

use crate::bsp::Polygon;
use crate::fpu::{ld, st};
use crate::landdefs::get_block_offset;
use crate::position::Position;
use crate::shapes::sphere::slide_sphere;
use crate::transition::collision::CollisionInfo;
use crate::transition::sphere_path::SpherePath;
use crate::transition::TransitionState;
use crate::vector::{Sphere, Vector};

impl SpherePath {
    /// `SPHEREPATH::cache_localspace_sphere`: the local spheres at the check and current
    /// positions moved into `pos`'s frame and scaled per axis (radius by `scale.z`), world up in
    /// that frame and the low point under the first sphere.
    pub(crate) fn cache_localspace_sphere(&mut self, pos: &Position, scale: Vector) {
        let (sx, sy, sz) = (ld(scale.x), ld(scale.y), ld(scale.z));
        for i in 0..self.num_sphere {
            let local = self.local_sphere[i];
            let c = pos.local_to_local(&self.check_pos, local.center);
            self.localspace_sphere[i] = Sphere::new(
                Vector::new(st(ld(c.x) * sx), st(ld(c.y) * sy), st(ld(c.z) * sz)),
                st(ld(local.radius) * sz),
            );
            let c = pos.local_to_local(&self.curr_pos, local.center);
            self.localspace_curr_center[i] =
                Vector::new(st(sx * ld(c.x)), st(sy * ld(c.y)), st(sz * ld(c.z)));
        }
        self.localspace_pos = *pos;
        let m = pos.frame.m.map(ld);
        let z = Vector::new(
            st((m[1] + m[0]) * 0.0 + m[2]),
            st((m[4] + m[3]) * 0.0 + m[5]),
            st((m[7] + m[6]) * 0.0 + m[8]),
        );
        self.localspace_z = z;
        let first = self.localspace_sphere[0];
        let r = ld(first.radius);
        let tx = ld(st(r * ld(z.x)));
        let ty = ld(st(r * ld(z.y)));
        let tz = r * ld(z.z);
        self.localspace_low_point = Vector::new(
            st(ld(first.center.x) - tx),
            st(ld(first.center.y) - ty),
            st(ld(first.center.z) - tz),
        );
    }

    /// `SPHEREPATH::set_collide`: remember the check position and the normal to step up along;
    /// the next insert is a placement on top of what was hit.
    pub(crate) fn set_collide(&mut self, normal: Vector) {
        self.collide = true;
        self.backup_cell = self.check_cell;
        self.backup_check_pos = self.check_pos;
        self.step_up_normal = normal;
        self.walk_interp = 1.0;
    }

    /// `SPHEREPATH::set_walkable`: the walkable polygon stepped onto, in the space of its mesh.
    pub(crate) fn set_walkable(
        &mut self,
        sphere: Sphere,
        polygon: &Polygon,
        up: Vector,
        local_pos: Position,
        scale: Vector,
    ) {
        self.walkable_check_pos = sphere;
        self.walkable = Some(polygon.clone());
        self.walkable_up = up;
        self.walkable_pos = local_pos;
        self.walkable_scale = scale;
    }

    /// `SPHEREPATH::set_neg_poly_hit`: a polygon hit from behind; the insertion resolves it with a
    /// step up or a slide along the reversed normal.
    pub(crate) fn set_neg_poly_hit(&mut self, step_up: bool, normal: Vector) {
        self.neg_step_up = step_up;
        self.neg_poly_hit = true;
        self.neg_collision_normal = Vector::new(-normal.x, -normal.y, -normal.z);
    }

    /// `SPHEREPATH::check_walkables`: no walkable polygon, or the half-radius walkable check
    /// sphere is still over it (`CPolygon::check_walkable`).
    pub(crate) fn check_walkables(&mut self) -> bool {
        let Some(polygon) = &self.walkable else {
            return true;
        };
        self.walkable_check_pos.radius = st(ld(self.walkable_check_pos.radius) * 0.5);
        polygon.check_walkable(&self.walkable_check_pos, self.walkable_up, false)
    }

    /// `SPHEREPATH::step_up_slide`: a failed step up slides the first sphere along the step-up
    /// normal (`CSphere::slide_sphere`).
    pub(crate) fn step_up_slide(&mut self, collisions: &mut CollisionInfo) -> TransitionState {
        collisions.clear_contact_plane();
        self.step_up = false;
        let mut normal = self.step_up_normal;
        let (sphere, curr) = (self.global_sphere[0], self.global_curr_center[0]);
        let ts = slide_sphere(sphere, self, collisions, &mut normal, curr);
        self.step_up_normal = normal;
        ts
    }

    /// `SPHEREPATH::precipice_slide`: at the edge of the walkable polygon, slide along the edge
    /// the low point crossed (its normal in world axes, turned to face back along the step).
    pub(crate) fn precipice_slide(&mut self, collisions: &mut CollisionInfo) -> TransitionState {
        let crossed = self.walkable.as_ref().and_then(|polygon| {
            polygon.find_crossed_edge(&self.walkable_check_pos, self.walkable_up)
        });
        self.walkable = None;
        let Some(edge) = crossed else {
            return TransitionState::Collided;
        };
        self.step_up = false;
        let g = self.walkable_pos.local_to_global_vec(edge);
        let block = get_block_offset(self.curr_pos.cell_id, self.check_pos.cell_id);
        let (sphere, curr) = (self.global_sphere[0], self.global_curr_center[0]);
        let a = ld(sphere.center.x) - ld(curr.x);
        let b = ld(sphere.center.y) - ld(curr.y);
        let c = ld(st(ld(sphere.center.z) - ld(curr.z)));
        let mx = ld(st(a + ld(block.x)));
        let my = ld(block.y) + b;
        let mz = ld(block.z) + c;
        let dot = (ld(g.z) * mz + my * ld(g.y)) + mx * ld(g.x);
        let mut normal = if dot > 0.0 {
            Vector::new(st(ld(g.x) * -1.0), st(ld(g.y) * -1.0), st(ld(g.z) * -1.0))
        } else {
            g
        };
        slide_sphere(sphere, self, collisions, &mut normal, curr)
    }
}
