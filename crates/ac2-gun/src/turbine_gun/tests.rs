//! Tests of `TurbineGun` construction, `Init`, the connection, tickets, routing and names. The
//! expected values are the ones the retail functions give (`0c_client.md` section 3 and the
//! decompile it cites), not the output of this port.

use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

use super::*;
use crate::ct::*;
use crate::interfaces::TagBlob;
use crate::message::{GunQueueMessage, Message, MessageBody, UserStatus};
use crate::message_id as id;
use crate::test_support::{Call, TestGunHandle, TestLoader};
use crate::zt::*;
use crate::{Hresult, MessageId};

pub(super) const LCID: u32 = 0x0409;

pub(super) type Log = Arc<Mutex<Vec<String>>>;

pub(super) fn log_of(log: &Log) -> Vec<String> {
    log.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

fn push(log: &Log, line: String) {
    log.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(line);
}

/// A listener / context recording what it was called with. One type is every `CT*` class.
pub(super) struct Probe {
    pub(super) name: &'static str,
    pub(super) log: Log,
    bullet: CtResultState,
    ticket: CtTicketState,
}

impl Probe {
    pub(super) fn new(name: &'static str, log: &Log) -> Rc<Self> {
        Rc::new(Self {
            name,
            log: Arc::clone(log),
            bullet: CtResultState::new(),
            ticket: CtTicketState::new(),
        })
    }

    fn rec(&self, what: String) {
        push(&self.log, format!("{}:{what}", self.name));
    }
}

impl Bullet for Probe {
    fn bullet(&self) -> &CtBullet {
        self.bullet.bullet()
    }
}
impl CtResult for Probe {
    fn result_state(&self) -> &CtResultState {
        &self.bullet
    }
}
impl CtEvent for Probe {}
impl CtConnectEvent for Probe {
    fn handle_connect_event(&self, evt: EConnectEvent) {
        self.rec(format!("connect {}", evt as u32));
    }
}
impl CtChatEvent for Probe {
    fn invited(&self, room_name: &str, invitation: &str) {
        self.rec(format!("invited {room_name} {invitation}"));
    }
    fn room_enter(&self, room_id: u32, user_name: &str) {
        self.rec(format!("enter {room_id} {user_name}"));
    }
    fn room_leave(&self, room_id: u32, user_name: &str) {
        self.rec(format!("leave {room_id} {user_name}"));
    }
    fn hear_room(&self, room_id: u32, user: &str, speech: &str, extra: Option<&TagBlob>) {
        self.rec(format!(
            "hear_room {room_id} {user} {speech} {:?}",
            extra.map(|b| b.bytes().to_vec())
        ));
    }
    fn kicked_out(&self, room_id: u32, reason: &str) {
        self.rec(format!("kicked {room_id} {reason}"));
    }
    fn receive_ownership(&self, room_id: u32) {
        self.rec(format!("owner {room_id}"));
    }
    fn hear_user(&self, user: &str, speech: &str, extra: Option<&TagBlob>) {
        self.rec(format!(
            "hear_user {user} {speech} {:?}",
            extra.map(|b| b.bytes().to_vec())
        ));
    }
}
impl CtFriendsEvent for Probe {
    fn friend_start(&self, statuses: &[UserStatus]) {
        let names: Vec<_> = statuses.iter().map(|s| s.user_name.as_str()).collect();
        self.rec(format!("start {names:?}"));
    }
    fn friend_status(&self, friend_name: &str, mode: u32) {
        self.rec(format!("status {friend_name} {mode}"));
    }
}
macro_rules! probe_result {
    ($tr:ident, $m:ident, $label:literal) => {
        impl $tr for Probe {
            fn $m(&self) {
                self.rec($label.to_owned());
            }
        }
    };
}
probe_result!(CtFriendAddResult, handle_friend_add_result, "friend_add");
probe_result!(
    CtFriendDeleteResult,
    handle_friend_delete_result,
    "friend_delete"
);
probe_result!(CtIgnoredAddResult, handle_ignored_add_result, "ignored_add");
probe_result!(
    CtIgnoredDeleteResult,
    handle_ignored_delete_result,
    "ignored_delete"
);
probe_result!(CtLocationResult, handle_location_result, "location");
probe_result!(
    CtSendToRoomResult,
    handle_send_to_room_result,
    "send_to_room"
);
probe_result!(
    CtSendToUserResult,
    handle_send_to_user_result,
    "send_to_user"
);
probe_result!(CtLeaveRoomResult, handle_leave_room_result, "leave_room");
probe_result!(
    CtLeaveAllRoomsResult,
    handle_leave_all_rooms_result,
    "leave_all"
);
probe_result!(
    CtDeletePersistentChatChannelResult,
    handle_delete_persistent_chat_channel_result,
    "delete_channel"
);
impl CtGetFriendsListResult for Probe {
    fn handle_get_friends_list_result(&self, statuses: &[UserStatus]) {
        self.rec(format!("friends_list {}", statuses.len()));
    }
}
impl CtGetIgnoredListResult for Probe {
    fn handle_get_ignored_list_result(&self, names: &[String]) {
        self.rec(format!("ignored_list {names:?}"));
    }
}
impl CtFlagsResult for Probe {
    fn handle_flags_result(&self, which_flag: u32, flags: u32) {
        self.rec(format!("flags {which_flag} {flags}"));
    }
}
impl CtCallCreateResult for Probe {
    fn handle_call_create_result(&self, queue_id: u32, queue_pos: u32) {
        self.rec(format!("call_create {queue_id} {queue_pos}"));
    }
}
impl CtJoinRoomResult for Probe {
    fn handle_join_room_result(&self, room_id: u32, server_id: u32) {
        self.rec(format!("join {room_id} {server_id}"));
    }
}
impl CtCreateRoomResult for Probe {
    fn handle_create_room_result(&self, room_id: u32, server_id: u32) {
        self.rec(format!("create {room_id} {server_id}"));
    }
}
impl CtCreatePersistentChatChannelResult for Probe {
    fn handle_create_persistent_chat_channel_result(&self, value: u32) {
        self.rec(format!(
            "create_channel {value} hr={:#x}",
            self.h_result().0
        ));
    }
}
impl CtPersonaResult for Probe {
    fn handle_enter_persona_result(&self, kind: u32, server_id: u32) {
        self.rec(format!("enter_persona {kind} {server_id}"));
    }
    fn handle_leave_persona_result(&self) {
        self.rec("leave_persona".to_owned());
    }
    fn handle_create_persona_result(&self) {
        self.rec("create_persona".to_owned());
    }
    fn handle_delete_persona_result(&self) {
        self.rec("delete_persona".to_owned());
    }
    fn handle_get_persona_list_result(&self, names: &[String]) {
        self.rec(format!("persona_list {names:?}"));
    }
}
impl CtTicketResult for Probe {
    fn ticket_state(&self) -> &CtTicketState {
        &self.ticket
    }
    fn handle_ticket_arrived(&self, result: ETicketResult) {
        self.rec(format!("arrived {:#x}", result as u32));
    }
    fn handle_authentication_result(&self, result: ETicketResult) {
        self.rec(format!("auth {:#x}", result as u32));
    }
    fn handle_release_ticket_result(&self, result: ETicketResult) {
        self.rec(format!("release {:#x}", result as u32));
    }
}

/// A `TurbineGun` with a recording `CGun` double, already `Init`ed, plus the lock that keeps the
/// other tests off `S_pTurbineGun`.
pub(super) struct Rig {
    pub(super) gun: Rc<TurbineGun>,
    pub(super) handle: TestGunHandle,
    pub(super) log: Log,
}

impl Rig {
    /// Unconnected: `Init` done, `RequestConnect` not.
    pub(super) fn new() -> Rig {
        TurbineGun::set_strip_world_name_from_room_chat_events(true);
        let gun = Rc::new(TurbineGun::new(LCID));
        let handle = TestGunHandle::new();
        let mut loader = TestLoader::new(handle.gun());
        assert!(gun.init("", &mut loader));
        Rig {
            gun,
            handle,
            log: Log::default(),
        }
    }

    /// `ConnectEv` delivered: `m_fConnected` set.
    pub(super) fn connected() -> Rig {
        let rig = Rig::new();
        rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
        rig.handle.clear_calls();
        rig
    }

    pub(super) fn probe(&self, name: &'static str) -> Rc<Probe> {
        Probe::new(name, &self.log)
    }

    pub(super) fn log(&self) -> Vec<String> {
        log_of(&self.log)
    }

    /// A world name so `Birch` becomes `Birch@Legions`.
    pub(super) fn with_world(self) -> Rig {
        assert!(self.gun.set_world_name("Legions"));
        self
    }
}

pub(super) fn event(id: MessageId) -> Message {
    Message {
        inited: true,
        id,
        result: None,
        body: MessageBody::None,
    }
}

pub(super) fn event_with(id: MessageId, body: MessageBody) -> Message {
    Message {
        inited: true,
        id,
        result: None,
        body,
    }
}

pub(super) fn queued(message: Message) -> GunQueueMessage {
    GunQueueMessage {
        flags: 4,
        gmid: 1,
        param: 0,
        cb: 0,
        message,
    }
}

#[test]
fn construction_is_all_zero_and_makes_the_gun_current() {
    // admin 0x00606250 / Legions 0x005C70F0
    let rig = Rig::new();
    assert!(!rig.gun.is_connected());
    assert!(!rig.gun.is_connecting());
    assert!(!rig.gun.is_logged_in());
    assert!(!rig.gun.is_persistent_connection());
    assert_eq!(rig.gun.port(), 0);
    assert_eq!(rig.gun.gun_server_name(), "");
    assert_eq!(rig.gun.world_name(), "");
    assert_eq!(rig.gun.world_suffix(), "");
    assert!(TurbineGun::strip_world_name_from_room_chat_events());
    let current = crate::turbine_gun_utils::s_p_turbine_gun().expect("S_pTurbineGun set");
    assert!(Arc::ptr_eq(&current, &rig.gun.world_names()));
}

#[test]
fn init_loads_gundll_by_default_and_asks_for_the_cgun_class_object() {
    // Init: an empty path is "GunDll.dll"; GunCreateInstance(hmod, CLSID {76633332-...},
    // IID_IGunConnection {eeeeee21-...}) goes through DllGetClassObject(clsid, IClassFactory).
    let gun = TurbineGun::new(LCID);
    let handle = TestGunHandle::new();
    let mut loader = TestLoader::new(handle.gun());
    assert!(gun.init("", &mut loader));
    assert_eq!(loader.loaded, ["GunDll.dll"]);
    assert!(gun.has_gun());
    let requests = loader.requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].0,
        crate::interfaces::Guid {
            data1: 0x76633332,
            data2: 0x52a2,
            data3: 0x4460,
            data4: [0xb0, 0x89, 0x35, 0x41, 0xa8, 0x44, 0x3c, 0x9a]
        }
    );
    assert_eq!(requests[0].1.data1, 1);
    assert_eq!(requests[0].1.data4, [0xC0, 0, 0, 0, 0, 0, 0, 0x46]);
}

