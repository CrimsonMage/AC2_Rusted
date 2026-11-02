//! The DirectPlay 8 surface GunDll calls, as Rust traits for the transport author
//! (`crates/ac2-gun/src/dplay8/`) to implement. Names and shapes follow
//! `third_party/directx8/include/dplay8.h` (DX8) and, for the thread pool (DX9 only), Wine's
//! `/usr/include/wine/windows/dplay8.h`. Only what GunDll calls is here
//! (`decompiled-pdb/gundll/spec/0a_send_session.md` section 2, verified):
//!
//! * `IDirectPlay8Client`: `Initialize` (slot 3, vtable +0xc), `CancelAsyncOperation` (+0x18),
//!   `Connect` (+0x1c), `Send` (+0x20), `Close` (+0x38), `ReturnBuffer` (+0x3c), `GetCaps` (+0x40),
//!   `SetCaps` (+0x44), `GetConnectionInfo` (+0x50). `SetClientInfo` is never called.
//! * `IDirectPlay8Address`: only as the values `SetSP` and `AddComponent` build, [`Dp8Address`].
//! * `IDirectPlay8ThreadPool`: `SetThreadCount`, `DoWork`.
//!
//! COM objects are `Rc<dyn ...>` here and `CoCreateInstance` is the [`Dp8Factory`]; all calls are
//! `&self` (interior mutability), single threaded: the handler runs inside `DoWork`
//! (GunDll sets zero worker threads).

use std::rc::Rc;

use crate::interfaces::Guid;
use crate::Hresult;

/// `DPN_MSGID_*` (dplay8.h:130-152).
pub mod msgid {
    pub const ASYNC_OP_COMPLETE: u32 = 0xFFFF_0003;
    pub const CONNECT_COMPLETE: u32 = 0xFFFF_0005;
    pub const CREATE_PLAYER: u32 = 0xFFFF_0007;
    pub const ENUM_HOSTS_RESPONSE: u32 = 0xFFFF_000B;
    pub const RECEIVE: u32 = 0xFFFF_0011;
    pub const RETURN_BUFFER: u32 = 0xFFFF_0013;
    pub const SEND_COMPLETE: u32 = 0xFFFF_0014;
    pub const TERMINATE_SESSION: u32 = 0xFFFF_0016;
}

/// `DPNINITIALIZE_DISABLELINKTUNING` (Wine dplay8.h:193); GunDll's `Initialize` flags are 4, then 0.
pub const DPNINITIALIZE_DISABLELINKTUNING: u32 = 4;
/// `DPNSESSION_CLIENT_SERVER` (dplay8.h:296).
pub const DPNSESSION_CLIENT_SERVER: u32 = 1;
/// `DPNCANCEL_CONNECT` (dplay8.h:195).
pub const DPNCANCEL_CONNECT: u32 = 0x1;
/// `DPNCANCEL_ENUM` (dplay8.h:197).
pub const DPNCANCEL_ENUM: u32 = 0x2;
/// `DPNCANCEL_SEND` (dplay8.h:198).
pub const DPNCANCEL_SEND: u32 = 0x4;
/// `DPNSEND_NOCOPY` (dplay8.h:284).
pub const DPNSEND_NOCOPY: u32 = 0x1;
/// `DPNSEND_GUARANTEED` (dplay8.h:287).
pub const DPNSEND_GUARANTEED: u32 = 0x8;
/// `DPNSUCCESS_PENDING` (0x0015800E): the success `Connect` and `Send` return for an asynchronous
/// operation, and what the message handler returns for `RECEIVE` to keep the buffer.
pub const DPNSUCCESS_PENDING: Hresult = Hresult(0x0015_800E);
/// `DPNERR_HOSTREJECTEDCONNECTION` (0x80158260): `CONNECT_COMPLETE`'s refusal.
pub const DPNERR_HOSTREJECTEDCONNECTION: Hresult = Hresult(0x8015_8260);
/// `DPNERR_HOSTTERMINATEDSESSION` (0x80158270).
pub const DPNERR_HOSTTERMINATEDSESSION: Hresult = Hresult(0x8015_8270);
/// `DPNERR_NOCONNECTION` (0x80158480).
pub const DPNERR_NOCONNECTION: Hresult = Hresult(0x8015_8480);
/// `DPNERR_USERCANCEL` (0x80158650).
pub const DPNERR_USERCANCEL: Hresult = Hresult(0x8015_8650);
/// `DPNERR_TIMEDOUT` (0x80158630).
pub const DPNERR_TIMEDOUT: Hresult = Hresult(0x8015_8630);
/// `DPNERR_NOTALLOWED` (0x80158520): `DoWork`'s answer with no worker-thread model.
pub const DPNERR_NOTALLOWED: Hresult = Hresult(0x8015_8520);

