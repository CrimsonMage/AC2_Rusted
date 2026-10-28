//! `WorkerPolygon`: a convex polygon in world space with its plane, the portal polygons the
//! interior visibility cull clips against a [`crate::Frustum`] (`InitFromPolygon`,
//! `RecalculatePlane`, `CalcArea`, `Clip`).

use ac2_math::Vec3;

/// `WorkerPolygon::Clip`'s result (`Sidedness`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sidedness {
    Positive,
    Negative,
    Crossing,
    InPlane,
}

/// A plane as `WorkerPolygon::m_plane`: unit normal and offset.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolyPlane {
    pub normal: [f32; 3],
    pub d: f32,
}

/// `WorkerPolygon`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkerPolygon {
    pub verts: Vec<Vec3>,
    pub plane: PolyPlane,
}

/// `D3DXVec3Normalize`: a zero vector stays zero.
pub(crate) fn normalize(v: [f64; 3]) -> [f32; 3] {
    let length = ((v[0] * v[0] + v[1] * v[1]) + v[2] * v[2]).sqrt();
    if length == 0.0 {
        return [0.0; 3];
    }
    let inv = 1.0 / length;
    [
        (v[0] * inv) as f32,
        (v[1] * inv) as f32,
        (v[2] * inv) as f32,
    ]
}

fn sub(a: Vec3, b: Vec3) -> [f64; 3] {
    [
        f64::from(a.x) - f64::from(b.x),
        f64::from(a.y) - f64::from(b.y),
        f64::from(a.z) - f64::from(b.z),
    ]
}

pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

impl WorkerPolygon {
    /// `WorkerPolygon::InitFromPolygon`: the polygon's vertices (`vertices`, in the polygon's own
    /// space) turned into the world by `to_world`, with the plane recalculated.
    pub fn from_polygon(vertices: &[Vec3], to_world: impl Fn(Vec3) -> Vec3) -> Self {
        let mut polygon = Self {
            verts: vertices.iter().map(|&v| to_world(v)).collect(),
            plane: PolyPlane::default(),
        };
        polygon.recalculate_plane();
        polygon
    }

    /// `WorkerPolygon::RecalculatePlane`: the normal sums the triangle fan's cross products from
    /// the first vertex; the offset is the negated mean of the vertices' distances along it.
    pub fn recalculate_plane(&mut self) {
        let mut sum = [0.0_f64; 3];
        if self.verts.len() >= 3 {
            let origin = self.verts[0];
            for pair in self.verts[1..].windows(2) {
                let c = cross(sub(pair[0], origin), sub(pair[1], origin));
                for axis in 0..3 {
                    sum[axis] += f64::from(c[axis] as f32);
                }
            }
        }
        let n = normalize(sum);
        let mut total = 0.0_f64;
        for v in &self.verts {
            total += (f64::from(n[0]) * f64::from(v.x) + f64::from(n[1]) * f64::from(v.y))
                + f64::from(n[2]) * f64::from(v.z);
        }
        let count = self.verts.len() as f64;
        self.plane = PolyPlane {
            normal: n,
            d: ((-1.0 / count) * total) as f32,
        };
    }

    /// `WorkerPolygon::CalcArea`: the length of the summed edge cross terms.
    pub fn calc_area(&self) -> f32 {
        let count = self.verts.len();
        let (mut x, mut y, mut z) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (i, a) in self.verts.iter().enumerate() {
            let b = self.verts[(i + 1) % count];
            let half_x = f64::from(b.x) + f64::from(a.x);
            x += (f64::from(a.y) - f64::from(b.y)) * half_x * 0.5;
            y += (f64::from(b.y) + f64::from(a.y)) * (f64::from(a.z) - f64::from(b.z)) * 0.5;
            z += (f64::from(b.z) - f64::from(a.z)) * half_x * 0.5;
        }
        (((z * z + y * y) + x * x).sqrt()) as f32
    }

