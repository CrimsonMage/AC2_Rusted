//! The lossy-session harness: the client's `NetClient` over the server double, behind a seeded
//! [`Channel`], one 64 Hz frame at a time, every frame recorded for the oracle.

#![allow(dead_code)]

use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use ac2_net::{Credential, Logon, NetClient, NetClientConfig, NetError, NetEvent, Transport};
use legions_server_double::{addr, double, Channel, Conditions, Double, Spawn, State, SERVER};

use crate::oracle::Frame;

/// Seconds per frame.
pub const STEP: f64 = 1.0 / 64.0;

/// The double's transport, recording each frame for the oracle.
struct Recorder {
    inner: Double,
    frames: Rc<RefCell<Vec<Frame>>>,
}

impl Transport for Recorder {
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
        let inbound = self.inner.receive()?;
        let time = self.inner.clock().0;
        self.frames.borrow_mut().push(Frame {
            time,
            inbound: inbound.iter().map(|(_, b)| b.clone()).collect(),
            outbound: Vec::new(),
        });
        Ok(inbound)
    }

    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
        if let Some(frame) = self.frames.borrow_mut().last_mut() {
            frame.outbound.push((to, datagram.to_vec()));
        }
        self.inner.send(to, datagram)
    }

    fn clock(&mut self) -> (f64, i64) {
        self.inner.clock()
    }

    fn sleep(&mut self, duration: Duration) {
        self.inner.sleep(duration);
    }
}

fn config() -> NetClientConfig {
    NetClientConfig {
        server: addr(SERVER),
        logon: Logon {
            version: ac2_net::LEGIONS_VERSION.into(),
            account: "local".into(),
            credential: Credential::Account,
            connection_sequence: legions_server_double::REAL as u32,
            crypto: None,
        },
    }
}

/// A blob's bytes: an opcode the double does not act on, then `len - 4` bytes from `n`.
pub fn payload(n: u32, len: usize) -> Vec<u8> {
    let mut out = (0xF000_0000 | n).to_le_bytes().to_vec();
    out.extend((0..len.saturating_sub(4)).map(|i| (n as usize * 31 + i) as u8));
    out
}

/// A client session over the lossy channel.
pub struct Session {
    pub client: NetClient,
    pub state: Rc<RefCell<State>>,
    frames: Rc<RefCell<Vec<Frame>>>,
    /// Every event, with the frame time it came in.
    pub events: Vec<(f64, NetEvent)>,
}

impl Session {
    pub fn new(seed: u64, conditions: Conditions) -> Self {
        let (inner, state) = double(Spawn::default());
        state.borrow_mut().channel = Some(Channel::new(seed, conditions));
        let frames = Rc::new(RefCell::new(Vec::new()));
        let recorder = Recorder {
            inner,
            frames: Rc::clone(&frames),
        };
        Self {
            client: NetClient::with_transport(config(), Box::new(recorder)),
            state,
            frames,
            events: Vec::new(),
        }
    }

    pub fn now(&self) -> f64 {
        self.state.borrow().elapsed
    }

    /// One frame: the client's `UseTime`, then the clock moves on. Returns its events.
    pub fn frame(&mut self) -> Vec<NetEvent> {
        let now = self.now();
        let events = self.client.poll();
        self.events
            .extend(events.iter().map(|event| (now, event.clone())));
        self.state.borrow_mut().elapsed += STEP;
        events
    }

    /// Frames until the client is logged on.
    pub fn log_on(&mut self) {
        for _ in 0..64 * 600 {
            if self.frame().contains(&NetEvent::LoggedOn) {
                return;
            }
        }
        panic!("no logon");
    }

    /// The recorded frames so far.
    pub fn frames(&self) -> Vec<Frame> {
        self.frames.borrow().clone()
    }
}

/// The client's standalone logoff (`ClientNet::LogOffServer` -> `SendOptionalHeader`).
pub fn logoff_datagram() -> Vec<u8> {
    legions_server_double::datagram(
        ac2_net::PacketHeader {
            flags: ac2_net::flags::LOGOFF,
            recipient: legions_server_double::NET_ID as u16,
            iteration: 1,
            ..Default::default()
        },
        &[],
        None,
    )
}

pub fn lossy() -> Conditions {
    Conditions {
        drop: 0.1,
        duplicate: 0.05,
        delay: 0.1,
        max_delay: 0.4,
    }
}
