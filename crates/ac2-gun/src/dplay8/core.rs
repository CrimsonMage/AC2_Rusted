//! The DirectPlay core session layer for a client (DPL8CS), over the reliable protocol:
//! `CONNECT` -> 0xC1 -> 0xC2 -> `CONNECT_COMPLETE` -> 0xC3, user data, 0xE0/0xE1, 0xC5, 0xDF,
//! and what a lost or ended connection means to the application.
//!
//! Functions are dpnet.dll's, by address: `DnCore_SendConnectRequest` 0x0040c921,
//! `DnCore_BuildConnectRequest` 0x0040c6c7, `DnCore_DispatchPacket` 0x0040489f, the 0xC2
//! processing `FUN_0040ca05`, `DnCore_OnConnectFailed` 0x0040bb24, `DnCore_OnTerminateSession`
//! 0x00403de6, `FUN_004106c6` (the core's receive callback), `FUN_00410478` (the connection
//! lost callback), `FUN_0040a0c8` (Send).

use std::collections::{BTreeMap, HashMap};

use crate::Hresult;

use super::connect_info::*;
use super::hresult::*;
use super::messages::*;
use super::protocol::{Event, OpId, Protocol};
use super::protocol::{Priority, SendMode};

/// The core's states, from the flag word the checks test (dpnet.dll.c:7715-7726, 8568-8585):
/// `0x80000000` initialised, `0x800000` connecting, `0x400000` connected, `0x300000` closing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Uninitialized,
    Idle,
    Connecting,
    Connected,
    Closing,
}

/// Public `DPNSEND_*` flags (dplay8.h:283-291).
pub mod send_flags {
    pub const NOCOPY: u32 = 0x0000_0001;
    pub const NOCOMPLETE: u32 = 0x0000_0002;
    pub const COMPLETEONPROCESS: u32 = 0x0000_0004;
    pub const GUARANTEED: u32 = 0x0000_0008;
    pub const NONSEQUENTIAL: u32 = 0x0000_0010;
    pub const NOLOOPBACK: u32 = 0x0000_0020;
    pub const PRIORITY_LOW: u32 = 0x0000_0040;
    pub const PRIORITY_HIGH: u32 = 0x0000_0080;
    /// DX9 only: the message may be coalesced (`FUN_00409898` passes 0x100 through, .c:5823-5835).
    pub const COALESCE: u32 = 0x0000_0100;
    pub const SYNC: u32 = 0x8000_0000;
}

/// Something the handler must be told about, in order.
#[derive(Debug, Clone)]
pub(crate) enum Indication {
    ConnectComplete(ConnectComplete),
    Receive {
        data: Vec<u8>,
        buffer: DpnHandle,
        e0_context: Option<u32>,
    },
    SendComplete(SendComplete),
    TerminateSession(TerminateSession),
}

#[derive(Debug)]
struct ConnectOp {
    handle: DpnHandle,
    context: usize,
    c1_op: OpId,
    c1_done: bool,
    reply: Option<Vec<u8>>,
    failure: Option<(Hresult, Vec<u8>)>,
    completed: bool,
}

#[derive(Debug)]
struct SendOp {
    handle: DpnHandle,
    context: usize,
    start: u32,
    notify: bool,
    /// `DPNSEND_COMPLETEONPROCESS`: finished by the host's 0xE1, not by the ack.
    wait_e1: bool,
    acked: bool,
    e1: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Internal {
    ConnectInfo,
    AckConnectInfo,
    ProcessCompletion,
}

pub(crate) struct Core {
    pub state: CoreState,
    next_handle: u32,
    next_op: OpId,
    connect: Option<ConnectOp>,
    c1_body: Vec<u8>,
    sends: BTreeMap<OpId, SendOp>,
    by_handle: HashMap<DpnHandle, OpId>,
    internal: HashMap<OpId, Internal>,
    pub(crate) buffers: HashMap<DpnHandle, Option<u32>>,
    peer_ended: bool,
}

impl Core {
    pub fn new() -> Self {
        Self {
            state: CoreState::Uninitialized,
            next_handle: 1,
            next_op: 1,
            connect: None,
            c1_body: Vec::new(),
            sends: BTreeMap::new(),
            by_handle: HashMap::new(),
            internal: HashMap::new(),
            buffers: HashMap::new(),
            peer_ended: false,
        }
    }