#[test]
fn init_keeps_an_explicit_path_and_fails_when_the_dll_does_not_load_or_export() {
    let gun = TurbineGun::new(LCID);
    let mut loader = TestLoader::new(TestGunHandle::new().gun());
    assert!(gun.init("alt\\gun.dll", &mut loader));
    assert_eq!(loader.loaded, ["alt\\gun.dll"]);

    let gun = TurbineGun::new(LCID);
    let mut loader = TestLoader::new(TestGunHandle::new().gun());
    loader.fail_load = Some(126);
    assert!(!gun.init("", &mut loader));
    assert!(!gun.has_gun());

    // DllGetClassObject missing: GunCreateInstance returns GetLastError() (126), a positive
    // number that Init takes for success; there is no object (retail crashes at the next QI).
    let gun = TurbineGun::new(LCID);
    let mut loader = TestLoader::new(TestGunHandle::new().gun());
    loader.export_missing = Some(127);
    assert!(!gun.init("", &mut loader));
    assert!(!gun.has_gun());
}

#[test]
fn request_connect_passes_retails_arguments_in_retails_order() {
    // RequestConnect (admin 0x00605AC0 / Legions 0x005C69A0)
    let rig = Rig::new();
    assert!(rig
        .gun
        .request_connect("gun.example", 2310, 7000, "1.0", "XXXX-YYYY-ZZZZ", 0));
    let calls = rig.handle.calls();
    assert_eq!(calls.len(), 2);
    let Call::Init {
        app,
        game_version,
        locale,
        cd_key,
        port_local,
    } = &calls[0]
    else {
        panic!("first call is Init: {calls:?}");
    };
    assert_eq!(app.data1, 0x2280767d);
    assert_eq!((app.data2, app.data3), (0x1689, 0x4d6b));
    assert_eq!(app.data4, [0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f, 0xb6, 0x55]);
    assert_eq!(game_version, "1.0");
    assert_eq!(*locale, LCID); // locale 0 -> GetUserDefaultLCID()
    assert_eq!(cd_key, "XXXX-YYYY-ZZZZ");
    assert_eq!(*port_local, 7000);
    assert_eq!(
        calls[1],
        Call::Connect {
            server: "gun.example".into(),
            port: 2310
        }
    );
    assert!(rig.gun.is_connecting());
    assert!(!rig.gun.is_connected());
    assert_eq!(rig.gun.gun_server_name(), "gun.example");
    assert_eq!(rig.gun.port(), 2310);
}

