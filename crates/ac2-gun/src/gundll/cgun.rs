//! `CGun`, GunDll's one COM object (`decompiled-pdb/gundll/GunDll/GunDll.dll.c`, `CGun::*`), and the
//! `IGunConnection` interface: `Init` (0x1000fab0), `Connect` (0x1000fbc0), `Disconnect` (0x1000fce0),
//! `GetStatus` (0x1000fd30), `GetLastError` (0x1000fd60), `DispatchQueue` (0x1000fd90). The other nine
//! interfaces are in `requests.rs` and `tickets.rs`; the handshake in `session.rs`.
//!
//! Field map (`CGun` offsets, `0a_send_session.md` 3.1-3.3 and `0a_receive_messages.md` 1.4):
//! `+0x38` initialised, `+0x40` the queue Q2, `+0x44` the current sink, `+0x48` the DP8 helper,
//! `+0x4c` the connection `conn`, `+0x50` status, `+0x54` last error, `+0x58` the application GUID,
//! `+0x68` the version string, `+0x6c` the GUID text, `+0x70` locale, `+0x74` the CD key, `+0x78` the
//! local port, `+0x60` the cached ticket, `+0x7c` the crypto object, `+0x80` the session key.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub(super) use super::crypto::{os_random, CryptContext, RandomSource, SessionKey};
use super::dp8::{Dp8Factory, IDirectPlay8ThreadPool};
use super::msg::GunMsg;
use super::queue::{read, Queue};
use super::registry::GunRegistry;
use super::transport::{NetEvent, Transport};
use crate::interfaces::*;
use crate::message::GunQueueMessage;
use crate::zt::*;
use crate::Hresult;

/// The DP8 thread pool, which retail keeps in a process-wide singleton (`DAT_10069870`, created by
/// `FUN_10003810`, `SetThreadCount(-1, 0, 0)` called once). `FUN_10003810` is the first thing the
/// connection-handler constructor `FUN_10035ae0` calls, i.e. the pool is made by the first
/// `IGunConnection::Connect` of any `CGun` (never by `Init`), and a failed creation is tried again
/// by every later `Connect` while the slot is empty. Every `CGun` that shares a slot shares the pool; [`SharedThreadPool::process`] is the slot of the
/// calling thread (Rust cannot hold an `Rc` in a process-wide static), [`SharedThreadPool::new`] a
/// private one.
#[derive(Clone, Default)]
pub struct SharedThreadPool(Rc<RefCell<Option<Rc<dyn IDirectPlay8ThreadPool>>>>);

thread_local! {
    static PROCESS_POOL: SharedThreadPool = SharedThreadPool::default();
}

impl SharedThreadPool {
    /// An empty private slot.
    pub fn new() -> SharedThreadPool {
        SharedThreadPool::default()
    }

    /// The slot every `CGun` of this thread shares by default.
    pub fn process() -> SharedThreadPool {
        PROCESS_POOL.with(Clone::clone)
    }
}

/// What GunDll takes from Win32 and COM, supplied by the caller.
pub struct GunEnv {
    /// The (shared) thread pool slot.
    pub pool: SharedThreadPool,
    /// `CoCreateInstance` of the DirectPlay 8 classes.
    pub dp8: Rc<dyn Dp8Factory>,
    /// The registry values under `HKCU\Software\Microsoft\Microsoft Games`.
    pub registry: Rc<dyn GunRegistry>,
    /// `CryptGenRandom`.
    pub random: RandomSource,
    /// `FUN_1003d6d0`'s local-port probe (`socket`/`bind`/`getsockname`/`closesocket` on UDP port 0);
    /// `None` when it fails.
    pub probe_port: Box<dyn FnMut() -> Option<u16>>,
}

