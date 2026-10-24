//! Retail `CPolygon` collision: the plane (`MakePlane`) and the sphere tests the physics BSP and
//! the transition's walkable checks run against one polygon. Edges run from the last vertex to
//! the first, then around in order; every expression follows the x87 order of its function.

use crate::fpu::{ld, st, F_EPSILON};
use crate::transition::sphere_path::SpherePath;
use crate::vector::{Plane, Sphere, Vector};

/// A collision polygon with its plane, in the space of its mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub vertices: Vec<Vector>,
    pub plane: Plane,
}

/// `CPolygon::MakePlane`: the fan of cross products `(v[i] - v0) x (v[i+1] - v0)` (two terms of
/// each stored as `float`) summed into the normal, `D3DXVec3Normalize`d, and `d` as minus the
/// mean of `N . v`.
pub(crate) fn make_plane(vertices: &[Vector]) -> Plane {
    let mut n = Vector::ZERO;
    if let Some(&v0) = vertices.first() {
        for pair in vertices[1..].windows(2) {
            let (vi, vj) = (pair[0], pair[1]);
            let ax = ld(vj.x) - ld(v0.x);
            let ay = ld(vj.y) - ld(v0.y);
            let az = ld(vj.z) - ld(v0.z);
            let bx = ld(st(ld(vi.x) - ld(v0.x)));
            let by = ld(st(ld(vi.y) - ld(v0.y)));
            let bz = ld(vi.z) - ld(v0.z);
            let tx = st(by * az - bz * ay);
            let ty = st(bz * ax - az * bx);
            let tz = ay * bx - by * ax;
            n = Vector::new(st(ld(tx) + ld(n.x)), st(ld(ty) + ld(n.y)), st(tz + ld(n.z)));
        }
    }
    let n = n.d3dx_normalize();
    let mut sum = 0.0;
    for v in vertices {
        sum += (ld(n.z) * ld(v.z) + ld(n.y) * ld(v.y)) + ld(n.x) * ld(v.x);
    }
    let count = vertices.len().max(1) as f64;
    Plane::new(n, st(-(sum / count)))
}

/// `CPolygon::point_in_polygon`: inside (or on) every edge, each edge's in-plane normal
/// `edge x N` with its first two terms stored. Edges run from the last vertex to the first.
pub(crate) fn point_in_polygon(vertices: &[Vector], plane: &Plane, p: Vector) -> bool {
    let count = vertices.len();
    let (nx, ny, nz) = (ld(plane.n.x), ld(plane.n.y), ld(plane.n.z));
    (0..count).all(|i| {
        let (prev, cur) = (vertices[(i + count - 1) % count], vertices[i]);
        let ex = ld(cur.x) - ld(prev.x);
        let ey = ld(cur.y) - ld(prev.y);
        let ez = ld(cur.z) - ld(prev.z);
        let a = ld(st(ez * ny - ey * nz));
        let b = ld(st(ex * nz - ez * nx));
        let c = ey * nx - ex * ny;
        let dx = ld(p.x) - ld(prev.x);
        let dy = ld(p.y) - ld(prev.y);
        let dz = ld(p.z) - ld(prev.z);
        (dz * c + dy * b) + a * dx >= 0.0
    })
}

impl Polygon {
    pub fn new(vertices: Vec<Vector>) -> Self {
        let plane = make_plane(&vertices);
        Self { vertices, plane }
    }

    fn n(&self) -> [f64; 3] {
        [ld(self.plane.n.x), ld(self.plane.n.y), ld(self.plane.n.z)]
    }

