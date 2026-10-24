//! Movement packs (`CPositionPack`, `PositionPack`) and their packed heading and motion word.

use crate::protocol::opcode::Opcode;
use crate::protocol::reader::{Reader, Writer};
use crate::protocol::ProtocolError;

/// A cell id and an AC-axis (z-up) offset relative to the landblock the cell names
/// (`PositionOffset::StreamPack`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellPosition {
    pub cell: u32,
    pub origin: [f32; 3],
}

impl CellPosition {
    fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let cell = reader.u32()?;
        let origin = reader.vec3()?;
        if cell == 0 || origin.iter().any(|v| !v.is_finite()) {
            return Err(ProtocolError::Invalid("position"));
        }
        Ok(Self { cell, origin })
    }

    fn write(&self, writer: &mut Writer) {
        writer.u32(self.cell);
        for value in self.origin {
            writer.f32(value);
        }
    }
}

/// `Heading::ToUInt8`: degrees clockwise from north at 255 units per revolution, truncated.
pub fn heading_to_u8(degrees: f32) -> u8 {
    let units = (degrees.rem_euclid(360.0) * 255.0 / 360.0) as u32;
    (units % 255) as u8
}

/// `Heading::FromUInt8`.
pub fn heading_from_u8(units: u8) -> f32 {
    f32::from(units) * 360.0 / 255.0
}

/// `Vector::UnitVectorToUInt24`: each axis of a unit vector as a byte, `(v + 1) / 2 * 255`
/// truncated, x in bits 16..23, y in 8..15 and z in 0..7. The zero vector packs as `0x7F7F7F`.
pub fn motion_to_u24(motion: [f32; 3]) -> u32 {
    let byte = |v: f32| ((v.clamp(-1.0, 1.0) + 1.0) * 0.5 * 255.0) as u32 & 0xFF;
    (byte(motion[0]) << 16) | (byte(motion[1]) << 8) | byte(motion[2])
}

/// `Vector::UnitVectorFromUInt24`.
pub fn motion_from_u24(word: u32) -> [f32; 3] {
    let axis = |shift: u32| f32::from(((word >> shift) & 0xFF) as u8) / 255.0 * 2.0 - 1.0;
    [axis(16), axis(8), axis(0)]
}

/// The client's movement report (`CPositionPack::StreamPack`), sent on the ordered queue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientPosition {
    /// The client's physics time, strictly increasing.
    pub time: f64,
    pub position: CellPosition,
    pub heading_degrees: f32,
    /// The body-local direction of intended motion (`m_doMotion`), a unit vector or zero.
    pub motion: [f32; 3],
    pub cancel_moveto: bool,
    /// Instance, teleport, forced-position and move-to stamps, echoing the server's.
    pub stamps: [u16; 4],
    pub jump_velocity: Option<[f32; 3]>,
}

impl ClientPosition {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::event(Opcode::CLIENT_POSITION);
        w.f64(self.time);
        self.position.write(&mut w);
        w.u32((u32::from(heading_to_u8(self.heading_degrees)) << 24) | motion_to_u24(self.motion));
        w.u32((u32::from(self.cancel_moveto) * 4) | (u32::from(self.jump_velocity.is_some()) * 2));
        for stamp in self.stamps {
            w.u16(stamp);
        }
        if let Some(velocity) = self.jump_velocity {
            for value in velocity {
                w.f32(value);
            }
        }
        w.align();
        w.bytes
    }
}

/// Another object's position from the server (`PositionPack::StreamPack`), after its
/// `InstanceIDWithStamp`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerPosition {
    pub id: u64,
    pub instance: u16,
    pub time: f64,
    pub position: CellPosition,
    pub heading_degrees: f32,
    pub motion: [f32; 3],
    pub contact: bool,
    /// Position, forced-position and teleport stamps.
    pub stamps: [u16; 3],
    /// Jump or impulse velocity.
    pub velocity: Option<[f32; 3]>,
}

