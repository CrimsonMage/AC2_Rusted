#![forbid(unsafe_code)]

//! The Asheron's Call 2 UDP transport.
//!
//! A datagram is a 20-byte header (`ProtoHeader`), the optional headers its flags name in
//! ascending flag order, then blob fragments. The core here is sans-IO: [`Session`] turns
//! received datagrams into completed blobs and queued blobs into datagrams; [`NetClient`] runs
//! the logon handshake and a [`Session`] on a socket thread.

mod checksum;
mod client;
mod error;
mod handshake;
mod header;
mod isaac;
mod packet;
pub mod protocol;
mod session;
mod string;

pub use checksum::checksum32;
pub use client::{NetClient, NetClientConfig, NetEvent};
pub use error::NetError;
pub use handshake::{Connect, Credential, Logon};
pub use header::{flags, PacketHeader, HEADER_LEN};
pub use isaac::Isaac;
pub use packet::{decode_packet, Fragment, InboundPacket, FRAGMENT_HEADER_LEN};
pub use session::{Blob, Inbound, Session};
pub use string::{pack_string, pack_wide_string, unpack_string, unpack_wide_string};
