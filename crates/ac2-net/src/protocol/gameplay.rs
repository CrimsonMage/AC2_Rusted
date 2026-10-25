//! Gameplay messages: script calls between the player's weenie scripts and the server.
//!
//! The client sends a message through its `WM_*` package's sender stub in the script library
//! (WLib 0x56000005): the stub packs its arguments into a `PackBuffer` and calls
//! `CWInterface::SendServerEvt` with the server function id `0x8000_0000 | package << 16 | n`
//! (`InterpSEvent`, 0x10053). The server calls a client function the same way
//! (`InterpCEvent`), which the player's `WM_*` package forwards to `CPlayer` or a `C*System`
//! script. Function ids, argument order and pack types below are read from those stubs.

use crate::protocol::client_event::ClientEvent;
use crate::protocol::native::{
    ExaminationProfile, ExaminationRequest, NativeObject, SelectionInfo,
};
use crate::protocol::pack::{pack_values, unpack_values, PackValue};
use crate::protocol::ProtocolError;

/// Script function ids, by the `WM_*` package that sends or receives them.
pub mod function {
    /// Client -> server, `WM_Player` stub 0x28 (called by `CPlayer::SetSelection`): the new
    /// selection, one long int.
    pub const SET_TARGET: u32 = 0x8212_0016;
    /// Client -> server, `WM_Examination` stub 0x8 (`CExaminationSystem`): an
    /// `ExaminationRequest` reference.
    pub const EXAMINE: u32 = 0x820A_0007;
    /// Server -> client, `WM_Player` 0x2C -> `CPlayer` 0x34: a `SelectionInfo` reference, then
    /// the object it describes (long int).
    pub const UPDATE_SELECTION_INFO: u32 = 0x8212_002C;
    /// Server -> client, `WM_Examination` 0x9 -> `CPlayer` 0xDA -> `CExaminationSystem` 0x2:
    /// an `ExaminationProfile` reference.
    pub const EXAMINATION_PROFILE: u32 = 0x820A_0009;
}

/// A message the client's scripts send to the server.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptRequest {
    /// [`function::SET_TARGET`].
    SetTarget(u64),
    /// [`function::EXAMINE`]: the request to examine, or a cleared request to stop.
    Examine(ExaminationRequest),
}

impl ScriptRequest {
    pub fn function(&self) -> u32 {
        match self {
            Self::SetTarget(_) => function::SET_TARGET,
            Self::Examine(_) => function::EXAMINE,
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
        let decoded = || {
            let values = unpack_values(arguments)?;
            Ok(match (function, values.as_slice()) {
                (function::SET_TARGET, [PackValue::LongInt(id)]) => Self::SetTarget(*id),
                (function::EXAMINE, [value]) => match value.as_native() {
                    Some(NativeObject::ExaminationRequest(request)) => Self::Examine(*request),
                    _ => return Err(ProtocolError::Invalid("examination request")),
                },
                _ => return Err(ProtocolError::Invalid("script request arguments")),
            })
        };
        match function {
            function::SET_TARGET | function::EXAMINE => Some(decoded()),
            _ => None,
        }
    }
}

/// A message the server sends to the client's scripts.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptMessage {
    /// [`function::UPDATE_SELECTION_INFO`].
    UpdateSelectionInfo { info: SelectionInfo, id: u64 },
    /// [`function::EXAMINATION_PROFILE`].
    ExaminationProfile(ExaminationProfile),
}

impl ScriptMessage {
    pub fn function(&self) -> u32 {
        match self {
            Self::UpdateSelectionInfo { .. } => function::UPDATE_SELECTION_INFO,
            Self::ExaminationProfile(_) => function::EXAMINATION_PROFILE,
        }
    }