    pub fn new_handle(&mut self) -> DpnHandle {
        let h = DpnHandle(self.next_handle);
        self.next_handle += 1;
        h
    }

    fn new_op(&mut self) -> OpId {
        let o = self.next_op;
        self.next_op += 1;
        o
    }

    /// Start a connect: remember the 0xC1 body and the op.
    pub fn begin_connect(&mut self, handle: DpnHandle, context: usize, c1_body: Vec<u8>) {
        self.state = CoreState::Connecting;
        self.peer_ended = false;
        self.c1_body = c1_body;
        self.connect = Some(ConnectOp {
            handle,
            context,
            c1_op: 0,
            c1_done: false,
            reply: None,
            failure: None,
            completed: false,
        });
    }

    /// The core sends a system message: reliable, sequential, normal priority, USER_1
    /// (`DnCore_SendPacket` with `param_8 = 1`).
    fn send_system(
        &mut self,
        proto: &mut Protocol,
        now: u32,
        ty: u32,
        body: &[u8],
        kind: Internal,
    ) -> Option<OpId> {
        let op = self.new_op();
        let mode = SendMode {
            reliable: true,
            nonsequential: false,
            priority: Priority::Normal,
            user_bits: 0x40,
        };
        match proto.queue_message(now, op, &with_type(ty, body), mode) {
            Ok(()) => {
                self.internal.insert(op, kind);
                Some(op)
            }
            Err(_) => None,
        }
    }

    /// The flag checks of the DX9 Send validator (`FUN_00419d26`, dpnet9.dll.c:21222-21244):
    /// only bits 0x1ff and 0x80000000 are valid; NOCOPY with NOCOMPLETE, COMPLETEONPROCESS
    /// with NOCOMPLETE or without GUARANTEED, GUARANTEED with NOCOMPLETE unless SYNC, and
    /// PRIORITY_HIGH with PRIORITY_LOW are `DPNERR_INVALIDFLAGS`.
    pub fn validate_send_flags(flags: u32) -> Result<(), Hresult> {
        if flags & 0x7fff_fe00 != 0 {
            return Err(DPNERR_INVALIDFLAGS);
        }
        let bad = (flags & send_flags::NOCOPY != 0 && flags & send_flags::NOCOMPLETE != 0)
            || (flags & send_flags::COMPLETEONPROCESS != 0
                && (flags & send_flags::NOCOMPLETE != 0 || flags & send_flags::GUARANTEED == 0))
            || (flags & send_flags::GUARANTEED != 0
                && flags & send_flags::NOCOMPLETE != 0
                && flags & send_flags::SYNC == 0)
            || (flags & send_flags::PRIORITY_LOW != 0 && flags & send_flags::PRIORITY_HIGH != 0);
        if bad {
            Err(DPNERR_INVALIDFLAGS)
        } else {
            Ok(())
        }
    }

    /// `Send` (dpnet 0x0040a0c8, dpnet.dll.c:7664). The buffers are joined into one message;
    /// flags map as dpnet.dll.c:7785-7794 (GUARANTEED -> reliable, NONSEQUENTIAL, HIGH/LOW
    /// priority). `COMPLETEONPROCESS` wraps the payload in 0xE0 (`FUN_00403b46`).
    pub fn send(
        &mut self,
        proto: &mut Protocol,
        now: u32,
        data: Vec<u8>,
        context: usize,
        flags: u32,
    ) -> Result<DpnHandle, Hresult> {
        match self.state {
            CoreState::Uninitialized => return Err(DPNERR_UNINITIALIZED),
            CoreState::Connecting => return Err(DPNERR_CONNECTING),
            CoreState::Connected => {}
            _ => return Err(DPNERR_NOCONNECTION),
        }
        if flags & send_flags::SYNC != 0 {
            // open: DPNSEND_SYNC (the call blocks until SEND_COMPLETE) is not ported; GunDll
            // sends asynchronously (flags 0x9).
            return Err(DPNERR_INVALIDFLAGS);
        }
        let priority = if flags & send_flags::PRIORITY_HIGH != 0 {
            Priority::High
        } else if flags & send_flags::PRIORITY_LOW != 0 {
            Priority::Low
        } else {
            Priority::Normal
        };
        let handle = self.new_handle();
        let op = self.new_op();
        let e0 = flags & send_flags::COMPLETEONPROCESS != 0;
        let (payload, user_bits) = if e0 {
            // 0xE0: type, the op handle as context, then the payload (FUN_00403b46); the item
            // flags are 0x41: reliable + USER_1.
            let mut body = handle.0.to_le_bytes().to_vec();
            body.extend_from_slice(&data);
            (with_type(TYPE_REQ_PROCESS_COMPLETION, &body), 0x40)
        } else {
            (data, 0)
        };
        let mode = SendMode {
            reliable: flags & send_flags::GUARANTEED != 0 || e0,
            nonsequential: flags & send_flags::NONSEQUENTIAL != 0 && !e0,
            priority,
            user_bits,
        };
        proto.queue_message(now, op, &payload, mode)?;
        self.sends.insert(
            op,
            SendOp {
                handle,
                context,
                start: now,
                notify: flags & send_flags::NOCOMPLETE == 0,
                wait_e1: e0,
                acked: false,
                e1: false,
            },
        );
        self.by_handle.insert(handle, op);
        Ok(handle)
    }

