//! Retail `CSphere` collision: a static object's sphere (in the check position's landblock) against
//! the moving body's global spheres, and `CSphere::slide_sphere`, the slide every shape shares.
//! Every expression follows the x87 order of its function; `disp` is the body sphere's centre
//! minus the obstacle's.

use crate::cells::CellSource;
use crate::fpu::{ld, st, F_EPSILON, LANDING_Z};
use crate::landdefs::get_block_offset;
use crate::position::Position;
use crate::transition::collision::{CollisionInfo, ObjectInfo, PATH_CLIPPED_OI, PERFECT_CLIP_OI};
use crate::transition::sphere_path::{InsertType, SpherePath};
use crate::transition::{Transition, TransitionState};
use crate::vector::{Plane, Sphere, Vector};

/// `a - b` per component, each stored.
fn sub(a: Vector, b: Vector) -> Vector {
    Vector::new(
        st(ld(a.x) - ld(b.x)),
        st(ld(a.y) - ld(b.y)),
        st(ld(a.z) - ld(b.z)),
    )
}

/// `CSphere::collides_with_sphere`: `(x^2 + y^2) + z^2 <= radsum^2`.
pub(crate) fn collides(disp: Vector, radsum: f32) -> bool {
    let (x, y, z) = (ld(disp.x), ld(disp.y), ld(disp.z));
    (x * x + y * y) + z * z <= ld(radsum) * ld(radsum)
}

/// `CSphere::find_time_of_collision`: the fraction of `movement` at which a sphere displaced by
/// `disp` comes within `radsum`; `-1` when it does not.
pub(crate) fn find_time_of_collision(movement: Vector, disp: Vector, radsum: f32) -> f64 {
    let (mx, my, mz) = (ld(movement.x), ld(movement.y), ld(movement.z));
    let (dx, dy, dz) = (ld(disp.x), ld(disp.y), ld(disp.z));
    let a = (mx * mx + my * my) + mz * mz;
    let b = -((dz * mz + dy * my) + mx * dx);
    let c = ((dx * dx + dy * dy) + dz * dz) - ld(radsum) * ld(radsum);
    if c < ld(F_EPSILON) {
        return -1.0;
    }
    let disc = b * b - c * a;
    if a < ld(F_EPSILON) * ld(F_EPSILON) || disc < 0.0 {
        return -1.0;
    }
    let s = disc.sqrt();
    if b - s < 0.0 {
        (s + b) / a
    } else {
        (b - s) / a
    }
}

/// The normalised direction from the obstacle centre to a body centre, with the squared length
/// summed `(z^2 + y^2) + x^2` (`y_last`: `(z^2 + x^2) + y^2`) and y stored first.
fn unit_from(center: Vector, curr: Vector, y_last: bool) -> Option<Vector> {
    let dx = ld(curr.x) - ld(center.x);
    let dy = ld(st(ld(curr.y) - ld(center.y)));
    let dz = ld(curr.z) - ld(center.z);
    let len = if y_last {
        ((dz * dz + dx * dx) + dy * dy).sqrt()
    } else {
        ((dz * dz + dy * dy) + dx * dx).sqrt()
    };
    if len < ld(F_EPSILON) {
        return None;
    }
    let inv = 1.0 / len;
    Some(Vector::new(st(inv * dx), st(dy * inv), st(dz * inv)))
}

