//! The friends list (`gmCWInterface`'s friend calls). The UI's friend add, delete, list, status and
//! the assistance call are the C++ `gmCWInterface::RequestGUN_*` calls
//! (`ac2_weenie::gun::interface`: `TurbineGun` directly, no script; 0 when there is none), and
//! the script's own requests are its `RequestGUN_*` efuns, which call the same functions. The
//! answers come back through `TurbineGun`'s result handlers (`ac2_weenie::gun::handlers`) and are
//! carried out by [`crate::gun`] as the script's IFuns, which raise the `CM_Friends` notices; the
//! list and each status change also go to the friends page
//! ([`crate::Client::take_friend_events`]).

use ac2_net::protocol::NativeObject as NetNative;
use ac2_script::host::IFunArg;
use ac2_ui::FriendsRequest;
use ac2_weenie::gun::interface as gmcw;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::scripts::*;

pub use ac2_ui::FriendEvent;

impl Client {
    /// `gmCWInterface::RequestGUN_FriendAdd` (the UI's add). Without a `TurbineGun` retail
    /// returns 0. Returns whether `TurbineGun` took the request.
    pub fn friend_add(&mut self, name: &str) -> bool {
        gmcw::request_gun_friend_add(name) != 0
    }

    /// `gmCWInterface::RequestGUN_FriendDelete`.
    pub fn friend_delete(&mut self, name: &str) -> bool {
        gmcw::request_gun_friend_delete(name) != 0
    }

    /// `gmCWInterface::RequestGUN_GetFriendsList(userRequested)`.
    pub fn friends_list(&mut self, user_requested: bool) -> bool {
        gmcw::request_gun_get_friends_list(u32::from(user_requested)) != 0
    }

    /// `gmCWInterface::RequestGUN_CallCreate(callType, worldName, targetUserName, description)`,
    /// the assistance page's call (Legions 0x44458B). Without a `TurbineGun` retail returns 0
    /// (the page then shows its "can't connect" box).
    pub fn call_create(
        &mut self,
        call_type: u32,
        world: &str,
        target: &str,
        description: &str,
    ) -> bool {
        gmcw::request_gun_call_create(call_type, world, target, description) != 0
    }

    /// `gmCWInterface::RequestGUN_SetLocation(mode, "")`, the status drop-down's request.
    pub fn friend_status(&mut self, flags: u32) -> bool {
        gmcw::request_gun_set_location(flags, "") != 0
    }

    /// The GUN list and status events the friends page has not taken yet, in order
    /// ([`FriendEvent`]: the `FriendStart` list and each `FriendStatus`).
    pub fn take_friend_events(&mut self) -> Vec<FriendEvent> {
        std::mem::take(&mut self.friend_events)
    }

    /// The friends page's requests as the client makes them: the list, add, delete and the status
    /// (`gmFriendsUI::HandleStatusChange`). The status's emote is `gmCWInterface::DoEmote`, the
    /// player's script's IFun; it runs whether or not there is a GUN service (the `DoEmote` is
    /// not behind the GUN call). Returns the IFun's output.
    pub(crate) fn friends_command(&mut self, request: &FriendsRequest) -> Vec<SessionEvent> {
        match request {
            FriendsRequest::GetList { user_requested } => {
                self.friends_list(*user_requested);
                Vec::new()
            }
            FriendsRequest::Add(name) => {
                self.friend_add(name);
                Vec::new()
            }
            FriendsRequest::Delete(name) => {
                self.friend_delete(name);
                Vec::new()
            }
            FriendsRequest::Status { flags, emote } => {
                self.friend_status(*flags);
                match emote {
                    Some(name) => self.do_emote(ac2_ui::tokenize(name)),
                    None => Vec::new(),
                }
            }
        }
    }

    /// `gmCWInterface::DoEmote(emoteName)`: the player's script's IFun 0xB400EC with the emote's
    /// `StringInfo` (the emote table's string `string_id`). Retail calls the IFun only with a
    /// process on the interface, so with no script running the player's script does nothing.
    pub(crate) fn do_emote(&mut self, string_id: u32) -> Vec<SessionEvent> {
        if !self.scripts_own(IFUN_DO_EMOTE) {
            return Vec::new();
        }
        let table_id = self
            .session
            .player_script()
            .string_tables()
            .did(ac2_weenie::TABLE_ENUM_EMOTE);
        let info = ac2_net::protocol::StringInfo {
            string_id,
            table_id,
            literal: None,
            variables: Vec::new(),
        };
        self.script_ifun_args(
            IFUN_DO_EMOTE,
            &[IFunArg::Native(NetNative::StringInfo(info))],
        )
    }
}
