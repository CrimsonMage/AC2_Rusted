//! A recording [`GunObject`] double (`test-support` feature): it takes GunDll's place under
//! [`crate::TurbineGun`], records every interface call, answers with scripted `HRESULT`s, and
//! runs `CGun::DispatchQueue`'s loop over a queue the test fills. It is a test double, not
//! GunDll: only `DispatchQueue`'s loop is taken from the decompile.
//!
//! `CGun::DispatchQueue` (GunDll 0x1000fd90, `decompiled-pdb/gundll/spec/0a_receive_messages.md`
//! section 1.4 and its adjudication): not initialised -> `ZT_E_NOT_INITED`; else the sink is
//! called for each queued message until the queue is empty (`E_PENDING` -> `S_OK`); a sink result
//! below zero stops the dispatch and `DispatchQueue` returns 1. A queued message with `flags`
//! bit 0 goes to the FRONT, otherwise to the tail (`FUN_10039c00`).

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::interfaces::*;
use crate::message::GunQueueMessage;
use crate::turbine_gun_utils::{ClassFactory, GunLoader, GunModule};
use crate::zt::{ZT_E_BUFFER_TOO_SMALL, ZT_E_NOT_INITED, ZT_E_NOT_TICKETED};
use crate::Hresult;

/// One recorded interface call (`Other` for the methods `TurbineGun` never calls).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Init {
        app: Guid,
        game_version: String,
        locale: u32,
        cd_key: String,
        port_local: u16,
    },
    Connect {
        server: String,
        port: u16,
    },
    Disconnect,
    DispatchQueue,
    GetTicket,
    SetTicket(Vec<u8>),
    PresentTicket {
        ticket: Vec<u8>,
        context: Context,
    },
    ReleaseTicket(Context),
    GetFriendsList(Context),
    GetIgnoredList(Context),
    FriendAdd(String, Context),
    FriendDelete(String, Context),
    IgnoredAdd(String, Context),
    IgnoredDelete(String, Context),
    Flags {
        op: FlagOperation,
        which: FlagWord,
        bits: u32,
        context: Context,
    },
    SetLocation {
        mode: u32,
        location: String,
        context: Context,
    },
    SendToRoomEx {
        room_id: u32,
        message: String,
        blob: Vec<u8>,
        context: Context,
    },
    SendToUserEx {
        user: String,
        message: String,
        blob: Vec<u8>,
        context: Context,
    },
    CallCreate {
        call_type: SupportCallType,
        target: Option<String>,
        description: Option<String>,
        blob: Vec<u8>,
        locale: u32,
        context: Context,
    },
    Other(&'static str),
}

/// What the double keeps.
#[derive(Debug, Default)]
pub struct TestGunState {
    /// Every call, in order.
    pub calls: Vec<Call>,
    /// The `GunDll` queue `DispatchQueue` drains.
    pub queue: VecDeque<GunQueueMessage>,
    /// `CGun`'s initialised state (`this+0x38`): set by `Init`.
    pub inited: bool,
    /// The ticket `GetTicket` returns.
    pub ticket: Option<Vec<u8>>,
    /// Scripted results by method name (`"connect"`, `"init"`, ...), consumed in order; then
    /// the default (`E_PENDING` for `connect`, `S_OK` otherwise).
    pub results: HashMap<&'static str, VecDeque<Hresult>>,
}

/// The test's view of the state the [`TestGun`] it gave away keeps updating.
#[derive(Debug, Clone, Default)]
pub struct TestGunHandle(Arc<Mutex<TestGunState>>);

