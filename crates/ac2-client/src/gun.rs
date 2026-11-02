//! GUN in the client's frame (`gmClient::Connect`, `gmClient::UseTime`, the notice receivers and
//! the `gmCWInterface::Result_*` / `GUNEvent_*` entry points), over `ac2_weenie::gun`.
//!
//! * `gmClient::Connect` (Legions 0x00404D20): `StartupGUN`, and only then the game connection
//!   (`Turbine::Client::Connect`: `InitAuth`, `ClientNet::Init`). The session is made held
//!   (`OnlineSession::connect_held`) and [`Client::use_time`] starts its logon when
//!   [`GunClient::connect`](ac2_weenie::gun::GunClient::connect) answers; `StartupGUN`'s two
//!   wait loops are that call, one iteration a frame, with the UI running (`KeepUIAlive`).
//! * `gmClient::UseTime` (Legions 0x00401DA0): `CheckForMessages` every frame.
//! * What the `CCli*` classes raised is carried out here, in order, right after
//!   `CheckForMessages` returns (retail runs the calls inside `DispatchQueue`).

use std::rc::Rc;

use ac2_net::protocol::StringInfo;
use ac2_net::NetError;
use ac2_script::host::interface_calls::InterfaceCalls;
use ac2_script::host::IFunArg;
use ac2_script::native::{ntype, NativeObject};
use ac2_weenie::gun::{shared, ConnectPoll, GunClient, GunError, GunNotice, GunOutput};
use ac2_weenie::{FriendInfo, SessionEvent};

use crate::client::Client;
use crate::friends::FriendEvent;
use crate::scripts::*;

/// `Result_GetFriendsList` (`CallIFun_Result_GetFriendsList`: the weenie result, `userRequested`,
/// the `RList<TurbineUserStatus>`; IFun 0xB40042).
pub const IFUN_RESULT_GET_FRIENDS_LIST: u32 = 0x00B4_0042;
/// `Result_GetIgnoredList` (`CallIFun_Result_GetIgnoredList`; IFun 0xB40047).
pub const IFUN_RESULT_GET_IGNORED_LIST: u32 = 0x00B4_0047;

/// The string-table enum of the re-authentication strings (0x4100000E).
pub const REAUTH_TABLE_ENUM: u32 = 0x4100_000E;

/// The connection errors' table enum (`NetError::m_tableID` of `ID_ConnectionError_*`).
const NET_ERROR_TABLE: u32 = 0x4000_0005;

/// `Tokenize(name)`.
fn tokenize(name: &str) -> u32 {
    ac2_net::tokenize(name.as_bytes())
}

/// A `NetError` as the pregame screens show it (`NetError::operator StringInfo`).
fn net_error(error: GunError) -> NetError {
    NetError::Server {
        string_id: tokenize(error.string),
        table_id: error.table,
    }
}

/// `StringInfo::SetLiteralValue(text, escape)`: the literal, escaped when asked.
fn literal(text: &str, escape: bool) -> StringInfo {
    let literal = if escape {
        let wide: Vec<u16> = text.encode_utf16().collect();
        String::from_utf16_lossy(&ac2_db::escape_string(&wide))
    } else {
        text.to_owned()
    };
    StringInfo {
        string_id: 0,
        table_id: 0,
        literal: Some(literal),
        variables: Vec::new(),
    }
}

fn status_native(friend: FriendInfo) -> NativeObject {
    NativeObject::TurbineUserStatus(Box::new(friend))
}

impl Client {
    /// `gmClient::Init`'s GUN steps (`GunClient::init`) and the program's GUN: attach it before
    /// the first frame. The session must be held (`OnlineSession::connect_held`); the logon
    /// starts when `StartupGUN` is done.
    pub fn attach_gun(&mut self, gun: Rc<GunClient>) {
        gun.init();
        self.gun = Some(gun);
    }

    /// The attached GUN client.
    pub fn gun(&self) -> Option<&Rc<GunClient>> {
        self.gun.as_ref()
    }