    /// `WorkerPolygon::Clip` keeping the positive side of `plane` (the `i != 0` form every
    /// frustum test uses). Vertices within `alpha` of the plane are on it. A polygon that only
    /// has vertices on or in front is `Positive` and unchanged, one with none in front is
    /// `InPlane` (all on it) or `Negative`; a crossing one is cut at the plane.
    pub fn clip(&mut self, plane: [f32; 3], d: f32, alpha: f32) -> Sidedness {
        let count = self.verts.len();
        let dist: Vec<f32> = self
            .verts
            .iter()
            .map(|v| {
                ((f64::from(plane[0]) * f64::from(v.x) + f64::from(v.y) * f64::from(plane[1]))
                    + f64::from(v.z) * f64::from(plane[2])
                    + f64::from(d)) as f32
            })
            .collect();
        // 0 in front, 1 behind, 2 on the plane.
        let sides: Vec<u8> = dist
            .iter()
            .map(|&x| {
                if x > alpha {
                    0
                } else if x >= -alpha {
                    2
                } else {
                    1
                }
            })
            .collect();
        let front = sides.iter().filter(|&&s| s == 0).count();
        let behind = sides.iter().filter(|&&s| s == 1).count();
        if front == 0 {
            return if behind == 0 {
                Sidedness::InPlane
            } else {
                Sidedness::Negative
            };
        }
        if behind == 0 {
            return Sidedness::Positive;
        }
        let mut kept = Vec::with_capacity(count + 2);
        for i in 0..count {
            let next = (i + 1) % count;
            if sides[i] == 2 {
                kept.push(self.verts[i]);
                continue;
            }
            if sides[i] == 0 {
                kept.push(self.verts[i]);
            }
            if sides[i] != sides[next] && sides[next] != 2 {
                let (a, b) = (self.verts[i], self.verts[next]);
                let t = f64::from(dist[i]) / (f64::from(dist[i]) - f64::from(dist[next]));
                let diff = [
                    (f64::from(b.x) - f64::from(a.x)) * t,
                    (f64::from(b.y) - f64::from(a.y)) * t,
                    (f64::from(b.z) - f64::from(a.z)) * t,
                ];
                kept.push(Vec3::new(
                    (diff[0] as f32) + a.x,
                    (diff[1] as f32) + a.y,
                    (diff[2] as f32) + a.z,
                ));
            }
        }
        self.verts = kept;
        Sidedness::Crossing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> WorkerPolygon {
        WorkerPolygon::from_polygon(
            &[
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(4.0, 0.0, 0.0),
                Vec3::new(4.0, 0.0, 4.0),
                Vec3::new(0.0, 0.0, 4.0),
            ],
            |v| v,
        )
    }

    #[test]
    fn the_plane_and_area_come_from_the_vertices() {
        let polygon = square();
        assert_eq!(polygon.plane.normal, [0.0, -1.0, 0.0]);
        assert!(polygon.plane.d.abs() < 1e-6);
        assert!((polygon.calc_area() - 16.0).abs() < 1e-4);
    }

    #[test]
    fn clipping_keeps_the_positive_side() {
        // x > 2 stays.
        let mut cut = square();
        assert_eq!(cut.clip([1.0, 0.0, 0.0], -2.0, 0.01), Sidedness::Crossing);
        assert!((cut.calc_area() - 8.0).abs() < 1e-4);
        assert!(cut.verts.iter().all(|v| v.x >= 2.0 - 1e-6));
        let mut whole = square();
        assert_eq!(whole.clip([1.0, 0.0, 0.0], 1.0, 0.01), Sidedness::Positive);
        assert_eq!(whole, square());
        let mut gone = square();
        assert_eq!(gone.clip([1.0, 0.0, 0.0], -9.0, 0.01), Sidedness::Negative);
        let mut flat = square();
        assert_eq!(flat.clip([0.0, 1.0, 0.0], 0.0, 0.01), Sidedness::InPlane);
    }

    #[test]
    fn a_vertex_on_the_plane_is_kept() {
        let mut triangle = WorkerPolygon::from_polygon(
            &[
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(6.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 4.0),
            ],
            |v| v,
        );
        assert_eq!(
            triangle.clip([1.0, 0.0, 0.0], -2.0, 0.01),
            Sidedness::Crossing
        );
        assert!(triangle.verts.contains(&Vec3::new(2.0, 0.0, 0.0)));
    }
}
