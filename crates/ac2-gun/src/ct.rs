//! The shared result and event base classes (`ac2/shared/src/gun`): `CTBullet`, `CTResult`,
//! `CTEvent` and their `CT*Result` / `CT*Event` subclasses, which the client's `CCli*` handlers
//! (`ac2/client/src/gun`) derive from. Retail's objects are `CTBullet`-derived classes with a
//! virtual `ReferenceCountTemplate<1048576,0>` base; here a handler is an `Arc<dyn Trait>` (the
//! reference count is the `Arc`'s) and a class's virtual slots from `+8` on are the trait's methods
//! in slot order. `+0 NumEventsHandled` and `+4 IncEventsHandled` are [`Bullet`]'s.
//!
//! Method names that the PDB gives (`HandleFriendAddResult`, `HearRoom`, ...) are used as they
//! are; a slot the PDB does not name (it carries only the prototype) is named for the message it
//! answers and says so.

use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use crate::interfaces::TagBlob;
use crate::message::UserStatus;
use crate::zt::{ZT_E_ACCOUNT_BANNED, ZT_E_ALREADY_TICKETED, ZT_E_MAC_BANNED, ZT_E_NOT_TICKETED};
use crate::Hresult;

/// `CTBullet`'s state: `m_eventsHandled` (+8).
#[derive(Debug, Default)]
pub struct CtBullet {
    events_handled: AtomicU32,
}

impl CtBullet {
    pub fn new() -> Self {
        Self::default()
    }

    /// `CTBullet::NumEventsHandled` (admin 0x00402190, Legions 0x00402130): `m_eventsHandled`.
    pub fn num_events_handled(&self) -> u32 {
        self.events_handled.load(Ordering::Relaxed)
    }

    /// `CTBullet::IncEventsHandled` (admin 0x004021A0, Legions 0x00402140): `m_eventsHandled + 1`.
    pub fn inc_events_handled(&self) {
        self.events_handled.fetch_add(1, Ordering::Relaxed);
    }
}

/// `CTBullet`: vtable `+0 NumEventsHandled`, `+4 IncEventsHandled`. (No `Send`/`Sync` bound: the
/// client is single threaded and a handler holds the same `Rc<TurbineGun>` it is called from.) `StartupGUN` polls a
/// listener's `NumEventsHandled` to see that an event arrived.
pub trait Bullet {
    /// The object's `CTBullet` state.
    fn bullet(&self) -> &CtBullet;

    /// `CTBullet::NumEventsHandled`.
    fn num_events_handled(&self) -> u32 {
        self.bullet().num_events_handled()
    }

    /// `CTBullet::IncEventsHandled`.
    fn inc_events_handled(&self) {
        self.bullet().inc_events_handled()
    }
}

/// `CTResult`'s state: the `CTBullet` part and `m_hResult` (+0xC), which
/// `TurbineGun::RcvResult` stores from the message's `m_hr`.
#[derive(Debug, Default)]
pub struct CtResultState {
    bullet: CtBullet,
    h_result: AtomicU32,
}

impl CtResultState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bullet(&self) -> &CtBullet {
        &self.bullet
    }

    /// `CTResult::m_hResult`.
    pub fn h_result(&self) -> Hresult {
        Hresult(self.h_result.load(Ordering::Relaxed))
    }

    /// Stores `m_hResult` (`RcvResult`: `ctx->m_hResult = res->m_hr`).
    pub fn set_h_result(&self, hr: Hresult) {
        self.h_result.store(hr.0, Ordering::Relaxed);
    }
}

/// `CTResult`: a request's context, answered once by a result message.
pub trait CtResult: Bullet {
    fn result_state(&self) -> &CtResultState;

    /// `CTResult::m_hResult`.
    fn h_result(&self) -> Hresult {
        self.result_state().h_result()
    }

    fn set_h_result(&self, hr: Hresult) {
        self.result_state().set_h_result(hr)
    }

    /// `CTResult::GetHResultString` (admin 0x006075B0, Legions 0x005C8450):
    /// `TurbineGunUtils::StringFromHR(m_hResult)`.
    fn get_h_result_string(&self) -> String {
        crate::turbine_gun_utils::string_from_hr(self.h_result())
    }
}

