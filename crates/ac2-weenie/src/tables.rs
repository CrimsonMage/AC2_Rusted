//! The string tables the player-script stand-in names by enum: `StringInfo::
//! SetStringIDandTableEnum` resolves the enum with `FileArray::GetDIDFromEnum(enum,
//! 0x40000004)` and keeps the DID only when `MasterDBMap::DivineType` says it is a string table
//! (otherwise 0). The DIDs come from the DATs, so the caller resolves them off the frame thread
//! and hands them to the session.

use std::collections::BTreeMap;

/// String table enum -> DID.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringTableIds {
    ids: BTreeMap<u32, u32>,
}

/// String table enum 0x41000001 (tells).
pub const TABLE_ENUM_TELL: u32 = 0x4100_0001;
/// String table enum 0x41000004 (the communication strings).
pub const TABLE_ENUM_COMMUNICATION: u32 = 0x4100_0004;
/// String table enum 0x41000019 (the usage strings).
pub const TABLE_ENUM_USAGE: u32 = 0x4100_0019;
/// String table enum 0x4100000D (the fellowship strings).
pub const TABLE_ENUM_FELLOWSHIP: u32 = 0x4100_000D;
/// String table enum 0x4100001B (chat channels, emotes, the chat server).
pub const TABLE_ENUM_CHANNELS: u32 = 0x4100_001B;
/// String table enum 0x41000018 (the chat window's strings).
pub const TABLE_ENUM_CHAT_WINDOW: u32 = 0x4100_0018;
/// String table enum 0x4100001C (the emote strings: `gmFriendsUI::HandleStatusChange`'s
/// `StringInfo` for `DoEmote`).
pub const TABLE_ENUM_EMOTE: u32 = 0x4100_001C;

impl StringTableIds {
    /// The enums the stand-in uses.
    pub const ENUMS: [u32; 7] = [
        TABLE_ENUM_TELL,
        TABLE_ENUM_COMMUNICATION,
        TABLE_ENUM_USAGE,
        TABLE_ENUM_FELLOWSHIP,
        TABLE_ENUM_CHANNELS,
        TABLE_ENUM_CHAT_WINDOW,
        TABLE_ENUM_EMOTE,
    ];

    pub fn insert(&mut self, table_enum: u32, did: u32) {
        self.ids.insert(table_enum, did);
    }

    /// The table's DID, or 0 when it did not resolve (retail's result for a failed lookup).
    pub fn did(&self, table_enum: u32) -> u32 {
        self.ids.get(&table_enum).copied().unwrap_or(0)
    }
}