#[test]
fn request_connect_keeps_a_given_locale_and_clears_persistence_first() {
    let rig = Rig::new();
    rig.gun.persistent_connection.set(true);
    // no connection object: persistence is still cleared before the NULL check
    let bare = TurbineGun::new(LCID);
    bare.persistent_connection.set(true);
    assert!(!bare.request_connect("s", 1, 2, "v", "k", 5));
    assert!(!bare.is_persistent_connection());
    assert!(rig.gun.request_connect("s", 1, 2, "v", "k", 0x0407));
    assert!(!rig.gun.is_persistent_connection());
    let Call::Init { locale, .. } = &rig.handle.calls()[0] else {
        panic!()
    };
    assert_eq!(*locale, 0x0407);
}

#[test]
fn request_connect_failures() {
    // Init failing (hr < 0) stops before Connect and leaves m_fConnecting alone.
    let rig = Rig::new();
    rig.handle.script("init", ZT_E_INTERNAL);
    assert!(!rig.gun.request_connect("s", 1, 2, "v", "k", 0));
    assert_eq!(rig.handle.calls().len(), 1);
    assert!(!rig.gun.is_connecting());
    assert_eq!(rig.gun.gun_server_name(), "");

    // Connect failing: m_fConnecting stays 1 and the name and port are stored.
    let rig = Rig::new();
    rig.handle.script("connect", ZT_E_FAILED_TO_CONNECT);
    assert!(!rig.gun.request_connect("s", 99, 2, "v", "k", 0));
    assert!(rig.gun.is_connecting());
    assert_eq!(
        (rig.gun.gun_server_name().as_str(), rig.gun.port()),
        ("s", 99)
    );

    // Connect answering S_OK (not E_PENDING) is not the success value either.
    let rig = Rig::new();
    rig.handle.script("connect", Hresult::S_OK);
    assert!(!rig.gun.request_connect("s", 1, 2, "v", "k", 0));
}

