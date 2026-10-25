//! The script calls skills, shortcuts, advancement, fellowships and allegiances exchange, as
//! codecs only: each call's function id and the kinds of the values its `WM_*` stub packs (or
//! its handler takes), from the Legions script library. What the values mean is the player
//! scripts' business (the script VM runs `CPlayer`, `CAgent`, `FellowshipSystem`,
//! `AllegianceSystem`); the objects they reference (`SkillInfo`, `SkillRepository`, ...) travel
//! as packed reference graphs ([`PackedReference`](crate::protocol::PackedReference)) of
//! package instances and natives.
//!
//! Names: a client call is named by the IFun (or script function) that sends it, a server call
//! by the script function its handler forwards to. A 32-bit handler argument whose type the
//! port has not traced is a [`ArgKind::Word`]: an int or a reference.

use super::Args;
use crate::protocol::client_event::ClientEvent;
use crate::protocol::pack::{pack_values, PackValue};
use crate::protocol::ProtocolError;

/// The kind of one packed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    Int,
    LongInt,
    Float,
    Reference,
    /// A 32-bit value: an int or a reference.
    Word,
}

impl ArgKind {
    fn accepts(self, value: &PackValue) -> bool {
        matches!(
            (self, value),
            (Self::Int | Self::Word, PackValue::Int(_))
                | (Self::LongInt, PackValue::LongInt(_))
                | (Self::Float, PackValue::Float(_))
                | (Self::Reference | Self::Word, PackValue::Reference(_))
        )
    }
}

/// Which way a call goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// `InterpSEvent` (a `WM_*` stub's `SendServerEvt`).
    ToServer,
    /// `InterpCEvent` (a `WM_*` handler).
    ToClient,
}

/// One call's layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallSpec {
    pub function: u32,
    pub direction: Direction,
    /// The `WM_*` package function (stub or handler).
    pub stub: &'static str,
    /// The IFun or script function that sends it, or that its handler forwards to.
    pub name: &'static str,
    pub args: &'static [ArgKind],
}

use ArgKind::{Float, Int, LongInt, Reference, Word};
use Direction::{ToClient, ToServer};

const fn call(
    function: u32,
    direction: Direction,
    stub: &'static str,
    name: &'static str,
    args: &'static [ArgKind],
) -> CallSpec {
    CallSpec {
        function,
        direction,
        stub,
        name,
        args,
    }
}

