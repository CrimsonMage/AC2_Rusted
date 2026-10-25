//! An independent check of every datagram the client sent during a lossy session: the
//! client's half of `SharedNet`, rebuilt from the datagrams that reached it (`ClientTracker`
//! run on its receive side) and from its own first transmissions.
//!
//! - New reliable packets: sequences 2, 3, 4... in order, checksums under the ISAAC key of
//!   each (`FlowQueue::TransmitNewPackets`), time-sensitive payloads stamped with the frame's
//!   clock (`NetPacket::UpdateTimeSensitiveHeaders`), which follows the server's time syncs
//!   (`ClientNet::HandleTimeSynch` -> `Timer::set_time`).
//! - Resends (`FlowQueue::TransmitAcks`): only of a sequence a NAK that arrived in the same
//!   frame named, byte for byte the first transmission without its unreliable headers
//!   (`RemoveDisposableOptionalHeaders`), time-sensitive payloads restamped, the original key,
//!   `RETRANSMISSION | ENCRYPTED_CHECKSUM`.
//! - Unreliable packets carry the highest reliable sequence sent.
//! - ACK and NAK lists (`SharedNet::EnqueuePak` / `EnqueueNaks`): exactly when the model says
//!   they are due, with exactly its contents.
//! - Empty acks only of sequences the server NAKed and that were not resent in that frame.

use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;

use ac2_net::{flags, split_packet, Fragment, Isaac, OptionalHeader, PacketHeader};
use legions_server_double::{
    datagram, fragment_header_bytes, ClientTracker, CLIENT_SEED, NET_ID, SERVER_SEED,
};

/// One client frame: its clock, what arrived, what it sent.
#[derive(Debug, Default, Clone)]
pub struct Frame {
    pub time: f64,
    pub inbound: Vec<Vec<u8>>,
    pub outbound: Vec<(SocketAddr, Vec<u8>)>,
}

/// What the oracle counted, for the test's sanity checks.
#[derive(Debug, Default, Clone, Copy)]
pub struct Counts {
    pub reliable: u32,
    pub resends: u32,
    pub unreliable: u32,
    pub acks: u32,
    pub naks: u32,
    pub empty_acks: u32,
}

/// Headers `RemoveDisposableOptionalHeaders` strips (`m_Flags & 1`).
const UNRELIABLE: u32 = flags::NAK
    | flags::EMPTY_ACK
    | flags::ACK
    | flags::LOGOFF
    | flags::EMPTY_HEADER
    | flags::LOGON_SERVER_ADDR
    | flags::LOGON
    | flags::WORLD_LOGIN_REQUEST
    | flags::NET_ERROR
    | flags::CICMD
    | flags::CONNECT
    | flags::CONNECT_ACK;

fn body_sum(bytes: &[u8]) -> u32 {
    let header = PacketHeader::decode(bytes).unwrap();
    header.checksum.wrapping_sub(header.header_checksum())
}

/// The key of the client's reliable sequence `seq` (keys are drawn in sequence order from 2).
struct ClientKeys {
    keys: Vec<u32>,
    isaac: Isaac,
}

impl ClientKeys {
    fn key(&mut self, seq: u32) -> u32 {
        let index = (seq - 2) as usize;
        while self.keys.len() <= index {
            self.keys.push(self.isaac.next_key());
        }
        self.keys[index]
    }
}

/// The body segments of a packet as `datagram` sums them: each header, then each fragment's
/// header and data.
fn segments(headers: &[OptionalHeader], fragments: &[Fragment]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = headers.iter().map(OptionalHeader::encode).collect();
    for fragment in fragments {
        out.push(fragment_header_bytes(fragment));
        out.push(fragment.data.clone());
    }
    out
}

/// The client clock as `Timer` keeps it: `update_time` at the top of each frame, `set_time`
/// when a time sync is processed.
#[derive(Default)]
struct Clock {
    base: f64,
    cur: f64,
    local: f64,
}

impl Clock {
    fn update(&mut self, elapsed: f64) {
        let old = self.cur;
        self.cur = elapsed + self.base;
        if self.cur < old {
            self.cur = old + 1e-12;
        }
        self.local = self.cur - self.base;
    }

    fn set(&mut self, time: f64, elapsed: f64) {
        let delta = time - (elapsed + self.base);
        if 1e-9 < delta {
            self.base += delta;
            self.cur = time;
        }
    }
}

/// Restamps a time-sensitive header as a send does: the server clock
/// (`get_elapsed_time() + base_time`) and `(float)local_time`.
fn restamp(header: &OptionalHeader, server_time: f64, local_time: f64) -> OptionalHeader {
    match header {
        OptionalHeader::TimeSync(_) => OptionalHeader::TimeSync(server_time),
        OptionalHeader::EchoRequest(_) => OptionalHeader::EchoRequest(local_time as f32),
        other => other.clone(),
    }
}

