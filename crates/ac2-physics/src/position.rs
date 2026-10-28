//! Retail `Position`: a cell id and a frame relative to that cell's landblock.

use crate::fpu::{ld, st};
use crate::frame::Frame;
use crate::landdefs::get_block_offset;
use crate::vector::{Plane, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub cell_id: u32,
    pub frame: Frame,
}

impl Position {
    pub fn new(cell_id: u32, frame: Frame) -> Self {
        Self { cell_id, frame }
    }

    /// `Position::lerp_origin`: takes `from`'s cell and the origin `t` of the way from `from`
    /// to `to` (the rotation is left as it is). Across landblocks `to`'s x and y gain the block
    /// offset on the x87 stack while its z is stored first.
    pub fn lerp_origin(&mut self, from: &Position, to: &Position, t: f32) {
        self.cell_id = from.cell_id;
        let (a, b, t) = (from.frame.origin, to.frame.origin, ld(t));
        let same_block = (from.cell_id == 0 && to.cell_id == 0)
            || (from.cell_id != 0 && to.cell_id != 0 && from.cell_id >> 16 == to.cell_id >> 16);
        let (x, y, z) = if same_block {
            (ld(b.x), ld(b.y), ld(b.z))
        } else {
            let offset = get_block_offset(from.cell_id, to.cell_id);
            (
                ld(offset.x) + ld(b.x),
                ld(offset.y) + ld(b.y),
                ld(st(ld(offset.z) + ld(b.z))),
            )
        };
        self.frame.origin = Vector::new(
            st((x - ld(a.x)) * t + ld(a.x)),
            st((y - ld(a.y)) * t + ld(a.y)),
            st((z - ld(a.z)) * t + ld(a.z)),
        );
    }

    /// `Position::slerp`: `lerp_origin` then `Frame::slerp_rotation`, `t` of the way from
    /// `from` to `to` (in `from`'s cell).
    pub fn slerp(&mut self, from: &Position, to: &Position, t: f32) {
        self.lerp_origin(from, to, t);
        self.frame.slerp_rotation(&from.frame, &to.frame, t);
    }

    /// `PositionOffset::GetOffset`: the vector from `self` to `other`. The landblock delta and
    /// `other`'s x and y are summed and stored before `self`'s origin is subtracted.
    pub fn offset_to(&self, other: &Position) -> Vector {
        offset_between(
            self.cell_id,
            self.frame.origin,
            other.cell_id,
            other.frame.origin,
        )
    }
}

/// `PositionOffset::GetOffset` on bare origins: from `from` in cell `from_cell` to `to` in
/// `to_cell`.
pub(crate) fn offset_between(from_cell: u32, from: Vector, to_cell: u32, to: Vector) -> Vector {
    let block = get_block_offset(from_cell, to_cell);
    let x = ld(st(ld(block.x) + ld(to.x)));
    let y = ld(st(ld(block.y) + ld(to.y)));
    Vector::new(
        st(x - ld(from.x)),
        st(y - ld(from.y)),
        st((ld(block.z) + ld(to.z)) - ld(from.z)),
    )
}

impl Position {
    /// `Position::localtoglobalvec`: a direction of this frame in its block's axes.
    pub(crate) fn local_to_global_vec(&self, v: Vector) -> Vector {
        let m = self.frame.m.map(ld);
        let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
        Vector::new(
            st((m[3] * y + m[0] * x) + m[6] * z),
            st((m[4] * y + m[1] * x) + m[7] * z),
            st((m[5] * y + m[2] * x) + m[8] * z),
        )
    }

    /// `Position::globaltolocalvec`: a direction of this block's axes in this frame (the
    /// `Frame::globaltolocalvec` sums).
    pub fn global_to_local_vec(&self, v: Vector) -> Vector {
        self.frame.global_to_local_vec(v)
    }

    /// `Position::localtoglobal`: a point of `local`'s frame in `self`'s landblock.
    pub(crate) fn local_to_global(&self, local: &Position, p: Vector) -> Vector {
        let g = local.frame_point(p);
        let block = get_block_offset(self.cell_id, local.cell_id);
        Vector::new(
            st(ld(block.x) + ld(g.x)),
            st(ld(block.y) + ld(g.y)),
            st(ld(block.z) + ld(g.z)),
        )
    }

    /// `Position::localtolocal`: a point of `from`'s frame in `self`'s frame.
    pub fn local_to_local(&self, from: &Position, p: Vector) -> Vector {
        let global = self.local_to_global(from, p);
        self.frame.global_to_local(global)
    }

