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

mod appearance;
mod audio;
mod chargen;
mod chargen_data;
mod chat_commands;
pub mod chat_text;
mod client;
mod client_ambient;
mod client_audio;
mod client_latent;
mod client_locations;
mod client_positions;
pub use client_locations::{saved_windows, ui_save_locations};
mod client_options;
mod client_prefs;
mod clipboard;
use ac2_smartbox::{camera, cells, scene};
mod client_frame;
mod client_objects;
mod client_radar;
mod client_story;
mod client_teleport;
mod fx_particles;
mod fx_scripts;
mod portal_sounds;
mod rain_sounds;
pub use client_options::{ApplyOutcome, Close, OptionsWindow, PageOutcome};
pub use client_prefs::{ClientPrefs, DatNames};
pub use client_radar::add_map_note;
pub use fx_particles::{ObjectLightStart, ObjectParticleStart};
pub use fx_scripts::{load_fx_scripts, FxScripts};
pub use hud_options::OptionsEffect;
pub use portal_sounds::{portal_sound_ids, PortalSoundIds};
pub use pref_labels::pref_label;
pub use rain_sounds::{rain_sound_ids, RainSoundIds};
pub use target_select::pick_targets;
mod consignment;
mod controls;
mod craft;
mod frame;
mod friends;
mod hud_assistance;
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
mod hud_options;
mod hud_pets;
#[cfg(feature = "window")]
mod hud_player;
#[cfg(feature = "window")]
mod hud_player_message;
#[cfg(feature = "window")]
mod hud_player_notice;
#[cfg(feature = "window")]
mod hud_player_tick;
mod hud_shop;
mod hud_shortcut_checks;
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
mod pref_labels;
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
mod ui_examination;
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
