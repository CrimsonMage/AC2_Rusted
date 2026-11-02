//! GunDll's DirectPlay 8 transport layer: the connect (`FUN_10003b90` -> `FUN_1003dbd0` ->
//! `FUN_1003d6d0`), the DP8 message handler (`Dp8MessageHandler`, 0x1003d1b0), the consumer
//! (`Dp8Dispatcher`, `FUN_1003c740` 0x1003c740) with the connection handler `H` (`FUN_10035ae0`,
//! vtable 0x1005c58c), the framing (`FUN_10003880`, `FUN_10042ed0`, `FUN_1003b820`) and the send
//! (`FUN_1003c640`). Specs: `0a_send_session.md` sections 1-3 and `0a_receive_messages.md` 1.1-1.2.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use super::dp8::*;
use super::msg::{parse_message, GunMsg};
use crate::interfaces::Guid;
use crate::zt::*;
use crate::Hresult;

/// The message type word `FUN_10003880` passes to the append call and `FUN_10042ed0` writes at
/// offset 0: 0xC350 (50000), bytes `50 C3`.
pub const WIRE_TYPE: u16 = 0xC350;
/// Frames with a type below this are control frames (`Dp8Dispatcher`: `*puVar5 < 1000`).
pub const APP_TYPE_MIN: u16 = 1000;
/// `GetCaps`/`SetCaps` keep-alive: `FUN_1003d080` is called with 0x3A98 (15000 ms).
pub const KEEP_ALIVE_MS: u32 = 15000;
/// `E_ACCESSDENIED`, the sink's answer to a refused connect (0x100035d0).
pub const E_ACCESSDENIED: Hresult = Hresult(0x8007_0005);

/// The Q1 entries the handler writes: the `GUNQueueMessage` `{flags, gmid 0xc350/0xc351, param, cb,
/// pvBuffer}` as the message it carries.
#[derive(Debug)]
pub struct Dp8Handler {
    q1: Rc<RefCell<VecDeque<Dp8Message>>>,
    /// The send contexts whose "notify" byte (`ctx->byte[8]`) is set: `FUN_10003170` makes every
    /// message notify (`P.vtable[16](P, 1)`), `FUN_1003c640` copies it into the context.
    notify: Rc<RefCell<HashSet<u32>>>,
    /// `timeGetTime()` at arrival of each RECEIVE, by buffer handle: the handler (0x1003d1b0, asm
    /// 0x1003d216) stores it at `+4` of the Q1 block, and the dispatcher passes it on.
    arrivals: Rc<RefCell<HashMap<u32, u32>>>,
    started: Instant,
}

impl Dp8Handler {
    pub fn new(
        q1: Rc<RefCell<VecDeque<Dp8Message>>>,
        notify: Rc<RefCell<HashSet<u32>>>,
    ) -> Dp8Handler {
        Dp8Handler {
            q1,
            notify,
            arrivals: Rc::default(),
            started: Instant::now(),
        }
    }
}

impl Dp8Handler {
    /// `timeGetTime()` on the handler's clock.
    fn now_ms(&self) -> u32 {
        self.started.elapsed().as_millis() as u32
    }

    /// The arrival time recorded for a RECEIVE's buffer handle (0 when unknown).
    fn arrival(&self, handle: u32) -> u32 {
        self.arrivals.borrow_mut().remove(&handle).unwrap_or(0)
    }
}

impl Dp8MessageHandler for Dp8Handler {
    /// `Dp8MessageHandler` (0x1003d1b0): RECEIVE, CONNECT_COMPLETE, TERMINATE_SESSION, CREATE_PLAYER
    /// and ENUM_HOSTS_RESPONSE are written to Q1 (`gmid` 0xc350 for RECEIVE, 0xc351 for the rest);
    /// SEND_COMPLETE only when its context's notify byte is set (otherwise the context is freed and
    /// nothing is queued); ASYNC_OP_COMPLETE is NOT queued (its jump-table entry, index 0, lands after
    /// the Q1 `Write` at 0x1003d3bd); RETURN_BUFFER frees GunDll's send buffer and queues nothing;
    /// every other id returns 0. RECEIVE returns `DPNSUCCESS_PENDING` so DP8 keeps the buffer until
    /// `ReturnBuffer`.
    fn message(&self, msg: &Dp8Message) -> Hresult {
        match msg {
            Dp8Message::ReturnBuffer { .. }
            | Dp8Message::Other { .. }
            | Dp8Message::AsyncOpComplete => Hresult::S_OK,
            Dp8Message::SendComplete { context, .. } => {
                if self.notify.borrow_mut().remove(context) {
                    self.q1.borrow_mut().push_back(msg.clone());
                }
                Hresult::S_OK
            }
            Dp8Message::Receive { buffer_handle, .. } => {
                let now = self.now_ms();
                self.arrivals.borrow_mut().insert(*buffer_handle, now);
                self.q1.borrow_mut().push_back(msg.clone());
                DPNSUCCESS_PENDING
            }
            _ => {
                self.q1.borrow_mut().push_back(msg.clone());
                Hresult::S_OK
            }
        }
    }
}

