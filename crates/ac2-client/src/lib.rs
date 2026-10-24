#![forbid(unsafe_code)]

//! The Asheron's Call 2 client core, headless: the online session (`ac2-weenie`), the player's
//! body (`ac2-physics`) and the movement module joined in retail's frame order
//! (`Turbine::Client::UseTime`), with the landblocks around the player loaded on worker threads
//! (`CellManager`), the player camera (`CameraManager`), and a plain-data [`ClientFrame`] per
//! frame for a renderer.

mod camera;
mod cells;
mod client;
mod client_objects;
mod frame;
mod loader;
mod remote;
mod scene;
mod targeting;
mod world_data;

pub use camera::{action as camera_action, CameraManager};
pub use cells::{block_radius, CellManager, FAR_RADIUS, NEAR_RADIUS};
pub use client::{
    CameraAction, Client, FrameInput, Notice, ENTER_WORLD_FLAGS, SYNC_POSITION_FLAGS,
};
pub use frame::{ClientFrame, Holder, ObjectFrame, PlayerFrame};
pub use scene::CameraFrame;
pub use world_data::{BodyData, DatWorld, WorldData, HUMAN_MOTION_INTERP_DESC};