    /// `FUN_0041cba0`/`FUN_0041cc10` for a user send (async handle): cancel while no frame of it
    /// has gone out.
    pub fn cancel_send(&mut self, proto: &mut Protocol, now: u32, handle: DpnHandle) -> Hresult {
        let Some(&op) = self.by_handle.get(&handle) else {
            return DPNERR_INVALIDHANDLE;
        };
        match proto.cancel_message(now, op, DPNERR_USERCANCEL) {
            Ok(()) => Hresult(0),
            Err(e) => e,
        }
    }

    /// `ReturnBuffer`: give a pending RECEIVE buffer back; a 0xE0 request is answered with 0xE1.
    pub fn return_buffer(&mut self, proto: &mut Protocol, now: u32, handle: DpnHandle) -> Hresult {
        match self.buffers.remove(&handle) {
            None => DPNERR_INVALIDHANDLE,
            Some(ctx) => {
                if let Some(ctx) = ctx {
                    self.send_system(
                        proto,
                        now,
                        TYPE_PROCESS_COMPLETION,
                        &ctx.to_le_bytes(),
                        Internal::ProcessCompletion,
                    );
                }
                Hresult(0)
            }
        }
    }

    /// A 0xE0 request whose buffer the handler did not keep is answered at once (the implicit
    /// `ReturnBuffer`).
    pub fn answer_e0(&mut self, proto: &mut Protocol, now: u32, ctx: u32) {
        self.send_system(
            proto,
            now,
            TYPE_PROCESS_COMPLETION,
            &ctx.to_le_bytes(),
            Internal::ProcessCompletion,
        );
    }

    /// Cancel everything that has not started (the first part of `Close`).
    pub fn cancel_unstarted(&mut self, proto: &mut Protocol, now: u32) {
        let ops: Vec<DpnHandle> = self.sends.values().map(|s| s.handle).collect();
        for h in ops {
            let _ = self.cancel_send(proto, now, h);
        }
    }

    /// Fold one protocol event into handler messages.
    pub fn on_event(
        &mut self,
        proto: &mut Protocol,
        now: u32,
        ev: Event,
        out: &mut Vec<Indication>,
    ) {
        match ev {
            Event::ConnectResult(Ok(())) => {
                let body = self.c1_body.clone();
                // `DnCore_SendConnectRequest`: 0xC1, reliable.
                let op = self.send_system(
                    proto,
                    now,
                    TYPE_PLAYER_CONNECT_INFO,
                    &body,
                    Internal::ConnectInfo,
                );
                match (op, self.connect.as_mut()) {
                    (Some(op), Some(c)) => c.c1_op = op,
                    _ => self.fail_connect(DPNERR_GENERIC, Vec::new(), out),
                }
            }
            Event::ConnectResult(Err(e)) => self.fail_connect(e, Vec::new(), out),
            Event::Receive { data, flags } => self.on_receive(proto, now, data, flags, out),
            Event::PeerEndStream => {
                // FUN_004103e3: the peer's END_STREAM; later data is dropped
                // (`FUN_004106c6` only passes connections in state 1 or 2).
                self.peer_ended = true;
            }
            Event::ConnectionLost(_) => self.on_connection_lost(out),
            Event::SendComplete { op, result } => self.on_send_complete(op, result, now, out),
            Event::DisconnectComplete { .. } => {}
        }
    }

