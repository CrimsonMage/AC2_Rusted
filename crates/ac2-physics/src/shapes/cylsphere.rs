//! Retail `CCylSphere` collision: an upright cylinder (a Setup's `Cylsphere`: low point, height,
//! radius) against the moving body's global spheres. Every expression follows the x87 order of
//! its function; `disp` is the body sphere's centre minus the cylinder's low point.

use crate::cells::CellSource;
use crate::fpu::{is_float_zero, ld, st, F_EPSILON, LANDING_Z};
use crate::landdefs::get_block_offset;
use crate::position::Position;
use crate::shapes::sphere::slide_sphere;
use crate::transition::collision::{CollisionInfo, ObjectInfo, PATH_CLIPPED_OI, PERFECT_CLIP_OI};
use crate::transition::sphere_path::{InsertType, SpherePath};
use crate::transition::{Transition, TransitionState};
use crate::vector::{Plane, Sphere, Vector};

/// `CCylSphere`: an upright cylinder standing on `low`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CylSphere {
    pub low: Vector,
    pub height: f32,
    pub radius: f32,
}

/// `a - b` per component, each stored.
fn sub(a: Vector, b: Vector) -> Vector {
    Vector::new(
        st(ld(a.x) - ld(b.x)),
        st(ld(a.y) - ld(b.y)),
        st(ld(a.z) - ld(b.z)),
    )
}

/// Normalised with the squared length summed `(z^2 + y^2) + x^2`.
fn normalize_zyx(n: Vector) -> Option<Vector> {
    let (x, y, z) = (ld(n.x), ld(n.y), ld(n.z));
    let len = ((z * z + y * y) + x * x).sqrt();
    if len < ld(F_EPSILON) {
        return None;
    }
    let inv = 1.0 / len;
    Some(Vector::new(st(x * inv), st(y * inv), st(z * inv)))
}

impl CylSphere {
    /// `CCylSphere::collides_with_sphere`: within `radsum` horizontally and the sphere (less
    /// `F_EPSILON`) overlapping the cylinder's height.
    pub(crate) fn collides(&self, check: &Sphere, disp: Vector, radsum: f32) -> bool {
        let (x, y) = (ld(disp.x), ld(disp.y));
        if ld(radsum) * ld(radsum) < x * x + y * y {
            return false;
        }
        let half = ld(self.height) * 0.5;
        (half - ld(disp.z)).abs() <= (ld(check.radius) - ld(F_EPSILON)) + half
    }

    /// `CCylSphere::normal_of_collision`: the side normal when the body was outside the
    /// cylinder's radius last step, else the cap it moves into. `false` when the side normal
    /// does not hold: the body was clear of the height and moved vertically.
    fn normal_of_collision(
        &self,
        path: &SpherePath,
        check: &Sphere,
        disp: Vector,
        radsum: f32,
        sphere_num: usize,
    ) -> (bool, Vector) {
        let curr = path.global_curr_center[sphere_num];
        let dx = ld(curr.x) - ld(self.low.x);
        let dy = ld(curr.y) - ld(self.low.y);
        let dz = ld(st(ld(curr.z) - ld(self.low.z)));
        if ld(radsum) * ld(radsum) < dy * dy + dx * dx {
            let normal = Vector::new(st(dx), st(dy), 0.0);
            let half = ld(self.height) * 0.5;
            let reach = (ld(check.radius) - ld(F_EPSILON)) + half;
            let side = !(reach < (half - dz).abs()) || !(ld(F_EPSILON) < (dz - ld(disp.z)).abs());
            return (side, normal);
        }
        let z = if ld(disp.z) - dz <= 0.0 { 1.0 } else { -1.0 };
        (true, Vector::new(0.0, 0.0, z))
    }

    /// `CCylSphere::intersects_sphere(Position*, Vector*, CTransition*)`: a Setup cylinder of a
    /// static object at `p`, scaled, in the check position's landblock (the local spheres are
    /// cached in `p`'s space for the step-up normal).
    pub(crate) fn intersects_object(
        &self,
        transition: &mut Transition,
        p: &Position,
        scale: Vector,
        cells: &dyn CellSource,
    ) -> TransitionState {
        transition
            .sphere_path
            .cache_localspace_sphere(p, Vector::new(1.0, 1.0, 1.0));
        let radius = st(((ld(scale.y) + ld(scale.x)) * 0.5) * ld(self.radius));
        let height = st(ld(self.height) * ld(scale.z));
        let low = Vector::new(
            st(ld(self.low.x) * ld(scale.x)),
            st(ld(self.low.y) * ld(scale.y)),
            st(ld(self.low.z) * ld(scale.z)),
        );
        let low = transition.sphere_path.check_pos.local_to_global(p, low);
        let global = CylSphere {
            low,
            height,
            radius,
        };
        global.intersects_sphere(transition, cells)
    }

