//! Movement packs (`CPositionPack`, `PositionPack`) and the packed heading and motion word.
//!
//! The word is `Heading::ToUInt8 << 24 | Vector::UnitVectorToUInt24`. The quantizers run on x87
//! at 53-bit precision with f32 constants; they are evaluated in f64 here and rounded where
//! retail stores a float.

use crate::protocol::opcode::Opcode;
use crate::protocol::reader::{Reader, Writer};
use crate::protocol::types::{CellPosition, InstanceIdWithStamp};
use crate::protocol::ProtocolError;

/// `1/360` and `1/255` as the f32 constants retail multiplies by.
const INV_360: f32 = 1.0 / 360.0;
const INV_255: f32 = 1.0 / 255.0;

/// `Heading::ToUInt8`: `_ftol(degrees * (1/360) * 255)`, truncated, low byte. Degrees are
/// `Heading::m_degrees`, kept in `[0, 360)` by `Heading::set`.
pub fn heading_to_u8(degrees: f32) -> u8 {
    let units = f64::from(degrees) * f64::from(INV_360) * 255.0;
    units.trunc() as i64 as u8
}

/// `Heading::FromUInt8`: `fmod(units * (1/255) * 360, 360)` stored as a float, wrapped once
/// when negative.
pub fn heading_from_u8(units: u8) -> f32 {
    let value = (f64::from(units) * f64::from(INV_255) * 360.0) % 360.0;
    if value < 0.0 {
        let wrapped = value + 360.0;
        return if 360.0 <= wrapped {
            0.0
        } else {
            wrapped as f32
        };
    }
    value as f32
}

/// `Vector::UnitVectorToUInt24`: each axis clamped to `[-1, 1]` (NaN to -1), then
/// `_ftol((v + 1) * 127.5)`; x in bits 16..23, y in 8..15, z in 0..7.
pub fn motion_to_u24(motion: [f32; 3]) -> u32 {
    let byte = |v: f32| {
        let v = if !(-1.0 < v) {
            -1.0
        } else if 1.0 <= v {
            1.0
        } else {
            v
        };
        ((f64::from(v) + 1.0) * 127.5).trunc() as i64 as u32 & 0xFF
    };
    (byte(motion[0]) << 16) | (byte(motion[1]) << 8) | byte(motion[2])
}

/// `Vector::UnitVectorFromUInt24`: `2 * (byte * (1/255)) - 1` per axis, stored as floats.
pub fn motion_from_u24(word: u32) -> [f32; 3] {
    let axis = |shift: u32| {
        let scaled = f64::from((word >> shift) & 0xFF) * f64::from(INV_255);
        (scaled + scaled - 1.0) as f32
    };
    [axis(16), axis(8), axis(0)]
}

/// The client's movement report (`CPositionPack::StreamPack`), event `0x1008E` on queue 3
/// (`ECM_Physics::SendEvt_CPosition`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientPosition {
    /// `Timer::cur_time` when sent (`PlayerMovementModule::SendPositionEvent`).
    pub time: f64,
    pub position: CellPosition,
    pub heading_degrees: f32,
    /// `m_doMotion`: the motion interpreter's last do-motion values.
    pub motion: [f32; 3],
    pub cancel_moveto: bool,
    pub instance_stamp: u16,
    pub teleport_stamp: u16,
    pub force_position_stamp: u16,
    pub moveto_stamp: u16,
    pub jump_velocity: Option<[f32; 3]>,
}

impl ClientPosition {
    /// The event bytes: opcode, then the pack.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::event(Opcode::CLIENT_POSITION);
        w.f64(self.time);
        self.position.write(&mut w);
        w.u32((u32::from(heading_to_u8(self.heading_degrees)) << 24) | motion_to_u24(self.motion));
        w.u32((u32::from(self.cancel_moveto) * 4) | (u32::from(self.jump_velocity.is_some()) * 2));
        w.u16(self.instance_stamp)
            .u16(self.teleport_stamp)
            .u16(self.force_position_stamp)
            .u16(self.moveto_stamp);
        if let Some(velocity) = self.jump_velocity {
            for value in velocity {
                w.f32(value);
            }
        }
        w.align();
        w.bytes
    }
}

/// An object's position from the server (`ECM_Physics::RecvEvt_Position`, `0x10086`, and
/// `RecvEvt_PositionCell`, `0x100C1`): the sender, then `PositionPack::StreamPack`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerPosition {
    pub sender: InstanceIdWithStamp,
    pub time: f64,
    pub position: CellPosition,
    pub heading_degrees: f32,
    pub motion: [f32; 3],
    pub contact: bool,
    pub position_stamp: u16,
    pub force_position_stamp: u16,
    pub teleport_stamp: u16,
    /// Jump (flag 2) or impulse (flag 8) velocity; never both.
    pub jump: bool,
    pub impulse: bool,
    pub velocity: Option<[f32; 3]>,
}

