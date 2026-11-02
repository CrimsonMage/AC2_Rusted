//! GUN, the MSN Gaming Zone service (friends, ignore list, presence, tells, room chat, tickets,
//! support calls). Retail split it in two: Microsoft's `GunDll.dll`, one COM object (`CGun`) that
//! answers ten `IGun*` interfaces and speaks DirectPlay 8 to the GUN server, and Turbine's
//! `NSTurbineGun::TurbineGun` (`ac2/shared/src/gun`), which drives those interfaces for the game.
//!
//! This crate keeps that seam: [`interfaces`] is the ten interfaces in the client PDB's slot order
//! (`decompiled-pdb/gundll/gun_interfaces.csv`), and the messages `CGun::DispatchQueue` hands the
//! client are [`message`]'s types, with the PDB's layouts. The behaviour comes from the verified
//! specs in the analysis workspace (`decompiled-pdb/gundll/spec/README.md`).

pub mod ct;
pub mod dp8_bridge;
pub mod dplay8;
pub mod gundll;
mod hr_table;
pub mod interfaces;
pub mod message;
pub mod message_id;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod turbine_gun;
pub mod turbine_gun_utils;
pub mod turbine_user_status;
pub mod zt;

pub use interfaces::{
    AccountValueType, ConnectStatus, Context, FlagOperation, FlagWord, Guid, GunObject,
    IGunAccount, IGunBrowser, IGunChat, IGunConnection, IGunFriends, IGunHost, IGunHostEnum,
    IGunNews, IGunPersona, IGunQueueRead, IGunSupport, IGunTicket, SupportCallType, TagBlob,
};
pub use message::{ChatRoomAttrs, GunQueueMessage, Message, MessageBody, ResultHeader, UserStatus};
pub use turbine_gun::TurbineGun;

/// A COM `HRESULT` (the PDB's `T_HRESULT`): negative is a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hresult(pub u32);

impl Hresult {
    /// `S_OK`.
    pub const S_OK: Hresult = Hresult(0);
    /// `E_PENDING` (0x8000000A): `IGunConnection::Connect`'s success (the connect goes on
    /// asynchronously), and the queue's "empty" that `CGun::DispatchQueue` turns into `S_OK`.
    pub const E_PENDING: Hresult = Hresult(0x8000_000A);

    /// `SUCCEEDED(hr)`: the sign bit clear.
    pub fn succeeded(self) -> bool {
        (self.0 as i32) >= 0
    }
}

/// The `0xf00` nibble of a [`MessageId`]. `TurbineGun::Read` routes 0x200 (results) and 0x300
/// (events) and answers `ZT_E_INTERNAL` for anything else; 0x100 is the requests' nibble in
/// GunDll's id table (`decompiled-pdb/gundll/gun_messages.csv`, e.g. 0x0f101 `PresentTicketCmd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Request,
    Result,
    Event,
    /// Any other nibble.
    Other(u32),
}

/// `Gun::MessageId`. The named ids are [`message_id`]'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageId(pub u32);

impl MessageId {
    /// `id & 0xf00`: 0x100 request, 0x200 result, 0x300 event.
    pub fn kind(self) -> MessageKind {
        match self.0 & 0xf00 {
            0x100 => MessageKind::Request,
            0x200 => MessageKind::Result,
            0x300 => MessageKind::Event,
            other => MessageKind::Other(other),
        }
    }

    /// `id & 0xff000`: the family `TurbineGun::RcvResult` dispatches a result by (0x2000
    /// account, 0x3000 news, 0x4000 friends, 0x5000 chat, 0x7000 browser, 0x8000 persona,
    /// 0xf000 ticket, 0x10000 support).
    pub fn family(self) -> u32 {
        self.0 & 0xff000
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_utils;
