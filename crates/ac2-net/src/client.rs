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
        let local: SocketAddr = if server.is_ipv4() {
            "0.0.0.0:0".parse().expect("valid address")
        } else {
            "[::]:0".parse().expect("valid address")
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
        if self.closed {
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
        if self.closed {
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