/// `CTEvent`: a listener (`CTBullet` with no members of its own).
pub trait CtEvent: Bullet {}

/// `CTConnectEvent::EConnectEvent` (PDB values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum EConnectEvent {
    Undef = 0,
    Connect = 1,
    Reconnected = 2,
    Disconnect = 3,
    ConnectFailed = 4,
}

/// `CTConnectEvent`: vtable `+8 HandleConnectEvent`.
pub trait CtConnectEvent: CtEvent {
    /// Slot `+8`: `void HandleConnectEvent(EConnectEvent)`.
    fn handle_connect_event(&self, evt: EConnectEvent);
}

/// `CTChatEvent`: the chat listener. The PDB gives the prototypes
/// (`(room, user)` twice, `(room, user, speech, extra)`, `(user, speech, extra)`,
/// `(roomName, invitation)`, `(room)` and `(room, reason)`) and only `HearRoom` / `HearUser` by
/// name (`CCliChatEvent`); the other slots are named for the event `TurbineGun::RcvChatEvent`
/// calls them for. `CCliChatEvent` overrides the others with `ret N` stubs, so they are required
/// here, not defaulted.
pub trait CtChatEvent: CtEvent {
    /// Slot `+8` (`RcvChatEvent` for `CInvitedEvent`): `(roomName, invitation)`.
    fn invited(&self, room_name: &str, invitation: &str);
    /// Slot `+0xC` (`CRoomEnterEvent`): `(roomId, userName)`.
    fn room_enter(&self, room_id: u32, user_name: &str);
    /// Slot `+0x10` (`CRoomLeaveEvent`): `(roomId, userName)`.
    fn room_leave(&self, room_id: u32, user_name: &str);
    /// Slot `+0x14`, `CCliChatEvent::HearRoom` (admin 0x007281F0, Legions 0x006A64D0).
    fn hear_room(&self, room_id: u32, user: &str, speech: &str, extra: Option<&TagBlob>);
    /// Slot `+0x18` (`CKickedOutEvent`): `(roomId, reason)`.
    fn kicked_out(&self, room_id: u32, reason: &str);
    /// Slot `+0x1C` (`CReceiveOwnershipEvent`): `(roomId)`.
    fn receive_ownership(&self, room_id: u32);
    /// Slot `+0x20`, `CCliChatEvent::HearUser` (admin 0x00728220, Legions 0x006A6500).
    fn hear_user(&self, user: &str, speech: &str, extra: Option<&TagBlob>);
}

/// `CTFriendsEvent`: the presence listener (PDB prototypes
/// `(ulong, Gun::UserStatus*)` and `(const PStringBase<ushort>&, ulong)`;
/// `CCliFriendsEvent::FriendStart` / `FriendStatus`).
pub trait CtFriendsEvent: CtEvent {
    /// Slot `+8`, `CCliFriendsEvent::FriendStart` (admin 0x00728500, Legions 0x006A6810):
    /// `(count, statusArray)`; `count` is `statuses.len()`.
    fn friend_start(&self, statuses: &[UserStatus]);
    /// Slot `+0xC`, `CCliFriendsEvent::FriendStatus` (admin 0x007283C0, Legions 0x006A66D0).
    fn friend_status(&self, friend_name: &str, mode: u32);
}

/// `CTFriendAddResult`: slot `+8` `CCliFriendAddResult::HandleFriendAddResult`.
pub trait CtFriendAddResult: CtResult {
    fn handle_friend_add_result(&self);
}

/// `CTFriendDeleteResult`: slot `+8` `HandleFriendDeleteResult`.
pub trait CtFriendDeleteResult: CtResult {
    fn handle_friend_delete_result(&self);
}

/// `CTIgnoredAddResult`: slot `+8` `HandleIgnoredAddResult`.
pub trait CtIgnoredAddResult: CtResult {
    fn handle_ignored_add_result(&self);
}

/// `CTIgnoredDeleteResult`: slot `+8` `HandleIgnoredDeleteResult`.
pub trait CtIgnoredDeleteResult: CtResult {
    fn handle_ignored_delete_result(&self);
}

