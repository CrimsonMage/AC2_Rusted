//! GunDll.dll's `CGun` (Microsoft's MSN Gaming Zone GUN client library) ported from the decompile
//! (`decompiled-pdb/gundll/GunDll/GunDll.dll.c`) and the verified specs in
//! `decompiled-pdb/gundll/spec/` (`0a_send_session.md`, `0a_receive_messages.md` and their `verify_*`
//! files, whose "Verification and adjudication" sections supersede the bodies). [`CGun`] implements
//! [`crate::GunObject`], the ten `IGun*` interfaces, over the DirectPlay 8 traits in [`dp8`].
//!
//! Modules: [`cgun`] (the object and `IGunConnection`), [`session`] (the connect handshake),
//! [`requests`], [`tickets`], [`misc`] (the other interfaces), [`transport`] (the DP8 connect, handler,
//! framing and sends), [`dp8`] (the DirectPlay 8 traits), [`crypto`] (CryptoAPI), [`blob`] (the
//! 6-bit and Base64 codecs), [`xml_parse`] (the parser and `IXmlPacket`), [`msg`] (message
//! render/parse), [`catalog`] (the message table), [`client_msg`] (to the client's message types),
//! [`queue`], [`registry`].

pub mod blob;
pub mod catalog;
mod catalog_chat;
mod catalog_conn_ticket;
mod catalog_friends;
mod catalog_names;
mod catalog_other;
pub mod cgun;
pub mod client_msg;
pub mod crypto;
pub mod dp8;
pub mod ids;
mod misc;
pub mod msg;
mod msg_extra;
pub mod queue;
pub mod registry;
mod requests;
mod session;
mod tickets;
pub mod transport;
pub mod xml_parse;

pub use cgun::{CGun, GunEnv, SharedThreadPool};
pub use registry::GunRegistry;

#[cfg(test)]
mod tests;