/// What the consumer raises for the connection handler `H` (`FUN_10035ae0` slots 2, 5) and for the
/// application (`H.OnReceive`, 0x10035ca0).
#[derive(Debug, PartialEq, Eq)]
pub enum NetEvent {
    /// `H.OnConnectComplete(hr)` (0x10035b60): `hr` is 0, `ZT_E_FAILED_TO_CONNECT` or `E_ACCESSDENIED`.
    ConnectComplete(Hresult),
    /// `H.OnDisconnected(hr)` (0x10035c00): `hr` is one of the `ZT_E_CONNECTION_*` codes.
    Disconnected(Hresult),
    /// `H.OnReceive` parsed a message.
    Message(GunMsg),
}

/// The connection service `S` (vtable 0x10060f80) with the DP8 client, the raw queue Q1 and the
/// sink object `A` (vtable 0x100599d4) that owns it. `A` and `S` are one value here.
pub struct Transport {
    /// `CoCreateInstance` of the DP8 client, for the reset `FUN_1003d600`.
    factory: Rc<dyn Dp8Factory>,
    /// The handler every (re-created) client is initialised with.
    handler: Rc<dyn Dp8MessageHandler>,
    /// The same handler, for its clock and arrival stamps.
    stamps: Rc<Dp8Handler>,
    /// `S+0xc`: the DP8 client; `None` after a failed re-creation or once `S` is destroyed.
    client: RefCell<Option<Rc<dyn IDirectPlay8Client>>>,
    q1: Rc<RefCell<VecDeque<Dp8Message>>>,
    notify: Rc<RefCell<HashSet<u32>>>,
    next_context: Cell<u32>,
    /// `A+0x20`: a pending connect's async handle. `FUN_1003d6d0` passes no handle slot
    /// (`block+0x30 == NULL`), so nothing sets it on this path and it stays 0; the CONNECT_COMPLETE
    /// sink slot (0x100035d0, `mov [esi+0x10],ebx`) clears it.
    connect_handle: Cell<u32>,
    /// `A+0xe`: set once the teardown `FUN_10003570` runs; the CONNECT_COMPLETE sink slot (0x100035d0,
    /// `cmp [esi-2],bl` at 0x100035fb) then skips the connection-handler callbacks. The RECEIVE slot
    /// (0x100036e0) and the TERMINATE slot (0x10003660) have no such test.
    shutting_down: Cell<bool>,
    /// `A+0xc`: the Q1 drain `A.vtable[1]` (0x10003b40) is running.
    busy: Cell<bool>,
    /// `A+0xd`: a destruction was requested while `busy`; the drain runs it when it ends.
    deferred: Cell<bool>,
    /// `A+4 != NULL`: the connection handler `H` is still linked. The TERMINATE slot clears it
    /// (`mov [esi-0xc],edi` at 0x100036b4); the RECEIVE slot drops datagrams without it
    /// (0x100036e7 `test eax,eax`, 0x1000370d).
    h_linked: Cell<bool>,
    /// `H+8 != NULL`: `H` still has its sender. Slot 1 on a failed connect and slot 7 clear it
    /// (`H+4`, `H+8`, `H+0xc` = 0), after which `SendOnConn` answers `ZT_E_NOT_CONNECTED`.
    sender_linked: Cell<bool>,
    /// `S` is gone (`FUN_10003570` ran to its end and freed `A`).
    destroyed: Cell<bool>,
    /// `S+0x28`: the smoothed round-trip time of the type-0 control frames (`FUN_1003cfc0`).
    latency_avg: Cell<u32>,
    /// The last value `FUN_1003cfc0` handed to sink slot 9 (0x100034b0, a `ret 8` stub).
    latency_sample: Cell<Option<u32>>,
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Transport")
    }
}

