//! The friends list (`gmCWInterface`'s friend calls and the GUN events and results). The requests go
//! to the GUN service: the UI's friend add and delete are the C++ `gmCWInterface::RequestGUN_*`
//! calls (`TurbineGun`, no script), and the script's own requests are its `RequestGUN_*` efuns,
//! which the host queues ([`crate::Client::take_gun_requests`]). The answers come back as the
//! script's IFuns, which raise the `CM_Friends` notices: `Result_FriendAdd` and `Result_FriendDelete`
//! take a weenie result and the name; the list comes in the `FriendStart` event (`GUNEvent_FriendStart`,
//! the Legions path: `CCliFriendsEvent::FriendStart` builds the list), and each status change in
//! `GUNEvent_FriendStatus`. Without a GUN service (`Client::set_gun_service`) the UI's requests
//! answer 0 and queue nothing, as retail's `RequestGUN_*` do when `TurbineGun` is absent. The
//! friends page (`ac2_ui::FriendsPanel`, the HUD's `hud_friends`) takes the list and status events
//! ([`crate::Client::take_friend_events`]) and sends its requests back here.

use ac2_net::protocol::NativeObject as NetNative;
use ac2_script::host::{ChatRequest, IFunArg};
use ac2_script::native::NativeObject;
use ac2_ui::FriendsRequest;
use ac2_weenie::{FriendInfo, SessionEvent};

use crate::client::Client;
use crate::scripts::*;

/// The weenie result of a GUN code that is not in the table: `CTBullet::ConvertToWeenieResult`'s
/// default (`0xc741`, its `"???"` path).
pub const UNKNOWN_RESULT: u32 = 0xc741;

/// `CTBullet::ConvertToWeenieResult` (admin 0x735C60): a GUN result code and the weenie result the
/// scripts read. Each row is one `if (result == …) return …;` of the admin decompile; the names
/// come from GunDll's `gun_errors.csv` (`0x8000000a` is not named there).
const WEENIE_RESULTS: &[(u32, u32)] = &[
    (0xc1121009, 0xc740), // ZT_E_ALREADY_IN_LIST
    (0xc1120010, 0xc74e), // ZT_E_TIMEOUT
    (0xc1120005, 0xc748), // ZT_E_STR_TOO_LONG
    (0x8000000a, 0xc73c), // (not in gun_errors.csv)
    (0x80070005, 0xc74f), // E_ACCESSDENIED
    (0xc112000c, 0xc8),   // ZT_E_INVALID_CHARACTER
    (0xc112000f, 0xc743), // ZT_E_INVALID_OP
    (0xc1120014, 0xc744), // ZT_E_ALREADY_EXISTS
    (0xc1120011, 0xc73b), // ZT_E_NOT_LOGGED_IN
    (0xc1120012, 0xc73d), // ZT_E_INVALID_USER
    (0xc1120018, 0xc73f), // ZT_E_REQUEST_DENIED
    (0xc1121003, 0xc746), // ZT_E_INVALID_PASSWORD
    (0xc1122000, 0xc74b), // ZT_E_INVALID_PERSONA
    (0xc1121802, 0xc747), // ZT_E_NOT_INVITED
    (0xc1121401, 0xc73e), // ZT_E_NOT_IN_LIST
    (0xc1121800, 0xc742), // ZT_E_INVALID_ROOM
    (0xc1121803, 0xc745), // ZT_E_ROOM_LIMIT
    (0xc1121c04, 0xc749), // ZT_E_NOT_FOUND
    (0x00000000, 0x0),    // S_OK
    (0xc1122002, 0xc74a), // ZT_E_PERSONA_ALREADY_TAKEN
    (0xc1122003, 0xc74c), // ZT_E_INVALID_SHARDID
    (0x4112001b, 0xc744), // ZT_S_ALREADY_EXISTS
    (0x4112001c, 0xc74d), // ZT_S_DOESNT_EXIST
];

/// The weenie result of a GUN result code (`CTBullet::ConvertToWeenieResult`).
pub fn weenie_result(gun: u32) -> u32 {
    WEENIE_RESULTS
        .iter()
        .find(|(code, _)| *code == gun)
        .map_or(UNKNOWN_RESULT, |(_, weenie)| *weenie)
}

/// A GUN result for a friend add or delete: the GUN result code (0: done) and the friend's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendResult {
    Added { gun: u32, name: String },
    Removed { gun: u32, name: String },
}

/// A GUN event for the friends list: the list (`FriendStart`) or one friend's status
/// (`FriendStatus`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendEvent {
    Start(Vec<FriendInfo>),
    Status(FriendInfo),
}

impl Client {
    /// `gmCWInterface::RequestGUN_FriendAdd` (the UI's add): queued for the GUN service. Without
    /// one retail returns 0 and queues nothing (`TurbineGun::S_pTurbineGun` is NULL). Returns
    /// whether it was queued.
    pub fn friend_add(&mut self, name: &str) -> bool {
        self.gun_request(ChatRequest::FriendAdd(name.to_owned()))
    }

