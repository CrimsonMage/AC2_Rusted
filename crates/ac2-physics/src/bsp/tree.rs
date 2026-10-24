//! `BSPTREE` collision: the body's spheres, cached in the mesh's space
//! (`SPHEREPATH::cache_localspace_sphere`), against a physics BSP for the transition's mode
//! (`BSPTREE::FindObjCollisions`), and the responses it hands back in world space.

use super::node::{PolyHit, SolidHit};
use super::BspTree;
use crate::cells::CellSource;
use crate::fpu::{ld, st, LANDING_Z};
use crate::shapes::sphere::slide_sphere;
use crate::transition::collision::{
    CollisionInfo, ObjectInfo, CONTACT_OI, PATH_CLIPPED_OI, PERFECT_CLIP_OI,
};
use crate::transition::sphere_path::{InsertType, SpherePath};
use crate::transition::{Transition, TransitionState};
use crate::vector::{Sphere, Vector};

const ROOT: usize = 0;

/// `a - b` per component, each stored.
fn sub(a: Vector, b: Vector) -> Vector {
    Vector::new(
        st(ld(a.x) - ld(b.x)),
        st(ld(a.y) - ld(b.y)),
        st(ld(a.z) - ld(b.z)),
    )
}

/// A world offset scaled per axis, each stored.
fn scaled(v: Vector, scale: Vector) -> Vector {
    Vector::new(
        st(ld(v.x) * ld(scale.x)),
        st(ld(v.y) * ld(scale.y)),
        st(ld(v.z) * ld(scale.z)),
    )
}

/// The contact flags `FindObjCollisions` passes through to `COLLISIONINFO::set_contact_plane`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ContactKind {
    pub water: bool,
    pub landscape: bool,
    pub environment: bool,
}

impl BspTree {
    /// `BSPTREE::FindObjCollisions`: `scale` takes mesh offsets to world ones, `inverse` world
    /// to mesh (the localspace spheres are already cached with it).
    pub(crate) fn find_collisions(
        &self,
        transition: &mut Transition,
        scale: Vector,
        inverse: Vector,
        kind: ContactKind,
        cells: &dyn CellSource,
    ) -> TransitionState {
        if self.nodes.is_empty() {
            return TransitionState::Ok;
        }
        let path = &transition.sphere_path;
        let s0 = path.localspace_sphere[0];
        let s1 = path.localspace_sphere[1];
        let two = path.num_sphere >= 2;
        let movement = sub(s0.center, path.localspace_curr_center[0]);
        if path.insert_type == InsertType::Placement || path.obstruction_ethereal {
            let center_check = !path.bldg_check || !path.hits_interior_cell;
            if self.sphere_intersects_solid(ROOT, &s0, center_check)
                || (two && self.sphere_intersects_solid(ROOT, &s1, center_check))
            {
                return TransitionState::Collided;
            }
            return TransitionState::Ok;
        }
        if path.check_walkable {
            return self.check_walkable(path, s0);
        }
        if path.step_down {
            let (path, collisions) = (&mut transition.sphere_path, &mut transition.collision_info);
            return self.step_sphere_down(path, collisions, s0, scale, inverse, kind);
        }
        if path.collide {
            let (path, collisions) = (&mut transition.sphere_path, &mut transition.collision_info);
            return self.land_on_walkable(path, collisions, s0, movement, scale, inverse, kind);
        }
        let state = transition.object_info.state;
        let mut hit = PolyHit::default();
        if state & CONTACT_OI != 0 {
            if self.sphere_intersects_poly(ROOT, &s0, movement, &mut hit) {
                let normal = self.polygons[hit.polygon.unwrap_or_default()].plane.n;
                return self.step_sphere_up(transition, normal, cells);
            }
            if !two {
                return TransitionState::Ok;
            }
            let mut second = PolyHit {
                polygon: None,
                contact: hit.contact,
            };
            let path = &mut transition.sphere_path;
            if self.sphere_intersects_poly(ROOT, &s1, movement, &mut second) {
                let normal = self.polygons[second.polygon.unwrap_or_default()].plane.n;
                return self.slide_sphere(path, &mut transition.collision_info, normal);
            }
            if let Some(index) = second.polygon {
                let normal = path
                    .localspace_pos
                    .local_to_global_vec(self.polygons[index].plane.n);
                path.set_neg_poly_hit(false, normal);
            } else if let Some(index) = hit.polygon {
                let normal = path
                    .localspace_pos
                    .local_to_global_vec(self.polygons[index].plane.n);
                path.set_neg_poly_hit(true, normal);
            }
            return TransitionState::Ok;
        }
        let path = &mut transition.sphere_path;
        if state & PATH_CLIPPED_OI == 0 {
            if self.sphere_intersects_poly(ROOT, &s0, movement, &mut hit) || hit.polygon.is_some() {
                let index = hit.polygon.unwrap_or_default();
                let normal = path
                    .localspace_pos
                    .local_to_global_vec(self.polygons[index].plane.n);
                path.set_collide(normal);
                path.walkable_allowance = LANDING_Z;
                return TransitionState::Adjusted;
            }
            if two
                && (self.sphere_intersects_poly(ROOT, &s1, movement, &mut hit)
                    || hit.polygon.is_some())
            {
                let index = hit.polygon.unwrap_or_default();
                let normal = path
                    .localspace_pos
                    .local_to_global_vec(self.polygons[index].plane.n);
                transition.collision_info.set_collision_normal(normal);
                return TransitionState::Collided;
            }
            return TransitionState::Ok;
        }
        if self.sphere_intersects_poly(ROOT, &s0, movement, &mut hit) || hit.polygon.is_some() {
            let object = transition.object_info;
            let curr = path.localspace_curr_center[0];
            let index = hit.polygon.unwrap_or_default();
            return self.collide_with_pt(
                &object,
                path,
                &mut transition.collision_info,
                s0,
                curr,
                index,
                hit.contact,
                scale,
            );
        }
        TransitionState::Ok
    }

