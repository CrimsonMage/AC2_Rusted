//! The socket thread: logon retries, the connect/connect-ack handshake, then a [`Session`] fed
//! from the socket and from [`NetClient::send`].

use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::handshake::{Connect, Logon};
use crate::header::{flags, PacketHeader};
use crate::packet::decode_handshake;
use crate::session::{Blob, Session};
use crate::NetError;

/// Interval between logon attempts and between connect-ack repeats.
const RETRY: Duration = Duration::from_millis(500);
/// Connect-acks sent before giving up on the server answering on its secondary port.
const CONNECT_ACK_ATTEMPTS: u32 = 5;
/// Socket read timeout, which is also the timed-work cadence.
const TICK: Duration = Duration::from_millis(10);

#[derive(Debug, Clone)]
pub struct NetClientConfig {
    /// The server's logon address; the connect-ack goes to its port plus one.
    pub server: SocketAddr,
    pub logon: Logon,
    /// How long to retry the logon before failing.
    pub connect_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetEvent {
    /// The handshake finished; `net_id` is this client's recipient id.
    Connected {
        net_id: u32,
        server_time: f64,
    },
    Blob(Blob),
    TimeSync(f64),
    /// The session ended; no further events follow.
    Disconnected(NetError),
}

enum Command {
    Send { queue: u16, data: Vec<u8> },
    Disconnect,
}

/// A connection to an AC2 server, running on its own thread.
pub struct NetClient {
    commands: Sender<Command>,
    events: Receiver<NetEvent>,
    thread: Option<JoinHandle<()>>,
}

impl NetClient {
    /// Binds a local socket and starts the handshake; progress arrives as [`NetEvent`]s.
    pub fn connect(config: NetClientConfig) -> Result<Self, NetError> {
        let local: SocketAddr = if config.server.is_ipv4() {
            "0.0.0.0:0".parse().expect("valid address")
        } else {
            "[::]:0".parse().expect("valid address")
        };
        let socket = UdpSocket::bind(local)?;
        socket.set_read_timeout(Some(TICK))?;
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("ac2-net".into())
            .spawn(move || {
                let outcome = run(&socket, &config, &command_rx, &event_tx);
                let _ = event_tx.send(NetEvent::Disconnected(match outcome {
                    Ok(()) => NetError::Io("disconnected".into()),
                    Err(error) => error,
                }));
            })?;
        Ok(Self {
            commands,
            events,
            thread: Some(thread),
        })
    }

    /// Queues a blob for the server on `queue`.
    pub fn send(&self, queue: u16, data: Vec<u8>) {
        let _ = self.commands.send(Command::Send { queue, data });
    }

    /// Every event received since the last call.
    pub fn poll(&self) -> Vec<NetEvent> {
        let mut out = Vec::new();
        loop {
            match self.events.try_recv() {
                Ok(event) => out.push(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return out,
            }
        }
    }

    /// Waits up to `timeout` for the next event.
    pub fn next_event(&self, timeout: Duration) -> Option<NetEvent> {
        match self.events.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => None,
        }
    }

