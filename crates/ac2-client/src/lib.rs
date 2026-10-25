#![forbid(unsafe_code)]

//! The Asheron's Call 2 client core, headless: the online session (`ac2-weenie`), the player's
//! body (`ac2-physics`) and the movement module joined in retail's frame order
//! (`Turbine::Client::UseTime`), with the landblocks around the player loaded on worker threads
//! (`CellManager`), the player camera (`CameraManager`), and a plain-data [`ClientFrame`] per
//! frame for a renderer.

mod appearance;
mod camera;
mod camera_mouse;
mod cells;
mod chat_commands;
pub mod chat_text;
mod client;
mod client_objects;
mod controls;
mod frame;
mod gameplay;
#[cfg(feature = "window")]
mod hud_player;
#[cfg(feature = "window")]
mod hud_view;
mod loader;
mod motion;
mod pregame;
#[cfg(feature = "window")]
pub mod pregame_view;
mod remote;
mod scene;
mod scripts;
mod scripts_io;
mod scripts_queries;
mod tables;
mod targeting;
#[cfg(feature = "window")]
mod ui_images;
#[cfg(feature = "window")]
mod ui_sources;
#[cfg(feature = "window")]
pub mod view;
mod world_data;

pub use appearance::local_visual_desc;
pub use camera::{action as camera_action, CameraManager};
pub use cells::{block_radius, CellManager, FAR_RADIUS, NEAR_RADIUS};
pub use chat_commands::chat_commands;
pub use client::{
    CameraAction, Client, FrameInput, Notice, ENTER_WORLD_FLAGS, SYNC_POSITION_FLAGS,
};
pub use controls::frame_input;
pub use frame::{ClientFrame, Holder, ObjectFrame, PlayerFrame};
pub use gameplay::{Command, ContainerFrame, InventoryFrame, ProgressFrame, SelectionFrame};
pub use motion::PartPose;
pub use pregame::{pregame_event, pregame_notice, pregame_requests, PregameOutcome};
pub use scene::CameraFrame;
pub use scripts::SCRIPT_VM_FUNCTIONS;
pub use scripts_queries::{ifun, ShortcutDisplay, ShortcutSettings, SkillLevel, SkillTime};
pub use tables::string_table_ids;
pub use world_data::{BodyData, DatWorld, WorldData, HUMAN_MOTION_INTERP_DESC};
