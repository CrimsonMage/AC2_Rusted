//! What `CGun::DispatchQueue` hands the client: a `GUNQueueMessage` whose `pvBuffer` is a
//! `Gun::CBaseMessage`, with the PDB's layouts (`ac2client.exe.h`). GunDll's own members of
//! `CBaseMessage` (`m_pXmlPkt`, `m_pszText`, `m_cbTextBuf`, `m_cbTextLen`) are its parser's state
//! and not read by the client; here a message carries the fields its class's getters return.
//! Bodies are added as the client's readers are ported; a class the client never reads has none.

use crate::interfaces::{Context, TagBlob};
use crate::{Hresult, MessageId};

/// `GUNQueueMessage` (0x14 bytes in retail): `flags`, `gmid`, `param`, `cb`, then `pvBuffer`,
/// the message. `TurbineGun::Read` reads only the message; GunDll fills the rest (for a parsed
/// message it queues `flags` 4, `gmid` 1, `param` 0, `cb` 0: `0a_receive_messages.md` §1.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GunQueueMessage {
    pub flags: u32,
    pub gmid: u32,
    pub param: u32,
    pub cb: u32,
    /// `pvBuffer`.
    pub message: Message,
}

/// `Gun::CBaseMessage` and its subclasses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// `m_bInited`: the class's getters return `ZT_E_NOT_INITED` while it is false, and
    /// `TurbineGun::RcvResult` skips the context.
    pub inited: bool,
    /// `m_msgId`.
    pub id: MessageId,
    /// `Gun::CBaseResult`'s members, for a result.
    pub result: Option<ResultHeader>,
    pub body: MessageBody,
}

/// `Gun::CBaseResult`: `m_Context` (+0x1c) and `m_hr` (+0x20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultHeader {
    /// `m_Context`: the context the request was made with.
    pub context: Context,
    /// `m_hr`.
    pub hr: Hresult,
}

/// The class-specific members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageBody {
    /// A class with no members the client reads.
    None,
    /// `Gun::CGetFriendsListResult` (0x4200): `m_Count` statuses at `m_AllFriendsStatus`.
    GetFriendsList(Vec<UserStatus>),
    /// `Gun::CGetIgnoredListResult` (0x4203): `m_Count` names at `m_Ignored`.
    GetIgnoredList(Vec<String>),
    /// `Gun::CFriendStartEvent` (0x430c): `m_Count` statuses at `m_AllFriendsStatus`.
    FriendStart(Vec<UserStatus>),
    /// `Gun::CFriendStatusEvent` (0x430d): `m_UserName`, `m_Mode`.
    FriendStatus { user_name: String, mode: u32 },
    /// `Gun::CFlagsResult` (0x4208): `m_WhichFlag` (+0x24), `m_Flags` (+0x28).
    Flags { which_flag: u32, flags: u32 },
    /// `Gun::CWhereIsResult` (0x4209): `m_UserName` (+0x24), `m_Mode` (+0x28), `m_Location`
    /// (+0x2c).
    WhereIs {
        user_name: String,
        mode: u32,
        location: String,
    },
    /// `Gun::CJoinRoomResult` (0x5201): `m_RoomId` (+0x24), `m_ServerId` (+0x28).
    JoinRoom { room_id: u32, server_id: u32 },
    /// `Gun::CCreateRoomResult` (0x520a): `m_RoomId` (+0x24), `m_ServerId` (+0x28).
    CreateRoom { room_id: u32, server_id: u32 },
    /// `Gun::CGetListOfRoomsResult` (0x5200): `m_Index` (+0x24), `m_Total` (+0x28), `m_Count`
    /// (+0x2c) rooms at `m_RoomList` (+0x30).
    GetListOfRooms {
        index: u32,
        total: u32,
        rooms: Vec<ChatRoomAttrs>,
    },
    /// `Gun::CCallCreateResult` (0x10200): `m_QueueID` (+0x24), `m_QueuePos` (+0x28).
    CallCreate { queue_id: u32, queue_pos: u32 },
    /// `Gun::CGetTicketResult` (0xf203): `m_Ticket` (+0x24), a `tagBLOB*`.
    GetTicket(Option<TagBlob>),
    /// `Gun::CEnterPersonaResult` (0x8200): `m_Type` (+0x24), `m_ServerId` (+0x28).
    EnterPersona { kind: u32, server_id: u32 },
    /// `Gun::CGetPersonaListResult` (0x8204): `m_Count` (+0x24) names at `m_Personas` (+0x28).
    GetPersonaList(Vec<String>),
    /// `Gun::CRoomEnterEvent` (0x5314): `m_RoomId` (+0x1c), `m_UserName` (+0x20).
    RoomEnter { room_id: u32, user_name: String },
    /// `Gun::CRoomLeaveEvent` (0x5315): the same members.
    RoomLeave { room_id: u32, user_name: String },
    /// `Gun::CRoomSpeakEvent` (0x5316): `m_RoomId` (+0x1c), `m_UserName` (+0x20), `m_Type`
    /// (+0x24), `m_Text` (+0x28), `m_ExtraInfo` (+0x2c).
    RoomSpeak {
        room_id: u32,
        user_name: String,
        kind: u32,
        text: String,
        extra: Option<TagBlob>,
    },
    /// `Gun::CUserSpeakEvent` (0x5317): `m_UserName` (+0x1c), `m_Type` (+0x20), `m_Text`
    /// (+0x24), `m_ExtraInfo` (+0x28).
    UserSpeak {
        user_name: String,
        kind: u32,
        text: String,
        extra: Option<TagBlob>,
    },
    /// `Gun::CKickedOutEvent` (0x5318): `m_RoomId` (+0x1c), `m_Reason` (+0x20).
    KickedOut { room_id: u32, reason: String },
    /// `Gun::CReceiveOwnershipEvent` (0x5319): `m_RoomId` (+0x1c).
    ReceiveOwnership { room_id: u32 },
    /// `Gun::CInvitedEvent` (0x531a): `m_UserName` (+0x1c), `m_RoomName` (+0x20),
    /// `m_Invitation` (+0x24).
    Invited {
        user_name: String,
        room_name: String,
        invitation: String,
    },
}

/// `Gun::ChatRoomAttrs` (20 bytes in retail): `pwszName`, `pwszTopic`, `mode`, `cUserCount`,
/// `cUserLimit`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatRoomAttrs {
    pub name: String,
    pub topic: String,
    pub mode: u32,
    pub user_count: u32,
    pub user_limit: u32,
}

/// `Gun::UserStatus` (12 bytes in retail): `pwszUserName`, `mode`, `pwszLocation`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserStatus {
    pub user_name: String,
    pub mode: u32,
    pub location: String,
}
