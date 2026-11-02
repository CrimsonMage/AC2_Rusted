//! `IDirectPlay8Client` for the subset GunDll uses (gundll spec 0a_send_session.md 1.5, 2.2,
//! 2.5; 0a_receive_messages.md 1.1): `Initialize`, `GetCaps`/`SetCaps`, `Connect`, `Send`,
//! `ReturnBuffer`, `Close`, with the DX9 thread pool's `DoWork` as the pump.
//!
//! Callbacks run on the caller's thread inside [`Dplay8Client::do_work`], as GunDll configured
//! the thread pool with zero worker threads. Nothing happens between calls.

use std::collections::HashMap;
use std::net::SocketAddrV4;

use crate::Hresult;

use super::address::{provider_host_sockaddr, Component, DpAddress, HostResolver, SystemResolver};
use super::clock::{Clock, OsRandom, RandomSource, SystemClock};
use super::connect_info::*;
use super::core::{Core, CoreState, Indication};
use super::hresult::*;
use super::messages::*;
use super::protocol::{Protocol, ProtocolConfig};
use super::sp::{
    bind_local_port, is_multi_homed, AdapterEnumerator, DatagramSocket, SocketOpener, StdAdapters,
    StdSocketOpener,
};

/// `DPN_CAPS` (dplay8.h:367-374). `dwSize` must be 0x14; dpnet's GetCaps/SetCaps move only the
/// three timing fields (`FUN_0041d360`, `FUN_0041d390`, dpnet.dll.c:24936-24954).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DpnCaps {
    pub size: u32,
    pub flags: u32,
    pub connect_timeout: u32,
    pub connect_retries: u32,
    pub timeout_until_keep_alive: u32,
}

pub const DPN_CAPS_SIZE: u32 = 0x14;
/// `DPN_CAPS_EX` (dwSize 0x30, DX9 `FUN_0042093c`/`FUN_0042098e`, dpnet9.dll.c:27657-27700).
pub const DPN_CAPS_EX_SIZE: u32 = 0x30;

/// `DPN_CAPS_EX`: the `DPN_CAPS` fields then `dwMaxRecvMsgSize`, `dwNumSendRetries`,
/// `dwMaxSendRetryInterval`, `dwDropThresholdRate`, `dwThrottleRate`,
/// `dwNumHardDisconnectSends`, `dwMaxHardDisconnectPeriod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DpnCapsEx {
    pub base: DpnCaps,
    pub max_recv_msg_size: u32,
    pub num_send_retries: u32,
    pub max_send_retry_interval: u32,
    pub drop_threshold_rate: u32,
    pub throttle_rate: u32,
    pub num_hard_disconnect_sends: u32,
    pub max_hard_disconnect_period: u32,
}

/// `DPN_APPLICATION_DESC` (dplay8.h:337-351), the part Connect reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DpnApplicationDesc {
    /// Must be 0x48 (`FUN_00413d63`).
    pub size: u32,
    pub flags: u32,
    pub guid_instance: [u8; 16],
    pub guid_application: [u8; 16],
    pub max_players: u32,
    pub current_players: u32,
    pub session_name: Option<String>,
    pub password: Option<String>,
}

/// `DPN_CONNECTION_INFO`: 0x5c bytes, `dwSize` then 22 dwords (DX9 0x00420a89): the rtt at +4,
/// throughput at +8 and +0xc, then 19 counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DpnConnectionInfo {
    pub size: u32,
    pub round_trip_latency_ms: u32,
    pub throughput_bps: u32,
    pub peak_throughput_bps: u32,
    pub counters: [u32; 19],
}

pub const DPN_CONNECTION_INFO_SIZE: u32 = 0x5c;

pub const DPN_APPLICATION_DESC_SIZE: u32 = 0x48;
pub const DPNSESSION_CLIENT_SERVER: u32 = 0x0001;