/// `FUN_1003d6d0`'s address building. The hostname is `MultiByteToWideChar(CP_ACP, 0, host, -1, buf,
/// 0x40)`: 63 characters at most (open: a longer name makes the call fail and leaves the buffer
/// undefined; the port truncates).
pub fn build_addresses(
    host: &str,
    port: u16,
    local_port: u16,
    probe_port: &mut dyn FnMut() -> Option<u16>,
) -> (Dp8Address, Dp8Address) {
    let wide: Vec<u16> = host.encode_utf16().take(63).collect();
    let host_name = String::from_utf16_lossy(&wide);
    let host_addr = Dp8Address {
        sp: DP8SP_TCPIP,
        components: vec![
            ("hostname".into(), Dp8AddrData::String(host_name)),
            ("port".into(), Dp8AddrData::Dword(port as u32)),
        ],
    };
    let mut device = Dp8Address {
        sp: DP8SP_TCPIP,
        components: Vec::new(),
    };
    if local_port != 0 {
        device
            .components
            .push(("port".into(), Dp8AddrData::Dword(local_port as u32)));
    } else if let Some(p) = probe_port() {
        // the probe: socket(AF_INET, SOCK_DGRAM), bind to port 0, getsockname, closesocket; a port
        // of 0 (or any failure) adds no component
        if p != 0 {
            device
                .components
                .push(("port".into(), Dp8AddrData::Dword(p as u32)));
        }
    }
    // the "device" GUID component is added only if the settings template's flag bytes are non-zero,
    // which GunDll never sets
    (host_addr, device)
}

impl Transport {
    /// `FUN_10003b90` for the connect path: creates the DP8 client (`FUN_1003d480`: `Initialize` with
    /// flags 4, then 0), builds the addresses, sets the keep-alive (`FUN_1003d080`) and calls
    /// `Connect` with the application description. Every failure is `ZT_E_FAILED_TO_CONNECT`
    /// (the function returns it unless the inner result is a success); a `DPNSUCCESS_PENDING` is
    /// success.
    pub fn connect(
        factory: Rc<dyn Dp8Factory>,
        host: &str,
        port: u16,
        app_guid: Guid,
        local_port: u16,
        probe_port: &mut dyn FnMut() -> Option<u16>,
    ) -> Result<Rc<Transport>, Hresult> {
        // open: when CoCreateInstance of the client fails FUN_1003d480 leaves the client NULL and its
        // result is not checked in the decompile; the port fails the connect.
        let client = factory
            .create_client()
            .map_err(|_| ZT_E_FAILED_TO_CONNECT)?;
        let q1 = Rc::new(RefCell::new(VecDeque::new()));
        let notify = Rc::new(RefCell::new(HashSet::new()));
        let stamps = Rc::new(Dp8Handler::new(q1.clone(), notify.clone()));
        let handler: Rc<dyn Dp8MessageHandler> = stamps.clone();
        if !client
            .initialize(handler.clone(), DPNINITIALIZE_DISABLELINKTUNING)
            .succeeded()
            && !client.initialize(handler.clone(), 0).succeeded()
        {
            return Err(ZT_E_FAILED_TO_CONNECT);
        }
        // FUN_1003dbd0 starts with S.vtable[13] (0x1003ccb0): CancelAsyncOperation(0, DPNCANCEL_ENUM)
        let _ = client.cancel_async_operation(0, DPNCANCEL_ENUM);
        let (host_addr, device) = build_addresses(host, port, local_port, probe_port);
        // FUN_1003d080(0x3a98): GetCaps (dwSize 0x14), keep-alive at +0x10, SetCaps, GetCaps
        set_keep_alive(client.as_ref(), KEEP_ALIVE_MS);
        // DPN_APPLICATION_DESC: dwSize 0x48, CLIENT_SERVER, instance GUID zero, application GUID
        let desc = Dp8ApplicationDesc {
            size: 0x48,
            flags: DPNSESSION_CLIENT_SERVER,
            guid_instance: Guid {
                data1: 0,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            },
            guid_application: app_guid,
        };
        let (hr, _handle) = client.connect(&desc, &host_addr, &device);
        let transport = Transport {
            factory,
            handler,
            stamps,
            client: RefCell::new(Some(client)),
            q1,
            notify,
            next_context: Cell::new(1),
            connect_handle: Cell::new(0),
            shutting_down: Cell::new(false),
            busy: Cell::new(false),
            deferred: Cell::new(false),
            h_linked: Cell::new(true),
            sender_linked: Cell::new(true),
            destroyed: Cell::new(false),
            latency_avg: Cell::new(0),
            latency_sample: Cell::new(None),
        };
        if hr != DPNSUCCESS_PENDING && !hr.succeeded() {
            // FUN_1003d6d0's failure branch (0x1003d9f9) calls S.vtable[8] (reset: Close #1,
            // CoCreate, Initialize, injected terminate); FUN_10003b90 then frees `A` raw (0x10003cd5,
            // Q1 is not drained) and runs S's destructor (0x10003ce9: reset = Close #2, then the
            // final Close #3); the result is ZT_E_FAILED_TO_CONNECT.
            let _ = transport.reset();
            let _ = transport.reset();
            if let Some(c) = transport.client.borrow_mut().take() {
                let _ = c.close(0);
            }
            return Err(ZT_E_FAILED_TO_CONNECT);
        }
        Ok(Rc::new(transport))
    }

