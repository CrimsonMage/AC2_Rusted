//! The client's GUN wiring: `gmClient::StartupGUN` (connect, then the ticket when `-gunticket`),
//! and the per-frame polling that carries the scripts' `RequestGUN_*` calls to
//! [`ac2_net::gun::TurbineGun`] and its results and events back to the client
//! (`TurbineGun::CheckForMessages`, the `CCli*` listeners). The protocol itself is `ac2_net::gun`.

use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Duration, Instant};

use ac2_net::gun::{flag_op, flag_word, GunConnection, GunEvent, GunUserStatus, TurbineGun};
use ac2_script::host::ChatRequest;
use ac2_weenie::{FriendInfo, SessionEvent};

use crate::client::Client;
use crate::friends::{FriendEvent, FriendResult};

/// How long `StartupGUN` waits for the connect and for the ticket's answer. Retail looped on
/// `RealClient::KeepUIAlive` until the event arrived or the window closed.
const STARTUP_WAIT: Duration = Duration::from_secs(5);

/// The GUN arguments of `gmClient` (`gunserver`, `gunport`, `gunticket`, `gunticketoverride`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GunConfig {
    /// `-gunserver`: the host. Empty means no GUN (`gmClient::StartupGUN` returns at once).
    pub server: String,
    /// `-gunport`.
    pub port: u16,
    /// `-gunticket`: authenticate with a ticket after connecting.
    pub use_ticket: bool,
    /// `-gunticketoverride`: the ticket text. Without it retail asked `GunDll.dll` for one;
    /// here the account name stands in (AC2e's developer ticket).
    pub ticket_override: Option<String>,
    /// The logon account, the default ticket.
    pub account: String,
    /// A `-glsticketdirect` ticket was given: 0930's `StartupGUN` then lets a GUN connect
    /// failure pass (it fails the start only without one).
    pub direct_ticket: bool,
}

/// A started GUN service.
pub struct GunService {
    connection: GunConnection,
}

fn resolve(server: &str, port: u16) -> Result<SocketAddr, String> {
    (server, port)
        .to_socket_addrs()
        .map_err(|e| format!("GUN server {server}: {e}"))?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| format!("GUN server {server} has no IPv4 address"))
}

impl GunService {
    /// `gmClient::StartupGUN`. `Ok(None)` when there is no `-gunserver`, or when the connect
    /// failed and a direct ticket makes that acceptable; an error when the start must fail.
    pub fn startup(config: &GunConfig) -> Result<Option<Self>, String> {
        if config.server.is_empty() {
            if config.use_ticket {
                return Err("You must specify a gun server in order to use GUN ticketing".into());
            }
            return Ok(None);
        }
        let address = resolve(&config.server, config.port)?;
        let mut gun = TurbineGun::new();
        if config.use_ticket {
            let ticket = config
                .ticket_override
                .clone()
                .unwrap_or_else(|| config.account.clone());
            gun.set_authentication_ticket(ticket.as_bytes());
        }
        let mut connection = GunConnection::connect(gun, address);
        let events = wait_for(&mut connection, |e| {
            matches!(e, GunEvent::Connected | GunEvent::ConnectFailed)
        });
        if events.first() != Some(&GunEvent::Connected) {
            if config.direct_ticket {
                eprintln!("ac2: GUN server {address} did not connect; going on without GUN");
                return Ok(None);
            }
            return Err(format!(
                "FATAL ERROR: Client cannot connect to the GUN server at {address}"
            ));
        }
        if config.use_ticket {
            // `gmClient::AuthenticateWithGUN`: GetAuthenticationTicket, then present it.
            if !connection.gun.request_present_authentication_ticket() {
                return Err(
                    "FATAL ERROR: Client cannot get present GUN authentication ticket!".into(),
                );
            }
            let events = wait_for(&mut connection, |e| {
                matches!(e, GunEvent::TicketResult { .. })
            });
            match events.first() {
                Some(GunEvent::TicketResult { hr: 0 }) => {}
                Some(GunEvent::TicketResult { hr }) if *hr == ac2_net::gun::ZT_E_ACCOUNT_BANNED => {
                    return Err("FATAL ERROR: Gun server says your account is banned!".into());
                }
                Some(GunEvent::TicketResult { hr }) => {
                    return Err(format!(
                        "FATAL ERROR: Client authentication with the GUN server failed! Error code 0x{hr:08x}"
                    ));
                }
                _ => {
                    return Err(
                        "FATAL ERROR: Client authentication with the GUN server failed!".into(),
                    );
                }
            }
            // The events that arrived with the answer (the friends snapshot) wait for the
            // client in the pump.
            connection.stash_events(events.into_iter().skip(1));
        }
        Ok(Some(Self { connection }))
    }