    /// `BSPTREE::placement_insert`: push the localspace spheres out of the polygons they touch
    /// (`CPolygon::adjust_to_placement_poly`), doubling the push radius when only a solid leaf
    /// holds them, up to 20 times; the check position follows the first sphere.
    pub(crate) fn placement_insert(&self, transition: &mut Transition) -> TransitionState {
        if self.nodes.is_empty() {
            return TransitionState::Ok;
        }
        let path = &mut transition.sphere_path;
        let original = path.localspace_sphere[0];
        let mut valid = original;
        let mut valid2 = path.localspace_sphere[1];
        let mut radius = valid.radius;
        let center_check = !path.bldg_check || !path.hits_interior_cell;
        let two = path.num_sphere >= 2;
        let mut hit = SolidHit::default();
        for attempt in 0..20 {
            hit.center_solid = false;
            let first =
                self.sphere_intersects_solid_poly(ROOT, &valid, radius, &mut hit, center_check);
            let struck_first = first;
            if !first {
                if !two {
                    if attempt == 0 {
                        return TransitionState::Ok;
                    }
                    let dx = ld(valid.center.x) - ld(original.center.x);
                    let dy = ld(valid.center.y) - ld(original.center.y);
                    let dz = ld(valid.center.z) - ld(original.center.z);
                    let m = path.localspace_pos.frame.m.map(ld);
                    let gx = ld(st((dx * m[0] + dy * m[3]) + dz * m[6]));
                    let gy = ld(st((dx * m[1] + dy * m[4]) + dz * m[7]));
                    let gz = (dx * m[2] + dy * m[5]) + dz * m[8];
                    path.cell_array_valid = false;
                    let origin = &mut path.check_pos.frame.origin;
                    origin.x = st(gx + ld(origin.x));
                    origin.y = st(gy + ld(origin.y));
                    origin.z = st(gz + ld(origin.z));
                    path.cache_global_sphere(Some(Vector::new(st(gx), st(gy), st(gz))));
                    return TransitionState::Adjusted;
                }
                if !self.sphere_intersects_solid_poly(ROOT, &valid2, radius, &mut hit, center_check)
                {
                    if attempt == 0 {
                        return TransitionState::Ok;
                    }
                    let offset = path
                        .localspace_pos
                        .local_to_global_vec(sub(valid.center, original.center));
                    path.add_offset_to_check_pos(offset);
                    return TransitionState::Adjusted;
                }
            }
            match hit.polygon.take() {
                None => radius = st(ld(radius) + ld(radius)),
                Some(index) => {
                    let polygon = &self.polygons[index];
                    let (center_solid, check) = (hit.center_solid, center_check);
                    if struck_first {
                        polygon.adjust_to_placement_poly(
                            &mut valid,
                            &mut valid2,
                            radius,
                            center_solid,
                            check,
                        );
                    } else {
                        polygon.adjust_to_placement_poly(
                            &mut valid2,
                            &mut valid,
                            radius,
                            center_solid,
                            check,
                        );
                    }
                    radius = valid.radius;
                }
            }
        }
        TransitionState::Collided
    }