impl GunEnv {
    /// An environment with the operating system's random source and a real UDP probe.
    pub fn new(dp8: Rc<dyn Dp8Factory>, registry: Rc<dyn GunRegistry>) -> GunEnv {
        GunEnv {
            pool: SharedThreadPool::process(),
            dp8,
            registry,
            random: os_random(),
            probe_port: Box::new(|| {
                std::net::UdpSocket::bind(("0.0.0.0", 0))
                    .ok()
                    .and_then(|s| s.local_addr().ok())
                    .map(|a| a.port())
            }),
        }
    }
}

/// What the queue Q2 holds (`GUNQueueMessage.gmid`): 0 an internal connection event `{type, H, hr}`
/// written by the handler `H`, 1 a message object.
#[derive(Debug)]
pub(super) enum Q2Entry {
    /// gmid 0: `FUN_10009e10` handles type 0 (connect completed: send `<Connect>`), 1 (failed) and 2
    /// (terminated); 3 and 4 do nothing.
    Internal { kind: u32, conn: u64, hr: Hresult },
    /// gmid 1: a message object (`flags` 4).
    Message(GunMsg),
}

/// `conn` (`FUN_10035ae0`, the 0x14-byte object at `CGun+0x4c`).
pub(super) struct Conn {
    pub id: u64,
    /// `conn+4`/`+8`/`+0xc` wired by `FUN_100034d0`; `None` when `FUN_10003b90` failed.
    pub transport: Option<Rc<Transport>>,
}

/// `CGun`'s mutable members.
pub(super) struct State {
    pub inited: bool,
    pub status: u32,
    pub last_error: Hresult,
    pub app_guid: Option<Guid>,
    /// `+0x68`: stored and never read (`0a_send_session.md` 3.1).
    pub version: String,
    pub guid_text: String,
    pub locale: u32,
    pub cd_key: String,
    pub port_local: u16,
    pub conn: Option<Conn>,
    pub crypt: Option<CryptContext>,
    pub session_key: Option<SessionKey>,
    pub ticket: Option<Vec<u8>>,
    pub next_conn_id: u64,
}

/// The GunDll COM object. All ten `IGun*` interfaces are implemented on it.
pub struct CGun {
    pub(super) env: RefCell<GunEnv>,
    pub(super) st: RefCell<State>,
    pub(super) q2: RefCell<Queue<Q2Entry>>,
    /// `CGun+0x44 != 0`: a `DispatchQueue` is running with its sink stored. `DispatchQueue` zeroes the
    /// field when it returns, so after a nested call the outer loop has no sink.
    pub(super) sink_set: Cell<bool>,
}

impl CGun {
    pub fn new(env: GunEnv) -> CGun {
        CGun {
            env: RefCell::new(env),
            st: RefCell::new(State {
                inited: false,
                status: 0,
                last_error: Hresult::S_OK,
                app_guid: None,
                version: String::new(),
                guid_text: String::new(),
                locale: 0,
                cd_key: String::new(),
                port_local: 0,
                conn: None,
                crypt: None,
                session_key: None,
                ticket: None,
                next_conn_id: 1,
            }),
            q2: RefCell::new(Queue::default()),
            sink_set: Cell::new(false),
        }
    }

    /// The value `<Connect Cx="0x...">` carries: "the `CGun` pointer" (`FUN_10009e10` passes `param_1`),
    /// as a 32-bit value.
    pub fn connect_cx(&self) -> u32 {
        self as *const CGun as usize as u32
    }

    /// `CGun+0x4c != 0`, the "connected" test of the friends, chat, browser, support, persona and
    /// ticket interfaces.
    pub(super) fn has_conn(&self) -> bool {
        self.st.borrow().conn.is_some()
    }

    pub(super) fn inited(&self) -> bool {
        self.st.borrow().inited
    }

    /// `CGun::SendRequest` (0x100018a0): `FUN_10004d00` renders the text to UTF-8 with its NUL,
    /// `FUN_10003880` sends it on `conn`. The length handed down is the UTF-8 length including the NUL,
    /// truncated to 16 bits by the transport.
    pub(super) fn send_request(&self, msg: &GunMsg) -> Hresult {
        let text = match msg.render() {
            Ok(t) => t,
            Err(e) => return e,
        };
        let mut bytes = text.into_bytes();
        bytes.push(0);
        let transport = {
            let st = self.st.borrow();
            st.conn.as_ref().and_then(|c| c.transport.clone())
        };
        match transport {
            Some(t) => t.send_text(&bytes),
            // conn+8 == 0: FUN_10003880 returns ZT_E_NOT_CONNECTED
            None => ZT_E_NOT_CONNECTED,
        }
    }

