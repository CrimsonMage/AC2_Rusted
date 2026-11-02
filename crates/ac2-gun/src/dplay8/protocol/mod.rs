//! The DPL8R reliable protocol layer of dpnet.dll (DirectX 8.0, 5.1.2258.400), client side: one
//! connection to one host. Every item cites the function it ports by address; the role names are
//! the Phase 0 names in `decompiled-pdb/dplay8/naming_pass.csv` (`Dpl8r_*`), `FUN_xxxxxxxx`
//! where the binary has none. Line numbers refer to the current
//! `decompiled-pdb/dplay8/dpnet/dpnet.dll.c`.
//!
//! The retail objects (a protocol-data object `pd`, an SP-data object `spd`, one connection
//! object `conn` with about 0x210 bytes of fields) are one [`Protocol`] here. Conn fields keep
//! their retail offsets in comments as `[0xNN]` (a dword index into the conn) or `+0xNNN` (a
//! byte offset). The `conn[4]` flag word is [`Conn::flags`] with the same bit values.
//!
//! Retail runs this code on an I/O-completion thread and timer threads under critical
//! sections; the port is single-threaded and everything happens inside the pump, in the order
//! the retail queues give (see [`timers`]).

pub mod frames;
pub mod signing;
pub mod timers;

mod ack;
mod congestion;
mod connect;
mod recv;
mod send;
pub use send::{Priority, SendMode};
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use crate::Hresult;

use timers::Timers;

/// `conn[4]` flag bits, as the decompile uses them.
pub mod flag {
    /// `0x2`: set by the allocator (0x00420900 starts at 0x43) and cleared by
    /// `Dpl8r_SendConnect`; while set a connector refuses CONNECTED (0x0041f340).
    pub const NEW: u32 = 0x2;
    /// `0x4`: a connect is under way (`Dpl8r_SendConnect` sets 0x14).
    pub const CONNECTING: u32 = 0x4;
    /// `0x8`: established; DFRAMEs are accepted (`Dpl8r_OnConnected` sets 0x208).
    pub const CONNECTED: u32 = 0x8;
    /// `0x10`: we initiated the connection.
    pub const INITIATOR: u32 = 0x10;
    /// `0x20`: the connection is going away (`Dpl8r_TearDownConnection`, 0x00421ed0).
    pub const DYING: u32 = 0x20;
    /// `0x40`: `Dpl8r_DeliverReceivedFrame` is running.
    pub const DELIVERING: u32 = 0x40;
    /// `0x100`: the adaptive-rate probe is idle (`FUN_00421160` sets it).
    pub const CC_IDLE: u32 = 0x100;
    /// `0x200`: the send window has room.
    pub const WINDOW_OPEN: u32 = 0x200;
    /// `0x400`: messages are queued to send.
    pub const DATA_QUEUED: u32 = 0x400;
    /// `0x800`: the send pump is scheduled.
    pub const PUMP_SCHEDULED: u32 = 0x800;
    /// `0x1000`: the next CONNECTED built answers with POLL (listener only).
    pub const POLL_NEXT: u32 = 0x1000;
    /// `0x2000`: a send mask is pending.
    pub const SENDMASK_PENDING: u32 = 0x2000;
    /// `0x4000`: a SACK mask is pending (out-of-order frames held).
    pub const SACK_PENDING: u32 = 0x4000;
    /// `0x8000`: an acknowledgement is owed.
    pub const ACK_PENDING: u32 = 0x8000;
    /// `0x10000`: a KeepAlive is queued.
    pub const KEEPALIVE_PENDING: u32 = 0x10000;
    /// `0x20000`: our END_STREAM is queued.
    pub const END_STREAM_QUEUED: u32 = 0x20000;
    /// `0x40000`: the peer's END_STREAM was received.
    pub const PEER_END_STREAM: u32 = 0x40000;
    /// `0x80000`: our END_STREAM was acknowledged.
    pub const END_STREAM_ACKED: u32 = 0x80000;
    /// `0x100000`: send completions are waiting (`FUN_004239b0`).
    pub const COMPLETIONS_PENDING: u32 = 0x100000;
    /// `0x200000`: the byte window is full.
    pub const BYTE_WINDOW_FULL: u32 = 0x200000;
    /// `0x400000`: the frame window is full.
    pub const FRAME_WINDOW_FULL: u32 = 0x400000;
    /// `0x800000`: set at establishment by `FUN_00420ae0`, never cleared anywhere in dpnet
    /// (verify_spec_client.md 2.5): POLL'd in-order frames always take the 4 ms delayed ACK.
    pub const FAST_ACK: u32 = 0x800000;
    /// `0x1000000`: the pump went idle.
    pub const PUMP_IDLE: u32 = 0x1000000;
    /// `0x2000000`: frames wait in the resend list.
    pub const RESEND_PENDING: u32 = 0x2000000;
    /// `0x4000000`: the congestion probe is in its backed-off state.
    pub const CC_BACKOFF: u32 = 0x4000000;
    /// `0x8000000`: `FUN_00421340` second-look flag.
    pub const CC_SECOND: u32 = 0x8000000;
    /// `0x10000000`: the loss was already reported to the core.
    pub const NOTIFIED: u32 = 0x1000_0000;
    /// `0x20000000`: flow-control toggle (`FUN_0041dc40` case 6).
    pub const CC_TOGGLE: u32 = 0x2000_0000;
    /// `0x40000000`: dead (version mismatch / forced teardown).
    pub const FORCED_DEAD: u32 = 0x4000_0000;
    /// `0x80000000`: released (`FUN_00420e50`).
    pub const RELEASED: u32 = 0x8000_0000;
}