    /// `gmCWInterface::RequestGUN_FriendDelete`: queued for the GUN service (see
    /// [`Client::friend_add`]).
    pub fn friend_delete(&mut self, name: &str) -> bool {
        self.gun_request(ChatRequest::FriendDelete(name.to_owned()))
    }

    /// `gmCWInterface::RequestGUN_GetFriendsList(userRequested)`: queued for the GUN service.
    pub fn friends_list(&mut self, user_requested: bool) -> bool {
        self.gun_request(ChatRequest::GetFriendsList { user_requested })
    }

    /// `gmCWInterface::RequestGUN_SetLocation(flags, "")`, the status drop-down's request: queued
    /// for the GUN service.
    pub fn friend_status(&mut self, flags: u32) -> bool {
        self.gun_request(ChatRequest::SetLocation {
            flags,
            location: String::new(),
        })
    }

    /// A GUN add or delete result: converted to the weenie result, then the script's
    /// `Result_FriendAdd` or `Result_FriendDelete` IFun, whose notices (`FriendAdded`,
    /// `FriendRemoved`) the friends page takes.
    pub fn gun_friend_result(&mut self, result: FriendResult) -> Vec<SessionEvent> {
        let (function, gun, name) = match result {
            FriendResult::Added { gun, name } => (IFUN_RESULT_FRIEND_ADD, gun, name),
            FriendResult::Removed { gun, name } => (IFUN_RESULT_FRIEND_DELETE, gun, name),
        };
        self.store_ifun(
            function,
            &[
                IFunArg::Int(weenie_result(gun)),
                IFunArg::Native(NetNative::WString(name)),
            ],
        )
    }

    /// The GUN list and status events the friends page has not taken yet, in order
    /// ([`FriendEvent`]: the `FriendStart` list and each `FriendStatus`).
    pub fn take_friend_events(&mut self) -> Vec<FriendEvent> {
        std::mem::take(&mut self.friend_events)
    }

    /// A GUN event for the list: the script's `GUNEvent_FriendStart` gets the list as an `RList`
    /// of `TurbineUserStatus` natives; `GUNEvent_FriendStatus` gets one status native. The
    /// notices they raise come back here; the list and the status go to the friends page
    /// ([`Client::take_friend_events`]), since the start body raises no list notice.
    pub fn gun_friend_event(&mut self, event: FriendEvent) -> Vec<SessionEvent> {
        match event {
            FriendEvent::Start(friends) => {
                self.friend_events.push(FriendEvent::Start(friends.clone()));
                self.store_ifun(
                    IFUN_GUN_EVENT_FRIEND_START,
                    &[IFunArg::VmList(
                        ac2_script::native::ntype::TURBINE_USER_STATUS,
                        friends.into_iter().map(status_native).collect(),
                    )],
                )
            }
            FriendEvent::Status(friend) => {
                self.friend_events.push(FriendEvent::Status(friend.clone()));
                self.store_ifun(
                    IFUN_GUN_EVENT_FRIEND_STATUS,
                    &[IFunArg::Vm(
                        ac2_script::native::ntype::TURBINE_USER_STATUS,
                        status_native(friend),
                    )],
                )
            }
        }
    }

    /// The friends page's requests as the client makes them: the list, add, delete and the status
    /// (`gmFriendsUI::HandleStatusChange`). The status's emote plays whether or not there is a
    /// GUN service (the `DoEmote` is not behind the GUN call); it is the game server's emote.
    pub(crate) fn friends_command(&mut self, request: &FriendsRequest) {
        match request {
            FriendsRequest::GetList { user_requested } => {
                self.friends_list(*user_requested);
            }
            FriendsRequest::Add(name) => {
                self.friend_add(name);
            }
            FriendsRequest::Delete(name) => {
                self.friend_delete(name);
            }
            FriendsRequest::Status { flags, emote } => {
                self.friend_status(*flags);
                let id = emote.and_then(|word| self.session.chat().commands().emote(word));
                if let Some(id) = id {
                    self.session.emote(id);
                }
            }
        }
    }

    /// Queues a GUN request for the service, as `gmCWInterface::RequestGUN_*` does: nothing
    /// without a GUN service (`false`, the retail 0), else the request is queued (`true`).
    fn gun_request(&mut self, request: ChatRequest) -> bool {
        match self.scripts.as_mut() {
            Some(scripts) if scripts.gun_service => {
                scripts.gun_requests.push(request);
                true
            }
            _ => false,
        }
    }
}

/// A friend as the script's `TurbineUserStatus` native.
fn status_native(friend: FriendInfo) -> NativeObject {
    NativeObject::TurbineUserStatus(Box::new(friend))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table's rows are the admin decompile's: the duplicate add, the not-in-list delete, the
    /// success (0), the default for an unnamed code.
    #[test]
    fn gun_results_convert_as_the_admin_build_does() {
        assert_eq!(weenie_result(0xc1121009), 0xc740, "already in the list");
        assert_eq!(weenie_result(0xc1121401), 0xc73e, "not in the list");
        assert_eq!(weenie_result(0xc1120012), 0xc73d, "invalid user");
        assert_eq!(weenie_result(0x00000000), 0, "done");
        assert_eq!(weenie_result(0x12345678), UNKNOWN_RESULT, "the default");
    }
}
