//! Client -> server script calls (`InterpSEvent`).

use super::{function, literal, Args};
use crate::protocol::client_event::ClientEvent;
use crate::protocol::inventory::{InvEquipDesc, InvMoveDesc};
use crate::protocol::native::{ExaminationRequest, NativeObject};
use crate::protocol::pack::{pack_values, PackValue};
use crate::protocol::string_info::StringInfo;
use crate::protocol::usage::UsageDesc;
use crate::protocol::ProtocolError;

/// A message the client's scripts send to the server.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptRequest {
    /// [`function::SET_TARGET`].
    SetTarget(u64),
    /// [`function::EXAMINE`]: the request to examine, or a cleared request to stop.
    Examine(ExaminationRequest),
    /// [`function::SAY`]: `CPlayer` 0xDB passes 0 for `mode`.
    Say { mode: u32, text: StringInfo },
    /// [`function::TELL`]: `CPlayer` 0x112 passes 0 for `mode`.
    Tell {
        mode: u32,
        text: StringInfo,
        recipient: StringInfo,
    },
    /// [`function::EMOTE`].
    Emote(u32),
    /// [`function::USE`].
    Use(UsageDesc),
    /// [`function::MOVE_ITEM`].
    MoveItem(InvMoveDesc),
    /// [`function::REORGANIZE_CONTENTS`].
    ReorganizeContents(InvMoveDesc),
    /// [`function::EQUIP_ITEM`].
    EquipItem(InvEquipDesc),
    /// [`function::UNEQUIP_ITEM`].
    UnequipItem(InvEquipDesc),
    /// [`function::OPEN_CONTAINER`].
    OpenContainer(u64),
    /// [`function::CLOSE_CONTAINER`].
    CloseContainer(u64),
}

impl ScriptRequest {
    pub fn function(&self) -> u32 {
        match self {
            Self::SetTarget(_) => function::SET_TARGET,
            Self::Examine(_) => function::EXAMINE,
            Self::Say { .. } => function::SAY,
            Self::Tell { .. } => function::TELL,
            Self::Emote(_) => function::EMOTE,
            Self::Use(_) => function::USE,
            Self::MoveItem(_) => function::MOVE_ITEM,
            Self::ReorganizeContents(_) => function::REORGANIZE_CONTENTS,
            Self::EquipItem(_) => function::EQUIP_ITEM,
            Self::UnequipItem(_) => function::UNEQUIP_ITEM,
            Self::OpenContainer(_) => function::OPEN_CONTAINER,
            Self::CloseContainer(_) => function::CLOSE_CONTAINER,
        }
    }

