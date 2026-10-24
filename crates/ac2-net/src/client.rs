//! A UDP socket driving [`ClientNet`] one frame per [`NetClient::pump`], as the retail client
//! runs its network once per frame on the main thread.

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

/// A connection to an AC2 server.
pub struct NetClient {
    socket: UdpSocket,
    net: ClientNet,
    start: Instant,
    pending: VecDeque<NetEvent>,
    closed: bool,
    buffer: Vec<u8>,
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
        let local: SocketAddr = if config.server.is_ipv4() {
            "0.0.0.0:0".parse().expect("valid address")
        } else {
            "[::]:0".parse().expect("valid address")
        };
        let socket = UdpSocket::bind(local)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            net: ClientNet::new(config.server, &config.logon),
            start: Instant::now(),
            pending: VecDeque::new(),
            closed: false,
            buffer: vec![0; 0x1_0000],
        })
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
        let elapsed = self.start.elapsed().as_secs_f64();
        let (out, events) = self.net.use_time(elapsed, real_time(), inbound);
        for (to, datagram) in out {
            self.socket.send_to(&datagram, to)?;
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
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Some(event);
            }
            if self.closed || Instant::now() >= deadline {
                return None;
            }
            if let Err(error) = self.pump() {
                self.closed = true;
                return Some(NetEvent::Disconnected(error));
            }
            if self.pending.is_empty() {
                std::thread::sleep(FRAME);
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
            let _ = self.socket.send_to(&datagram, to);
        }
        self.closed = true;
    }
}

impl Drop for NetClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}