    /// The rotation `localtoglobalvec` inlines in the BSP slides (`BSPTREE::step_sphere_up`,
    /// `BSPTREE::slide_sphere`): `(m6 z + m0 x) + m3 y`.
    pub(crate) fn local_to_global_vec_zxy(&self, v: Vector) -> Vector {
        let m = self.frame.m.map(ld);
        let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
        Vector::new(
            st((m[6] * z + m[0] * x) + m[3] * y),
            st((m[7] * z + m[1] * x) + m[4] * y),
            st((m[8] * z + m[2] * x) + m[5] * y),
        )
    }

    /// `Plane::localtoglobal`: a plane of `from`'s frame in `self`'s landblock frame.
    pub(crate) fn plane_local_to_global(&self, from: &Position, plane: Plane) -> Plane {
        let minus_d = -ld(plane.d);
        let point = Vector::new(
            st(minus_d * ld(plane.n.x)),
            st(minus_d * ld(plane.n.y)),
            st(minus_d * ld(plane.n.z)),
        );
        let n = from.local_to_global_vec(plane.n);
        let p = self.local_to_global(from, point);
        let d = -((ld(n.z) * ld(p.z) + ld(n.y) * ld(p.y)) + ld(n.x) * ld(p.x));
        Plane::new(n, st(d))
    }

    /// `Position::distance`: `|GetOffset(other)|`, an x87 result.
    pub fn distance(&self, other: &Position) -> f64 {
        self.distance_squared(other).sqrt()
    }

    /// `Position::distance_squared`: `(x*x + y*y) + z*z` of the offset.
    pub fn distance_squared(&self, other: &Position) -> f64 {
        let o = self.offset_to(other);
        let (x, y, z) = (ld(o.x), ld(o.y), ld(o.z));
        (x * x + y * y) + z * z
    }

    /// `Position::heading`: the compass heading from `self` toward `other` (0 when nearer
    /// than `F_EPSILON` horizontally).
    pub fn heading_to(&self, other: &Position) -> crate::heading::Heading {
        let o = self.offset_to(other);
        let (x, y) = (ld(o.x), ld(o.y));
        let length = (y * y + x * x).sqrt();
        if length < ld(crate::fpu::F_EPSILON) {
            return crate::heading::Heading::from_radians(0.0);
        }
        let inverse = 1.0 / length;
        let nx = ld(st(x * inverse));
        let ny = y * inverse;
        crate::heading::Heading::from_radians(st(crate::frame::FIVE_HALVES_PI - ny.atan2(nx)))
    }

    /// `Position::add_local_offset`: move by `offset` in the frame's local axes.
    pub fn add_local_offset(&mut self, offset: Vector) {
        let m = self.frame.m.map(ld);
        let (x, y, z) = (ld(offset.x), ld(offset.y), ld(offset.z));
        let gx = (m[3] * y + m[0] * x) + m[6] * z;
        let gy = (m[4] * y + m[1] * x) + m[7] * z;
        let gz = ld(st((m[5] * y + m[2] * x) + m[8] * z));
        let o = &mut self.frame.origin;
        o.x = st(gx + ld(o.x));
        o.y = st(gy + ld(o.y));
        o.z = st(gz + ld(o.z));
    }

    /// The frame transform `localtoglobal`/`localtolocal` inline: rotate, add the origin, store.
    fn frame_point(&self, p: Vector) -> Vector {
        let m = self.frame.m.map(ld);
        let o = self.frame.origin;
        let (x, y, z) = (ld(p.x), ld(p.y), ld(p.z));
        Vector::new(
            st(((m[3] * y + m[0] * x) + m[6] * z) + ld(o.x)),
            st(((m[4] * y + m[1] * x) + m[7] * z) + ld(o.y)),
            st(((m[5] * y + m[2] * x) + m[8] * z) + ld(o.z)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::Quaternion;

    #[test]
    fn global_to_local_vec_undoes_local_to_global_vec() {
        let mut p = Position::default();
        p.frame.grotate(Vector::new(0.0, 0.0, 1.0));
        let local = Vector::new(1.0, 2.0, 3.0);
        let back = p.global_to_local_vec(p.local_to_global_vec(local));
        assert!(
            (back.x - 1.0).abs() < 1e-5
                && (back.y - 2.0).abs() < 1e-5
                && (back.z - 3.0).abs() < 1e-5
        );
    }

    #[test]
    fn offsets_cross_landblocks() {
        let here = Position::new(
            0x7F7F_0040,
            Frame::new(Vector::new(155.0, 80.0, 10.0), Quaternion::IDENTITY),
        );
        let there = Position::new(
            0x807F_0001,
            Frame::new(Vector::new(5.0, 80.0, 12.0), Quaternion::IDENTITY),
        );
        assert_eq!(here.offset_to(&there), Vector::new(10.0, 0.0, 2.0));
        assert_eq!(there.offset_to(&here), Vector::new(-10.0, 0.0, -2.0));
    }
}
