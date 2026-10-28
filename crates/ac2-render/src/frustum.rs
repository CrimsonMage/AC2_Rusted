//! `Frustum`: the four side planes of the camera's view and its near plane (no far plane), as
//! `SceneTool::SetupPerspective` builds it (`m_fCameraHorizFOV = aspect * vertical FOV degrees`)
//! and `Frustum::CheckRelationship` tests boxes against it.

use ac2_math::Vec3;

use crate::worker_polygon::{cross, normalize, Sidedness, WorkerPolygon};
use crate::Camera;

/// `Frustum::CheckRelationship` treats a plane distance under this as outside.
const PLANE_EPSILON: f64 = 0.0002;
/// Degrees to the half-angle in radians (`Generate`'s `0.008726646`).
const HALF_DEGREE: f64 = 0.008726646;

/// `BoundingType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Outside,
    Partial,
    Inside,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plane {
    normal: [f32; 3],
    d: f32,
}

impl Plane {
    fn distance(&self, p: [f64; 3]) -> f64 {
        let n = self.normal.map(f64::from);
        ((p[0] * n[0] + p[1] * n[1]) + p[2] * n[2]) + f64::from(self.d)
    }
}

/// `Frustum::m_aPlanes` holds 32 planes; a portal frustum is one per polygon edge plus the near
/// plane, so a polygon of 32 or more edges does not generate one.
pub const MAX_PLANES: usize = 33;

/// The camera's view volume: the side planes then the near plane (`m_nPlanes` of them), and the
/// camera they were made from (`m_matCamera`: its origin and forward row).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frustum {
    planes: [Plane; MAX_PLANES],
    count: usize,
    origin: [f32; 3],
    forward: [f32; 3],
    near: f32,
}

impl Frustum {
    /// `Frustum::Generate(camera, horizontal FOV, vertical FOV, near)`: camera-space inward
    /// normals of the left, right, top and bottom planes turned into the world by the camera
    /// basis (`right`, `forward`, `up`, AC axes) and through `origin`, then the near plane along
    /// `forward` at `near`. FOVs are full angles in degrees.
    pub fn generate(
        origin: Vec3,
        [right, forward, up]: [Vec3; 3],
        horizontal_degrees: f32,
        vertical_degrees: f32,
        near: f32,
    ) -> Self {
        let half = |degrees: f32| f64::from(degrees) * HALF_DEGREE;
        let (sh, ch) = (
            half(horizontal_degrees).sin(),
            half(horizontal_degrees).cos(),
        );
        let (sv, cv) = (half(vertical_degrees).sin(), half(vertical_degrees).cos());
        let st = |v: f64| v as f32;
        // Camera-space normals, each stored as a `float`.
        let local = [
            [st(ch), st(sh), 0.0],
            [st(-ch), st(sh), 0.0],
            [0.0, st(cv), st(-sv)],
            [0.0, st(cv), st(sv)],
            [0.0, 1.0, 0.0],
        ];
        let o = origin.to_array().map(f64::from);
        let axes = [right, forward, up].map(|axis| axis.to_array().map(f64::from));
        let mut planes = [Plane {
            normal: [0.0; 3],
            d: 0.0,
        }; MAX_PLANES];
        for (index, n) in local.iter().enumerate() {
            let n = n.map(f64::from);
            // The camera matrix's last column is (0, 0, 0, 1), so the homogeneous divide is 1.
            let world =
                [0, 1, 2].map(|c| st((n[1] * axes[1][c] + n[2] * axes[2][c]) + axes[0][c] * n[0]));
            let w = world.map(f64::from);
            let at = if index == 4 {
                let near = f64::from(near);
                [0, 1, 2].map(|c| near * w[c] + o[c])
            } else {
                o
            };
            planes[index] = Plane {
                normal: world,
                d: st(-((w[0] * at[0] + w[1] * at[1]) + w[2] * at[2])),
            };
        }
        Self {
            planes,
            count: 5,
            origin: origin.to_array(),
            forward: forward.to_array(),
            near,
        }
    }

    /// The frustum of a look-at camera at `aspect` (`SceneTool::SetupPerspective`).
    pub fn from_camera(camera: &Camera, aspect: f32) -> Self {
        let forward = (camera.target - camera.eye).normalize_or_zero();
        let right = forward.cross(Vec3::Z).normalize_or_zero();
        let up = right.cross(forward);
        let vertical = camera.fov_y.to_degrees();
        Self::generate(
            camera.eye,
            [right, forward, up],
            aspect * vertical,
            vertical,
            camera.near,
        )
    }

    /// The camera position the frustum was made at (`m_matCamera`'s translation row).
    pub fn origin(&self) -> Vec3 {
        Vec3::from_array(self.origin)
    }

