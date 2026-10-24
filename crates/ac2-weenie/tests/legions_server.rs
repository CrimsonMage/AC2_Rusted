//! A synthetic Legions 7160 server double: an in-process [`Transport`] that answers the client
//! as a 7160 server would (the `CharacterSet` with its two Legions words), on a scripted clock.
//! It drives an [`OnlineSession`] through logon, world entry, a walk and logoff, verifies every
//! client checksum against its own ISAAC stream, and checks the client's datagrams byte for
//! byte. Pyrealium only speaks the admin 7148 layouts, so this is the 7160 path's test.

use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use ac2_net::protocol::{ClientEvent, ClientPosition};
use ac2_net::{
    checksum32, flags, pack_string, pack_wide_string, split_packet, Connect, Credential, Isaac,
    NetError, OptionalHeader, PacketHeader, Transport,
};
use ac2_weenie::{
    BodyState, CellPosition, CharacterChoice, OnlineConfig, OnlineSession, SessionEvent,
};

const SERVER: &str = "127.0.0.1:9000";
const ACK_PORT: &str = "127.0.0.1:9001";
const SLOT: u16 = 3;
const NET_ID: u32 = 0x2A;
const SERVER_SEED: u32 = 0x1357_9BDF;
const CLIENT_SEED: u32 = 0x2468_ACE0;
const REAL: i64 = 1_700_000_000;
const PLAYER: u64 = 0x5000_0000_0000_0001;
const CELL: u32 = 0x0299_0109;

fn addr(text: &str) -> SocketAddr {
    text.parse().unwrap()
}

fn words(out: &mut Vec<u8>, values: &[u32]) {
    out.extend(values.iter().flat_map(|v| v.to_le_bytes()));
}

/// A datagram from its header fields and body segments (`ChecksumHeader` plus the segment
/// sums, XORed with `key` when reliable).
fn datagram(mut header: PacketHeader, segments: &[&[u8]], key: Option<u32>) -> Vec<u8> {
    let mut body = Vec::new();
    let mut sum = 0u32;
    for segment in segments {
        sum = sum.wrapping_add(checksum32(segment));
        body.extend_from_slice(segment);
    }
    if let Some(key) = key {
        sum ^= key;
    }
    header.size = body.len() as u16;
    header.checksum = sum.wrapping_add(header.header_checksum());
    let mut out = header.encode().to_vec();
    out.extend(body);
    out
}

fn fragment_header(id: u64, size: u16, queue: u16) -> Vec<u8> {
    let mut out = id.to_le_bytes().to_vec();
    out.extend(1u16.to_le_bytes());
    out.extend(size.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out.extend(queue.to_le_bytes());
    out
}

/// The 7160 `CharacterSet`: one character, then the two words Legions packs after the
/// turbine-chat flag.
fn character_set() -> Vec<u8> {
    let mut b = Vec::new();
    words(&mut b, &[0x0001_0055, 1]);
    b.extend(PLAYER.to_le_bytes());
    pack_wide_string(&mut b, 0, "Explorer");
    words(&mut b, &[0, 2, 0x1F00_0023, 0, 0, 5]);
    pack_string(&mut b, 0, b"local");
    words(&mut b, &[0, 1, 2]);
    b
}

/// `CharGenVerification`: response, the character's identity, the weenie's result.
fn char_gen_verification() -> Vec<u8> {
    let mut b = Vec::new();
    words(&mut b, &[0x0001_0057, 1]);
    b.extend(PLAYER.to_le_bytes());
    pack_wide_string(&mut b, 0, "Explorer");
    words(&mut b, &[0, 2, 0x1F00_0023, 0x11]);
    b
}

fn create_player() -> Vec<u8> {
    let mut b = 0x0001_0060u32.to_le_bytes().to_vec();
    b.extend(PLAYER.to_le_bytes());
    words(&mut b, &[1]);
    b
}

/// The player's `CreateObject`: a visual parent, a physics position and the stamps
/// (position 4, teleport 5, force position 6, move-to 7), no weenie fields.
fn create_object() -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(PLAYER.to_le_bytes());
    words(&mut b, &[2, 0x1F00_0023, 0x8000]);
    b.extend(9u16.to_le_bytes()); // instance stamp
    b.extend(0u16.to_le_bytes());
    words(&mut b, &[CELL]);
    for v in [59.0f32, 240.0, -44.5, 1.0, 0.0, 0.0, 0.0] {
        b.extend(v.to_le_bytes());
    }
    for stamp in [4u16, 5, 6, 7] {
        b.extend(stamp.to_le_bytes());
    }
    words(&mut b, &[0]);
    b
}