/// `CTGetFriendsListResult`: slot `+8` `HandleGetFriendsListResult(ulong count, UserStatus*)`.
pub trait CtGetFriendsListResult: CtResult {
    fn handle_get_friends_list_result(&self, statuses: &[UserStatus]);
}

/// `CTGetIgnoredListResult`: slot `+8` `HandleGetIgnoredListResult(ulong count, wchar_t**)`.
pub trait CtGetIgnoredListResult: CtResult {
    fn handle_get_ignored_list_result(&self, names: &[String]);
}

/// `CTFlagsResult`: slot `+8` takes `(ulong, ulong)` (the `m_WhichFlag` and `m_Flags` it is
/// called with); the PDB names no method, `CCliFlagsResult` leaves the slot a stub.
pub trait CtFlagsResult: CtResult {
    fn handle_flags_result(&self, which_flag: u32, flags: u32);
}

/// `CTLocationResult`: slot `+8`, called for `SetLocationRes`; unnamed in the PDB,
/// `CCliLocationResult` leaves it a stub.
pub trait CtLocationResult: CtResult {
    fn handle_location_result(&self);
}

/// `CTSendToRoomResult`: slot `+8` `CCliSendToRoomResult::HandleSendToRoomResult`.
pub trait CtSendToRoomResult: CtResult {
    fn handle_send_to_room_result(&self);
}

/// `CTSendToUserResult`: slot `+8` `CCliSendToUserResult::HandleSendToUserResult`.
pub trait CtSendToUserResult: CtResult {
    fn handle_send_to_user_result(&self);
}

/// `CTCallCreateResult`: slot `+8` `HandleCallCreateResult(ulong queueID, ulong queuePos)`.
pub trait CtCallCreateResult: CtResult {
    fn handle_call_create_result(&self, queue_id: u32, queue_pos: u32);
}

/// `CTJoinRoomResult`: slot `+8` `(ulong roomId, ulong serverId)`; unnamed in the PDB.
pub trait CtJoinRoomResult: CtResult {
    fn handle_join_room_result(&self, room_id: u32, server_id: u32);
}

/// `CTCreateRoomResult`: slot `+8` `(ulong roomId, ulong serverId)`; unnamed in the PDB.
pub trait CtCreateRoomResult: CtResult {
    fn handle_create_room_result(&self, room_id: u32, server_id: u32);
}

/// `CTLeaveRoomResult`: slot `+8`, no arguments; unnamed in the PDB.
pub trait CtLeaveRoomResult: CtResult {
    fn handle_leave_room_result(&self);
}

/// `CTLeaveAllRoomsResult`: slot `+8`, no arguments; unnamed in the PDB.
pub trait CtLeaveAllRoomsResult: CtResult {
    fn handle_leave_all_rooms_result(&self);
}

/// `CTCreatePersistentChatChannelResult` (a `CTCreateRoomResult`): its own slot `+0xC`,
/// `(ulong)`; `RcvChatResult` calls it with 0. Unnamed in the PDB.
pub trait CtCreatePersistentChatChannelResult: CtCreateRoomResult {
    fn handle_create_persistent_chat_channel_result(&self, value: u32);
}

/// `CTDeletePersistentChatChannelResult`: slot `+8`, no arguments; unnamed in the PDB.
pub trait CtDeletePersistentChatChannelResult: CtResult {
    fn handle_delete_persistent_chat_channel_result(&self);
}

/// `CTPersonaResult`: slots `+8 (IGunPersona::PersonaType, ulong)` enter, `+0xC` leave,
/// `+0x10` create, `+0x14` delete, `+0x18 (PList<PStringBase<ushort>>)` list; unnamed in the PDB
/// (`RcvPersonaResult` calls them for the five persona results).
pub trait CtPersonaResult: CtResult {
    fn handle_enter_persona_result(&self, kind: u32, server_id: u32);
    fn handle_leave_persona_result(&self);
    fn handle_create_persona_result(&self);
    fn handle_delete_persona_result(&self);
    fn handle_get_persona_list_result(&self, names: &[String]);
}

/// `CTTicketResult::ETicketResult` (PDB values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ETicketResult {
    Success = 0,
    NoTicket = 0xC112_080B,
    OldTicketStillExists = 0xC112_080C,
    Banned = 0xC112_1005,
    Failure = 0xC112_1006,
}

