//! Quality updates (`ECM_Qualities::RecvEvt_Update*_Private` and `_Visual`, identical in admin
//! 7148 and Legions 7160): the private form updates the player, the visual form starts with the
//! object's `InstanceIDWithStamp`. Then the quality type id and its value, each followed by
//! `STREAMPACK_ALIGN` (positions, strings and string infos unpack through their own `UnPack`).

use crate::protocol::opcode::Opcode;
use crate::protocol::reader::{Reader, Writer};
use crate::protocol::string_info::StringInfo;
use crate::protocol::types::{InstanceIdWithStamp, Position};
use crate::protocol::ProtocolError;

/// A quality's new value, by table.
#[derive(Debug, Clone, PartialEq)]
pub enum QualityValue {
    Int(i32),
    Bool(i32),
    Float(f32),
    String(Vec<u8>),
    DataId(u32),
    InstanceId(u64),
    Position(Position),
    Timestamp(f64),
    StringInfo(Box<StringInfo>),
    LongInt(i64),
}

impl QualityValue {
    /// The table code `CWRepository::RecvEvt_Update*_Private` passes to its change notice:
    /// bool 1, DID 2, float 3, int 4, IID 5, position 6, string 7, timestamp 8, string info 10,
    /// long int 12.
    pub fn table(&self) -> u32 {
        match self {
            Self::Bool(_) => 1,
            Self::DataId(_) => 2,
            Self::Float(_) => 3,
            Self::Int(_) => 4,
            Self::InstanceId(_) => 5,
            Self::Position(_) => 6,
            Self::String(_) => 7,
            Self::Timestamp(_) => 8,
            Self::StringInfo(_) => 10,
            Self::LongInt(_) => 12,
        }
    }
}

/// One quality update.
#[derive(Debug, Clone, PartialEq)]
pub struct QualityUpdate {
    /// `None` for the private form (the player).
    pub object: Option<InstanceIdWithStamp>,
    /// The quality type id (`_stype`).
    pub quality: u32,
    pub value: QualityValue,
}

/// (private, visual) opcode pairs and the value each carries.
const KINDS: [(u32, u32, u8); 10] = [
    (Opcode::UPDATE_INT_PRIVATE, Opcode::UPDATE_INT_VISUAL, b'i'),
    (
        Opcode::UPDATE_BOOL_PRIVATE,
        Opcode::UPDATE_BOOL_VISUAL,
        b'b',
    ),
    (
        Opcode::UPDATE_FLOAT_PRIVATE,
        Opcode::UPDATE_FLOAT_VISUAL,
        b'f',
    ),
    (
        Opcode::UPDATE_STRING_PRIVATE,
        Opcode::UPDATE_STRING_VISUAL,
        b's',
    ),
    (
        Opcode::UPDATE_DATA_ID_PRIVATE,
        Opcode::UPDATE_DATA_ID_VISUAL,
        b'd',
    ),
    (
        Opcode::UPDATE_INSTANCE_ID_PRIVATE,
        Opcode::UPDATE_INSTANCE_ID_VISUAL,
        b'o',
    ),
    (
        Opcode::UPDATE_POSITION_PRIVATE,
        Opcode::UPDATE_POSITION_VISUAL,
        b'p',
    ),
    (
        Opcode::UPDATE_TIMESTAMP_PRIVATE,
        Opcode::UPDATE_TIMESTAMP_VISUAL,
        b't',
    ),
    (
        Opcode::UPDATE_STRING_INFO_PRIVATE,
        Opcode::UPDATE_STRING_INFO_VISUAL,
        b'n',
    ),
    (
        Opcode::UPDATE_LONG_INT_PRIVATE,
        Opcode::UPDATE_LONG_INT_VISUAL,
        b'l',
    ),
];