/// The settings `Dpl8r_InitProtocolData` (0x00420220) and the SP's `GetCaps` give the protocol.
#[derive(Debug, Clone, Copy)]
pub struct ProtocolConfig {
    /// `pd[5]`: ms of silence before a KeepAlive; default 60000, GunDll sets 15000
    /// (`GetCaps`/`SetCaps` slot `dwTimeoutUntilKeepAlive`).
    pub keepalive_timeout: u32,
    /// `pd[6]`: first CONNECT retry interval, 200 ms.
    pub connect_timeout: u32,
    /// `pd[7]`: CONNECT retries, 8.
    pub connect_retries: u32,
    /// `spd[6]` = `SpWsock_GetCaps` (dpnwsock 0x0040cb48) `+8` = 0x5c0 = 1472: the largest
    /// datagram.
    pub sp_max_datagram: u32,
    /// Initialize flag 4 (`DPNINITIALIZE_DISABLELINKTUNING`): `pd[0] |= 4`, then every conn has
    /// `conn[5] |= 0x400` and a fixed send window of 63 frames (DX9 0x00423298:30041-30057,
    /// 0x00423765:30292-30310).
    pub link_tuning_off: bool,
    /// Initialize flag 2 (LAN hint): the initial send window is 32 frames with tuning on
    /// (`pd+0x44 = 0x20`, FUN_00423765:30292).
    pub lan_hint: bool,
    /// `DPN_CAPS_EX` fields (DX9 `pd+0x1c..+0x40`, defaults FUN_00423765:30294-30304). They are
    /// stored and reported; the protocol still runs on the equal literals.
    pub max_recv_msg_size: u32,
    pub num_send_retries: u32,
    pub max_send_retry_interval: u32,
    pub drop_threshold_rate: u32,
    pub throttle_rate: u32,
    pub num_hard_disconnect_sends: u32,
    pub max_hard_disconnect_period: u32,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            // DX9 0x00423765:30283-30300 (`pd+0x10 = 25000`, `pd+0x14 = 200`, `pd+0x18 = 0xe`).
            keepalive_timeout: 25000,
            connect_timeout: 200,
            connect_retries: 14,
            // SP GetCaps +8 = 1384 (DX9 0x0042e1ba:41016; binding+0x14).
            sp_max_datagram: 1384,
            link_tuning_off: false,
            lan_hint: false,
            max_recv_msg_size: 0xffff_ffff,
            num_send_retries: 10,
            max_send_retry_interval: 5000,
            drop_threshold_rate: 7,
            throttle_rate: 25,
            num_hard_disconnect_sends: 3,
            max_hard_disconnect_period: 500,
        }
    }
}

impl ProtocolConfig {
    /// `binding+0x18 = binding+0x14 - 0x1c` (DX9 0x004239fc:30423; 0x1c = the largest DFRAME
    /// header, 4 + 4 masks x 4 + 8 signature): the DFRAME payload per frame, 1356.
    pub fn frame_payload(&self) -> u32 {
        self.sp_max_datagram - 0x1c
    }

