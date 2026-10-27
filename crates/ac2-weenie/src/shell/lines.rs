//! The lines the chat commands show (`CPlayer` 0x2B, `gmCEntity` 0x1B / 0x1D, all ending in
//! `gmCEntity` 0x1A, `CM_Communication::SendNotice_DisplayStringInfo`) and the strings they
//! name.

use ac2_net::protocol::{StringInfo, StringVariable};

use super::ShellAction;
use crate::notice::GameplayNotice;
use crate::tables::StringTableIds;

/// Display types the handlers pass.
pub(super) const KIND_ERROR: u32 = 1;
pub(super) const KIND_CHAT_SERVER: u32 = 4;
pub(super) const KIND_INFO: u32 = 8;
pub(super) const KIND_TELL_ERROR: u32 = 33;
/// The fellowship and allegiance chat windows (`TEXT_TYPE_FELLOWSHIP`, `TEXT_TYPE_ALLEGIANCE`).
pub(super) const KIND_FELLOWSHIP: u32 = 0x1_0000;
pub(super) const KIND_ALLEGIANCE: u32 = 0x2_0000;

/// String table enums.
pub(super) const TABLE_COMMUNICATION: u32 = crate::tables::TABLE_ENUM_COMMUNICATION;
pub(super) const TABLE_ALLEGIANCE: u32 = crate::tables::TABLE_ENUM_TELL;
pub(super) const TABLE_FELLOWSHIP: u32 = crate::tables::TABLE_ENUM_FELLOWSHIP;
pub(super) const TABLE_CHANNELS: u32 = crate::tables::TABLE_ENUM_CHANNELS;
pub(super) const TABLE_CHAT_WINDOW: u32 = crate::tables::TABLE_ENUM_CHAT_WINDOW;

/// Variables: a player's name, a channel's name.
pub(super) const VAR_PLAYER: u32 = 0x1C06_9478;
pub(super) const VAR_TELL_NAME: u32 = 0xB664_A4E8;
pub(super) const VAR_CHANNEL: u32 = 0x937E_C8A5;

