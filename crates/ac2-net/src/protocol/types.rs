//! Small structures many events share.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::ProtocolError;

/// `InstanceIDWithStamp::StreamPack`: the object id, its instance stamp and a second stamp
/// (the visual-ordering stamp, or a position stamp, depending on the event).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InstanceIdWithStamp {
    pub id: u64,
    pub instance: u16,
    pub other: u16,
}

impl InstanceIdWithStamp {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            id: reader.u64()?,
            instance: reader.u16()?,
            other: reader.u16()?,
        })
    }

    #[cfg(test)]
    pub(crate) fn write(&self, writer: &mut Writer) {
        writer.u64(self.id).u16(self.instance).u16(self.other);
    }
}

/// `PositionOffset::StreamPack`: a cell id and an AC-axis (z-up) offset relative to the
/// landblock the cell names.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellPosition {
    pub cell: u32,
    pub origin: [f32; 3],
}

impl CellPosition {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            cell: reader.u32()?,
            origin: reader.vec3()?,
        })
    }

    pub(crate) fn write(&self, writer: &mut Writer) {
        writer.u32(self.cell);
        for value in self.origin {
            writer.f32(value);
        }
    }
}

/// `Position::UnPack`: a cell id and a `Frame` (origin, then the quaternion w, x, y, z).
/// `AFrame::UnPack` refuses a NaN component or a quaternion whose squared norm is not within
/// 0.0002 of 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub cell: u32,
    pub origin: [f32; 3],
    /// (w, x, y, z).
    pub rotation: [f32; 4],
}

impl Position {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let cell = reader.u32()?;
        let origin = reader.vec3()?;
        let rotation = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
        if origin.iter().chain(&rotation).any(|v| v.is_nan()) {
            return Err(ProtocolError::Invalid("frame"));
        }
        // `w*w + x*x + y*y + z*z` is stored to a float before the comparison.
        let norm = rotation
            .iter()
            .map(|&v| f64::from(v) * f64::from(v))
            .sum::<f64>() as f32;
        if norm.is_nan() || (f64::from(norm) - 1.0).abs() >= 0.0002 {
            return Err(ProtocolError::Invalid("frame"));
        }
        Ok(Self {
            cell,
            origin,
            rotation,
        })
    }
}

/// `PHashTable::StreamPack`'s header: the count in the low 24 bits, the bucket-size index
/// in the high 8.
pub(crate) fn hash_count(reader: &mut Reader<'_>) -> Result<usize, ProtocolError> {
    let count = (reader.u32()? & 0x00FF_FFFF) as usize;
    if count > crate::protocol::reader::MAX_ITEMS {
        return Err(ProtocolError::Invalid("hash table count"));
    }
    Ok(count)
}

/// `PHashTable<unsigned long, float>::StreamPack`: key and float pairs in table order.
pub(crate) fn float_table(reader: &mut Reader<'_>) -> Result<Vec<(u32, f32)>, ProtocolError> {
    (0..hash_count(reader)?)
        .map(|_| Ok((reader.u32()?, reader.f32()?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{InstanceIdWithStamp, Position};
    use crate::protocol::reader::{Reader, Writer};

    #[test]
    fn instance_id_with_stamp_is_twelve_bytes() {
        let sender = InstanceIdWithStamp {
            id: 0x0102_0304_0506_0708,
            instance: 9,
            other: 10,
        };
        let mut writer = Writer { bytes: Vec::new() };
        sender.write(&mut writer);
        assert_eq!(writer.bytes, [8, 7, 6, 5, 4, 3, 2, 1, 9, 0, 10, 0]);
        let mut reader = Reader::new(&writer.bytes);
        assert_eq!(InstanceIdWithStamp::read(&mut reader).unwrap(), sender);
    }

    #[test]
    fn positions_need_a_unit_quaternion() {
        let mut bytes = 5u32.to_le_bytes().to_vec();
        for v in [1.0f32, 2.0, 3.0, 1.0, 0.0, 0.0, 0.0] {
            bytes.extend(v.to_le_bytes());
        }
        assert!(Position::read(&mut Reader::new(&bytes)).is_ok());
        bytes[16..20].copy_from_slice(&0.99f32.to_le_bytes());
        assert!(Position::read(&mut Reader::new(&bytes)).is_err());
    }
}