    /// `BSPTREE::check_walkable`: a walkable polygon under the sphere ends the probe
    /// (`Collided`).
    fn check_walkable(&self, path: &SpherePath, sphere: Sphere) -> TransitionState {
        if self.hits_walkable(ROOT, path, &sphere, path.localspace_z) {
            TransitionState::Collided
        } else {
            TransitionState::Ok
        }
    }

    /// The walkable polygon `find_walkable` moved `valid` onto, made the contact plane (in the
    /// check position's frame, rescaled) and the path's walkable polygon.
    #[allow(clippy::too_many_arguments)]
    fn record_walkable(
        &self,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        valid: Sphere,
        polygon: usize,
        scale: Vector,
        inverse: Vector,
        kind: ContactKind,
    ) {
        let polygon = &self.polygons[polygon];
        let plane = path
            .check_pos
            .plane_local_to_global(&path.localspace_pos, polygon.plane)
            .scale(inverse);
        collisions.set_contact_plane(plane, kind.water, kind.landscape, kind.environment);
        collisions.contact_plane_cell_id = path.check_pos.cell_id;
        let (up, local) = (path.localspace_z, path.localspace_pos);
        path.set_walkable(valid, polygon, up, local, scale);
    }

    /// The `collide` branch of `FindObjCollisions`: after a collision the placement pass puts the
    /// sphere back onto the walkable polygon under its path.
    #[allow(clippy::too_many_arguments)]
    fn land_on_walkable(
        &self,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        s0: Sphere,
        movement: Vector,
        scale: Vector,
        inverse: Vector,
        kind: ContactKind,
    ) -> TransitionState {
        let mut valid = s0;
        let (mut polygon, mut changed) = (None, false);
        let up = path.localspace_z;
        self.find_walkable(
            ROOT,
            path,
            &mut valid,
            &mut polygon,
            movement,
            up,
            &mut changed,
        );
        let (true, Some(polygon)) = (changed, polygon) else {
            return TransitionState::Ok;
        };
        let offset = path
            .localspace_pos
            .local_to_global_vec(sub(valid.center, s0.center));
        path.add_offset_to_check_pos(scaled(offset, scale));
        self.record_walkable(path, collisions, valid, polygon, scale, inverse, kind);
        TransitionState::Adjusted
    }

    /// `BSPTREE::step_sphere_down`: the step-down amount, in mesh space, onto a walkable polygon.
    fn step_sphere_down(
        &self,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        s0: Sphere,
        scale: Vector,
        inverse: Vector,
        kind: ContactKind,
    ) -> TransitionState {
        let amount = -(ld(path.step_down_amt) * ld(path.walk_interp));
        let up = path.localspace_z;
        let ty = ld(st(amount * ld(up.y)));
        let tz = ld(st(amount * ld(up.z)));
        let movement = Vector::new(
            st(amount * ld(up.x) / ld(scale.x)),
            st(ty / ld(scale.y)),
            st(tz / ld(scale.z)),
        );
        let mut valid = s0;
        let (mut polygon, mut changed) = (None, false);
        self.find_walkable(
            ROOT,
            path,
            &mut valid,
            &mut polygon,
            movement,
            up,
            &mut changed,
        );
        let (true, Some(polygon)) = (changed, polygon) else {
            return TransitionState::Ok;
        };
        let dx = ld(valid.center.x) - ld(s0.center.x);
        let dy = ld(valid.center.y) - ld(s0.center.y);
        let dz = ld(valid.center.z) - ld(s0.center.z);
        let m = path.localspace_pos.frame.m.map(ld);
        let gx = ld(st((dz * m[6] + dy * m[3]) + dx * m[0]));
        let gy = ld(st((dz * m[7] + dy * m[4]) + dx * m[1]));
        let gz = (dz * m[8] + dy * m[5]) + dx * m[2];
        path.add_offset_to_check_pos(Vector::new(
            st(gx * ld(scale.x)),
            st(gy * ld(scale.y)),
            st(gz * ld(scale.z)),
        ));
        self.record_walkable(path, collisions, valid, polygon, scale, inverse, kind);
        TransitionState::Adjusted
    }