#[test]
fn connect_event_codes_and_the_persistent_flag() {
    // RcvConnectEvent (admin 0x00605CA0 / Legions 0x005C6B80)
    let rig = Rig::new();
    let listener = rig.probe("c");
    assert!(rig.gun.register_for_connect_events(Some(listener.clone())));
    assert_eq!(
        rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT)),
        Hresult::S_OK
    );
    assert!(rig.gun.is_connected());
    assert!(rig.gun.is_persistent_connection());
    assert!(!rig.gun.is_connecting());
    // a second ConnectEv with the flag still set is RECONNECTED (2)
    rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
    // an id the family does not know is UNDEF (0), with no state change
    rig.gun.rcv_connect_event(&event(MessageId(0x1303)));
    assert_eq!(rig.log(), ["c:connect 1", "c:connect 2", "c:connect 0"]);
    assert_eq!(listener.num_events_handled(), 3);
    assert!(rig.gun.is_connected());
}

#[test]
fn a_connect_after_request_connect_is_connect_not_reconnected() {
    let rig = Rig::new();
    let listener = rig.probe("c");
    rig.gun.register_for_connect_events(Some(listener));
    rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
    rig.gun.request_connect("s", 1, 2, "v", "k", 0);
    rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
    assert_eq!(rig.log(), ["c:connect 1", "c:connect 1"]);
}

#[test]
fn disconnect_reconnects_with_the_stored_server_and_never_gives_up() {
    // The reconnect rule: no delay, retry counter or limit; Init is not called again.
    let rig = Rig::new();
    let listener = rig.probe("c");
    rig.gun.register_for_connect_events(Some(listener));
    rig.gun
        .request_connect("gun.example", 2310, 0, "1.0", "k", 0);
    rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
    rig.handle.clear_calls();
    rig.handle.script("connect", ZT_E_FAILED_TO_CONNECT);
    for _ in 0..3 {
        assert_eq!(
            rig.gun.rcv_connect_event(&event(id::DISCONNECT_EVENT)),
            Hresult::S_OK
        );
        assert!(!rig.gun.is_connected());
        assert!(!rig.gun.is_logged_in());
        assert!(rig.gun.is_persistent_connection());
    }
    rig.gun.rcv_connect_event(&event(id::CONNECT_FAILED_EVENT));
    let connect = Call::Connect {
        server: "gun.example".into(),
        port: 2310,
    };
    assert_eq!(
        rig.handle.calls(),
        [connect.clone(), connect.clone(), connect.clone(), connect]
    );
    assert_eq!(
        rig.log(),
        [
            "c:connect 1",
            "c:connect 3",
            "c:connect 3",
            "c:connect 3",
            "c:connect 4"
        ]
    );
}

#[test]
fn no_reconnect_without_a_previous_connect() {
    let rig = Rig::new();
    rig.gun.request_connect("s", 1, 2, "v", "k", 0);
    rig.handle.clear_calls();
    // ConnectFailed before any ConnectEv: m_fPersistentConnection is 0
    rig.gun.rcv_connect_event(&event(id::CONNECT_FAILED_EVENT));
    assert!(rig.handle.calls().is_empty());
    assert!(rig.gun.is_connecting()); // not reset by 0x1301/0x1302
}