    /// The send window at establishment: 63 frames fixed with link tuning off, else 2.
    pub fn initial_frames(&self) -> u32 {
        if self.link_tuning_off {
            63
        } else if self.lan_hint {
            32
        } else {
            2
        }
    }
}

/// The CRT `rand()` the session id comes from (`Dpl8r_InitProtocolData`: `srand(timeGetTime())`
/// at 0x004297b3, then `rand() | rand() << 16`): the MSVC LCG `h = h * 0x343FD + 0x269EC3`,
/// result `(h >> 16) & 0x7FFF`.
#[derive(Debug, Clone, Copy)]
pub struct CrtRand(u32);

impl CrtRand {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }

    pub fn rand(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(0x343FD).wrapping_add(0x269EC3);
        (self.0 >> 16) & 0x7FFF
    }
}

/// A pending CONNECT (`Dpl8r_BuildCFrame` allocates one on `conn+0x2c` for a CFRAME with POLL):
/// its `bMsgID` (+0x10) and send time (+8).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingConnect {
    pub msg_id: u8,
    pub sent: u32,
}

/// What a queued message stands for (the retail operation object's type, `op[0]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpKind {
    /// A user message; completion is the core's SEND_COMPLETE.
    User,
    /// 7: the core's graceful disconnect of the connection (`FUN_0041d200`).
    Disconnect,
    /// 8: our END_STREAM answering the peer's (`Dpl8r_OnEndStream`).
    DisconnectReply,
    /// 10: a KeepAlive (`Dpl8r_SendKeepAlive`).
    KeepAlive,
}

/// An operation handle the core passes in and gets back in an [`Event`].
pub type OpId = u32;

/// One message waiting to be framed and sent (`Dpl8r_QueueUserMessage` 0x0041dcc0 builds these;
/// `Dpl8r_QueueEndStream` 0x0041d560 and `Dpl8r_SendKeepAlive` 0x00426cf0 build the one-frame
/// special ones).
#[derive(Debug)]
pub(crate) struct SendMsg {
    pub op: OpId,
    pub op_kind: OpKind,
    pub reliable: bool,
    pub frames: Vec<SendFrame>,
    /// `msg[0x11]`: frames not yet handed to the SP (unreliable completion, `FUN_00424170` case 2).
    pub frames_unsent: usize,
    /// `msg[4] & 2`: the last frame was acknowledged (`Dpl8r_ProcessAck`).
    pub complete: bool,
}

/// One frame of a message (the 0x20-byte-header frame item).
#[derive(Debug, Clone)]
pub(crate) struct SendFrame {
    /// The byte at item `+0x20`: DATA | RELIABLE | SEQUENTIAL | user bits, |END_MSG on the last.
    pub cmd: u8,
    /// The item flags `F[2]`: bit 0x4 last frame, 0x8 END_STREAM, 0x100 POLL (KeepAlive).
    pub item_flags: u32,
    pub payload: Rc<Vec<u8>>,
}

pub(crate) const ITEM_LAST: u32 = 0x4;
pub(crate) const ITEM_END_STREAM: u32 = 0x8;
pub(crate) const ITEM_POLL: u32 = 0x100;
/// Port-side marker for the KeepAlive item (DX9 item flags 0x100104 + 0x100000 no-coalesce).
pub(crate) const ITEM_KEEPALIVE: u32 = 0x0040_0000;

/// A DFRAME on the in-flight list (`conn[0x34]`, kept in sequence order).
#[derive(Debug, Clone)]
pub(crate) struct InFlight {
    pub msg: u32,
    pub reliable: bool,
    /// Header as built (`F+0x88`).
    pub cmd: u8,
    pub ctl: u8,
    pub seq: u8,
    pub payload: Rc<Vec<u8>>,
    /// `F[1]`: payload bytes counted in the byte window; 0 once an unreliable frame was given up.
    pub len: u32,
    /// `F[0x34/4+2]`: retransmissions so far (0..=10).
    pub retries: u32,
    /// `F+0x40..`: send time of try 0, 1, ...
    pub sent: [u32; 11],
    /// `F[2] & 0x800`: waiting in the resend list.
    pub in_resend: bool,
    /// `F[2] & 0x20`: this frame owns the retry timer.
    pub timer_owner: bool,
    /// The message's op and whether this frame completes it when acknowledged.
    pub completes: Option<(OpId, OpKind)>,
}

