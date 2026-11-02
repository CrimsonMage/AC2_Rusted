//! `NSTurbineGun::TurbineGun` (`ac2/shared/src/gun/turbinegun.cpp.c`): Turbine's client-side
//! wrapper around GunDll's `CGun`. It makes the object, queues requests with a context, and is
//! the `IGUNQueueRead` sink that `IGunConnection::DispatchQueue` hands each message to
//! ([`IGunQueueRead::read`]). Admin 7148 and Legions 7160 run the same code in every function
//! (`decompiled-pdb/gundll/spec/0c_client.md` section 0), so each function cites both addresses.
//!
//! Layout in retail (0x8C bytes): `+0 / +4` vptrs, `+8 m_ConnectEventList`, `+0x18
//! m_ChatEventList`, `+0x28 m_FriendsEventList`, `+0x38..+0x5C` ten `IGun*` pointers (one object),
//! `+0x60 m_fConnected`, `+0x64 m_fConnecting`, `+0x68 m_fLoggedIn`, `+0x6C
//! m_fPersistentConnection`, `+0x70 m_gunServerName`, `+0x74 m_port`, `+0x78 m_HrErrString`,
//! `+0x7C m_shardID` (1), `+0x80 m_worldName`, `+0x84 m_worldSuffix`, `+0x88 m_hGunDll`.
//! `m_HrErrString` and `m_shardID` are never read by any function and are not kept.
//!
//! Representation (deviations, listed in the crate's report to the lead):
//! * The ten interface pointers are one `Rc<dyn GunObject>` (`CGun` answers all ten; `Init`
//!   fails unless every `QueryInterface` succeeds, which a `GunObject` always does), handed over
//!   through a [`GunLoader`] because Rust cannot `LoadLibrary` GunDll.
//! * A listener or context is an `Rc<dyn ...>` (its retail refcount is the `Rc`'s); the context
//!   pointer GunDll carries in `Cx` is a [`Context`] id for an entry of `contexts`.
//! * Listener lists keep retail's order: `List::insert` puts the new node before the head, so the
//!   newest listener is first and is notified first (confirmed in `List<>::insert`, Legions
//!   0x005F8DD0).
//! * `GetUserDefaultLCID()` is a Win32 call; the caller gives its value to [`TurbineGun::new`].

mod connect;
mod events;
pub(crate) mod names;
mod requests;
mod results;
mod routing;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_requests;
#[cfg(test)]
mod tests_results;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
#[cfg(not(test))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::ct::{CtChatEvent, CtConnectEvent, CtFriendsEvent, PendingContext};
use crate::interfaces::{Context, Guid, GunObject};
use crate::turbine_gun_utils::{
    gun_create_instance, print_cmd_hresult, set_s_p_turbine_gun, GunLoader, GunModule, WorldNames,
};
use crate::Hresult;

/// `TurbineGun::S_fStripWorldNameFromRoomChatEvents` (Legions 0x00A3B694, admin 0x00AD662C):
/// file-initialised to 1 in both builds; `gmClient` clears it for `-tchat` and a character set
/// that uses Turbine chat.
#[cfg(not(test))]
static S_F_STRIP_WORLD_NAME_FROM_ROOM_CHAT_EVENTS: AtomicBool = AtomicBool::new(true);
#[cfg(test)]
thread_local! {
    static S_F_STRIP_WORLD_NAME_FROM_ROOM_CHAT_EVENTS: std::cell::Cell<bool> =
        const { std::cell::Cell::new(true) };
}

/// `CLSID` of GunDll's `CGun` {76633332-52a2-4460-b089-3541a8443c9a} (`Init`, Legions 0x009E8C14).
pub const CLSID_GUN: Guid = Guid {
    data1: 0x7663_3332,
    data2: 0x52a2,
    data3: 0x4460,
    data4: [0xb0, 0x89, 0x35, 0x41, 0xa8, 0x44, 0x3c, 0x9a],
};

