//! Gameplay messages: script calls between the player's weenie scripts and the server.
//!
//! The client sends a message through its `WM_*` package's sender stub in the script library
//! (WLib 0x56000005): the stub packs its arguments into a `PackBuffer` and calls
//! `CWInterface::SendServerEvt` with the server function id `0x8000_0000 | package << 16 | n`
//! (`InterpSEvent`, 0x10053). The server calls a client function the same way
//! (`InterpCEvent`), which the player's `WM_*` package forwards to `CPlayer`, `gmCEntity` or a
//! `C*System` script. Function ids, argument order and pack types are read from those stubs and
//! handlers; a handler takes the values in `PackBuffer` order.

mod calls;
mod message;
mod request;

pub use calls::{call_spec, ArgKind, CallSpec, Direction, ScriptCall, CALLS};
pub use message::{broadcast, ScriptMessage};
pub use request::ScriptRequest;

use crate::protocol::native::NativeObject;
use crate::protocol::pack::{unpack_values, PackValue, PackedReference};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// Script function ids, by the `WM_*` package that sends or receives them (package
/// 0x212 `WM_Player`, 0x20A `WM_Examination`, 0x205 `WM_Communication`).
pub mod function {
    /// C->S, `WM_Player` stub 0x28 (from `CPlayer` 0x6A, `SetSelection`): long int.
    pub const SET_TARGET: u32 = 0x8212_0016;
    /// C->S, `WM_Examination` stub 0x8 (`CExaminationSystem`): `ExaminationRequest`.
    pub const EXAMINE: u32 = 0x820A_0007;
    /// C->S, `WM_Communication` stub 0x14 (`CCommunicationSystem` 0xD, from `CPlayer` 0xDB):
    /// int, then the text (`StringInfo`).
    pub const SAY: u32 = 0x8205_0006;
    /// C->S, `WM_Communication` stub 0xF (`CCommunicationSystem` 0xB): int, the text, the
    /// recipient's name (both `StringInfo`).
    pub const TELL: u32 = 0x8205_0002;
    /// C->S, `WM_Communication` stub 0xE (`CCommunicationSystem` 0x8, from `CPlayer` 0x15B,
    /// IFun `DoEmote`): the emote, one int.
    pub const EMOTE: u32 = 0x8205_000B;
    /// S->C, `WM_Player` 0x27 -> `CPlayer` 0x65: the player's login state (fourteen 32-bit
    /// arguments: money, the options profile, the faction, inventory and effect systems'
    /// data...), after which `CPlayer` raises the hero status and the vitals.
    pub const INIT_PLAYER: u32 = 0x8212_0027;
    /// C->S, `WM_Usage` stub 0x22 (`CUsageSystem` 0x6, from `HandleItemUsage` 0x9 or
    /// `TryToAddToUsageList` 0x3): a `UsageDesc` reference.
    pub const USE: u32 = 0x8216_0016;
    /// S->C, `WM_Usage` 0x21 -> `CPlayer` 0x7D -> `CUsageSystem` 0xA: the `UsageDesc` back,
    /// with its outcome.
    pub const USAGE_DONE: u32 = 0x8216_0021;
    /// S->C, `WM_Usage` 0x23 -> `CPlayer` 0x67 -> `CUsageSystem` 0x2: a use failed (the error,
    /// the target, the item).
    pub const USAGE_ERROR: u32 = 0x8216_0023;
    /// C->S, `WM_Inventory` stub 0x57 (`CInventorySystem` 0x12, `DirectiveMoveItem`): an
    /// `InvMoveDesc`.
    pub const MOVE_ITEM: u32 = 0x820E_0036;
    /// C->S, `WM_Inventory` stub 0x5A (`CInventorySystem` 0x19, a move within the player's own
    /// pack): an `InvMoveDesc`.
    pub const REORGANIZE_CONTENTS: u32 = 0x820E_0049;
    /// C->S, `WM_Inventory` stub 0x59 (`CInventorySystem` 0x16, `DirectiveEquipItem`): an
    /// `InvEquipDesc`.
    pub const EQUIP_ITEM: u32 = 0x820E_004F;
    /// C->S, `WM_Inventory` stub 0x5B (`CInventorySystem` 0xA, `DirectiveUnEquipItem`): an
    /// `InvEquipDesc`.
    pub const UNEQUIP_ITEM: u32 = 0x820E_0015;
    /// C->S, `WM_Inventory` stub 0x52 (`CInventorySystem` 0x11, from `CContainer` 0x9): open a
    /// container (long int).
    pub const OPEN_CONTAINER: u32 = 0x820E_000C;
    /// C->S, `WM_Inventory` stub 0x54 (`CInventorySystem` 0x18, from `CContainer` 0x3): close a
    /// container (long int).
    pub const CLOSE_CONTAINER: u32 = 0x820E_0047;
    /// C->S, `WM_Inventory` stub 0x69 (`CInventorySystem` 0x1C, from `CPlayer` 0x157
    /// `LootAllFromContainer`): an `InvTakeAllDesc`.
    pub const LOOT_ALL: u32 = 0x820E_0068;
    /// C->S, `WM_Inventory` stub 0x6E (`CInventorySystem` 0x21, from `CPlayer` 0x158
    /// `TransmuteAllFromContainer`): an `InvTransmuteAllDesc`.
    pub const TRANSMUTE_ALL: u32 = 0x820E_006C;
    /// S->C, `WM_Inventory` 0x6A -> `CPlayer` 0x14F -> `CInventorySystem` 0x1D.
    pub const LOOT_ALL_DONE: u32 = 0x820E_006A;
    /// S->C, `WM_Inventory` 0x6D -> `CPlayer` 0x159 -> `CInventorySystem` 0x20.
    pub const TRANSMUTE_ALL_DONE: u32 = 0x820E_006D;
    /// S->C, `WM_Inventory` 0x5D -> `CPlayer` 0x8C -> `CInventorySystem` 0xF: a container
    /// opened (error, packs, items, the container).
    pub const OPEN_CONTAINER_DONE: u32 = 0x820E_005D;
    /// S->C, `WM_Inventory` 0x60 -> `CPlayer` 0x21 -> `CInventorySystem` 0x1B
    /// (`CloseContainer_Done`): error, the container.
    pub const CLOSE_CONTAINER_DONE: u32 = 0x820E_0060;
    /// S->C, `WM_Inventory` 0x53 -> `CPlayer` 0x37 -> `CInventorySystem` 0x1A: a container's
    /// contents (segments, packs, items).
    pub const CONTAINER_CONTENTS: u32 = 0x820E_0053;
    /// S->C, `WM_Inventory` 0x5F -> `CPlayer` 0x6F -> `CInventorySystem` 0x6: the equipment.
    pub const EQUIPMENT: u32 = 0x820E_005F;
    /// S->C, `WM_Inventory` 0x5E -> `CPlayer` 0xE5 -> `CInventorySystem` 0x2
    /// (`MoveItem_Done`): the move's `InvMoveDesc` back.
    pub const MOVE_ITEM_DONE: u32 = 0x820E_005E;
    /// S->C, `WM_Inventory` 0x55 -> `CPlayer` 0x11E -> `CInventorySystem` 0xE
    /// (`ReorganizeContents_Done`).
    pub const REORGANIZE_CONTENTS_DONE: u32 = 0x820E_0055;
    /// S->C, `WM_Inventory` 0x51 -> `CPlayer` 0xC8 -> `CInventorySystem` 0xC (`EquipItem_Done`).
    pub const EQUIP_ITEM_DONE: u32 = 0x820E_0051;
    /// S->C, `WM_Inventory` 0x56 -> `CPlayer` 0x7B -> `CInventorySystem` 0x14
    /// (`UnEquipItem_Done`).
    pub const UNEQUIP_ITEM_DONE: u32 = 0x820E_0056;
    /// S->C, `WM_Player` 0x2C -> `CPlayer` 0x34: `SelectionInfo`, then the object (long int).
    pub const UPDATE_SELECTION_INFO: u32 = 0x8212_002C;
    /// S->C, `WM_Examination` 0x9 -> `CPlayer` 0xDA -> `CExaminationSystem` 0x2:
    /// `ExaminationProfile`.
    pub const EXAMINATION_PROFILE: u32 = 0x820A_0009;
    /// S->C, `WM_Communication` 0x10 -> `gmCEntity` 0x1 -> `CCommunicationSystem` 0x6: someone
    /// speaks (int, text, speaker's name, speaker).
    pub const HEAR_SPEECH: u32 = 0x8205_0010;
    /// S->C, `WM_Communication` 0x11 -> `gmCEntity` 0x1F -> 0x1A: a `StringInfo` and its
    /// display type, straight to `CM_Communication::SendNotice_DisplayStringInfo`.
    pub const DISPLAY_STRING_INFO: u32 = 0x8205_0011;
    /// S->C, `WM_Communication` 0x12 -> `gmCEntity` 0x1C -> 0x1D: a `wpstring` made a literal,
    /// and its display type.
    pub const DISPLAY_TEXT: u32 = 0x8205_0012;
    /// S->C, `WM_Communication` 0x17 -> `gmCEntity` 0x13 -> 0x2 (`HearTell`) ->
    /// `CCommunicationSystem` 0x1: a tell (kind, text, teller's name, teller).
    pub const HEAR_TELL: u32 = 0x8205_0017;
    /// S->C, `WM_Communication` 0x23 -> `gmCEntity` 0x21 -> 0x1A: as [`DISPLAY_STRING_INFO`].
    pub const DISPLAY_STRING_INFO_2: u32 = 0x8205_0023;
}