    /// `Frustum::Generate(camera matrix, polygon, near)`: the planes through the camera and each
    /// edge of the portal polygon, then the near plane. `None` for a polygon of 32 or more
    /// vertices, where retail generates nothing and the caller keeps its own frustum.
    pub fn from_portal(&self, polygon: &WorkerPolygon) -> Option<Self> {
        let count = polygon.verts.len();
        if count >= MAX_PLANES - 1 {
            return None;
        }
        let o = self.origin.map(f64::from);
        let mut planes = [Plane {
            normal: [0.0; 3],
            d: 0.0,
        }; MAX_PLANES];
        for (i, plane) in planes.iter_mut().enumerate().take(count) {
            let (a, b) = (polygon.verts[i], polygon.verts[(i + 1) % count]);
            let ra = [
                f64::from(a.x) - o[0],
                f64::from(a.y) - o[1],
                f64::from(a.z) - o[2],
            ];
            let rb = [
                f64::from(b.x) - o[0],
                f64::from(b.y) - o[1],
                f64::from(b.z) - o[2],
            ];
            let n = normalize(cross(ra, rb));
            let n64 = n.map(f64::from);
            *plane = Plane {
                normal: n,
                d: (-((n64[0] * o[0] + n64[1] * o[1]) + n64[2] * o[2])) as f32,
            };
        }
        let f = self.forward.map(f64::from);
        let near = f64::from(self.near);
        planes[count] = Plane {
            normal: self.forward,
            d: (-((near * f[1] + o[1]) * f[1]
                + (near * f[0] + o[0]) * f[0]
                + f[2] * (near * f[2] + o[2]))) as f32,
        };
        Some(Self {
            planes,
            count: count + 1,
            origin: self.origin,
            forward: self.forward,
            near: self.near,
        })
    }

    /// `Frustum::CheckPolygon`: the polygon clipped to the frustum's side planes (and the near
    /// plane when `near_check`), or `None` when a plane leaves nothing of it. A camera within
    /// 0.01 of the polygon's plane sees it whole.
    pub fn check_polygon(
        &self,
        polygon: &WorkerPolygon,
        near_check: bool,
    ) -> Option<WorkerPolygon> {
        let mut clipped = polygon.clone();
        let n = polygon.plane.normal.map(f64::from);
        let o = self.origin.map(f64::from);
        let distance =
            (((o[0] * n[0] + o[1] * n[1]) + o[2] * n[2]) + f64::from(polygon.plane.d)) as f32;
        if distance <= 0.01 && distance >= -0.01 {
            return Some(clipped);
        }
        let planes = if near_check {
            self.count
        } else {
            self.count - 1
        };
        for plane in &self.planes[..planes] {
            if clipped.clip(plane.normal, plane.d, 0.01) == Sidedness::Negative {
                return None;
            }
        }
        Some(clipped)
    }

    /// `Frustum::CheckRelationship` of the box `min..max`: outside when its corner furthest
    /// along any plane's normal is behind it, inside when its nearest corner is in front of
    /// every plane.
    pub fn check(&self, min: Vec3, max: Vec3) -> Relation {
        let (min, max) = (min.to_array(), max.to_array());
        let mut inside = true;
        for plane in &self.planes[..self.count] {
            let (mut near, mut far) = ([0.0_f64; 3], [0.0_f64; 3]);
            for axis in 0..3 {
                let (lo, hi) = (f64::from(min[axis]), f64::from(max[axis]));
                if plane.normal[axis] >= 0.0 {
                    (near[axis], far[axis]) = (lo, hi);
                } else {
                    (near[axis], far[axis]) = (hi, lo);
                }
            }
            if plane.distance(far) < -PLANE_EPSILON {
                return Relation::Outside;
            }
            if inside && plane.distance(near) < -PLANE_EPSILON {
                inside = false;
            }
        }
        if inside {
            Relation::Inside
        } else {
            Relation::Partial
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxes_ahead_beside_and_behind_the_camera() {
        let camera = Camera {
            eye: Vec3::ZERO,
            target: Vec3::Y,
            fov_y: 60_f32.to_radians(),
            near: 1.0,
            far: 1000.0,
        };
        let frustum = Frustum::from_camera(&camera, 1.0);
        let at = |c: Vec3| frustum.check(c - Vec3::splat(1.0), c + Vec3::splat(1.0));
        assert_eq!(at(Vec3::new(0.0, 20.0, 0.0)), Relation::Inside);
        assert_eq!(at(Vec3::new(0.0, -20.0, 0.0)), Relation::Outside);
        // 60 degrees wide: 45 degrees off the axis is outside, the edge is partial.
        assert_eq!(at(Vec3::new(20.0, 20.0, 0.0)), Relation::Outside);
        assert_eq!(at(Vec3::new(20.0 * 0.577, 20.0, 0.0)), Relation::Partial);
        // Straddling the near plane.
        assert_eq!(at(Vec3::new(0.0, 1.0, 0.0)), Relation::Partial);
        // No far plane: a box past the camera's far distance still draws.
        assert_eq!(at(Vec3::new(0.0, 5000.0, 0.0)), Relation::Inside);
    }
}
