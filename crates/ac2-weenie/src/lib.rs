#![forbid(unsafe_code)]

//! The online session: logon, character choice, world entry, the event routing and object
//! bookkeeping of `CEventQueue`/`CObjectMaint`, and the player's movement reports
//! (`PlayerMovementModule`).
//!
//! Positions are cell-local (a cell id and AC-axis coordinates relative to its landblock), as the
//! server sends them; callers convert to their own frame.

mod appearance;
mod chat;
mod ddd;
mod destruction;
mod movement_module;
mod objects;
mod ordering;
mod player_script;
mod qualities;
mod reporter;
mod session;
mod vitals;
mod world;

pub use ac2_net::protocol::{CellPosition, CharacterIdentity, CharacterSet};
pub use ac2_net::Credential;
pub use appearance::{Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};
pub use chat::{to_iid, DISPLAY_SPEECH, DISPLAY_TELL};
pub use ddd::{dat_file_id, dat_iterations};
pub use movement_module::{
    ActiveAction, BodyFacts, CameraState, MovementCommands, MovementModule, ACTION_ABOUT_FACE,
    ACTION_CLASS_MOVEMENT, ACTION_FORWARD, ACTION_JUMP, ACTION_STRAFE_NEGATIVE,
    ACTION_STRAFE_POSITIVE, ACTION_TURN_MINUS_90, ACTION_TURN_NEGATIVE, ACTION_TURN_PLUS_90,
    ACTION_TURN_POSITIVE,
};
pub use objects::{
    ObjectTable, RemoteObject, STAMP_FORCE_POSITION, STAMP_MOVE_TO, STAMP_POSITION, STAMP_TELEPORT,
};
pub use player_script::{GameplayNotice, PlayerScript, ScriptOutput, NO_SELECTION};
pub use qualities::WeenieQualities;
pub use reporter::{heading_equals, BodyState, PositionReporter, ReportStamps};
pub use session::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent, WorldEntry};
pub use vitals::{vital, PlayerVitals, Vital};
pub use world::{World, WorldEvent, WorldState};