    /// `CCylSphere::intersects_sphere(CTransition*)`.
    fn intersects_sphere(
        &self,
        transition: &mut Transition,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let path = &transition.sphere_path;
        let s0 = path.global_sphere[0];
        let s1 = path.global_sphere[1];
        let two = path.num_sphere >= 2;
        let disp = sub(s0.center, self.low);
        let disp2 = sub(s1.center, self.low);
        let rs = st((ld(self.radius) - ld(F_EPSILON)) + ld(s0.radius));
        let second = || two && self.collides(&s1, disp2, rs);
        if path.insert_type == InsertType::Placement || path.obstruction_ethereal {
            if self.collides(&s0, disp, rs) || second() {
                return TransitionState::Collided;
            }
            return TransitionState::Ok;
        }
        if path.step_down {
            return self.step_sphere_down(
                &mut transition.sphere_path,
                &mut transition.collision_info,
                s0,
                disp,
                rs,
            );
        }
        if path.check_walkable {
            if self.collides(&s0, disp, rs) || second() {
                return TransitionState::Collided;
            }
            return TransitionState::Ok;
        }
        if path.collide {
            if !self.collides(&s0, disp, rs) && !second() {
                return TransitionState::Ok;
            }
            return self.land_on_top(transition, s0, disp, rs);
        }
        let object = transition.object_info;
        if object.state & 3 != 0 {
            if self.collides(&s0, disp, rs) {
                return self.step_sphere_up(transition, s0, disp, rs, cells);
            }
            if !second() {
                return TransitionState::Ok;
            }
            let (path, collisions) = (&mut transition.sphere_path, &mut transition.collision_info);
            return self.slide_sphere(path, collisions, s1, disp2, rs, 1);
        }
        let (path, collisions) = (&mut transition.sphere_path, &mut transition.collision_info);
        if object.state & PATH_CLIPPED_OI != 0 {
            if !self.collides(&s0, disp, rs) {
                return TransitionState::Ok;
            }
            return self.collide_with_point(&object, path, collisions, s0, disp, rs, 0);
        }
        if self.collides(&s0, disp, rs) {
            return self.land_on_cylinder(path, s0, disp, rs);
        }
        if !second() {
            return TransitionState::Ok;
        }
        self.collide_with_point(&object, path, collisions, s1, disp2, rs, 1)
    }

    /// The `collide` branch of `intersects_sphere`: the placement pass after a collision puts
    /// the body on the cylinder's top, along its last step.
    fn land_on_top(
        &self,
        transition: &mut Transition,
        s0: Sphere,
        disp: Vector,
        rs: f32,
    ) -> TransitionState {
        let path = &mut transition.sphere_path;
        let v = sub(path.global_curr_center[0], s0.center);
        let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
        let w = sub(v, block);
        if is_float_zero(w.z, F_EPSILON) {
            return TransitionState::Collided;
        }
        let t = ((ld(self.height) + ld(s0.radius)) - ld(disp.z)) / ld(w.z);
        let o = Vector::new(st(ld(w.x) * t), st(ld(w.y) * t), st(t * ld(w.z)));
        let ux = ld(o.x) + ld(disp.x);
        let uy = ld(o.y) + ld(disp.y);
        if ld(rs) * ld(rs) < uy * uy + ux * ux {
            return TransitionState::Ok;
        }
        let interp = (1.0 - t) * ld(path.walk_interp);
        if !(interp < ld(path.walk_interp)) || interp < -0.1 {
            return TransitionState::Collided;
        }
        let contact = Vector::new(
            st(ld(o.x) + ld(s0.center.x)),
            st(ld(o.y) + ld(s0.center.y)),
            st((ld(o.z) + ld(s0.center.z)) - ld(s0.radius)),
        );
        let plane = Plane::from_point(Vector::new(0.0, 0.0, 1.0), contact);
        transition
            .collision_info
            .set_contact_plane(plane, false, false, false);
        transition.collision_info.contact_plane_cell_id = transition.sphere_path.check_pos.cell_id;
        transition.sphere_path.walk_interp = st(interp);
        transition.sphere_path.add_offset_to_check_pos(o);
        TransitionState::Adjusted
    }