/// The thread pool GunDll sets up (`IDirectPlay8ThreadPool`: Initialize, Close, SetThreadCount,
/// DoWork; CLSID `fc47060e-6153-4b34-b975-8e4121eb7f3c`, GunDll spec 0a_send_session 2.1).
/// GunDll calls `SetThreadCount(-1, 0, 0)`: zero worker threads, so the work runs only when the
/// application calls `DoWork`. The process-wide pool is the explicit pump parameter here.
#[derive(Debug, Default)]
pub struct ThreadPool {
    initialized: bool,
    thread_count: u32,
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::default()
    }

    /// `Initialize(pvUserContext, pfn, dwFlags)`. A second call is `DPNERR_ALREADYINITIALIZED`,
    /// which GunDll treats as success.
    pub fn initialize(&mut self, _flags: u32) -> Hresult {
        if self.initialized {
            return DPNERR_ALREADYINITIALIZED;
        }
        self.initialized = true;
        Hresult(0)
    }

    pub fn close(&mut self, _flags: u32) -> Hresult {
        self.initialized = false;
        Hresult(0)
    }

    /// `SetThreadCount(dwProcessorNum, dwNumThreads, dwFlags)`; `0xFFFFFFFF` = every processor.
    pub fn set_thread_count(&mut self, _processor: u32, threads: u32, _flags: u32) -> Hresult {
        self.thread_count = threads;
        Hresult(0)
    }

    pub fn get_thread_count(&self) -> u32 {
        self.thread_count
    }

    /// `DoWork(dwAllowedTimeSlice, dwFlags)`: run the work of `client`. The time slice is not
    /// honoured: all ready work is drained (the slice is a limit, not a duty).
    pub fn do_work(&mut self, client: &mut Dplay8Client, _time_slice: u32, _flags: u32) -> Hresult {
        client.do_work()
    }
}

/// The template `DAT_004517dc` {4CE725F4-7B00-4397-BA6F-11F965BC4299} in memory order.
const DEVICE_GUID_TEMPLATE: [u8; 16] = [
    0xf4, 0x25, 0xe7, 0x4c, 0x00, 0x7b, 0x97, 0x43, 0xba, 0x6f, 0x11, 0xf9, 0x65, 0xbc, 0x42, 0x99,
];

/// Decode a device GUID into the IPv4 address of its `sockaddr_in` (family 2, zero padding).
pub fn decode_device_guid(g: &[u8; 16]) -> Result<std::net::Ipv4Addr, Hresult> {
    let mut sa = [0u8; 16];
    for i in 0..16 {
        sa[i] = g[i] ^ DEVICE_GUID_TEMPLATE[i];
    }
    if sa[0] != 2 || sa[1] != 0 || sa[8..].iter().any(|b| *b != 0) {
        return Err(DPNERR_INVALIDDEVICEADDRESS);
    }
    Ok(std::net::Ipv4Addr::new(sa[4], sa[5], sa[6], sa[7]))
}

/// One client object.
pub struct Dplay8Client {
    clock: Box<dyn Clock>,
    opener: Box<dyn SocketOpener>,
    resolver: Box<dyn HostResolver>,
    adapters: Box<dyn AdapterEnumerator>,
    random: Box<dyn RandomSource>,
    handler: Option<Box<dyn MessageHandler>>,
    core: Core,
    caps: ProtocolConfig,
    proto: Option<Protocol>,
    socket: Option<Box<dyn DatagramSocket>>,
    remote: Option<SocketAddrV4>,
    held: HashMap<DpnHandle, Vec<u8>>,
    recv_buf: Vec<u8>,
    /// Indications produced by the pump and not yet handed to the handler (or, for the bridge,
    /// to [`Dplay8Client::take_indications`]).
    outq: Vec<Indication>,
    /// `[core+8] & 0x10`: the parameter validators run (cleared by Initialize flag 1).
    param_validation: bool,
    /// The source of the signing secrets handed to the next connection (tests).
    secret_random: Option<Box<dyn RandomSource>>,
}

impl Dplay8Client {
    /// A client on the real clock, UDP sockets and resolver.
    pub fn with_system() -> Self {
        Self::new(
            Box::new(SystemClock::new()),
            Box::new(StdSocketOpener),
            Box::new(SystemResolver),
            Box::new(StdAdapters),
            Box::new(OsRandom::new()),
        )
    }

