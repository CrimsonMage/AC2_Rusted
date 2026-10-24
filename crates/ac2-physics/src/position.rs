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

    /// `PositionOffset::GetOffset`: the vector from `self` to `other`. The landblock delta and
    /// `other`'s x and y are summed and stored before `self`'s origin is subtracted.
    pub(crate) fn offset_to(&self, other: &Position) -> Vector {
        let block = get_block_offset(self.cell_id, other.cell_id);
        let x = ld(st(ld(block.x) + ld(other.frame.origin.x)));
        let y = ld(st(ld(block.y) + ld(other.frame.origin.y)));
        Vector::new(
            st(x - ld(self.frame.origin.x)),
            st(y - ld(self.frame.origin.y)),
            st((ld(block.z) + ld(other.frame.origin.z)) - ld(self.frame.origin.z)),
        )
    }
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
    pub(crate) fn local_to_local(&self, from: &Position, p: Vector) -> Vector {
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
