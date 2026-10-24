#![forbid(unsafe_code)]

//! The Asheron's Call 2 client core, headless: the online session (`ac2-weenie`), the player's
//! body (`ac2-physics`) and the movement module joined in retail's frame order
//! (`Turbine::Client::UseTime`), with the landblocks around the player resident and a
//! plain-data [`ClientFrame`] per frame for a renderer.

mod cells;
mod client;
mod frame;
mod world_data;

pub use cells::{block_radius, CellManager};
pub use client::{Client, FrameInput, SYNC_POSITION_FLAGS};
pub use frame::{ClientFrame, ObjectFrame, PlayerFrame};
pub use world_data::{BodyData, DatWorld, WorldData, HUMAN_MOTION_INTERP_DESC};
