//! Server -> client script calls (`InterpCEvent`).

use super::{function, literal, Args};
use crate::protocol::native::{ExaminationProfile, NativeObject, SelectionInfo};
use crate::protocol::pack::{pack_values, PackValue};
use crate::protocol::string_info::StringInfo;
use crate::protocol::usage::UsageDesc;
use crate::protocol::ProtocolError;

/// The server's side of any script call to the player: the `InterpCEvent_Broadcast`
/// (0x10066) body, the function id, then the `PackBuffer` (its size, then the data).
pub fn broadcast(function: u32, arguments: &[PackValue]) -> Vec<u8> {
    let arguments = pack_values(arguments);
    let mut bytes = crate::protocol::Opcode::INTERP_C_EVENT_BROADCAST
        .to_le_bytes()
        .to_vec();
    bytes.extend(function.to_le_bytes());
    bytes.extend((arguments.len() as u32).to_le_bytes());
    bytes.extend(arguments);
    bytes
}

/// A message the server sends to the client's scripts.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptMessage {
    /// [`function::UPDATE_SELECTION_INFO`].
    UpdateSelectionInfo { info: SelectionInfo, id: u64 },
    /// [`function::EXAMINATION_PROFILE`].
    ExaminationProfile(ExaminationProfile),
    /// [`function::HEAR_SPEECH`]: `kind` is not read by `CCommunicationSystem` 0x6.
    HearSpeech {
        kind: u32,
        text: StringInfo,
        speaker_name: StringInfo,
        speaker: u64,
    },
    /// [`function::DISPLAY_STRING_INFO`] or [`function::DISPLAY_STRING_INFO_2`] (`function`):
    /// the text and its display type.
    DisplayStringInfo {
        function: u32,
        text: StringInfo,
        kind: u32,
    },
    /// [`function::DISPLAY_TEXT`].
    DisplayText { text: String, kind: u32 },
    /// [`function::HEAR_TELL`]: `kind` picks the string (0x40000001-0x40000003 or other).
    HearTell {
        kind: u32,
        text: StringInfo,
        teller_name: StringInfo,
        teller: u64,
    },
    /// [`function::USAGE_DONE`].
    UsageDone(UsageDesc),
    /// [`function::USAGE_ERROR`].
    UsageError { error: u32, target: u64, item: u64 },
}

impl ScriptMessage {
    pub fn function(&self) -> u32 {
        match self {
            Self::UpdateSelectionInfo { .. } => function::UPDATE_SELECTION_INFO,
            Self::ExaminationProfile(_) => function::EXAMINATION_PROFILE,
            Self::HearSpeech { .. } => function::HEAR_SPEECH,
            Self::DisplayStringInfo { function, .. } => *function,
            Self::DisplayText { .. } => function::DISPLAY_TEXT,
            Self::HearTell { .. } => function::HEAR_TELL,
            Self::UsageDone(_) => function::USAGE_DONE,
            Self::UsageError { .. } => function::USAGE_ERROR,
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
            Self::HearSpeech {
                kind,
                text,
                speaker_name: name,
                speaker: id,
            }
            | Self::HearTell {
                kind,
                text,
                teller_name: name,
                teller: id,
            } => vec![
                PackValue::Int(*kind),
                literal(text),
                literal(name),
                PackValue::LongInt(*id),
            ],
            Self::DisplayStringInfo { text, kind, .. } => {
                vec![literal(text), PackValue::Int(*kind)]
            }
            Self::DisplayText { text, kind } => vec![
                PackValue::native(NativeObject::WString(text.clone())),
                PackValue::Int(*kind),
            ],
            Self::UsageDone(desc) => vec![PackValue::Reference(desc.to_reference())],
            Self::UsageError {
                error,
                target,
                item,
            } => vec![
                PackValue::Int(*error),
                PackValue::LongInt(*target),
                PackValue::LongInt(*item),
            ],
        }
    }

    /// The `InterpCEvent_Broadcast` (0x10066) body for the player: function id, then the
    /// `PackBuffer` (its size, then the data).
    pub fn to_broadcast(&self) -> Vec<u8> {
        broadcast(self.function(), &self.arguments())
    }

