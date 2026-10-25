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
mod catalog;
mod consignment;
mod craft;
mod craft_records;
mod message;
mod request;
mod skill;
mod store;
mod trade;

pub use calls::{call_spec, ArgKind, CallSpec, Direction, ScriptCall, CALLS};
pub use catalog::StoreCatalog;
pub use consignment::{Consignment, ConsignmentList};
pub use craft::CraftMessage;
pub use message::{broadcast, ScriptMessage};
pub use request::ScriptRequest;
pub use skill::{SkillMessage, SkillRecord, SkillRepository};
pub use store::{GameSale, Purchase, StoreMessage, StoreView};
pub use trade::TradeMessage;

use crate::protocol::native::NativeObject;
use crate::protocol::pack::{unpack_values, PackValue, PackedReference};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// Script function ids, by the `WM_*` package that sends or receives them (package
/// 0x212 `WM_Player`, 0x20A `WM_Examination`, 0x205 `WM_Communication`).
pub mod function {
    /// S->C, `WM_Skill` 0xD -> `CAgent` 0x7: one `SkillInfo` added or replaced.
    pub const SKILL_UPDATE: u32 = 0x8214_000D;
    /// S->C, `WM_Skill` 0xE -> `CAgent` 0x3: the whole `SkillRepository` (the credits too).
    pub const SKILL_SET_REPOSITORY: u32 = 0x8214_000E;
    /// S->C, `WM_Skill` 0x11 -> `CAgent` 0x2: the skill table (an `ARHash` of `SkillInfo`s).
    pub const SKILL_SET_SKILLS: u32 = 0x8214_0011;
    /// S->C, `WM_Skill` 0x14 -> `CAgent` 0x1: one skill removed (its enum, an int).
    pub const SKILL_REMOVE: u32 = 0x8214_0014;
    /// C->S, `WM_Skill` stub 0x13 (`RequestTrainSkill`): the skill, an int.
    pub const SKILL_TRAIN: u32 = 0x8214_0002;
    /// C->S, `WM_Skill` stub 0x12 (`RequestUntrainSkill`): the skill, an int.
    pub const SKILL_UNTRAIN: u32 = 0x8214_0007;
    /// C->S, `WM_Skill` stub 0x10 (`RequestRaiseSkill`): the skill, an int.
    pub const SKILL_RAISE: u32 = 0x8214_000C;
    /// C->S, `WM_Skill` stub 0xF (`RequestUntrainSkillCancel`): no values.
    pub const SKILL_CANCEL_UNTRAIN: u32 = 0x8214_0001;

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
    /// C->S, `WM_Communication` stub 0x15 (`CCommunicationSystem` 0x5, from `CPlayer` 0x5E,
    /// the emote command): the text, a `wpstring`.
    pub const EMOTE_TEXT: u32 = 0x8205_0004;
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
    /// C->S, `WM_Trade` stub 0x2F (`gmCWInterface::OpenTradeNegotiations`, IFun 0xB400B8): the
    /// other party (long int).
    pub const OPEN_TRADE: u32 = 0x8215_002F;
    /// C->S, `WM_Trade` stub 0x3E (`CloseTradeNegotiations`, IFun 0xB400B9): no arguments.
    pub const CLOSE_TRADE: u32 = 0x8215_003E;
    /// C->S, `WM_Trade` stub 0x35 (`AcceptTrade`, IFun 0xB400BA): one reference (the stub packs
    /// the accepting side's script instance).
    pub const ACCEPT_TRADE: u32 = 0x8215_0035;
    /// C->S, `WM_Trade` stub 0x23 (`DeclineTrade`, IFun 0xB400BB): no arguments.
    pub const DECLINE_TRADE: u32 = 0x8215_0023;
    /// C->S, `WM_Trade` stub 0x15 (`OfferItemForTrade`, IFun 0xB400BC): the amount (int), then
    /// the item (long int).
    pub const OFFER_ITEM: u32 = 0x8215_0015;
    /// C->S, `WM_Trade` stub 0x3F (`RevokeItemFromTrade`, IFun 0xB400BD): the item (long int).
    pub const REVOKE_ITEM: u32 = 0x8215_003F;
    /// S->C, `WM_Trade` forwarder 0x46 (`CPlayer` 0xF5 -> `CM_Trade::SendNotice_Trade_BeRegistered`):
    /// the wire packs the partner's name, the partner, then the initiator (the handler's
    /// parameters reversed).
    pub const TRADE_REGISTERED: u32 = 0x8215_0046;
    /// S->C, `WM_Trade` forwarder 0x4E (`CPlayer` 0x08 -> `BeOpened`): no arguments.
    pub const TRADE_OPENED: u32 = 0x8215_004E;
    /// S->C, `WM_Trade` forwarder 0x53 (`CPlayer` 0x102 -> `BeClosed`): the kind (int), then the
    /// source (long int).
    pub const TRADE_CLOSED: u32 = 0x8215_0053;
    /// S->C, `WM_Trade` forwarder 0x50 (`CPlayer` 0x35 -> `BeAccepted`): the source (long int).
    pub const TRADE_ACCEPTED: u32 = 0x8215_0050;
    /// S->C, `WM_Trade` forwarder 0x56 (`CPlayer` 0xB0 -> `BeDeclined`): the source (long int).
    pub const TRADE_DECLINED: u32 = 0x8215_0056;
    /// S->C, `WM_Trade` forwarder 0x52 (`CPlayer` 0xC9 -> `BeReset`): no arguments.
    pub const TRADE_RESET: u32 = 0x8215_0052;
    /// S->C, `WM_Trade` forwarder 0x51 (`CPlayer` 0x106 -> `BeAcceptanceCleared`): no
    /// arguments. Forwarder 0x4C (`CPlayer` 0x29) raises the same notice.
    pub const TRADE_ACCEPTANCE_CLEARED: u32 = 0x8215_0051;
    /// S->C, `WM_Trade` forwarder 0x4C (`CPlayer` 0x29 -> `BeAcceptanceCleared`): as 0x51.
    pub const TRADE_ACCEPTANCE_CLEARED_ALT: u32 = 0x8215_004C;
    /// S->C, `WM_Trade` forwarder 0x44 (`CPlayer` 0x122 -> `BeOffered`): the amount (int), the
    /// item (long int), the source (long int).
    pub const TRADE_OFFERED: u32 = 0x8215_0044;
    /// S->C, `WM_Trade` forwarder 0x48 (`CPlayer` 0x10F -> `BeRevoked`): the item (long int),
    /// the source (long int).
    pub const TRADE_REVOKED: u32 = 0x8215_0048;
    /// C->S, `WM_Store` stub 0x4D (`RequestEnterStore`, IFun 0xB40137, via `CStoreSystem` 0x10):
    /// the store (int), then the storekeeper (long int).
    pub const STORE_ENTER: u32 = 0xB83A_0044;
    /// C->S, `WM_Store` stub 0x12 (`RequestLeaveStore`, IFun 0xB40131, via `CStoreSystem` 0x2):
    /// the storekeeper (long int).
    pub const STORE_LEAVE: u32 = 0xB83A_0003;
    /// C->S, `WM_Store` stub 0x14 (`PurchaseConfirmed`, IFun 0xB40130, via `CStoreSystem` 0x4):
    /// the purchase's `TransactionBlob` (a package reference).
    pub const STORE_PURCHASE: u32 = 0xB83A_000B;
    /// C->S, `WM_Store` stub 0x4E (`RequestSell`, IFun 0xB40138, the sell body in the client's
    /// library at 0x11642C): an int (the body's local 0), the price (int), the quantity (int), the
    /// item (long int), a long int (the body's local 3). Not decoded yet (no shopper opens).
    pub const STORE_SELL: u32 = 0xB83A_0043;
    /// S->C, `WM_Store` forwarder 0x13 (`CPlayer` 0x1DE -> `CStoreSystem` 0x6 -> `ViewStore`): the
    /// store (a package reference to a `StoreView`, see [`StoreView`]), the storekeeper (long int).
    /// The forwarder reads the store as a one-slot reference, so an int store crashes.
    pub const STORE_VIEW: u32 = 0xB83A_0013;
    /// S->C, `WM_Store` forwarder 0x2A (`CPlayer` 0x1E2 -> `CStoreSystem` 0x8 -> `UpdateStore`):
    /// the store (a package reference to a `StoreView`, as `ViewStore`), the storekeeper (long
    /// int). The body needs an open shopper (`CPlayer` 0x1EC) or it logs and returns.
    pub const STORE_UPDATE: u32 = 0xB83A_002A;
    /// S->C, `WM_Store` forwarder 0x10 (`CPlayer` 0x1DD -> `CStoreSystem` 0x14 -> `LeaveStore`):
    /// the storekeeper (long int).
    pub const STORE_LEFT: u32 = 0xB83A_0010;
    /// S->C, `WM_Store` forwarder 0x29 (`CPlayer` 0x1E3 -> `CStoreSystem` 0x9 -> `UpdateSale`):
    /// the stock (int), the item (long int), the storekeeper (long int).
    pub const STORE_UPDATE_SALE: u32 = 0xB83A_0029;
    /// C->S, `WM_Store` stub 0x46 (`RequestEnterConsignment`, IFun 0xB40136): the storekeeper
    /// (long int). The stub sends it only with a shopper open (`CStoreSystem`'s shopper check).
    pub const STORE_ENTER_CONSIGNMENT: u32 = 0xB83A_0046;
    /// C->S, `WM_Store` stub 0x45 (`RequestLeaveConsignment`, IFun 0xB40135): the storekeeper
    /// (long int).
    pub const STORE_LEAVE_CONSIGNMENT: u32 = 0xB83A_0045;
    /// S->C, `WM_Store` forwarder 0x53 (`CPlayer` 0x1F1 -> `CStoreSystem` 0x17 -> `ViewConsignment`):
    /// the consignments (an `RList` of `Consignment`s, see [`super::ConsignmentList`]), the
    /// storekeeper (long int). The body closes the open store and catalog first, as `ViewStore`'s
    /// does.
    pub const STORE_VIEW_CONSIGNMENT: u32 = 0xB83A_0053;
    /// S->C, `WM_Store` forwarder 0x52 (`CPlayer` 0x1F3 -> `CStoreSystem` 0xF -> `ViewCatalog`):
    /// the store sizes (an `AAHash`), the `StoreGroup` DID, the storekeeper (long int); see
    /// [`super::StoreCatalog`].
    pub const STORE_VIEW_CATALOG: u32 = 0xB83A_0052;
    /// S->C, `WM_Store` forwarder 0x57 (`CPlayer` 0x1F6 -> `CStoreSystem` 0x16 -> `UpdateConsignment`):
    /// the consignments (an `RList` of `Consignment`s), the storekeeper (long int).
    pub const STORE_UPDATE_CONSIGNMENT: u32 = 0xB83A_0057;
    /// C->S, `WM_Craft` stub 0x06 (`RequestAddIngredient`, IFun 0xB4009D): the spinner (int), the
    /// object (long int), the slot's ordinal (int), the recipe (int), in stub order.
    pub const CRAFT_ADD_INGREDIENT: u32 = 0x8206_0006;
    /// C->S, `WM_Craft` stub 0x12 (`RequestExecuteCraft`, IFun 0xB4009E): a long int (the stub's
    /// `PackageIDTable` helper), the ingredient table (`ALHash`), the spinner (int), the recipe
    /// (int), in stub order.
    pub const CRAFT_EXECUTE: u32 = 0x8206_0012;
    /// C->S, `WM_Craft` stub 0x2E (`RequestRaiseCraftSkill`, IFun 0xB4011C): the craft skill (int).
    pub const CRAFT_RAISE_SKILL: u32 = 0x8206_002E;
    /// S->C, `WM_Craft` forwarder 0x16 (`CPlayer` 0x30 -> `CM_Craft::SendNotice_AddIngredient_Done`).
    pub const CRAFT_ADD_INGREDIENT_DONE: u32 = 0x8206_0016;
    /// S->C, `WM_Craft` forwarder 0x1C (`CPlayer` 0xF7 -> `CM_Craft::SendNotice_ExecuteCraft_Done`).
    pub const CRAFT_EXECUTE_DONE: u32 = 0x8206_001C;
    /// S->C, `WM_Craft` forwarder 0x18 (`CPlayer` 0xF9): calls efun 0x8F0003 (unbound in the glue)
    /// and raises `CM_Craft::SendNotice_UpdateCraftPanel`; its argument is not traced.
    pub const CRAFT_UPDATE_PANEL_18: u32 = 0x8206_0018;
    /// S->C, `WM_Craft` 0x19 -> `CPlayer` 0xC3 -> `CPlayer` 0x1B: the player's `CraftRegistry`
    /// (package 0x1D2) stored in player slot 0x21 (a null reference makes an empty one), then
    /// `UpdateCraftPanel`. The skill hash is slot 5, the recipe hash slot 6, both keyed by DID.
    pub const CRAFT_SET_REGISTRY: u32 = 0x8206_0019;
    /// S->C, `WM_Craft` 0x1B -> `CPlayer` 0x105 -> `CraftSystem` 0x1B: a recipe's DID (int). An
    /// unknown recipe gets a new `RecipeRecord` from its DID (`CraftRegistry` 0x5, `RecipeRecord`
    /// 0xC); a known one only raises the system's messages. Then `UpdateCraftPanel`.
    pub const CRAFT_LEARN_RECIPE: u32 = 0x8206_001B;
    /// S->C, `WM_Craft` 0x27 -> `CPlayer` 0x1AE -> `CraftSystem` 0x26 -> `CraftRegistry` 0x1A: one
    /// `CraftSkillRecord` (a heap object) set in the registry's skill hash by its DID (`ARHash::set`,
    /// added or replaced), then `CM_Craft::SendNotice_UpdateCraftSkill` and `UpdateCraftPanel`.
    pub const CRAFT_SET_SKILL: u32 = 0x8206_0027;
    /// S->C, `WM_Craft` 0x34 -> `CPlayer` 0x1C4: an `RList` of craft skill records, each set as
    /// [`CRAFT_SET_SKILL`] does.
    pub const CRAFT_SET_SKILLS: u32 = 0x8206_0034;
    /// S->C, `WM_Craft` 0x1A -> `CPlayer` 0x61 -> `CraftSystem` 0x15 -> `CraftRegistry` 0x12: one
    /// `RecipeRecord` (a heap object) replacing the known record of its DID in the recipe hash; an
    /// unknown recipe (`CraftRegistry` 0xC, `ARHash::contains`) returns 0x9C51 and changes nothing.
    /// The forwarder raises `UpdateCraftPanel` either way.
    pub const CRAFT_UPDATE_RECIPE: u32 = 0x8206_001A;
    /// S->C, `WM_Craft` 0x37 -> `CPlayer` 0x1C5: an `RList` of recipe records, each updated as
    /// [`CRAFT_UPDATE_RECIPE`] does.
    pub const CRAFT_UPDATE_RECIPES: u32 = 0x8206_0037;
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
