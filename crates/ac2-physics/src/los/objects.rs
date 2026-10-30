//! Objects in the line of sight: `CObjCell::TraceRayObjects` over a cell's shadow objects,
//! `CPhysicsObj::CheckRayIntersect` (physics BSPs via `CPartGroup::TraceRayPhysics` /
//! `CPhysicsPart::TraceRayPhysics`, else its spheres or cylinders), and the `Sphere::TraceRay` and
//! `Cylsphere::TraceRay` shape tests.

use super::{LosParameters, LosTrace, Ray, RayIntersection};
use crate::cells::ObjCell;
use crate::collision_types::is_blocking;
use crate::fpu::{epsilon_squared, ld, st, F_EPSILON};
use crate::landdefs::get_block_offset;
use crate::objects::{StaticObject, HAS_PHYSICS_BSP_PS};
use crate::shapes::cylsphere::CylSphere;
use crate::vector::{Sphere, Vector};

impl<'a> LosTrace<'a> {
    /// `CObjCell::TraceRayObjects`: every shadow object of the cell in order; the last nonzero
    /// result stands and the final cell becomes this cell.
    pub(super) fn trace_ray_objects(
        &mut self,
        params: &mut LosParameters,
        cell: ObjCell<'a>,
    ) -> u32 {
        let mut result = 0;
        for &id in cell.objects() {
            let Some(object) = self.cells.object(id) else {
                continue;
            };
            let hit = check_ray_intersect(object, params);
            if hit != 0 {
                result = hit;
            }
        }
        if result != 0 {
            params.final_cell_id = match cell {
                ObjCell::Land(land) => land.cell_id,
                ObjCell::Env(env) => env.cell_id,
            };
        }
        result
    }
}

/// `CPhysicsObj::CheckRayIntersect` for an object in the cell's shadow list (the static ones and
/// the server objects, both `StaticObject`s; `IsLOSBlocking` first, against the ray's sets). With the physics
/// representation, an object without spheres or cylinders traces its part BSPs over the ray
/// shortened to `delta`; otherwise the nearest sphere (or, without spheres, cylinder) in the
/// object's unscaled frame sets `delta` and the impact normal `-dir`.
fn check_ray_intersect(object: &StaticObject, params: &mut LosParameters) -> u32 {
    // `CPhysicsObj::IsLOSBlocking` -> `IsBlocking(m_weenie_LOS, m_physics_LOS)`.
    if !is_blocking(
        &params.weenie_los,
        params.physics_los,
        object.instance_id.unwrap_or(0),
        object.blocking,
    ) {
        return 0;
    }
    if params.intersect != RayIntersection::PhysicsRep {
        // `UseExactDrawingRep` traces the drawn parts (`CPhysicsObj::TraceRay`), not ported.
        return 0;
    }
    let shape = &object.shape;
    let presence =
        !shape.spheres.is_empty() || !shape.cylspheres.is_empty() || !shape.bsps.is_empty();
    if !presence {
        return 0;
    }
    let ray = params.current_ray;
    let delta = params.delta;
    if shape.spheres.is_empty() && shape.cylspheres.is_empty() {
        let reach = ld(ray.length) * ld(delta);
        let offset = Vector::new(
            st(ld(ray.dir.x) * reach),
            st(ld(ray.dir.y) * reach),
            st(ld(ray.dir.z) * reach),
        );
        let short = Ray::from_offset(ray.pt, offset);
        let mut fraction = delta;
        let mut normal = Vector::new(0.0, 0.0, 1.0);
        let hit = trace_ray_physics(
            object,
            &short,
            params.current_cell_id,
            &mut fraction,
            &mut normal,
        );
        if hit != 0 {
            params.impact_normal = normal;
            params.delta = fraction;
        }
        return hit;
    }
    let s = object.scale;
    let inverse = Vector::new(st(1.0 / ld(s.x)), st(1.0 / ld(s.y)), st(1.0 / ld(s.z)));
    let position = &object.position;
    let offset = get_block_offset(position.cell_id, params.current_cell_id);
    let start = Vector::new(
        st(ld(ray.pt.x) + ld(offset.x)),
        st(ld(ray.pt.y) + ld(offset.y)),
        st(ld(ray.pt.z) + ld(offset.z)),
    );
    let local = position.frame.global_to_local(start);
    let local_start = scale_by(local, inverse);
    let reach = ld(ray.length) * ld(delta);
    let reach_y = st(ld(ray.dir.y) * reach);
    let reach_z = st(ld(ray.dir.z) * reach);
    let end_z = st(ld(ray.pt.z) + ld(reach_z));
    let end = Vector::new(
        st((ld(ray.pt.x) + ld(ray.dir.x) * reach) + ld(offset.x)),
        st((ld(ray.pt.y) + ld(reach_y)) + ld(offset.y)),
        st(ld(end_z) + ld(offset.z)),
    );
    let local_end = scale_by(position.frame.global_to_local(end), inverse);
    let local_ray = Ray::line_segment(local_start, local_end);
    let mut best = 1.0_f32;
    let mut hit = 0;
    let mut take = |t: Option<f32>, params: &mut LosParameters| {
        if let Some(t) = t.filter(|&t| t < best) {
            best = t;
            hit = 1;
            params.impact_normal = Vector::new(-ray.dir.x, -ray.dir.y, -ray.dir.z);
        }
    };
    if !shape.spheres.is_empty() {
        for sphere in &shape.spheres {
            take(trace_ray_sphere(sphere, &local_ray), params);
        }
    } else {
        for cylinder in &shape.cylspheres {
            take(trace_ray_cylsphere(cylinder, &local_ray), params);
        }
    }
    if best < 1.0 {
        params.delta = st(ld(best) * ld(delta));
    }
    hit
}