/// A DFRAME held in the receive list or being assembled (`FUN_00422340`'s frame record).
#[derive(Debug, Clone)]
pub(crate) struct RxFrame {
    pub seq: u8,
    pub cmd: u8,
    pub ctl: u8,
    pub payload: Vec<u8>,
    pub time: u32,
    /// Flag 0x10: a stray frame (no NEW_MSG while no message was open): the whole message is
    /// dropped (`FUN_00422840`, `FUN_00423070`).
    pub stray: bool,
    /// Flag 0x2: already delivered as part of an early non-sequential message.
    pub delivered: bool,
    /// The messages of a coalesced DFRAME (`bControl & 4`), delivered in order when the frame is
    /// consumed in sequence.
    pub subs: Option<Vec<frames::SubMessage>>,
}

/// A complete received message: its frames in order (`msg[2]` count, `msg[3]` bytes).
#[derive(Debug, Clone)]
pub(crate) struct RxMessage {
    pub frames: Vec<RxFrame>,
}

/// A timer or queued callback (the callback pointer given to `FUN_00427620`/`FUN_004274c0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Work {
    /// `FUN_00425790`: flush the SP's CFRAME list.
    FlushCframes,
    /// `FUN_00425870`: the send pump entry.
    Pump,
    /// `Dpl8r_ConnectRetryTimer`.
    ConnectRetry,
    /// DX9 `FUN_00427ad2`: the next HARD_DISCONNECT of a disconnect.
    HardDisconnect,
    /// `FUN_00426be0` on `conn[99]`: the delayed acknowledgement.
    DelayedAck,
    /// `FUN_00426be0` on `conn[0x65]`: the SACK / send-mask timer.
    SackTimer,
    /// `FUN_00426160`: the retransmit timer.
    Retransmit,
    /// `Dpl8r_KeepAliveTimer`.
    KeepAlive,
    /// `FUN_00427290`: tick-period housekeeping.
    Housekeeping,
    /// The SP's connect completion (`FUN_00424170` case 3 -> `Dpl8r_SendConnect`).
    SpConnectComplete,
}

/// Notifications for the core, in the order the retail code made the calls (the
/// `(*(core+0x20) + k)` callback table of dpnet.dll.c:25780-26121 and 28045-28490).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Callback `+0x2c` (`FUN_0041e920`): the connect finished; `Ok` or the error code.
    ConnectResult(Result<(), Hresult>),
    /// Callback `+0x18` (`Dpl8r_DeliverReceivedFrame`): a message from the peer;
    /// `flags` is `bCommand & 0xC0` (USER_1 = 0x40, USER_2 = 0x80).
    Receive { data: Vec<u8>, flags: u8 },
    /// Callback `+0x10` (`Dpl8r_OnEndStream`): the peer ended the stream.
    PeerEndStream,
    /// Callback `+0x14` (`Dpl8r_TearDownConnection`): the connection is gone, with the code
    /// (`0x80158160`, `DPNERR_CONNECTIONLOST`).
    ConnectionLost(Hresult),
    /// Callback `+0x34` (`FUN_00424f80`/`FUN_00424e60`): a user send finished.
    SendComplete { op: OpId, result: Hresult },
    /// Callback `+0x30` (`FUN_00424f80`, op type 7): the graceful disconnect finished.
    DisconnectComplete { op: OpId, result: Hresult },
}