impl TestGunHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> MutexGuard<'_, TestGunState> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The recorded calls.
    pub fn calls(&self) -> Vec<Call> {
        self.state().calls.clone()
    }

    /// Forgets the recorded calls.
    pub fn clear_calls(&self) {
        self.state().calls.clear();
    }

    /// Queues a message (`CGun`'s `Write`: flags bit 0 goes to the front).
    pub fn queue_message(&self, message: GunQueueMessage) {
        let mut state = self.state();
        if message.flags & 1 != 0 {
            state.queue.push_front(message);
        } else {
            state.queue.push_back(message);
        }
    }

    /// Scripts the next result of `method` (`"connect"`, `"init"`, `"friend_add"`, ...).
    pub fn script(&self, method: &'static str, hr: Hresult) {
        self.state()
            .results
            .entry(method)
            .or_default()
            .push_back(hr);
    }

    pub fn set_inited(&self, inited: bool) {
        self.state().inited = inited;
    }

    pub fn set_ticket(&self, ticket: Option<Vec<u8>>) {
        self.state().ticket = ticket;
    }

    /// The double itself, to give to a [`TestLoader`].
    pub fn gun(&self) -> TestGun {
        TestGun {
            state: self.clone(),
        }
    }
}

/// The recording `CGun` double: all ten interfaces.
pub struct TestGun {
    state: TestGunHandle,
}

impl TestGun {
    fn record(&self, call: Call) {
        self.state.state().calls.push(call);
    }

    fn result(&self, method: &'static str) -> Hresult {
        let mut state = self.state.state();
        match state.results.get_mut(method).and_then(VecDeque::pop_front) {
            Some(hr) => hr,
            None if method == "connect" => Hresult::E_PENDING,
            None => Hresult::S_OK,
        }
    }

    fn call(&self, call: Call, method: &'static str) -> Hresult {
        self.record(call);
        self.result(method)
    }
}

macro_rules! other {
    ($self:ident, $name:literal) => {
        $self.call(Call::Other($name), $name)
    };
}

impl IGunConnection for TestGun {
    fn init(
        &self,
        a: &Guid,
        game_version: &str,
        locale: u32,
        cd_key: &str,
        port_local: u16,
    ) -> Hresult {
        self.state.state().inited = true;
        self.call(
            Call::Init {
                app: *a,
                game_version: game_version.to_owned(),
                locale,
                cd_key: cd_key.to_owned(),
                port_local,
            },
            "init",
        )
    }
    fn connect(&self, gun_server_name: &str, port: u16) -> Hresult {
        self.call(
            Call::Connect {
                server: gun_server_name.to_owned(),
                port,
            },
            "connect",
        )
    }
    fn disconnect(&self) -> Hresult {
        self.call(Call::Disconnect, "disconnect")
    }
    fn get_status(&self, _a: &mut ConnectStatus) -> Hresult {
        other!(self, "get_status")
    }
    fn get_last_error(&self) -> Hresult {
        other!(self, "get_last_error")
    }
    /// `CGun::DispatchQueue`, see the module documentation. The state lock is released before every
    /// sink call: the sink re-enters this object (`Connect`, `GetTicket`, `PresentTicket`).
    fn dispatch_queue(&self, sink: &dyn IGunQueueRead) -> Hresult {
        self.record(Call::DispatchQueue);
        if !self.state.state().inited {
            return ZT_E_NOT_INITED;
        }
        loop {
            let next = self.state.state().queue.pop_front();
            let Some(message) = next else {
                return Hresult::S_OK;
            };
            if !sink.read(&message).succeeded() {
                return Hresult(1);
            }
        }
    }
}

impl IGunNews for TestGun {
    fn get_news(&self, _a: &str, _b: &str, _c: u16, _context: Context) -> Hresult {
        other!(self, "get_news")
    }
}

impl IGunAccount for TestGun {
    fn create_account(&self, _: &str, _: &str, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "create_account")
    }
    fn change_password(&self, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "change_password")
    }
    fn change_password_from_hint(&self, _: &str, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "change_password_from_hint")
    }
    fn get_account_status(&self, _: &str, _: Context) -> Hresult {
        other!(self, "get_account_status")
    }
    fn get_password_hint(&self, _: &str, _: Context) -> Hresult {
        other!(self, "get_password_hint")
    }
    fn get_account_name(&self, _: &str, _: Context) -> Hresult {
        other!(self, "get_account_name")
    }
    fn set_account_value(&self, _: AccountValueType, _: &[u8]) -> Hresult {
        other!(self, "set_account_value")
    }
}