/// Each component of `v` times `scale`, stored.
fn scale_by(v: Vector, scale: Vector) -> Vector {
    Vector::new(
        st(ld(v.x) * ld(scale.x)),
        st(ld(v.y) * ld(scale.y)),
        st(ld(v.z) * ld(scale.z)),
    )
}

/// `CPartGroup::TraceRayPhysics` -> `CPartArray::TraceRayThroughPartsPhysics` ->
/// `CPhysicsPart::TraceRayPhysics`: every part BSP, in the object's frame scaled by the inverse
/// total scale. Each hit multiplies `fraction` by the part's fraction (retail compounds them
/// part after part) and the last hit's result stands.
fn trace_ray_physics(
    object: &StaticObject,
    ray: &Ray,
    cell_id: u32,
    fraction: &mut f32,
    normal: &mut Vector,
) -> u32 {
    if object.state & HAS_PHYSICS_BSP_PS == 0 {
        return 0;
    }
    let s = object.scale;
    let inverse = Vector::new(st(1.0 / ld(s.x)), st(1.0 / ld(s.y)), st(1.0 / ld(s.z)));
    let position = &object.position;
    let mut result = 0;
    for tree in &object.shape.bsps {
        let offset = get_block_offset(position.cell_id, cell_id);
        let start = Vector::new(
            st(ld(offset.x) + ld(ray.pt.x)),
            st(ld(offset.y) + ld(ray.pt.y)),
            st(ld(offset.z) + ld(ray.pt.z)),
        );
        let local_start = scale_by(position.frame.global_to_local(start), inverse);
        let length = ld(ray.length);
        let end_z = st(length * ld(ray.dir.z) + ld(ray.pt.z));
        let end = Vector::new(
            st((length * ld(ray.dir.x) + ld(ray.pt.x)) + ld(offset.x)),
            st((length * ld(ray.dir.y) + ld(ray.pt.y)) + ld(offset.y)),
            st(ld(end_z) + ld(offset.z)),
        );
        let local_end = position.frame.global_to_local(end);
        let local_end_z = st(ld(local_end.z) * ld(inverse.z));
        let along = Vector::new(
            st(ld(local_end.x) * ld(inverse.x) - ld(local_start.x)),
            st(ld(local_end.y) * ld(inverse.y) - ld(local_start.y)),
            st(ld(local_end_z) - ld(local_start.z)),
        );
        let local = Ray::from_offset(local_start, along);
        if let Some(t) = tree.trace_ray(&local, normal) {
            *fraction = st(ld(t) * ld(*fraction));
            result = 1;
        }
    }
    result
}