/// `IID_IGunConnection` {eeeeee21-85ea-40f7-8157-cd07a138b49f} (`Init`, Legions 0x009E8C24).
pub const IID_IGUN_CONNECTION: Guid = Guid {
    data1: 0xeeee_ee21,
    data2: 0x85ea,
    data3: 0x40f7,
    data4: [0x81, 0x57, 0xcd, 0x07, 0xa1, 0x38, 0xb4, 0x9f],
};

/// The application GUID `RequestConnect` passes to `IGunConnection::Init`
/// {2280767d-1689-4d6b-81dd-3ad9e80fb655} (Legions 0x009E8B10).
pub const APP_GUID: Guid = Guid {
    data1: 0x2280_767d,
    data2: 0x1689,
    data3: 0x4d6b,
    data4: [0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f, 0xb6, 0x55],
};

/// `0x8000000A`, `IGunConnection::Connect`'s only success value.
const E_PENDING: Hresult = Hresult::E_PENDING;

/// `NSTurbineGun::TurbineGun`.
///
/// Every method takes `&self` and the fields are `Cell`/`RefCell`s whose borrows are never held
/// across a listener, context or GunDll call: retail is re-entered from inside `DispatchQueue`
/// (`Read` -> `RcvConnectEvent` -> `IGunConnection::Connect`) and its listeners call back into
/// `TurbineGun` (`RequestPresentAuthenticationTicket` from the RECONNECTED handler), so a
/// `CCli*` handler holding the same `TurbineGun` can call its request methods directly.
pub struct TurbineGun {
    /// `m_pGunConnect` .. `m_pGunSupport` (+0x38..+0x5C): `None` until `Init` succeeds.
    pub(crate) gun: RefCell<Option<Rc<dyn GunObject>>>,
    /// `m_hGunDll` (+0x88): the loaded module, kept for as long as the object.
    module: RefCell<Option<Box<dyn GunModule>>>,
    /// `m_ConnectEventList` (+8), head first (the newest listener).
    pub(crate) connect_event_list: RefCell<Vec<Rc<dyn CtConnectEvent>>>,
    /// `m_ChatEventList` (+0x18).
    pub(crate) chat_event_list: RefCell<Vec<Rc<dyn CtChatEvent>>>,
    /// `m_FriendsEventList` (+0x28).
    pub(crate) friends_event_list: RefCell<Vec<Rc<dyn CtFriendsEvent>>>,
    /// `m_fConnected` (+0x60).
    pub(crate) connected: Cell<bool>,
    /// `m_fConnecting` (+0x64).
    pub(crate) connecting: Cell<bool>,
    /// `m_fLoggedIn` (+0x68): cleared by a disconnect and a failed connect and by an
    /// `AccountsRes` 0x2200, set by no function.
    pub(crate) logged_in: Cell<bool>,
    /// `m_fPersistentConnection` (+0x6C).
    pub(crate) persistent_connection: Cell<bool>,
    /// `m_gunServerName` (+0x70).
    pub(crate) gun_server_name: RefCell<String>,
    /// `m_port` (+0x74).
    pub(crate) port: Cell<u16>,
    /// `m_worldName` (+0x80) and `m_worldSuffix` (+0x84), shared with `S_pTurbineGun`.
    pub(crate) names: Arc<WorldNames>,
    /// `GetUserDefaultLCID()`, which `RequestConnect` (locale 0) and `RequestCallCreate` use.
    pub(crate) user_default_lcid: u32,
    /// The request contexts waiting for their result, by [`Context`] id.
    pub(crate) contexts: RefCell<HashMap<u32, PendingContext>>,
    next_context: Cell<u32>,
}