    /// Writes a message object to Q2 as `{flags 4, gmid 1, ...}`.
    pub(super) fn post(&self, msg: GunMsg) -> Hresult {
        self.q2.borrow_mut().write(4, Q2Entry::Message(msg))
    }

    /// `FUN_10003920`: pump DP8 (`DoWork(-1)` when `DAT_10069870` is set), then drain Q1 and let the
    /// connection handler `H` write to Q2.
    fn pump(&self) {
        let (transport, conn_id) = {
            let st = self.st.borrow();
            let Some(conn) = st.conn.as_ref() else {
                return;
            };
            (conn.transport.clone(), conn.id)
        };
        // conn+4 == 0 -> ZT_E_NOT_INITED, ignored by DispatchQueue
        let Some(transport) = transport else {
            return;
        };
        let pool = self.env.borrow().pool.0.borrow().clone();
        if let Some(pool) = pool {
            // wrapper 0x1003b590: DoWork(-1, 0), DPNERR_NOTALLOWED mapped to 0
            let _ = pool.do_work(u32::MAX, 0);
        }
        self.route_net_events(conn_id, transport.drain());
    }

    /// What the connection handler `H` does with the events the consumer raises: H slot 2
    /// (0x10035b60) writes Q2 `{type 0 when hr >= 0 else 1, H, hr}`, H slot 5 (0x10035c00) `{2, H, hr}`,
    /// `H.OnReceive` (0x10035ca0) `{flags 4, gmid 1, message}`.
    fn route_net_events(&self, conn_id: u64, events: Vec<NetEvent>) {
        for ev in events {
            match ev {
                NetEvent::ConnectComplete(hr) => {
                    self.q2.borrow_mut().write(
                        0,
                        Q2Entry::Internal {
                            kind: if hr.succeeded() { 0 } else { 1 },
                            conn: conn_id,
                            hr,
                        },
                    );
                }
                NetEvent::Disconnected(hr) => {
                    self.q2.borrow_mut().write(
                        0,
                        Q2Entry::Internal {
                            kind: 2,
                            conn: conn_id,
                            hr,
                        },
                    );
                }
                NetEvent::Message(m) => {
                    self.post(m);
                }
            }
        }
    }

    /// `FUN_10003810` (0x10003810), run by the connection-handler constructor `FUN_10035ae0` of every
    /// `Connect`: while `DAT_10069870` is NULL the DP8 thread pool is created (`FUN_1003ba10`) and set
    /// to zero worker threads (`SetThreadCount(-1, 0, 0)`, wrapper 0x1003b530). A failed creation
    /// is ignored; the next `Connect` tries again.
    fn ensure_thread_pool(&self) {
        let env = self.env.borrow();
        if env.pool.0.borrow().is_some() {
            return;
        }
        if let Ok(p) = env.dp8.create_thread_pool() {
            let _ = p.set_thread_count(u32::MAX, 0, 0);
            *env.pool.0.borrow_mut() = Some(p);
        }
    }
}

/// `StringFromGUID2` text: `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` (upper case).
pub fn guid_text(g: &Guid) -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        g.data1,
        g.data2,
        g.data3,
        g.data4[0],
        g.data4[1],
        g.data4[2],
        g.data4[3],
        g.data4[4],
        g.data4[5],
        g.data4[6],
        g.data4[7]
    )
}