    /// `BSPTREE::step_sphere_up`: step up along the hit polygon's normal, else slide
    /// (`SPHEREPATH::step_up_slide`).
    fn step_sphere_up(
        &self,
        transition: &mut Transition,
        normal: Vector,
        cells: &dyn CellSource,
    ) -> TransitionState {
        let global = transition
            .sphere_path
            .localspace_pos
            .local_to_global_vec_zxy(normal);
        if transition.step_up(global, cells) {
            return TransitionState::Ok;
        }
        transition
            .sphere_path
            .step_up_slide(&mut transition.collision_info)
    }

    /// `BSPTREE::slide_sphere`: `CSphere::slide_sphere` of the first sphere along the hit
    /// polygon's normal in world axes.
    fn slide_sphere(
        &self,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        normal: Vector,
    ) -> TransitionState {
        let mut global = path.localspace_pos.local_to_global_vec_zxy(normal);
        let (sphere, curr) = (path.global_sphere[0], path.global_curr_center[0]);
        slide_sphere(sphere, path, collisions, &mut global, curr)
    }

    /// `BSPTREE::collide_with_pt`: a perfect-clip body is moved back to touch the polygon
    /// (`adjust_to_plane`); any other body records the polygon's normal and collides.
    #[allow(clippy::too_many_arguments)]
    fn collide_with_pt(
        &self,
        object: &ObjectInfo,
        path: &mut SpherePath,
        collisions: &mut CollisionInfo,
        check: Sphere,
        curr: Vector,
        polygon: usize,
        contact: Vector,
        scale: Vector,
    ) -> TransitionState {
        let normal = path
            .localspace_pos
            .local_to_global_vec(self.polygons[polygon].plane.n);
        if object.state & PERFECT_CLIP_OI == 0 {
            collisions.set_collision_normal(normal);
            return TransitionState::Collided;
        }
        let mut valid = check;
        if !self.adjust_to_plane(&mut valid, curr, polygon, contact) {
            return TransitionState::Collided;
        }
        collisions.set_collision_normal(normal);
        let offset = path
            .localspace_pos
            .local_to_global_vec(sub(valid.center, check.center));
        path.add_offset_to_check_pos(scaled(offset, scale));
        TransitionState::Adjusted
    }

    /// `BSPTREE::adjust_to_plane`: the latest point along `curr -> sphere` clear of the polygons,
    /// first by `CPolygon::adjust_sphere_to_poly` steps, then by bisection to 0.02.
    fn adjust_to_plane(
        &self,
        sphere: &mut Sphere,
        curr: Vector,
        polygon: usize,
        contact: Vector,
    ) -> bool {
        let movement = sub(sphere.center, curr);
        let mut hit = PolyHit {
            polygon: Some(polygon),
            contact,
        };
        let place = |sphere: &mut Sphere, t: f64| {
            let px = ld(st(ld(movement.x) * t));
            let py = ld(st(ld(movement.y) * t));
            let pz = ld(movement.z) * t;
            sphere.center = Vector::new(
                st(px + ld(curr.x)),
                st(py + ld(curr.y)),
                st(pz + ld(curr.z)),
            );
        };
        let (mut lower, mut upper) = (0.0_f64, 1.0_f64);
        let mut attempts = 0;
        loop {
            let index = hit.polygon.unwrap_or(polygon);
            let t = self.polygons[index].adjust_sphere_to_poly(sphere, curr, movement);
            if t == 1.0 {
                break;
            }
            place(sphere, t);
            if !self.sphere_intersects_poly(ROOT, sphere, movement, &mut hit) {
                lower = t;
                break;
            }
            upper = t;
            attempts += 1;
            if attempts >= 15 {
                return false;
            }
        }
        if attempts >= 15 {
            return false;
        }
        loop {
            let mid = (lower + upper) * 0.5;
            place(sphere, mid);
            if self.sphere_intersects_poly(ROOT, sphere, movement, &mut hit) {
                upper = mid;
            } else {
                lower = mid;
            }
            if upper - lower < 0.02 {
                break;
            }
            attempts += 1;
            if attempts >= 15 {
                break;
            }
        }
        place(sphere, lower);
        true
    }
}
