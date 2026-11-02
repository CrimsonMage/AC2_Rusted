//! The `DPN_MSGID_*` messages dpnet hands the application's message handler, with the layouts
//! the client path fills (dpnet's "indicate" functions, dpnet.dll.c:21806-22217; the struct
//! definitions are `third_party/directx8/include/dplay8.h:476-740`).
//!
//! Only the messages a client/server **client** raises are modelled. `CREATE_PLAYER`,
//! `DESTROY_PLAYER` and the group/peer messages are host and peer messages: the client path
//! never raises them (`FUN_004123ad`, dpnet.dll.c:15141, calls `FUN_00419219` only when the core
//! flag `0x2` is clear, and the client sets it). `RETURN_BUFFER` is raised only for the reply
//! buffer an `INDICATE_CONNECT` handler returns (dpnet.dll.c:9687, 10209), a host path.

use crate::Hresult;

/// `DPN_MSGID_OFFSET`-based ids (dplay8.h:130-152).
pub const DPN_MSGID_ASYNC_OP_COMPLETE: u32 = 0xFFFF_0003;
pub const DPN_MSGID_CONNECT_COMPLETE: u32 = 0xFFFF_0005;
pub const DPN_MSGID_CREATE_PLAYER: u32 = 0xFFFF_0007;
pub const DPN_MSGID_RECEIVE: u32 = 0xFFFF_0011;
pub const DPN_MSGID_RETURN_BUFFER: u32 = 0xFFFF_0013;
pub const DPN_MSGID_SEND_COMPLETE: u32 = 0xFFFF_0014;
pub const DPN_MSGID_TERMINATE_SESSION: u32 = 0xFFFF_0016;

/// A `DPNHANDLE`. The binary's values come from a handle table (`FUN_00402c83`); the encoding is
/// not traced, so these are a plain counter (opaque to callers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DpnHandle(pub u32);

/// `DPNMSG_CONNECT_COMPLETE` (dwSize 0x18: `FUN_00419127`, dpnet.dll.c:21806): the async handle
/// and user context of the Connect call, the result, and the host's application reply data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectComplete {
    pub async_op: DpnHandle,
    pub user_context: usize,
    pub result: Hresult,
    pub reply_data: Vec<u8>,
}

/// `DPNMSG_RECEIVE` (dwSize 0x18: `FUN_004195a3`, dpnet.dll.c:22132). For a client
/// `dpnidSender` is 0 (`(core.flags & 2) ? 0 : dpnid`). The data stays valid until the handler
/// returns, or, if it returned `DPNSUCCESS_PENDING`, until `return_buffer(buffer)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receive<'a> {
    pub sender: u32,
    pub data: &'a [u8],
    pub buffer: DpnHandle,
}

/// `DPNMSG_SEND_COMPLETE` (dwSize 0x14: `FUN_0041954a`, dpnet.dll.c:22101): the send's handle
/// and context, the result, and `dwSendTime` = `timeGetTime() - start of Send`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendComplete {
    pub async_op: DpnHandle,
    pub user_context: usize,
    pub result: Hresult,
    pub send_time: u32,
}

/// `DPNMSG_TERMINATE_SESSION` (dwSize 0x10: `FUN_0041964e`, dpnet.dll.c:22181): `0xDF` carries
/// `DPNERR_HOSTTERMINATEDSESSION` and the host's data; a lost or ended connection carries
/// `DPNERR_CONNECTIONLOST` and no data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminateSession {
    pub result: Hresult,
    pub data: Vec<u8>,
}

/// `DPNMSG_ASYNC_OP_COMPLETE` (dwSize 0x10): modelled for completeness; the client's own
/// operations complete through `CONNECT_COMPLETE` and `SEND_COMPLETE` and never raise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncOpComplete {
    pub async_op: DpnHandle,
    pub user_context: usize,
    pub result: Hresult,
}

/// One message to the handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DpnMessage<'a> {
    ConnectComplete(&'a ConnectComplete),
    Receive(Receive<'a>),
    SendComplete(&'a SendComplete),
    TerminateSession(&'a TerminateSession),
    AsyncOpComplete(&'a AsyncOpComplete),
}

impl DpnMessage<'_> {
    /// The `DPN_MSGID_*` value.
    pub fn id(&self) -> u32 {
        match self {
            DpnMessage::ConnectComplete(_) => DPN_MSGID_CONNECT_COMPLETE,
            DpnMessage::Receive(_) => DPN_MSGID_RECEIVE,
            DpnMessage::SendComplete(_) => DPN_MSGID_SEND_COMPLETE,
            DpnMessage::TerminateSession(_) => DPN_MSGID_TERMINATE_SESSION,
            DpnMessage::AsyncOpComplete(_) => DPN_MSGID_ASYNC_OP_COMPLETE,
        }
    }
}

/// `PFNDPNMESSAGEHANDLER`: the callback registered with `Initialize`. The return value matters
/// only for `RECEIVE`: `DPNSUCCESS_PENDING` keeps the buffer until `ReturnBuffer`; anything else
/// returns it at once (`FUN_004195a3`: a result other than `0x15800E` is dropped to 0).
pub trait MessageHandler {
    fn message(&mut self, msg: DpnMessage<'_>) -> Hresult;
}