    pub fn new(
        clock: Box<dyn Clock>,
        opener: Box<dyn SocketOpener>,
        resolver: Box<dyn HostResolver>,
        adapters: Box<dyn AdapterEnumerator>,
        random: Box<dyn RandomSource>,
    ) -> Self {
        Self {
            clock,
            opener,
            resolver,
            adapters,
            random,
            handler: None,
            core: Core::new(),
            caps: ProtocolConfig::default(),
            proto: None,
            socket: None,
            remote: None,
            held: HashMap::new(),
            recv_buf: vec![0; 2048],
            outq: Vec::new(),
            param_validation: true,
            secret_random: None,
        }
    }

    /// `IDirectPlay8Client::Initialize` (DX9 0x00408fd8, dpnet9.dll.c:5298-5337): checks in
    /// order object (0x80158400), handler (0x80070057), `flags & 0xfffffff8` (0x80158340),
    /// already initialised (0x80158080). Flag 1 (`DISABLEPARAMVAL`) clears parameter validation
    /// (not modelled); flag 2 is forwarded; flag 4 (`DISABLELINKTUNING`) turns link tuning off:
    /// the send window is then a fixed 63 frames (0x00423298:30041-30057).
    // open: Initialize's call into the internal thread-pool object (5333) is not modelled.
    pub fn initialize(&mut self, handler: Box<dyn MessageHandler>, flags: u32) -> Hresult {
        let hr = self.initialize_external(flags);
        if hr == Hresult(0) {
            self.handler = Some(handler);
        }
        hr
    }

    /// `Initialize` for a caller that takes the indications itself
    /// ([`Dplay8Client::do_work_collect`]); the checks are those of [`Dplay8Client::initialize`].
    pub(crate) fn initialize_external(&mut self, flags: u32) -> Hresult {
        if flags & 0xffff_fff8 != 0 {
            return DPNERR_INVALIDFLAGS;
        }
        if self.core.state != CoreState::Uninitialized {
            return DPNERR_ALREADYINITIALIZED;
        }
        self.caps = ProtocolConfig::default();
        self.caps.link_tuning_off = flags & 4 != 0;
        self.caps.lan_hint = flags & 2 != 0;
        // flag 1 (DISABLEPARAMVAL) clears the parameter validation (`[core+8] = 0x10`, .c:5330)
        self.param_validation = flags & 1 == 0;
        self.core.state = CoreState::Idle;
        Hresult(0)
    }

    /// Test hook: the signing secrets of the next connection come from this source.
    pub fn set_secret_source(&mut self, r: Box<dyn RandomSource>) {
        self.secret_random = Some(r);
    }

    /// Test hook: every session id comes from this value instead of the random source.
    pub fn set_session_id_source(&mut self, r: Box<dyn RandomSource>) {
        self.random = r;
    }

    /// `GetConnectionInfo` (DX9 client 0x004077e3 -> FUN_004076b7 -> FUN_0040c64f -> FUN_00420a89,
    /// dpnet9.dll.c:27747-27778). `dwSize` 0x5c, 0xd4 or 0xdc (else 0x80070057); uninitialised
    /// 0x80158640, connecting 0x80158150, not connected 0x80158480 (and `FUN_0040c64f` turns the
    /// protocol's 0x80158330 into 0x80158420). With 0x5c only dwords 1..=22 are written: latency
    /// (`conn+0x3c`), throughput and peak (`conn+0x68/0x6c << 2`, zero here), the 19 counters.
    // open: throughput, retried, dropped, queued-per-priority are 0 (the timed-out counters are 0
    // in DX9 too); sent and received bytes/packets and messages received are counted. The
    // 0xd4/0xdc tail is not filled.
    pub fn get_connection_info(&self, info: &mut DpnConnectionInfo, _flags: u32) -> Hresult {
        if ![DPN_CONNECTION_INFO_SIZE, 0xd4, 0xdc].contains(&info.size) {
            return DPNERR_INVALIDPARAM;
        }
        match self.core.state {
            CoreState::Uninitialized => return DPNERR_UNINITIALIZED,
            CoreState::Connecting => return DPNERR_CONNECTING,
            CoreState::Connected => {}
            _ => return DPNERR_NOCONNECTION,
        }
        match &self.proto {
            Some(p) if p.is_connected() => {
                *info = DpnConnectionInfo {
                    size: info.size,
                    round_trip_latency_ms: p.round_trip_ms(),
                    counters: p.stats(),
                    ..DpnConnectionInfo::default()
                };
                Hresult(0)
            }
            _ => Hresult(0x8015_8420),
        }
    }

