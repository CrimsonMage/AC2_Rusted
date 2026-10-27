#![forbid(unsafe_code)]

//! The Asheron's Call 2 client core, headless: the online session (`ac2-weenie`), the player's
//! body (`ac2-physics`) and the movement module joined in retail's frame order
//! (`Turbine::Client::UseTime`), with the landblocks around the player loaded on worker threads
//! (`CellManager`), the player camera (`CameraManager`), and a plain-data [`ClientFrame`] per
//! frame for a renderer.

mod appearance;
mod audio;
mod chargen;
mod chargen_data;
mod chat_commands;
pub mod chat_text;
mod client;
mod client_ambient;
mod client_audio;
mod client_locations;
pub use client_locations::{saved_windows, ui_save_locations};
mod client_options;
mod client_prefs;
use ac2_smartbox::{camera, cells, scene};
mod client_frame;
mod client_objects;
mod client_radar;
mod client_story;
mod client_teleport;
pub use client_options::{ApplyOutcome, Close, OptionsWindow, PageOutcome};
pub use client_prefs::{ClientPrefs, DatNames};
pub use client_radar::add_map_note;
mod consignment;
mod controls;
mod craft;
mod frame;
mod friends;
mod hud_friends;
pub use friends::{FriendEvent, FriendResult};
mod gameplay;
#[cfg(feature = "window")]
mod hud_book;
#[cfg(feature = "window")]
mod hud_character;
mod hud_chat_options;
mod hud_craft;
#[cfg(feature = "window")]
mod hud_death;
mod hud_game_dialogs;
#[cfg(feature = "window")]
mod hud_ingredient;
#[cfg(feature = "window")]
mod hud_items_3d;
#[cfg(feature = "window")]
mod hud_navigation;
mod hud_pets;
#[cfg(feature = "window")]
mod hud_player;
mod hud_shop;
#[cfg(feature = "window")]
mod hud_status;
mod hud_story;
#[cfg(feature = "window")]
mod hud_view;
mod inventory_queries;
mod item_queries;
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
mod skills;
mod social_queries;
mod tables;
#[cfg(feature = "window")]
mod target_select;
#[cfg(feature = "window")]
mod ui_images;
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
mod view_portal;
mod world_data;

/// `EffectUINode`: one effect as the status bar's strip shows it ([`Client::effect_list`]).
pub use ac2_net::protocol::EffectUiNode as EffectNode;
pub use appearance::local_visual_desc;
pub use audio::{play_ui_sounds, sound_manager};
pub use camera::{action as camera_action, CameraManager};
pub use cells::{block_radius, CellManager, FAR_RADIUS, NEAR_RADIUS};
pub use chargen::ScriptCharGen;
pub use chargen_data::{capture_image, reassemble as reassemble_capture, CharGenCapture};
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
pub use hud_shop::{
    catalog_commands, catalog_notice, consignment_notice, consignment_requests, new_sale_commands,
    shop_commands, shop_notice, shop_stack_calls,
};
pub use item_queries::ItemDisplay;
pub use motion::PartPose;
pub use pregame::{
    pregame_event, pregame_notice, pregame_requests, pregame_requests_capturing, PregameOutcome,
};
pub use scene::CameraFrame;
pub use scripts::SCRIPT_VM_FUNCTIONS;
pub use scripts_queries::{ifun, ShortcutDisplay, ShortcutSettings, SkillLevel, SkillTime};
pub use tables::string_table_ids;
pub use world_data::{BodyData, DatWorld, WorldData, HUMAN_MOTION_INTERP_DESC};