    /// `FUN_00410478`, client branch (dpnet.dll.c:13395-13405): a connected core tells the
    /// application `TERMINATE_SESSION` with `DPNERR_CONNECTIONLOST` (the constant `0x80158160`
    /// whatever the protocol's code was); then the session state is dropped (`FUN_0040afef`).
    fn on_connection_lost(&mut self, out: &mut Vec<Indication>) {
        match self.state {
            CoreState::Connected => {
                self.state = CoreState::Idle;
                out.push(Indication::TerminateSession(TerminateSession {
                    result: DPNERR_CONNECTIONLOST,
                    data: Vec::new(),
                }));
            }
            CoreState::Connecting => {
                // open: the connect op's result when the connection dies between the protocol
                // connect and 0xC2 (the object is in state 1) is set by FUN_0040afef; the
                // port reports DPNERR_CONNECTIONLOST.
                self.fail_connect(DPNERR_CONNECTIONLOST, Vec::new(), out);
            }
            _ => {}
        }
    }

    fn fail_connect(&mut self, hr: Hresult, reply: Vec<u8>, out: &mut Vec<Indication>) {
        if let Some(c) = self.connect.as_mut() {
            if c.completed {
                return;
            }
            c.failure = Some((hr, reply));
        }
        self.state = CoreState::Idle;
        self.try_complete_connect(out);
    }

    /// The connect op finishes when its references are gone (`FUN_00404f3b`): the 0xC1 send is
    /// done and either 0xC2 or a failure was seen. A failure that came from the protocol itself
    /// (no 0xC1 was ever sent) completes at once.
    fn try_complete_connect(&mut self, out: &mut Vec<Indication>) {
        let Some(c) = self.connect.as_mut() else {
            return;
        };
        if c.completed {
            return;
        }
        let c1_pending = c.c1_op != 0 && !c.c1_done;
        if let Some((hr, reply)) = c.failure.clone() {
            if c1_pending && hr != DPNERR_CONNECTIONLOST {
                return;
            }
            c.completed = true;
            out.push(Indication::ConnectComplete(ConnectComplete {
                async_op: c.handle,
                user_context: c.context,
                result: hr,
                reply_data: reply,
            }));
        } else if let (Some(reply), true) = (c.reply.clone(), c.c1_done) {
            c.completed = true;
            self.state = CoreState::Connected;
            out.push(Indication::ConnectComplete(ConnectComplete {
                async_op: c.handle,
                user_context: c.context,
                result: Hresult(0),
                reply_data: reply,
            }));
        }
    }

    fn on_send_complete(&mut self, op: OpId, result: Hresult, now: u32, out: &mut Vec<Indication>) {
        if let Some(kind) = self.internal.remove(&op) {
            match kind {
                Internal::ConnectInfo => {
                    if let Some(c) = self.connect.as_mut() {
                        c.c1_done = true;
                    }
                    if result.0 != 0 {
                        self.fail_connect(result, Vec::new(), out);
                    } else {
                        self.try_complete_connect(out);
                    }
                }
                Internal::AckConnectInfo | Internal::ProcessCompletion => {}
            }
            return;
        }
        let Some(s) = self.sends.get_mut(&op) else {
            return;
        };
        s.acked = true;
        let finished = result.0 != 0 || !s.wait_e1 || s.e1;
        if finished {
            self.finish_send(op, result, now, out);
        }
    }

    fn finish_send(&mut self, op: OpId, result: Hresult, now: u32, out: &mut Vec<Indication>) {
        if let Some(s) = self.sends.remove(&op) {
            self.by_handle.remove(&s.handle);
            if s.notify {
                out.push(Indication::SendComplete(SendComplete {
                    async_op: s.handle,
                    user_context: s.context,
                    result,
                    send_time: now.wrapping_sub(s.start),
                }));
            }
        }
    }

