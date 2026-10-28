#![forbid(unsafe_code)]

//! The online session: logon, character choice, world entry, the event routing and object
//! bookkeeping of `CEventQueue`/`CObjectMaint`, and the player's movement reports
//! (`PlayerMovementModule`).
//!
//! Positions are cell-local (a cell id and AC-axis coordinates relative to its landblock), as the
//! server sends them; callers convert to their own frame.

mod appearance;
mod chat;
mod craft;
mod ddd;
mod destruction;
mod friends;
mod inventory;
mod items;
mod movement_module;
mod notice;
mod objects;
mod ordering;
mod player_script;
mod progress;
mod qualities;
mod reporter;
mod session;
mod shell;
mod social;
mod story;
mod tables;
mod ui_data;
mod usage;
mod vitals;
mod world;

pub use ac2_net::protocol::{CellPosition, CharacterIdentity, CharacterSet};
pub use ac2_net::Credential;
pub use appearance::{Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};
pub use chat::{to_iid, DISPLAY_SPEECH, DISPLAY_TELL};
pub use craft::{Ingredient, Recipe};
pub use ddd::{dat_file_id, dat_iterations};
pub use friends::{FriendInfo, MODE_AWAY, MODE_DO_NOT_DISTURB};
pub use inventory::{Contents, InventoryState, INVENTORY_REFRESH};
pub use items::{item_quality, ItemStats, EQUIP_LOCATIONS, MISSING_NAME};
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
pub use progress::{
    skill_error, Experience, Progress, SelectionState, Shortcuts, Skill, SkillRequest, Skills,
};
pub use qualities::{DescKind, WeenieQualities};
pub use reporter::{heading_equals, BodyState, PositionReporter, ReportStamps};
pub use session::{
    CharacterChoice, OnlineConfig, OnlineSession, SessionEvent, WorldEntry, IFUN_HANDLE_COLLISION,
    IFUN_ON_STAT_CHANGED,
};
pub use shell::{
    handler as chat_handler, ChatCommands, ChatHistory, ChatState, ShellAction, Speaker,
    COMMAND_TABLE_ENUM, DEFAULT_COMMAND, EMOTE_TABLE_PACKAGE, HELP_STRINGS, REGISTERED_COMMANDS,
    SQUELCH_WORDS,
};
pub use social::{
    AllegianceMember, AllegianceProfile, FellowInfo, FellowshipInfo, Social, SocialContext,
    SocialRequest,
};
pub use story::{
    Keyframe, SceneInfo, TRACK_AUDIO, TRACK_MOVIE, TRACK_SUBTITLE, TRACK_VIDEO, TRACK_VOICE,
    UNDEFINED_DURATION,
};
pub use tables::{
    StringTableIds, TABLE_ENUM_CHANNELS, TABLE_ENUM_CHAT_WINDOW, TABLE_ENUM_COMMUNICATION,
    TABLE_ENUM_EMOTE, TABLE_ENUM_FELLOWSHIP, TABLE_ENUM_TELL, TABLE_ENUM_USAGE,
};
pub use ui_data::{CraftSkill, DamageText, Shop, ShopSale};
pub use usage::DISPLAY_USAGE;
pub use vitals::{vital, PlayerVitals, VitaeStatus, Vital};
pub use world::{World, WorldEvent, WorldState};
