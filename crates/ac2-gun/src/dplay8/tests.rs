//! End-to-end tests of the client against a scripted fake host over a fake socket and clock.
//!
//! Every byte the client sends is compared with a literal written from the spec (spec_client.md
//! and its adjudication) and the decompile, never with the encoder's own output. The timing
//! expectations come from the timer-wheel arithmetic in `protocol::timers` (dpnet 0x00427020 and
//! 0x00427620) with the pump ticking every 20 ms like the default `timeSetEvent` period; each
//! test says how its numbers follow.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::rc::Rc;

use crate::Hresult;

use super::address::*;
use super::client::*;
use super::clock::{Clock, FakeClock, FixedRandom};
use super::hresult::*;
use super::messages::*;
use super::sp::{DatagramSocket, SocketOpener};

pub(crate) const HOST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 5), 2300);
pub(crate) const ADAPTER: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);
const SID: u32 = 0x1122_3344;

pub(crate) struct FakeAdapters(pub(crate) Vec<Ipv4Addr>);
impl super::sp::AdapterEnumerator for FakeAdapters {
    fn adapters(&self, _: Ipv4Addr) -> Vec<Ipv4Addr> {
        self.0.clone()
    }
}
pub(crate) const APP_GUID: [u8; 16] = [
    0x7d, 0x76, 0x80, 0x22, 0x89, 0x16, 0x6b, 0x4d, 0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f, 0xb6, 0x55,
];

/// A scripted host that reacts to what the client sends.
pub(crate) type Responder = Box<dyn FnMut(&[u8]) -> Vec<Vec<u8>>>;

#[derive(Default)]
pub(crate) struct Net {
    pub responder: Option<Responder>,
    pub to_host: Vec<Vec<u8>>,
    pub from_host: VecDeque<Vec<u8>>,
    pub binds: Vec<u16>,
    pub busy: Vec<u16>,
}

struct FakeSock {
    net: Rc<RefCell<Net>>,
    local: SocketAddrV4,
}

impl DatagramSocket for FakeSock {
    fn send_to(&mut self, data: &[u8], to: SocketAddrV4) -> io::Result<usize> {
        assert_eq!(to, HOST);
        let mut n = self.net.borrow_mut();
        n.to_host.push(data.to_vec());
        if let Some(mut r) = n.responder.take() {
            let replies = r(data);
            n.responder = Some(r);
            n.from_host.extend(replies);
        }
        Ok(data.len())
    }
    fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<Option<(usize, SocketAddrV4)>> {
        match self.net.borrow_mut().from_host.pop_front() {
            Some(d) => {
                buf[..d.len()].copy_from_slice(&d);
                Ok(Some((d.len(), HOST)))
            }
            None => Ok(None),
        }
    }
    fn local_addr(&self) -> io::Result<SocketAddrV4> {
        Ok(self.local)
    }
}

pub(crate) struct FakeOpener(pub(crate) Rc<RefCell<Net>>);

impl SocketOpener for FakeOpener {
    fn bind(&mut self, ip: Ipv4Addr, port: u16, _ex: bool) -> io::Result<Box<dyn DatagramSocket>> {
        let mut n = self.0.borrow_mut();
        assert_eq!(ip, ADAPTER, "the first adapter's address is bound");
        n.binds.push(port);
        if n.busy.contains(&port) {
            return Err(io::Error::from(io::ErrorKind::AddrInUse));
        }
        let p = if port == 0 { 49152 } else { port };
        Ok(Box::new(FakeSock {
            net: self.0.clone(),
            local: SocketAddrV4::new(ip, p),
        }))
    }
}