/// The values a handler takes, in order, each of the type the stub packed.
pub(crate) struct Args {
    values: std::vec::IntoIter<PackValue>,
}

impl Args {
    pub(crate) fn new(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(Self {
            values: unpack_values(bytes)?.into_iter(),
        })
    }

    fn next(&mut self) -> Result<PackValue, ProtocolError> {
        self.values
            .next()
            .ok_or(ProtocolError::Invalid("missing script argument"))
    }

    pub(crate) fn int(&mut self) -> Result<u32, ProtocolError> {
        match self.next()? {
            PackValue::Int(v) => Ok(v),
            _ => Err(ProtocolError::Invalid("script argument: int")),
        }
    }

    pub(crate) fn long(&mut self) -> Result<u64, ProtocolError> {
        match self.next()? {
            PackValue::LongInt(v) => Ok(v),
            _ => Err(ProtocolError::Invalid("script argument: long int")),
        }
    }

    /// A 32-bit value: an int or a reference.
    pub(crate) fn word(&mut self) -> Result<PackValue, ProtocolError> {
        match self.next()? {
            v @ (PackValue::Int(_) | PackValue::Reference(_)) => Ok(v),
            _ => Err(ProtocolError::Invalid("script argument: word")),
        }
    }

    /// A reference (a packed reference graph).
    pub(crate) fn reference(&mut self) -> Result<PackedReference, ProtocolError> {
        match self.next()? {
            PackValue::Reference(r) => Ok(r),
            _ => Err(ProtocolError::Invalid("script argument: reference")),
        }
    }

    /// A reference whose root holds a native object.
    pub(crate) fn native(&mut self) -> Result<NativeObject, ProtocolError> {
        match self.next()? {
            PackValue::Reference(r) => r
                .root_native()
                .cloned()
                .ok_or(ProtocolError::Invalid("script argument: native object")),
            _ => Err(ProtocolError::Invalid("script argument: reference")),
        }
    }

    pub(crate) fn string_info(&mut self) -> Result<StringInfo, ProtocolError> {
        match self.native()? {
            NativeObject::StringInfo(info) => Ok(info),
            _ => Err(ProtocolError::Invalid("script argument: StringInfo")),
        }
    }

    /// The values not yet taken.
    pub(crate) fn into_values(self) -> Vec<PackValue> {
        self.values.collect()
    }

    pub(crate) fn finish(mut self) -> Result<(), ProtocolError> {
        match self.values.next() {
            None => Ok(()),
            Some(_) => Err(ProtocolError::Invalid("extra script argument")),
        }
    }
}

/// A `StringInfo` argument the client builds from text (`gStringInfo::SetLiteralValue`).
pub(crate) fn literal(text: &StringInfo) -> PackValue {
    PackValue::native(NativeObject::StringInfo(text.clone()))
}
