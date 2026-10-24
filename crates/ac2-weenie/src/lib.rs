#![forbid(unsafe_code)]

//! The online session: logon, character choice, world entry, the event routing and object
//! bookkeeping of `CEventQueue`/`CObjectMaint`, and the player's movement reports
//! (`PlayerMovementModule`).
//!
//! Positions are cell-local (a cell id and AC-axis coordinates relative to its landblock), as the
//! server sends them; callers convert to their own frame.

mod ddd;
mod objects;
mod ordering;
mod reporter;
mod session;
mod world;

pub use ac2_net::protocol::{CellPosition, CharacterIdentity, CharacterSet};
pub use ac2_net::Credential;
pub use ddd::{dat_file_id, dat_iterations};
pub use objects::{
    ObjectTable, RemoteObject, STAMP_FORCE_POSITION, STAMP_MOVE_TO, STAMP_POSITION, STAMP_TELEPORT,
};
pub use reporter::{heading_equals, BodyState, PositionReporter, ReportStamps};
pub use session::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent, WorldEntry};
pub use world::{World, WorldEvent, WorldState};
