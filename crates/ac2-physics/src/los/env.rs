//! The indoor line of sight: `CEnvCell::TraceRay` (the cell's physics BSP, its shadow objects,
//! then the nearest portal it walks through), `BSPTREE`/`BSPNODE`/`BSPLEAF::TraceRay` (only
//! solid leaves block) and `CPolygon::TraceRay` with `Plane::intersect_line`.

use super::objects::trace_ray_sphere;
use super::{sub, LosParameters, LosTrace, Ray, RayIntersection};
use crate::bsp::polygon::point_in_polygon;
use crate::bsp::{BspTree, Node};
use crate::cells::ObjCell;
use crate::envcell::EnvCell;
use crate::fpu::{ld, st, F_EPSILON};
use crate::landdefs::adjust_to_outside;
use crate::vector::{Plane, Vector};

/// `BSPNODE::TraceRay`'s side tolerance.
const BSP_SIDE_EPSILON: f32 = 0.1;

/// `Plane::intersect_line`: where the segment `p0`..`p1` crosses the plane, as `1 - t` (kept
/// as a `double`) and the crossing point. `None` when the segment is parallel to the plane or
/// does not reach it.
fn intersect_line(plane: &Plane, p0: Vector, p1: Vector) -> Option<(f64, Vector)> {
    let e = sub(p1, p0);
    let n = plane.n;
    let den = (ld(e.z) * ld(n.z) + ld(e.y) * ld(n.y)) + ld(e.x) * ld(n.x);
    if den.abs() < ld(F_EPSILON) {
        return None;
    }
    let dot = (ld(p0.x) * ld(n.x) + ld(n.z) * ld(p0.z)) + ld(n.y) * ld(p0.y);
    let t = (-ld(plane.d) - dot) / den;
    if t < 0.0 || t > 1.0 {
        return None;
    }
    let point = Vector::new(
        st(ld(e.x) * t + ld(p0.x)),
        st(ld(st(ld(e.y) * t)) + ld(p0.y)),
        st(ld(st(ld(e.z) * t)) + ld(p0.z)),
    );
    Some((1.0 - t, point))
}

/// `CPolygon::TraceRay`: the fraction of the ray at which it crosses the polygon.
pub(super) fn trace_polygon(vertices: &[Vector], plane: &Plane, ray: &Ray) -> Option<f32> {
    let end = ray.end_stored_yz();
    let (remaining, point) = intersect_line(plane, ray.pt, end)?;
    point_in_polygon(vertices, plane, point).then(|| st(1.0 - remaining))
}

impl BspTree {
    /// `BSPTREE::TraceRay`: through the root's bounding sphere first.
    pub(crate) fn trace_ray(&self, ray: &Ray, normal: &mut Vector) -> Option<f32> {
        let Some(sphere) = self.bounding_sphere() else {
            return None;
        };
        trace_ray_sphere(&sphere, ray)?;
        self.trace_ray_node(0, ray, normal)
    }

    /// The root node's `TraceRay` (`CEnvCell::TraceRay` skips the sphere test).
    pub(crate) fn trace_ray_root(&self, ray: &Ray, normal: &mut Vector) -> Option<f32> {
        if self.nodes.is_empty() {
            return None;
        }
        self.trace_ray_node(0, ray, normal)
    }

    /// `BSPNODE::TraceRay` / `BSPLEAF::TraceRay`: a solid leaf blocks at once; a node passes a
    /// ray lying on one side (within 0.1) to that child, else splits it at the plane and tries
    /// the start's side first. A hit right at a split takes the plane's normal (flipped when the
    /// start is behind it).
    fn trace_ray_node(&self, index: usize, ray: &Ray, normal: &mut Vector) -> Option<f32> {
        let (plane, pos, neg) = match &self.nodes[index] {
            Node::Leaf { solid, .. } => return solid.then_some(0.0),
            Node::Internal {
                plane, pos, neg, ..
            } => (*plane, *pos, *neg),
        };
        let start = ray.pt;
        let end = ray.end_stored_yz();
        let n = plane.n;
        let dist =
            |p: Vector| ((ld(p.z) * ld(n.z) + ld(p.y) * ld(n.y)) + ld(p.x) * ld(n.x)) + ld(plane.d);
        let side = |d: f64| {
            if d > ld(BSP_SIDE_EPSILON) {
                0
            } else if d < -ld(BSP_SIDE_EPSILON) {
                1
            } else {
                2
            }
        };
        let (d0, d1) = (dist(start), dist(end));
        let (s0, s1) = (side(d0), side(d1));
        if let Some(child) = pos.filter(|_| s0 != 1 && s1 != 1) {
            return self.trace_ray_node(child, ray, normal);
        }
        if let Some(child) = neg.filter(|_| s0 != 0 && s1 != 0) {
            return self.trace_ray_node(child, ray, normal);
        }
        let (first, second) = if s0 == 1 { (neg, pos) } else { (pos, neg) };
        let diff = d0 - d1;
        let t = if ld(F_EPSILON) < diff.abs() {
            ld(st(d0 / diff))
        } else {
            0.0
        };
        let dz = st(ld(end.z) - ld(start.z));
        let cx = st((ld(end.x) - ld(start.x)) * t);
        let mid = Vector::new(
            st(ld(cx) + ld(start.x)),
            st((ld(end.y) - ld(start.y)) * t + ld(start.y)),
            st(ld(dz) * t + ld(start.z)),
        );
        let split_normal = |normal: &mut Vector, hit: f32| {
            if ld(hit).abs() < ld(F_EPSILON) {
                *normal = if s0 != 1 {
                    n
                } else {
                    Vector::new(-n.x, -n.y, -n.z)
                };
            }
        };
        let near = Ray::from_offset(start, sub(mid, start));
        if let Some(hit) = first.and_then(|child| self.trace_ray_node(child, &near, normal)) {
            split_normal(normal, hit);
            return Some(st(t * ld(hit)));
        }
        let far = Ray::from_offset(mid, sub(end, mid));
        if let Some(hit) = second.and_then(|child| self.trace_ray_node(child, &far, normal)) {
            split_normal(normal, hit);
            return Some(st((1.0 - t) * ld(hit) + t));
        }
        None
    }
}