/// The connection (dpnet `conn`, allocated by `FUN_00420900` at 0x00420900).
pub(crate) struct Conn {
    /// `conn[4]`.
    pub flags: u32,
    /// `[0x31]` +0xc4.
    pub sess_id: u32,
    /// `[0xe]` byte +0x38: the next `bMsgID`.
    pub msg_id: u8,
    /// `[0xb]` +0x2c.
    pub pending: VecDeque<PendingConnect>,
    /// `[0x5d]` +0x174: CONNECT retries left.
    pub connect_retries: u32,
    /// `[0x5e]` +0x178: the retransmit base; before establishment the CONNECT interval.
    pub retry_base: u32,
    /// `[0x15]` +0x54.
    pub rtt: u32,
    /// `[0x16]` +0x58: smoothed rtt, 16.16.
    pub srtt: u32,
    /// `[0x1b]` +0x6c: tick of the last datagram received.
    pub last_rx: u32,
    /// `[0x4c]` byte +0x130.
    pub next_send: u8,
    /// byte +0x131.
    pub next_recv: u8,
    /// `[0x4d]` +0x134: tick of the last DFRAME received.
    pub rx_tick: u32,
    /// `[0x4e]` byte +0x138: last DFRAME had RETRY.
    pub rx_retry: u8,
    /// `conn+0x138`: the sequence of the last frame an rtt sample was taken from (DX9
    /// 0x00421cca:28800 sets 0xff).
    pub rtt_seq: u8,
    /// `GetConnectionInfo` dwords 4..=22 in order (`conn+0x1cc..+0x1c4`): sent guaranteed
    /// bytes/packets, sent non-guaranteed bytes/packets, retried, dropped, queued per priority,
    /// timed out per priority, received guaranteed/non-guaranteed bytes/packets, messages.
    pub stats: [u32; 19],
    /// `[0x4f]`/`[0x50]` +0x13c/+0x140: received-out-of-order masks (bit i = nrcv + 1 + i).
    pub sack1: u32,
    pub sack2: u32,
    /// `[0x51]` +0x144: when the SACK timer was armed.
    pub sack_stamp: u32,
    /// `[0x52]`/`[0x53]` +0x148/+0x14c: send masks.
    pub send1: u32,
    pub send2: u32,
    /// `conn+0x158` list: out-of-order frames sorted by distance from `next_recv`.
    pub ooo: Vec<RxFrame>,
    /// `[0x54]`/`[0x55]`: the message being assembled.
    pub rx_open: Vec<RxFrame>,
    /// `conn+0x160` list `[0x58]`: complete messages awaiting delivery.
    pub delivery: VecDeque<RxMessage>,
    // --- send side
    pub msgs: BTreeMap<u32, SendMsg>,
    pub next_msg: u32,
    /// `conn+0xa4`, `+0xac`, `+0xb4`: high, normal, low queues of message ids.
    pub queues: [VecDeque<u32>; 3],
    /// `[0x28]` +0xa0: queued messages.
    pub queued: u32,
    /// `conn+0xbc` list `[0x2f]`: messages the pump started, in order, until they complete.
    pub started: VecDeque<u32>,
    /// `[0x32]`, `[0x33]`: the message being sent and its next frame index.
    pub cur_msg: Option<u32>,
    pub cur_frame: usize,
    /// `[0x34]`: in flight, ascending sequence.
    pub inflight: VecDeque<InFlight>,
    /// `[0x36]`: sequence numbers waiting to be resent.
    pub resend: VecDeque<u8>,
    /// `[0x38]` +0xe0: frame window.
    pub frame_window: u32,
    /// `[0x39]` +0xe4: byte window.
    pub byte_window: u32,
    /// `[0x3a]` +0xe8: byte-window steps.
    pub byte_steps: u32,
    /// `[0x3b]`, `[0x3c]` +0xec/+0xf0.
    pub inflight_count: u32,
    pub inflight_bytes: u32,
    /// `[0x3d]` +0xf4, `[0x3e]` +0xf8, `[0x3f]` +0xfc, `[0x40]` +0x100, `[0x41]` +0x104.
    pub pace_factor: u32,
    pub pace_bytes: u32,
    pub pace_ms: u32,
    pub pace_carry: i32,
    pub pace_next: u32,
    pub cc: congestion::CongestionState,
    // --- timer handles
    pub t_connect: u32,
    pub t_ack: u32,
    pub t_sack: u32,
    pub t_retry: u32,
    pub t_keepalive: u32,
    pub t_pump: u32,
    /// The connect op's pending state (`conn[6]`): the connect is waiting for the result.
    pub connect_pending: bool,
    /// `conn[5] & 0x40`: the host speaks DPL8R 1.6 (host minor >= 6, DX9 0x00421d70:28862-28865).
    pub peer_v16: bool,
    /// `conn[5] & 0x180` and `+0x78..+0xa9`: packet signing, once a CONNECTED_SIGNED was accepted.
    pub sig: Option<signing::SigState>,
    /// `conn[5] & 8`: coalescing off (host minor < 5 or major != 1, 0x00421c4e:28787-28790).
    pub no_coalesce: bool,
    /// A HARD_DISCONNECT was received and answered (`conn[4] & 0x80000000` in DX9 0x004247fc).
    pub hd_answered: bool,
    /// The hard disconnect we started has frames left to send.
    pub hd_left: u32,
    pub t_hd: u32,
}