    /// `S+0x28`, the smoothed round-trip time of the type-0 control frames.
    #[cfg(test)]
    pub(crate) fn latency_average(&self) -> u32 {
        self.latency_avg.get()
    }

    /// The last argument `FUN_1003cfc0` passed to the stub sink slot 9.
    #[cfg(test)]
    pub(crate) fn latency_sample(&self) -> Option<u32> {
        self.latency_sample.get()
    }

    /// `FUN_10003570` has run to its end (`A` freed).
    #[cfg(test)]
    pub(crate) fn is_destroyed(&self) -> bool {
        self.destroyed.get()
    }

    /// The DP8 client `S+0xc`.
    fn client(&self) -> Option<Rc<dyn IDirectPlay8Client>> {
        self.client.borrow().clone()
    }

    /// `FUN_1003d600` (S slot 8, 0x1003d600): with a client, `Close(0)`, Release, `CoCreateInstance` a
    /// new one, `Initialize(flags 4)` and on failure `Initialize(flags 0)`; both failing releases it
    /// (`S+0xc = NULL`). A new client gets a synthetic `TERMINATE_SESSION` with `DPNERR_USERCANCEL`
    /// written into Q1 (`FUN_1003b440`: `{flags 0, gmid 0xc351, param 0xffff0016, cb 0x10,
    /// {0x10, 0x80158650, 0, 0}}`). Without a client it does nothing. A failed `CoCreateInstance`
    /// leaves the client NULL and returns its error (the retail "GunService" trace is not ported).
    fn reset(&self) -> Hresult {
        let Some(old) = self.client.borrow_mut().take() else {
            return Hresult::S_OK;
        };
        let _ = old.close(0);
        drop(old);
        let client = match self.factory.create_client() {
            Ok(c) => c,
            Err(hr) => return hr,
        };
        let mut hr = client.initialize(self.handler.clone(), DPNINITIALIZE_DISABLELINKTUNING);
        if !hr.succeeded() {
            hr = client.initialize(self.handler.clone(), 0);
            if !hr.succeeded() {
                return hr;
            }
        }
        *self.client.borrow_mut() = Some(client);
        self.q1
            .borrow_mut()
            .push_back(Dp8Message::TerminateSession {
                hresult: DPNERR_USERCANCEL,
                data: Vec::new(),
            });
        hr
    }