impl IGunConnection for CGun {
    /// `CGun::Init` (0x1000fab0): `ZT_E_ALREADY_INITED` when already initialised; (`FUN_10001910`)
    /// the queues; (`FUN_100019a0`, `FUN_100037d0`) the helper, `CoInitializeEx` and the global heap;
    /// (`FUN_10002840`) the GUID text; then the stores of `+0x38`, `+0x58`, `+0x68`, `+0x6c`, `+0x70`,
    /// `+0x74`, `+0x78`. It creates no DP8 thread pool: that is `Connect`'s
    /// ([`CGun::ensure_thread_pool`]). (A NULL `p2` / `p4` is `0x80070057` in retail; `&str` cannot be
    /// NULL.)
    fn init(
        &self,
        a: &Guid,
        game_version: &str,
        locale: u32,
        cd_key: &str,
        port_local: u16,
    ) -> Hresult {
        if self.st.borrow().inited {
            return ZT_E_ALREADY_INITED;
        }
        let mut st = self.st.borrow_mut();
        st.inited = true;
        st.app_guid = Some(*a);
        st.version = game_version.to_owned();
        st.guid_text = guid_text(a);
        st.locale = locale;
        st.cd_key = cd_key.to_owned();
        st.port_local = port_local;
        Hresult::S_OK
    }

    /// `CGun::Connect` (0x1000fbc0): not initialised -> `ZT_E_NOT_INITED`; an earlier connection is
    /// disconnected first (also when the new call is then rejected); `FUN_10003b90` makes the
    /// connection; success sets status 1 and answers `E_PENDING`, failure sets status 4 and the last
    /// error. `conn` is stored in both cases. (`serverName == NULL` is `0x80070057`; `&str` cannot be NULL.)
    fn connect(&self, gun_server_name: &str, port: u16) -> Hresult {
        if !self.inited() {
            return ZT_E_NOT_INITED;
        }
        if self.has_conn() {
            self.disconnect();
        }
        // `FUN_10035ae0` (the handler constructor) runs `FUN_10003810` before `FUN_10003b90`
        self.ensure_thread_pool();
        let (guid, local_port, id) = {
            let mut st = self.st.borrow_mut();
            let id = st.next_conn_id;
            st.next_conn_id += 1;
            (
                st.app_guid.unwrap_or(Guid {
                    data1: 0,
                    data2: 0,
                    data3: 0,
                    data4: [0; 8],
                }),
                st.port_local,
                id,
            )
        };
        let result = {
            let mut env = self.env.borrow_mut();
            let dp8 = env.dp8.clone();
            Transport::connect(
                dp8,
                gun_server_name,
                port,
                guid,
                local_port,
                &mut *env.probe_port,
            )
        };
        let mut st = self.st.borrow_mut();
        match result {
            Err(hr) => {
                st.status = 4;
                st.last_error = hr;
                st.conn = Some(Conn {
                    id,
                    transport: None,
                });
                hr
            }
            Ok(t) => {
                st.status = 1;
                st.conn = Some(Conn {
                    id,
                    transport: Some(t),
                });
                Hresult::E_PENDING
            }
        }
    }

    /// `CGun::Disconnect` (0x1000fce0): `ZT_E_NOT_INITED`; no connection -> `ZT_E_CLOSED`; else the
    /// connection is destroyed (`FUN_100038f0` -> `A.vtable[0](A, 0)`, see [`Transport::close`]: the
    /// DP8 client is reset first, then Q1 is drained, which turns the injected `USERCANCEL`
    /// terminate into a `DisconnectEv` (0x1301) for the next `DispatchQueue`), `conn` cleared, the
    /// host cleanup `FUN_1000d7d0` run (nothing to do without a hosted game) and the status set to 3.
    fn disconnect(&self) -> Hresult {
        let conn = {
            let mut st = self.st.borrow_mut();
            if !st.inited {
                return ZT_E_NOT_INITED;
            }
            match st.conn.take() {
                None => return ZT_E_CLOSED,
                Some(c) => {
                    st.status = 3;
                    c
                }
            }
        };
        self.retire(conn);
        Hresult::S_OK
    }