    /// Each edge as (previous vertex, current vertex).
    fn edges(&self) -> impl Iterator<Item = (Vector, Vector)> + '_ {
        let count = self.vertices.len();
        (0..count).map(move |i| (self.vertices[(i + count - 1) % count], self.vertices[i]))
    }

    /// `N . p + d` as the sphere tests evaluate it: `((Ny*y + Nz*z) + x*Nx) + d`.
    fn distance(&self, p: Vector) -> f64 {
        let [nx, ny, nz] = self.n();
        ((ny * ld(p.y) + nz * ld(p.z)) + ld(p.x) * nx) + ld(self.plane.d)
    }

    /// `N . v` as `pos_hits_sphere` and `walkable_hits_sphere` evaluate it.
    fn dot(&self, v: Vector) -> f64 {
        let [nx, ny, nz] = self.n();
        (nz * ld(v.z) + ny * ld(v.y)) + nx * ld(v.x)
    }

    /// The in-plane edge normal `edge x N` with its first two terms stored (`point_in_polygon`,
    /// `polygon_hits_sphere`'s first pass).
    fn edge_normal_stored(&self, prev: Vector, cur: Vector) -> (f64, f64, f64) {
        let [nx, ny, nz] = self.n();
        let ex = ld(cur.x) - ld(prev.x);
        let ey = ld(cur.y) - ld(prev.y);
        let ez = ld(cur.z) - ld(prev.z);
        let a = ld(st(ez * ny - ey * nz));
        let b = ld(st(ex * nz - ez * nx));
        let c = ey * nx - ex * ny;
        (a, b, c)
    }

    /// `CPolygon::point_in_polygon`.
    #[cfg(test)]
    pub(crate) fn point_in_polygon(&self, p: Vector) -> bool {
        point_in_polygon(&self.vertices, &self.plane, p)
    }

    /// The edge pass `polygon_hits_sphere_slow_but_sure`, `check_walkable` and
    /// `check_small_walkable` share once the centre projected onto the plane (`c`) lies outside
    /// an edge: a hit when the circle of squared radius `radius2` reaches an edge segment or a
    /// vertex. `outside_default` is the result when no edge decides (`check_walkable` returns 0
    /// once an outside edge failed to touch).
    fn circle_hits_edges(&self, c: Vector, radius2: f32, outside_default: bool) -> bool {
        let [nx, ny, nz] = self.n();
        let mut result = outside_default;
        for (prev, cur) in self.edges() {
            let ex = ld(st(ld(cur.x) - ld(prev.x)));
            let ey = ld(st(ld(cur.y) - ld(prev.y)));
            let ez = ld(st(ld(cur.z) - ld(prev.z)));
            let a = ez * ny - ey * nz;
            let b = ex * nz - ez * nx;
            let cc = ey * nx - ex * ny;
            let dx = ld(st(ld(c.x) - ld(prev.x)));
            let dy = ld(st(ld(c.y) - ld(prev.y)));
            let dz = ld(st(ld(c.z) - ld(prev.z)));
            let side = (dz * cc + dy * b) + a * dx;
            if side < 0.0 {
                if ((cc * cc + b * b) + a * a) * ld(radius2) < side * side {
                    return false;
                }
                let along = (dz * ez + dy * ey) + dx * ex;
                if along >= 0.0 && along <= (ez * ez + ey * ey) + ex * ex {
                    return true;
                }
                result = false;
            }
            if (dz * dz + dy * dy) + dx * dx <= ld(radius2) {
                return true;
            }
        }
        result
    }

    /// `CPolygon::polygon_hits_sphere_slow_but_sure`: the sphere (less `F_EPSILON`) reaches the
    /// polygon. `contact` receives the centre projected onto the plane.
    pub(crate) fn polygon_hits_sphere(&self, sphere: &Sphere, contact: &mut Vector) -> bool {
        let [nx, ny, nz] = self.n();
        let dist = self.distance(sphere.center);
        let reach = ld(sphere.radius) - ld(F_EPSILON);
        if !(reach >= dist.abs()) {
            return false;
        }
        let radius2 = st(reach * reach - dist * dist);
        let pz = ld(st(dist * nz));
        let c = Vector::new(
            st(ld(sphere.center.x) - dist * nx),
            st(ld(sphere.center.y) - dist * ny),
            st(ld(sphere.center.z) - pz),
        );
        *contact = c;
        let inside = self.edges().all(|(prev, cur)| {
            let (a, b, cc) = self.edge_normal_stored(prev, cur);
            let dx = ld(c.x) - ld(prev.x);
            let dy = ld(c.y) - ld(prev.y);
            let dz = ld(c.z) - ld(prev.z);
            (dz * cc + dy * b) + a * dx >= 0.0
        });
        inside || self.circle_hits_edges(c, radius2, false)
    }

    /// `CPolygon::hits_sphere`.
    pub(crate) fn hits_sphere(&self, sphere: &Sphere) -> bool {
        let mut contact = Vector::ZERO;
        self.polygon_hits_sphere(sphere, &mut contact)
    }

    /// `CPolygon::pos_hits_sphere`: a hit counts only while moving into the polygon's front;
    /// any hit is recorded as the struck polygon.
    pub(crate) fn pos_hits_sphere(
        &self,
        sphere: &Sphere,
        movement: Vector,
        contact: &mut Vector,
        struck: &mut bool,
    ) -> bool {
        let hit = self.polygon_hits_sphere(sphere, contact);
        if hit {
            *struck = true;
        }
        self.dot(movement) < 0.0 && hit
    }

    /// `CPolygon::walkable_hits_sphere`: a polygon facing `up` more than the walkable allowance
    /// that the sphere reaches.
    pub(crate) fn walkable_hits_sphere(
        &self,
        path: &SpherePath,
        sphere: &Sphere,
        up: Vector,
    ) -> bool {
        if !(self.dot(up) > ld(path.walkable_allowance)) {
            return false;
        }
        self.hits_sphere(sphere)
    }

    /// The centre moved along `up` onto the plane (`check_walkable`, `find_crossed_edge`).
    fn project_along(&self, sphere: &Sphere, up: Vector) -> Vector {
        let [nx, ny, nz] = self.n();
        let num = self.distance(sphere.center);
        let den = (ny * ld(up.y) + nz * ld(up.z)) + ld(up.x) * nx;
        let t = num / den;
        let ty = ld(st(t * ld(up.y)));
        let tz = ld(st(t * ld(up.z)));
        Vector::new(
            st(ld(sphere.center.x) - t * ld(up.x)),
            st(ld(sphere.center.y) - ty),
            st(ld(sphere.center.z) - tz),
        )
    }

    /// `CPolygon::check_walkable` (`small`: `check_small_walkable`, a quarter of the squared
    /// radius): the sphere's centre, dropped along `up` onto the plane, is over the polygon.
    pub(crate) fn check_walkable(&self, sphere: &Sphere, up: Vector, small: bool) -> bool {
        let c = self.project_along(sphere, up);
        let r = ld(sphere.radius);
        let radius2 = if small {
            st(r * r * ld(0.25_f32))
        } else {
            st(r * r)
        };
        self.circle_hits_edges(c, radius2, true)
    }

    /// `CPolygon::find_crossed_edge`: for the first edge the dropped centre is outside of, the
    /// in-plane normal `N x edge` (pointing into the polygon), normalised.
    pub(crate) fn find_crossed_edge(&self, sphere: &Sphere, up: Vector) -> Option<Vector> {
        let [nx, ny, nz] = self.n();
        let c = self.project_along(sphere, up);
        for (prev, cur) in self.edges() {
            let ex = ld(st(ld(cur.x) - ld(prev.x)));
            let ey = ld(st(ld(cur.y) - ld(prev.y)));
            let ez_unrounded = ld(cur.z) - ld(prev.z);
            let ez = ld(st(ez_unrounded));
            let a = ez_unrounded * ny - ey * nz;
            let b = ex * nz - ez * nx;
            let cc = ey * nx - ex * ny;
            let dx = ld(c.x) - ld(prev.x);
            let dy = ld(c.y) - ld(prev.y);
            let dz = ld(c.z) - ld(prev.z);
            if (dz * cc + dy * b) + a * dx < 0.0 {
                let a = ez * ny - ey * nz;
                return Some(Vector::new(st(a), st(b), st(cc)).d3dx_normalize());
            }
        }
        None
    }

    /// `CPolygon::adjust_sphere_to_plane`: move `sphere` back along `movement` until it touches
    /// the plane, when that shortens the path's walk interpolation (but not below -0.5).
    pub(crate) fn adjust_sphere_to_plane(
        &self,
        path: &mut SpherePath,
        sphere: &mut Sphere,
        movement: Vector,
    ) -> bool {
        let dist = self.distance(sphere.center);
        let dp = self.dot_movement(movement);
        let num = if dp > 0.0 {
            -ld(sphere.radius) - dist
        } else if dp == 0.0 || dp.is_nan() {
            return false;
        } else {
            dist - ld(sphere.radius)
        };
        let t = num / dp;
        let interp = (1.0 - t) * ld(path.walk_interp);
        let stored = st(interp);
        if !(interp < ld(path.walk_interp)) || ld(stored) < -0.5 {
            return false;
        }
        let ty = ld(st(t * ld(movement.y)));
        let tz = ld(st(t * ld(movement.z)));
        sphere.center = Vector::new(
            st(ld(sphere.center.x) - t * ld(movement.x)),
            st(ld(sphere.center.y) - ty),
            st(ld(sphere.center.z) - tz),
        );
        path.walk_interp = stored;
        true
    }

    /// `N . m` in `adjust_sphere_to_plane`'s order.
    fn dot_movement(&self, m: Vector) -> f64 {
        let [nx, ny, nz] = self.n();
        (nz * ld(m.z) + ny * ld(m.y)) + nx * ld(m.x)
    }

    /// `CPolygon::adjust_sphere_to_poly`: the fraction of `movement` that brings a sphere at
    /// `center` to touch the plane (1 when already within its radius, 0 when moving parallel).
    pub(crate) fn adjust_sphere_to_poly(
        &self,
        sphere: &Sphere,
        center: Vector,
        movement: Vector,
    ) -> f64 {
        let [nx, ny, nz] = self.n();
        let dist =
            st(((nz * ld(center.z) + ny * ld(center.y)) + ld(center.x) * nx) + ld(self.plane.d));
        if ld(dist).abs() < ld(sphere.radius) {
            return 1.0;
        }
        let dp = (nz * ld(movement.z) + ny * ld(movement.y)) + ld(movement.x) * nx;
        if dp == 0.0 {
            return 0.0;
        }
        let r = if ld(dist) <= 0.0 {
            -ld(sphere.radius)
        } else {
            ld(sphere.radius)
        };
        (r - ld(dist)) / dp
    }

    /// `CPolygon::adjust_to_placement_poly`: push both spheres out of the plane to `radius` from
    /// it (on the side `center_solid`/`solid_check` choose).
    pub(crate) fn adjust_to_placement_poly(
        &self,
        struck: &mut Sphere,
        other: &mut Sphere,
        radius: f32,
        center_solid: bool,
        solid_check: bool,
    ) {
        let [nx, ny, nz] = self.n();
        let dist = self.distance(struck.center);
        let amount = if solid_check {
            let mut r = ld(radius);
            if center_solid {
                r *= -1.0;
            }
            if dist <= 0.0 {
                r = -r;
            }
            r - dist
        } else {
            ld(radius) - dist
        };
        let px = amount * nx;
        let py = ld(st(amount * ny));
        let pz = ld(st(amount * nz));
        for sphere in [struck, other] {
            let c = &mut sphere.center;
            *c = Vector::new(st(px + ld(c.x)), st(py + ld(c.y)), st(pz + ld(c.z)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> Polygon {
        Polygon::new(vec![
            Vector::new(0.0, 0.0, 0.0),
            Vector::new(4.0, 0.0, 0.0),
            Vector::new(4.0, 4.0, 0.0),
            Vector::new(0.0, 4.0, 0.0),
        ])
    }

    #[test]
    fn quads_get_unit_planes() {
        let p = floor();
        assert_eq!(p.plane, Plane::new(Vector::new(0.0, 0.0, 1.0), 0.0));
        assert!(p.point_in_polygon(Vector::new(2.0, 2.0, 0.0)));
        assert!(!p.point_in_polygon(Vector::new(5.0, 2.0, 0.0)));
    }

    #[test]
    fn sphere_hits_face_edge_and_corner() {
        let p = floor();
        let mut contact = Vector::ZERO;
        let s = |x, y, z| Sphere::new(Vector::new(x, y, z), 0.5);
        assert!(p.polygon_hits_sphere(&s(2.0, 2.0, 0.4), &mut contact));
        assert_eq!(contact, Vector::new(2.0, 2.0, 0.0));
        assert!(!p.polygon_hits_sphere(&s(2.0, 2.0, 0.6), &mut contact));
        assert!(p.polygon_hits_sphere(&s(4.3, 2.0, 0.0), &mut contact));
        assert!(!p.polygon_hits_sphere(&s(4.6, 2.0, 0.0), &mut contact));
        assert!(p.polygon_hits_sphere(&s(4.2, 4.2, 0.0), &mut contact));
        assert!(!p.polygon_hits_sphere(&s(4.4, 4.4, 0.0), &mut contact));
    }

    #[test]
    fn moving_into_the_back_does_not_count() {
        let p = floor();
        let (mut contact, mut struck) = (Vector::ZERO, false);
        let s = Sphere::new(Vector::new(2.0, 2.0, 0.3), 0.5);
        assert!(p.pos_hits_sphere(&s, Vector::new(0.0, 0.0, -1.0), &mut contact, &mut struck));
        struck = false;
        assert!(!p.pos_hits_sphere(&s, Vector::new(0.0, 0.0, 1.0), &mut contact, &mut struck));
        assert!(struck);
    }

    #[test]
    fn walkable_checks_and_crossed_edges() {
        let p = floor();
        let up = Vector::new(0.0, 0.0, 1.0);
        let s = |x| Sphere::new(Vector::new(x, 2.0, 1.0), 0.5);
        assert!(p.check_walkable(&s(3.0), up, false));
        assert!(p.check_walkable(&s(4.3), up, false));
        assert!(!p.check_walkable(&s(4.3), up, true));
        let edge = p
            .find_crossed_edge(&s(4.3), up)
            .expect("outside the +x edge");
        assert!(
            (edge.x + 1.0).abs() < 1e-6 && edge.y.abs() < 1e-6,
            "{edge:?}"
        );
        assert!(p.find_crossed_edge(&s(3.0), up).is_none());
    }

    #[test]
    fn spheres_move_back_onto_the_plane() {
        let p = floor();
        let mut path = SpherePath {
            walk_interp: 1.0,
            ..SpherePath::default()
        };
        let mut s = Sphere::new(Vector::new(2.0, 2.0, 0.2), 0.5);
        assert!(p.adjust_sphere_to_plane(&mut path, &mut s, Vector::new(0.0, 0.0, -1.0)));
        assert!((s.center.z - 0.5).abs() < 1e-6, "{s:?}");
        assert!((path.walk_interp - 0.7).abs() < 1e-6);
        let t =
            p.adjust_sphere_to_poly(&s, Vector::new(2.0, 2.0, 2.0), Vector::new(0.0, 0.0, -1.0));
        assert!((t - 1.5).abs() < 1e-6, "{t}");
    }
}