    /// A known message from an `InterpCEvent`, or `None` for another function id. The client's
    /// handler takes the values the VM pushes, so a mismatched argument list fails here.
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let decode = |read: &dyn Fn(&mut Args) -> Result<Self, ProtocolError>| {
            let mut args = Args::new(arguments)?;
            let message = read(&mut args)?;
            args.finish()?;
            Ok(message)
        };
        Some(match function {
            function::UPDATE_SELECTION_INFO => decode(&|a| match a.native()? {
                NativeObject::SelectionInfo(info) => Ok(Self::UpdateSelectionInfo {
                    info,
                    id: a.long()?,
                }),
                _ => Err(ProtocolError::Invalid("selection info")),
            }),
            function::EXAMINATION_PROFILE => decode(&|a| match a.native()? {
                NativeObject::ExaminationProfile(profile) => Ok(Self::ExaminationProfile(profile)),
                _ => Err(ProtocolError::Invalid("examination profile")),
            }),
            function::HEAR_SPEECH => decode(&|a| {
                Ok(Self::HearSpeech {
                    kind: a.int()?,
                    text: a.string_info()?,
                    speaker_name: a.string_info()?,
                    speaker: a.long()?,
                })
            }),
            function::HEAR_TELL => decode(&|a| {
                Ok(Self::HearTell {
                    kind: a.int()?,
                    text: a.string_info()?,
                    teller_name: a.string_info()?,
                    teller: a.long()?,
                })
            }),
            function::DISPLAY_STRING_INFO | function::DISPLAY_STRING_INFO_2 => decode(&|a| {
                Ok(Self::DisplayStringInfo {
                    function,
                    text: a.string_info()?,
                    kind: a.int()?,
                })
            }),
            function::DISPLAY_TEXT => decode(&|a| match a.native()? {
                NativeObject::WString(text) => Ok(Self::DisplayText {
                    text,
                    kind: a.int()?,
                }),
                _ => Err(ProtocolError::Invalid("display text")),
            }),
            function::USAGE_DONE => {
                decode(&|a| Ok(Self::UsageDone(UsageDesc::from_reference(&a.reference()?)?)))
            }
            function::USAGE_ERROR => decode(&|a| {
                Ok(Self::UsageError {
                    error: a.int()?,
                    target: a.long()?,
                    item: a.long()?,
                })
            }),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    fn round_trip(message: ScriptMessage) -> Vec<u8> {
        let bytes = message.to_broadcast();
        assert_eq!(&bytes[..8], &words(&[0x0001_0066, message.function()])[..]);
        let size = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        assert_eq!(size, bytes.len() - 12);
        assert_eq!(
            ScriptMessage::decode(message.function(), &bytes[12..]),
            Some(Ok(message))
        );
        bytes[12..].to_vec()
    }

    #[test]
    fn selection_info_is_the_native_then_the_object() {
        let info = SelectionInfo {
            kind: SelectionInfo::CREATURE,
            cur_health: 5,
            max_health: 10,
            ..SelectionInfo::default()
        };
        let args = round_trip(ScriptMessage::UpdateSelectionInfo { info, id: 0x77 });
        let mut expected = words(&[5, 1, 1, 0x4400_0100, 0, 0xFFFF_00AB]);
        expected.extend(words(&[0x4000_0001, 5, 0, 10, 0, 0, 0, 0, 2]));
        expected.extend(0x77u64.to_le_bytes());
        assert_eq!(args, expected);
        round_trip(ScriptMessage::ExaminationProfile(
            ExaminationProfile::default(),
        ));
        // The arguments in the other order fail, as the handler's frame would not match.
        let swapped = pack_values(&[PackValue::LongInt(1), PackValue::LongInt(2)]);
        assert!(ScriptMessage::decode(0x8212_002C, &swapped)
            .unwrap()
            .is_err());
    }

    #[test]
    fn chat_messages_keep_their_handler_order() {
        let text = StringInfo {
            literal: Some("Hello".into()),
            ..StringInfo::default()
        };
        let name = StringInfo {
            literal: Some("Birch".into()),
            ..StringInfo::default()
        };
        let speech = round_trip(ScriptMessage::HearSpeech {
            kind: 0,
            text: text.clone(),
            speaker_name: name.clone(),
            speaker: 9,
        });
        assert_eq!(&speech[..8], &words(&[1, 0]));
        assert_eq!(&speech[speech.len() - 12..], &words(&[2, 9, 0]));
        round_trip(ScriptMessage::HearTell {
            kind: 0x4000_0001,
            text: text.clone(),
            teller_name: name,
            teller: 9,
        });
        let display = round_trip(ScriptMessage::DisplayStringInfo {
            function: function::DISPLAY_STRING_INFO,
            text,
            kind: 4,
        });
        assert_eq!(&display[display.len() - 8..], &words(&[1, 4]));
        let shown = round_trip(ScriptMessage::DisplayText {
            text: "Hi".into(),
            kind: 1,
        });
        assert_eq!(
            &shown[..24],
            &words(&[5, 1, 1, 0x4400_0100, 0, 0xFFFF_0076])
        );
        assert!(ScriptMessage::decode(0x8205_0099, &[]).is_none());
    }

    #[test]
    fn usage_outcomes_carry_the_desc_or_the_error() {
        let mut desc = UsageDesc::new(1, 2, 3);
        desc.error = 0x2711;
        let args = round_trip(ScriptMessage::UsageDone(desc));
        assert_eq!(&args[..8], &words(&[5, 2]));
        let args = round_trip(ScriptMessage::UsageError {
            error: 0x2711,
            target: 3,
            item: 2,
        });
        let mut expected = words(&[1, 0x2711, 2]);
        expected.extend(3u64.to_le_bytes());
        expected.extend(words(&[2]));
        expected.extend(2u64.to_le_bytes());
        assert_eq!(args, expected);
    }
}