/// `CSphere::slide_sphere(SPHEREPATH*, COLLISIONINFO*, Vector*, Vector*)`: slide `sphere` (a
/// body sphere at `curr` last step) along the crease of the collision normal and the contact
/// plane; with no crease, along the plane, or back out (rewriting `normal`) when moving into it.
pub(crate) fn slide_sphere(
    sphere: Sphere,
    path: &mut SpherePath,
    collisions: &mut CollisionInfo,
    normal: &mut Vector,
    curr: Vector,
) -> TransitionState {
    let n = *normal;
    let c = sphere.center;
    if n.x == 0.0 && n.y == 0.0 && n.z == 0.0 {
        let a = ld(curr.x) - ld(c.x);
        let b = ld(curr.y) - ld(c.y);
        let z = ld(st(ld(curr.z) - ld(c.z)));
        path.add_offset_to_check_pos(Vector::new(st(a * 0.5), st(b * 0.5), st(z * 0.5)));
        return TransitionState::Adjusted;
    }
    collisions.set_collision_normal(n);
    let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
    let a = ld(c.x) - ld(curr.x);
    let b = ld(c.y) - ld(curr.y);
    let z = ld(st(ld(c.z) - ld(curr.z)));
    let m = Vector::new(
        st(a + ld(block.x)),
        st(ld(block.y) + b),
        st(ld(block.z) + z),
    );
    let p = if collisions.contact_plane_valid {
        collisions.contact_plane.n
    } else {
        collisions.last_known_contact_plane.n
    };
    let (nx, ny, nz) = (ld(n.x), ld(n.y), ld(n.z));
    let (px, py, pz) = (ld(p.x), ld(p.y), ld(p.z));
    let kx = ld(st(ny * pz - nz * py));
    let ky = nz * px - pz * nx;
    let kz = py * nx - ny * px;
    let kzs = ld(st(kz));
    let length = (kz * kzs + ky * ky) + kx * kx;
    if !(length < ld(F_EPSILON) * ld(F_EPSILON)) {
        let dd = (ld(m.z) * kzs + ld(m.y) * ky) + kx * ld(m.x);
        let rx_num = kx * dd;
        let uy = ld(st(dd * ky));
        let uz = ld(st(dd * kzs));
        let inverse = 1.0 / ld(st(length));
        let inverse_s = ld(st(inverse));
        let rx = inverse * rx_num;
        let ry = ld(st(inverse_s * uy));
        let rz = inverse_s * uz;
        let rzs = ld(st(rz));
        if (rz * rzs + ry * ry) + rx * rx < ld(F_EPSILON) {
            return TransitionState::Collided;
        }
        path.add_offset_to_check_pos(Vector::new(
            st(rx - ld(m.x)),
            st(ry - ld(m.y)),
            st(rz - ld(m.z)),
        ));
        return TransitionState::Slid;
    }
    if !((nz * pz + ny * py) + px * nx < 0.0) {
        let d = ld(st((ld(m.x) * nx + ld(m.z) * nz) + ld(m.y) * ny));
        path.add_offset_to_check_pos(Vector::new(st(-nx * d), st(-ny * d), st(-nz * d)));
        return TransitionState::Slid;
    }
    let back = Vector::new(-m.x, -m.y, -m.z);
    *normal = back;
    let (bx, by, bz) = (ld(back.x), ld(back.y), ld(back.z));
    let len = ((bz * bz + by * by) + bx * bx).sqrt();
    if len < ld(F_EPSILON) {
        return TransitionState::Collided;
    }
    let inv = 1.0 / len;
    *normal = Vector::new(st(inv * bx), st(by * inv), st(bz * inv));
    collisions.set_collision_normal(*normal);
    TransitionState::Collided
}

impl Transition {
    /// `CSphere::intersects_sphere(Position*, Vector*, CTransition*, int)`: a Setup sphere of a
    /// static object at `p`, scaled, moved into the check position's landblock.
    pub(crate) fn intersects_object_sphere(
        &mut self,
        local: Sphere,
        p: &Position,
        scale: Vector,
        cannot_stand_on: bool,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let radius = st(ld(local.radius) * ld(scale.z));
        let center = Vector::new(
            st(ld(scale.x) * ld(local.center.x)),
            st(ld(scale.y) * ld(local.center.y)),
            st(ld(local.center.z) * ld(scale.z)),
        );
        let center = self.sphere_path.check_pos.local_to_global(p, center);
        self.intersects_sphere(Sphere::new(center, radius), cannot_stand_on, cells)
    }