impl ETicketResult {
    /// The `HRESULT` value the enum holds.
    pub fn hresult(self) -> Hresult {
        Hresult(self as u32)
    }
}

// The enum's values are the ZT_* codes (checked against the zt table).
const _: () = {
    assert!(ETicketResult::NoTicket as u32 == ZT_E_NOT_TICKETED.0);
    assert!(ETicketResult::OldTicketStillExists as u32 == ZT_E_ALREADY_TICKETED.0);
    assert!(ETicketResult::Banned as u32 == ZT_E_ACCOUNT_BANNED.0);
    assert!(ETicketResult::Failure as u32 == ZT_E_MAC_BANNED.0);
};

/// `CTTicketResult`'s state: `m_pTicket` (+0x10), an owned `tagBLOB*`; dropping it is
/// `CTTicketResult::~CTTicketResult` (admin 0x00607470, Legions 0x005C8310), which frees the
/// ticket and its data.
#[derive(Debug, Default)]
pub struct CtTicketState {
    ticket: Mutex<Option<TagBlob>>,
}

impl CtTicketState {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<TagBlob>> {
        self.ticket.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// `CTTicketResult::AdoptTicket` (admin 0x006074B0, Legions 0x005C8350): frees the old
    /// ticket (if any), stores the given one, returns 1.
    pub fn adopt_ticket(&self, ticket: TagBlob) -> bool {
        *self.lock() = Some(ticket);
        true
    }

    /// `CTTicketResult::CopyTicket` (admin 0x00607500, Legions 0x005C83A0): frees the old
    /// ticket; `m_pTicket = new tagBLOB` with `cbSize = src->cbSize` and a new `cbSize`-byte
    /// buffer holding the source's bytes; returns 1 (0 when the buffer allocation failed, which
    /// Rust does not report: an allocation failure aborts).
    pub fn copy_ticket(&self, src: &TagBlob) -> bool {
        let mut data = vec![0u8; src.cb_size as usize];
        let bytes = src.bytes();
        data[..bytes.len()].copy_from_slice(bytes);
        *self.lock() = Some(TagBlob {
            cb_size: src.cb_size,
            data: Some(data),
        });
        true
    }

    /// The folded `DiskConBase::GetDataset` the client calls as `ctx->GetTicketBlob()`: returns
    /// `m_pTicket` (here a copy), `None` when it is NULL.
    pub fn get_ticket_blob(&self) -> Option<TagBlob> {
        self.lock().clone()
    }
}

/// `CTTicketResult` (admin 0x00607400 ctor: `m_pTicket = NULL`): the ticket context. Slots
/// `+8 HandleTicketArrived(ETicketResult)`, `+0xC HandleAuthenticationResult(ETicketResult)`
/// (`CCliTicketResult`, admin 0x007282A0 / 0x007282C0, Legions 0x006A6580 / 0x006A65A0) and
/// `+0x10` (the release-ticket result: a `ret 4` stub in `CCliTicketResult`; unnamed in the PDB).
pub trait CtTicketResult: CtResult {
    fn ticket_state(&self) -> &CtTicketState;

    fn handle_ticket_arrived(&self, result: ETicketResult);
    fn handle_authentication_result(&self, result: ETicketResult);
    /// Slot `+0x10`, called for `ReleaseTicketRes` with 0, `NoTicket` or `Failure`.
    fn handle_release_ticket_result(&self, result: ETicketResult);

    /// `CTTicketResult::AdoptTicket`.
    fn adopt_ticket(&self, ticket: TagBlob) -> bool {
        self.ticket_state().adopt_ticket(ticket)
    }

    /// `CTTicketResult::CopyTicket`.
    fn copy_ticket(&self, src: &TagBlob) -> bool {
        self.ticket_state().copy_ticket(src)
    }