    /// `CCylSphere::step_sphere_down`: stepping down onto the cylinder's top.
    fn step_sphere_down(
        &self,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        check: Sphere,
        disp: Vector,
        rs: f32,
    ) -> TransitionState {
        if !self.collides(&check, disp, rs) {
            let two = path.num_sphere >= 2;
            let s1 = path.global_sphere[1];
            if !two || !self.collides(&s1, sub(s1.center, self.low), rs) {
                return TransitionState::Ok;
            }
        }
        let amount = st(ld(path.step_down_amt) * ld(path.walk_interp));
        if ld(amount).abs() < ld(F_EPSILON) {
            return TransitionState::Collided;
        }
        let rise = (ld(self.height) + ld(check.radius)) - ld(disp.z);
        let interp = (1.0 - rise / ld(amount)) * ld(path.walk_interp);
        if !(interp < ld(path.walk_interp)) || interp < -0.1 {
            return TransitionState::Collided;
        }
        let c = check.center;
        let d = st((ld(c.x) + ld(c.y)) * 0.0 - ((rise - ld(check.radius)) + ld(c.z)));
        collisions.set_contact_plane(
            Plane::new(Vector::new(0.0, 0.0, 1.0), d),
            false,
            false,
            false,
        );
        collisions.contact_plane_cell_id = path.check_pos.cell_id;
        path.walk_interp = st(interp);
        path.add_offset_to_check_pos(Vector::new(0.0, 0.0, st(rise)));
        TransitionState::Adjusted
    }

    /// `CCylSphere::step_sphere_up`: a cylinder lower than the step-up height is stepped onto
    /// (the normal goes through `localspace_pos` first, as retail does); a taller one is slid
    /// along.
    fn step_sphere_up(
        &self,
        transition: &mut Transition,
        check: Sphere,
        disp: Vector,
        rs: f32,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let rise = (ld(check.radius) + ld(self.height)) - ld(disp.z);
        if ld(transition.object_info.step_up_height) < rise {
            return self.slide_sphere(
                &mut transition.sphere_path,
                &mut transition.collision_info,
                check,
                disp,
                rs,
                0,
            );
        }
        let (_, normal) = self.normal_of_collision(&transition.sphere_path, &check, disp, rs, 0);
        let Some(normal) = normalize_zyx(normal) else {
            return TransitionState::Collided;
        };
        let global = transition
            .sphere_path
            .localspace_pos
            .local_to_global_vec(normal);
        if transition.step_up(global, cells) {
            return TransitionState::Ok;
        }
        transition
            .sphere_path
            .step_up_slide(&mut transition.collision_info)
    }

    /// `CCylSphere::slide_sphere`: `CSphere::slide_sphere` along the collision normal.
    fn slide_sphere(
        &self,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        check: Sphere,
        disp: Vector,
        rs: f32,
        sphere_num: usize,
    ) -> TransitionState {
        let (_, normal) = self.normal_of_collision(path, &check, disp, rs, sphere_num);
        let Some(mut normal) = normalize_zyx(normal) else {
            return TransitionState::Collided;
        };
        let curr = path.global_curr_center[sphere_num];
        slide_sphere(check, path, collisions, &mut normal, curr)
    }

    /// `CCylSphere::land_on_cylinder`: collide, then try a placement on top.
    fn land_on_cylinder(
        &self,
        path: &mut SpherePath,
        check: Sphere,
        disp: Vector,
        rs: f32,
    ) -> TransitionState {
        let (_, normal) = self.normal_of_collision(path, &check, disp, rs, 0);
        let Some(normal) = normalize_zyx(normal) else {
            return TransitionState::Collided;
        };
        path.set_collide(normal);
        path.walkable_allowance = LANDING_Z;
        TransitionState::Adjusted
    }