#[test]
fn listeners_are_notified_newest_first() {
    // List::insert (Legions 0x005F8DD0) puts the node before the head.
    let rig = Rig::new();
    for name in ["first", "second", "third"] {
        let l = rig.probe(name);
        assert!(rig.gun.register_for_connect_events(Some(l)));
    }
    assert!(!rig.gun.register_for_connect_events(None));
    assert!(!rig.gun.register_for_chat_events(None));
    assert!(!rig.gun.register_for_friends_events(None));
    rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
    assert_eq!(
        rig.log(),
        ["third:connect 1", "second:connect 1", "first:connect 1"]
    );
}

#[test]
fn check_for_messages_drains_the_queue_through_read() {
    // CheckForMessages (admin 0x006049F0): DispatchQueue(this); hr != 0 -> 0.
    let rig = Rig::new();
    let listener = rig.probe("c");
    rig.gun.register_for_connect_events(Some(listener));
    rig.gun.request_connect("s", 1, 2, "v", "k", 0);
    rig.handle.queue_message(queued(event(id::CONNECT_EVENT)));
    rig.handle
        .queue_message(queued(event(id::DISCONNECT_EVENT)));
    rig.handle.clear_calls();
    assert!(rig.gun.check_for_messages());
    assert!(!rig.gun.is_connected());
    assert_eq!(rig.log(), ["c:connect 1", "c:connect 3"]);
    // the reconnect for the disconnect is made once DispatchQueue returns
    assert_eq!(
        rig.handle.calls(),
        [
            Call::DispatchQueue,
            Call::Connect {
                server: "s".into(),
                port: 1
            }
        ]
    );
    assert!(rig.handle.state().queue.is_empty());
}

#[test]
fn check_for_messages_without_a_connection_object_is_true_and_a_failing_dispatch_is_false() {
    let bare = TurbineGun::new(LCID);
    assert!(bare.check_for_messages());
    // CGun::DispatchQueue not initialised -> ZT_E_NOT_INITED -> PrintCmdHRESULT, return 0
    let rig = Rig::new();
    assert!(!rig.gun.check_for_messages());
    rig.handle.set_inited(true);
    assert!(rig.gun.check_for_messages());
}

#[test]
fn a_negative_read_result_stops_the_dispatch() {
    // GunDll: a sink result below zero stops the dispatch and DispatchQueue returns 1.
    let rig = Rig::new();
    rig.handle.set_inited(true);
    let listener = rig.probe("c");
    rig.gun.register_for_connect_events(Some(listener));
    // an id with the request nibble: Read answers ZT_E_INTERNAL (negative)
    rig.handle.queue_message(queued(event(MessageId(0x0f101))));
    rig.handle.queue_message(queued(event(id::CONNECT_EVENT)));
    assert!(!rig.gun.check_for_messages());
    assert!(rig.log().is_empty());
    assert_eq!(rig.handle.state().queue.len(), 1);
    assert!(rig.gun.check_for_messages());
    assert_eq!(rig.log(), ["c:connect 1"]);
}

#[test]
fn read_routes_on_the_kind_nibble_and_rcv_event_on_the_family() {
    // Read (admin 0x00604A80): 0x200 -> RcvResult, 0x300 -> RcvEvent, else ZT_E_INTERNAL.
    let rig = Rig::new();
    assert_eq!(
        rig.gun.read_message(&queued(event(MessageId(0x0f101)))),
        ZT_E_INTERNAL
    );
    assert_eq!(
        rig.gun.read_message(&queued(event(MessageId(0x1000)))),
        ZT_E_INTERNAL
    );
    assert_eq!(
        rig.gun.read_message(&queued(event(id::CONNECT_EVENT))),
        Hresult::S_OK
    );
    // a result message with no handler in family 0x1000 (InitRes 0x1204): ZT_E_INTERNAL
    assert_eq!(
        rig.gun.read_message(&queued(event(id::INIT_RESULT))),
        ZT_E_INTERNAL
    );
    // RcvEvent (admin 0x00604AD0): families 0x1000 0x2000 0x4000 0x5000 0x8000
    assert_eq!(rig.gun.rcv_event(&event(MessageId(0x3301))), ZT_E_INTERNAL);
    assert_eq!(rig.gun.rcv_event(&event(id::LOGOUT_EVENT)), Hresult::S_OK);
    assert_eq!(rig.gun.rcv_event(&event(MessageId(0x2300))), ZT_E_INTERNAL);
    assert_eq!(
        rig.gun.rcv_event(&event(id::ENTER_PERSONA_EVENT)),
        Hresult::S_OK
    );
    assert_eq!(
        rig.gun.rcv_event(&event(id::LEAVE_PERSONA_EVENT)),
        Hresult::S_OK
    );
    assert_eq!(rig.gun.rcv_event(&event(MessageId(0x8302))), ZT_E_INTERNAL);
    assert_eq!(rig.gun.rcv_event(&event(id::INVITED_EVENT)), Hresult::S_OK);
    assert_eq!(rig.gun.rcv_event(&event(id::USER_SAY_EVENT)), Hresult::S_OK);
}