/// Checks every frame; panics with the frame time on the first disagreement.
pub fn check(frames: &[Frame]) -> Counts {
    let mut counts = Counts::default();
    let mut model: Option<ClientTracker> = None;
    let mut connected = false;
    let mut keys = ClientKeys {
        keys: Vec::new(),
        isaac: Isaac::new(CLIENT_SEED),
    };
    let mut highest_sent = 1u32;
    // First transmissions: headers and fragments.
    let mut first: BTreeMap<u32, (Vec<OptionalHeader>, Vec<Fragment>)> = BTreeMap::new();
    let mut ever_naked = BTreeSet::new();
    let mut clock = Clock::default();
    for frame in frames {
        let t = frame.time;
        clock.update(t);
        // The receive side: what the client's `ReceiverData` makes of each arrival. Every
        // sequence the frame's NAKs named (`FlowQueue::EnqueueAcks` merges them).
        let mut naked_now = BTreeSet::new();
        for bytes in &frame.inbound {
            let header = PacketHeader::decode(bytes).unwrap();
            if header.sequence == 0 {
                if header.flags & flags::CONNECT != 0 && model.is_none() {
                    model = Some(ClientTracker::new(SERVER_SEED, clock.cur));
                }
                continue;
            }
            let Some(tracker) = model.as_mut() else {
                continue;
            };
            let key = match tracker.process(&header) {
                legions_server_double::Verdict::Drop => continue,
                legions_server_double::Verdict::Accept(key) => key,
            };
            let sum = body_sum(bytes) ^ key.unwrap_or(0);
            let packet = split_packet(bytes, sum, true)
                .unwrap_or_else(|e| panic!("{t}: server packet fails its checksum: {e}"));
            connected |= header.flags & flags::CONNECT == 0;
            for optional in &packet.optional {
                match optional {
                    OptionalHeader::Nak(ids) => {
                        naked_now.extend(ids.iter().copied());
                        ever_naked.extend(ids.iter().copied());
                    }
                    OptionalHeader::EmptyAck(ids) => tracker.forget(ids),
                    OptionalHeader::TimeSync(time) => clock.set(*time, t),
                    _ => {}
                }
            }
        }
        let expected = match (connected, model.as_mut()) {
            (true, Some(tracker)) => tracker.due(clock.cur),
            _ => None,
        };
        // The send side, flushed on the clock after this frame's time syncs.
        let server_time = t + clock.base;
        let mut acknowledgements = Vec::new();
        let mut resent = BTreeSet::new();
        for (to, bytes) in &frame.outbound {
            let header = PacketHeader::decode(bytes).unwrap();
            if header.sequence == 0 {
                continue;
            }
            assert_eq!(
                (header.recipient, header.iteration),
                (NET_ID as u16, 1),
                "{t}"
            );
            let encrypted = header.flags & flags::ENCRYPTED_CHECKSUM != 0;
            let retransmission = header.flags & flags::RETRANSMISSION != 0;
            let key = encrypted.then(|| keys.key(header.sequence));
            let packet = split_packet(bytes, body_sum(bytes) ^ key.unwrap_or(0), true)
                .unwrap_or_else(|e| panic!("{t}: client packet {header:?} fails: {e}"));
            let _ = to;
            if encrypted && !retransmission {
                counts.reliable += 1;
                assert_eq!(header.sequence, highest_sent + 1, "{t}: new sequence");
                highest_sent = header.sequence;
                for optional in &packet.optional {
                    let stamped = restamp(optional, server_time, clock.local);
                    assert_eq!(optional, &stamped, "{t}: time-sensitive payload");
                }
                first.insert(
                    header.sequence,
                    (packet.optional.clone(), packet.fragments.clone()),
                );
            } else if encrypted {
                counts.resends += 1;
                assert!(
                    naked_now.contains(&header.sequence),
                    "{t}: resent {} without a NAK this frame",
                    header.sequence
                );
                assert!(resent.insert(header.sequence), "{t}: resent twice");
                let (headers, fragments) = &first[&header.sequence];
                let kept: Vec<OptionalHeader> = headers
                    .iter()
                    .filter(|h| h.mask() & UNRELIABLE == 0)
                    .map(|h| restamp(h, server_time, clock.local))
                    .collect();
                let mut bits = flags::RETRANSMISSION | flags::ENCRYPTED_CHECKSUM;
                if !fragments.is_empty() {
                    bits |= flags::BLOB_FRAGMENTS;
                }
                bits |= kept.iter().fold(0, |all, h| all | h.mask());
                let parts = segments(&kept, fragments);
                let parts: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
                let expected = datagram(
                    PacketHeader {
                        sequence: header.sequence,
                        flags: bits,
                        recipient: NET_ID as u16,
                        interval: header.interval,
                        iteration: 1,
                        ..Default::default()
                    },
                    &parts,
                    key,
                );
                assert_eq!(bytes, &expected, "{t}: resend of {}", header.sequence);
            } else {
                counts.unreliable += 1;
                assert_eq!(header.sequence, highest_sent, "{t}: unreliable sequence");
            }
            for optional in &packet.optional {
                match optional {
                    OptionalHeader::Ack(_) | OptionalHeader::Nak(_) => {
                        acknowledgements.push(optional.clone())
                    }
                    OptionalHeader::EmptyAck(ids) => {
                        counts.empty_acks += 1;
                        for id in ids {
                            assert!(ever_naked.contains(id), "{t}: empty ack of {id}");
                            assert!(!resent.contains(id), "{t}: {id} resent and empty-acked");
                        }
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(
            acknowledgements,
            expected.into_iter().collect::<Vec<_>>(),
            "{t}: ACK/NAK"
        );
        for header in &acknowledgements {
            match header {
                OptionalHeader::Ack(_) => counts.acks += 1,
                _ => counts.naks += 1,
            }
        }
    }
    counts
}
