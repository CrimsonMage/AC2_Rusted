#![forbid(unsafe_code)]

//! The Asheron's Call 2 client core, headless: the online session (`ac2-weenie`), the player's
//! body (`ac2-physics`) and the movement module joined in retail's frame order
//! (`Turbine::Client::UseTime`), with the landblocks around the player loaded on worker threads
//! (`CellManager`), the player camera (`CameraManager`), and a plain-data [`ClientFrame`] per
//! frame for a renderer.

/// A CPU profiling scope for the frame's steps. With the `profiling` feature it is a puffin scope
/// that lasts to the end of the enclosing block; without it, it compiles to nothing. Scopes stay
/// inside this crate: the `cfg` is evaluated where the macro expands.
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "profiling")]
        puffin::profile_scope!($name);
    };
}

mod audio;
mod chargen;
mod chat_commands;
mod client;
mod client_ambient;
mod client_audio;
mod client_fx_objects;
mod client_latent;
mod client_locations;
mod client_positions;
mod client_prefs;
mod clipboard;
pub use help_content::{content_dir, resolve_content_dir, set_content_dir, CONTENT_DIR_ENV};
mod help_content;
use ac2_smartbox::{camera, cells, scene};
mod client_frame;
mod client_objects;
mod client_radar;
mod client_story;
mod client_story_missiles;
mod client_teleport;
mod sound_ids;
mod sound_space;
pub use ac2_audio::{PortalSoundIds, RainSoundIds};
pub use ac2_smartbox::fx_objects::FxLooks;
pub use ac2_ui::add_map_note;
pub use ac2_ui::OptionsEffect;
pub use client_prefs::{ClientPrefs, DatNames};
pub use client_teleport::{portal_materials, PortalMaterials};
pub use sound_ids::{portal_sound_ids, rain_sound_ids};
pub use target_select::pick_targets;
pub use window_presentation::WindowPresentation;
mod consignment;
mod controls;
mod craft;
mod frame;
mod friends;
mod hud_friends;
pub use friends::{FriendEvent, FriendResult};
mod gameplay;
#[cfg(feature = "window")]
#[cfg(feature = "window")]
mod hud_character;
mod hud_chat_options;
mod hud_craft;
#[cfg(feature = "window")]
mod hud_help;
#[cfg(feature = "window")]
mod hud_ingredient;
#[cfg(feature = "window")]
#[cfg(feature = "window")]
mod hud_navigation;
mod hud_pets;
#[cfg(feature = "window")]
mod hud_player;
#[cfg(feature = "window")]
mod hud_player_message;
#[cfg(feature = "window")]
#[cfg(feature = "window")]
mod hud_request;
mod hud_shop;
#[cfg(feature = "window")]
#[cfg(feature = "window")]
mod hud_view;

mod loader;
mod motion;
#[cfg(feature = "window")]
mod paper_doll_scene;
mod pregame;
#[cfg(feature = "window")]
pub mod pregame_scene;
#[cfg(feature = "window")]
pub mod pregame_view;
mod remote;
mod scripts;
mod scripts_io;
mod scripts_queries;
mod tables;
#[cfg(feature = "window")]
mod target_select;
#[cfg(feature = "window")]
mod ui_examination;
#[cfg(feature = "window")]
mod ui_inventory;
#[cfg(feature = "window")]
mod ui_quests;
pub mod ui_scale;
#[cfg(feature = "window")]
pub mod ui_scene;
mod ui_social;
#[cfg(feature = "window")]
mod ui_sources;
#[cfg(feature = "window")]
pub mod view;
#[cfg(feature = "window")]
mod window_presentation;
mod world_data;

/// `EffectUINode`: one effect as the status bar's strip shows it ([`Client::effect_list`]).
pub use ac2_net::protocol::EffectUiNode as EffectNode;
pub use ac2_script::host::item_queries::ItemDisplay;
pub use ac2_weenie::{capture_image, reassemble as reassemble_capture, CharGenCapture};
pub use audio::{play_ui_sounds, sound_manager};
pub use camera::{action as camera_action, CameraManager};
pub use cells::{block_radius, CellManager, FAR_RADIUS, NEAR_RADIUS};
pub use chargen::ScriptCharGen;
pub use chat_commands::chat_commands;
pub use client::{
    camera_state, CameraAction, Client, FrameInput, Notice, ENTER_WORLD_FLAGS, SYNC_POSITION_FLAGS,
};
pub use client_audio::FxSoundData;
pub use controls::frame_input;
pub use frame::{
    ClientFrame, Holder, MapMarker, ObjectFrame, PlayerFrame, PortalFrame, RadarBlip, RadarFrame,
};
pub use gameplay::{
    Command, ContainerFrame, InventoryFrame, ItemFrame, ProgressFrame, SelectionFrame,
};
pub use hud_chat_options::{chat_option_commands, ui_chat_settings};
pub use hud_shop::{catalog_commands, new_sale_commands, shop_commands};
pub use pregame::{
    pregame_event, pregame_notice, pregame_requests, pregame_requests_capturing, PregameOutcome,
};
pub use scene::CameraFrame;
pub use scripts::SCRIPT_VM_FUNCTIONS;
pub use scripts_queries::{ifun, ShortcutDisplay, ShortcutSettings, SkillLevel, SkillTime};
pub use tables::string_table_ids;
pub use world_data::{BodyData, DatWorld, WorldData, WorldServices, HUMAN_MOTION_INTERP_DESC};
