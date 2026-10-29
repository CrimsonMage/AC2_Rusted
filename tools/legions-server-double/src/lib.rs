//! A synthetic Legions 7160 server double (test tooling): an in-process [`Transport`] that
//! answers the client as a 7160 server would (`CharGenVerification`, the `CharacterSet` with its
//! two Legions words, world entry, logoff) on a scripted clock, and verifies every client
//! checksum against its own ISAAC stream. Pyrealium only speaks the admin 7148 layouts, so this
//! is the 7160 path's test peer.

mod allegiance;
mod appearance;
mod assembly;
mod channel;
mod character;
pub use characters::{CharGenRules, CharacterStore, StoredCharacter};
mod butcher;
mod characters;
mod chest;
mod combat;
mod corpse;
pub use combat::{
    lifestone, Fight, BEHAVIOR_MELEE_AUTO, BEHAVIOR_MELEE_NORMAL, BOW_AUTO_ATTACK, CREATURE_HEALTH,
    DEATH_SECONDS, HEALTH, HEALTH_MAX, MODE_COMBAT, MODE_COMBAT_MONSTER, PLAYER_HEALTH,
    PLAYER_HEALTH_MAX, THROW_AUTO_ATTACK,
};
mod craft;
pub use craft::{CRAFT_SKILL, KNOWN_RECIPE};
mod effects;
mod friends;
pub use friends::{
    FriendRecord, FriendsBook, GUN_ALREADY_IN_LIST, GUN_DONE, GUN_INVALID_USER, GUN_NOT_IN_LIST,
};
mod description;
mod items;
mod lossy;
mod objects;
mod radar;
mod reliable;
mod script;
mod social;
mod store;
mod trade;
mod vitae;
mod vitals;
mod world_objects;

pub use allegiance::{Allegiance, Liege};
pub use appearance::{
    contain_bytes, create_object_with_visual, part_group_bytes, quality_bytes,
    update_visual_desc_bytes, visual_desc_bytes,
};
pub use assembly::{fragment_header_bytes, Assembler};
pub use butcher::{BUTCHER_KNIFE, BUTCHER_SKIN};
pub use channel::{Channel, ChannelStats, Conditions, Direction, InFlight, SplitMix};
pub use character::{
    default_options, Character, SkillCost, DEFAULT_SKILL_COST, MELEE_ATTACK, MELEE_AUTO_ATTACK,
    RACE_HUMAN,
};
pub use chest::{CHEST, CHEST_CAPACITY, CHEST_GEM, CHEST_SCROLL};
pub use corpse::{
    CORPSE, CORPSE_ITEM, CORPSE_SKIN, CORPSE_WEENIE_TYPE, FOREIGN_CORPSE, FOREIGN_ITEM,
    TROPHY_MIMBU_SKIN_PACKAGE,
};
pub use effects::{
    effect_registry, record_graph, EffectGrant, Effects, SkillEffect, ADD_EFFECT, EFFECT_PULSED,
    EFFECT_TIMED, REMOVE_EFFECT,
};
pub use items::{
    item_object_bytes, segments, starter_items, ItemSpawn, MAIN_PACK_CAPACITY, PACK, POTIONS,
    PRIMARY_HAND, RATIONS, SECONDARY_HAND, STARTER_MONEY, SWORD,
};
pub use lossy::LossyTransport;
pub use objects::{
    create_object_bytes, object_event_bytes, position_bytes, weenie_object_bytes,
    weenie_object_with, ObjectPosition, ObjectSpawn,
};
pub use radar::{
    radar_objects, CREATURE_ONE, CREATURE_TWO, DRUDGE_VISUAL, MONSTER_PACKAGE, NPC, NPC_PACKAGE,
};
pub use reliable::{newer, ClientTracker, SentStore, Stored, Verdict, MAX_IDS};
pub use script::ScriptWorld;
pub use social::{fellow_graph, Fellow, Fellowship, FELLOW, FELLOWSHIP_ROOM};
pub use trade::{Trade, TRADE_CANCELLED, TRADE_COMPLETED};
pub use vitae::{
    Vitae, CURRENT_VITAE, VITAE_FLOOR, VITAE_PENALTY, XP_PER_VITAE_POINT, XP_TO_RAISE_VITAE,
};
pub use vitals::{
    Vitals, COMBAT_NOT_ENOUGH_VIGOR, PLAYER_VIGOR_MAX, REGEN_INTERVAL, VIGOR, VIGOR_MAX,
};
pub use world_objects::{
    BLOCKING_CREATURE, BLOCKING_CREATURE_VISUAL, DOOR, DOOR_OPEN, DOOR_SHUT, DOOR_VISUAL,
    GROUND_ITEM, LEVER, LEVER_FLIP, LEVER_VISUAL, SWORD_VISUAL,
};

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
/// The player's instance id: type byte 0x20, a player (`IIDUtils::GetType`); the double's
/// other characters (`CREATED`, Birch, the vassals) are players too, its creatures and items
/// dynamic (0x80).
pub const PLAYER: u64 = 0x2000_0000_0000_0001;
/// The id the double gives a character the client creates.
pub const CREATED: u64 = 0x2000_0000_0000_0002;
pub const CELL: u32 = 0x0299_0109;
/// The server time the connect answer carries (`Connect::server_time`).
pub const SERVER_TIME: f64 = 5000.0;

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