    /// `FUN_004106c6` (the core's receive callback, dpnet.dll.c:13413) and
    /// `DnCore_DispatchPacket`.
    fn on_receive(
        &mut self,
        proto: &mut Protocol,
        now: u32,
        data: Vec<u8>,
        flags: u8,
        out: &mut Vec<Indication>,
    ) {
        if self.peer_ended {
            return;
        }
        if flags & 0x40 != 0 && flags & 0x80 == 0 {
            if data.len() < 4 {
                return;
            }
            let ty = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let body = &data[4..];
            match ty {
                TYPE_SEND_CONNECT_INFO => self.on_connect_info(proto, now, body, out),
                TYPE_CONNECT_FAILED => {
                    if let Some((hr, reply)) = parse_connect_failed(body) {
                        // `DnCore_OnConnectFailed`: the op takes the host's result and reply.
                        self.fail_connect(hr, reply, out);
                    }
                }
                TYPE_TERMINATE_SESSION => {
                    // `DnCore_OnTerminateSession`: TERMINATE_SESSION(HOSTTERMINATEDSESSION,
                    // data), then the session state is dropped (`FUN_0040afef`).
                    let d = parse_terminate_session(body);
                    if self.state == CoreState::Connected {
                        out.push(Indication::TerminateSession(TerminateSession {
                            result: DPNERR_HOSTTERMINATEDSESSION,
                            data: d,
                        }));
                    }
                    self.state = CoreState::Idle;
                }
                TYPE_REQ_PROCESS_COMPLETION => {
                    // `FUN_00403d58`: the payload after the context is delivered as a RECEIVE;
                    // `ReturnBuffer` sends 0xE1 {context}.
                    if body.len() >= 4 && self.state == CoreState::Connected {
                        let ctx = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
                        let h = self.new_handle();
                        self.buffers.insert(h, Some(ctx));
                        out.push(Indication::Receive {
                            data: body[4..].to_vec(),
                            buffer: h,
                            e0_context: Some(ctx),
                        });
                    }
                }
                TYPE_PROCESS_COMPLETION if body.len() >= 4 => {
                    // `FUN_00403d86`: complete the send whose handle the host echoes.
                    let h = DpnHandle(u32::from_le_bytes([body[0], body[1], body[2], body[3]]));
                    let op = self.by_handle.get(&h).copied();
                    if let Some(s) = op.and_then(|op| self.sends.get_mut(&op)) {
                        s.e1 = true;
                        if s.acked {
                            self.finish_send(op.unwrap(), Hresult(0), now, out);
                        }
                    }
                }
                _ => {}
            }
        } else if flags & 0x80 == 0 && self.state == CoreState::Connected {
            let h = self.new_handle();
            self.buffers.insert(h, None);
            out.push(Indication::Receive {
                data,
                buffer: h,
                e0_context: None,
            });
        }
        // open: USER_2 (voice) frames are dropped; the DX8 client path for them (FUN_004161d5
        // with flags & 0x80) was not traced.
    }

    /// `FUN_0040ca05` for a client: 0xC2 arrived. The application reply data is read, 0xC3 is
    /// sent, and CONNECT_COMPLETE is raised once the 0xC1 send is complete too.
    // open: the order of CONNECT_COMPLETE relative to the 0xC3 send follows dpnet.dll.c:9998-10046
    // (`FUN_00411cae` releases the connect op before `DnCore_SendPacket(.., 0xc3, ..)`); with the
    // 0xC1 send still unacknowledged the completion waits for it (reference count) and 0xC3
    // goes first.
    fn on_connect_info(
        &mut self,
        proto: &mut Protocol,
        now: u32,
        body: &[u8],
        out: &mut Vec<Indication>,
    ) {
        if self.state != CoreState::Connecting {
            return;
        }
        let Some(reply) = parse_send_connect_info(body) else {
            return;
        };
        let ready = match self.connect.as_mut() {
            Some(c) => {
                c.reply = Some(reply);
                c.c1_done
            }
            None => return,
        };
        if ready {
            self.try_complete_connect(out);
            self.send_system(
                proto,
                now,
                TYPE_ACK_CONNECT_INFO,
                &[],
                Internal::AckConnectInfo,
            );
        } else {
            self.send_system(
                proto,
                now,
                TYPE_ACK_CONNECT_INFO,
                &[],
                Internal::AckConnectInfo,
            );
        }
    }
}