impl Conn {
    /// `FUN_00420900` (0x00420900): the allocator's initial values.
    pub(crate) fn new(cfg: &ProtocolConfig) -> Self {
        Self {
            flags: 0x43,
            sess_id: 0,
            msg_id: 0,
            pending: VecDeque::new(),
            connect_retries: 0,
            retry_base: 0,
            rtt: 0,
            srtt: 0,
            last_rx: 0,
            next_send: 0,
            next_recv: 0,
            rx_tick: 0,
            rx_retry: 0,
            rtt_seq: 0xff,
            stats: [0; 19],
            sack1: 0,
            sack2: 0,
            sack_stamp: 0,
            send1: 0,
            send2: 0,
            ooo: Vec::new(),
            rx_open: Vec::new(),
            delivery: VecDeque::new(),
            msgs: BTreeMap::new(),
            next_msg: 1,
            queues: [VecDeque::new(), VecDeque::new(), VecDeque::new()],
            queued: 0,
            started: VecDeque::new(),
            cur_msg: None,
            cur_frame: 0,
            inflight: VecDeque::new(),
            resend: VecDeque::new(),
            frame_window: cfg.initial_frames(),
            // byte limit = binding+0x14 (1384) x frames (verify_spec_client_delta 2.6.1)
            byte_window: cfg.sp_max_datagram * cfg.initial_frames(),
            byte_steps: 1,
            inflight_count: 0,
            inflight_bytes: 0,
            pace_factor: 1,
            pace_bytes: cfg.sp_max_datagram,
            pace_ms: 100,
            pace_carry: 0,
            pace_next: 0,
            // DX9 has no loss credit: its loss state (`+0x44`, `+0x48`, `+0x50`) starts at 0
            // (FUN_004233e3:30090-30095) and `0x4000000` is the CC_BACKOFF flag bit. This 8.0
            // counter only matters with link tuning on; never reached with Initialize flag 4.
            cc: congestion::CongestionState {
                credit: 0x0400_0000,
                ..Default::default()
            },
            t_connect: 0,
            t_ack: 0,
            t_sack: 0,
            t_retry: 0,
            t_keepalive: 0,
            t_pump: 0,
            connect_pending: false,
            peer_v16: false,
            sig: None,
            no_coalesce: false,
            hd_answered: false,
            hd_left: 0,
            t_hd: 0,
        }
    }

    pub(crate) fn has(&self, bit: u32) -> bool {
        self.flags & bit != 0
    }

    pub(crate) fn set(&mut self, bit: u32) {
        self.flags |= bit;
    }

    pub(crate) fn clear(&mut self, bit: u32) {
        self.flags &= !bit;
    }
}

/// The protocol object: `pd` + `spd` + the single connection.
pub struct Protocol {
    pub(crate) cfg: ProtocolConfig,
    pub(crate) conn: Conn,
    pub(crate) timers: Timers<Work>,
    /// `timeGetTime()` for the current entry point.
    pub(crate) now: u32,
    /// `pd[4]`: the next session id (`rand() | rand() << 16`, then ++ per attempt).
    pub(crate) next_sess: u32,
    /// `spd+0x3c`: CFRAMEs waiting for `FUN_00425790` / `FUN_004256e0` to send them.
    pub(crate) cframes: VecDeque<Vec<u8>>,
    /// `spd[3] & 0x20`: the flush work item is queued.
    pub(crate) flush_scheduled: bool,
    /// Datagrams ready for the SP socket, in send order.
    pub(crate) outbox: Vec<Vec<u8>>,
    pub(crate) events: VecDeque<Event>,
    /// A graceful disconnect op waits for both END_STREAMs (`conn[6]` when `op type 7`).
    pub(crate) disconnect_op: Option<(OpId, OpKind)>,
    /// `CryptGenRandom` for the signing secrets (0x00421f01:28975-28982).
    pub(crate) secret_rng: Box<dyn crate::dplay8::clock::RandomSource>,
}

impl Protocol {
    /// `Dpl8r_InitProtocolData` (0x00420220) and the SPD creation (0x00420350).
    pub fn new(cfg: ProtocolConfig, now: u32) -> Self {
        let mut rand = CrtRand::new(now);
        let lo = rand.rand();
        let hi = rand.rand();
        Self::with_session_seed(cfg, now, lo | (hi << 16))
    }

