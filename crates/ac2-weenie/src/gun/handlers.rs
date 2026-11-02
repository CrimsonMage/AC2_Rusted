//! The client's `CCli*` classes (`ac2/client/src/gun/event`, `.../result` and `gmclient.cpp`),
//! which implement `ac2-gun`'s `Ct*` traits. Retail's objects derive from `CTBullet` with a
//! virtual `ReferenceTable` count; here each is an `Rc` (the count) with a `CtBullet` /
//! `CtResultState`. Every handler starts with `ClientGunInterface::GetPlayerInterface()`: with
//! no player Legions returns silently (admin reports a `FATAL ERROR` first, which Legions
//! dropped); `CCliCallCreateResult` has no check even in admin.

use std::cell::Cell;
use std::rc::Rc;

use ac2_gun::ct::{
    convert_to_weenie_result, Bullet, CtBullet, CtCallCreateResult, CtChatEvent, CtConnectEvent,
    CtEvent, CtFlagsResult, CtFriendAddResult, CtFriendDeleteResult, CtFriendsEvent,
    CtGetFriendsListResult, CtGetIgnoredListResult, CtIgnoredAddResult, CtIgnoredDeleteResult,
    CtLocationResult, CtResult, CtResultState, CtSendToRoomResult, CtSendToUserResult,
    CtTicketResult, CtTicketState, EConnectEvent, ETicketResult,
};
use ac2_gun::turbine_gun_utils::global_to_local_name;
use ac2_gun::{TagBlob, UserStatus};

use super::chat_extra::{gun_event_hear_from_room, hear_tell};
use super::output::{GunNotice, GunOutput};
use super::shared::shared;
use crate::friends::FriendInfo;

/// `CCliConnectEvent` (constructor admin 0x004035C0, Legions 0x00403530; `HandleConnectEvent`
/// admin 0x00728380, Legions 0x006A6660): `StartupGUN` waits on its `NumEventsHandled`, and each
/// connect-family event becomes a `CM_UI` notice.
#[derive(Default)]
pub struct CCliConnectEvent {
    bullet: CtBullet,
}

impl CCliConnectEvent {
    pub fn new() -> Rc<Self> {
        Rc::new(Self::default())
    }

    /// `NumEventsHandled()` (`CTBullet`, admin 0x00402190, Legions 0x00402130): what `StartupGUN`
    /// spins on.
    pub fn num_events_handled_now(&self) -> u32 {
        self.bullet.num_events_handled()
    }
}

impl Bullet for CCliConnectEvent {
    fn bullet(&self) -> &CtBullet {
        &self.bullet
    }
}
impl CtEvent for CCliConnectEvent {}

impl CtConnectEvent for CCliConnectEvent {
    /// `switch (evt)`: CONNECT -> `SendNotice_GunConnected` (0x469); RECONNECTED ->
    /// `SendNotice_GunReconnected` (0x46B); DISCONNECT, UNDEF and anything else ->
    /// `SendNotice_GunConnectionLost` (0x46A); CONNECT_FAILED -> nothing.
    fn handle_connect_event(&self, evt: EConnectEvent) {
        let notice = match evt {
            EConnectEvent::Connect => GunNotice::Connected,
            EConnectEvent::Reconnected => GunNotice::Reconnected,
            EConnectEvent::Disconnect | EConnectEvent::Undef => GunNotice::ConnectionLost,
            EConnectEvent::ConnectFailed => return,
        };
        shared().send_notice(notice);
    }
}

/// `CCliChatEvent` (the only chat listener the game registers): `HearRoom` (admin 0x007281F0,
/// Legions 0x006A64D0) and `HearUser` (admin 0x00728220, Legions 0x006A6500); the other slots are
/// `ret N` stubs in both builds.
#[derive(Default)]
pub struct CCliChatEvent {
    bullet: CtBullet,
}

impl CCliChatEvent {
    pub fn new() -> Rc<Self> {
        Rc::new(Self::default())
    }
}

impl Bullet for CCliChatEvent {
    fn bullet(&self) -> &CtBullet {
        &self.bullet
    }
}
impl CtEvent for CCliChatEvent {}

