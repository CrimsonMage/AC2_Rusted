//! Encode and decode of Asheron's Call 2 network events.
//!
//! An event is a blob whose first u32 is its opcode (`DispatchEvent`), followed by the event's
//! `StreamPack` fields. Strings and nested structures align to 4 bytes measured from the start of
//! the event. Positions are cell-local: a cell id and AC-axis (z-up) coordinates relative to the
//! landblock the cell names.

mod character;
mod client_event;
mod error;
mod object;
mod opcode;
mod position;
mod reader;
mod server_event;

pub use character::{CharacterIdentity, CharacterSet};
pub use client_event::ClientEvent;
pub use error::ProtocolError;
pub use object::{CreateObject, PhysicsDesc, VisualDesc};
pub use opcode::Opcode;
pub use position::{
    heading_from_u8, heading_to_u8, motion_from_u24, motion_to_u24, CellPosition, ClientPosition,
    ServerPosition,
};
pub use server_event::ServerEvent;

/// Receive queue the client's `CEventQueue` reads world and login events from.
pub const QUEUE_EVENTS: u16 = 1;
/// Ordered client queue for in-world actions (movement, script calls).
pub const QUEUE_ORDERED: u16 = 3;
/// Unordered client queue for login and session control.
pub const QUEUE_CONTROL: u16 = 4;