/// `ID_Chat_UnknownCommand` (`ChatWindow::ProcessCommand`).
pub(super) const UNKNOWN_COMMAND: u32 = 0xA968_48DC;
/// Tells: " is not a valid name.", "You must specify a message to send to !".
pub(super) const BAD_NAME: u32 = 0x913D_4776;
pub(super) const NO_MESSAGE: u32 = 0x2BA2_9D26;
/// "You can't /retell until you've used /tell first.", "Someone has to /tell you before you
/// can /reply."
pub(super) const NO_RETELL: u32 = 0xE409_1888;
pub(super) const NO_REPLY: u32 = 0x3CD0_9224;
/// Rooms: not in a fellowship / allegiance, no room yet, the chat server refused.
pub(super) const NO_FELLOWSHIP: u32 = 0x6B9F_0584;
pub(super) const FELLOWSHIP_UNAVAILABLE: u32 = 0x6DB7_1CB3;
pub(super) const NO_ALLEGIANCE: u32 = 0x257A_6C5D;
pub(super) const ALLEGIANCE_UNAVAILABLE: u32 = 0xEEF5_B265;
pub(super) const CHAT_SERVER: u32 = 0x0608_F1BF;
/// Channels: unavailable (the player listens to it) or not subscribed, and their names.
pub(super) const CHANNEL_UNAVAILABLE: u32 = 0x9FEE_074C;
pub(super) const NOT_SUBSCRIBED: u32 = 0x556D_D29F;
pub(super) const CHANNEL_NAMES: [(u32, u32); 4] = [
    (0x10_0000, 0x6EE3_82E5),
    (0x40_0000, 0x7946_03BF),
    (0x20_0000, 0x7DAD_14C5),
    (0x8_0000, 0xF20E_98B3),
];
/// The allegiance channel's name and the "joined" / "left" lines (`CPlayer` 0x134).
pub(super) const ALLEGIANCE_CHANNEL: u32 = 0xC097_7719;
pub(super) const CHANNEL_JOINED: u32 = 0x6B5B_5AAD;
pub(super) const CHANNEL_LEFT: u32 = 0x34C2_844D;
/// Squelch: who, yourself, "Squelching ...", "Unsquelching ...", no GUN service.
pub(super) const SQUELCH_WHO: u32 = 0xF911_D128;
pub(super) const UNSQUELCH_WHO: u32 = 0x96E3_45B1;
pub(super) const SQUELCH_SELF: u32 = 0x7DDA_2C3C;
pub(super) const SQUELCHING: u32 = 0x309D_0867;
pub(super) const UNSQUELCHING: u32 = 0x6AE5_BC25;
pub(super) const NO_SQUELCH_SERVICE: u32 = 0x3E6E_8EF5;
/// The GUN results (`CPlayer` 0x32, 0x5D, 0x7): done, and the errors by status.
pub(super) const SQUELCHED: u32 = 0x12CB_9094;
pub(super) const UNSQUELCHED: u32 = 0x0929_357F;
pub(super) const SQUELCH_LIST_EMPTY: u32 = 0x306C_C339;
pub(super) const SQUELCH_LIST: u32 = 0xF69C_DCC5;
pub(super) const SQUELCH_LIST_ERROR: u32 = 0x9140_E535;
pub(super) const ADD_ERRORS: [(u32, u32); 4] = [
    (0xC740, 0x42B2_EBD5),
    (0xC73D, 0x134C_E336),
    (0xC73B, 0x04BA_081B),
    (0xC741, 0x9140_E535),
];
pub(super) const DELETE_ERRORS: [(u32, u32); 4] = [
    (0xC73E, 0xA34C_EFA4),
    (0xC73D, 0x134C_E336),
    (0xC73B, 0x04BA_081B),
    (0xC741, 0x9140_E535),
];
/// The squelch command's sub-commands, in table enum 0x41000007: add, remove, status.
pub const SQUELCH_WORDS: [u32; 3] = [0x0C9C_0D60, 0x7B6B_F742, 0x76E8_C961];

/// `StringInfo::SetLiteralValue`.
pub(super) fn literal(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.to_owned()),
        ..StringInfo::default()
    }
}

/// `StringInfo::SetStringIDandTableEnum`.
pub(super) fn line(tables: &StringTableIds, table_enum: u32, id: u32) -> StringInfo {
    StringInfo {
        string_id: id,
        table_id: tables.did(table_enum),
        ..StringInfo::default()
    }
}

/// `StringInfo::AddVariable_String` / `AddVariable_StringInfo`.
pub(super) fn with(mut info: StringInfo, variable: u32, value: StringInfo) -> StringInfo {
    info.variables
        .push((variable, StringVariable::Nested(Box::new(value))));
    info
}

/// `gmCEntity` 0x1A: show `text` with display type `kind`.
pub(super) fn show(kind: u32, text: StringInfo) -> ShellAction {
    ShellAction::Notice(GameplayNotice::DisplayStringInfo { kind, text })
}

/// `gmCEntity` 0x1B: a table string.
pub(super) fn show_line(
    tables: &StringTableIds,
    table_enum: u32,
    id: u32,
    kind: u32,
) -> ShellAction {
    show(kind, line(tables, table_enum, id))
}

/// `CPlayer` 0x103 / 0xE4 / 0x14C when `RequestGUN_SendToRoom` fails: "Unable to
/// communicate with the GUN chat server".
pub(crate) fn room_refused(tables: &StringTableIds) -> GameplayNotice {
    GameplayNotice::DisplayStringInfo {
        kind: KIND_CHAT_SERVER,
        text: line(tables, TABLE_CHANNELS, CHAT_SERVER),
    }
}