    /// `CSphere::intersects_sphere(CTransition*, int)`: the response of the body's spheres to
    /// one obstacle sphere for the transition's current mode.
    pub(crate) fn intersects_sphere(
        &mut self,
        obstacle: Sphere,
        cannot_stand_on: bool,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let path = &self.sphere_path;
        let s0 = path.global_sphere[0];
        let s1 = path.global_sphere[1];
        let two = path.num_sphere >= 2;
        let disp = sub(s0.center, obstacle.center);
        let rs = st((ld(obstacle.radius) + ld(s0.radius)) - ld(F_EPSILON));
        let second = || {
            if !two {
                return TransitionState::Ok;
            }
            if collides(sub(s1.center, obstacle.center), rs) {
                TransitionState::Collided
            } else {
                TransitionState::Ok
            }
        };
        if path.obstruction_ethereal || path.insert_type == InsertType::Placement {
            let (x, y, z) = (ld(disp.x), ld(disp.y), ld(disp.z));
            if (y * y + x * x) + z * z <= ld(rs) * ld(rs) {
                return TransitionState::Collided;
            }
            return second();
        }
        if path.step_down {
            if cannot_stand_on {
                return TransitionState::Ok;
            }
            return step_sphere_down(
                obstacle,
                &mut self.sphere_path,
                &mut self.collision_info,
                disp,
                rs,
            );
        }
        if path.check_walkable {
            if collides(disp, rs) {
                return TransitionState::Collided;
            }
            return second();
        }
        if path.collide {
            if cannot_stand_on {
                return TransitionState::Ok;
            }
            if !collides(disp, rs) && !(two && collides(sub(s1.center, obstacle.center), rs)) {
                return TransitionState::Ok;
            }
            return self.land_on_sphere_walkable(disp, rs);
        }
        let state = self.object_info.state;
        if state & 3 != 0 {
            if collides(disp, rs) {
                return self.step_sphere_up(obstacle, disp, rs, cells);
            }
            if !two {
                return TransitionState::Ok;
            }
            let disp2 = sub(s1.center, obstacle.center);
            if !collides(disp2, rs) {
                return TransitionState::Ok;
            }
            return slide_toward(obstacle, &mut self.sphere_path, &mut self.collision_info, 1);
        }
        if state & PATH_CLIPPED_OI != 0 {
            if !collides(disp, rs) {
                return TransitionState::Ok;
            }
            let object = self.object_info;
            return collide_with_point(
                obstacle,
                &object,
                &mut self.sphere_path,
                &mut self.collision_info,
                s0,
                rs,
                0,
            );
        }
        if collides(disp, rs) {
            return land_on_sphere(obstacle, &mut self.sphere_path);
        }
        if !two {
            return TransitionState::Ok;
        }
        if !collides(sub(s1.center, obstacle.center), rs) {
            return TransitionState::Ok;
        }
        let object = self.object_info;
        collide_with_point(
            obstacle,
            &object,
            &mut self.sphere_path,
            &mut self.collision_info,
            s1,
            rs,
            1,
        )
    }

    /// The `collide` branch of `intersects_sphere`: after a collision on a placement pass, the
    /// body lands on top of the sphere when the landing point faces up past the walkable
    /// allowance and shortens the walk interpolation.
    fn land_on_sphere_walkable(&mut self, disp: Vector, rs: f32) -> TransitionState {
        let path = &mut self.sphere_path;
        let s0 = path.global_sphere[0];
        let v = sub(path.global_curr_center[0], s0.center);
        let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
        let w = sub(v, block);
        let rs2 = st(ld(F_EPSILON) + ld(rs));
        let (wx, wy, wz) = (ld(w.x), ld(w.y), ld(w.z));
        let (dx, dy, dz) = (ld(disp.x), ld(disp.y), ld(disp.z));
        let a = (wx * wx + wy * wy) + wz * wz;
        let b = -((dy * wy + wx * dx) + wz * dz);
        let c = ((dy * dy + dx * dx) + dz * dz) - ld(rs2) * ld(rs2);
        let mut t = (b * b - c * a).sqrt() + b;
        if t > 1.0 {
            t = (b + b) - t;
        }
        let t = t / a;
        let interp = (1.0 - t) * ld(path.walk_interp);
        if !(interp < ld(path.walk_interp)) || interp < -0.1 {
            return TransitionState::Collided;
        }
        let o = Vector::new(st(wx * t), st(wy * t), st(t * wz));
        let qx = ld(o.x) + dx;
        let qy = ld(o.y) + dy;
        let qz = ld(st(ld(o.z) + dz));
        let inv = 1.0 / ld(rs2);
        let inv_s = ld(st(inv));
        let n = Vector::new(st(inv * qx), st(qy * inv_s), st(inv_s * qz));
        if !(ld(n.z) > ld(path.walkable_allowance)) {
            return TransitionState::Ok;
        }
        let r = ld(s0.radius);
        let py = ld(st(ld(n.y) * r));
        let pz = ld(st(r * ld(n.z)));
        let contact = Vector::new(
            st(ld(s0.center.x) - ld(n.x) * r),
            st(ld(s0.center.y) - py),
            st(ld(s0.center.z) - pz),
        );
        self.collision_info
            .set_contact_plane(Plane::from_point(n, contact), false, false, false);
        self.collision_info.contact_plane_cell_id = self.sphere_path.check_pos.cell_id;
        self.sphere_path.walk_interp = st(interp);
        self.sphere_path.add_offset_to_check_pos(o);
        TransitionState::Adjusted
    }