    /// Sends a logoff and stops the thread.
    pub fn disconnect(&mut self) {
        let _ = self.commands.send(Command::Disconnect);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for NetClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

fn send_all(socket: &UdpSocket, to: SocketAddr, datagrams: &[Vec<u8>]) -> Result<(), NetError> {
    for datagram in datagrams {
        socket.send_to(datagram, to)?;
    }
    Ok(())
}

struct Handshake {
    connect: Connect,
    header: PacketHeader,
    acks_sent: u32,
    last_ack: Instant,
}

fn run(
    socket: &UdpSocket,
    config: &NetClientConfig,
    commands: &Receiver<Command>,
    events: &Sender<NetEvent>,
) -> Result<(), NetError> {
    let server = config.server;
    let ack_address = SocketAddr::new(server.ip(), server.port().wrapping_add(1));
    let logon = config.logon.datagram();
    let started = Instant::now();
    let mut last_logon: Option<Instant> = None;
    let mut handshake: Option<Handshake> = None;
    let mut session: Option<Session> = None;
    let mut buffer = vec![0u8; 65_536];
    loop {
        let now = Instant::now();
        if session.is_none() {
            if now.saturating_duration_since(started) > config.connect_timeout {
                return Err(NetError::Timeout);
            }
            if last_logon.is_none_or(|at| now.saturating_duration_since(at) >= RETRY) {
                socket.send_to(&logon, server)?;
                last_logon = Some(now);
            }
        }
        // Until the first sequenced datagram arrives, the connect-ack may have been lost.
        if let Some(pending) = handshake.as_mut() {
            if now.saturating_duration_since(pending.last_ack) >= RETRY {
                if pending.acks_sent >= CONNECT_ACK_ATTEMPTS {
                    return Err(NetError::Timeout);
                }
                let ack = pending
                    .connect
                    .ack_datagram(pending.header.recipient, pending.header.iteration);
                socket.send_to(&ack, ack_address)?;
                pending.acks_sent += 1;
                pending.last_ack = now;
            }
        }
        match socket.recv_from(&mut buffer) {
            Ok((len, from)) if from.ip() == server.ip() => {
                let bytes = &buffer[..len];
                let Some(header) = PacketHeader::decode(bytes) else {
                    continue;
                };
                if header.flags == flags::CONNECT {
                    if let (None, Ok((header, payload))) = (&session, decode_handshake(bytes)) {
                        if let (None, Ok(connect)) = (&handshake, Connect::decode(payload)) {
                            socket.send_to(
                                &connect.ack_datagram(header.recipient, header.iteration),
                                ack_address,
                            )?;
                            session = Some(Session::new(
                                &connect,
                                header.recipient,
                                header.iteration,
                                now,
                            ));
                            handshake = Some(Handshake {
                                connect,
                                header,
                                acks_sent: 1,
                                last_ack: now,
                            });
                            let _ = events.send(NetEvent::Connected {
                                net_id: connect.net_id,
                                server_time: connect.server_time,
                            });
                        }
                    }
                    continue;
                }
                if let Some(active) = session.as_mut() {
                    match active.receive(bytes, now) {
                        Ok(inbound) => {
                            // Any sequenced reply from the server proves the connect-ack landed.
                            handshake = None;
                            send_all(socket, server, &inbound.replies)?;
                            if let Some(time) = inbound.time_sync {
                                let _ = events.send(NetEvent::TimeSync(time));
                            }
                            for blob in inbound.blobs {
                                let _ = events.send(NetEvent::Blob(blob));
                            }
                            if let Some(error) = inbound.disconnect {
                                return Err(error);
                            }
                            if inbound.logoff {
                                return Ok(());
                            }
                        }
                        // A corrupt or stale datagram is dropped; the sender retransmits.
                        Err(_) => {}
                    }
                }
            }
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            // ICMP port-unreachable surfaces as a reset on some platforms; keep retrying.
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
            Err(error) => return Err(error.into()),
        }
        loop {
            match commands.try_recv() {
                Ok(Command::Send { queue, data }) => {
                    if let Some(active) = session.as_mut() {
                        let datagrams = active.send(queue, &data, Instant::now())?;
                        send_all(socket, server, &datagrams)?;
                    }
                }
                Ok(Command::Disconnect) | Err(TryRecvError::Disconnected) => {
                    if let Some(active) = session.as_mut() {
                        let logoff = active.logoff_datagram(Instant::now());
                        let _ = socket.send_to(&logoff, server);
                    }
                    return Ok(());
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        if let Some(active) = session.as_mut() {
            let datagrams = active.poll(Instant::now())?;
            send_all(socket, server, &datagrams)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use std::time::{Duration, Instant};

    use super::{NetClient, NetClientConfig, NetEvent};
    use crate::handshake::{Connect, Credential, Logon};
    use crate::header::{flags, PacketHeader};
    use crate::isaac::Isaac;
    use crate::packet::{decode_handshake, decode_packet, encode_packet, Fragment};

    /// A loopback stand-in for a server: answers the logon with a connect, takes the
    /// connect-ack on port plus one, sends one blob and checks the client's blob and ack.
    #[test]
    fn handshake_blob_exchange_and_logoff_over_loopback() {
        let (primary, secondary) = (0..50)
            .find_map(|_| {
                let a = UdpSocket::bind("127.0.0.1:0").ok()?;
                let port = a.local_addr().ok()?.port().checked_add(1)?;
                let b = UdpSocket::bind(("127.0.0.1", port)).ok()?;
                Some((a, b))
            })
            .expect("adjacent loopback ports");
        primary
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        secondary
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut client = NetClient::connect(NetClientConfig {
            server: primary.local_addr().unwrap(),
            logon: Logon {
                version: "7148".into(),
                account: "local".into(),
                credential: Credential::Account,
                connection_sequence: 0,
            },
            connect_timeout: Duration::from_secs(5),
        })
        .unwrap();
        let mut buffer = [0u8; 4096];
        let (len, client_addr) = primary.recv_from(&mut buffer).unwrap();
        let (header, payload) = decode_handshake(&buffer[..len]).unwrap();
        assert_eq!(header.flags, flags::LOGON);
        assert_eq!(Logon::decode_payload(payload).unwrap().2, "local");
        let connect = Connect {
            server_time: 3.0,
            cookie: 77,
            net_id: 6,
            outgoing_seed: 0xAB,
            incoming_seed: 0xCD,
        };
        let reply = encode_packet(
            PacketHeader {
                recipient: 6,
                iteration: 2,
                ..Default::default()
            },
            &[(flags::CONNECT, connect.encode().to_vec())],
            &[],
            None,
        );
        primary.send_to(&reply, client_addr).unwrap();
        let (len, _) = secondary.recv_from(&mut buffer).unwrap();
        let (ack_header, cookie) = decode_handshake(&buffer[..len]).unwrap();
        assert_eq!(
            (ack_header.flags, ack_header.recipient),
            (flags::CONNECT_ACK, 6)
        );
        assert_eq!(cookie, &77u64.to_le_bytes());
        // Server to client: one unordered blob at sequence 2.
        let mut server_keys = Isaac::new(connect.outgoing_seed);
        let blob = encode_packet(
            PacketHeader {
                sequence: 2,
                recipient: 6,
                iteration: 2,
                ..Default::default()
            },
            &[],
            &[Fragment {
                blob_id: 0x2300_0001_0000_0001,
                count: 1,
                index: 0,
                queue: 5,
                data: vec![0x3B, 0, 1, 0],
            }],
            Some(server_keys.next_key()),
        );
        primary.send_to(&blob, client_addr).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut got_blob = false;
        while !got_blob && Instant::now() < deadline {
            if let Some(NetEvent::Blob(blob)) = client.next_event(Duration::from_millis(100)) {
                assert_eq!(
                    (blob.queue, blob.data.as_slice()),
                    (5, &[0x3B, 0, 1, 0][..])
                );
                got_blob = true;
            }
        }
        assert!(got_blob);
        let (len, _) = primary.recv_from(&mut buffer).unwrap();
        assert_eq!(decode_packet(&buffer[..len], None).unwrap().ack, Some(2));
        client.send(4, vec![0x2B, 0, 1, 0]);
        let (len, _) = primary.recv_from(&mut buffer).unwrap();
        let mut client_keys = Isaac::new(connect.incoming_seed);
        let packet = decode_packet(&buffer[..len], Some(client_keys.next_key())).unwrap();
        assert_eq!(packet.fragments[0].data, vec![0x2B, 0, 1, 0]);
        client.disconnect();
        let (len, _) = primary.recv_from(&mut buffer).unwrap();
        assert!(decode_packet(&buffer[..len], None).unwrap().logoff);
    }
}