    /// `GetCaps` (dpnet 0x0041d360): the three protocol timings. `dwSize` must be 0x14.
    pub fn get_caps(&self, caps: &mut DpnCaps, _flags: u32) -> Hresult {
        if self.core.state == CoreState::Uninitialized {
            return DPNERR_UNINITIALIZED;
        }
        if caps.size != DPN_CAPS_SIZE {
            return DPNERR_INVALIDPARAM;
        }
        let (ct, cr, ka) = match &self.proto {
            Some(p) => p.caps(),
            None => (
                self.caps.connect_timeout,
                self.caps.connect_retries,
                self.caps.keepalive_timeout,
            ),
        };
        caps.connect_timeout = ct;
        caps.connect_retries = cr;
        caps.timeout_until_keep_alive = ka;
        Hresult(0)
    }

    /// `GetCaps` with `dwSize` 0x30 (DX9 0x0042093c): the base fields and the seven extended ones.
    pub fn get_caps_ex(&self, caps: &mut DpnCapsEx, flags: u32) -> Hresult {
        if caps.base.size != DPN_CAPS_EX_SIZE {
            return DPNERR_INVALIDPARAM;
        }
        let mut b = DpnCaps {
            size: DPN_CAPS_SIZE,
            ..caps.base
        };
        let hr = self.get_caps(&mut b, flags);
        if hr != Hresult(0) {
            return hr;
        }
        let c = &self.caps;
        *caps = DpnCapsEx {
            base: DpnCaps {
                size: DPN_CAPS_EX_SIZE,
                ..b
            },
            max_recv_msg_size: c.max_recv_msg_size,
            num_send_retries: c.num_send_retries,
            max_send_retry_interval: c.max_send_retry_interval,
            drop_threshold_rate: c.drop_threshold_rate,
            throttle_rate: c.throttle_rate,
            num_hard_disconnect_sends: c.num_hard_disconnect_sends,
            max_hard_disconnect_period: c.max_hard_disconnect_period,
        };
        Hresult(0)
    }

    /// `SetCaps` with `dwSize` 0x30 (DX9 0x0042098e, validator FUN_00418d9b .c:20082): the
    /// extended fields are range-checked (NumSendRetries <= 0x100, MaxSendRetryInterval
    /// 10..=60000, DropThresholdRate <= 100, ThrottleRate <= 100, NumHardDisconnectSends 2..=50,
    /// MaxHardDisconnectPeriod 10..=5000, else 0x80070057) and clamped as stored. A client
    /// forces `dwMaxRecvMsgSize` to -1 (asm 0x407420).
    // open: the stored values do not yet drive the protocol timers (it runs on the equal
    // default literals); `core+0x28 = MaxSendRetryInterval * NumSendRetries` is not modelled.
    pub fn set_caps_ex(&mut self, caps: &DpnCapsEx, flags: u32) -> Hresult {
        if caps.base.size != DPN_CAPS_EX_SIZE {
            return DPNERR_INVALIDPARAM;
        }
        if caps.num_send_retries > 0x100
            || !(10..=60000).contains(&caps.max_send_retry_interval)
            || caps.drop_threshold_rate > 100
            || caps.throttle_rate > 100
            || !(2..=50).contains(&caps.num_hard_disconnect_sends)
            || !(10..=5000).contains(&caps.max_hard_disconnect_period)
        {
            return DPNERR_INVALIDPARAM;
        }
        let hr = self.set_caps(
            &DpnCaps {
                size: DPN_CAPS_SIZE,
                ..caps.base
            },
            flags,
        );
        if hr != Hresult(0) {
            return hr;
        }
        self.caps.max_recv_msg_size = 0xffff_ffff;
        self.caps.num_send_retries = caps.num_send_retries;
        self.caps.max_send_retry_interval = caps.max_send_retry_interval;
        self.caps.drop_threshold_rate = caps.drop_threshold_rate;
        self.caps.throttle_rate = caps.throttle_rate;
        self.caps.num_hard_disconnect_sends = caps.num_hard_disconnect_sends;
        self.caps.max_hard_disconnect_period = caps.max_hard_disconnect_period;
        Hresult(0)
    }