    /// `CSphere::step_sphere_up`: a sphere lower than the step-up height is stepped onto
    /// (`CTransition::step_up`, else `SPHEREPATH::step_up_slide`); a taller one is slid along.
    fn step_sphere_up(
        &mut self,
        obstacle: Sphere,
        disp: Vector,
        rs: f32,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let reach = (ld(F_EPSILON) + ld(rs)) - ld(disp.z);
        if ld(self.object_info.step_up_height) < reach {
            return slide_sphere_object(
                obstacle,
                &mut self.sphere_path,
                &mut self.collision_info,
                disp,
                0,
            );
        }
        let curr = self.sphere_path.global_curr_center[0];
        let normal = sub(curr, obstacle.center);
        if self.step_up(normal, cells) {
            return TransitionState::Ok;
        }
        self.sphere_path.step_up_slide(&mut self.collision_info)
    }
}

/// `CSphere::step_sphere_down`: stepping down onto the top of the sphere, when the body is within
/// it (or its second sphere is) and the landing shortens the walk interpolation.
fn step_sphere_down(
    obstacle: Sphere,
    path: &mut SpherePath,
    collisions: &mut CollisionInfo,
    disp: Vector,
    rs: f32,
) -> TransitionState {
    let yy = ld(st(ld(disp.y) * ld(disp.y)));
    let xx = ld(st(ld(disp.x) * ld(disp.x)));
    let dz = ld(disp.z);
    if ld(rs) * ld(rs) < (dz * dz + xx) + yy {
        if path.num_sphere < 2 || !collides(sub(path.global_sphere[1].center, obstacle.center), rs)
        {
            return TransitionState::Ok;
        }
    }
    let reach = st(ld(F_EPSILON) + ld(rs));
    let amount = st(ld(path.step_down_amt) * ld(path.walk_interp));
    let t = if ld(amount).abs() < ld(F_EPSILON) {
        0.0
    } else {
        ((ld(reach) * ld(reach) - (xx + yy)).sqrt() - dz) / ld(amount)
    };
    let interp = (1.0 - t) * ld(path.walk_interp);
    if !(interp < ld(path.walk_interp)) || interp < -0.1 {
        return TransitionState::Collided;
    }
    let lift = st(ld(amount) * t);
    let z = ld(lift) + dz;
    let inv = 1.0 / ld(reach);
    let n = Vector::new(st(inv * ld(disp.x)), st(inv * ld(disp.y)), st(inv * z));
    if !(inv * z > ld(path.walkable_allowance)) {
        return TransitionState::Ok;
    }
    let r = ld(obstacle.radius);
    let py = ld(st(ld(n.y) * r));
    let pz = ld(st((inv * z) * r));
    let contact = Vector::new(
        st(ld(n.x) * r + ld(obstacle.center.x)),
        st(py + ld(obstacle.center.y)),
        st(pz + ld(obstacle.center.z)),
    );
    collisions.set_contact_plane(Plane::from_point(n, contact), false, false, false);
    collisions.contact_plane_cell_id = path.check_pos.cell_id;
    path.walk_interp = st(interp);
    path.add_offset_to_check_pos(Vector::new(0.0, 0.0, lift));
    TransitionState::Adjusted
}

/// `CSphere::collide_with_point`: a perfect-clip body is moved back along its step to touch the
/// sphere; any other body only records the normal and collides.
fn collide_with_point(
    obstacle: Sphere,
    object: &ObjectInfo,
    path: &mut SpherePath,
    collisions: &mut CollisionInfo,
    check: Sphere,
    rs: f32,
    sphere_num: usize,
) -> TransitionState {
    let curr = path.global_curr_center[sphere_num];
    if object.state & PERFECT_CLIP_OI == 0 {
        if let Some(n) = unit_from(obstacle.center, curr, false) {
            collisions.set_collision_normal(n);
        }
        return TransitionState::Collided;
    }
    let rs2 = st(ld(F_EPSILON) + ld(rs));
    let old_disp = sub(curr, obstacle.center);
    let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
    let movement = Vector::new(
        st((ld(check.center.x) - ld(curr.x)) + ld(block.x)),
        st(ld(block.y) + (ld(check.center.y) - ld(curr.y))),
        st(ld(block.z) + ld(st(ld(check.center.z) - ld(curr.z)))),
    );
    let t = find_time_of_collision(movement, old_disp, rs2);
    if t < ld(F_EPSILON) || !(t <= 1.0) {
        return TransitionState::Collided;
    }
    let py = st(ld(movement.y) * t);
    let pz = st(ld(movement.z) * t);
    let offset = Vector::new(
        st(ld(movement.x) * t - ld(movement.x)),
        st(ld(py) - ld(movement.y)),
        st(ld(pz) - ld(movement.z)),
    );
    let nx = ld(st(
        (ld(offset.x) + ld(check.center.x)) - ld(obstacle.center.x)
    ));
    let ny = (ld(offset.y) + ld(check.center.y)) - ld(obstacle.center.y);
    let nz = ld(st(ld(offset.z) + ld(check.center.z))) - ld(obstacle.center.z);
    let inv = 1.0 / ld(rs2);
    collisions.set_collision_normal(Vector::new(st(nx * inv), st(ny * inv), st(nz * inv)));
    path.add_offset_to_check_pos(offset);
    TransitionState::Adjusted
}