pub(crate) struct NoResolver;
impl HostResolver for NoResolver {
    fn resolve(&self, _: &str) -> Option<Vec<Ipv4Addr>> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Got {
    Connect(Hresult, Vec<u8>, usize),
    Receive(Vec<u8>, DpnHandle),
    Send(Hresult, usize, u32),
    Terminate(Hresult, Vec<u8>),
}

struct Recorder {
    log: Rc<RefCell<Vec<Got>>>,
    keep: bool,
}

impl MessageHandler for Recorder {
    fn message(&mut self, msg: DpnMessage<'_>) -> Hresult {
        match msg {
            DpnMessage::ConnectComplete(m) => self.log.borrow_mut().push(Got::Connect(
                m.result,
                m.reply_data.clone(),
                m.user_context,
            )),
            DpnMessage::Receive(r) => {
                self.log
                    .borrow_mut()
                    .push(Got::Receive(r.data.to_vec(), r.buffer));
                if self.keep {
                    return DPNSUCCESS_PENDING;
                }
            }
            DpnMessage::SendComplete(m) => {
                self.log
                    .borrow_mut()
                    .push(Got::Send(m.result, m.user_context, m.send_time))
            }
            DpnMessage::TerminateSession(m) => self
                .log
                .borrow_mut()
                .push(Got::Terminate(m.result, m.data.clone())),
            DpnMessage::AsyncOpComplete(_) => {}
        }
        Hresult(0)
    }
}

pub(crate) struct H {
    pub client: Dplay8Client,
    pub clock: FakeClock,
    pub net: Rc<RefCell<Net>>,
    pub log: Rc<RefCell<Vec<Got>>>,
}

pub(crate) fn app_desc() -> DpnApplicationDesc {
    DpnApplicationDesc {
        size: DPN_APPLICATION_DESC_SIZE,
        flags: DPNSESSION_CLIENT_SERVER,
        guid_instance: [0; 16],
        guid_application: APP_GUID,
        max_players: 0,
        current_players: 0,
        session_name: None,
        password: None,
    }
}

impl H {
    /// GunDll's setup: Initialize flag 4 (fixed 63-frame window).
    pub fn new() -> Self {
        Self::with_keep(true, 4)
    }

    pub fn with_keep(keep: bool, init_flags: u32) -> Self {
        let clock = FakeClock::new(10_000);
        let net = Rc::new(RefCell::new(Net::default()));
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut client = Dplay8Client::new(
            Box::new(clock.clone()),
            Box::new(FakeOpener(net.clone())),
            Box::new(NoResolver),
            Box::new(FakeAdapters(vec![ADAPTER])),
            Box::new(FixedRandom(vec![SID], 0)),
        );
        assert_eq!(
            client.initialize(
                Box::new(Recorder {
                    log: log.clone(),
                    keep
                }),
                8
            ),
            DPNERR_INVALIDFLAGS
        );
        assert_eq!(
            client.initialize(
                Box::new(Recorder {
                    log: log.clone(),
                    keep
                }),
                init_flags
            ),
            Hresult(0)
        );
        Self {
            client,
            clock,
            net,
            log,
        }
    }

    /// GunDll's `GetCaps; dwTimeoutUntilKeepAlive = 15000; SetCaps`.
    pub fn gundll_caps(&mut self) {
        let mut c = DpnCaps {
            size: DPN_CAPS_SIZE,
            ..DpnCaps::default()
        };
        assert_eq!(self.client.get_caps(&mut c, 0), Hresult(0));
        assert_eq!(
            (
                c.connect_timeout,
                c.connect_retries,
                c.timeout_until_keep_alive
            ),
            (200, 14, 25000)
        );
        c.timeout_until_keep_alive = 15000;
        assert_eq!(self.client.set_caps(&c, 0), Hresult(0));
    }

    /// One 20 ms tick, then `DoWork`.
    pub fn tick(&mut self) {
        self.clock.advance(20);
        self.client.do_work();
    }

    pub fn ticks(&mut self, n: u32) {
        for _ in 0..n {
            self.tick();
        }
    }

    pub fn now(&self) -> u32 {
        self.clock.now_ms()
    }

    pub fn sent(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.net.borrow_mut().to_host)
    }

    pub fn host(&mut self, d: &[u8]) {
        self.net.borrow_mut().from_host.push_back(d.to_vec());
    }

    pub fn got(&mut self) -> Vec<Got> {
        std::mem::take(&mut *self.log.borrow_mut())
    }
}

fn le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