    /// `CCylSphere::collide_with_point`: a perfect-clip body is moved back along its step to the
    /// cylinder's surface (cap or side); any other body records the normal and collides.
    #[allow(clippy::too_many_arguments)]
    fn collide_with_point(
        &self,
        object: &ObjectInfo,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        check: Sphere,
        disp: Vector,
        rs: f32,
        sphere_num: usize,
    ) -> TransitionState {
        let (side, normal) = self.normal_of_collision(path, &check, disp, rs, sphere_num);
        let (x, y, z) = (ld(normal.x), ld(normal.y), ld(normal.z));
        let len = ((y * y + x * x) + z * z).sqrt();
        if len < ld(F_EPSILON) {
            return TransitionState::Collided;
        }
        let inv = 1.0 / len;
        let mut normal = Vector::new(st(x * inv), st(y * inv), st(inv * z));
        if object.state & PERFECT_CLIP_OI == 0 {
            collisions.set_collision_normal(normal);
            return TransitionState::Collided;
        }
        let curr = path.global_curr_center[0];
        let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
        let c = check.center;
        let m = Vector::new(
            st((ld(c.x) - ld(curr.x)) + ld(block.x)),
            st(ld(block.y) + (ld(c.y) - ld(curr.y))),
            st(ld(block.z) + ld(st(ld(c.z) - ld(curr.z)))),
        );
        let (mx, my, mz) = (ld(m.x), ld(m.y), ld(m.z));
        let qx = ld(st(ld(curr.x) - ld(self.low.x)));
        let qy = ld(st(ld(curr.y) - ld(self.low.y)));
        let qz = ld(curr.z) - ld(self.low.z);
        let rs2 = st(ld(F_EPSILON) + ld(rs));
        let cap = |qz: f64| {
            if !(mz <= 0.0) {
                -((qz + ld(check.radius)) / mz)
            } else {
                ((ld(check.radius) + ld(self.height)) - qz) / mz
            }
        };
        let along = |t: f64| Vector::new(st(mx * t), st(t * my), st(t * mz));
        let side_normal = |offset: Vector| {
            let nx = ld(st((ld(offset.x) + ld(curr.x)) - ld(self.low.x)));
            let ny = (ld(offset.y) + ld(curr.y)) - ld(self.low.y);
            let inv = 1.0 / ld(rs2);
            Vector::new(st(nx * inv), st(ny * inv), st(inv * 0.0))
        };
        let circle = |rr: f64| {
            let a = my * my + mx * mx;
            let b = -(qy * my + qx * mx);
            let disc = b * b - ((qy * qy + qx * qx) - rr) * a;
            (a, b, disc)
        };
        let root = |a: f64, b: f64, disc: f64| {
            let s = disc.sqrt();
            if b - s < 0.0 {
                (s + b) / a
            } else {
                (b - s) / a
            }
        };
        let (t, offset);
        if !side {
            normal = Vector::new(0.0, 0.0, if !(mz <= 0.0) { -1.0 } else { 1.0 });
            let mut tt = cap(qz);
            let mut o = along(tt);
            let ux = ld(o.x) + qx;
            let uy = ld(o.y) + qy;
            let rr = ld(st(ld(rs2) * ld(rs2)));
            if !(uy * uy + ux * ux < rr) {
                let (a, b, disc) = circle(rr);
                if disc >= 0.0 && a > ld(F_EPSILON) {
                    tt = root(a, b, disc);
                    o = along(tt);
                }
                normal = side_normal(o);
            }
            (t, offset) = (tt, o);
        } else if normal.z == 0.0 {
            let (a, b, disc) = circle(ld(rs2) * ld(rs2));
            if disc < 0.0 || !(a > ld(F_EPSILON)) {
                return TransitionState::Collided;
            }
            let tt = root(a, b, disc);
            let o = Vector::new(st(mx * tt), st(my * tt), st(tt * mz));
            if tt < 0.0 || tt > 1.0 {
                return TransitionState::Collided;
            }
            normal = side_normal(o);
            (t, offset) = (tt, o);
        } else {
            let tt = cap(qz);
            (t, offset) = (tt, along(tt));
        }
        if t < 0.0 || t > 1.0 {
            return TransitionState::Collided;
        }
        let y = st(ld(offset.y) + ld(curr.y));
        let z = st(ld(offset.z) + ld(curr.z));
        let offset = Vector::new(
            st((ld(offset.x) + ld(curr.x)) - ld(c.x)),
            st(ld(y) - ld(c.y)),
            st(ld(z) - ld(c.z)),
        );
        collisions.set_collision_normal(normal);
        path.add_offset_to_check_pos(offset);
        TransitionState::Adjusted
    }
}