    /// `SetCaps` (dpnet 0x0041d390).
    pub fn set_caps(&mut self, caps: &DpnCaps, _flags: u32) -> Hresult {
        if self.core.state == CoreState::Uninitialized {
            return DPNERR_UNINITIALIZED;
        }
        if caps.size != DPN_CAPS_SIZE {
            return DPNERR_INVALIDPARAM;
        }
        self.caps.connect_timeout = caps.connect_timeout;
        self.caps.connect_retries = caps.connect_retries;
        self.caps.keepalive_timeout = caps.timeout_until_keep_alive;
        if let Some(p) = &mut self.proto {
            p.set_caps(
                caps.connect_timeout,
                caps.connect_retries,
                caps.timeout_until_keep_alive,
            );
        }
        Hresult(0)
    }

    /// `IDirectPlay8Client::Connect` (dpnet 0x00409809, dpnet.dll.c:7240-7660), asynchronous
    /// (`dwFlags` 0): returns the async handle and `DPNSUCCESS_PENDING`; the result arrives as
    /// `CONNECT_COMPLETE`. `host` must carry the TCP/IP provider and `hostname` (`port` defaults to
    /// 6073); `device` supplies the local `port` (absent or 0: the provider scans 2302..2400).
    #[allow(clippy::too_many_arguments)]
    pub fn connect(
        &mut self,
        app: &DpnApplicationDesc,
        host: &DpAddress,
        device: &DpAddress,
        has_security: bool,
        has_credentials: bool,
        user_connect_data: Option<&[u8]>,
        async_context: usize,
        flags: u32,
    ) -> Result<DpnHandle, Hresult> {
        match self.core.state {
            CoreState::Uninitialized => return Err(DPNERR_UNINITIALIZED),
            CoreState::Connecting => return Err(DPNERR_CONNECTING),
            CoreState::Connected => return Err(DPNERR_ALREADYCONNECTED),
            CoreState::Closing => return Err(DPNERR_ALREADYCLOSING),
            CoreState::Idle => {}
        }
        // FUN_00413d63 (dpnet.dll.c:16660): size 0x48, flags within 0xC5.
        if app.size != DPN_APPLICATION_DESC_SIZE {
            return Err(DPNERR_INVALIDPARAM);
        }
        if app.flags & 0xffff_ff3a != 0 {
            return Err(DPNERR_INVALIDFLAGS);
        }
        if flags & !1 != 0 {
            // open: DPNCONNECT_SYNC is not ported; GunDll passes 0. Bit 0
            // (OKTOQUERYFORADDRESSING) only picks AI_NUMERICHOST for the first resolution pass
            // (0x0040e00f); the pass that returns 0x80158630 is retried with name resolution
            // allowed, so the result is the same here.
            return Err(DPNERR_INVALIDFLAGS);
        }
        if has_security || has_credentials {
            // open: security descriptors and credentials (DX8 SSPI) are not ported; GunDll
            // passes NULL for both.
            return Err(Hresult(0x8000_4001));
        }
        let now = self.clock.now_ms();
        let mut host = host.clone();
        let remote = match provider_host_sockaddr(&mut host, self.resolver.as_ref()) {
            Ok(r) => r,
            Err(e) if e == DPNERR_INVALIDHOSTADDRESS => {
                // DX9: resolution runs as a thread-pool job after Connect returned (numeric pass
                // 0x80158630, then the name pass, FUN_00436d7e dpnet9.dll.c:49286); its failure
                // reaches the application as CONNECT_COMPLETE with 0x80158370 and is not
                // remapped (FUN_0040d7a8:9615). Connect itself returned DPNSUCCESS_PENDING.
                let handle = self.core.new_handle();
                self.outq.push(Indication::ConnectComplete(ConnectComplete {
                    async_op: handle,
                    user_context: async_context,
                    result: e,
                    reply_data: Vec::new(),
                }));
                return Ok(handle);
            }
            Err(e) => return Err(e),
        };
        // The core's device fix-up (DX9 0x0040e00f:10106-10139): an address without a `device`
        // GUID gets SetDevice(first adapter); an empty adapter list fails with 0x80158320. The
        // SP then binds in mode 1: the adapter's IP, the `port` component (0 when absent).
        // A `device` GUID is the adapter's socket address XOR a fixed template (FUN_00432d61
        // dpnet9.dll.c:45455, decode FUN_00433d44:46472; template DAT_004517dc). A decoded
        // sockaddr that is not IPv4 with zero padding is 0x80158320; an address that is not
        // ours fails later at bind (0x80158080 with an exact port, else 0x80158480). Without a
        // device GUID the first enumerated adapter is used (core FUN_0040e00f:10092-10139).
        let adapter = match device.component("device") {
            Some(Component::Guid(g)) => decode_device_guid(g)?,
            _ => self
                .adapters
                .adapters(*remote.ip())
                .into_iter()
                .next()
                .ok_or(DPNERR_INVALIDDEVICEADDRESS)?,
        };
        let local_port = match device.component("port") {
            Some(Component::Dword(p)) => *p as u16,
            _ => 0,
        };
        let multi_homed = is_multi_homed(&self.adapters.adapters(*remote.ip()));
        let socket = bind_local_port(self.opener.as_mut(), adapter, local_port, multi_homed)?;
        let password: Option<Vec<u16>> = app.password.as_ref().map(|p| p.encode_utf16().collect());
        let body = build_connect_request(&ConnectRequest {
            flags: CORE_FLAG_CLIENT,
            name: None,
            data: None,
            password: password.as_deref(),
            connect_data: user_connect_data,
            guid_instance: app.guid_instance,
            guid_application: app.guid_application,
        });
        // DX9 0x00424bef:31475-31481: a random non-zero session id per connection.
        let sid = loop {
            let v = self.random.next_u32();
            if v != 0 {
                break v;
            }
        };
        let handle = self.core.new_handle();
        self.core.begin_connect(handle, async_context, body);
        let mut proto = Protocol::with_session_seed(self.caps, now, sid);
        if let Some(r) = self.secret_random.take() {
            proto.set_secret_source(r);
        }
        proto.connect(now);
        self.proto = Some(proto);
        self.socket = Some(socket);
        self.remote = Some(remote);
        Ok(handle)
    }