/// A CFRAME written out byte by byte (spec_client.md 2.2; version 0x00010006 in DX9, 2.2.3).
pub(crate) fn cframe(poll: bool, op: u8, mid: u8, rsp: u8, sid: u32, tick: u32) -> Vec<u8> {
    let mut v = vec![
        if poll { 0x88 } else { 0x80 },
        op,
        mid,
        rsp,
        0x06,
        0x00,
        0x01,
        0x00,
    ];
    v.extend_from_slice(&le(sid));
    v.extend_from_slice(&le(tick));
    v
}

#[test]
fn initialize_flags_caps_and_state() {
    let mut h = H::new();
    // A second Initialize is rejected; GetCaps reports the protocol defaults.
    assert_eq!(
        h.client.initialize(
            Box::new(Recorder {
                log: h.log.clone(),
                keep: true
            }),
            0
        ),
        DPNERR_ALREADYINITIALIZED
    );
    h.gundll_caps();
    let mut c = DpnCaps {
        size: DPN_CAPS_SIZE,
        ..DpnCaps::default()
    };
    h.client.get_caps(&mut c, 0);
    assert_eq!(c.timeout_until_keep_alive, 15000);
    let mut bad = DpnCaps {
        size: 0x10,
        ..DpnCaps::default()
    };
    assert_eq!(h.client.get_caps(&mut bad, 0), DPNERR_INVALIDPARAM);
}

#[test]
fn connect_validation_errors() {
    let mut h = H::new();
    let host = DpAddress::build_address("10.0.0.5", 2300);
    let dev = DpAddress::new();
    let mut bad = app_desc();
    bad.size = 0x40;
    assert_eq!(
        h.client
            .connect(&bad, &host, &dev, false, false, None, 0, 0),
        Err(DPNERR_INVALIDPARAM)
    );
    let mut bad = app_desc();
    bad.flags = 0x2;
    assert_eq!(
        h.client
            .connect(&bad, &host, &dev, false, false, None, 0, 0),
        Err(DPNERR_INVALIDFLAGS)
    );
    let no_host = {
        let mut a = DpAddress::new();
        a.set_sp(CLSID_DP8SP_TCPIP);
        a
    };
    assert_eq!(
        h.client
            .connect(&app_desc(), &no_host, &dev, false, false, None, 0, 0),
        Err(DPNERR_INCOMPLETEADDRESS)
    );
    let unknown = DpAddress::build_address("no.such.host", 1);
    // DX9: the failed name resolution is reported by CONNECT_COMPLETE, not by Connect.
    assert_eq!(
        h.client
            .connect(&app_desc(), &unknown, &dev, false, false, None, 7, 0),
        Ok(DpnHandle(1))
    );
    h.tick();
    assert_eq!(
        h.got(),
        vec![Got::Connect(DPNERR_INVALIDHOSTADDRESS, vec![], 7)]
    );
    // A device GUID is the sockaddr XOR the template: 10.0.0.2 decodes, a bad family is 0x80158320.
    let mut g = [0u8; 16];
    let sa = [2u8, 0, 0, 0, 10, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0];
    let t = [
        0xf4u8, 0x25, 0xe7, 0x4c, 0x00, 0x7b, 0x97, 0x43, 0xba, 0x6f, 0x11, 0xf9, 0x65, 0xbc, 0x42,
        0x99,
    ];
    for i in 0..16 {
        g[i] = sa[i] ^ t[i];
    }
    assert_eq!(decode_device_guid(&g), Ok(ADAPTER));
    g[0] ^= 1;
    assert_eq!(decode_device_guid(&g), Err(DPNERR_INVALIDDEVICEADDRESS));
    // A device without a port searches from 2302 (a single-homed host).
    h.net.borrow_mut().busy = vec![2302, 2303];
    let hd = h
        .client
        .connect(&app_desc(), &host, &dev, false, false, None, 0, 0);
    assert!(hd.is_ok());
    assert_eq!(h.net.borrow().binds, vec![2302, 2303, 2304]);
    assert_eq!(
        h.client
            .connect(&app_desc(), &host, &dev, false, false, None, 0, 0),
        Err(DPNERR_CONNECTING)
    );
}
