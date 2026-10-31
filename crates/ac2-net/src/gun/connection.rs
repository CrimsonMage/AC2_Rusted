//! A TCP socket under [`TurbineGun`], polled once per frame as the client's loop polled GUN
//! (`TurbineGun::CheckForMessages`).

use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use super::turbine_gun::{GunEvent, TurbineGun};

/// How long the connect may take before it counts as `ConnectFailed`.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// A [`TurbineGun`] and its socket.
#[derive(Debug)]
pub struct GunConnection {
    pub gun: TurbineGun,
    stream: Option<TcpStream>,
    unsent: Vec<u8>,
    /// Events taken early (while waiting for another) that the next pump returns first.
    stashed: Vec<GunEvent>,
}

impl GunConnection {
    /// `gmClient::StartupGUN`'s `RequestConnect`: opens the socket (a failure queues
    /// [`GunEvent::ConnectFailed`]) and sends the first frame.
    pub fn connect(mut gun: TurbineGun, server: SocketAddr) -> Self {
        let mut connection = Self {
            stream: None,
            unsent: Vec::new(),
            stashed: Vec::new(),
            gun: TurbineGun::new(),
        };
        let stream = TcpStream::connect_timeout(&server, CONNECT_TIMEOUT).and_then(|stream| {
            stream.set_nodelay(true)?;
            stream.set_nonblocking(true)?;
            Ok(stream)
        });
        gun.request_connect();
        match stream {
            Ok(stream) => connection.stream = Some(stream),
            Err(_) => {
                gun.take_output();
                gun.connection_failed();
            }
        }
        connection.gun = gun;
        connection
    }

    /// Sends what is queued and reads what has arrived, without blocking. Returns the events
    /// that came out of the frame.
    pub fn pump(&mut self) -> Vec<GunEvent> {
        self.unsent.extend(self.gun.take_output());
        if let Some(stream) = &mut self.stream {
            let mut lost = false;
            while !self.unsent.is_empty() {
                match stream.write(&self.unsent) {
                    Ok(0) => {
                        lost = true;
                        break;
                    }
                    Ok(n) => {
                        self.unsent.drain(..n);
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(_) => {
                        lost = true;
                        break;
                    }
                }
            }
            let mut buffer = [0u8; 4096];
            while !lost {
                match stream.read(&mut buffer) {
                    Ok(0) => lost = true,
                    Ok(n) => {
                        if self.gun.receive(&buffer[..n]).is_err() {
                            lost = true;
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(_) => lost = true,
                }
            }
            if lost {
                self.stream = None;
                self.gun.connection_lost();
            }
        }
        let mut events = std::mem::take(&mut self.stashed);
        events.extend(self.gun.take_events());
        events
    }

    /// Puts events back at the front of the next [`GunConnection::pump`].
    pub fn stash_events(&mut self, events: impl IntoIterator<Item = GunEvent>) {
        self.stashed.extend(events);
    }

    /// Whether the socket is still open.
    pub fn is_open(&self) -> bool {
        self.stream.is_some()
    }

    /// Flushes what is queued, ending the session (`~TurbineGun`).
    pub fn close(&mut self) {
        self.pump();
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gun::{frame, Elem, FrameReader};
    use std::net::TcpListener;

    fn events(connection: &mut GunConnection, want: usize) -> Vec<GunEvent> {
        let mut seen = Vec::new();
        for _ in 0..600 {
            seen.extend(connection.pump());
            if seen.len() >= want {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        seen
    }

    #[test]
    fn a_connection_talks_to_a_socket_and_notices_it_close() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = FrameReader::default();
            let mut buffer = [0u8; 256];
            let mut seen = Vec::new();
            while seen.len() < 2 {
                let n = stream.read(&mut buffer).unwrap();
                reader.push(&buffer[..n]);
                while let Some(elem) = reader.next_element().unwrap() {
                    let reply = match elem.get("seq") {
                        Some(seq) => Elem::new("SetLocRes")
                            .attr("seq", seq)
                            .attr("hr", "0x00000000"),
                        None => Elem::new("ConnectRes").attr("hr", "0x00000000"),
                    };
                    stream.write_all(&frame(&reply)).unwrap();
                    seen.push(elem.encode());
                }
            }
            seen
        });
        let mut connection = GunConnection::connect(TurbineGun::new(), address);
        assert_eq!(events(&mut connection, 1), vec![GunEvent::Connected]);
        assert!(connection.gun.request_set_location(2, "Holtburg"));
        // The server ends after its second answer: the connection reports it once.
        assert_eq!(
            events(&mut connection, 2),
            vec![GunEvent::Location { hr: 0 }, GunEvent::Disconnected]
        );
        assert_eq!(
            server.join().unwrap(),
            vec![
                r#"<Connect ProtoVer="1"/>"#.to_owned(),
                r#"<SetLoc Flags="2" Location="Holtburg" seq="1"/>"#.to_owned()
            ]
        );
        assert!(!connection.is_open());
    }

    #[test]
    fn nothing_listening_is_a_connect_failure() {
        let address = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let mut connection = GunConnection::connect(TurbineGun::new(), address);
        assert_eq!(connection.pump(), vec![GunEvent::ConnectFailed]);
        assert!(!connection.is_open());
    }
}