impl IGunFriends for TestGun {
    fn get_friends_list(&self, context: Context) -> Hresult {
        self.call(Call::GetFriendsList(context), "get_friends_list")
    }
    fn friend_add(&self, user: &str, context: Context) -> Hresult {
        self.call(Call::FriendAdd(user.to_owned(), context), "friend_add")
    }
    fn friend_delete(&self, user: &str, context: Context) -> Hresult {
        self.call(
            Call::FriendDelete(user.to_owned(), context),
            "friend_delete",
        )
    }
    fn get_ignored_list(&self, context: Context) -> Hresult {
        self.call(Call::GetIgnoredList(context), "get_ignored_list")
    }
    fn ignored_add(&self, user: &str, context: Context) -> Hresult {
        self.call(Call::IgnoredAdd(user.to_owned(), context), "ignored_add")
    }
    fn ignored_delete(&self, user: &str, context: Context) -> Hresult {
        self.call(
            Call::IgnoredDelete(user.to_owned(), context),
            "ignored_delete",
        )
    }
    fn flags(
        &self,
        op: FlagOperation,
        which_flag: FlagWord,
        bits_to_set_or_clear: u32,
        context: Context,
    ) -> Hresult {
        self.call(
            Call::Flags {
                op,
                which: which_flag,
                bits: bits_to_set_or_clear,
                context,
            },
            "flags",
        )
    }
    fn where_is(&self, _: &str, _: Context) -> Hresult {
        other!(self, "where_is")
    }
    fn set_location(&self, mode: u32, location: &str, context: Context) -> Hresult {
        self.call(
            Call::SetLocation {
                mode,
                location: location.to_owned(),
                context,
            },
            "set_location",
        )
    }
    fn accept_add_friend(&self, _: bool, _: &str, _: Context) -> Hresult {
        other!(self, "accept_add_friend")
    }
}

impl IGunChat for TestGun {
    fn get_list_of_rooms(&self, _: Context) -> Hresult {
        other!(self, "get_list_of_rooms")
    }
    fn join_room(&self, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "join_room")
    }
    fn leave_room(&self, _: u32, _: Context) -> Hresult {
        other!(self, "leave_room")
    }
    fn leave_all_rooms(&self, _: Context) -> Hresult {
        other!(self, "leave_all_rooms")
    }
    fn get_users_in_room(&self, _: u32, _: Context) -> Hresult {
        other!(self, "get_users_in_room")
    }
    fn send_to_room(&self, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "send_to_room")
    }
    fn send_to_user(&self, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "send_to_user")
    }
    fn get_squelch_list(&self, _: Context) -> Hresult {
        other!(self, "get_squelch_list")
    }
    fn squelch_add(&self, _: &str, _: Context) -> Hresult {
        other!(self, "squelch_add")
    }
    fn squelch_delete(&self, _: &str, _: Context) -> Hresult {
        other!(self, "squelch_delete")
    }
    fn create_room(&self, _: &str, _: &str, _: u32, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "create_room")
    }
    fn get_room_owner(&self, _: u32, _: Context) -> Hresult {
        other!(self, "get_room_owner")
    }
    fn assign_ownership(&self, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "assign_ownership")
    }
    fn set_room_topic(&self, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "set_room_topic")
    }
    fn set_room_mode(&self, _: u32, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "set_room_mode")
    }
    fn invite_user(&self, _: u32, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "invite_user")
    }
    fn kick_out_user(&self, _: u32, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "kick_out_user")
    }
    fn retrieve_chat_log(&self, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "retrieve_chat_log")
    }
    fn send_to_room_ex(
        &self,
        room_id: u32,
        message: &str,
        blob: &[u8],
        context: Context,
    ) -> Hresult {
        self.call(
            Call::SendToRoomEx {
                room_id,
                message: message.to_owned(),
                blob: blob.to_vec(),
                context,
            },
            "send_to_room_ex",
        )
    }
    fn send_to_user_ex(&self, user: &str, message: &str, blob: &[u8], context: Context) -> Hresult {
        self.call(
            Call::SendToUserEx {
                user: user.to_owned(),
                message: message.to_owned(),
                blob: blob.to_vec(),
                context,
            },
            "send_to_user_ex",
        )
    }
}