    /// `IDirectPlay8Client::Send` for one or more buffers (the descriptors are joined). Always
    /// copies, so `DPNSEND_NOCOPY` changes nothing here. Returns the async handle
    /// (`DPNSUCCESS_PENDING`); the result arrives as `SEND_COMPLETE` unless `NOCOMPLETE`.
    /// `timeout` is not applied (open: dpnet arms a timer, `FUN_0041e210`, only for a non-zero
    /// `dwTimeOut`; GunDll passes 0).
    pub fn send(
        &mut self,
        buffers: &[&[u8]],
        _timeout: u32,
        async_context: usize,
        flags: u32,
    ) -> Result<DpnHandle, Hresult> {
        // DX9 `Send` (FUN_00409898:5700-5985): the parameter validator (FUN_00419d26) runs
        // before the state checks. 1..=8 descriptors, else 0x80070057; only the FIRST
        // descriptor's length is tested (zero: 0x80070057); then the flag rules.
        // open: no 0x80158420 (no host player) / 0x80158160 (no connection) distinction: the
        // client has one connection; the null-pointer and phAsyncHandle checks are Rust types.
        if self.param_validation {
            if buffers.is_empty() || buffers.len() > 8 || buffers[0].is_empty() {
                return Err(DPNERR_INVALIDPARAM);
            }
            Core::validate_send_flags(flags)?;
        }
        let now = self.clock.now_ms();
        let Some(proto) = self.proto.as_mut() else {
            return Err(match self.core.state {
                CoreState::Uninitialized => DPNERR_UNINITIALIZED,
                _ => DPNERR_NOCONNECTION,
            });
        };
        let data: Vec<u8> = buffers.iter().flat_map(|b| b.iter().copied()).collect();
        self.core.send(proto, now, data, async_context, flags)
    }

    /// `IDirectPlay8Client::ReturnBuffer`.
    pub fn return_buffer(&mut self, handle: DpnHandle, _flags: u32) -> Hresult {
        let now = self.clock.now_ms();
        self.held.remove(&handle);
        match self.proto.as_mut() {
            Some(proto) => self.core.return_buffer(proto, now, handle),
            None => DPNERR_INVALIDHANDLE,
        }
    }