impl QualityUpdate {
    /// Decodes `opcode`'s body, or `None` when it is not a quality update.
    pub(crate) fn read(opcode: u32, r: &mut Reader<'_>) -> Option<Result<Self, ProtocolError>> {
        let (visual, kind) = KINDS.iter().find_map(|&(private, visual, kind)| {
            (opcode == private || opcode == visual).then_some((opcode == visual, kind))
        })?;
        let decode = |r: &mut Reader<'_>| -> Result<Self, ProtocolError> {
            let object = visual.then(|| InstanceIdWithStamp::read(r)).transpose()?;
            let quality = r.u32()?;
            r.align()?;
            let value = match kind {
                b'i' => QualityValue::Int(r.u32()? as i32),
                b'b' => QualityValue::Bool(r.u32()? as i32),
                b'f' => QualityValue::Float(r.f32()?),
                b'd' => QualityValue::DataId(r.u32()?),
                b'o' => QualityValue::InstanceId(r.u64()?),
                b't' => QualityValue::Timestamp(r.f64()?),
                b'l' => QualityValue::LongInt(r.u64()? as i64),
                b's' => QualityValue::String(r.string()?),
                b'p' => QualityValue::Position(Position::read(r)?),
                _ => QualityValue::StringInfo(Box::new(StringInfo::read(r)?)),
            };
            r.align()?;
            Ok(Self {
                object,
                quality,
                value,
            })
        };
        Some(decode(r))
    }

    /// The server's side (`ECM_Qualities::SendEvt_Update*`): the event as
    /// [`QualityUpdate::read`] takes it, its opcode first.
    pub fn encode(&self) -> Vec<u8> {
        let kind = match &self.value {
            QualityValue::Int(_) => b'i',
            QualityValue::Bool(_) => b'b',
            QualityValue::Float(_) => b'f',
            QualityValue::String(_) => b's',
            QualityValue::DataId(_) => b'd',
            QualityValue::InstanceId(_) => b'o',
            QualityValue::Position(_) => b'p',
            QualityValue::Timestamp(_) => b't',
            QualityValue::StringInfo(_) => b'n',
            QualityValue::LongInt(_) => b'l',
        };
        let (private, visual, _) = KINDS.iter().find(|k| k.2 == kind).copied().unwrap();
        let mut w = Writer::event(if self.object.is_some() {
            visual
        } else {
            private
        });
        if let Some(object) = &self.object {
            object.write(&mut w);
        }
        w.u32(self.quality).align();
        match &self.value {
            QualityValue::Int(v) | QualityValue::Bool(v) => _ = w.u32(*v as u32),
            QualityValue::Float(v) => _ = w.f32(*v),
            QualityValue::String(v) => _ = w.string(v),
            QualityValue::DataId(v) => _ = w.u32(*v),
            QualityValue::InstanceId(v) => _ = w.u64(*v),
            QualityValue::Position(v) => v.write(&mut w),
            QualityValue::Timestamp(v) => _ = w.f64(*v),
            QualityValue::StringInfo(v) => v.write(&mut w),
            QualityValue::LongInt(v) => _ = w.u64(*v as u64),
        }
        w.align();
        w.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::{QualityUpdate, QualityValue};
    use crate::protocol::opcode::Opcode;
    use crate::protocol::reader::Reader;

    fn decode(opcode: u32, body: &[u8]) -> QualityUpdate {
        let mut r = Reader::new(body);
        let update = QualityUpdate::read(opcode, &mut r).unwrap().unwrap();
        r.finish().unwrap();
        update
    }

    #[test]
    fn private_updates_carry_the_type_then_the_value() {
        let mut b = 0x19u32.to_le_bytes().to_vec();
        b.extend((-5i32).to_le_bytes());
        let update = decode(Opcode::UPDATE_INT_PRIVATE, &b);
        assert_eq!(
            (update.object, update.quality, update.value),
            (None, 0x19, QualityValue::Int(-5))
        );
        let mut b = 2u32.to_le_bytes().to_vec();
        b.extend(1234.5f64.to_le_bytes());
        assert_eq!(
            decode(Opcode::UPDATE_TIMESTAMP_PRIVATE, &b).value,
            QualityValue::Timestamp(1234.5)
        );
        let mut b = 1u32.to_le_bytes().to_vec();
        crate::pack_string(&mut b, 0, b"abc");
        assert_eq!(
            decode(Opcode::UPDATE_STRING_PRIVATE, &b).value,
            QualityValue::String(b"abc".to_vec())
        );
    }

    #[test]
    fn visual_updates_name_their_object_first() {
        let mut b = 0x55u64.to_le_bytes().to_vec();
        b.extend([2, 0, 3, 0]);
        b.extend(7u32.to_le_bytes());
        b.extend((-9i64).to_le_bytes());
        let update = decode(Opcode::UPDATE_LONG_INT_VISUAL, &b);
        assert_eq!(update.object.map(|o| (o.id, o.instance)), Some((0x55, 2)));
        assert_eq!(update.value, QualityValue::LongInt(-9));
        assert_eq!(update.value.table(), 12);
        for end in 0..b.len() {
            let mut r = Reader::new(&b[..end]);
            assert!(QualityUpdate::read(Opcode::UPDATE_LONG_INT_VISUAL, &mut r)
                .unwrap()
                .is_err());
        }
        assert!(QualityUpdate::read(0x1234, &mut Reader::new(&b)).is_none());
    }

    #[test]
    fn encoded_updates_are_the_bytes_read_takes() {
        let private = QualityUpdate {
            object: None,
            quality: 0x102,
            value: QualityValue::Int(-3),
        };
        let mut expected = Opcode::UPDATE_INT_PRIVATE.to_le_bytes().to_vec();
        expected.extend(0x102u32.to_le_bytes());
        expected.extend((-3i32).to_le_bytes());
        assert_eq!(private.encode(), expected);
        let visual = QualityUpdate {
            object: Some(crate::protocol::InstanceIdWithStamp {
                id: 0x55,
                instance: 2,
                other: 3,
            }),
            quality: 0x11E,
            value: QualityValue::Float(95.0),
        };
        let mut expected = Opcode::UPDATE_FLOAT_VISUAL.to_le_bytes().to_vec();
        expected.extend(0x55u64.to_le_bytes());
        expected.extend([2, 0, 3, 0]);
        expected.extend(0x11Eu32.to_le_bytes());
        expected.extend(95.0f32.to_le_bytes());
        assert_eq!(visual.encode(), expected);
        let values = [
            QualityValue::Bool(1),
            QualityValue::String(b"abcde".to_vec()),
            QualityValue::DataId(7),
            QualityValue::InstanceId(9),
            QualityValue::Timestamp(2.5),
            QualityValue::LongInt(-9),
            QualityValue::StringInfo(Box::default()),
            QualityValue::Position(crate::protocol::Position {
                cell: 5,
                origin: [1.0, 2.0, 3.0],
                rotation: [1.0, 0.0, 0.0, 0.0],
            }),
        ];
        for value in values {
            for object in [None, visual.object] {
                let update = QualityUpdate {
                    object,
                    quality: 4,
                    value: value.clone(),
                };
                let bytes = update.encode();
                assert_eq!(bytes.len() % 4, 0);
                let opcode = u32::from_le_bytes(bytes[..4].try_into().unwrap());
                assert_eq!(decode(opcode, &bytes[4..]), update);
            }
        }
    }
}