impl CtChatEvent for CCliChatEvent {
    fn invited(&self, _room_name: &str, _invitation: &str) {}
    fn room_enter(&self, _room_id: u32, _user_name: &str) {}
    fn room_leave(&self, _room_id: u32, _user_name: &str) {}
    /// `if (p = ClientGunInterface::GetPlayerInterface()) p->GUNEvent_HearFromRoom(user, speech,
    /// roomID, extra)`.
    fn hear_room(&self, room_id: u32, user: &str, speech: &str, extra: Option<&TagBlob>) {
        let shared = shared();
        if shared.player_present() {
            gun_event_hear_from_room(&shared, user, speech, room_id, extra);
        }
    }
    fn kicked_out(&self, _room_id: u32, _reason: &str) {}
    fn receive_ownership(&self, _room_id: u32) {}
    /// `if (p) p->HearTell(user, speech, extra)`.
    fn hear_user(&self, user: &str, speech: &str, extra: Option<&TagBlob>) {
        let shared = shared();
        if shared.player_present() {
            hear_tell(&shared, user, speech, extra);
        }
    }
}

/// A `TurbineUserStatus` from a GUN `UserStatus`: the name through
/// `TurbineGunUtils::GlobalToLocalName`, the location when it is not empty, the mode.
fn friend_info(status: &UserStatus) -> FriendInfo {
    FriendInfo {
        name: global_to_local_name(&status.user_name),
        location: status.location.clone(),
        mode: status.mode,
    }
}

/// `CCliFriendsEvent`: `FriendStatus` (admin 0x007283C0, Legions 0x006A66D0) and `FriendStart`
/// (admin 0x00728500, Legions 0x006A6810).
#[derive(Default)]
pub struct CCliFriendsEvent {
    bullet: CtBullet,
}

impl CCliFriendsEvent {
    pub fn new() -> Rc<Self> {
        Rc::new(Self::default())
    }
}

impl Bullet for CCliFriendsEvent {
    fn bullet(&self) -> &CtBullet {
        &self.bullet
    }
}
impl CtEvent for CCliFriendsEvent {}

impl CtFriendsEvent for CCliFriendsEvent {
    /// `FriendStart(count, arr)`: each name through `GlobalToLocalName`, the location when it is
    /// not NULL or empty, the mode; `p->GUNEvent_FriendStart(list)`.
    fn friend_start(&self, statuses: &[UserStatus]) {
        let shared = shared();
        if shared.player_present() {
            let list = statuses.iter().map(friend_info).collect();
            shared.push(GunOutput::EventFriendStart(list));
        }
    }

    /// `FriendStatus(friendName, mode)`: a `TurbineUserStatus` with the name (already local:
    /// `RcvFriendsEvent` converted it), an empty location and the mode;
    /// `p->GUNEvent_FriendStatus(status)`.
    fn friend_status(&self, friend_name: &str, mode: u32) {
        let shared = shared();
        if shared.player_present() {
            shared.push(GunOutput::EventFriendStatus(FriendInfo {
                name: friend_name.to_owned(),
                location: String::new(),
                mode,
            }));
        }
    }
}

/// `CCliTicketResult` (0x2C bytes; constructor admin 0x007282E0, Legions 0x006A65C0): the
/// context of `PresentTicket`, and `G_pGunTicketHandler`'s class.
pub struct CCliTicketResult {
    state: CtResultState,
    ticket: CtTicketState,
    /// `m_fGetTicketRequestArrived` (+0x14).
    get_ticket_request_arrived: Cell<bool>,
    /// `m_fGetTicketResult` (+0x18).
    get_ticket_result: Cell<ETicketResult>,
    /// `m_fAuthenticateRequestArrived` (+0x1C).
    authenticate_request_arrived: Cell<bool>,
    /// `m_fAuthenticateResult` (+0x20).
    authenticate_result: Cell<ETicketResult>,
}

impl CCliTicketResult {
    /// The constructor: both results FAILURE (0xC1121006), both arrived flags 0.
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
            ticket: CtTicketState::new(),
            get_ticket_request_arrived: Cell::new(false),
            get_ticket_result: Cell::new(ETicketResult::Failure),
            authenticate_request_arrived: Cell::new(false),
            authenticate_result: Cell::new(ETicketResult::Failure),
        })
    }

    pub fn get_ticket_request_arrived(&self) -> bool {
        self.get_ticket_request_arrived.get()
    }

    pub fn get_ticket_result(&self) -> ETicketResult {
        self.get_ticket_result.get()
    }

    /// `m_fAuthenticateRequestArrived`: `StartupGUN`'s second wait loop.
    pub fn authenticate_request_arrived(&self) -> bool {
        self.authenticate_request_arrived.get()
    }

    /// `m_fAuthenticateResult`.
    pub fn authenticate_result(&self) -> ETicketResult {
        self.authenticate_result.get()
    }
}

impl Bullet for CCliTicketResult {
    fn bullet(&self) -> &CtBullet {
        self.state.bullet()
    }
}
impl CtResult for CCliTicketResult {
    fn result_state(&self) -> &CtResultState {
        &self.state
    }
}

