#![forbid(unsafe_code)]

//! The online session: logon, character choice, world entry, the table of objects the server
//! reports, and the player's movement reports.
//!
//! Positions are cell-local (a cell id and AC-axis coordinates relative to its landblock), as the
//! server sends them; callers convert to their own frame.

mod objects;
mod reporter;
mod session;

pub use ac2_net::protocol::{CellPosition, CharacterIdentity, CharacterSet};
pub use ac2_net::Credential;
pub use objects::{ObjectTable, RemoteObject};
pub use reporter::PositionReporter;
pub use session::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent, WorldEntry};