    /// `A.vtable[0](A, 0)` = `FUN_10003ae0(A, 0)` (0x10003ae0), what `CGun::Disconnect` reaches through
    /// `FUN_100038f0`: a pending connect handle is cancelled (`CancelAsyncOperation(handle,
    /// DPNCANCEL_CONNECT)`, S slot 17 0x1003ccf0), then the DP8 client is reset (`FUN_1003d600`: see
    /// [`Transport::reset`], which injects a `USERCANCEL` terminate into Q1) BEFORE anything is drained;
    /// while the Q1 drain is running (`A+0xc`) the destruction is deferred (`A+0xd`), else
    /// `FUN_10003570` runs ([`Transport::finish`]). Returns what the connection handler `H` raises:
    /// datagrams still in Q1 become messages, and the injected terminate becomes
    /// `Disconnected(ZT_E_CONNECTION_CLIENTDISCONNECT)` because the TERMINATE slot has no
    /// shutting-down test.
    pub fn close(&self) -> Vec<NetEvent> {
        let mut events = Vec::new();
        self.destroy(false, &mut events);
        events
    }

    /// `FUN_10003ae0(A, skip_reset)`. A nonzero `skip_reset` is what the CONNECT_COMPLETE and
    /// TERMINATE sink slots pass (`push 1` at 0x10003650 / 0x100036bd).
    fn destroy(&self, skip_reset: bool, events: &mut Vec<NetEvent>) {
        if self.shutting_down.get() || self.destroyed.get() {
            return;
        }
        if !skip_reset {
            let handle = self.connect_handle.replace(0);
            if handle != 0 {
                if let Some(c) = self.client() {
                    let _ = c.cancel_async_operation(handle, DPNCANCEL_CONNECT);
                }
            }
            let _ = self.reset();
        }
        if self.busy.get() {
            self.deferred.set(true);
            return;
        }
        self.finish(events);
    }

    /// `FUN_10003570` (0x10003570): the shutting-down flag, the Q1 drain through the dispatcher, then
    /// `S`'s destructor (S slot 15 `FUN_1003e160` -> `FUN_1003dfa0`: reset again, `Close(0)`,
    /// Release) and the queue release; `A` is freed.
    fn finish(&self, events: &mut Vec<NetEvent>) {
        self.shutting_down.set(true);
        loop {
            let next = self.q1.borrow_mut().pop_front();
            let Some(msg) = next else {
                break;
            };
            self.dispatch(msg, events);
        }
        let _ = self.reset();
        if let Some(c) = self.client.borrow_mut().take() {
            let _ = c.close(0);
        }
        self.destroyed.set(true);
    }

    /// `SendOnConn` (`FUN_10003880`, 0x10003880) with `FUN_10042ed0` and `FUN_10043070` ->
    /// `FUN_10042c90` -> `FUN_1003c640`: the buffer is `u16 0xC350` followed by the UTF-8 text with
    /// its NUL (the size word the append writes at offset 2 is overwritten by the payload), sent as
    /// one `Send` with flags `NOCOPY | GUARANTEED` (0x9) and timeout 0. `text_len` is the length
    /// `SendRequest` passes, truncated to 16 bits. A total of `max(2 + len, 0x578) + 12 >= 0x10000`
    /// is `0x80070057` (`FUN_10042af0`); a failing `Send` is `ZT_E_NOT_CONNECTED`, `PENDING` is 0.
    /// `H+8 == NULL` (cleared by the failed-connect and TERMINATE slots) is `ZT_E_NOT_CONNECTED`.
    pub fn send_text(&self, text_with_nul: &[u8]) -> Hresult {
        if !self.sender_linked.get() {
            return ZT_E_NOT_CONNECTED;
        }
        let len16 = text_with_nul.len() as u16;
        let total16 = 2u16.wrapping_add(len16);
        if 2 + len16 as u32 > 0xFFFF {
            // open: a UTF-8 length of 0xFFFE or 0xFFFF wraps the 16-bit total to 0 or 1, which passes
            // the size test below (the pool buffer is `max(total, 0x578) + 12` bytes); the payload
            // copy then overflows that block (FUN_10042ed0), and for a total of 0 FUN_10043070 also
            // skips the Send. Retail corrupts memory here; the port stops and rejects the message.
            return Hresult(0x8007_0057);
        }
        if (total16.max(0x578) as u32) + 12 >= 0x1_0000 {
            return Hresult(0x8007_0057);
        }
        let mut buf = Vec::with_capacity(2 + len16 as usize);
        buf.extend_from_slice(&WIRE_TYPE.to_le_bytes());
        buf.extend_from_slice(&text_with_nul[..len16 as usize]);
        let ctx = self.next_context.get();
        self.next_context.set(ctx.wrapping_add(1));
        self.notify.borrow_mut().insert(ctx);
        let Some(client) = self.client() else {
            return ZT_E_NOT_CONNECTED;
        };
        let hr = client.send(&buf, 0, ctx, DPNSEND_NOCOPY | DPNSEND_GUARANTEED);
        if hr == DPNSUCCESS_PENDING {
            Hresult::S_OK
        } else if !hr.succeeded() {
            ZT_E_NOT_CONNECTED
        } else {
            hr
        }
    }