impl ServerPosition {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let id = reader.u64()?;
        let instance = reader.u16()?;
        reader.align()?;
        let time = reader.f64()?;
        let position = CellPosition::read(reader)?;
        let word = reader.u32()?;
        let flags = reader.u32()?;
        let jump = flags & 2 != 0;
        let impulse = flags & 8 != 0;
        if jump && impulse {
            return Err(ProtocolError::Invalid("jump and impulse together"));
        }
        let stamps = [reader.u16()?, reader.u16()?, reader.u16()?];
        let velocity = if jump || impulse {
            Some(reader.vec3()?)
        } else {
            None
        };
        reader.align()?;
        Ok(Self {
            id,
            instance,
            time,
            position,
            heading_degrees: heading_from_u8((word >> 24) as u8),
            motion: motion_from_u24(word),
            contact: flags & 4 != 0,
            stamps,
            velocity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        heading_from_u8, heading_to_u8, motion_from_u24, motion_to_u24, CellPosition,
        ClientPosition, ServerPosition,
    };
    use crate::protocol::reader::Reader;

    #[test]
    fn heading_and_motion_pack_like_the_client() {
        assert_eq!(heading_to_u8(0.0), 0);
        assert_eq!(heading_to_u8(180.0), 127);
        assert_eq!(heading_to_u8(359.9), 254);
        assert!((heading_from_u8(127) - 179.29).abs() < 0.01);
        assert_eq!(motion_to_u24([0.0, 0.0, 0.0]), 0x7F7F7F);
        assert_eq!(motion_to_u24([0.0, 1.0, 0.0]), 0x7FFF7F);
        let back = motion_from_u24(0x7FFF7F);
        assert!(back[1] == 1.0 && back[0].abs() < 0.01);
    }

    #[test]
    fn client_position_layout_matches_cpositionpack() {
        let report = ClientPosition {
            time: 2.0,
            position: CellPosition {
                cell: 0x0299_0109,
                origin: [59.0, 240.0, -44.5],
            },
            heading_degrees: 270.0,
            motion: [0.0; 3],
            cancel_moveto: false,
            stamps: [1, 0, 0, 0],
            jump_velocity: None,
        };
        let bytes = report.encode();
        assert_eq!(bytes.len(), 4 + 40);
        assert_eq!(&bytes[..4], &0x0001_008Eu32.to_le_bytes());
        assert_eq!(&bytes[12..16], &0x0299_0109u32.to_le_bytes());
        assert_eq!(bytes[31], heading_to_u8(270.0));
        assert_eq!(&bytes[36..38], &1u16.to_le_bytes());
        let jumping = ClientPosition {
            jump_velocity: Some([0.0, 0.0, 5.0]),
            ..report
        };
        let bytes = jumping.encode();
        assert_eq!(bytes.len(), 4 + 52);
        assert_eq!(&bytes[32..36], &2u32.to_le_bytes());
    }

    #[test]
    fn server_position_decodes_after_the_instance_stamp() {
        let mut bytes = Vec::new();
        bytes.extend(7u64.to_le_bytes());
        bytes.extend(3u16.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(1.5f64.to_le_bytes());
        bytes.extend(0x0299_0109u32.to_le_bytes());
        for v in [1.0f32, 2.0, 3.0] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend(((64u32 << 24) | 0x7F7F7F).to_le_bytes());
        bytes.extend(4u32.to_le_bytes());
        for stamp in [9u16, 0, 0] {
            bytes.extend(stamp.to_le_bytes());
        }
        bytes.extend([0, 0]);
        let mut reader = Reader::new(&bytes);
        let position = ServerPosition::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!(
            (position.id, position.instance, position.stamps[0]),
            (7, 3, 9)
        );
        assert!(position.contact && position.velocity.is_none());
        assert_eq!(position.position.origin, [1.0, 2.0, 3.0]);
    }
}