#[test]
fn world_name_is_set_once_and_makes_the_suffix() {
    // SetWorldName (admin 0x004033A0 / Legions 0x00403310)
    let rig = Rig::new();
    assert!(rig.gun.set_world_name("Legions"));
    assert_eq!(rig.gun.world_name(), "Legions");
    assert_eq!(rig.gun.world_suffix(), "@Legions");
    assert!(!rig.gun.set_world_name("Other"));
    assert_eq!(rig.gun.world_suffix(), "@Legions");
    // an empty name leaves it unset (m_len == 1), so it can still be set
    let empty = Rig::new();
    assert!(empty.gun.set_world_name(""));
    assert!(empty.gun.set_world_name("Later"));
    assert_eq!(empty.gun.world_suffix(), "@Later");
}

#[test]
fn local_to_global_appends_the_suffix_only_to_a_name_without_an_at() {
    // LocalToGlobalName (admin 0x006064B0 / Legions 0x005C7350)
    let rig = Rig::new().with_world();
    assert_eq!(rig.gun.local_to_global_name("Birch"), "Birch@Legions");
    assert_eq!(rig.gun.local_to_global_name("Birch@Other"), "Birch@Other");
    assert_eq!(rig.gun.local_to_global_name("@x"), "@x");
    assert_eq!(rig.gun.local_to_global_name("a@"), "a@");
    assert_eq!(rig.gun.local_to_global_name("birch"), "birch@Legions");
    // no suffix yet (the world name notice has not arrived): unchanged
    let bare = Rig::new();
    assert_eq!(bare.gun.local_to_global_name("Birch"), "Birch");
}

#[test]
fn global_to_local_strips_only_an_exact_suffix_at_the_first_at() {
    // GlobalToLocalName (admin 0x00606590 / Legions 0x005C7430)
    let rig = Rig::new().with_world();
    assert_eq!(rig.gun.global_to_local_name("Birch@Legions"), "Birch");
    // case-sensitive wcscmp
    assert_eq!(
        rig.gun.global_to_local_name("Birch@legions"),
        "Birch@legions"
    );
    assert_eq!(rig.gun.global_to_local_name("Birch@Other"), "Birch@Other");
    assert_eq!(rig.gun.global_to_local_name("Birch"), "Birch");
    // only the FIRST '@' matters: right = "@b@Legions" != "@Legions"
    assert_eq!(rig.gun.global_to_local_name("a@b@Legions"), "a@b@Legions");
    // first '@' at index 0: left clamps to the whole string
    assert_eq!(rig.gun.global_to_local_name("@Legions"), "@Legions");
    assert_eq!(rig.gun.global_to_local_name("@Other"), "@Other");
    // a trailing '@' is the one-character right half "@"
    assert_eq!(rig.gun.global_to_local_name("Birch@"), "Birch@");
    // before the world name is known (suffix is the null string) nothing is stripped
    let bare = Rig::new();
    assert_eq!(
        bare.gun.global_to_local_name("Birch@Legions"),
        "Birch@Legions"
    );
    assert_eq!(bare.gun.global_to_local_name("Birch@"), "Birch@");
}

#[test]
fn set_authentication_ticket_hands_over_the_string_with_its_nul() {
    // SetAuthenticationTicket (admin 0x00605BA0): cbSize = strlen + 1, the NUL included.
    let rig = Rig::new();
    assert!(rig.gun.set_authentication_ticket("abc"));
    assert_eq!(rig.handle.calls(), [Call::SetTicket(b"abc\0".to_vec())]);
    rig.handle.clear_calls();
    assert!(rig.gun.set_authentication_ticket(""));
    assert_eq!(rig.handle.calls(), [Call::SetTicket(vec![0])]);
    // hr != 0 -> 0
    rig.handle.script("set_ticket", ZT_E_INTERNAL);
    assert!(!rig.gun.set_authentication_ticket("abc"));
    // no m_pGunTicket -> 0 without a call
    let bare = TurbineGun::new(LCID);
    assert!(!bare.set_authentication_ticket("abc"));
}