    /// `A.vtable[1]` (0x10003b40): drains Q1 through `Dp8Dispatcher` with the busy flag set and returns
    /// what the connection handler `H` raises. A destruction requested while it runs (`A+0xd`, from
    /// the CONNECT_COMPLETE failure and TERMINATE slots, or a `Disconnect` made from a DP8 callback)
    /// is carried out when the queue is empty (`FUN_10003570`).
    pub fn drain(&self) -> Vec<NetEvent> {
        let mut events = Vec::new();
        if self.destroyed.get() {
            return events;
        }
        self.busy.set(true);
        loop {
            let next = self.q1.borrow_mut().pop_front();
            let Some(msg) = next else {
                break;
            };
            self.dispatch(msg, &mut events);
        }
        self.busy.set(false);
        if self.deferred.replace(false) {
            self.finish(&mut events);
        }
        events
    }

    /// `Dp8Dispatcher` (`FUN_1003c740`).
    fn dispatch(&self, msg: Dp8Message, events: &mut Vec<NetEvent>) {
        match msg {
            Dp8Message::Receive {
                data,
                buffer_handle,
            } => {
                let arrival = self.stamps.arrival(buffer_handle);
                self.dispatch_receive(&data, arrival, events);
                // finishes with ReturnBuffer(hBufferHandle, 0)
                if let Some(c) = self.client() {
                    let _ = c.return_buffer(buffer_handle, 0);
                }
            }
            Dp8Message::ConnectComplete { hresult, reply } => {
                // sink slot 1 (0x100035d0): `A+0x20 = 0`; the reply data is replayed
                // (reply.vtable[0](reply, sink+8, 0)) whatever the flag; with the shutting-down flag
                // (`A+0xe`) that is all. Otherwise the DP8 result is mapped 0 -> 0, 0x80158260 -> 2
                // (E_ACCESSDENIED), other -> 1 (ZT_E_FAILED_TO_CONNECT); H.OnConnectComplete(hr) runs
                // (after `H+4`, `H+8`, `H+0xc` are cleared for a failure) and a failure destroys `A`
                // (`A.vtable[0](A, 1)`, deferred while the drain runs).
                self.connect_handle.set(0);
                if !reply.is_empty() {
                    self.replay_reply(&reply);
                }
                if self.shutting_down.get() {
                    return;
                }
                let hr = if hresult == Hresult::S_OK {
                    Hresult::S_OK
                } else if hresult == DPNERR_HOSTREJECTEDCONNECTION {
                    E_ACCESSDENIED
                } else {
                    ZT_E_FAILED_TO_CONNECT
                };
                let failed = hr != Hresult::S_OK;
                if self.h_linked.get() {
                    if failed {
                        self.sender_linked.set(false);
                    }
                    events.push(NetEvent::ConnectComplete(hr));
                }
                if failed {
                    self.destroy(true, events);
                }
            }
            Dp8Message::TerminateSession { hresult, .. } => {
                // sink slot 7 (0x10003660) has no shutting-down test. 0x80158480 -> 0, 0x80158270 ->
                // 1, 0x80158650 -> 2, else 3 map to 0xC112040E / 040C / 040D / 040F; with `H` linked
                // its links are cleared, H.OnDisconnected(hr) runs and `A+4` is cleared; then `A` is
                // destroyed (`A.vtable[0](A, 1)`).
                let hr = if hresult == DPNERR_NOCONNECTION {
                    ZT_E_CONNECTION_DROPPED_GONE
                } else if hresult == DPNERR_HOSTTERMINATEDSESSION {
                    ZT_E_CONNECTION_SERVERBOOT
                } else if hresult == DPNERR_USERCANCEL {
                    ZT_E_CONNECTION_CLIENTDISCONNECT
                } else {
                    ZT_E_CONNECTION_DROPPED_LOST
                };
                if self.h_linked.replace(false) {
                    self.sender_linked.set(false);
                    events.push(NetEvent::Disconnected(hr));
                }
                self.destroy(true, events);
            }
            // SEND_COMPLETE: sink slot 6 is a stub and the context is freed; ASYNC_OP_COMPLETE,
            // CREATE_PLAYER, ENUM_HOSTS_RESPONSE (not on the connect path) have no case
            _ => {}
        }
    }