impl TurbineGun {
    /// `NSTurbineGun::TurbineGun::TurbineGun` (admin 0x00606250, Legions 0x005C70F0): no
    /// interfaces, the flags 0, empty lists, `m_gunServerName` the null string, `m_port` 0,
    /// `m_worldName` `L""`, `m_worldSuffix` the null string, `m_hGunDll` 0, and
    /// `S_pTurbineGun = this`. (`m_pGunSupport` is left uninitialised by retail; `Init` sets it.)
    pub fn new(user_default_lcid: u32) -> Self {
        let names = Arc::new(WorldNames::default());
        set_s_p_turbine_gun(Arc::clone(&names));
        Self {
            gun: RefCell::new(None),
            module: RefCell::new(None),
            connect_event_list: RefCell::new(Vec::new()),
            chat_event_list: RefCell::new(Vec::new()),
            friends_event_list: RefCell::new(Vec::new()),
            connected: Cell::new(false),
            connecting: Cell::new(false),
            logged_in: Cell::new(false),
            persistent_connection: Cell::new(false),
            gun_server_name: RefCell::new(String::new()),
            port: Cell::new(0),
            names,
            user_default_lcid,
            contexts: RefCell::new(HashMap::new()),
            next_context: Cell::new(0),
        }
    }

    /// `S_fStripWorldNameFromRoomChatEvents`.
    pub fn strip_world_name_from_room_chat_events() -> bool {
        #[cfg(not(test))]
        {
            S_F_STRIP_WORLD_NAME_FROM_ROOM_CHAT_EVENTS.load(Ordering::Relaxed)
        }
        #[cfg(test)]
        S_F_STRIP_WORLD_NAME_FROM_ROOM_CHAT_EVENTS.with(std::cell::Cell::get)
    }

    /// Writes `S_fStripWorldNameFromRoomChatEvents` (`gmClient::EvaluateCommandLineArg` for
    /// `-tchat` and `RecvNotice_CharacterSet` clear it).
    pub fn set_strip_world_name_from_room_chat_events(strip: bool) {
        #[cfg(not(test))]
        S_F_STRIP_WORLD_NAME_FROM_ROOM_CHAT_EVENTS.store(strip, Ordering::Relaxed);
        #[cfg(test)]
        S_F_STRIP_WORLD_NAME_FROM_ROOM_CHAT_EVENTS.with(|f| f.set(strip));
    }

    /// `m_fConnected`: `gmClient::StartupGUN` tests it after the first connect event.
    pub fn is_connected(&self) -> bool {
        self.connected.get()
    }

    /// `m_fConnecting`.
    pub fn is_connecting(&self) -> bool {
        self.connecting.get()
    }

    /// `m_fLoggedIn`.
    pub fn is_logged_in(&self) -> bool {
        self.logged_in.get()
    }

    /// `m_fPersistentConnection`.
    pub fn is_persistent_connection(&self) -> bool {
        self.persistent_connection.get()
    }

    /// `m_gunServerName`.
    pub fn gun_server_name(&self) -> String {
        self.gun_server_name.borrow().clone()
    }

    /// `m_port`.
    pub fn port(&self) -> u16 {
        self.port.get()
    }

    /// Whether `Init` stored the interfaces (`m_pGunConnect != NULL`).
    pub fn has_gun(&self) -> bool {
        self.gun.borrow().is_some()
    }

    /// The shared GunDll object (a clone of the pointer; no borrow is kept).
    pub(crate) fn gun_rc(&self) -> Option<Rc<dyn GunObject>> {
        self.gun.borrow().clone()
    }

    /// The names `S_pTurbineGun` reaches for this object.
    pub fn world_names(&self) -> Arc<WorldNames> {
        Arc::clone(&self.names)
    }