fn player_description() -> Vec<u8> {
    let mut b = Vec::new();
    words(&mut b, &[0x0001_0068, 2, 0x8100_110F, 1, 0x19, 7]);
    b
}

#[derive(Default)]
struct State {
    elapsed: f64,
    /// Datagrams for the client.
    inbox: Vec<(SocketAddr, Vec<u8>)>,
    /// Every client datagram, with its destination.
    sent: Vec<(SocketAddr, Vec<u8>)>,
    /// Client blobs in arrival order: queue and bytes.
    blobs: Vec<(u64, u16, Vec<u8>)>,
    server_keys: Option<Isaac>,
    client_keys: Option<Isaac>,
    sequence: u32,
    connected: bool,
    character_set_sent: bool,
    logged_off: bool,
}

impl State {
    fn interval(&self) -> u16 {
        1 + (self.elapsed / 0.5) as u16
    }

    fn reliable_blob(&mut self, id: u64, queue: u16, data: &[u8]) {
        self.sequence += 1;
        let header = PacketHeader {
            sequence: self.sequence,
            flags: flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            recipient: SLOT,
            interval: self.interval(),
            iteration: 1,
            ..Default::default()
        };
        let fragment = fragment_header(id, data.len() as u16 + 16, queue);
        let key = self.server_keys.as_mut().unwrap().next_key();
        let packet = datagram(header, &[&fragment, data], Some(key));
        self.inbox.push((addr(SERVER), packet));
    }

    /// How a 7160 server answers each client datagram.
    fn answer(&mut self, to: SocketAddr, bytes: &[u8]) {
        let header = PacketHeader::decode(bytes).expect("a header");
        let mut sum = header.checksum.wrapping_sub(header.header_checksum());
        if header.flags & flags::LOGON != 0 {
            // A server-side header: the logon is the whole body.
            let payload = &bytes[20..];
            assert_eq!(sum, checksum32(payload));
            let (version, ..) = ac2_net::Logon::decode_payload(payload).unwrap();
            assert_eq!(version, "7160");
            let connect = OptionalHeader::Connect(Connect {
                server_time: 5000.0,
                cookie: 0x0102_0304_0506_0708,
                net_id: NET_ID,
                outgoing_seed: SERVER_SEED,
                incoming_seed: CLIENT_SEED,
                padding: 0,
            })
            .encode();
            let header = PacketHeader {
                flags: flags::CONNECT,
                recipient: SLOT,
                interval: self.interval(),
                iteration: 1,
                ..Default::default()
            };
            self.inbox
                .push((addr(SERVER), datagram(header, &[&connect], None)));
            self.server_keys = Some(Isaac::new(SERVER_SEED));
            // Reliable sequences start at 2, as `TransmitNewPackets` numbers them.
            self.sequence = 1;
            self.client_keys = Some(Isaac::new(CLIENT_SEED));
            self.connected = true;
            return;
        }
        if header.flags & flags::ENCRYPTED_CHECKSUM != 0 {
            sum ^= self.client_keys.as_mut().expect("connected").next_key();
        }
        let packet = split_packet(bytes, sum, self.connected).expect("a valid client packet");
        if header.flags & flags::CONNECT_ACK != 0 {
            assert_eq!(to, addr(ACK_PORT));
            if !self.character_set_sent {
                self.character_set_sent = true;
                self.reliable_blob(0x2300_0001_0000_0000, 1, &char_gen_verification());
                self.reliable_blob(0x2300_0002_0000_0001, 1, &character_set());
            }
            return;
        }
        if header.flags & flags::LOGOFF != 0 {
            self.logged_off = true;
            return;
        }
        for fragment in packet.fragments {
            let opcode = u32::from_le_bytes(fragment.data[..4].try_into().unwrap());
            self.blobs
                .push((fragment.blob_id, fragment.queue, fragment.data.clone()));
            match opcode {
                // `Proto_UI::SendEnterWorld`.
                0x0001_002B => {
                    self.reliable_blob(0x2300_0003_0000_0002, 1, &create_player());
                    self.reliable_blob(0x0300_0004_0000_0003, 1, &create_object());
                    self.reliable_blob(0x0300_0005_0000_0004, 1, &player_description());
                }
                // `Proto_UI::LogOffCharacter`.
                0x0001_0027 => {
                    self.reliable_blob(0x2300_0006_0000_0005, 1, &0x0001_0027u32.to_le_bytes())
                }
                _ => {}
            }
        }
    }
}