/// `Sphere::TraceRay`: 0 when the ray starts inside; else the fraction where it enters,
/// approaching and reaching the sphere within the ray's length.
pub(crate) fn trace_ray_sphere(sphere: &Sphere, ray: &Ray) -> Option<f32> {
    let c = sphere.center;
    let dx = ld(ray.pt.x) - ld(c.x);
    let dy = ld(ray.pt.y) - ld(c.y);
    let dz = ld(ray.pt.z) - ld(c.z);
    let dist2 = st((dz * dz + dx * dx) + dy * dy);
    let radius = ld(sphere.radius);
    let radius2 = st(radius * radius);
    if dist2 < radius2 {
        return Some(0.0);
    }
    let along = (dz * ld(ray.dir.z) + dy * ld(ray.dir.y)) + dx * ld(ray.dir.x);
    let b = st(along);
    if !(along < 0.0) {
        return None;
    }
    let disc_wide = ld(b) * ld(b) - (ld(dist2) - ld(radius2));
    let disc = st(disc_wide);
    if !(disc_wide > 0.0) {
        return None;
    }
    let reach = ld(ray.length) + radius;
    if !(ld(dist2) < reach * reach) {
        return None;
    }
    let t = (-ld(b) - ld(disc).sqrt()) / ld(ray.length);
    if !(t < 1.0) {
        return None;
    }
    Some(if t < 0.0 { 0.0 } else { st(t) })
}

/// `Cylsphere::TraceRay`: 0 when the ray starts inside the cylinder; else the fraction where
/// it enters through the side or a cap within the ray's length.
pub(crate) fn trace_ray_cylsphere(cylinder: &CylSphere, ray: &Ray) -> Option<f32> {
    let eps = ld(F_EPSILON);
    let low = cylinder.low;
    let (px, py, pz) = (ld(ray.pt.x), ld(ray.pt.y), ld(ray.pt.z));
    let (dir_x, dir_y, dir_z) = (ld(ray.dir.x), ld(ray.dir.y), ld(ray.dir.z));
    let rx = st(px - ld(low.x));
    let ry = st(py - ld(low.y));
    let length = ld(ray.length);
    let radius = ld(cylinder.radius);
    let reach = length + radius;
    let low_z = ld(low.z);
    if length < eps {
        let flat = ld(ry) * ld(ry) + ld(rx) * ld(rx);
        if !(radius * radius < flat) && !(pz < low_z) && !(ld(cylinder.height) + low_z < pz) {
            return Some(0.0);
        }
    }
    let dist2 = st(ld(rx) * ld(rx) + ld(ry) * ld(ry));
    if ld(dist2) > reach * reach {
        return None;
    }
    let radius2 = st(radius * radius);
    if ld(dist2) < ld(radius2) {
        // Within the radius: through a cap.
        let plane = if low_z <= pz {
            let top = st(ld(cylinder.height) + low_z);
            if !(ld(top) < pz) {
                return Some(0.0);
            }
            if -eps < dir_z {
                return None;
            }
            ld(top)
        } else {
            if dir_z < eps {
                return None;
            }
            low_z
        };
        let s_wide = (plane - pz) / dir_z;
        let s = st(s_wide);
        if !(s_wide < length) {
            return None;
        }
        let hit_x = st(dir_x * ld(s) + px);
        let hit_y = dir_y * ld(s) + py;
        let ex = ld(hit_x) - ld(low.x);
        let ey = hit_y - ld(low.y);
        if !((ey * ey + ex * ex) < ld(radius2)) {
            return None;
        }
        return Some(st(ld(s) / length));
    }
    // Outside the radius: through the side, or a cap past it.
    let a = st(dir_y * dir_y + dir_x * dir_x);
    let b = st(dir_y * ld(ry) + dir_x * ld(rx));
    if epsilon_squared() > ld(a) {
        return None;
    }
    if !(ld(b) < 0.0) {
        return None;
    }
    let disc = ld(b) * ld(b) - (ld(dist2) - radius * radius) * ld(a);
    if !(disc > eps) {
        return None;
    }
    let root = st(disc.sqrt());
    let inverse_a = st(1.0 / ld(a));
    let s1_wide = (-ld(b) - ld(root)) * ld(inverse_a);
    let s1 = st(s1_wide);
    if s1_wide > length {
        return None;
    }
    let z1 = ld(s1) * dir_z + pz;
    let z2 = st(((ld(root) - ld(b)) * ld(inverse_a)) * dir_z + pz);
    let flat_dir = dir_z.abs() <= eps;
    if z1 < low_z && !flat_dir {
        if ld(z2) < low_z {
            return None;
        }
        let s = (low_z - pz) / dir_z;
        return (s < length).then(|| st(s / length));
    }
    let top = st(ld(cylinder.height) + low_z);
    if z1 <= ld(top) || flat_dir {
        return Some(st(ld(s1) / length));
    }
    if ld(z2) > ld(top) {
        return None;
    }
    let s = (ld(top) - pz) / dir_z;
    (s < length).then(|| st(s / length))
}