/// `CharacterSet` (`ECM_Login::RecvEvt_CharacterSet`) listing `characters` with their visuals,
/// each with no greyed time; no deleted characters, status 0, five allowed. Legions packs two
/// more words after the chat flag.
pub(crate) fn character_set_of(characters: &[(u64, &str, &[u8])]) -> Vec<u8> {
    let mut b = Vec::new();
    words(&mut b, &[0x0001_0055, characters.len() as u32]);
    for (id, name, visual) in characters {
        b.extend(id.to_le_bytes());
        pack_wide_string(&mut b, 0, name);
        words(&mut b, &[0]);
        b.extend(*visual);
    }
    words(&mut b, &[0, 0, 5]);
    pack_string(&mut b, 0, b"local");
    words(&mut b, &[0, 1, 2]);
    b
}

/// `CreatePlayer` (`ECM_Physics::RecvEvt_CreatePlayer`): the player's object id and region.
fn create_player(player: u64) -> Vec<u8> {
    let mut b = 0x0001_0060u32.to_le_bytes().to_vec();
    b.extend(player.to_le_bytes());
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

/// `CPlayer`, the player weenie's script package.
pub const PLAYER_PACKAGE: u32 = 0x1AA4;

/// The player's `CreateObject`: a visual parent, a physics position and the stamps
/// (position 4, teleport 5, force position 6, move-to 7), its script package (`CPlayer`) and
/// its name.
pub(crate) fn create_object(spawn: &Spawn, visual: &[u8]) -> Vec<u8> {
    weenie_object_with(
        &ObjectSpawn {
            id: PLAYER,
            instance: 9,
            visual: 0x1F00_0023,
            cell: spawn.cell,
            origin: spawn.origin,
            rotation: spawn.rotation,
            stamps: [4, 5, 6, 7],
            name: "Explorer",
        },
        visual,
        PLAYER_PACKAGE,
        "Explorer",
    )
}

/// A client packet the double accepted (checksum verified, not a duplicate).
#[derive(Debug, Clone)]
pub struct Accepted {
    pub time: f64,
    pub to: SocketAddr,
    pub header: PacketHeader,
    pub optional: Vec<OptionalHeader>,
    pub fragments: Vec<ac2_net::Fragment>,
    pub bytes: Vec<u8>,
    /// The ISAAC key its checksum was encrypted under (reliable packets).
    pub key: Option<u32>,
}

/// One server message of a teleport, in the order the double sends it.
#[derive(Debug, Clone, PartialEq)]
pub enum TeleportMessage {
    /// `PositionPack` for the player with a newer teleport stamp (`ECM_Physics::RecvEvt_Position`).
    PositionPack {
        cell: u32,
        origin: [f32; 3],
        teleport_stamp: u16,
    },
}

/// The server's view of the session: what it received and what it will send.
#[derive(Default)]
pub struct State {
    pub spawn: Spawn,
    /// Whether the door stands open (a use flips it).
    pub door_open: bool,
    /// The teleports the double sent, in order (see [`State::teleport_player`]).
    pub teleport_trace: Vec<TeleportMessage>,
    pub elapsed: f64,
    /// Datagrams for the client.
    pub inbox: Vec<(SocketAddr, Vec<u8>)>,
    /// Every client datagram, with its destination.
    pub sent: Vec<(SocketAddr, Vec<u8>)>,
    /// Client blobs in completion order: id, queue and bytes.
    pub blobs: Vec<(u64, u16, Vec<u8>)>,
    /// The client's fragments, assembling.
    pub assembler: Assembler,
    server_keys: Option<Isaac>,
    /// The client's sequences and keys (`ReceiverData`), from the connect.
    pub tracker: Option<ClientTracker>,
    /// The double's reliable packets, for resends.
    pub store: SentStore,
    /// When each entry of `sent` left the client (the double's clock).
    pub sent_at: Vec<f64>,
    /// Every client packet the double accepted, in acceptance order.
    pub accepted: Vec<Accepted>,
    /// Every datagram the double sent (before the channel), with its time.
    pub server_sent: Vec<(f64, Vec<u8>)>,
    /// The lossy network, when a test installs one.
    pub channel: Option<Channel>,
    /// Sequences the double resent, in order.
    pub resent: Vec<u32>,
    /// `NetErrorDisconnect` headers the client sent.
    pub disconnects_received: u32,
    /// Stop answering the client entirely (a dead server).
    pub silent: bool,
    /// Double sequences whose first transmission is lost, whatever the channel says.
    pub drop_server_once: std::collections::BTreeSet<u32>,
    /// Client sequences whose first arrival is lost.
    pub drop_client_once: std::collections::BTreeSet<u32>,
    /// The connect answer, sent again when the logon repeats (its answer was lost).
    connect: Option<Vec<u8>>,
    /// The ordering stamp of the last ordered event.
    ordered_stamp: u16,
    sequence: u32,
    connected: bool,
    character_set_sent: bool,
    pub logged_off: bool,
    next_object_blob: u16,
    /// What the double answers script calls with.
    pub script_world: ScriptWorld,
    /// The fight against the double's creatures (`combat.rs`).
    pub fight: Fight,
    /// The `VisualDesc` the radar creatures are created with (0: none, as the logic tests use;
    /// a drawn test sets a monster's, e.g. [`DRUDGE_VISUAL`]).
    pub creature_visual: u32,
    /// The `VisualDesc` the ground item is created with (0: none; a drawn test sets one, e.g.
    /// [`SWORD_VISUAL`]).
    pub ground_item_visual: u32,
    /// Every unordered queue-1 event the double sent, byte for byte, in order (`send_event`).
    pub events_sent: Vec<Vec<u8>>,
    /// The player's vigor, its costs and regeneration (`vitals.rs`).
    pub vitals: Vitals,
    /// The effects the double applies to the player (`effects.rs`).
    pub effects: Effects,
    /// The double's clock when the client connected (the server time 5000).
    connected_at: f64,
    /// The fellowship the double made for the player (`WM_Fellowship` 0x22), if any.
    pub fellowship: Option<Fellowship>,
    /// The player's allegiance.
    pub allegiance: Allegiance,
    /// The open secure trade, if any (`WM_Trade`).
    pub trade: Trade,
    /// Who last asked to swear to the player (`ask_to_swear`).
    pub asking_vassal: Option<Liege>,
    /// Who last invited the player into a fellowship: id, name, the fellowship's name.
    pub recruiter: Option<(u64, String, String)>,
    /// The client's script calls, decoded, in arrival order.
    pub script_requests: Vec<ac2_net::protocol::ScriptRequest>,
    /// Every script request as it arrived: its function id and packed arguments, byte for byte.
    pub script_bytes: Vec<(u32, Vec<u8>)>,
    /// The skill answers the double sent (`WM_Skill` 0xD, 0xE), in order.
    pub skill_answers: Vec<ac2_net::protocol::SkillMessage>,
    /// The client's calls in `ac2_net::protocol::SCRIPT_CALLS` (skills, shortcuts,
    /// fellowships, allegiances), decoded, in arrival order.
    pub script_calls: Vec<ac2_net::protocol::ScriptCall>,
    /// Script calls with a function id the double does not know: id and packed arguments.
    pub script_unknown: Vec<(u32, Vec<u8>)>,
    /// Script calls that did not decode.
    pub script_errors: Vec<String>,
    /// The client's chat-client requests, decoded.
    pub chat_requests: Vec<ac2_net::protocol::ChatRequest>,
    /// `Proto_UI::SendCharGenResult`s: account and result.
    pub char_gen_results: Vec<(String, ac2_net::protocol::CharGenResult)>,
    /// The created characters (`double_with`'s store).
    pub characters: CharacterStore,
    /// The player's object id once a character entered (`None`: `PLAYER`, the `Explorer`).
    pub player_id: Option<u64>,
    /// The server's character rules (`double_with`).
    pub rules: Option<Box<dyn CharGenRules>>,
    /// `Explorer`'s `VisualDesc` bytes when the rules make an appearance for it (empty: the
    /// bare parent).
    pub(crate) explorer_visual: Vec<u8>,
    /// `Proto_UI::SendDeleteCharacter`s: the character ids.
    pub deletions: Vec<u64>,
}

impl State {
    /// Queues a server position event for the player (`ECM_Physics::RecvEvt_Position`):
    /// `PositionPack` with the given stamps (position, teleport, force position).
    pub fn send_player_position(&mut self, cell: u32, origin: [f32; 3], stamps: [u16; 3]) {
        let mut b = 0x0001_0086u32.to_le_bytes().to_vec();
        b.extend(self.player().to_le_bytes());
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

    /// A teleport of the player to `cell` at `origin` (a portal: `CPhysicsObj::RecvEvt_Position`
    /// with a newer teleport stamp). Legions has no teleport message of its own, so the teleport
    /// is this one `PositionPack` (the force stamp is kept at the spawn's 6). The trace records the
    /// server's messages in the order they are sent.
    pub fn teleport_player(
        &mut self,
        cell: u32,
        origin: [f32; 3],
        position_stamp: u16,
        teleport_stamp: u16,
    ) {
        self.send_player_position(cell, origin, [position_stamp, teleport_stamp, 6]);
        self.teleport_trace.push(TeleportMessage::PositionPack {
            cell,
            origin,
            teleport_stamp,
        });
    }

    /// Refuses a character request (`ECM_Login::RecvEvt_CharacterError`: the `charError`).
    pub fn send_character_error(&mut self, error: u32) {
        let mut data = Vec::new();
        words(&mut data, &[0x0001_0056, error]);
        // A login event (type 0x23), like the character set's.
        self.next_object_blob += 1;
        let id = 0x2300_0000_0000_0000 | u64::from(self.next_object_blob) << 32;
        self.reliable_blob(id, 1, &data);
    }

    /// Queues an unordered queue-1 event blob (`CEventQueue::DispatchEvent` order); returns
    /// its id.
    pub fn send_event(&mut self, data: &[u8]) -> u64 {
        self.events_sent.push(data.to_vec());
        self.next_object_blob += 1;
        let id = 0x0300_0000_0000_0000 | u64::from(self.next_object_blob) << 32 | 0x100;
        self.reliable_blob(id, 1, data);
        id
    }

    /// Queues a player-ordered queue-1 event blob (`CEventQueue::ProcessOrderedNetBlob`): the
    /// ordering stamps run 1, 2, 3...
    pub fn send_ordered(&mut self, data: &[u8]) {
        self.next_object_blob += 1;
        self.ordered_stamp = self.ordered_stamp.wrapping_add(1);
        let high = u64::from(ac2_net::ORDERING_ORDERED | u32::from(self.ordered_stamp));
        let id = high << 32 | u64::from(self.next_object_blob) << 8;
        self.reliable_blob(id, 1, data);
    }

    /// Queues an unordered queue-1 event of any size, fragmented as `NetBlob::Fragmentize`
    /// does, one reliable packet per fragment. Returns the blob id.
    pub fn send_fragmented(&mut self, data: &[u8]) -> u64 {
        self.next_object_blob += 1;
        let id = 0x0300_0000_0000_0000 | u64::from(self.next_object_blob) << 32 | 0x100;
        for fragment in ac2_net::fragmentize(id, 1, data) {
            let segments = vec![fragment_header_bytes(&fragment), fragment.data];
            self.reliable(segments, flags::BLOB_FRAGMENTS);
        }
        id
    }

    /// A queue-1 blob under `id`, fragmented as `NetBlob::Fragmentize` does when it is larger
    /// than a packet (a character set with several characters is).
    pub(crate) fn reliable_blob_fragmented(&mut self, id: u64, queue: u16, data: &[u8]) {
        for fragment in ac2_net::fragmentize(id, queue, data) {
            let segments = vec![fragment_header_bytes(&fragment), fragment.data];
            self.reliable(segments, flags::BLOB_FRAGMENTS);
        }
    }

    /// A reliable packet carrying one optional header (`TimeSync`, `NetErrorDisconnect`):
    /// sequenced, its checksum encrypted, stored for resending.
    pub fn reliable_header(&mut self, optional: OptionalHeader) {
        let mask = optional.mask();
        self.reliable(vec![optional.encode()], mask);
    }

    fn interval(&self) -> u16 {
        1 + (self.elapsed / 0.5) as u16
    }

    /// A datagram to the client, through the channel when one is installed.
    fn emit(&mut self, packet: Vec<u8>) {
        if self.silent {
            return;
        }
        self.server_sent.push((self.elapsed, packet.clone()));
        let now = self.elapsed;
        match &mut self.channel {
            Some(channel) => {
                for flight in channel.send(now, Direction::ToClient, addr(SERVER), packet) {
                    self.inbox.push((flight.addr, flight.bytes));
                }
            }
            None => self.inbox.push((addr(SERVER), packet)),
        }
    }

    fn reliable_blob(&mut self, id: u64, queue: u16, data: &[u8]) {
        let fragment = fragment_header(id, data.len() as u16 + 16, queue);
        self.reliable(vec![fragment, data.to_vec()], flags::BLOB_FRAGMENTS);
    }

    /// The next reliable packet: sequence, key, stored for resends, then sent (or lost when a
    /// test asked for its sequence to be).
    fn reliable(&mut self, segments: Vec<Vec<u8>>, bits: u32) {
        self.sequence += 1;
        let header = PacketHeader {
            sequence: self.sequence,
            flags: bits | flags::ENCRYPTED_CHECKSUM,
            recipient: SLOT,
            interval: self.interval(),
            iteration: 1,
            ..Default::default()
        };
        let key = self.server_keys.as_mut().unwrap().next_key();
        let parts: Vec<&[u8]> = segments.iter().map(Vec::as_slice).collect();
        let packet = datagram(header, &parts, Some(key));
        self.store.packets.insert(
            self.sequence,
            Stored {
                key,
                segments,
                flags: bits,
            },
        );
        if self.drop_server_once.remove(&self.sequence) {
            self.server_sent.push((self.elapsed, packet));
            return;
        }
        self.emit(packet);
    }

    /// `FlowQueue::TransmitAcks` on the double's side: a NAKed packet goes again under its
    /// original sequence and key with `RETRANSMISSION`; one no longer stored is empty-acked.
    fn resend(&mut self, ids: &[u32]) {
        let mut empty = Vec::new();
        for &seq in ids {
            let Some(stored) = self.store.packets.get(&seq).cloned() else {
                empty.push(seq);
                continue;
            };
            let header = PacketHeader {
                sequence: seq,
                flags: flags::RETRANSMISSION | flags::ENCRYPTED_CHECKSUM | stored.flags,
                recipient: SLOT,
                interval: self.interval(),
                iteration: 1,
                ..Default::default()
            };
            self.resent.push(seq);
            let segments: Vec<&[u8]> = stored.segments.iter().map(Vec::as_slice).collect();
            let packet = datagram(header, &segments, Some(stored.key));
            self.emit(packet);
        }
        if !empty.is_empty() {
            self.control(OptionalHeader::EmptyAck(empty));
        }
    }

    /// An unreliable control packet (ACK, NAK, empty ack, disconnect): the header alone under the
    /// double's highest reliable sequence.
    pub fn control(&mut self, optional: OptionalHeader) {
        let header = PacketHeader {
            sequence: self.sequence,
            flags: optional.mask(),
            recipient: SLOT,
            interval: self.interval(),
            iteration: 1,
            ..Default::default()
        };
        let packet = datagram(header, &[&optional.encode()], None);
        self.emit(packet);
    }

    /// Each frame: release the channel's held datagrams, then the double's ACK/NAK if due.
    fn pump(&mut self) {
        let now = self.elapsed;
        let due = self
            .channel
            .as_mut()
            .map(|c| c.release(now))
            .unwrap_or_default();
        for flight in due {
            match flight.direction {
                Direction::ToClient => self.inbox.push((flight.addr, flight.bytes)),
                Direction::ToServer => self.answer(flight.addr, &flight.bytes),
            }
        }
        if self.silent || !self.connected {
            return;
        }
        if let Some(header) = self.tracker.as_mut().and_then(|t| t.due(now)) {
            self.control(header);
        }
    }

    /// The client sent a datagram: through the channel to [`State::answer`].
    fn arrive(&mut self, to: SocketAddr, bytes: &[u8]) {
        let now = self.elapsed;
        let arrived = match &mut self.channel {
            Some(channel) => channel.send(now, Direction::ToServer, to, bytes.to_vec()),
            None => vec![InFlight {
                release: now,
                direction: Direction::ToServer,
                addr: to,
                bytes: bytes.to_vec(),
            }],
        };
        for flight in arrived {
            self.answer(flight.addr, &flight.bytes);
        }
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
            self.connected_at = self.elapsed;
            let connect = OptionalHeader::Connect(Connect {
                server_time: SERVER_TIME,
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
            if self.connected {
                // A repeated logon (its answer was lost): the session stands; answer again.
                if let Some(packet) = self.connect.clone() {
                    self.emit(packet);
                }
                return;
            }
            let packet = datagram(header, &[&connect], None);
            self.connect = Some(packet.clone());
            self.emit(packet);
            self.server_keys = Some(Isaac::new(SERVER_SEED));
            // Reliable sequences start at 2, as `TransmitNewPackets` numbers them.
            self.sequence = 1;
            self.tracker = Some(ClientTracker::new(CLIENT_SEED, self.elapsed));
            self.connected = true;
            return;
        }
        if self.silent {
            return;
        }
        if header.flags & flags::RETRANSMISSION == 0
            && header.sequence != 0
            && header.flags & flags::ENCRYPTED_CHECKSUM != 0
            && self.drop_client_once.remove(&header.sequence)
        {
            return;
        }
        let mut key = None;
        if header.sequence != 0 {
            let Some(tracker) = self.tracker.as_mut() else {
                return;
            };
            match tracker.process(&header) {
                Verdict::Drop => return,
                Verdict::Accept(Some(k)) => {
                    sum ^= k;
                    key = Some(k);
                }
                Verdict::Accept(None) => {}
            }
        }
        let packet = split_packet(bytes, sum, self.connected).expect("a valid client packet");
        self.accepted.push(Accepted {
            time: self.elapsed,
            to,
            header: packet.header,
            optional: packet.optional.clone(),
            fragments: packet.fragments.clone(),
            bytes: bytes.to_vec(),
            key,
        });
        for optional in &packet.optional {
            match optional {
                OptionalHeader::Nak(ids) => self.resend(ids),
                OptionalHeader::Ack(highest) => self.store.retire(*highest),
                OptionalHeader::EmptyAck(ids) => {
                    if let Some(tracker) = self.tracker.as_mut() {
                        tracker.forget(ids);
                    }
                }
                OptionalHeader::NetErrorDisconnect(_) => self.disconnects_received += 1,
                _ => {}
            }
        }
        if header.flags & flags::CONNECT_ACK != 0 {
            assert_eq!(to, addr(ACK_PORT));
            if !self.character_set_sent && self.connected {
                self.character_set_sent = true;
                self.send_character_set(0x2300_0002_0000_0001);
            }
            return;
        }
        if header.flags & flags::LOGOFF != 0 {
            self.logged_off = true;
            return;
        }
        for fragment in packet.fragments {
            let Some((id, queue, data)) = self.assembler.add(&fragment) else {
                continue;
            };
            let opcode = u32::from_le_bytes(data[..4].try_into().unwrap());
            self.blobs.push((id, queue, data));
            match opcode {
                // `Proto_UI::SendEnterWorld`: the player is the character entered (its record's
                // visual and name, or the `Explorer` the double has always sent).
                0x0001_002B => {
                    let data = self.blobs.last().unwrap().2.clone();
                    let character = data
                        .get(4..12)
                        .map_or(PLAYER, |b| u64::from_le_bytes(b.try_into().unwrap()));
                    let (object, description) = self.player_entry(character);
                    self.reliable_blob(0x2300_0003_0000_0002, 1, &create_player(character));
                    self.reliable_blob(0x0300_0004_0000_0003, 1, &object);
                    self.reliable_blob(0x0300_0005_0000_0004, 1, &description);
                }
                // `ECM_Interp::SendEvt_InterpSEvent` / `SendEvt_InterpSEventEncrypt`.
                0x0001_0053 | 0x0001_00D0 => {
                    let data = self.blobs.last().unwrap().2.clone();
                    self.script_event(&data);
                }
                // `CChatEvent::CreateForSend`: the chat client's data.
                0x0001_00D3 => {
                    let data = self.blobs.last().unwrap().2.clone();
                    self.chat_event(&data);
                }
                // `Proto_UI::SendCharGenResult` (queue 8, in the clear without a session key).
                0x0001_002A => {
                    let data = self.blobs.last().unwrap().2.clone();
                    match ac2_net::protocol::decode_char_gen_result(&data) {
                        Ok((account, result)) => self.create_character(account, result),
                        Err(error) => self.script_errors.push(format!("char gen: {error}")),
                    }
                }
                // `Proto_UI::SendDeleteCharacter`: account, then the id; confirmed with
                // `ECM_Login::RecvEvt_CharacterDeletion`.
                0x0001_0029 => {
                    let data = self.blobs.last().unwrap().2.clone();
                    if let Some(id) = data
                        .get(data.len() - 8..)
                        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
                    {
                        let mut b = 0x0001_0029u32.to_le_bytes().to_vec();
                        b.extend(id.to_le_bytes());
                        self.reliable_blob(0x2300_0008_0000_0007, 1, &b);
                        self.deletions.push(id);
                    }
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

/// A double whose created characters are kept in `store` (a file, read now and written on every
/// creation) and whose appearances come from `rules`.
pub fn double_with(
    spawn: Spawn,
    store: &std::path::Path,
    mut rules: Option<Box<dyn CharGenRules>>,
) -> std::io::Result<(Double, Rc<RefCell<State>>)> {
    let explorer_visual = characters::explorer_visual_of(rules.as_mut());
    let state = Rc::new(RefCell::new(State {
        spawn,
        characters: CharacterStore::open(store)?,
        rules,
        explorer_visual,
        ..State::default()
    }));
    Ok((Double(Rc::clone(&state)), state))
}

/// A double with `rules` and no character file: created characters are kept in memory, and
/// `Explorer` wears the rules' default appearance.
pub fn double_with_rules(
    spawn: Spawn,
    mut rules: Box<dyn CharGenRules>,
) -> (Double, Rc<RefCell<State>>) {
    let explorer_visual = characters::explorer_visual_of(Some(&mut rules));
    let state = Rc::new(RefCell::new(State {
        spawn,
        rules: Some(rules),
        explorer_visual,
        ..State::default()
    }));
    (Double(Rc::clone(&state)), state)
}

impl Transport for Double {
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
        let mut state = self.0.borrow_mut();
        state.pump();
        Ok(std::mem::take(&mut state.inbox))
    }

    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
        let mut state = self.0.borrow_mut();
        state.sent.push((to, datagram.to_vec()));
        let now = state.elapsed;
        state.sent_at.push(now);
        state.arrive(to, datagram);
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