    /// The packed arguments, in the order the client's handler takes them.
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::UpdateSelectionInfo { info, id } => vec![
                PackValue::native(NativeObject::SelectionInfo(*info)),
                PackValue::LongInt(*id),
            ],
            Self::ExaminationProfile(profile) => {
                vec![PackValue::native(NativeObject::ExaminationProfile(
                    profile.clone(),
                ))]
            }
        }
    }

    /// The `InterpCEvent_Broadcast` (0x10066) body for the player: function id, then the
    /// `PackBuffer` (its size, then the data).
    pub fn to_broadcast(&self) -> Vec<u8> {
        let arguments = pack_values(&self.arguments());
        let mut bytes = crate::protocol::Opcode::INTERP_C_EVENT_BROADCAST
            .to_le_bytes()
            .to_vec();
        bytes.extend(self.function().to_le_bytes());
        bytes.extend((arguments.len() as u32).to_le_bytes());
        bytes.extend(arguments);
        bytes
    }

    /// A known message from an `InterpCEvent`, or `None` for another function id. The client's
    /// handler takes the values the VM pushes, so a mismatched argument list fails here.
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let decoded = || {
            let values = unpack_values(arguments)?;
            Ok(match (function, values.as_slice()) {
                (function::UPDATE_SELECTION_INFO, [info, PackValue::LongInt(id)]) => {
                    match info.as_native() {
                        Some(NativeObject::SelectionInfo(info)) => Self::UpdateSelectionInfo {
                            info: *info,
                            id: *id,
                        },
                        _ => return Err(ProtocolError::Invalid("selection info")),
                    }
                }
                (function::EXAMINATION_PROFILE, [profile]) => match profile.as_native() {
                    Some(NativeObject::ExaminationProfile(profile)) => {
                        Self::ExaminationProfile(profile.clone())
                    }
                    _ => return Err(ProtocolError::Invalid("examination profile")),
                },
                _ => return Err(ProtocolError::Invalid("script message arguments")),
            })
        };
        match function {
            function::UPDATE_SELECTION_INFO | function::EXAMINATION_PROFILE => Some(decoded()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn set_target_is_one_long_int_on_queue_3() {
        let (queue, bytes) = ScriptRequest::SetTarget(0x8000_0001_0000_0123)
            .to_event()
            .encode();
        assert_eq!(queue, 3);
        let mut expected = words(&[0x0001_0053, 0x8212_0016, 12, 2]);
        expected.extend(0x8000_0001_0000_0123u64.to_le_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(
            ScriptRequest::decode(0x8212_0016, &bytes[12..]),
            Some(Ok(ScriptRequest::SetTarget(0x8000_0001_0000_0123)))
        );
        assert!(ScriptRequest::decode(0x8212_0016, &words(&[1, 5]))
            .unwrap()
            .is_err());
        assert!(ScriptRequest::decode(0x8212_0017, &bytes[12..]).is_none());
    }

    #[test]
    fn examine_packs_the_request_as_a_native_reference() {
        let request = ExaminationRequest::object(0x55);
        let (_, bytes) = ScriptRequest::Examine(request).to_event().encode();
        let mut args = words(&[5, 1, 1, 0x4400_0100, 0, 0xFFFF_009A]);
        args.extend(words(&[0x4000_0002, 0, 0x55, 0, 0x203, 0]));
        let mut expected = words(&[0x0001_0053, 0x820A_0007, args.len() as u32]);
        expected.extend(&args);
        assert_eq!(bytes, expected);
        assert_eq!(
            ScriptRequest::decode(0x820A_0007, &args),
            Some(Ok(ScriptRequest::Examine(request)))
        );
    }

    #[test]
    fn server_messages_round_trip_through_their_broadcast() {
        let info = SelectionInfo {
            kind: SelectionInfo::CREATURE,
            cur_health: 5,
            max_health: 10,
            ..SelectionInfo::default()
        };
        let message = ScriptMessage::UpdateSelectionInfo { info, id: 0x77 };
        let bytes = message.to_broadcast();
        let mut args = words(&[5, 1, 1, 0x4400_0100, 0, 0xFFFF_00AB]);
        args.extend(words(&[0x4000_0001, 5, 0, 10, 0, 0, 0, 0, 2]));
        args.extend(0x77u64.to_le_bytes());
        let mut expected = words(&[0x0001_0066, 0x8212_002C, args.len() as u32]);
        expected.extend(&args);
        assert_eq!(bytes, expected);
        assert_eq!(ScriptMessage::decode(0x8212_002C, &args), Some(Ok(message)));
        let profile = ScriptMessage::ExaminationProfile(ExaminationProfile::default());
        let bytes = profile.to_broadcast();
        assert_eq!(
            ScriptMessage::decode(0x820A_0009, &bytes[12..]),
            Some(Ok(profile))
        );
        // The arguments in the other order fail, as the handler's frame would not match.
        let swapped = pack_values(&[PackValue::LongInt(1), PackValue::LongInt(2)]);
        assert!(ScriptMessage::decode(0x8212_002C, &swapped)
            .unwrap()
            .is_err());
    }
}
