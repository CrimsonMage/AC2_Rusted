//! A [`Transport`] driving [`ClientNet`] one frame per [`NetClient::pump`], as the retail
//! client runs its network once per frame on the main thread. The default transport is a UDP
//! socket on the system clock; tests substitute a scripted one.

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::handshake::Logon;
use crate::net::{ClientNet, NetEvent};
use crate::NetError;

/// Pause between frames while blocking for an event.
const FRAME: Duration = Duration::from_millis(10);

#[derive(Debug, Clone)]
pub struct NetClientConfig {
    /// The server's logon address; the connect-ack goes to its port plus one.
    pub server: SocketAddr,
    pub logon: Logon,
}

/// Where datagrams and time come from.
pub trait Transport {
    /// Every datagram that arrived since the last call.
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError>;
    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError>;
    /// Seconds since the client started (`Timer` elapsed time) and Unix seconds
    /// (`Timer::get_real_time`).
    fn clock(&mut self) -> (f64, i64);
    /// Waits between frames.
    fn sleep(&mut self, duration: Duration);
}

/// A UDP socket on the system clock.
pub struct UdpTransport {
    socket: UdpSocket,
    start: Instant,
    buffer: Vec<u8>,
}

impl UdpTransport {
    /// Binds a local socket of `server`'s address family.
    pub fn bind(server: SocketAddr) -> Result<Self, NetError> {
        Self::bind_port(server, 0)
    }

    /// [`Self::bind`] on local `port` (`ConnectionOptions::dwPort`, the client's `outport`); 0
    /// takes one the system picks.
    pub fn bind_port(server: SocketAddr, port: u16) -> Result<Self, NetError> {
        let local: SocketAddr = if server.is_ipv4() {
            SocketAddr::from(([0, 0, 0, 0], port))
        } else {
            SocketAddr::from(([0u16; 8], port))
        };
        let socket = UdpSocket::bind(local)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            start: Instant::now(),
            buffer: vec![0; 0x1_0000],
        })
    }
}

impl Transport for UdpTransport {
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
        let mut inbound = Vec::new();
        loop {
            match self.socket.recv_from(&mut self.buffer) {
                Ok((len, from)) => inbound.push((from, self.buffer[..len].to_vec())),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                // ICMP port-unreachable surfaces as a reset on some platforms.
                Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(inbound)
    }

    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
        self.socket.send_to(datagram, to)?;
        Ok(())
    }

    fn clock(&mut self) -> (f64, i64) {
        (self.start.elapsed().as_secs_f64(), real_time())
    }

