//! The friends list (`gmCWInterface`'s friend calls and the GUN results). The requests go to the
//! GUN service: the UI's friend add and delete are the C++ `gmCWInterface::RequestGUN_*` calls
//! (`TurbineGun`, no script), and the script's own requests are its `RequestGUN_*` efuns, which the
//! host queues ([`crate::Client::take_gun_requests`]). The answers come back as the script's
//! `Result_FriendAdd` / `Result_FriendDelete` IFuns, which raise the `CM_Friends` notices.

use ac2_net::protocol::NativeObject;
use ac2_script::host::{ChatRequest, IFunArg};
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::scripts::*;

/// A GUN result for a friend add or delete: the weenie result (0: done) and the friend's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendResult {
    Added { result: u32, name: String },
    Removed { result: u32, name: String },
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

    /// A GUN result delivered to the script: `Result_FriendAdd` or `Result_FriendDelete` (the
    /// result, then the name as a wide string), whose notices come back here.
    pub fn gun_friend_result(&mut self, result: FriendResult) -> Vec<SessionEvent> {
        let (function, value, name) = match result {
            FriendResult::Added { result, name } => (IFUN_RESULT_FRIEND_ADD, result, name),
            FriendResult::Removed { result, name } => (IFUN_RESULT_FRIEND_DELETE, result, name),
        };
        self.store_ifun(
            function,
            &[
                IFunArg::Int(value),
                IFunArg::Native(NativeObject::WString(name)),
            ],
        )
    }

    /// Queues a GUN request for the service (no VM needed: the UI's calls are C++).
    fn gun_request(&mut self, request: ChatRequest) {
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.gun_requests.push(request);
        }
    }
}