    /// Sink slot 1's `reply.vtable[0](reply, sink+8, 0)` for a CONNECT_COMPLETE with reply data (the
    /// object `FUN_10043230` builds over the data). GunDll sends no connect data, so a GUN server is
    /// not expected to reply with any.
    // open: what the reply object does with its frames is not decoded; the call shape is kept and the
    // data is not interpreted.
    fn replay_reply(&self, _reply: &[u8]) {}

    /// The RECEIVE arm (asm 0x1003c778-0x1003c87f): the position is a 32-bit sum (`edx`), the frame
    /// address and the loop test use its low 16 bits (`movzx eax,dx`, `cmp dx,cx`) against the 16-bit
    /// length, the remaining length is `length - position` (`sub ecx,edx`). Types below 1000 are
    /// control frames ([`Transport::control_frame`]); 999 would go to a filter object that nothing
    /// installs, so it is handled like an application frame; an application frame passes the rest of
    /// the datagram to `H.OnReceive` and consumes it (sink slot 0, 0x100036e0, which drops the data
    /// when `H` is unlinked, `A+4 == NULL`).
    fn dispatch_receive(&self, data: &[u8], arrival: u32, events: &mut Vec<NetEvent>) {
        // the length is truncated to 16 bits (`mov cx, [edi+0x10]`)
        let len = data.len() as u16;
        let mut pos: u32 = 0;
        // retail has no bound; the port stops a loop that would otherwise spin forever
        let mut steps = 0u32;
        while (pos as u16) < len && steps < 0x2_0000 {
            steps += 1;
            let at = (pos as u16) as usize;
            // `sub ecx,edx`: 32-bit; the sink takes its low 16 bits (`mov si,[esp+0x18]`)
            let remaining = (len as u32).wrapping_sub(pos) as u16;
            let Some(ty) = read_u16(data, at) else {
                break;
            };
            let consumed: u32 = if ty < APP_TYPE_MIN && ty != 999 {
                match ty {
                    0 | 1 => self.control_frame(data, at, arrival),
                    // retail returns 0 consumed here and spins forever on the datagram; the port
                    // drops it (deviation: GunDll never terminates this loop)
                    _ => break,
                }
            } else {
                // sink slot 0 (0x100036e0): H.OnReceive(ptr + 2, remaining - 2), returns `remaining`
                // (`mov ax,si`; the upper half of eax is the callee's leftover, never used: only
                // the low 16 bits of the position are)
                let end = (at + remaining as usize).min(data.len());
                if self.h_linked.get() && remaining >= 2 && at + 2 <= end {
                    if let Some(msg) = on_receive(&data[at + 2..end]) {
                        events.push(NetEvent::Message(msg));
                    }
                }
                remaining as u32
            };
            if consumed & 0xFFFF == 0 {
                // a control frame whose size is a multiple of 0x10000 leaves the low 16 bits of the
                // position unchanged: retail spins here
                break;
            }
            pos = pos.wrapping_add(consumed);
        }
    }

    /// `FUN_1003b820` (0x1003b820) for a frame at `at`: type 0 calls `FUN_1003b4a0` and is 8 bytes;
    /// type 1 calls `FUN_1003b4f0` and is `((u16[+6] + u16[+4]) & 0xFFFF) + 8` bytes, the 16-bit sum
    /// zero-extended and the 8 added in 32 bits (asm 0x1003b849-0x1003b852).
    fn control_frame(&self, data: &[u8], at: usize, arrival: u32) -> u32 {
        let a = read_u16(data, at + 4).unwrap_or(0);
        let b = read_u16(data, at + 6).unwrap_or(0);
        if read_u16(data, at) == Some(0) {
            self.latency_frame(data, at, arrival);
            8
        } else {
            self.keep_alive_frame(data, at);
            a.wrapping_add(b) as u32 + 8
        }
    }