#[test]
fn get_authentication_ticket_calls_get_ticket_twice_on_buffer_too_small() {
    // GetAuthenticationTicket (admin 0x00604A20 / Legions 0x005C5900)
    let rig = Rig::connected();
    rig.handle.set_ticket(Some(vec![1, 2, 3, 4, 5]));
    let mut blob = TagBlob::default();
    assert!(rig.gun.get_authentication_ticket(Some(&mut blob)));
    assert_eq!(rig.handle.calls(), [Call::GetTicket, Call::GetTicket]);
    assert_eq!(blob.cb_size, 5);
    assert_eq!(blob.bytes(), [1, 2, 3, 4, 5]);
}

#[test]
fn get_authentication_ticket_guards_and_failures() {
    let rig = Rig::connected();
    // no ticket: GetTicket fails with NOT_TICKETED, a single call
    let mut blob = TagBlob::default();
    assert!(!rig.gun.get_authentication_ticket(Some(&mut blob)));
    assert_eq!(rig.handle.calls(), [Call::GetTicket]);
    rig.handle.clear_calls();
    assert!(!rig.gun.get_authentication_ticket(None));
    assert!(rig.handle.calls().is_empty());
    // not connected
    let idle = Rig::new();
    idle.handle.set_ticket(Some(vec![9]));
    let mut blob = TagBlob::default();
    assert!(!idle.gun.get_authentication_ticket(Some(&mut blob)));
    assert!(idle.handle.calls().is_empty());
}

#[test]
fn present_and_finish_with_ticket_addref_the_context_and_return_hr_is_zero() {
    // RequestPresentAuthenticationTicket (admin 0x00605640 / Legions 0x005C6520),
    // RequestFinishWithTicket (admin 0x006056A0 / Legions 0x005C6580)
    let rig = Rig::connected();
    let ctx = rig.probe("t");
    // GetTicketBlob() == NULL: refused, no AddRef
    assert!(!rig
        .gun
        .request_present_authentication_ticket(Some(ctx.clone())));
    assert_eq!(rig.gun.pending_context_count(), 0);
    ctx.adopt_ticket(TagBlob {
        cb_size: 3,
        data: Some(vec![7, 8, 9]),
    });
    assert!(!rig.gun.request_present_authentication_ticket(None));
    assert!(rig
        .gun
        .request_present_authentication_ticket(Some(ctx.clone())));
    let calls = rig.handle.calls();
    let Call::PresentTicket { ticket, context } = &calls[0] else {
        panic!("{calls:?}")
    };
    assert_eq!(ticket, &[7, 8, 9]);
    assert_ne!(context.0, 0);
    assert_eq!(rig.gun.pending_context_count(), 1);
    // a failing hr is false and leaves the AddRef undone
    rig.handle.script("present_ticket", ZT_E_INTERNAL);
    assert!(!rig
        .gun
        .request_present_authentication_ticket(Some(ctx.clone())));
    assert_eq!(rig.gun.pending_context_count(), 2);
    // finish: no blob needed
    let fresh = rig.probe("f");
    assert!(rig.gun.request_finish_with_ticket(Some(fresh)));
    assert!(matches!(
        rig.handle.calls().last(),
        Some(Call::ReleaseTicket(_))
    ));
    assert!(!rig.gun.request_finish_with_ticket(None));
    let idle = Rig::new();
    assert!(!idle.gun.request_finish_with_ticket(Some(idle.probe("x"))));
}

#[test]
fn drop_disconnects_a_connected_gun_and_clears_persistence() {
    // ~TurbineGun (admin 0x00606330 / Legions 0x005C71D0)
    let rig = Rig::connected();
    let handle = rig.handle.clone();
    let Rig { gun, .. } = rig;
    drop(gun);
    assert_eq!(handle.calls(), [Call::Disconnect]);
    // never connected: no Disconnect
    let rig = Rig::new();
    let handle = rig.handle.clone();
    let Rig { gun, .. } = rig;
    drop(gun);
    assert!(handle.calls().is_empty());
}

