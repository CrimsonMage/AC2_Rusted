#![forbid(unsafe_code)]

//! Asheron's Call 2 `SmartBox` (Legions `portal/client/src/smartbox`): the cells resident around
//! the player (`CellManager`, over [`BlockPool`]'s loader threads), the player camera
//! (`CameraManager`, `TargetingModule`) and the scene camera's render position
//! (`SceneCamera`). It knows nothing of the session or the network: the online client and the
//! offline world viewer both drive it with plain data.

mod blocks;
pub mod camera;
mod camera_mouse;
pub mod cells;
mod player;
pub mod scene;
pub mod targeting;

pub use blocks::{
    load_dat_block, prefetch_dat_far_block, BlockLoaded, BlockLoader, BlockPool, BlockSource,
};
pub use camera::{action as camera_action, CameraManager, CameraSubject};
pub use cells::{block_radius, BlockChange, CellManager, FAR_RADIUS, NEAR_RADIUS};
pub use player::{load_body_data, BodyData, PlayerBody, HUMAN_MOTION_INTERP_DESC};
pub use scene::{frame_is_valid, CameraFrame, SceneCamera};