impl CtTicketResult for CCliTicketResult {
    fn ticket_state(&self) -> &CtTicketState {
        &self.ticket
    }

    /// `HandleTicketArrived` (admin 0x007282A0, Legions 0x006A6580): `m_fGetTicketRequestArrived
    /// = 1; m_fGetTicketResult = result`.
    fn handle_ticket_arrived(&self, result: ETicketResult) {
        self.get_ticket_request_arrived.set(true);
        self.get_ticket_result.set(result);
    }

    /// `HandleAuthenticationResult` (admin 0x007282C0, Legions 0x006A65A0):
    /// `m_fAuthenticateResult = result; m_fAuthenticateRequestArrived = 1;
    /// CM_UI::SendNotice_GunAuthenticated(result == SUCCESS)`.
    fn handle_authentication_result(&self, result: ETicketResult) {
        self.authenticate_result.set(result);
        self.authenticate_request_arrived.set(true);
        shared().send_notice(GunNotice::Authenticated(result == ETicketResult::Success));
    }

    /// Slot `+0x10`: a stub in `CCliTicketResult`.
    fn handle_release_ticket_result(&self, _result: ETicketResult) {}
}

/// The result classes' shared parts.
macro_rules! result_base {
    ($t:ty) => {
        impl Bullet for $t {
            fn bullet(&self) -> &CtBullet {
                self.state.bullet()
            }
        }
        impl CtResult for $t {
            fn result_state(&self) -> &CtResultState {
                &self.state
            }
        }
        impl $t {
            /// `CTBullet::ConvertToWeenieResult(m_hResult)`.
            #[allow(dead_code)]
            fn weenie_result(&self) -> u32 {
                convert_to_weenie_result(self.h_result())
            }
        }
    };
}

/// A `CCli<X>Result` whose payload is one name (`+0x10 m_name`), 0x1C bytes.
macro_rules! name_result {
    ($(#[$doc:meta])* $cls:ident, $trait_:ident, $method:ident, $variant:ident) => {
        $(#[$doc])*
        pub struct $cls {
            state: CtResultState,
            name: String,
        }
        impl $cls {
            pub fn new(name: &str) -> Rc<Self> {
                Rc::new(Self {
                    state: CtResultState::new(),
                    name: name.to_owned(),
                })
            }
        }
        result_base!($cls);
        impl $trait_ for $cls {
            fn $method(&self) {
                let shared = shared();
                if !shared.player_present() {
                    return;
                }
                shared.push(GunOutput::$variant {
                    result: self.weenie_result(),
                    name: self.name.clone(),
                });
            }
        }
    };
}

name_result!(
    /// `CCliFriendAddResult::HandleFriendAddResult` (admin 0x00728690, Legions 0x006A69A0):
    /// `Result_FriendAdd(p, w, &m_name)`.
    CCliFriendAddResult,
    CtFriendAddResult,
    handle_friend_add_result,
    ResultFriendAdd
);
name_result!(
    /// `CCliFriendDeleteResult` (admin 0x00728780, Legions 0x006A6AE0): `Result_FriendDelete`.
    CCliFriendDeleteResult,
    CtFriendDeleteResult,
    handle_friend_delete_result,
    ResultFriendDelete
);
name_result!(
    /// `CCliIgnoredAddResult` (admin 0x00728AD0, Legions 0x006A6E40): `Result_IgnoredAdd`.
    CCliIgnoredAddResult,
    CtIgnoredAddResult,
    handle_ignored_add_result,
    ResultIgnoredAdd
);
name_result!(
    /// `CCliIgnoredDeleteResult` (admin 0x00728BC0, Legions 0x006A6F20): `Result_IgnoredDelete`.
    CCliIgnoredDeleteResult,
    CtIgnoredDeleteResult,
    handle_ignored_delete_result,
    ResultIgnoredDelete
);

/// `CCliGetFriendsListResult` (admin 0x00728910, Legions 0x006A6C90), `+0x10 m_userRequested`.
pub struct CCliGetFriendsListResult {
    state: CtResultState,
    user_requested: u32,
}

impl CCliGetFriendsListResult {
    pub fn new(user_requested: u32) -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
            user_requested,
        })
    }
}
result_base!(CCliGetFriendsListResult);

impl CtGetFriendsListResult for CCliGetFriendsListResult {
    /// `handler(count, arr)`: the list as `FriendStart` builds it, then
    /// `Result_GetFriendsList(p, w, m_userRequested, list)`.
    fn handle_get_friends_list_result(&self, statuses: &[UserStatus]) {
        let shared = shared();
        if !shared.player_present() {
            return;
        }
        shared.push(GunOutput::ResultGetFriendsList {
            result: self.weenie_result(),
            user_requested: self.user_requested,
            list: statuses.iter().map(friend_info).collect(),
        });
    }
}

