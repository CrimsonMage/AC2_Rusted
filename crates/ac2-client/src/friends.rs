//! The friends list (`gmCWInterface`'s friend calls and the GUN events and results). The requests go
//! to the GUN service: the UI's friend add and delete are the C++ `gmCWInterface::RequestGUN_*`
//! calls (`TurbineGun`, no script), and the script's own requests are its `RequestGUN_*` efuns,
//! which the host queues ([`crate::Client::take_gun_requests`]). The answers come back as the
//! script's IFuns, which raise the `CM_Friends` notices: `Result_FriendAdd` and `Result_FriendDelete`
//! take a weenie result and the name; the list comes in the `FriendStart` event (`GUNEvent_FriendStart`,
//! the Legions path: `CCliFriendsEvent::FriendStart` builds the list), and each status change in
//! `GUNEvent_FriendStatus`.

use ac2_net::protocol::NativeObject as NetNative;
use ac2_script::host::{ChatRequest, IFunArg};
use ac2_script::native::NativeObject;
use ac2_ui::FriendsRequest;
use ac2_weenie::{FriendInfo, GameplayNotice, SessionEvent};

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
    /// `gmCWInterface::RequestGUN_FriendAdd` (the UI's add): queued for the GUN service.
    pub fn friend_add(&mut self, name: &str) {
        self.gun_request(ChatRequest::FriendAdd(name.to_owned()));
    }

    /// `gmCWInterface::RequestGUN_FriendDelete`: queued for the GUN service.
    pub fn friend_delete(&mut self, name: &str) {
        self.gun_request(ChatRequest::FriendDelete(name.to_owned()));
    }

    /// `gmCWInterface::RequestGUN_GetFriendsList(userRequested)`: queued for the GUN service.
    pub fn friends_list(&mut self, user_requested: bool) {
        self.gun_request(ChatRequest::GetFriendsList { user_requested });
    }

    /// A GUN add or delete result: converted to the weenie result, then the script's
    /// `Result_FriendAdd` or `Result_FriendDelete` IFun, whose notices come back here.
    pub fn gun_friend_result(&mut self, result: FriendResult) -> Vec<SessionEvent> {
        let (function, gun, name) = match result {
            FriendResult::Added { gun, name } => (IFUN_RESULT_FRIEND_ADD, gun, name),
            FriendResult::Removed { gun, name } => (IFUN_RESULT_FRIEND_DELETE, gun, name),
        };
        let events = self.store_ifun(
            function,
            &[
                IFunArg::Int(weenie_result(gun)),
                IFunArg::Native(NetNative::WString(name)),
            ],
        );
        // The panel follows the notices: an added friend asks for the list again, a removed one
        // leaves it (`gmFriendsUI::RecvNotice_GUNResult_AddFriend` and `..._RemoveFriend`).
        for event in &events {
            match event {
                SessionEvent::Notice(GameplayNotice::FriendAdded(_)) => {
                    let FriendsRequest::GetList { user_requested } = self.friends.recv_added();
                    self.friends_list(user_requested);
                }
                SessionEvent::Notice(GameplayNotice::FriendRemoved(name)) => {
                    self.friends.recv_removed(name);
                }
                _ => {}
            }
        }
        events
    }

    /// The friends panel's state (its rows, selection and visible window).
    pub fn friends_panel(&self) -> &ac2_ui::FriendsUi {
        &self.friends
    }

    /// Selects a friend row in the panel (`UIElement_ListBox::SetSelectedIndex`).
    pub fn friend_select(&mut self, index: Option<usize>) {
        self.friends.select(index);
    }

    /// `gmFriendsUI::HandleRemoveButton`: deletes the selected friend at the GUN service. Nothing
    /// without a selection.
    pub fn friend_remove_selected(&mut self) {
        if let Some(name) = self.friends.remove_request() {
            self.friend_delete(&name);
        }
    }

    /// A GUN event for the list: the script's `GUNEvent_FriendStart` gets the list as an `RList`
    /// of `TurbineUserStatus` natives; `GUNEvent_FriendStatus` gets one status native. The
    /// notices they raise come back here.
    pub fn gun_friend_event(&mut self, event: FriendEvent) -> Vec<SessionEvent> {
        match event {
            FriendEvent::Start(friends) => {
                // The panel takes the GUN list itself (`BuildUIList`'s input): the script's start
                // body raises no list notice in Legions.
                self.friends.recv_list(&friends);
                self.store_ifun(
                    IFUN_GUN_EVENT_FRIEND_START,
                    &[IFunArg::VmList(
                        ac2_script::native::ntype::TURBINE_USER_STATUS,
                        friends.into_iter().map(status_native).collect(),
                    )],
                )
            }
            FriendEvent::Status(friend) => {
                self.friends.recv_status(&friend);
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

    /// Queues a GUN request for the service (no VM needed: the UI's calls are C++).
    fn gun_request(&mut self, request: ChatRequest) {
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.gun_requests.push(request);
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
