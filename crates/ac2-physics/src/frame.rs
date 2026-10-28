//! Retail `Frame`: an origin, a rotation quaternion and the cached rotation matrix `fl2gv`
//! (`Frame::cache_matrix`). `m[0..3]`, `m[3..6]` and `m[6..9]` are the local x, y and z axes in
//! global coordinates. Every transform here follows the x87 operation order of the retail
//! function it names; the matrix sums are not interchangeable between functions.

use crate::fpu::{epsilon_squared, ld, st};
use crate::heading::Heading;
use crate::vector::{Quaternion, Vector};

/// `5*pi/2` as the double `Frame::get_heading` subtracts the `fpatan` result from.
pub(crate) const FIVE_HALVES_PI: f64 = 7.853_981_633_974_483;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub origin: Vector,
    pub q: Quaternion,
    pub m: [f32; 9],
}

impl Default for Frame {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Frame {
    pub const IDENTITY: Self = Self {
        origin: Vector::ZERO,
        q: Quaternion::IDENTITY,
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// A frame at `origin` turned by `q` (`Frame::set_rotate`, which renormalises).
    pub fn new(origin: Vector, q: Quaternion) -> Self {
        let mut frame = Self {
            origin,
            ..Self::IDENTITY
        };
        frame.set_rotate(q);
        frame
    }

    /// `Frame::cache_matrix` through `Quaternion::InqRotationAxes`. The doubled `z` and the
    /// `w`/`x` cross terms are stored as `float`; `yy`, `yz` and `zz` stay on the x87 stack.
    pub(crate) fn cache_matrix(&mut self) {
        let (w, x, y, z) = (ld(self.q.w), ld(self.q.x), ld(self.q.y), ld(self.q.z));
        let x2 = x + x;
        let y2 = y + y;
        let z2 = ld(st(z + z));
        let wx = ld(st(x2 * w));
        let wy = ld(st(y2 * w));
        let wz = ld(st(z2 * w));
        let xx = ld(st(x2 * x));
        let xy = ld(st(y2 * x));
        let xz = ld(st(z2 * x));
        let yy = y2 * y;
        let yz = z2 * y;
        let zz = z2 * z;
        self.m = [
            st(1.0 - (zz + yy)),
            st(xy + wz),
            st(xz - wy),
            st(xy - wz),
            st(1.0 - (zz + xx)),
            st(wx + yz),
            st(xz + wy),
            st(yz - wx),
            st(1.0 - (yy + xx)),
        ];
    }

    /// `Frame::set_rotate`: stores `q`, renormalises it when its squared norm exceeds
    /// `F_EPSILON^2`, and recaches the matrix.
    pub(crate) fn set_rotate(&mut self, q: Quaternion) {
        self.q = q;
        self.q.normalize_if_large();
        self.cache_matrix();
    }

    /// `Frame::grotate`: turns the frame by the global rotation vector `rotation` (axis times
    /// angle in radians) with `Quaternion::rotate`, then renormalises and recaches like
    /// `set_rotate`. A rotation under `F_EPSILON` leaves the frame untouched.
    pub(crate) fn grotate(&mut self, rotation: Vector) {
        let (rx, ry, rz) = (ld(rotation.x), ld(rotation.y), ld(rotation.z));
        let angle2 = (rx * rx + ry * ry) + rz * rz;
        if angle2 < epsilon_squared() {
            return;
        }
        let angle = angle2.sqrt();
        let inverse = 1.0 / angle;
        let half = angle * 0.5;
        let sine = ld(st(half.sin()));
        let cosine = st(half.cos());
        let turn = Quaternion::new(
            cosine,
            st(sine * rx * inverse),
            st(sine * inverse * ry),
            st(sine * inverse * rz),
        );
        self.q = Quaternion::multiply(turn, self.q);
        self.q.normalize_if_large();
        self.cache_matrix();
    }

    /// `Frame::combine(f1, f2)`: the frame `f2` (an `AFrame`: origin and rotation) placed in
    /// `f1`. The origin is `f1`'s origin plus `f2`'s origin rotated by `f1`'s matrix, each
    /// component summed on the x87 stack (`(m6 z + m0 x) + m3 y`, then the origin) and stored as
    /// `float`; the rotation is `Quaternion::multiply(f1.q, f2.q)` through `set_rotate`.
    pub fn combine(f1: &Frame, origin: Vector, q: Quaternion) -> Frame {
        let m = f1.m.map(ld);
        let (x, y, z) = (ld(origin.x), ld(origin.y), ld(origin.z));
        Self::combined(f1, x, y, z, m, q)
    }

    /// `Frame::combine(f1, f2, scale)`: as [`Frame::combine`] with `f2`'s origin scaled first. The
    /// x product stays on the x87 stack; the y and z products are stored as `float`.
    pub fn combine_scaled(f1: &Frame, origin: Vector, q: Quaternion, scale: Vector) -> Frame {
        let m = f1.m.map(ld);
        let sx = ld(origin.x) * ld(scale.x);
        let sy = ld(st(ld(origin.y) * ld(scale.y)));
        let sz = ld(st(ld(origin.z) * ld(scale.z)));
        Self::combined(f1, sx, sy, sz, m, q)
    }

    fn combined(f1: &Frame, x: f64, y: f64, z: f64, m: [f64; 9], q: Quaternion) -> Frame {
        let o = f1.origin;
        let origin = Vector::new(
            st(((x * m[0] + z * m[6]) + y * m[3]) + ld(o.x)),
            st(((x * m[1] + z * m[7]) + y * m[4]) + ld(o.y)),
            st(((x * m[2] + z * m[8]) + y * m[5]) + ld(o.z)),
        );
        let mut frame = Frame {
            origin,
            ..Frame::IDENTITY
        };
        frame.set_rotate(Quaternion::multiply(f1.q, q));
        frame
    }

    /// `Frame::localtoglobalvec`.
    pub fn local_to_global_vec(&self, v: Vector) -> Vector {
        let m = self.m.map(ld);
        let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
        Vector::new(
            st((m[6] * z + m[0] * x) + m[3] * y),
            st((m[7] * z + m[1] * x) + m[4] * y),
            st((m[8] * z + m[2] * x) + m[5] * y),
        )
    }

    /// `Frame::globaltolocalvec`.
    pub fn global_to_local_vec(&self, v: Vector) -> Vector {
        let m = self.m.map(ld);
        let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
        Vector::new(
            st((m[2] * z + m[0] * x) + m[1] * y),
            st((m[5] * z + m[3] * x) + m[4] * y),
            st((m[8] * z + m[6] * x) + m[7] * y),
        )
    }

    /// `Frame::globaltolocal` (a point): the offset from the origin is stored as `float` first.
    pub fn global_to_local(&self, v: Vector) -> Vector {
        let m = self.m.map(ld);
        let dx = ld(st(ld(v.x) - ld(self.origin.x)));
        let dy = ld(st(ld(v.y) - ld(self.origin.y)));
        let dz = ld(st(ld(v.z) - ld(self.origin.z)));
        Vector::new(
            st((dx * m[0] + dz * m[2]) + dy * m[1]),
            st((dx * m[3] + dz * m[5]) + dy * m[4]),
            st((dx * m[6] + dz * m[8]) + dy * m[7]),
        )
    }

    /// `Frame::get_heading`: the compass heading of the local y axis.
    pub fn heading(&self) -> Heading {
        let angle = ld(self.m[4]).atan2(ld(self.m[3]));
        Heading::from_radians(st(FIVE_HALVES_PI - angle))
    }

    /// `Frame::get_vector_heading`: the local y axis (forward) in global coordinates.
    pub fn vector_heading(&self) -> Vector {
        Vector::new(self.m[3], self.m[4], self.m[5])
    }

    /// `Frame::set_heading`: face the compass heading, keeping the forward axis' z
    /// (`set_vector_heading((sin, cos, m[5]))`).
    pub fn set_heading(&mut self, heading: Heading) {
        let z = self.m[5];
        let x = st(heading.radians().sin());
        let y = st(heading.radians().cos());
        self.set_vector_heading(Vector::new(x, y, z));
    }

    /// `Frame::set_vector_heading`: rebuild the rotation with `heading` as the forward axis and
    /// up as close to +z as it allows (+y when facing straight up), via
    /// `Quaternion::SetRotationAxes`. A degenerate heading leaves the frame unchanged, a zero one
    /// resets the rotation.
    pub fn set_vector_heading(&mut self, heading: Vector) {
        let (hx, hy, hz) = (ld(heading.x), ld(heading.y), ld(heading.z));
        if (hx * hx + hy * hy) + hz * hz < epsilon_squared() {
            self.set_rotate(Quaternion::IDENTITY);
            return;
        }
        let length = ((hz * hz + hx * hx) + hy * hy).sqrt();
        if length < ld(crate::fpu::F_EPSILON) {
            return;
        }
        let inverse = 1.0 / length;
        let fx = ld(st(inverse * hx));
        let fy = ld(st(hy * inverse));
        let fz = ld(st(hz * inverse));
        let d = (fy * 0.0 + fx * 0.0) + fz;
        let mut up = (
            ld(st(-(fx * d))),
            ld(st(-ld(st(fy * d)))),
            ld(st(1.0 - ld(st(fz * d)))),
        );
        let up_length = ((up.2 * up.2 + up.1 * up.1) + up.0 * up.0).sqrt();
        let unit = if up_length >= ld(crate::fpu::F_EPSILON) {
            // The length is stored before the divide; the x term uses the unrounded inverse.
            let inverse = 1.0 / ld(st(up_length));
            let stored = ld(st(inverse));
            (inverse * up.0, up.1 * stored, up.2 * stored)
        } else {
            let d = (fz * 0.0 + fx * 0.0) + fy;
            up = (
                -(fx * d),
                ld(st(1.0 - ld(st(fy * d)))),
                ld(st(-ld(st(fz * d)))),
            );
            let up_length = ((up.2 * up.2 + up.1 * up.1) + up.0 * up.0).sqrt();
            if up_length < ld(crate::fpu::F_EPSILON) {
                return;
            }
            let stored = ld(st(1.0 / up_length));
            (up.0 * stored, up.1 * stored, up.2 * stored)
        };
        let (ux, uy) = (unit.0, unit.1);
        let uz = ld(st(unit.2));
        self.m = [
            st(uz * fy - uy * fz),
            st(fz * ux - uz * fx),
            st(uy * fx - ux * fy),
            st(fx),
            st(fy),
            st(fz),
            st(ux),
            st(uy),
            st(uz),
        ];
        self.cache_quaternion();
    }

    /// `Frame::cache_quaternion` (`Quaternion::SetRotationAxes` of the cached axes).
    fn cache_quaternion(&mut self) {
        let m = self.m.map(ld);
        let (x, y, z) = ([m[0], m[1], m[2]], [m[3], m[4], m[5]], [m[6], m[7], m[8]]);
        let yz_trace = ld(st(z[2] + y[1]));
        let trace = (z[2] + y[1]) + x[0];
        self.q = if trace >= 0.0 {
            let s = (trace + 1.0).sqrt();
            let r = 0.5 / s;
            Quaternion::new(
                st(0.5 * s),
                st((y[2] - z[1]) * r),
                st((z[0] - x[2]) * r),
                st((x[1] - y[0]) * r),
            )
        } else {
            let mut largest = if x[0] < y[1] { y[1] } else { x[0] };
            if largest < z[2] {
                largest = z[2];
            }
            if largest == x[0] {
                let s = ((x[0] - yz_trace) + 1.0).sqrt();
                let r = 0.5 / s;
                Quaternion::new(
                    st((y[2] - z[1]) * r),
                    st(0.5 * s),
                    st((x[1] + y[0]) * r),
                    st((x[2] + z[0]) * r),
                )
            } else if largest == y[1] {
                let s = ((y[1] - (x[0] + z[2])) + 1.0).sqrt();
                let r = 0.5 / s;
                Quaternion::new(
                    st((z[0] - x[2]) * r),
                    st((x[1] + y[0]) * r),
                    st(0.5 * s),
                    st((y[2] + z[1]) * r),
                )
            } else {
                let s = ((z[2] - (x[0] + y[1])) + 1.0).sqrt();
                let r = 0.5 / s;
                Quaternion::new(
                    st((x[1] - y[0]) * r),
                    st((x[2] + z[0]) * r),
                    st((y[2] + z[1]) * r),
                    st(0.5 * s),
                )
            }
        };
    }

    /// `AFrame::operator==`: origin and quaternion components within `F_EPSILON`.
    pub(crate) fn approx_eq(&self, other: &Self) -> bool {
        let near = |a: f32, b: f32| (ld(a) - ld(b)).abs() < ld(crate::fpu::F_EPSILON);
        near(self.origin.x, other.origin.x)
            && near(self.origin.y, other.origin.y)
            && near(self.origin.z, other.origin.z)
            && near(self.q.w, other.q.w)
            && near(self.q.x, other.q.x)
            && near(self.q.y, other.q.y)
            && near(self.q.z, other.q.z)
    }

    /// `Frame::slerp_rotation`: the rotation `t` of the way from `from` to `to` by
    /// `D3DXQuaternionSlerp` (the x87 `c_` path the statically linked D3DX takes without
    /// 3DNow!), stored through `set_rotate`.
    pub fn slerp_rotation(&mut self, from: &Frame, to: &Frame, t: f32) {
        let (w1, x1, y1, z1) = (ld(from.q.w), ld(from.q.x), ld(from.q.y), ld(from.q.z));
        let (w2, x2, y2, z2) = (ld(to.q.w), ld(to.q.x), ld(to.q.y), ld(to.q.z));
        let tt = ld(t);
        let s = 1.0 - tt;
        let dot = ((x1 * x2 + z2 * z1) + y2 * y1) + w2 * w1;
        let sign: f64 = if dot < 0.0 { -1.0 } else { 1.0 };
        let d = sign * dot;
        let (a, b) = if 1.0 - d > ld(1.192_092_9e-7_f32) {
            let sin_theta = (1.0 - d * d).sqrt();
            let theta = ld(st(sin_theta.atan2(ld(st(d)))));
            let inverse = ld(st(1.0 / sin_theta));
            ((theta * s).sin() * inverse, (theta * tt).sin() * inverse)
        } else {
            (s, tt)
        };
        let b = b * sign;
        let q = Quaternion::new(
            st(b * w2 + a * w1),
            st(a * x1 + b * x2),
            st(b * y2 + a * y1),
            st(b * z2 + a * z1),
        );
        self.set_rotate(q);
    }
}

impl From<ac2_math::Frame> for Frame {
    fn from(value: ac2_math::Frame) -> Self {
        let q = value.orientation;
        Self::new(value.origin.into(), Quaternion::new(q.w, q.x, q.y, q.z))
    }
}

impl From<Frame> for ac2_math::Frame {
    fn from(value: Frame) -> Self {
        let q = value.q;
        ac2_math::Frame::new(
            value.origin.into(),
            ac2_math::Quat::from_xyzw(q.x, q.y, q.z, q.w),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn identity_matrix_from_identity_quaternion() {
        let mut frame = Frame::IDENTITY;
        frame.cache_matrix();
        assert_eq!(frame.m, Frame::IDENTITY.m);
    }

    #[test]
    fn combine_places_the_child_origin_in_the_parent_frame() {
        let mut parent = Frame::IDENTITY;
        parent.origin = Vector::new(10.0, 20.0, 30.0);
        parent.grotate(Vector::new(0.0, 0.0, std::f32::consts::FRAC_PI_2));
        // Local +y is global -x after a quarter turn about z.
        let child = Frame::combine(&parent, Vector::new(0.0, 2.0, 1.0), Quaternion::IDENTITY);
        assert!(close(child.origin.x, 8.0), "{:?}", child.origin);
        assert!(close(child.origin.y, 20.0) && close(child.origin.z, 31.0));
        assert_eq!(child.m, parent.m);
        let scaled = Frame::combine_scaled(
            &parent,
            Vector::new(0.0, 2.0, 1.0),
            Quaternion::IDENTITY,
            Vector::new(1.0, 3.0, 2.0),
        );
        assert!(close(scaled.origin.x, 4.0) && close(scaled.origin.z, 32.0));
    }

    #[test]
    fn grotate_about_z_turns_the_y_axis_west() {
        let mut frame = Frame::IDENTITY;
        frame.grotate(Vector::new(0.0, 0.0, std::f32::consts::FRAC_PI_2));
        // Local +y (north) now points to global -x; the compass heading is 270.
        let forward = frame.local_to_global_vec(Vector::new(0.0, 1.0, 0.0));
        assert!(
            close(forward.x, -1.0) && close(forward.y, 0.0),
            "{forward:?}"
        );
        assert!((frame.heading().degrees() - 270.0).abs() < 1e-3);
        let back = frame.global_to_local_vec(forward);
        assert!(close(back.y, 1.0) && close(back.x, 0.0));
    }

    #[test]
    fn tiny_rotations_are_ignored() {
        let mut frame = Frame::IDENTITY;
        frame.grotate(Vector::new(0.0, 0.0, 1e-5));
        assert_eq!(frame, Frame::IDENTITY);
    }

    #[test]
    fn set_heading_faces_the_compass_heading() {
        for degrees in [0.0_f32, 45.0, 90.0, 200.0, 359.0] {
            let mut frame = Frame::IDENTITY;
            frame.set_heading(Heading::from_degrees(degrees));
            assert!(
                (frame.heading().degrees() - degrees).abs() < 1e-3,
                "{degrees}"
            );
            assert!((frame.q.norm2() - 1.0).abs() < 1e-6);
            let up = frame.local_to_global_vec(Vector::new(0.0, 0.0, 1.0));
            assert!((up.z - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn slerp_halfway_between_headings() {
        let from = Frame::IDENTITY;
        let mut to = Frame::IDENTITY;
        to.grotate(Vector::new(0.0, 0.0, 1.0));
        let mut mid = Frame::IDENTITY;
        mid.slerp_rotation(&from, &to, 0.5);
        let mut expected = Frame::IDENTITY;
        expected.grotate(Vector::new(0.0, 0.0, 0.5));
        assert!(mid.approx_eq(&expected));
    }
}