/// The calls, by package.
#[rustfmt::skip]
pub const CALLS: &[CallSpec] = &[
    // Skills and advancement (`WM_Skill`, package 0x214).
    call(0x8214_000D, ToClient, "WM_Skill 0xD", "CAgent 0x7 (SkillRepository 0x9: a SkillInfo)", &[Reference]),
    call(0x8214_000E, ToClient, "WM_Skill 0xE", "CAgent 0x3 (Agent 0x6C: the SkillRepository)", &[Reference]),
    call(0x8214_0011, ToClient, "WM_Skill 0x11", "CAgent 0x2 (SkillRepository 0x13)", &[Reference]),
    call(0x8214_0014, ToClient, "WM_Skill 0x14", "CAgent 0x1 (SkillRepository 0x14, SendNotice_UpdateSkill)", &[Int]),
    call(0x8214_0001, ToServer, "WM_Skill 0xF", "RequestUntrainSkillCancel", &[]),
    call(0x8214_000C, ToServer, "WM_Skill 0x10", "RequestRaiseSkill", &[Int]),
    call(0x8214_0007, ToServer, "WM_Skill 0x12", "RequestUntrainSkill", &[Int]),
    call(0x8214_0002, ToServer, "WM_Skill 0x13", "RequestTrainSkill", &[Int]),
    // Shortcuts and options (`WM_Player`, package 0x212).
    call(0x8212_0018, ToServer, "WM_Player 0x1A", "SetShortcut", &[Reference, Int]),
    call(0x8212_000C, ToServer, "WM_Player 0x1D", "UploadGameOptions", &[Int, Int, Int, Float]),
    call(0x8212_002D, ToServer, "WM_Player 0x2E", "UploadChatOptions", &[Reference, Reference, Reference, Reference, Reference]),
    call(0x8212_0031, ToServer, "WM_Player 0x32", "UploadUISettings", &[Reference, Reference]),
    // Fellowships (`WM_Fellowship`, package 0x20C).
    call(0x820C_001B, ToServer, "WM_Fellowship 0x22", "DoFellowshipCreate", &[Int, Int, Reference]),
    call(0x820C_000E, ToServer, "WM_Fellowship 0x23", "DoFellowshipDismiss", &[LongInt]),
    call(0x820C_000B, ToServer, "WM_Fellowship 0x24", "CAdmin 0xEC", &[]),
    call(0x820C_000D, ToServer, "WM_Fellowship 0x25", "DoFellowshipRecruitAccept", &[LongInt]),
    call(0x820C_001C, ToServer, "WM_Fellowship 0x29", "DoFellowshipLeave", &[]),
    call(0x820C_0015, ToServer, "WM_Fellowship 0x2A", "DoFellowshipRecruit", &[LongInt]),
    call(0x820C_0003, ToServer, "WM_Fellowship 0x2C", "DoFellowshipDisband", &[]),
    call(0x820C_001A, ToServer, "WM_Fellowship 0x2F", "DoFellowshipRecruitDecline", &[Int, LongInt]),
    call(0x820C_0021, ToClient, "WM_Fellowship 0x21", "CPlayer 0xC1 -> FellowshipSystem 0x3A", &[Word, LongInt]),
    call(0x820C_0026, ToClient, "WM_Fellowship 0x26", "CPlayer 0xF4 -> FellowshipSystem 0x39", &[LongInt]),
    call(0x820C_0027, ToClient, "WM_Fellowship 0x27", "CPlayer 0x8A -> FellowshipSystem 0x3C", &[Word, LongInt]),
    call(0x820C_0028, ToClient, "WM_Fellowship 0x28", "CPlayer 0xFE -> FellowshipSystem 0x8", &[]),
    call(0x820C_002B, ToClient, "WM_Fellowship 0x2B", "CPlayer 0xF2 -> FellowshipSystem 0x3D", &[Word]),
    call(0x820C_002D, ToClient, "WM_Fellowship 0x2D", "CPlayer 0x10 -> FellowshipSystem 0x3B", &[Word, Word, LongInt]),
    call(0x820C_002E, ToClient, "WM_Fellowship 0x2E", "CPlayer 0xDD -> FellowshipSystem 0x3F", &[Word, Word, LongInt]),
    call(0x820C_0030, ToClient, "WM_Fellowship 0x30", "CPlayer 0xC4 -> FellowshipSystem 0x3E", &[Word, Word, LongInt]),
    call(0x820C_0031, ToClient, "WM_Fellowship 0x31", "CPlayer 0x39 -> FellowshipSystem 0x37", &[LongInt]),
    call(0x820C_0032, ToClient, "WM_Fellowship 0x32", "CPlayer 0xBC -> FellowshipSystem 0x38", &[Word, LongInt]),
    call(0x820C_0033, ToClient, "WM_Fellowship 0x33", "CPlayer 0x13 -> FellowshipSystem 0x40", &[Word, LongInt]),
    call(0x820C_003F, ToClient, "WM_Fellowship 0x3F", "CPlayer 0x1E7 -> FellowshipSystem 0x49", &[Word, LongInt]),
    // Allegiances (`WM_Allegiance`, package 0x203).
    call(0x8203_001C, ToServer, "WM_Allegiance 0x31", "AcceptVassal", &[LongInt]),
    call(0x8203_0009, ToServer, "WM_Allegiance 0x32", "BreakAllegiance", &[LongInt]),
    call(0x8203_0010, ToServer, "WM_Allegiance 0x34", "CAdmin 0xD5", &[LongInt]),
    call(0x8203_0018, ToServer, "WM_Allegiance 0x35", "CPlayer 0x5C", &[Reference]),
    call(0x8203_0005, ToServer, "WM_Allegiance 0x36", "QueryAllegianceProfile", &[LongInt]),
    call(0x8203_0028, ToServer, "WM_Allegiance 0x37", "CPlayer 0x2D", &[Reference]),
    call(0x8203_000B, ToServer, "WM_Allegiance 0x38", "CAdmin 0xD5", &[]),
    call(0x8203_000A, ToServer, "WM_Allegiance 0x39", "SwearAllegiance", &[LongInt]),
    call(0x8203_0025, ToServer, "WM_Allegiance 0x3A", "CPlayer 0xFF", &[Reference]),
    call(0x8203_0029, ToServer, "WM_Allegiance 0x3B", "DeclineVassal", &[Int, LongInt]),
    call(0x8203_003C, ToServer, "WM_Allegiance 0x42", "CPlayer 0x15D", &[]),
    call(0x8203_003F, ToServer, "WM_Allegiance 0x43", "CPlayer 0x15D", &[Reference]),
    call(0x8203_0046, ToServer, "WM_Allegiance 0x49", "CPlayer 0x180", &[Reference]),
    call(0x8203_0048, ToServer, "WM_Allegiance 0x4A", "CPlayer 0x17F", &[Reference]),
    call(0x8203_0047, ToServer, "WM_Allegiance 0x4B", "CPlayer 0x183", &[Reference]),
    call(0x8203_0059, ToServer, "WM_Allegiance 0x54", "CPlayer 0x189", &[Reference]),
    call(0x8203_005D, ToServer, "WM_Allegiance 0x55", "CPlayer 0x18D", &[Reference, Reference]),
    call(0x8203_005B, ToServer, "WM_Allegiance 0x56", "CPlayer 0x195/0x196/0x197", &[Int, Int]),
    call(0x8203_005A, ToServer, "WM_Allegiance 0x57", "CPlayer 0x18B", &[Reference]),
    call(0x8203_005C, ToServer, "WM_Allegiance 0x58", "CPlayer 0x18E", &[Reference]),
    call(0x8203_002D, ToClient, "WM_Allegiance 0x2D", "CPlayer 0x11C -> AllegianceSystem 0x4D", &[Word, Word, LongInt]),
    call(0x8203_002E, ToClient, "WM_Allegiance 0x2E", "CPlayer 0x129 -> AllegianceSystem 0x4C", &[Word]),
    call(0x8203_002F, ToClient, "WM_Allegiance 0x2F", "CPlayer 0x136 -> AllegianceSystem 0x4F", &[Word, LongInt]),
    call(0x8203_0030, ToClient, "WM_Allegiance 0x30", "CPlayer 0x134", &[Word]),
    call(0x8203_0033, ToClient, "WM_Allegiance 0x33", "CPlayer 0x24 -> AllegianceSystem 0x4E", &[Word, Word, LongInt]),
    call(0x8203_0053, ToClient, "WM_Allegiance 0x53", "CPlayer 0x186", &[Word, Word]),
];