impl<'a> LosTrace<'a> {
    /// `CEnvCell::TraceRay`: the cell's physics BSP (in cell space) and shadow objects nearer
    /// than `delta`, then the nearest portal (other than `skip`, the one the ray came in by).
    /// Through a portal the trace continues in the next cell (the outside: `CLandCell::Trace`);
    /// a portal it cannot pass counts as clear unless the trace is portal-only. A cell already
    /// on the trace's stack is clear.
    pub(super) fn env_trace_ray(
        &mut self,
        params: &mut LosParameters,
        cell: &'a EnvCell,
        skip: Option<usize>,
    ) -> u32 {
        if self.on_stack.contains(&cell.cell_id) {
            return 0;
        }
        self.on_stack.push(cell.cell_id);
        let result = self.env_trace_ray_inner(params, cell, skip);
        self.on_stack.pop();
        result
    }

    fn env_trace_ray_inner(
        &mut self,
        params: &mut LosParameters,
        cell: &'a EnvCell,
        skip: Option<usize>,
    ) -> u32 {
        let ray = params.current_ray;
        let mut best = params.delta;
        let length = ld(ray.length);
        let reach_z = st(ld(ray.dir.z) * length);
        let start = ray.pt;
        let end = Vector::new(
            st(ld(start.x) + ld(ray.dir.x) * length),
            st(ld(start.y) + ld(ray.dir.y) * length),
            st(ld(start.z) + ld(reach_z)),
        );
        let frame = &cell.position.frame;
        let local_end = frame.global_to_local(end);
        let local_start = frame.global_to_local(start);
        let local = Ray::from_offset(local_start, sub(local_end, local_start));
        let portals_only = matches!(
            params.intersect,
            RayIntersection::PortalsOnly | RayIntersection::UnconnectedPortalsOnly
        );
        // 1 portal, 2 environment, 3 object.
        let mut kind = 0;
        // The BSP writes its normal into a buffer whose z held `dir.z * length`.
        let mut local_normal = Vector::new(0.0, 0.0, reach_z);
        if !portals_only {
            if let Some(tree) = cell.structure.physics.as_deref() {
                // `UseExactDrawingRep` traces the cell's render mesh
                // (`CEnvCell::TraceRayAgainstMesh`), which physics does not hold.
                if params.intersect != RayIntersection::ExactDrawingRep {
                    let hit = tree.trace_ray_root(&local, &mut local_normal);
                    if let Some(t) = hit.filter(|&t| t < best) {
                        best = t;
                        params.delta = t;
                        kind = 2;
                    }
                }
            }
            if params.intersect == RayIntersection::PhysicsRep
                && self.trace_ray_objects(params, ObjCell::Env(cell)) != 0
            {
                best = params.delta;
                kind = 3;
            }
        }
        let mut portal = None;
        if params.traverse_portals {
            for (index, candidate) in cell.portals.iter().enumerate() {
                if skip == Some(index) {
                    continue;
                }
                let polygon = &candidate.polygon;
                let hit = trace_polygon(&polygon.vertices, &polygon.plane, &local);
                if let Some(t) = hit.filter(|&t| t < best) {
                    best = t;
                    kind = 1;
                    portal = Some(index);
                }
            }
        }
        let clear = |params: &mut LosParameters| {
            params.final_cell_id = cell.cell_id;
            params.final_ray = ray;
            params.impact_normal = Vector::ZERO;
            params.trace_result = 0;
            0
        };
        let normal = match (kind, portal) {
            (1, Some(index)) => {
                let through = &cell.portals[index];
                let unconnected = params.intersect == RayIntersection::UnconnectedPortalsOnly
                    && through.other_cell.is_some();
                if (!portals_only || unconnected) && params.traverse_portals {
                    let t = ld(best);
                    let cx = st((ld(end.x) - ld(start.x)) * t);
                    let entry = Vector::new(
                        st(ld(cx) + ld(start.x)),
                        st((ld(end.y) - ld(start.y)) * t + ld(start.y)),
                        st((ld(end.z) - ld(start.z)) * t + ld(start.z)),
                    );
                    let next = Ray::line_segment(entry, end);
                    params.current_ray = next;
                    params.delta = 1.0;
                    match through.other_cell {
                        None => {
                            let mut cell_id = cell.cell_id;
                            let mut origin = next.pt;
                            adjust_to_outside(&mut cell_id, &mut origin);
                            if let Some(outside) = ObjCell::get(self.cells, cell_id) {
                                return self.trace(params, outside);
                            }
                        }
                        Some(other) => {
                            if let Some(other) = self.cells.env_cell(other) {
                                return self.env_trace_ray(params, other, through.other_portal);
                            }
                        }
                    }
                }
                if !portals_only {
                    return clear(params);
                }
                params.portal_index = Some(index);
                params.delta = best;
                through.polygon.plane.n
            }
            (2, _) => frame.local_to_global_vec(local_normal),
            (3, _) => Vector::ZERO,
            _ => return clear(params),
        };
        params.final_cell_id = cell.cell_id;
        params.calc_final_ray();
        params.impact_normal = normal;
        params.trace_result = 1;
        1
    }
}
