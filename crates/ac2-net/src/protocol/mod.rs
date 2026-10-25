//! Encode and decode of Asheron's Call 2 network events.
//!
//! An event is a blob whose first u32 is its opcode (`CEventQueue::PostEvent`), followed by the
//! event's `StreamPack` fields. Strings and nested structures align to 4 bytes measured from
//! the start of the blob. Positions are cell-local: a cell id and AC-axis (z-up) coordinates
//! relative to the landblock the cell names.

mod char_gen;
mod character;
mod client_event;
mod dat_event;
mod error;
mod fx;
mod gameplay;
mod movement;
mod native;
mod object;
mod opcode;
mod pack;
mod physics;
mod physics_event;
mod position;
mod property;
mod qualities;
mod quality_event;
mod reader;
mod server_event;
mod string_info;
mod turbine_chat;
mod types;
mod usage;
mod visual;
mod weenie_desc;

pub use char_gen::{decode_char_gen_result, CharGenResult};
pub use character::{CharacterIdentity, CharacterSet, MinimalCharacterSet};
pub use client_event::ClientEvent;
pub use dat_event::{
    dat_request, end_ddd, interrogation_response, read_int_set, DatEvent, MissingIteration,
};
pub use error::ProtocolError;
pub use fx::{DecalFx, EntityDesc, FxData, FxNode, LightSourceFx, RepulsorFx, StreakFx, Waveform};
pub use gameplay::{
    broadcast as script_broadcast, function as script_function, ScriptMessage, ScriptRequest,
};
pub use movement::{MoveTarget, MovementParameters, PhysicsStory, StoryHook, TargetPosition};
pub use native::{
    ntype, ExaminationDataNode, ExaminationProfile, ExaminationRequest, ExaminationValue,
    NativeObject, SelectionInfo,
};
pub use object::CreateObject;
pub use opcode::Opcode;
pub use pack::{
    entry_flags, pack_values, unpack_values, HeapObject, PackValue, PackedEntry, PackedReference,
};
pub use physics::{BehaviorParams, PhysicsDesc};
pub use physics_event::{ObjectAction, ObjectEvent};
pub use position::{
    heading_from_u8, heading_to_u8, motion_from_u24, motion_to_u24, ClientPosition, ServerPosition,
};
pub use property::{Property, PropertyGroup, PropertyTypes, PropertyValue};
pub use qualities::Qualities;
pub use quality_event::{QualityUpdate, QualityValue};
pub use server_event::ServerEvent;
pub use string_info::{StringInfo, StringVariable};
pub use turbine_chat::{
    blob_type as chat_blob_type, ChatHeader, ChatMessage, ChatPacket, ChatRequest, ChatResponse,
    GunChatExtraInfo, RoomChat, EVENT_SEND_TO_ROOM, METHOD_SEND_TO_ROOM_BY_ID,
    METHOD_SEND_TO_ROOM_BY_NAME,
};
pub use types::{CellPosition, InstanceIdWithStamp, Position};
pub use usage::{UsageDesc, USAGE_DESC_PACKAGE, USAGE_FLAG_SILENT};
pub use visual::{IconLayer, PartGroupDesc, VisualDesc};
pub use weenie_desc::WeenieDesc;

/// Receive queue `CEventQueue` reads world and login events from (`Turbine::Client::InitNet`).
pub const QUEUE_EVENTS: u16 = 1;
/// Receive queue whose events the client discards (`ProcessControlEventQueue`).
pub const QUEUE_SERVER_CONTROL: u16 = 2;
/// Ordered client queue (`Proto_UI::SendBuff` orders queues 3 and 7).
pub const QUEUE_ORDERED: u16 = 3;
/// Unordered client queue to the logon connection (login control, logoff).
pub const QUEUE_CONTROL: u16 = 4;
/// Data-download queue.
pub const QUEUE_DAT: u16 = 5;

/// The client build whose login layouts a decoder follows: the Legions DVD client (7160) or
/// the admin build (7148) the PDB names come from. They differ in `CharacterSet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Build {
    Admin7148,
    #[default]
    Legions7160,
}

impl Build {
    /// The build a logon version string names.
    pub fn from_version(version: &str) -> Self {
        if version == "7148" {
            Self::Admin7148
        } else {
            Self::Legions7160
        }
    }
}