    /// `NSTurbineGun::TurbineGun::Init` (admin 0x00605F00, Legions 0x005C6DA0):
    /// `int Init(char* gunDllPath)`.
    ///
    /// 1. An empty path is `"GunDll.dll"`.
    /// 2. `LoadLibraryA`: a failure returns 0 (`m_hGunDll` is stored even then; the module is
    ///    kept here as soon as it loads).
    /// 3. `TurbineDwExceptionFilter::AddDllToDumpCapture` (crash-dump bookkeeping) is not ported.
    /// 4. `GunCreateInstance(hmod, CLSID_GUN, IID_IGunConnection)`: `hr < 0` returns 0.
    /// 5. The nine `QueryInterface` calls (account, news, chat, friends, host, browser, persona,
    ///    ticket, support), each failing `Init`: a [`GunObject`] answers all of them.
    /// 6. Stores the interfaces and returns 1.
    ///
    /// When `DllGetClassObject` is missing, retail's `GunCreateInstance` returns
    /// `GetLastError()` (positive, so `hr >= 0` here) with a NULL connection and the next
    /// `QueryInterface` call crashes; no object means `false` here.
    pub fn init(&self, gun_dll_path: &str, loader: &mut dyn GunLoader) -> bool {
        let path = if gun_dll_path.is_empty() {
            "GunDll.dll"
        } else {
            gun_dll_path
        };
        let Ok(module) = loader.load_library(path) else {
            return false;
        };
        let (hr, connection) = {
            let mut slot = self.module.borrow_mut();
            let module = slot.insert(module);
            gun_create_instance(module.as_mut(), &CLSID_GUN, &IID_IGUN_CONNECTION)
        };
        if !hr.succeeded() {
            return false;
        }
        let Some(connection) = connection else {
            return false;
        };
        *self.gun.borrow_mut() = Some(Rc::from(connection));
        true
    }

    /// `AddRef(ctx)` (an `InterlockedIncrement` of the context's reference count) plus the
    /// pointer GunDll will carry back: the pending entry, under a fresh nonzero id that no
    /// pending entry has.
    pub(crate) fn add_ref(&self, ctx: PendingContext) -> Context {
        let mut contexts = self.contexts.borrow_mut();
        let mut id = self.next_context.get();
        loop {
            id = id.wrapping_add(1);
            if id != 0 && !contexts.contains_key(&id) {
                break;
            }
        }
        self.next_context.set(id);
        contexts.insert(id, ctx);
        Context(id)
    }

    /// The reference count of the contexts waiting for a result (a failed request leaves its
    /// `AddRef` un-undone, as in retail).
    pub fn pending_context_count(&self) -> usize {
        self.contexts.borrow().len()
    }

    /// Registers a context the way a `Request*` wrapper does (`AddRef`) without calling an
    /// interface, for tests and for result classes that have no wrapper.
    #[cfg(any(test, feature = "test-support"))]
    pub fn register_context(&self, ctx: PendingContext) -> Context {
        self.add_ref(ctx)
    }
}

impl Drop for TurbineGun {
    /// `NSTurbineGun::TurbineGun::~TurbineGun` (admin 0x00606330, Legions 0x005C71D0), with
    /// `scalar_deleting_destructor` (admin 0x00606DD0, Legions 0x005C7C70): `m_fPersistentConnection
    /// = 0`; if connected `IGunConnection::Disconnect()` and `PrintCmdHRESULT("Disconnect")`;
    /// `UnRegisterEvents()`; release the interfaces; `m_hGunDll = 0` (no `FreeLibrary`); release
    /// the names and flush the three lists. `S_pTurbineGun` is NOT cleared.
    ///
    /// Deviation: retail never releases `m_pGunSupport` (the `CGun` object leaks one reference);
    /// dropping the `Box` frees the object. The module is dropped here, retail never unloads it.
    fn drop(&mut self) {
        self.persistent_connection.set(false);
        if self.connected.get() {
            if let Some(gun) = self.gun_rc() {
                let hr = gun.disconnect();
                print_cmd_hresult("Disconnect", hr);
            }
        }
        self.un_register_events();
        *self.gun.get_mut() = None;
        *self.module.get_mut() = None;
    }
}