    fn sleep(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// A connection to an AC2 server.
pub struct NetClient {
    transport: Box<dyn Transport>,
    net: ClientNet,
    pending: VecDeque<NetEvent>,
    closed: bool,
    /// The configuration of a connection that has not started yet ([`NetClient::with_transport_held`]):
    /// `Turbine::Client::Connect` runs `InitAuth` and then `ClientNet::Init`, and the GUN start
    /// (`gmClient::Connect`'s `StartupGUN`) comes before both.
    held: Option<NetClientConfig>,
}

/// `Timer::get_real_time`: Unix seconds.
pub fn real_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

impl NetClient {
    /// Binds a local socket; the logon goes out at the first [`NetClient::pump`].
    pub fn connect(config: NetClientConfig) -> Result<Self, NetError> {
        let transport = UdpTransport::bind(config.server)?;
        Ok(Self::with_transport(config, Box::new(transport)))
    }

    /// Runs over `transport` instead of a socket.
    pub fn with_transport(config: NetClientConfig, transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            net: ClientNet::new(config.server, &config.logon),
            pending: VecDeque::new(),
            closed: false,
            held: None,
        }
    }

    /// [`NetClient::with_transport`] for a connection that waits: nothing is sent or read until
    /// [`NetClient::start`] gives the logon its credential (`gmClient::Connect`: `StartupGUN`, then
    /// `Turbine::Client::Connect` = `InitAuth` + `ClientNet::Init`).
    pub fn with_transport_held(config: NetClientConfig, transport: Box<dyn Transport>) -> Self {
        let mut client = Self::with_transport(config.clone(), transport);
        client.held = Some(config);
        client
    }

    /// Whether the connection still waits for [`NetClient::start`].
    pub fn is_held(&self) -> bool {
        self.held.is_some()
    }

    /// `ClientNet::Init` with the logon carrying `credential` (`gmClient::InitAuth`'s
    /// `NetAuthenticator`): the logon goes out at the next frame. No effect on a connection that
    /// is not held.
    pub fn start(&mut self, credential: crate::Credential) {
        if let Some(mut config) = self.held.take() {
            config.logon.credential = credential;
            self.net = ClientNet::new(config.server, &config.logon);
        }
    }

    /// The transport's elapsed seconds.
    pub fn elapsed(&mut self) -> f64 {
        self.transport.clock().0
    }

    /// Waits between frames on the transport's clock.
    pub fn sleep(&mut self, duration: Duration) {
        self.transport.sleep(duration);
    }

    pub fn net(&self) -> &ClientNet {
        &self.net
    }

    pub fn net_mut(&mut self) -> &mut ClientNet {
        &mut self.net
    }

    /// One frame: read what arrived, run [`ClientNet::use_time`], send what it produced.
    pub fn pump(&mut self) -> Result<(), NetError> {
        if self.closed || self.held.is_some() {
            return Ok(());
        }
        let inbound = self.transport.receive()?;
        let (elapsed, real) = self.transport.clock();
        let (out, events) = self.net.use_time(elapsed, real, inbound);
        for (to, datagram) in out {
            self.transport.send(to, &datagram)?;
        }
        for event in events {
            if matches!(event, NetEvent::Disconnected(_)) {
                self.closed = true;
            }
            self.pending.push_back(event);
        }
        Ok(())
    }

    /// Queues a blob for the server on `queue` (`Proto_UI::SendBuff`); it leaves next frame.
    pub fn send(&mut self, queue: u16, data: Vec<u8>) -> Option<u64> {
        self.net.send_buff(queue, data)
    }

    /// Pumps one frame and returns every event since the last call.
    pub fn poll(&mut self) -> Vec<NetEvent> {
        if let Err(error) = self.pump() {
            self.closed = true;
            self.pending.push_back(NetEvent::Disconnected(error));
        }
        self.pending.drain(..).collect()
    }

    /// The link averages of the last world heartbeat, once (see
    /// [`ClientNet::take_link_heartbeat`]).
    pub fn take_link_heartbeat(&mut self) -> Option<Box<crate::LinkAverages>> {
        self.net.take_link_heartbeat()
    }

    /// Pumps frames until an event arrives or `timeout` passes.
    pub fn next_event(&mut self, timeout: Duration) -> Option<NetEvent> {
        let deadline = self.elapsed() + timeout.as_secs_f64();
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Some(event);
            }
            if self.closed || self.elapsed() >= deadline {
                return None;
            }
            if let Err(error) = self.pump() {
                self.closed = true;
                return Some(NetEvent::Disconnected(error));
            }
            if self.pending.is_empty() {
                self.transport.sleep(FRAME);
            }
        }
    }

    /// `ClientNet::LogOffServer`: the logoff goes out now; nothing is sent after.
    pub fn disconnect(&mut self) {
        if self.closed || self.held.is_some() {
            return;
        }
        self.net.log_off_server();
        for (to, datagram) in self.net.take_datagrams() {
            let _ = self.transport.send(to, &datagram);
        }
        self.closed = true;
    }
}

impl Drop for NetClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod held_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::handshake::Credential;

    /// A transport that records what is sent.
    struct Recording(Rc<RefCell<Vec<Vec<u8>>>>);

    impl Transport for Recording {
        fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
            Ok(Vec::new())
        }
        fn send(&mut self, _to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
            self.0.borrow_mut().push(datagram.to_vec());
            Ok(())
        }
        fn clock(&mut self) -> (f64, i64) {
            (1.0, 1_700_000_000)
        }
        fn sleep(&mut self, _duration: Duration) {}
    }

    fn config() -> NetClientConfig {
        NetClientConfig {
            server: "127.0.0.1:9000".parse().unwrap(),
            logon: Logon {
                version: "7160".into(),
                account: "a".into(),
                credential: Credential::Account,
                connection_sequence: 1,
                crypto: None,
            },
        }
    }

    /// `gmClient::Connect` runs `StartupGUN` before `Turbine::Client::Connect`: a held
    /// connection sends nothing until it is started, and then the logon carries the
    /// `InitAuth` credential (auth type 0x41000001 with the ticket bytes).
    #[test]
    fn a_held_connection_sends_nothing_until_started() {
        let sent = Rc::new(RefCell::new(Vec::new()));
        let mut client =
            NetClient::with_transport_held(config(), Box::new(Recording(Rc::clone(&sent))));
        assert!(client.is_held());
        for _ in 0..3 {
            assert!(client.poll().is_empty());
        }
        assert!(sent.borrow().is_empty());
        client.start(Credential::GunTicket(vec![9, 8, 7]));
        assert!(!client.is_held());
        client.poll();
        let sent = sent.borrow();
        assert_eq!(
            sent.len(),
            1,
            "the logon request goes out at the first frame"
        );
        let ticket = [9u8, 8, 7];
        assert!(sent[0].windows(3).any(|w| w == ticket));
        let auth_type = 0x4100_0001u32.to_le_bytes();
        assert!(sent[0].windows(4).any(|w| w == auth_type));
    }
}