    /// `ctx->GetTicketBlob()`.
    fn get_ticket_blob(&self) -> Option<TagBlob> {
        self.ticket_state().get_ticket_blob()
    }
}

/// `CTBullet::ConvertToWeenieResult` (admin 0x00735C60, Legions 0x006B1900): the weenie result
/// number the scripts see for a GUN `HRESULT`. Any other value goes through `StringFromHR`
/// (result discarded) and gives 0xC741.
pub fn convert_to_weenie_result(hr: Hresult) -> u32 {
    match hr.0 {
        0 => 0,
        0x4112_001B => 0xC744,
        0x4112_001C => 0xC74D,
        0x8000_000A => 0xC73C,
        0x8007_0005 => 0xC74F,
        0xC112_0005 => 0xC748,
        0xC112_000C => 200,
        0xC112_000F => 0xC743,
        0xC112_0010 => 0xC74E,
        0xC112_0011 => 0xC73B,
        0xC112_0012 => 0xC73D,
        0xC112_0014 => 0xC744,
        0xC112_0018 => 0xC73F,
        0xC112_1003 => 0xC746,
        0xC112_1009 => 0xC740,
        0xC112_1401 => 0xC73E,
        0xC112_1800 => 0xC742,
        0xC112_1802 => 0xC747,
        0xC112_1803 => 0xC745,
        0xC112_1C04 => 0xC749,
        0xC112_2000 => 0xC74B,
        0xC112_2002 => 0xC74A,
        0xC112_2003 => 0xC74C,
        _ => 0xC741,
    }
}

/// A request's context waiting for its result: `TurbineGun::Request*` `AddRef`s it and
/// `RcvResult` releases it, so each lives for exactly one result. Retail keeps the object's
/// pointer in the `Cx` attribute; here [`crate::TurbineGun`] keys these by a
/// [`crate::Context`] id. The variants are the context classes the result handlers call.
#[derive(Clone)]
pub enum PendingContext {
    FriendAdd(Rc<dyn CtFriendAddResult>),
    FriendDelete(Rc<dyn CtFriendDeleteResult>),
    IgnoredAdd(Rc<dyn CtIgnoredAddResult>),
    IgnoredDelete(Rc<dyn CtIgnoredDeleteResult>),
    GetFriendsList(Rc<dyn CtGetFriendsListResult>),
    GetIgnoredList(Rc<dyn CtGetIgnoredListResult>),
    Flags(Rc<dyn CtFlagsResult>),
    Location(Rc<dyn CtLocationResult>),
    SendToRoom(Rc<dyn CtSendToRoomResult>),
    SendToUser(Rc<dyn CtSendToUserResult>),
    CallCreate(Rc<dyn CtCallCreateResult>),
    Ticket(Rc<dyn CtTicketResult>),
    JoinRoom(Rc<dyn CtJoinRoomResult>),
    CreateRoom(Rc<dyn CtCreateRoomResult>),
    LeaveRoom(Rc<dyn CtLeaveRoomResult>),
    LeaveAllRooms(Rc<dyn CtLeaveAllRoomsResult>),
    CreatePersistentChatChannel(Rc<dyn CtCreatePersistentChatChannelResult>),
    DeletePersistentChatChannel(Rc<dyn CtDeletePersistentChatChannelResult>),
    Persona(Rc<dyn CtPersonaResult>),
}

impl PendingContext {
    /// The `CTResult` part every context has (`m_hResult`, `IncEventsHandled`).
    pub fn as_result(&self) -> &dyn CtResult {
        match self {
            PendingContext::FriendAdd(c) => &**c,
            PendingContext::FriendDelete(c) => &**c,
            PendingContext::IgnoredAdd(c) => &**c,
            PendingContext::IgnoredDelete(c) => &**c,
            PendingContext::GetFriendsList(c) => &**c,
            PendingContext::GetIgnoredList(c) => &**c,
            PendingContext::Flags(c) => &**c,
            PendingContext::Location(c) => &**c,
            PendingContext::SendToRoom(c) => &**c,
            PendingContext::SendToUser(c) => &**c,
            PendingContext::CallCreate(c) => &**c,
            PendingContext::Ticket(c) => &**c,
            PendingContext::JoinRoom(c) => &**c,
            PendingContext::CreateRoom(c) => &**c,
            PendingContext::LeaveRoom(c) => &**c,
            PendingContext::LeaveAllRooms(c) => &**c,
            PendingContext::CreatePersistentChatChannel(c) => &**c,
            PendingContext::DeletePersistentChatChannel(c) => &**c,
            PendingContext::Persona(c) => &**c,
        }
    }
}
