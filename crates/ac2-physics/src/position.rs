//! Retail `Position`: a cell id and a frame relative to that cell's landblock.

use crate::fpu::{ld, st};
use crate::frame::Frame;
use crate::landdefs::get_block_offset;
use crate::vector::Vector;

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