/// The layout of `function`, if it is one of [`CALLS`].
pub fn call_spec(function: u32) -> Option<&'static CallSpec> {
    CALLS.iter().find(|c| c.function == function)
}

/// One of [`CALLS`] with its packed values.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptCall {
    pub function: u32,
    pub arguments: Vec<PackValue>,
}

impl ScriptCall {
    /// A call whose values match its layout.
    pub fn new(function: u32, arguments: Vec<PackValue>) -> Result<Self, ProtocolError> {
        let spec = call_spec(function).ok_or(ProtocolError::Unsupported("script call"))?;
        if spec.args.len() != arguments.len()
            || !spec.args.iter().zip(&arguments).all(|(k, v)| k.accepts(v))
        {
            return Err(ProtocolError::Invalid("script call arguments"));
        }
        Ok(Self {
            function,
            arguments,
        })
    }

    pub fn spec(&self) -> &'static CallSpec {
        call_spec(self.function).expect("a ScriptCall has a layout")
    }

    /// A call from an `InterpSEvent`/`InterpCEvent`'s `PackBuffer`, or `None` for a function
    /// not in [`CALLS`].
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        call_spec(function)?;
        Some(Args::new(arguments).and_then(|args| Self::new(function, args.into_values())))
    }

    /// The `InterpSEvent` a client call goes out as.
    pub fn to_event(&self) -> ClientEvent {
        ClientEvent::Interp {
            function: self.function,
            arguments: pack_values(&self.arguments),
            encrypted: false,
        }
    }

    /// The `InterpCEvent_Broadcast` body a server call comes in as.
    pub fn to_broadcast(&self) -> Vec<u8> {
        super::broadcast(self.function, &self.arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::native::{ntype, NativeObject, ScriptHash, ShortcutInfo};

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn every_call_is_listed_once() {
        for (i, a) in CALLS.iter().enumerate() {
            assert!(
                CALLS[i + 1..].iter().all(|b| b.function != a.function),
                "{a:?}"
            );
            assert_eq!(a.function >> 28, 8, "{a:?}");
        }
    }

    #[test]
    fn calls_check_their_values_and_pack_byte_exact() {
        // `RequestTrainSkill`: one int.
        let train = ScriptCall::new(0x8214_0002, vec![PackValue::Int(0x41)]).unwrap();
        let (queue, bytes) = train.to_event().encode();
        assert_eq!(queue, 3);
        assert_eq!(bytes, words(&[0x0001_0053, 0x8214_0002, 8, 1, 0x41]));
        assert!(ScriptCall::new(0x8214_0002, vec![PackValue::LongInt(1)]).is_err());
        assert!(ScriptCall::new(0x8214_0002, vec![]).is_err());
        assert!(ScriptCall::new(0x1234, vec![]).is_err());
        // `SetShortcut`: the shortcut (a native `ShortcutInfo`), then its slot.
        let shortcut = PackValue::native(NativeObject::ShortcutInfo(ShortcutInfo::Enum {
            kind: 0x4000_0001,
            value: 0x41,
        }));
        let set = ScriptCall::new(0x8212_0018, vec![shortcut, PackValue::Int(3)]).unwrap();
        let (_, bytes) = set.to_event().encode();
        let mut expected = words(&[0x0001_0053, 0x8212_0018, 44, 5, 1, 1, 0x4400_0100]);
        expected.extend(words(&[0, 0xFFFF_00AC, 0x4000_0001, 0x41, 0, 1, 3]));
        assert_eq!(bytes, expected);
        assert_eq!(ScriptCall::decode(0x8212_0018, &bytes[12..]), Some(Ok(set)));
        // A server call with a word argument takes an int or a reference.
        let chat = PackValue::native(NativeObject::Hash(ScriptHash {
            native_type: ntype::AA_HASH,
            bucket_index: 0,
            keys: vec![],
            values: vec![],
        }));
        for value in [PackValue::Int(2), chat] {
            let call = ScriptCall::new(0x8203_002E, vec![value]).unwrap();
            let bytes = call.to_broadcast();
            assert_eq!(
                ScriptCall::decode(0x8203_002E, &bytes[12..]),
                Some(Ok(call))
            );
        }
        assert!(ScriptCall::decode(0x8212_0016, &[]).is_none());
    }
}