    /// As [`Protocol::new`] with the first session id chosen by the caller (tests).
    pub fn with_session_seed(cfg: ProtocolConfig, now: u32, first_sess: u32) -> Self {
        let mut timers = Timers::new(now);
        // FUN_00427960: FUN_00427620(1000, 500, FUN_00427290, ...) housekeeping from the start.
        timers.schedule(now, 1000, 500, Work::Housekeeping);
        Self {
            cfg,
            conn: Conn::new(&cfg),
            timers,
            now,
            next_sess: first_sess,
            cframes: VecDeque::new(),
            flush_scheduled: false,
            outbox: Vec::new(),
            events: VecDeque::new(),
            disconnect_op: None,
            secret_rng: Box::new(crate::dplay8::clock::OsRandom::new()),
        }
    }

    /// Replace the source of the signing secrets (tests).
    pub fn set_secret_source(&mut self, r: Box<dyn crate::dplay8::clock::RandomSource>) {
        self.secret_rng = r;
    }

    pub fn config(&self) -> &ProtocolConfig {
        &self.cfg
    }

    /// `FUN_0041d390` (SetCaps): connect timeout (`pd[6]`), retries (`pd[7]`), keepalive
    /// (`pd[5]`).
    pub fn set_caps(&mut self, connect_timeout: u32, connect_retries: u32, keepalive: u32) {
        self.cfg.connect_timeout = connect_timeout;
        self.cfg.connect_retries = connect_retries;
        self.cfg.keepalive_timeout = keepalive;
    }

    /// `FUN_0041d360` (GetCaps): `(connect_timeout, connect_retries, keepalive)`.
    pub fn caps(&self) -> (u32, u32, u32) {
        (
            self.cfg.connect_timeout,
            self.cfg.connect_retries,
            self.cfg.keepalive_timeout,
        )
    }

    /// Datagrams to hand to the SP, drained.
    pub fn take_outbox(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.outbox)
    }

    /// Notifications for the core, drained.
    pub fn take_events(&mut self) -> Vec<Event> {
        self.events.drain(..).collect()
    }

    /// The `GetConnectionInfo` counters (dwords 4..=22).
    pub fn stats(&self) -> [u32; 19] {
        self.conn.stats
    }

    /// The smoothed round-trip time in ms (`conn+0x3c` in DX9, what `GetConnectionInfo` serves).
    pub fn round_trip_ms(&self) -> u32 {
        self.conn.rtt
    }

    pub fn is_connected(&self) -> bool {
        self.conn.has(flag::CONNECTED)
    }

    /// The connection is finished: released by `FUN_00420e50` (every teardown and kill ends
    /// there). `DYING` alone is not enough: a graceful disconnect sets it (`0x20020`) while the
    /// END_STREAM exchange is still under way.
    pub fn is_dead(&self) -> bool {
        self.conn.has(flag::RELEASED)
    }

    /// The tick period of the timer wheel (see [`Timers::period`]).
    pub fn timer_period(&self) -> u32 {
        self.timers.period()
    }

    /// `FUN_00427620` wrapper that stores the handle on the conn.
    pub(crate) fn arm(&mut self, delay: u32, gran: u32, what: Work) -> u32 {
        self.timers.schedule(self.now, delay, gran, what)
    }

    /// `FUN_004277c0` for a stored handle; clears it.
    pub(crate) fn disarm(&mut self, handle: &mut u32) {
        if *handle != 0 {
            self.timers.cancel(*handle);
            *handle = 0;
        }
    }

    /// One pass of the worker: tick the wheel, then run every ready callback in order. The
    /// caller feeds received datagrams with [`Protocol::on_datagram`] before this.
    pub fn do_work(&mut self, now: u32) {
        self.now = now;
        self.timers.tick(now);
        while let Some((id, work)) = self.timers.pop_ready() {
            self.run_work(id, work);
        }
    }

    fn run_work(&mut self, id: u32, work: Work) {
        match work {
            Work::FlushCframes => self.flush_cframes_work(),
            Work::Pump => self.pump_entry(id),
            Work::ConnectRetry => self.connect_retry_timer(id),
            Work::HardDisconnect => self.hard_disconnect_timer(id),
            Work::DelayedAck => self.ack_timer(id, true),
            Work::SackTimer => self.ack_timer(id, false),
            Work::Retransmit => self.retransmit_timer(id),
            Work::KeepAlive => self.keepalive_timer(id),
            Work::Housekeeping => {
                self.timers.housekeeping();
                self.arm(1000, 100, Work::Housekeeping);
            }
            Work::SpConnectComplete => self.sp_connect_complete(),
        }
    }
}