    /// `CGun::GetStatus` (0x1000fd30).
    fn get_status(&self, a: &mut ConnectStatus) -> Hresult {
        let st = self.st.borrow();
        if !st.inited {
            return ZT_E_NOT_INITED;
        }
        *a = match st.status {
            1 => ConnectStatus::Connecting,
            2 => ConnectStatus::Connected,
            3 => ConnectStatus::Disconnected,
            4 => ConnectStatus::Failed,
            _ => ConnectStatus::Created,
        };
        Hresult::S_OK
    }

    /// `CGun::GetLastError` (0x1000fd60): the saved HRESULT while the status is 4, else 0.
    fn get_last_error(&self) -> Hresult {
        let st = self.st.borrow();
        if !st.inited {
            return ZT_E_NOT_INITED;
        }
        if st.status == 4 {
            st.last_error
        } else {
            Hresult::S_OK
        }
    }

    /// `CGun::DispatchQueue` (0x1000fd90): `ZT_E_NOT_INITED` when not initialised; else the sink is
    /// stored, a connection is pumped (`FUN_10003920`) and Q2 is read until the sink answers below
    /// zero (the queue then returns 1) or the queue is empty (`E_PENDING` becomes `S_OK`). The sink
    /// re-enters this object, so no borrow is held across its call: one entry is popped, the borrow
    /// released, the sink called, repeat.
    fn dispatch_queue(&self, sink: &dyn IGunQueueRead) -> Hresult {
        if !self.inited() {
            // `+0x44 = 0`, then ZT_E_NOT_INITED
            self.sink_set.set(false);
            return ZT_E_NOT_INITED;
        }
        self.sink_set.set(true);
        self.pump();
        let mut hr;
        loop {
            hr = read(&self.q2, |entry| self.handle_entry(entry, sink));
            if hr != Hresult::S_OK {
                break;
            }
        }
        if hr == Hresult::E_PENDING {
            hr = Hresult::S_OK;
        }
        // `*(CGun+0x44) = 0` on every return, also for a nested call (the outer loop then has no sink)
        self.sink_set.set(false);
        hr
    }
}

impl CGun {
    /// `FUN_100038f0(H)` for a connection `Disconnect` takes away: the transport is destroyed
    /// ([`Transport::close`]) and what the handler `H` raised meanwhile (datagrams still in Q1 as
    /// messages, the injected terminate as a type-2 event for this connection) goes to Q2.
    pub(super) fn retire(&self, conn: Conn) {
        if let Some(t) = conn.transport {
            self.route_net_events(conn.id, t.close());
        }
    }

    /// `FUN_1000a380` (0x1000a380): gmid 0 -> `FUN_10009e10`, 1 -> `FUN_1000a300`.
    fn handle_entry(&self, entry: Q2Entry, sink: &dyn IGunQueueRead) -> Hresult {
        match entry {
            Q2Entry::Internal { kind, conn, hr } => self.connection_event(kind, conn, hr),
            Q2Entry::Message(m) => self.route_message(m, sink),
        }
    }

    /// `FUN_1000a300` (0x1000a300): 0x1200 -> `FUN_1000a100` (consumed), 0x1203 -> `FUN_10009fd0`
    /// (consumed), 0x7200 swallowed; everything else to the sink at `CGun+0x44` (none: dropped, 0).
    /// The object is deleted afterwards either way.
    fn route_message(&self, m: GunMsg, sink: &dyn IGunQueueRead) -> Hresult {
        use super::client_msg::to_client;
        use super::ids::*;
        match m.def.id {
            CONNECT_RES => self.on_connect_res(&m),
            CON_INFO_RES => self.on_con_info_res(&m),
            HOST_DATA_RES => Hresult::S_OK,
            // a NULL sink (zeroed by a nested DispatchQueue that returned) drops the message
            _ if !self.sink_set.get() => Hresult::S_OK,
            _ => sink.read(&GunQueueMessage {
                flags: 4,
                gmid: 1,
                param: 0,
                cb: 0,
                message: to_client(&m),
            }),
        }
    }
}
