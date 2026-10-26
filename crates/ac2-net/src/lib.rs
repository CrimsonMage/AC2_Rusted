#![forbid(unsafe_code)]

//! The Asheron's Call 2 UDP transport and event codecs.
//!
//! A datagram is a 20-byte `ProtoHeader`, the optional headers its bits name in ascending mask
//! order, then blob fragments. [`ClientNet`] is the client side, sans IO, one retail frame per
//! [`ClientNet::use_time`]; [`NetClient`] drives it from a UDP socket.

mod blob;
mod checksum;
mod client;
mod controller;
mod error;
mod flow;
mod handshake;
mod header;
pub mod iid;
mod isaac;
mod net;
mod optional;
mod packet;
mod pqueue;
pub mod protocol;
mod receiver;
mod resolve;
mod session_key;
mod string;

pub use blob::{
    fragmentize, is_cell, is_ephemeral, is_out_of_world, make_blob_id, ordering_stamp,
    ordering_type, sequence_id, Blob, FRAGMENT_DATA, ORDERING_ORDERED, ORDERING_UNORDERED,
    ORDERING_VISUAL,
};
pub use checksum::checksum32;
pub use client::{real_time, NetClient, NetClientConfig, Transport, UdpTransport};
pub use error::NetError;
pub use flow::Destination;
pub use handshake::{Credential, Logon, LEGIONS_VERSION, RELAUNCH_VERSION};
pub use header::{flags, PacketHeader, HEADER_LEN};
pub use isaac::Isaac;
pub use net::{tokenize, ClientNet, NetEvent};
pub use optional::{Connect, NetErrorId, OptionalHeader, Referral, SockAddrIn};
pub use packet::{split_packet, Fragment, InboundPacket, FRAGMENT_HEADER_LEN, MAX_PACKET_BODY};
pub use receiver::ConnectionState;
pub use resolve::{parse_c_number, parse_port, resolve_server, DEFAULT_PORT};
pub use session_key::{
    ServerPublicKey, SessionCrypto, SessionKey, RETAIL_PUBLIC_KEY_LEN, SESSION_KEY_LEN,
};
pub use string::{pack_string, pack_wide_string, unpack_string, unpack_wide_string};