/// `DPNA_DATATYPE` of an address component (dpaddr.h): 1 STRING, 2 DWORD, 3 GUID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dp8AddrData {
    /// `DPNA_DATATYPE_STRING` (wide string, without the NUL).
    String(String),
    /// `DPNA_DATATYPE_DWORD`.
    Dword(u32),
    /// `DPNA_DATATYPE_GUID`.
    Guid(Guid),
}

/// An `IDirectPlay8Address` after `SetSP` and `AddComponent` calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dp8Address {
    /// `SetSP(&GUID)` (vtable +0x34): the service provider.
    pub sp: Guid,
    /// `AddComponent(name, data, size, type)` (vtable +0x48), in call order.
    pub components: Vec<(String, Dp8AddrData)>,
}

/// `DPN_APPLICATION_DESC` (dplay8.h:337), the fields GunDll fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dp8ApplicationDesc {
    /// `dwSize` = 0x48.
    pub size: u32,
    /// `dwFlags` = `DPNSESSION_CLIENT_SERVER`.
    pub flags: u32,
    pub guid_instance: Guid,
    pub guid_application: Guid,
}

/// `DPN_CAPS` (dplay8.h:367): GunDll sets `dwTimeoutUntilKeepAlive` (offset +0x10) after a
/// `GetCaps` with `dwSize` 0x14 and leaves the other fields as read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dp8Caps {
    pub size: u32,
    pub flags: u32,
    pub connect_timeout: u32,
    pub connect_retries: u32,
    pub timeout_until_keep_alive: u32,
}

/// `DPN_CONNECTION_INFO` (dplay8.h:419), the part GunDll reads: `FUN_1003cfc0` (0x1003cfc0) sets
/// `dwSize` 0x5c and reads `dwRoundTripLatencyMS` (+4, asm 0x1003cfe5 `add eax,[esp+8]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dp8ConnectionInfo {
    pub size: u32,
    pub round_trip_latency_ms: u32,
}

/// The `DPNMSG_*` structures the handler receives (the ones GunDll's handler 0x1003d1b0 treats;
/// every other id is `Other` and ignored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dp8Message {
    /// `DPN_MSGID_ASYNC_OP_COMPLETE`.
    AsyncOpComplete,
    /// `DPN_MSGID_CONNECT_COMPLETE`: `hResultCode` and the connect reply data.
    ConnectComplete { hresult: Hresult, reply: Vec<u8> },
    /// `DPN_MSGID_CREATE_PLAYER`: `dpnidPlayer`.
    CreatePlayer { dpnid: u32 },
    /// `DPN_MSGID_RECEIVE`: the datagram and its `hBufferHandle`; the handler returns
    /// [`DPNSUCCESS_PENDING`] and the buffer is handed back through `ReturnBuffer`.
    Receive { data: Vec<u8>, buffer_handle: u32 },
    /// `DPN_MSGID_RETURN_BUFFER`: `pvBuffer` of a send.
    ReturnBuffer { context: u32 },
    /// `DPN_MSGID_SEND_COMPLETE`: `pvUserContext` and `hResultSend`.
    SendComplete { context: u32, hresult: Hresult },
    /// `DPN_MSGID_TERMINATE_SESSION`: `hResultCode` and the terminate data.
    TerminateSession { hresult: Hresult, data: Vec<u8> },
    /// Any other message id.
    Other { id: u32 },
}

/// `PFNDPNMESSAGEHANDLER` with its `pvUserContext` captured: GunDll's handler is
/// `Dp8MessageHandler` (0x1003d1b0), [`super::transport::Dp8Handler`].
pub trait Dp8MessageHandler {
    /// Returns the handler's `HRESULT` (`DPNSUCCESS_PENDING` for `RECEIVE`, else 0).
    fn message(&self, msg: &Dp8Message) -> Hresult;
}