    /// `RecvNotice_WorldName`: `TurbineGun::SetWorldName`.
    pub fn set_world_name(&mut self, world: &str) {
        self.connection.gun.set_world_name(world);
    }

    /// One frame of `TurbineGun::CheckForMessages`: the scripts' requests go out, the
    /// answers and events come in. Returns the session events the scripts produced.
    pub fn pump(&mut self, client: &mut Client) -> Vec<SessionEvent> {
        for request in client.take_gun_requests() {
            issue(&mut self.connection.gun, request);
        }
        let mut out = Vec::new();
        for event in self.connection.pump() {
            out.extend(deliver(client, event));
        }
        out
    }
}

/// Pumps until `done` matches an event; the events up to and including it are returned. Gives
/// up after [`STARTUP_WAIT`], returning what arrived.
fn wait_for(connection: &mut GunConnection, done: impl Fn(&GunEvent) -> bool) -> Vec<GunEvent> {
    let started = Instant::now();
    let mut seen = Vec::new();
    while started.elapsed() < STARTUP_WAIT {
        let events = connection.pump();
        seen.extend(events);
        if let Some(at) = seen.iter().position(&done) {
            return seen.split_off(at);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    seen
}

/// `gmCWInterface::RequestGUN_*` to `TurbineGun::Request*`. Returns whether the request went out.
pub fn issue(gun: &mut TurbineGun, request: ChatRequest) -> bool {
    match request {
        ChatRequest::SendToUser { user, text, flags } => {
            gun.request_send_to_user(&user, &text, flags)
        }
        // Room chat goes to Turbine chat when the client has it (`CWBookKeeper`); GUN rooms
        // are not served.
        ChatRequest::SendToRoom { .. } | ChatRequest::CallCreate { .. } => false,
        ChatRequest::SetStateFlags(flags) => {
            gun.request_flags(flag_word::STATE, flag_op::SET, flags)
        }
        ChatRequest::SetLocation { flags, location } => gun.request_set_location(flags, &location),
        ChatRequest::FriendAdd(name) => gun.request_friend_add(&name),
        ChatRequest::FriendDelete(name) => gun.request_friend_delete(&name),
        ChatRequest::GetFriendsList { user_requested } => {
            gun.request_get_friends_list(user_requested)
        }
        ChatRequest::IgnoredAdd(name) => gun.request_ignored_add(&name),
        ChatRequest::IgnoredDelete(name) => gun.request_ignored_delete(&name),
        ChatRequest::GetIgnoredList { user_requested } => {
            gun.request_get_ignored_list(user_requested)
        }
    }
}

fn friend(status: GunUserStatus) -> FriendInfo {
    FriendInfo {
        name: status.name,
        location: status.location,
        mode: status.mode,
    }
}

/// A GUN event to the client: friend results and events reach the scripts and the friends page.
/// Results with no consumer yet (the ignore list, flags, location, tells) are not delivered.
fn deliver(client: &mut Client, event: GunEvent) -> Vec<SessionEvent> {
    match event {
        GunEvent::FriendStart(list) => {
            client.gun_friend_event(FriendEvent::Start(list.into_iter().map(friend).collect()))
        }
        GunEvent::FriendStatus(status) => {
            client.gun_friend_event(FriendEvent::Status(friend(status)))
        }
        GunEvent::FriendsList { hr: 0, friends, .. } => {
            // `RecvNotice_GUNResult_GetFriendsList`: the list reaches the page.
            client.friend_events.push(FriendEvent::Start(
                friends.into_iter().map(friend).collect(),
            ));
            Vec::new()
        }
        GunEvent::FriendAdd { hr, name } => {
            client.gun_friend_result(FriendResult::Added { gun: hr, name })
        }
        GunEvent::FriendDelete { hr, name } => {
            client.gun_friend_result(FriendResult::Removed { gun: hr, name })
        }
        GunEvent::Disconnected => {
            eprintln!("ac2: GUN connection lost");
            Vec::new()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn requests_map_to_gun_calls() {
        let mut gun = TurbineGun::new();
        assert!(
            !issue(&mut gun, ChatRequest::FriendAdd("Bob".into())),
            "not connected"
        );
        let sent = |gun: &mut TurbineGun| String::from_utf8_lossy(&gun.take_output()).into_owned();
        gun.request_connect();
        gun.take_output();
        let mut frame = br#"<ConnectRes hr="0x00000000" Random=""/>"#.to_vec();
        let mut wire = (frame.len() as u32).to_be_bytes().to_vec();
        wire.append(&mut frame);
        gun.receive(&wire).unwrap();
        assert!(issue(&mut gun, ChatRequest::SetStateFlags(4)));
        assert!(sent(&mut gun).contains(r#"<Flags Op="1" Which="1" Flags="4" seq="1"/>"#));
        assert!(issue(
            &mut gun,
            ChatRequest::GetFriendsList {
                user_requested: true
            }
        ));
        assert!(sent(&mut gun).contains(r#"<GetFriendsList seq="2"/>"#));
        assert!(!issue(
            &mut gun,
            ChatRequest::SendToRoom {
                room: 1,
                text: "x".into(),
                flags: 0
            }
        ));
    }

    /// A one-shot GUN server: accepts the connect and a ticket, then answers like AC2e.
    fn fake_server(answer_hr: &'static str) -> (u16, std::thread::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut got = Vec::new();
            let mut reader = ac2_net::gun::FrameReader::default();
            let mut buffer = [0u8; 1024];
            let send = |stream: &mut std::net::TcpStream, text: &str| {
                let mut out = (text.len() as u32).to_be_bytes().to_vec();
                out.extend_from_slice(text.as_bytes());
                stream.write_all(&out).unwrap();
            };
            loop {
                let n = match stream.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                reader.push(&buffer[..n]);
                while let Some(elem) = reader.next_element().unwrap() {
                    got.push(elem.encode());
                    match elem.name.as_str() {
                        "Connect" => {
                            send(&mut stream, r#"<ConnectRes hr="0x00000000" Random=""/>"#)
                        }
                        "PresentTicketCmd" => {
                            let seq = elem.get("seq").unwrap();
                            send(
                                &mut stream,
                                &format!(
                                    r#"<PresentTicketRes seq="{seq}" hr="{answer_hr}" Status="0"/>"#
                                ),
                            );
                            send(
                                &mut stream,
                                r#"<FStart><Friend Name="Bob@AC2e" Mode="1" Location=""/></FStart>"#,
                            );
                            return got;
                        }
                        _ => {}
                    }
                }
            }
            got
        });
        (port, worker)
    }

    fn config(port: u16) -> GunConfig {
        GunConfig {
            server: "127.0.0.1".into(),
            port,
            use_ticket: true,
            ticket_override: None,
            account: "alice".into(),
            direct_ticket: false,
        }
    }

    #[test]
    fn startup_connects_and_presents_the_account_as_the_ticket() {
        let (port, server) = fake_server("0x00000000");
        let mut service = GunService::startup(&config(port)).unwrap().unwrap();
        let got = server.join().unwrap();
        assert_eq!(got[0], r#"<Connect ProtoVer="1"/>"#);
        assert_eq!(got[1], r#"<PresentTicketCmd Ticket="616c696365" seq="1"/>"#);
        // The friends snapshot that followed the answer is waiting for the client.
        let events = service.connection.pump();
        // (The one-shot server then hangs up, which follows as a disconnect.)
        assert!(matches!(events.first(), Some(GunEvent::FriendStart(list)) if list.len() == 1));
    }

    #[test]
    fn a_refused_ticket_fails_the_start() {
        let (port, server) = fake_server("0xC1120806");
        let error = GunService::startup(&config(port)).err().unwrap();
        server.join().unwrap();
        assert!(error.contains("0xc1120806"), "{error}");
    }

    #[test]
    fn no_server_means_no_gun_and_a_dead_one_is_tolerated_with_a_direct_ticket() {
        assert!(GunService::startup(&GunConfig::default())
            .unwrap()
            .is_none());
        let mut ticketed = GunConfig {
            use_ticket: true,
            ..GunConfig::default()
        };
        assert!(
            GunService::startup(&ticketed).is_err(),
            "-gunticket needs -gunserver"
        );
        // A port nobody listens on.
        let free = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        ticketed.server = "127.0.0.1".into();
        ticketed.port = free;
        assert!(GunService::startup(&ticketed).is_err());
        ticketed.direct_ticket = true;
        assert!(GunService::startup(&ticketed).unwrap().is_none());
    }
}