/// The in-process transport: the client's datagrams go to [`State::answer`]; time moves only
/// when the client sleeps.
struct Double(Rc<RefCell<State>>);

impl Transport for Double {
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
        Ok(std::mem::take(&mut self.0.borrow_mut().inbox))
    }

    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
        let mut state = self.0.borrow_mut();
        state.sent.push((to, datagram.to_vec()));
        state.answer(to, datagram);
        Ok(())
    }

    fn clock(&mut self) -> (f64, i64) {
        let elapsed = self.0.borrow().elapsed;
        (elapsed, REAL + elapsed as i64)
    }

    fn sleep(&mut self, duration: Duration) {
        self.0.borrow_mut().elapsed += duration.as_secs_f64();
    }
}

fn config() -> OnlineConfig {
    OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    }
}

#[test]
fn a_7160_session_logs_on_enters_walks_and_logs_off() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut session =
        OnlineSession::with_transport(config(), Box::new(Double(Rc::clone(&state)))).unwrap();
    let entry = session.wait_for_world(Duration::from_secs(10)).unwrap();
    assert_eq!((entry.player, entry.instance), (PLAYER, 9));
    assert_eq!(entry.position.cell, CELL);
    let events = session.poll();
    assert!(events.contains(&SessionEvent::LoginFinished), "{events:?}");
    assert!(events.contains(&SessionEvent::CharGenVerification {
        response: 1,
        character: PLAYER,
        name: "Explorer".into(),
        result: 0x11,
    }));
    session.scene_ready();
    // Walk north for half a second at 64 frames a second, then stand.
    let mut reports = Vec::new();
    for frame in 0..48 {
        let t = frame as f32 / 64.0;
        let walking = frame < 32;
        let body = BodyState {
            position: CellPosition {
                cell: CELL,
                origin: [59.0, 240.0 + 2.0 * t.min(0.5), -44.5],
            },
            heading_degrees: 0.0,
            turn_to_heading: None,
            motion: if walking { [0.0, 1.0, 0.0] } else { [0.0; 3] },
            times_jumped: 0,
            velocity: [0.0; 3],
            actively_moving: walking,
        };
        reports.push((session.cur_time(), body));
        session.report_body(&body);
        session.poll();
        state.borrow_mut().elapsed += 1.0 / 64.0;
    }
    assert!(session.log_off(Duration::from_secs(3)));
    drop(session);
    let state = state.borrow();
    assert!(state.logged_off);

    // The logon: `ClientNet::SendLoginRequest`, alone, sequence 0, the 7160 version.
    let mut payload = vec![4, 0, 0x4B, 0x81, 0xE7, 0xE6, 0, 0]; // "7160", obfuscated
    words(&mut payload, &[28, 1, 0, REAL as u32]);
    payload.extend([5, 0, 0xFE, 0x64, 0xB2, 0xF3, 0xC6, 0]); // "local"
    words(&mut payload, &[0, 0]);
    let logon = datagram(
        PacketHeader {
            flags: flags::LOGON,
            ..Default::default()
        },
        &[&payload],
        None,
    );
    assert_eq!(state.sent[0], (addr(SERVER), logon));
    // The connect-ack to port + 1 echoes the cookie.
    let ack = OptionalHeader::ConnectAck(0x0102_0304_0506_0708).encode();
    let connect_ack = datagram(
        PacketHeader {
            flags: flags::CONNECT_ACK,
            recipient: NET_ID as u16,
            iteration: 1,
            ..Default::default()
        },
        &[&ack],
        None,
    );
    assert_eq!(state.sent[1], (addr(ACK_PORT), connect_ack));

    // The client's blobs, in order: EnterWorld, SceneRenderingComplete, the movement reports,
    // LogOffCharacter.
    let (_, queue, enter) = &state.blobs[0];
    let (_, enter_world) = ClientEvent::EnterWorld {
        character: PLAYER,
        account: "local".into(),
    }
    .encode();
    assert_eq!((*queue, enter), (4, &enter_world));
    // Its datagram, rebuilt: the client's first reliable packet (sequence 2, the first key of
    // its ISAAC stream), the blob alone with its unordered `SendBuff` id.
    let (to, packet) = state
        .sent
        .iter()
        .find(|(_, d)| PacketHeader::decode(d).unwrap().sequence == 2)
        .unwrap();
    let header = PacketHeader::decode(packet).unwrap();
    let rebuilt = datagram(
        PacketHeader {
            sequence: 2,
            flags: flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            recipient: NET_ID as u16,
            interval: header.interval,
            iteration: 1,
            ..Default::default()
        },
        &[
            &fragment_header(0x0300_0001_0000_0000, enter_world.len() as u16 + 16, 4),
            &enter_world,
        ],
        Some(Isaac::new(CLIENT_SEED).next_key()),
    );
    assert_eq!((to, packet), (&addr(SERVER), &rebuilt));
    assert_eq!(state.blobs[1].1, 3);
    assert_eq!(state.blobs[1].2, [0xD1, 0, 1, 0]);
    let positions: Vec<_> = state
        .blobs
        .iter()
        .filter(|(_, _, b)| b[..4] == 0x0001_008Eu32.to_le_bytes())
        .collect();
    // Reports: the first frame; then, while walking, the first frame more than 0.2 s on
    // (`Timer::cur_time` is the time of the frame before, so 13 frames of 1/64 s later: 14, 27);
    // then the stop at once (a motion change).
    let expected: Vec<Vec<u8>> = [0usize, 14, 27, 32]
        .iter()
        .map(|&frame| {
            let (time, body) = reports[frame];
            ClientEvent::Position(ClientPosition {
                time,
                position: body.position,
                heading_degrees: 0.0,
                motion: body.motion,
                cancel_moveto: body.actively_moving,
                instance_stamp: 9,
                teleport_stamp: 5,
                force_position_stamp: 6,
                moveto_stamp: 7,
                jump_velocity: None,
            })
            .encode()
            .1
        })
        .collect();
    let got: Vec<Vec<u8>> = positions.iter().map(|(_, _, b)| b.clone()).collect();
    assert_eq!(got, expected);
    let (_, queue, last) = state.blobs.last().unwrap();
    assert_eq!(
        (*queue, last.clone()),
        ClientEvent::LogOffCharacter { character: PLAYER }.encode()
    );

    // Logoff last: `SendOptionalHeader`, sequence 0, interval 0, our net id.
    let logoff = datagram(
        PacketHeader {
            flags: flags::LOGOFF,
            recipient: NET_ID as u16,
            iteration: 1,
            ..Default::default()
        },
        &[],
        None,
    );
    assert_eq!(state.sent.last().unwrap(), &(addr(SERVER), logoff));
}