    /// `FUN_1003b4a0` (0x1003b4a0) -> `FUN_1003cfc0` (0x1003cfc0), the type-0 frame:
    /// `u32[+4] - arrival + timeGetTime()` (`arrival` is the time the DP8 handler stamped on the
    /// datagram, `[Q1 block + 4]`) is the argument; `S.client.GetConnectionInfo(info{size 0x5c}, 0)`;
    /// on success `S+0x28 = (S+0x28 + dwRoundTripLatencyMS) >> 1` and the sink's slot 9 (0x100034b0,
    /// a `ret 8` stub) gets `(S+0x28 >> 1) + argument`.
    fn latency_frame(&self, data: &[u8], at: usize, arrival: u32) {
        let stamp = read_u32(data, at + 4).unwrap_or(0);
        let now = self.stamps.now_ms();
        let arg = stamp.wrapping_sub(arrival).wrapping_add(now);
        let Some(client) = self.client() else {
            return;
        };
        let mut info = Dp8ConnectionInfo {
            size: 0x5c,
            ..Default::default()
        };
        if client.get_connection_info(&mut info, 0).succeeded() {
            let avg = self
                .latency_avg
                .get()
                .wrapping_add(info.round_trip_latency_ms)
                >> 1;
            self.latency_avg.set(avg);
            self.latency_sample.set(Some((avg >> 1).wrapping_add(arg)));
        }
    }

    /// `FUN_1003b4f0` (0x1003b4f0) -> `FUN_1003da50` (0x1003da50), the type-1 frame: subtype byte
    /// `[+2]` 1 sets the keep-alive to `0xFFFFFFFF` (`FUN_1003d080`), 2 to 15000; any subtype then
    /// builds the payload object (`u16[+6]` bytes at `+8 + u16[+4]`, none when that is 0) and
    /// passes `(subtype, frame + 8, object)` to the sink's slot 8 (0x100034a0, a `ret 0x10` stub);
    /// the object is never freed.
    // open: `FUN_1003da50` also keeps an elapsed-time double at S+0x50 / S+0x30 (`FUN_1003c510`,
    // QueryPerformanceCounter); the only other reader found is the getter at 0x1003e2c0, not on a
    // CGun path, so it is not modelled.
    fn keep_alive_frame(&self, data: &[u8], at: usize) {
        let subtype = data.get(at + 2).copied().unwrap_or(0);
        let keep_alive = match subtype {
            1 => Some(0xFFFF_FFFF),
            2 => Some(KEEP_ALIVE_MS),
            _ => None,
        };
        if let (Some(ms), Some(client)) = (keep_alive, self.client()) {
            set_keep_alive(client.as_ref(), ms);
        }
    }
}

/// `FUN_1003d080(S, ms)` (0x1003d080): `GetCaps` (`dwSize` 0x14), `dwTimeoutUntilKeepAlive` (+0x10) =
/// `ms`, `SetCaps`, `GetCaps`.
fn set_keep_alive(client: &dyn IDirectPlay8Client, ms: u32) {
    let mut caps = Dp8Caps {
        size: 0x14,
        ..Default::default()
    };
    let _ = client.get_caps(&mut caps, 0);
    caps.timeout_until_keep_alive = ms;
    let _ = client.set_caps(&caps, 0);
    let _ = client.get_caps(&mut caps, 0);
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *data.get(at)?,
        *data.get(at + 1)?,
        *data.get(at + 2)?,
        *data.get(at + 3)?,
    ]))
}

fn read_u16(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*data.get(at)?, *data.get(at + 1)?]))
}

/// `H.OnReceive(data, len)` (0x10035ca0): `memchr(data, 0, len)`; none is `E_INVALIDARG` (dropped);
/// otherwise the text is cut after the first NUL and `FUN_10009ad0` parses it. A parse failure
/// drops the packet (the caller ignores the HRESULT).
pub fn on_receive(data: &[u8]) -> Option<GunMsg> {
    let nul = data.iter().position(|&b| b == 0)?;
    parse_message(&data[..=nul]).ok()
}
