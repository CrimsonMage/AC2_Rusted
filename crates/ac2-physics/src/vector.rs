//! Retail value types as the physics code stores them: `float` fields, with the arithmetic done by
//! each caller in `f64` and in retail operation order (see [`crate::fpu`]).

use crate::fpu::{epsilon_squared, ld, st, F_EPSILON};

/// Retail `Vector` (`x`, `y`, `z` floats, AC axes with z up).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// `x*x + y*y + z*z`.
    #[inline]
    pub(crate) fn mag2_xyz(self) -> f64 {
        ld(self.x) * ld(self.x) + ld(self.y) * ld(self.y) + ld(self.z) * ld(self.z)
    }

    /// `Vector::normalize_check_small` as `COLLISIONINFO::set_collision_normal` inlines it:
    /// `|v| = sqrt((x*x + y*y) + z*z)`; under `F_EPSILON` the vector becomes zero and the call
    /// reports failure, else each component is scaled by `1 / |v|` and stored.
    pub(crate) fn normalize_check_small(&mut self) -> bool {
        let length = self.mag2_xyz().sqrt();
        if length < ld(F_EPSILON) {
            *self = Self::ZERO;
            return false;
        }
        let inverse = 1.0 / length;
        self.x = st(inverse * ld(self.x));
        self.y = st(inverse * ld(self.y));
        self.z = st(inverse * ld(self.z));
        true
    }

    /// `D3DXVec3Normalize`. Retail dispatches this through a CPU-specific D3DX table (x87, 3DNow!
    /// or SSE); this is the exact-length x87 form: a zero vector stays zero.
    pub(crate) fn d3dx_normalize(self) -> Self {
        let length = self.mag2_xyz().sqrt();
        if length == 0.0 {
            return Self::ZERO;
        }
        Self::new(
            st(ld(self.x) / length),
            st(ld(self.y) / length),
            st(ld(self.z) / length),
        )
    }
}

impl From<ac2_math::Vec3> for Vector {
    fn from(value: ac2_math::Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<Vector> for ac2_math::Vec3 {
    fn from(value: Vector) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

/// Retail `Quaternion` (`w`, `x`, `y`, `z`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quaternion {
    pub const IDENTITY: Self = Self {
        w: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// `Quaternion::multiply` (`q1 * q2`), term order as the x87 code accumulates it.
    pub(crate) fn multiply(q1: Self, q2: Self) -> Self {
        let (aw, ax, ay, az) = (ld(q1.w), ld(q1.x), ld(q1.y), ld(q1.z));
        let (bw, bx, by, bz) = (ld(q2.w), ld(q2.x), ld(q2.y), ld(q2.z));
        Self {
            w: st(((aw * bw - bx * ax) - by * ay) - bz * az),
            x: st(((ax * bw + bx * aw) + bz * ay) - by * az),
            y: st(((by * aw - bz * ax) + ay * bw) + bx * az),
            z: st(((bz * aw + by * ax) - bx * ay) + az * bw),
        }
    }

    /// `((w*w + x*x) + y*y) + z*z`, the squared norm `Frame::set_rotate` and `Frame::grotate`
    /// test before normalising.
    pub(crate) fn norm2(self) -> f64 {
        ((ld(self.w) * ld(self.w) + ld(self.x) * ld(self.x)) + ld(self.y) * ld(self.y))
            + ld(self.z) * ld(self.z)
    }

    /// The renormalisation `Frame::set_rotate` and `Frame::grotate` share: when the squared norm
    /// exceeds `F_EPSILON^2` every component is scaled by `1 / sqrt(norm2)` and stored.
    pub(crate) fn normalize_if_large(&mut self) {
        let norm2 = self.norm2();
        if norm2 > epsilon_squared() {
            let inverse = 1.0 / norm2.sqrt();
            self.w = st(inverse * ld(self.w));
            self.x = st(inverse * ld(self.x));
            self.y = st(inverse * ld(self.y));
            self.z = st(inverse * ld(self.z));
        }
    }
}

/// Retail `Plane`: `N . p + d = 0`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Plane {
    pub n: Vector,
    pub d: f32,
}

impl Plane {
    pub const fn new(n: Vector, d: f32) -> Self {
        Self { n, d }
    }

    /// `Plane::snap_to_plane`: replaces `offset.z` so the offset lies in the plane through the
    /// origin, when the plane is not vertical.
    pub(crate) fn snap_to_plane(self, offset: &mut Vector) {
        if ld(self.n.z).abs() <= ld(F_EPSILON) {
            return;
        }
        offset.z = 0.0;
        let inverse = 1.0 / ld(self.n.z);
        let along = -(((ld(offset.y) * ld(self.n.y) + ld(offset.x) * ld(self.n.x))
            + ld(self.n.z) * 0.0)
            + ld(self.d))
            * inverse;
        let base = -ld(self.d) * inverse;
        offset.z = st(along - base);
    }
}

/// Retail `CSphere` (`center`, `radius`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sphere {
    pub center: Vector,
    pub radius: f32,
}

impl Sphere {
    pub const fn new(center: Vector, radius: f32) -> Self {
        Self { center, radius }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiply_composes_rotations_like_hamilton_product() {
        let half = std::f32::consts::FRAC_1_SQRT_2;
        let quarter_z = Quaternion::new(half, 0.0, 0.0, half);
        let half_z = Quaternion::multiply(quarter_z, quarter_z);
        assert!((half_z.w).abs() < 1e-6);
        assert!((half_z.z - 1.0).abs() < 1e-6);
        assert_eq!(
            Quaternion::multiply(Quaternion::IDENTITY, quarter_z),
            quarter_z
        );
    }

    #[test]
    fn snap_to_plane_keeps_offsets_in_a_slope() {
        let n = Vector::new(0.0, -0.6, 0.8);
        let plane = Plane::new(n, 3.0);
        let mut offset = Vector::new(1.0, 2.0, 5.0);
        plane.snap_to_plane(&mut offset);
        let along = n.x * offset.x + n.y * offset.y + n.z * offset.z;
        assert!(along.abs() < 1e-6, "{offset:?}");
        let mut vertical = Vector::new(1.0, 2.0, 5.0);
        Plane::new(Vector::new(1.0, 0.0, 0.0), 0.0).snap_to_plane(&mut vertical);
        assert_eq!(vertical.z, 5.0);
    }

    #[test]
    fn small_normals_collapse_to_zero() {
        let mut tiny = Vector::new(1e-5, 0.0, 0.0);
        assert!(!tiny.normalize_check_small());
        assert_eq!(tiny, Vector::ZERO);
        let mut unit = Vector::new(0.0, 3.0, 4.0);
        assert!(unit.normalize_check_small());
        assert!((unit.y - 0.6).abs() < 1e-7 && (unit.z - 0.8).abs() < 1e-7);
    }
}
