//! Engine structures held as natives (`CNativeFactory`): `Vector` (0x70), `MissileParameters`
//! (0x46) and `RandomSelectionTable_Int` (0x60). `Position` (0x5A) and
//! `BehaviorParams` (0x17) use the event codecs' own types.

use crate::protocol::reader::{Reader, Writer, MAX_ITEMS};
use crate::protocol::ProtocolError;

/// `MissileParameters::StreamPack`: the bitfield, then for each set bit its fields, every one a
/// 32-bit word (a float, an int or a bool). Bit 0x1000 guards two groups (`m_conserve_fuel`..
/// after 0x80, and `m_use_env_max_velocities` after 0x800); no group follows bit 0x100.
const MISSILE_GROUPS: [(u32, usize); 18] = [
    (0x1, 2),
    (0x2, 6),
    (0x4, 3),
    (0x8, 2),
    (0x10, 1),
    (0x20, 1),
    (0x40, 1),
    (0x80, 2),
    (0x1000, 3),
    (0x200, 2),
    (0x400, 3),
    (0x800, 1),
    (0x1000, 1),
    (0x2000, 2),
    (0x4000, 1),
    (0x8000, 1),
    (0x10000, 1),
    (0x20000, 2),
];

/// `MissileParameters`: the bitfield and the packed words, in stream order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MissileParameters {
    pub bitfield: u32,
    pub words: Vec<u32>,
}

impl MissileParameters {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let bitfield = r.u32()?;
        let mut words = Vec::new();
        for (bit, n) in MISSILE_GROUPS {
            if bitfield & bit != 0 {
                for _ in 0..n {
                    words.push(r.u32()?);
                }
            }
        }
        Ok(Self { bitfield, words })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.bitfield);
        let packed: usize = MISSILE_GROUPS
            .iter()
            .filter(|(bit, _)| self.bitfield & bit != 0)
            .map(|(_, n)| n)
            .sum();
        for i in 0..packed {
            w.u32(self.words.get(i).copied().unwrap_or(0));
        }
    }
}

/// One `RandomSelectionTable<unsigned long>::ElementStruct`: object, weight, subtree weight.
pub type SelectionElement = (u32, f32, f32);

/// `RandomSelectionTable<unsigned long>::StreamPack`: the unused then the used elements, each
/// a count and the elements.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RandomSelectionTable {
    pub unused: Vec<SelectionElement>,
    pub used: Vec<SelectionElement>,
}

impl RandomSelectionTable {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let read = |r: &mut Reader<'_>| -> Result<Vec<SelectionElement>, ProtocolError> {
            let n = r.u32()? as usize;
            if n > MAX_ITEMS || n * 12 > r.remaining() {
                return Err(ProtocolError::Invalid("selection table count"));
            }
            (0..n).map(|_| Ok((r.u32()?, r.f32()?, r.f32()?))).collect()
        };
        Ok(Self {
            unused: read(r)?,
            used: read(r)?,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        for list in [&self.unused, &self.used] {
            w.u32(list.len() as u32);
            for (object, weight, subtree) in list {
                w.u32(*object).f32(*weight).f32(*subtree);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::NativeObject;
    use super::*;

    fn round_trip(object: NativeObject) -> Vec<u8> {
        let mut w = Writer::new();
        object.write(&mut w);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(
            NativeObject::read(object.native_type(), &mut r).unwrap(),
            object
        );
        r.finish().unwrap();
        w.bytes
    }

    #[test]
    fn missile_groups_follow_their_bits() {
        // Bit 0x1000 packs three words after the 0x80 group and one after 0x800.
        let params = MissileParameters {
            bitfield: 0x1 | 0x1000,
            words: vec![1, 2, 3, 4, 5, 6],
        };
        let bytes = round_trip(NativeObject::MissileParameters(params));
        assert_eq!(bytes.len(), 4 * 7);
        round_trip(NativeObject::Vector([1.0, -2.0, 0.5]));
        let table = RandomSelectionTable {
            unused: vec![(1, 0.5, 1.0)],
            used: vec![],
        };
        assert_eq!(
            round_trip(NativeObject::RandomSelectionTable(table)).len(),
            4 * 5
        );
    }
}