    /// The packed arguments, in the stub's pack order.
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::SetTarget(id) => vec![PackValue::LongInt(*id)],
            Self::Examine(request) => {
                vec![PackValue::native(NativeObject::ExaminationRequest(
                    *request,
                ))]
            }
            Self::Say { mode, text } => vec![PackValue::Int(*mode), literal(text)],
            Self::Tell {
                mode,
                text,
                recipient,
            } => vec![PackValue::Int(*mode), literal(text), literal(recipient)],
            Self::Emote(emote) => vec![PackValue::Int(*emote)],
            Self::Use(desc) => vec![PackValue::Reference(desc.to_reference())],
            Self::MoveItem(desc) | Self::ReorganizeContents(desc) => {
                vec![PackValue::Reference(desc.to_reference())]
            }
            Self::EquipItem(desc) | Self::UnequipItem(desc) => {
                vec![PackValue::Reference(desc.to_reference())]
            }
            Self::OpenContainer(id) | Self::CloseContainer(id) => vec![PackValue::LongInt(*id)],
        }
    }

    /// The `InterpSEvent` the stub's `SendServerEvt` produces.
    pub fn to_event(&self) -> ClientEvent {
        ClientEvent::Interp {
            function: self.function(),
            arguments: pack_values(&self.arguments()),
            encrypted: false,
        }
    }

    /// The server's side: a known request from an `InterpSEvent`, or `None` for another
    /// function id.
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let decode = |read: fn(&mut Args) -> Result<Self, ProtocolError>| {
            let mut args = Args::new(arguments)?;
            let request = read(&mut args)?;
            args.finish()?;
            Ok(request)
        };
        Some(match function {
            function::SET_TARGET => decode(|a| Ok(Self::SetTarget(a.long()?))),
            function::EXAMINE => decode(|a| match a.native()? {
                NativeObject::ExaminationRequest(request) => Ok(Self::Examine(request)),
                _ => Err(ProtocolError::Invalid("examination request")),
            }),
            function::SAY => decode(|a| {
                Ok(Self::Say {
                    mode: a.int()?,
                    text: a.string_info()?,
                })
            }),
            function::TELL => decode(|a| {
                Ok(Self::Tell {
                    mode: a.int()?,
                    text: a.string_info()?,
                    recipient: a.string_info()?,
                })
            }),
            function::EMOTE => decode(|a| Ok(Self::Emote(a.int()?))),
            function::USE => decode(|a| Ok(Self::Use(UsageDesc::from_reference(&a.reference()?)?))),
            function::MOVE_ITEM => decode(|a| {
                Ok(Self::MoveItem(InvMoveDesc::from_reference(
                    &a.reference()?,
                )?))
            }),
            function::REORGANIZE_CONTENTS => decode(|a| {
                Ok(Self::ReorganizeContents(InvMoveDesc::from_reference(
                    &a.reference()?,
                )?))
            }),
            function::EQUIP_ITEM => decode(|a| {
                Ok(Self::EquipItem(InvEquipDesc::from_reference(
                    &a.reference()?,
                )?))
            }),
            function::UNEQUIP_ITEM => decode(|a| {
                Ok(Self::UnequipItem(InvEquipDesc::from_reference(
                    &a.reference()?,
                )?))
            }),
            function::OPEN_CONTAINER => decode(|a| Ok(Self::OpenContainer(a.long()?))),
            function::CLOSE_CONTAINER => decode(|a| Ok(Self::CloseContainer(a.long()?))),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::reader::Writer;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    fn literal_bytes(text: &str) -> Vec<u8> {
        let mut w = Writer::new();
        StringInfo {
            literal: Some(text.into()),
            ..StringInfo::default()
        }
        .write(&mut w);
        let mut b = words(&[5, 1, 1, 0x4400_0100, 0, 0xFFFF_006A]);
        b.extend(w.bytes);
        b.extend(words(&[0]));
        b
    }

    fn round_trip(request: ScriptRequest, args: Vec<u8>) {
        let (queue, bytes) = request.to_event().encode();
        assert_eq!(queue, 3);
        let mut expected = words(&[0x0001_0053, request.function(), args.len() as u32]);
        expected.extend(&args);
        assert_eq!(bytes, expected);
        assert_eq!(
            ScriptRequest::decode(request.function(), &args),
            Some(Ok(request))
        );
    }

    #[test]
    fn set_target_is_one_long_int() {
        let mut args = words(&[2]);
        args.extend(0x8000_0001_0000_0123u64.to_le_bytes());
        round_trip(ScriptRequest::SetTarget(0x8000_0001_0000_0123), args);
        assert!(ScriptRequest::decode(0x8212_0016, &words(&[1, 5]))
            .unwrap()
            .is_err());
        assert!(ScriptRequest::decode(0x8212_0017, &[]).is_none());
    }

    #[test]
    fn examine_packs_the_request_as_a_native_reference() {
        let mut args = words(&[5, 1, 1, 0x4400_0100, 0, 0xFFFF_009A]);
        args.extend(words(&[0x4000_0002, 0, 0x55, 0, 0x203, 0]));
        round_trip(
            ScriptRequest::Examine(ExaminationRequest::object(0x55)),
            args,
        );
    }

    #[test]
    fn chat_packs_the_mode_then_literal_strings() {
        let text = |s: &str| StringInfo {
            literal: Some(s.into()),
            ..StringInfo::default()
        };
        let mut args = words(&[1, 0]);
        args.extend(literal_bytes("Hello"));
        round_trip(
            ScriptRequest::Say {
                mode: 0,
                text: text("Hello"),
            },
            args,
        );
        let mut args = words(&[1, 0]);
        args.extend(literal_bytes("hi"));
        args.extend(literal_bytes("Explorer"));
        round_trip(
            ScriptRequest::Tell {
                mode: 0,
                text: text("hi"),
                recipient: text("Explorer"),
            },
            args.clone(),
        );
        // A missing recipient or an extra value fails.
        let mut extra = args.clone();
        extra.extend(words(&[1, 0]));
        assert!(ScriptRequest::decode(0x8205_0002, &extra).unwrap().is_err());
        round_trip(ScriptRequest::Emote(0x1D), words(&[1, 0x1D]));
    }

    #[test]
    fn use_packs_the_usage_desc_graph() {
        let desc = UsageDesc::new(0x5000_0000_0000_0001, 0x42, 0x43);
        let args = pack_values(&[PackValue::Reference(desc.to_reference())]);
        assert_eq!(&args[..20], &words(&[5, 2, 1, 2, 0x4400_0100]));
        round_trip(ScriptRequest::Use(desc), args);
    }
}