    /// `IDirectPlay8Client::CancelAsyncOperation` for a send that has not gone out.
    pub fn cancel_async_operation(&mut self, handle: DpnHandle, flags: u32) -> Hresult {
        let now = self.clock.now_ms();
        // DPNCANCEL_CONNECT (1) / DPNCANCEL_SEND (4) with handle 0 cancel every such operation.
        if handle.0 == 0 {
            if flags & 1 != 0 {
                if let Some(p) = self.proto.as_mut() {
                    let _ = p.cancel_connect(now, DPNERR_USERCANCEL);
                }
            }
            if flags & 4 != 0 {
                if let Some(p) = self.proto.as_mut() {
                    self.core.cancel_unstarted(p, now);
                }
            }
            return Hresult(0);
        }
        match self.proto.as_mut() {
            Some(proto) => self.core.cancel_send(proto, now, handle),
            None => DPNERR_INVALIDHANDLE,
        }
    }

    /// `IDirectPlay8Client::Close` (DX9 0x0040b856, dpnet9.dll.c:7428-7600). In order: flags
    /// other than `DPNCLOSE_IMMEDIATE` (1) are 0x80158340; not initialised 0x80158640; already
    /// closing 0x80158050. Then the connect is cancelled, everything unstarted completes with
    /// `DPNERR_USERCANCEL`, and each connection gets `FUN_0040cb7f` -> `FUN_004214b8`: without the
    /// flag an END_STREAM is queued (cmd 0x27, 0x10000c), with it a HARD_DISCONNECT goes out at
    /// once and twice more at `clamp(rtt/2, 10, 500)` ms. **There is no drain loop**: DX9 has no
    /// counterpart of 8.0's 500 x Sleep(10) (FUN_00420510); Close returns once the disconnect is
    /// queued, and the protocol data is freed. This port pumps once so a queued datagram leaves,
    /// then drops the connection.
    // open: whether the END_STREAM / later HARD_DISCONNECTs leave in the binary when no DoWork
    // follows Close (the send pump is a thread-pool work item, 0x00423d7a) is not determined; the
    // single pump here is the closest observable behaviour. The 0x80158520 return for a Close
    // from inside a callback is raised by the bridge, which knows when a handler is running.
    pub fn close(&mut self, flags: u32) -> Hresult {
        if flags & !1 != 0 {
            return DPNERR_INVALIDFLAGS;
        }
        if self.core.state == CoreState::Uninitialized {
            return DPNERR_UNINITIALIZED;
        }
        if self.core.state == CoreState::Closing {
            return DPNERR_ALREADYCLOSING;
        }
        let now = self.clock.now_ms();
        let connected = self.core.state == CoreState::Connected;
        self.core.state = CoreState::Closing;
        if let Some(proto) = self.proto.as_mut() {
            self.core.cancel_unstarted(proto, now);
            if connected && proto.is_connected() {
                if flags & 1 != 0 {
                    proto.hard_disconnect(now);
                } else {
                    let _ = proto.graceful_disconnect(now, u32::MAX);
                }
            } else if !connected && !proto.is_dead() {
                let _ = proto.cancel_connect(now, DPNERR_USERCANCEL);
            }
        }
        self.pump(true);
        self.deliver_queued();
        self.proto = None;
        self.socket = None;
        self.remote = None;
        self.held.clear();
        self.core = Core::new();
        self.core.state = CoreState::Idle;
        Hresult(0)
    }

    /// The thread pool's `DoWork`: receive, run timers and ready work, send, and call the
    /// handler for what came out. Returns `S_OK`.
    pub fn do_work(&mut self) -> Hresult {
        self.pump(true);
        self.deliver_queued();
        Hresult(0)
    }

    /// `DoWork` for a caller that delivers the indications itself with no borrow of the client
    /// held across the handler (the `dp8_bridge`): the pump runs, the indications come back.
    /// Each `Receive` must then be settled with [`Dplay8Client::finish_receive`].
    pub(crate) fn do_work_collect(&mut self) -> Vec<Indication> {
        self.pump(true);
        self.take_indications()
    }

    /// The indications queued so far, in order.
    pub(crate) fn take_indications(&mut self) -> Vec<Indication> {
        std::mem::take(&mut self.outq)
    }