/// `CSphere::slide_sphere(OBJECTINFO*, SPHEREPATH*, COLLISIONINFO*, Vector*, float, int)`: slide
/// body sphere `sphere_num` along the crease of the sphere's normal and the contact plane, or
/// along the plane when the two are parallel and `disp` does not head into it.
fn slide_sphere_object(
    obstacle: Sphere,
    path: &mut SpherePath,
    collisions: &mut CollisionInfo,
    disp: Vector,
    sphere_num: usize,
) -> TransitionState {
    let sphere = path.global_sphere[sphere_num];
    let curr = path.global_curr_center[sphere_num];
    let Some(n) = unit_from(obstacle.center, curr, true) else {
        return TransitionState::Collided;
    };
    collisions.set_collision_normal(n);
    let p = if collisions.contact_plane_valid {
        collisions.contact_plane.n
    } else {
        collisions.last_known_contact_plane.n
    };
    let (nx, ny, nz) = (ld(n.x), ld(n.y), ld(n.z));
    let (px, py, pz) = (ld(p.x), ld(p.y), ld(p.z));
    let cx = ld(st(pz * ny - py * nz));
    let cy = ld(st(nz * px - pz * nx));
    let cz = ld(st(py * nx - px * ny));
    let block = get_block_offset(path.curr_pos.cell_id, path.check_pos.cell_id);
    let a = ld(sphere.center.x) - ld(curr.x);
    let b = ld(sphere.center.y) - ld(curr.y);
    let c = ld(st(ld(sphere.center.z) - ld(curr.z)));
    let m = Vector::new(
        st(a + ld(block.x)),
        st(ld(block.y) + b),
        st(ld(block.z) + c),
    );
    let (mx, my, mz) = (ld(m.x), ld(m.y), ld(m.z));
    let length = (cz * cz + cy * cy) + cx * cx;
    if length < ld(F_EPSILON) {
        if (pz * ld(disp.z) + py * ld(disp.y)) + px * ld(disp.x) < 0.0 {
            return TransitionState::Collided;
        }
        let d = ld(st((mz * nz + mx * nx) + my * ny));
        path.add_offset_to_check_pos(Vector::new(st(-nx * d), st(-ny * d), st(-nz * d)));
        return TransitionState::Slid;
    }
    let dd = (mz * cz + my * cy) + mx * cx;
    let q = Vector::new(st(cx * dd), st(cy * dd), st(cz * dd));
    let inv = 1.0 / length;
    let rx = ld(st(ld(q.x) * inv));
    let ry = ld(st(ld(q.y) * inv));
    let rz = inv * ld(q.z);
    if (rz * rz + ry * ry) + rx * rx < ld(F_EPSILON) {
        return TransitionState::Collided;
    }
    path.add_offset_to_check_pos(Vector::new(st(rx - mx), st(ry - my), st(rz - mz)));
    TransitionState::Slid
}

/// `CSphere::land_on_sphere`: a falling body that reaches the sphere collides with it and tries
/// a placement on top (`SPHEREPATH::set_collide`).
fn land_on_sphere(obstacle: Sphere, path: &mut SpherePath) -> TransitionState {
    let curr = path.global_curr_center[0];
    let Some(n) = unit_from(obstacle.center, curr, false) else {
        return TransitionState::Collided;
    };
    path.set_collide(n);
    path.walkable_allowance = LANDING_Z;
    TransitionState::Adjusted
}

/// `CSphere::slide_sphere(OBJECTINFO*, SPHEREPATH*, COLLISIONINFO*, CSphere*, Vector*, float*,
/// int)`: slide body sphere `sphere_num` away from the obstacle centre.
fn slide_toward(
    obstacle: Sphere,
    path: &mut SpherePath,
    collisions: &mut CollisionInfo,
    sphere_num: usize,
) -> TransitionState {
    let curr = path.global_curr_center[sphere_num];
    let Some(mut n) = unit_from(obstacle.center, curr, true) else {
        return TransitionState::Collided;
    };
    let sphere = path.global_sphere[sphere_num];
    slide_sphere(sphere, path, collisions, &mut n, curr)
}