impl ServerPosition {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let sender = InstanceIdWithStamp::read(reader)?;
        let time = reader.f64()?;
        let position = CellPosition::read(reader)?;
        let word = reader.u32()?;
        let flags = reader.u32()?;
        let jump = flags & 2 != 0;
        let impulse = flags & 8 != 0;
        if jump && impulse {
            return Err(ProtocolError::Invalid("jump and impulse together"));
        }
        let position_stamp = reader.u16()?;
        let force_position_stamp = reader.u16()?;
        let teleport_stamp = reader.u16()?;
        let velocity = (jump || impulse).then(|| reader.vec3()).transpose()?;
        reader.align()?;
        Ok(Self {
            sender,
            time,
            position,
            heading_degrees: heading_from_u8((word >> 24) as u8),
            motion: motion_from_u24(word),
            contact: flags & 4 != 0,
            position_stamp,
            force_position_stamp,
            teleport_stamp,
            jump,
            impulse,
            velocity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        heading_from_u8, heading_to_u8, motion_from_u24, motion_to_u24, ClientPosition,
        ServerPosition,
    };
    use crate::protocol::reader::Reader;
    use crate::protocol::types::CellPosition;

    #[test]
    fn heading_and_motion_quantize_like_the_x87_code() {
        assert_eq!(heading_to_u8(0.0), 0);
        assert_eq!(heading_to_u8(180.0), 127);
        assert_eq!(heading_to_u8(359.99), 254);
        // 90 * (f32)(1/360) * 255 = 63.7500003...: 63.
        assert_eq!(heading_to_u8(90.0), 63);
        assert_eq!(heading_from_u8(0), 0.0);
        // 255 * (f32)(1/255) is a hair above 1, so the fmod leaves a tiny heading, not 0 or 360.
        let wrapped = heading_from_u8(255);
        assert!(wrapped > 0.0 && wrapped < 1e-4, "{wrapped}");
        assert_eq!(
            heading_from_u8(64),
            (64.0 * f64::from(1.0f32 / 255.0) * 360.0) as f32
        );
        assert_eq!(motion_to_u24([0.0, 0.0, 0.0]), 0x7F7F7F);
        assert_eq!(motion_to_u24([0.0, 1.0, -1.0]), 0x7FFF00);
        assert_eq!(motion_to_u24([f32::NAN, 2.0, -3.0]), 0x00FF00);
        let back = motion_from_u24(0x7FFF00);
        // (f32)(1/255) is a hair above 1/255: byte 255 decodes to 1.0000001, as in retail.
        assert_eq!(back[1], 1.000_000_1);
        assert_eq!(back[2], -1.0);
        assert_eq!(
            back[0],
            ((127.0 * f64::from(1.0f32 / 255.0)) * 2.0 - 1.0) as f32
        );
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
            instance_stamp: 1,
            teleport_stamp: 2,
            force_position_stamp: 3,
            moveto_stamp: 4,
            jump_velocity: None,
        };
        let bytes = report.encode();
        let mut expected = 0x0001_008Eu32.to_le_bytes().to_vec();
        expected.extend(2.0f64.to_le_bytes());
        expected.extend(0x0299_0109u32.to_le_bytes());
        for v in [59.0f32, 240.0, -44.5] {
            expected.extend(v.to_le_bytes());
        }
        expected.extend(((191u32 << 24) | 0x7F7F7F).to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        for stamp in [1u16, 2, 3, 4] {
            expected.extend(stamp.to_le_bytes());
        }
        assert_eq!(bytes, expected);
        let jumping = ClientPosition {
            jump_velocity: Some([0.0, 0.0, 5.0]),
            cancel_moveto: true,
            ..report
        };
        let bytes = jumping.encode();
        assert_eq!(bytes.len(), 4 + 52);
        assert_eq!(&bytes[32..36], &6u32.to_le_bytes());
    }

    #[test]
    fn server_position_reads_the_sender_then_the_pack() {
        let mut bytes = Vec::new();
        bytes.extend(7u64.to_le_bytes());
        bytes.extend(3u16.to_le_bytes());
        bytes.extend(4u16.to_le_bytes());
        bytes.extend(1.5f64.to_le_bytes());
        bytes.extend(0x0299_0109u32.to_le_bytes());
        for v in [1.0f32, 2.0, 3.0] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend(((64u32 << 24) | 0x7F7F7F).to_le_bytes());
        bytes.extend((4u32 | 8).to_le_bytes());
        for stamp in [9u16, 10, 11] {
            bytes.extend(stamp.to_le_bytes());
        }
        for v in [0.0f32, 0.0, 2.0] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend([0, 0]);
        let mut reader = Reader::new(&bytes);
        let p = ServerPosition::read(&mut reader).unwrap();
        reader.finish().unwrap();
        assert_eq!((p.sender.id, p.sender.instance, p.sender.other), (7, 3, 4));
        assert_eq!(
            (p.position_stamp, p.force_position_stamp, p.teleport_stamp),
            (9, 10, 11)
        );
        assert!(p.contact && p.impulse && !p.jump);
        assert_eq!(p.velocity, Some([0.0, 0.0, 2.0]));
    }
}