impl IGunHost for TestGun {
    fn create_game(&self, _: &str) -> Hresult {
        other!(self, "create_game")
    }
    fn end_game(&self) -> Hresult {
        other!(self, "end_game")
    }
    fn add_player(&self, _: u32, _: &str) -> Hresult {
        other!(self, "add_player")
    }
    fn delete_player(&self, _: u32) -> Hresult {
        other!(self, "delete_player")
    }
    fn get_num_players(&self, _: &mut u32) -> Hresult {
        other!(self, "get_num_players")
    }
    fn enum_players(&self, _: &mut dyn IGunHostEnum) -> Hresult {
        other!(self, "enum_players")
    }
    fn set_item_value(&self, _: u32, _: &str, _: &str) -> Hresult {
        other!(self, "set_item_value")
    }
    fn get_item_value(&self, _: u32, _: &str, _: &mut [u16]) -> Hresult {
        other!(self, "get_item_value")
    }
    fn delete_item_value(&self, _: u32, _: &str) -> Hresult {
        other!(self, "delete_item_value")
    }
    fn get_num_item_values(&self, _: u32, _: &mut u32) -> Hresult {
        other!(self, "get_num_item_values")
    }
    fn enum_item_values(&self, _: u32, _: &mut dyn IGunHostEnum) -> Hresult {
        other!(self, "enum_item_values")
    }
}

impl IGunBrowser for TestGun {
    fn get_view_list(&self, _: Context) -> Hresult {
        other!(self, "get_view_list")
    }
    fn get_header_row_of_view(&self, _: u32, _: Context) -> Hresult {
        other!(self, "get_header_row_of_view")
    }
    fn get_page(&self, _: u32, _: u32, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "get_page")
    }
    fn get_row_page(&self, _: u32, _: u32, _: u32, _: &str, _: Context) -> Hresult {
        other!(self, "get_row_page")
    }
}

impl IGunSupport for TestGun {
    fn call_create(
        &self,
        call_type: SupportCallType,
        a: Option<&str>,
        b: Option<&str>,
        c: &[u8],
        d: u32,
        context: Context,
    ) -> Hresult {
        self.call(
            Call::CallCreate {
                call_type,
                target: a.map(str::to_owned),
                description: b.map(str::to_owned),
                blob: c.to_vec(),
                locale: d,
                context,
            },
            "call_create",
        )
    }
    fn call_get_status(&self, _: u32, _: Context) -> Hresult {
        other!(self, "call_get_status")
    }
}

impl IGunPersona for TestGun {
    fn enter_persona(&self, _: &str, _: u32, _: Context) -> Hresult {
        other!(self, "enter_persona")
    }
    fn leave_persona(&self, _: Context) -> Hresult {
        other!(self, "leave_persona")
    }
    fn create_persona(&self, _: &str, _: u32, _: Context) -> Hresult {
        other!(self, "create_persona")
    }
    fn delete_persona(&self, _: &str, _: u32, _: Context) -> Hresult {
        other!(self, "delete_persona")
    }
    fn get_persona_list(&self, _: u32, _: Context) -> Hresult {
        other!(self, "get_persona_list")
    }
}