    /// The GUN part of a client frame, before the network: `gmClient::Connect` while the
    /// session is held, `gmClient::UseTime`'s `CheckForMessages` otherwise, then the calls the
    /// handlers raised. Returns the events a failed start and the script calls made.
    pub(crate) fn gun_frame(&mut self) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        let shared = shared();
        // `ClientGunInterface::GetPlayerInterface()`, and the name `GunChatExtraInfo` reads.
        shared.set_player(self.player.is_some());
        // `AC2ClientChatManager::GetIChatClient()`: the Turbine chat client exists in the world.
        shared.set_chat_client(self.player.is_some());
        if let Some(name) = self.player_name() {
            shared.set_player_name(&name);
        }
        let Some(gun) = self.gun.clone() else {
            // No gmClient GUN half attached (a headless client): the calls the classes raised
            // are still carried out.
            return self.carry_out_gun_outputs();
        };
        if self.session.is_logon_held() && !self.gun_failed {
            // `Preferences::InqBool(0x4000000F, 0x41000001)` of the user's preferences file; no
            // default, absent leaves `m_gunPortLocal` 0.
            let port_local = self
                .prefs
                .as_ref()
                .and_then(|p| p.file().inq_bool(0x4000_000F, 0x4100_0001))
                .unwrap_or(0);
            // open: `ui_alive` is always true: closing the window ends the process elsewhere, so
            // `StartupGUN`'s `!RealClient::KeepUIAlive()` failure branches (`DoEventLoop() == 0`)
            // cannot happen in this client.
            match gun.connect(&|| port_local, true) {
                ConnectPoll::Pending => {}
                ConnectPoll::Ready(credential) => self.session.start_logon(credential),
                ConnectPoll::Failed(error) => {
                    // `Connect` returns the NetError and never reaches `Turbine::Client::
                    // Connect`: the failure goes to the connecting screen once.
                    self.gun_failed = true;
                    events.push(SessionEvent::Disconnected(net_error(error)));
                }
                ConnectPoll::InitAuthFailed => {
                    // `ID_ConnectionError_Generic` (Legions `DAT_00AF596C/70`).
                    self.gun_failed = true;
                    events.push(SessionEvent::Disconnected(NetError::Server {
                        string_id: tokenize("ID_ConnectionError_Generic"),
                        table_id: NET_ERROR_TABLE,
                    }));
                }
            }
        } else {
            gun.use_time();
        }
        events.extend(self.carry_out_gun_outputs());
        events
    }

    /// The notices a session event is: `RecvNotice_CharacterSet` and `RecvNotice_WorldName` reach
    /// the gmClient's GUN half.
    pub(crate) fn gun_session_event(&mut self, event: &SessionEvent) {
        match event {
            SessionEvent::Connected => self.logon_connected = true,
            // `OnSendToRoomByIDResult` (admin 0x00402240): the chat reply answers the context
            // `RequestGUN_SendToRoom` kept.
            SessionEvent::Notice(ac2_weenie::GameplayNotice::SendToRoomResult {
                context,
                result,
            }) => shared().answer_turbine_send(*context, *result),
            SessionEvent::Characters(set) => {
                if let Some(gun) = &self.gun {
                    gun.recv_notice_character_set(set.turbine_chat);
                }
            }
            SessionEvent::WorldName(info) => {
                if let (Some(gun), Some(name)) = (&self.gun, &info.literal) {
                    gun.recv_notice_world_name(name);
                }
            }
            _ => {}
        }
    }

    /// The player's name (`gmCWInterface::InqName`, string stat 0x100): the character entered.
    fn player_name(&self) -> Option<String> {
        let id = self.player.as_ref()?.id;
        self.session.character_name(id)
    }

    /// A `StringInfo` of the table enum `table_enum` shown through the gmClient's UI sink (virtual
    /// `+0x38` with `(string, 0, 0)`): a chat line.
    // open: which window the sink (`this+0x64`, virtual `+0x38`) is; shown as a system chat line.
    fn gun_ui_line(&mut self, name: &str, table_enum: u32) -> Vec<SessionEvent> {
        let table_id = self.session.player_script().string_tables().did(table_enum);
        vec![SessionEvent::Notice(
            ac2_weenie::GameplayNotice::DisplayStringInfo {
                kind: 0,
                text: StringInfo {
                    string_id: tokenize(name),
                    table_id,
                    literal: None,
                    variables: Vec::new(),
                },
            },
        )]
    }

    /// Carries out what the GUN classes raised, in order: each `gmCWInterface` entry point runs
    /// its IFun only when the player's script runs (`vtbl[+0x10]()` non-NULL).
    pub(crate) fn carry_out_gun_outputs(&mut self) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        for output in shared().take_outputs() {
            match output {
                GunOutput::Notice(GunNotice::ConnectionLost) => {
                    // `gmUIFlow::RecvNotice_GunConnectionLost` (Legions 0x004061B0): "server down"
                    // while the flow is on the data patch (connecting) screen, else "lost".
                    // open: the flow state word is read as "the logon has not connected yet".
                    let name = if self.logon_connected {
                        "ID_CHAR_ERROR_GUN_SERVER_LOST"
                    } else {
                        "ID_CHAR_ERROR_GUN_SERVER_DOWN"
                    };
                    events.extend(self.gun_ui_line(name, REAUTH_TABLE_ENUM));
                }
                GunOutput::Notice(_) => {}
                GunOutput::UiString { name, table } => {
                    events.extend(self.gun_ui_line(name, table));
                }
                GunOutput::UiNetError(error) => {
                    events.extend(self.gun_ui_line(error.string, error.table));
                }
                GunOutput::HandleReauthentication => {
                    // No process check in retail; the call needs the VM here.
                    let _ = self.gun_handle_reauthentication();
                }
                GunOutput::ResultSendToUser {
                    user,
                    message,
                    result,
                } => {
                    let _ = self.result_send_to_user(&user, &message, result);
                }
                GunOutput::ResultSendToRoom {
                    message,
                    room,
                    result,
                } => {
                    let _ = self.result_send_to_room(&message, room, result);
                }
                GunOutput::ResultFriendAdd { result, name } => {
                    events.extend(self.store_ifun(
                        IFUN_RESULT_FRIEND_ADD,
                        &[IFunArg::Int(result), IFunArg::Text(name)],
                    ));
                }
                GunOutput::ResultFriendDelete { result, name } => {
                    events.extend(self.store_ifun(
                        IFUN_RESULT_FRIEND_DELETE,
                        &[IFunArg::Int(result), IFunArg::Text(name)],
                    ));
                }
                GunOutput::ResultIgnoredAdd { result, name } => {
                    let _ = self.result_ignored_add(result, &name);
                }
                GunOutput::ResultIgnoredDelete { result, name } => {
                    let _ = self.result_ignored_delete(result, &name);
                }
                GunOutput::ResultCallCreate {
                    result,
                    call_type,
                    target,
                    description,
                    queue_id,
                    queue_pos,
                } => {
                    let _ = self.result_call_create(
                        result,
                        call_type,
                        &target,
                        &description,
                        queue_id,
                        queue_pos,
                    );
                }
                GunOutput::ResultGetFriendsList {
                    result,
                    user_requested,
                    list,
                } => {
                    events.extend(self.store_ifun(
                        IFUN_RESULT_GET_FRIENDS_LIST,
                        &[
                            IFunArg::Int(result),
                            IFunArg::Int(user_requested),
                            IFunArg::VmList(
                                ntype::TURBINE_USER_STATUS,
                                list.into_iter().map(status_native).collect(),
                            ),
                        ],
                    ));
                }
                GunOutput::ResultGetIgnoredList {
                    result,
                    user_requested,
                    list,
                } => {
                    let names = list
                        .into_iter()
                        .map(|n| NativeObject::WString(n.encode_utf16().collect()))
                        .collect();
                    events.extend(self.store_ifun(
                        IFUN_RESULT_GET_IGNORED_LIST,
                        &[
                            IFunArg::Int(result),
                            IFunArg::Int(user_requested),
                            IFunArg::VmList(ntype::WPSTRING, names),
                        ],
                    ));
                }
                GunOutput::EventFriendStart(friends) => {
                    self.friend_events.push(FriendEvent::Start(friends.clone()));
                    events.extend(self.store_ifun(
                        IFUN_GUN_EVENT_FRIEND_START,
                        &[IFunArg::VmList(
                            ntype::TURBINE_USER_STATUS,
                            friends.into_iter().map(status_native).collect(),
                        )],
                    ));
                }
                GunOutput::EventFriendStatus(friend) => {
                    self.friend_events.push(FriendEvent::Status(friend.clone()));
                    events.extend(self.store_ifun(
                        IFUN_GUN_EVENT_FRIEND_STATUS,
                        &[IFunArg::Vm(
                            ntype::TURBINE_USER_STATUS,
                            status_native(friend),
                        )],
                    ));
                }
                GunOutput::EventHearFromRoom {
                    speaker,
                    message,
                    room,
                    flags,
                } => {
                    let _ =
                        self.gun_hear_from_room(literal(&speaker, false), &message, room, flags);
                }
                GunOutput::HearTell {
                    teller,
                    message,
                    flags,
                } => {
                    // `CallIFun_HearTell(process, INVALID_IID, teller (escape 0), msg (escape 1), flags)`.
                    let _ =
                        self.hear_tell(0, literal(&teller, false), literal(&message, true), flags);
                }
            }
        }
        events
    }
}