/// `IDirectPlay8Client`, the subset GunDll calls.
pub trait IDirectPlay8Client {
    /// Slot 3, +0xc: `Initialize(pvUserContext, pfn, dwFlags)`.
    fn initialize(&self, handler: Rc<dyn Dp8MessageHandler>, flags: u32) -> Hresult;
    /// Slot 6, +0x18: `CancelAsyncOperation(hAsyncHandle, dwFlags)`. GunDll calls it `(0, DPNCANCEL_ENUM)`
    /// at the start of every connect (S slot 13, 0x1003ccb0, from `FUN_1003dbd0`) and
    /// `(handle, DPNCANCEL_CONNECT)` when it tears a connection down with a pending connect handle
    /// (S slot 17, 0x1003ccf0, from `FUN_10003ae0`).
    fn cancel_async_operation(&self, handle: u32, flags: u32) -> Hresult;
    /// Slot 7, +0x1c: `Connect(pdnAppDesc, pHostAddr, pDeviceInfo, pdnSecurity = NULL,
    /// pdnCredentials = NULL, pvUserConnectData = NULL, dwUserConnectDataSize = 0,
    /// pvAsyncContext = NULL, phAsyncHandle, dwFlags = 0)`. Returns the `HRESULT`
    /// (`DPNSUCCESS_PENDING` while the connect is in progress) and the async handle.
    fn connect(
        &self,
        app_desc: &Dp8ApplicationDesc,
        host: &Dp8Address,
        device: &Dp8Address,
    ) -> (Hresult, u32);
    /// Slot 8, +0x20: `Send(prgBufferDesc, cBufferDesc = 1, dwTimeOut = 0, pvAsyncContext,
    /// phAsyncHandle, dwFlags = DPNSEND_NOCOPY | DPNSEND_GUARANTEED)`: one buffer. `context` is the
    /// `pvAsyncContext` (GunDll passes its pool block) that `SEND_COMPLETE` hands back.
    fn send(&self, buffer: &[u8], timeout: u32, context: u32, flags: u32) -> Hresult;
    /// Slot 14, +0x38: `Close(dwFlags)`.
    fn close(&self, flags: u32) -> Hresult;
    /// Slot 15, +0x3c: `ReturnBuffer(hBufferHandle, dwFlags)`.
    fn return_buffer(&self, buffer_handle: u32, flags: u32) -> Hresult;
    /// Slot 16, +0x40: `GetCaps(pdpCaps, dwFlags)`.
    fn get_caps(&self, caps: &mut Dp8Caps, flags: u32) -> Hresult;
    /// Slot 17, +0x44: `SetCaps(pdpCaps, dwFlags)`.
    fn set_caps(&self, caps: &Dp8Caps, flags: u32) -> Hresult;
    /// Slot 20, +0x50: `GetConnectionInfo(pdpConnectionInfo, dwFlags)`, called with `dwSize` 0x5c and
    /// flags 0 by the type-0 control frame (`FUN_1003cfc0`, asm 0x1003cfdb `call [ecx+0x50]`).
    fn get_connection_info(&self, info: &mut Dp8ConnectionInfo, flags: u32) -> Hresult;
}

/// `IDirectPlay8ThreadPool` (DX9; Wine dplay8.h:911), the calls GunDll makes.
pub trait IDirectPlay8ThreadPool {
    /// `SetThreadCount(dwProcessorNum, dwNumThreads, dwFlags)`: GunDll passes (-1, 0, 0), no worker
    /// threads. (Its `Initialize(0, LAB_1003b520, 0)` is the factory's business.)
    fn set_thread_count(&self, processor_num: u32, num_threads: u32, flags: u32) -> Hresult;
    /// `DoWork(dwAllowedTimeSlice, dwFlags)`: GunDll passes (-1, 0); runs the pending callbacks on
    /// the calling thread. `DPNERR_NOTALLOWED` is mapped to 0 by GunDll's wrapper (0x1003b590).
    fn do_work(&self, allowed_time_slice: u32, flags: u32) -> Hresult;
}

/// `CoCreateInstance` of the DirectPlay 8 classes GunDll creates.
pub trait Dp8Factory {
    /// `CLSID_DirectPlay8Client` {743f1dc6-5aba-429f-8bdf-c54d03253dc2}, `CLSCTX` 0x17.
    fn create_client(&self) -> Result<Rc<dyn IDirectPlay8Client>, Hresult>;
    /// `CLSID_DirectPlay8ThreadPool` {fc47060e-6153-4b34-b975-8e4121eb7f3c}, `CLSCTX` 3, then
    /// `Initialize(0, pfn, 0)` (`DPNERR_ALREADYINITIALIZED` counts as success).
    fn create_thread_pool(&self) -> Result<Rc<dyn IDirectPlay8ThreadPool>, Hresult>;
}

/// `CLSID_DP8SP_TCPIP` {ebfe7ba0-628d-11d2-ae0f-006097b01411} (dplay8.h:61), GunDll 0x10060d1c.
pub const DP8SP_TCPIP: Guid = Guid {
    data1: 0xebfe7ba0,
    data2: 0x628d,
    data3: 0x11d2,
    data4: [0xae, 0x0f, 0x00, 0x60, 0x97, 0xb0, 0x14, 0x11],
};