impl IGunTicket for TestGun {
    fn simulate_authentication(&self, _: &str, _: &str, _: &mut TagBlob) -> Hresult {
        other!(self, "simulate_authentication")
    }
    /// The double's `GetTicket`: no ticket -> `ZT_E_NOT_TICKETED`; a blob that cannot hold the
    /// ticket gets `cbSize` set and `ZT_E_BUFFER_TOO_SMALL`; else the bytes are copied in.
    fn get_ticket(&self, blob: &mut TagBlob) -> Hresult {
        self.record(Call::GetTicket);
        let ticket = self.state.state().ticket.clone();
        let Some(ticket) = ticket else {
            return ZT_E_NOT_TICKETED;
        };
        match &mut blob.data {
            Some(data) if data.len() >= ticket.len() => {
                data[..ticket.len()].copy_from_slice(&ticket);
                blob.cb_size = ticket.len() as u32;
                Hresult::S_OK
            }
            _ => {
                blob.cb_size = ticket.len() as u32;
                ZT_E_BUFFER_TOO_SMALL
            }
        }
    }
    fn get_ticket_ex(&self, _: Context) -> Hresult {
        other!(self, "get_ticket_ex")
    }
    fn set_ticket(&self, ticket: &mut TagBlob) -> Hresult {
        self.call(Call::SetTicket(ticket.bytes().to_vec()), "set_ticket")
    }
    fn present_ticket(&self, ticket: &[u8], context: Context) -> Hresult {
        self.call(
            Call::PresentTicket {
                ticket: ticket.to_vec(),
                context,
            },
            "present_ticket",
        )
    }
    fn release_ticket(&self, context: Context) -> Hresult {
        self.call(Call::ReleaseTicket(context), "release_ticket")
    }
    fn check_ticket(&self, _: &[u8], _: Context) -> Hresult {
        other!(self, "check_ticket")
    }
    fn boot_user(&self, _: &str, _: &str, _: Context) -> Hresult {
        other!(self, "boot_user")
    }
}

struct TestFactory(Option<TestGun>);

impl ClassFactory for TestFactory {
    fn create_instance(&mut self, _riid: &Guid) -> (Hresult, Option<Box<dyn GunObject>>) {
        match self.0.take() {
            Some(gun) => (Hresult::S_OK, Some(Box::new(gun))),
            None => (Hresult(0x8004_0110), None),
        }
    }
}

/// The module [`TestLoader`] loads: `DllGetClassObject` hands out the double once.
pub struct TestModule {
    gun: Option<TestGun>,
    export_missing: bool,
    last_error: u32,
    /// Every `(clsid, riid)` `DllGetClassObject` was called with.
    pub requests: Arc<Mutex<Vec<(Guid, Guid)>>>,
}

impl GunModule for TestModule {
    fn dll_get_class_object(
        &mut self,
        rclsid: &Guid,
        riid: &Guid,
    ) -> Option<(Hresult, Option<Box<dyn ClassFactory>>)> {
        if self.export_missing {
            return None;
        }
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((*rclsid, *riid));
        Some((
            Hresult::S_OK,
            Some(Box::new(TestFactory(self.gun.take())) as Box<dyn ClassFactory>),
        ))
    }
    fn get_last_error(&self) -> u32 {
        self.last_error
    }
}

/// `LoadLibraryA` for tests.
pub struct TestLoader {
    gun: Option<TestGun>,
    /// `Some(GetLastError())`: `LoadLibraryA` fails.
    pub fail_load: Option<u32>,
    /// The module has no `DllGetClassObject` export; `GetLastError()` is this.
    pub export_missing: Option<u32>,
    /// Every path `LoadLibraryA` was asked for.
    pub loaded: Vec<String>,
    /// The `(clsid, riid)` pairs `DllGetClassObject` received.
    pub requests: Arc<Mutex<Vec<(Guid, Guid)>>>,
}

impl TestLoader {
    pub fn new(gun: TestGun) -> Self {
        Self {
            gun: Some(gun),
            fail_load: None,
            export_missing: None,
            loaded: Vec::new(),
            requests: Arc::default(),
        }
    }
}

impl GunLoader for TestLoader {
    fn load_library(&mut self, path: &str) -> Result<Box<dyn GunModule>, u32> {
        self.loaded.push(path.to_owned());
        if let Some(error) = self.fail_load {
            return Err(error);
        }
        Ok(Box::new(TestModule {
            gun: self.gun.take(),
            export_missing: self.export_missing.is_some(),
            last_error: self.export_missing.unwrap_or(0),
            requests: Arc::clone(&self.requests),
        }))
    }
}
