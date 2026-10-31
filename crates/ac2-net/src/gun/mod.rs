//! `NSTurbineGun::TurbineGun`: the client side of the MSN Gaming Zone "GUN" service that the
//! retail client reaches through `GunDll.dll` (ticket login, friends, ignore list, presence,
//! tells). Retail built `TurbineGun` only for a `-gunserver` argument (`gmClient::StartupGUN`),
//! connected, and polled it from the main loop (`TurbineGun::CheckForMessages`).
//!
//! The retail client never spoke GUN itself: `GunDll.dll` did, over a session layer (an RSA key
//! exchange) that is not decoded and whose servers are gone. This port keeps what the client
//! program did (the connect, ticket and request/result sequence, the `Name@World` mapping, the
//! result codes) and speaks the AC2e server's GUN transport, `docs/gun-protocol.md` of AC2e: a
//! big-endian length and one XML element per frame, named as `GunDll.dll` names its messages.
//!
//! [`TurbineGun`] is sans IO: it queues frames ([`TurbineGun::take_output`]) and decodes what
//! arrives ([`TurbineGun::receive`]) into [`GunEvent`]s. [`GunConnection`] drives it from a TCP
//! socket, one [`GunConnection::pump`] per frame as the client's loop polled GUN.

mod connection;
mod turbine_gun;
mod xml;

pub use connection::GunConnection;
pub use turbine_gun::{
    GunError, GunEvent, GunUserStatus, TurbineGun, ZT_E_ACCOUNT_BANNED, ZT_E_ALREADY_IN_LIST,
    ZT_E_NOT_IN_LIST,
};
pub use xml::{frame, Elem, FrameReader};

/// The default GUN port (`gmClient::gmClient`: `m_gunPort = 0x906`).
pub const DEFAULT_GUN_PORT: u16 = 2310;

/// `IGunFriends::Flags` operations (`NSTurbineGun::FLAG_OP`).
pub mod flag_op {
    pub const GET: u32 = 0;
    pub const SET: u32 = 1;
    pub const CLEAR: u32 = 2;
}

/// `IGunFriends::Flags` words (`NSTurbineGun::FLAG_WHICH`).
pub mod flag_word {
    pub const CONTROL: u32 = 0;
    pub const STATE: u32 = 1;
}