/// `CCliGetIgnoredListResult` (admin 0x00728DD0, Legions 0x006A70C0).
pub struct CCliGetIgnoredListResult {
    state: CtResultState,
    user_requested: u32,
}

impl CCliGetIgnoredListResult {
    pub fn new(user_requested: u32) -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
            user_requested,
        })
    }
}
result_base!(CCliGetIgnoredListResult);

impl CtGetIgnoredListResult for CCliGetIgnoredListResult {
    /// `handler(count, names[])`: each name through `GlobalToLocalName`, then
    /// `Result_GetIgnoredList(p, w, m_userRequested, list)`.
    fn handle_get_ignored_list_result(&self, names: &[String]) {
        let shared = shared();
        if !shared.player_present() {
            return;
        }
        shared.push(GunOutput::ResultGetIgnoredList {
            result: self.weenie_result(),
            user_requested: self.user_requested,
            list: names.iter().map(|n| global_to_local_name(n)).collect(),
        });
    }
}

/// `CCliSendToUserResult` (0x20 bytes; admin 0x00729060, Legions 0x006A7340).
pub struct CCliSendToUserResult {
    state: CtResultState,
    target: String,
    message: String,
}

impl CCliSendToUserResult {
    pub fn new(target: &str, message: &str) -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
            target: target.to_owned(),
            message: message.to_owned(),
        })
    }
}
result_base!(CCliSendToUserResult);

impl CtSendToUserResult for CCliSendToUserResult {
    /// `Result_SendToUser(p, &m_target, &m_message, w)`.
    fn handle_send_to_user_result(&self) {
        let shared = shared();
        if !shared.player_present() {
            return;
        }
        shared.push(GunOutput::ResultSendToUser {
            user: self.target.clone(),
            message: self.message.clone(),
            result: self.weenie_result(),
        });
    }
}

/// `CCliSendToRoomResult` (0x20 bytes; admin 0x007291F0, Legions 0x006A74C0).
pub struct CCliSendToRoomResult {
    state: CtResultState,
    room_id: u32,
    message: String,
}

impl CCliSendToRoomResult {
    pub fn new(room_id: u32, message: &str) -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
            room_id,
            message: message.to_owned(),
        })
    }
}
result_base!(CCliSendToRoomResult);

impl CtSendToRoomResult for CCliSendToRoomResult {
    /// `Result_SendToRoom(p, &m_message, m_roomID, w)`.
    fn handle_send_to_room_result(&self) {
        let shared = shared();
        if !shared.player_present() {
            return;
        }
        shared.push(GunOutput::ResultSendToRoom {
            message: self.message.clone(),
            room: self.room_id,
            result: self.weenie_result(),
        });
    }
}

/// `CCliCallCreateResult` (0x24 bytes; admin 0x00728EE0, Legions 0x006A71C0): `+0x10 callType,
/// +0x14 target, +0x18 description`.
pub struct CCliCallCreateResult {
    state: CtResultState,
    call_type: u32,
    target: String,
    description: String,
}

impl CCliCallCreateResult {
    pub fn new(call_type: u32, target: &str, description: &str) -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
            call_type,
            target: target.to_owned(),
            description: description.to_owned(),
        })
    }
}
result_base!(CCliCallCreateResult);

impl CtCallCreateResult for CCliCallCreateResult {
    /// `Result_CallCreate(p, w, callType, &target, &description, queueId, queuePos)`; no
    /// player check even in admin.
    fn handle_call_create_result(&self, queue_id: u32, queue_pos: u32) {
        shared().push(GunOutput::ResultCallCreate {
            result: self.weenie_result(),
            call_type: self.call_type,
            target: self.target.clone(),
            description: self.description.clone(),
            queue_id,
            queue_pos,
        });
    }
}

/// `CCliFlagsResult` (0x18 bytes): the vtable slot is a stub, the result is ignored.
pub struct CCliFlagsResult {
    state: CtResultState,
}

impl CCliFlagsResult {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
        })
    }
}
result_base!(CCliFlagsResult);

impl CtFlagsResult for CCliFlagsResult {
    fn handle_flags_result(&self, _which_flag: u32, _flags: u32) {}
}

/// `CCliLocationResult` (0x18 bytes): the slot is a stub.
pub struct CCliLocationResult {
    state: CtResultState,
}

impl CCliLocationResult {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            state: CtResultState::new(),
        })
    }
}
result_base!(CCliLocationResult);

impl CtLocationResult for CCliLocationResult {
    fn handle_location_result(&self) {}
}
