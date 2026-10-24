//! A synthetic Legions 7160 server double (test tooling): an in-process [`Transport`] that
//! answers the client as a 7160 server would (`CharGenVerification`, the `CharacterSet` with its
//! two Legions words, world entry, logoff) on a scripted clock, and verifies every client
//! checksum against its own ISAAC stream. Pyrealium only speaks the admin 7148 layouts, so this
//! is the 7160 path's test peer.

use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use ac2_net::{
    checksum32, flags, pack_string, pack_wide_string, split_packet, Connect, Isaac, NetError,
    OptionalHeader, PacketHeader, Transport,
};

pub const SERVER: &str = "127.0.0.1:9000";
pub const ACK_PORT: &str = "127.0.0.1:9001";
pub const SLOT: u16 = 3;
pub const NET_ID: u32 = 0x2A;
pub const SERVER_SEED: u32 = 0x1357_9BDF;
pub const CLIENT_SEED: u32 = 0x2468_ACE0;
pub const REAL: i64 = 1_700_000_000;
pub const PLAYER: u64 = 0x5000_0000_0000_0001;
pub const CELL: u32 = 0x0299_0109;

pub fn addr(text: &str) -> SocketAddr {
    text.parse().unwrap()
}

pub fn words(out: &mut Vec<u8>, values: &[u32]) {
    out.extend(values.iter().flat_map(|v| v.to_le_bytes()));
}

/// A datagram from its header fields and body segments (`ChecksumHeader` plus the segment
/// sums, XORed with `key` when reliable).
pub fn datagram(mut header: PacketHeader, segments: &[&[u8]], key: Option<u32>) -> Vec<u8> {
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

pub fn fragment_header(id: u64, size: u16, queue: u16) -> Vec<u8> {
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

/// Where the player enters the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spawn {
    pub cell: u32,
    pub origin: [f32; 3],
    /// (w, x, y, z).
    pub rotation: [f32; 4],
}

impl Default for Spawn {
    fn default() -> Self {
        Self {
            cell: CELL,
            origin: [59.0, 240.0, -44.5],
            rotation: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

/// The player's `CreateObject`: a visual parent, a physics position and the stamps
/// (position 4, teleport 5, force position 6, move-to 7), no weenie fields.
fn create_object(spawn: &Spawn) -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(PLAYER.to_le_bytes());
    words(&mut b, &[2, 0x1F00_0023, 0x8000]);
    b.extend(9u16.to_le_bytes()); // instance stamp
    b.extend(0u16.to_le_bytes());
    words(&mut b, &[spawn.cell]);
    for v in spawn.origin.iter().chain(spawn.rotation.iter()) {
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

/// The server's view of the session: what it received and what it will send.
#[derive(Default)]
pub struct State {
    pub spawn: Spawn,
    pub elapsed: f64,
    /// Datagrams for the client.
    pub inbox: Vec<(SocketAddr, Vec<u8>)>,
    /// Every client datagram, with its destination.
    pub sent: Vec<(SocketAddr, Vec<u8>)>,
    /// Client blobs in arrival order: queue and bytes.
    pub blobs: Vec<(u64, u16, Vec<u8>)>,
    server_keys: Option<Isaac>,
    client_keys: Option<Isaac>,
    sequence: u32,
    connected: bool,
    character_set_sent: bool,
    pub logged_off: bool,
    next_object_blob: u16,
}

impl State {
    /// Queues a server position event for the player (`ECM_Physics::RecvEvt_Position`):
    /// `PositionPack` with the given stamps (position, teleport, force position).
    pub fn send_player_position(&mut self, cell: u32, origin: [f32; 3], stamps: [u16; 3]) {
        let mut b = 0x0001_0086u32.to_le_bytes().to_vec();
        b.extend(PLAYER.to_le_bytes());
        b.extend([9, 0, 0, 0]); // instance 9, visual stamp 0
        b.extend(0.0f64.to_le_bytes());
        words(&mut b, &[cell]);
        for v in origin {
            b.extend(v.to_le_bytes());
        }
        // Heading 0, motion (0, 0, 0); no flags.
        words(&mut b, &[0x007F_7F7F, 0]);
        for stamp in [stamps[0], stamps[2], stamps[1]] {
            b.extend(stamp.to_le_bytes());
        }
        b.extend([0, 0]);
        self.next_object_blob += 1;
        let id = 0x0300_0000_0000_0000 | u64::from(self.next_object_blob) << 32 | 0x100;
        self.reliable_blob(id, 1, &b);
    }

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
                    let object = create_object(&self.spawn);
                    self.reliable_blob(0x0300_0004_0000_0003, 1, &object);
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
pub struct Double(pub Rc<RefCell<State>>);

/// A double and a handle on its state.
pub fn double(spawn: Spawn) -> (Double, Rc<RefCell<State>>) {
    let state = Rc::new(RefCell::new(State {
        spawn,
        ..State::default()
    }));
    (Double(Rc::clone(&state)), state)
}

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