    /// Milliseconds the timer wheel wants between pumps (the tick period).
    pub fn timer_period(&self) -> u32 {
        self.proto.as_ref().map(|p| p.timer_period()).unwrap_or(4)
    }

    fn pump(&mut self, notify: bool) {
        let mut rounds = 0;
        loop {
            rounds += 1;
            let now = self.clock.now_ms();
            let mut activity = false;
            if let (Some(sock), Some(proto), Some(remote)) =
                (self.socket.as_mut(), self.proto.as_mut(), self.remote)
            {
                // The SP's receive demux: a first byte of 0 is an SP-level frame (enumeration),
                // anything else is protocol payload (dpnwsock.dll.c:7561-7576). Datagrams from
                // anyone but the host are dropped.
                while let Ok(Some((n, from))) = sock.recv_from(&mut self.recv_buf) {
                    if from == remote && n > 0 && self.recv_buf[0] != 0 {
                        proto.on_datagram(now, &self.recv_buf[..n]);
                        activity = true;
                    }
                }
                proto.do_work(now);
                for d in proto.take_outbox() {
                    let _ = sock.send_to(&d, remote);
                    activity = true;
                }
            }
            let mut out: Vec<Indication> = Vec::new();
            if let Some(proto) = self.proto.as_mut() {
                for ev in proto.take_events() {
                    self.core.on_event(proto, now, ev, &mut out);
                    activity = true;
                }
            }
            if let (Some(sock), Some(proto), Some(remote)) =
                (self.socket.as_mut(), self.proto.as_mut(), self.remote)
            {
                for d in proto.take_outbox() {
                    let _ = sock.send_to(&d, remote);
                }
            }
            if notify {
                self.outq.extend(out);
            }
            if !activity || rounds > 64 {
                break;
            }
        }
    }

    /// Call the handler. A RECEIVE whose handler returns `DPNSUCCESS_PENDING` keeps its buffer
    /// until `return_buffer`, any other result frees it (`FUN_004195a3`).
    fn deliver_queued(&mut self) {
        if self.handler.is_none() {
            return;
        }
        for ind in self.take_indications() {
            let Some(handler) = self.handler.as_mut() else {
                continue;
            };
            match ind {
                Indication::ConnectComplete(m) => {
                    handler.message(DpnMessage::ConnectComplete(&m));
                }
                Indication::SendComplete(m) => {
                    handler.message(DpnMessage::SendComplete(&m));
                }
                Indication::TerminateSession(m) => {
                    handler.message(DpnMessage::TerminateSession(&m));
                }
                Indication::Receive {
                    data,
                    buffer,
                    e0_context,
                } => {
                    let r = handler.message(DpnMessage::Receive(Receive {
                        sender: 0,
                        data: &data,
                        buffer,
                    }));
                    self.finish_receive(buffer, data, e0_context, r);
                }
            }
        }
    }

    /// What follows the handler's answer to a `RECEIVE` (`FUN_004195a3`): `DPNSUCCESS_PENDING`
    /// keeps the buffer until `return_buffer`; any other result returns it at once, and a 0xE0
    /// request is then answered.
    pub(crate) fn finish_receive(
        &mut self,
        buffer: DpnHandle,
        data: Vec<u8>,
        e0_context: Option<u32>,
        result: Hresult,
    ) {
        if result == DPNSUCCESS_PENDING {
            self.held.insert(buffer, data);
        } else {
            self.core.buffers.remove(&buffer);
            if let (Some(ctx), Some(proto)) = (e0_context, self.proto.as_mut()) {
                let now = self.clock.now_ms();
                self.core.answer_e0(proto, now, ctx);
            }
        }
    }

    /// Bytes of a held RECEIVE buffer (valid until `return_buffer`).
    pub fn held_buffer(&self, handle: DpnHandle) -> Option<&[u8]> {
        self.held.get(&handle).map(|v| v.as_slice())
    }

    /// The core's state (tests and the lead's glue).
    pub fn state(&self) -> CoreState {
        self.core.state
    }

    /// The local port of the connect socket, once bound.
    pub fn local_port(&self) -> Option<u16> {
        self.socket
            .as_ref()
            .and_then(|s| s.local_addr().ok())
            .map(|a| a.port())
    }
}