#[test]
fn convert_to_weenie_result_table() {
    // CTBullet::ConvertToWeenieResult (admin 0x00735C60 / Legions 0x006B1900)
    for (hr, expected) in [
        (0u32, 0u32),
        (0x4112001B, 0xC744),
        (0x4112001C, 0xC74D),
        (0x8000000A, 0xC73C),
        (0x80070005, 0xC74F),
        (0xC1120005, 0xC748),
        (0xC112000C, 200),
        (0xC112000F, 0xC743),
        (0xC1120010, 0xC74E),
        (0xC1120011, 0xC73B),
        (0xC1120012, 0xC73D),
        (0xC1120014, 0xC744),
        (0xC1120018, 0xC73F),
        (0xC1121003, 0xC746),
        (0xC1121009, 0xC740),
        (0xC1121401, 0xC73E),
        (0xC1121800, 0xC742),
        (0xC1121802, 0xC747),
        (0xC1121803, 0xC745),
        (0xC1121C04, 0xC749),
        (0xC1122000, 0xC74B),
        (0xC1122002, 0xC74A),
        (0xC1122003, 0xC74C),
        (0xC1120003, 0xC741),
        (0x12345678, 0xC741),
    ] {
        assert_eq!(convert_to_weenie_result(Hresult(hr)), expected, "{hr:#x}");
    }
}

/// A connect listener that checks the order of the GunDll calls around it.
struct OrderProbe {
    bullet: CtBullet,
    gun: Rc<TurbineGun>,
    handle: TestGunHandle,
    log: Log,
    ticket_ctx: Rc<Probe>,
}

impl Bullet for OrderProbe {
    fn bullet(&self) -> &CtBullet {
        &self.bullet
    }
}
impl CtEvent for OrderProbe {}
impl CtConnectEvent for OrderProbe {
    fn handle_connect_event(&self, evt: EConnectEvent) {
        let calls = self.handle.calls();
        let connects = calls
            .iter()
            .filter(|c| matches!(c, Call::Connect { .. }))
            .count();
        let presents = calls
            .iter()
            .filter(|c| matches!(c, Call::PresentTicket { .. }))
            .count();
        let line = match evt {
            EConnectEvent::Reconnected => {
                // retail: RecvNotice_GunReconnected -> AuthenticateWithGUN(true) ->
                // RequestPresentAuthenticationTicket, run inside Read inside DispatchQueue
                let ok = self
                    .gun
                    .request_present_authentication_ticket(Some(self.ticket_ctx.clone()));
                format!("reconnected present={ok}")
            }
            other => format!("{} connects={connects} presents={presents}", other as u32),
        };
        push(&self.log, line);
    }
}

fn order_probe(rig: &Rig) -> Rc<OrderProbe> {
    let ticket_ctx = rig.probe("t");
    ticket_ctx.adopt_ticket(TagBlob {
        cb_size: 1,
        data: Some(vec![1]),
    });
    Rc::new(OrderProbe {
        bullet: CtBullet::new(),
        gun: Rc::clone(&rig.gun),
        handle: rig.handle.clone(),
        log: Arc::clone(&rig.log),
        ticket_ctx,
    })
}

#[test]
fn a_connect_made_from_read_is_recorded_before_the_next_queued_message() {
    // Retail re-enters CGun from inside DispatchQueue: RcvConnectEvent(0x1301) calls
    // IGunConnection::Connect before DispatchQueue reads the next message.
    let rig = Rig::connected();
    rig.handle.set_inited(true);
    rig.gun.request_connect("s", 1, 2, "v", "k", 0);
    rig.gun.rcv_connect_event(&event(id::CONNECT_EVENT));
    rig.handle.clear_calls();
    assert!(rig.gun.register_for_connect_events(Some(order_probe(&rig))));
    rig.handle
        .queue_message(queued(event(id::DISCONNECT_EVENT)));
    rig.handle.queue_message(queued(event(MessageId(0x1303))));
    assert!(rig.gun.check_for_messages());
    // the disconnect is notified before its reconnect; the next message (UNDEF, code 0) sees it
    assert_eq!(
        rig.log(),
        ["3 connects=0 presents=0", "0 connects=1 presents=0"]
    );
    let calls = rig.handle.calls();
    assert!(matches!(calls[0], Call::DispatchQueue));
    assert!(matches!(calls[1], Call::Connect { .. }));
}

#[test]
fn a_listener_may_call_back_into_the_turbine_gun_during_dispatch() {
    // RECONNECTED: the listener calls RequestPresentAuthenticationTicket on the same TurbineGun
    // (and so IGunTicket::PresentTicket on the same CGun) inside Read inside DispatchQueue; the
    // PresentTicket is recorded before the next queued message is processed.
    let rig = Rig::connected();
    rig.handle.set_inited(true);
    assert!(rig.gun.register_for_connect_events(Some(order_probe(&rig))));
    rig.handle.queue_message(queued(event(id::CONNECT_EVENT))); // persistent -> RECONNECTED
    rig.handle.queue_message(queued(event(MessageId(0x1303))));
    assert!(rig.gun.check_for_messages());
    assert_eq!(
        rig.log(),
        ["reconnected present=true", "0 connects=0 presents=1"]
    );
    assert_eq!(rig.gun.pending_context_count(), 1);
}
