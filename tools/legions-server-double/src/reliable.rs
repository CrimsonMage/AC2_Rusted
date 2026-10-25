//! The double's half of the reliable transport (test tooling), the server side of the same
//! `SharedNet` rules the client follows: the client's sequences with the ISAAC key reserved for
//! each gap (`ProcessNewSeqNum`), its NAK/ACK cadence (`EnqueueNaks` every 0.6 s while
//! sequences are missing, `EnqueuePak` every 2 s), and a store of the double's reliable
//! packets resent on a NAK (`TransmitAcks`) or answered with an empty ack.

use std::collections::BTreeMap;

use ac2_net::{flags, Isaac, PacketHeader};

/// `MAX_SEQUENCE_LIST`: the most ids one NAK or empty ack carries.
pub const MAX_IDS: usize = 0x72;

/// `TimeStampUtils` 32-bit sequence comparison.
pub fn newer(lhs: u32, rhs: u32) -> bool {
    lhs != rhs && lhs.wrapping_sub(rhs) < 0x8000_0000
}

/// What the double knows about the client's sends (`ReceiverData`).
#[derive(Debug)]
pub struct ClientTracker {
    pub highest: u32,
    /// Missing sequences and the key reserved for each.
    pub naked: BTreeMap<u32, u32>,
    keys: Isaac,
    /// `timeStamp_`: when the last ACK or NAK list went out.
    pub ack_stamp: f64,
}

/// The tracker's verdict on a client packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Process it; the key for an encrypted packet.
    Accept(Option<u32>),
    /// A duplicate, or a sequence the client was never asked to resend.
    Drop,
}

impl ClientTracker {
    pub fn new(seed: u32, now: f64) -> Self {
        Self {
            highest: 1,
            naked: BTreeMap::new(),
            keys: Isaac::new(seed),
            ack_stamp: now,
        }
    }

    /// `SharedNet::ProcessNewSeqNum` / `ProcessNewestSeqNum`.
    pub fn process(&mut self, header: &PacketHeader) -> Verdict {
        let seq = header.sequence;
        let encrypted = header.flags & flags::ENCRYPTED_CHECKSUM != 0;
        let mut key = None;
        if encrypted && !newer(seq, self.highest) {
            match self.naked.remove(&seq) {
                Some(k) => key = Some(k),
                None => return Verdict::Drop,
            }
        }
        if newer(seq, self.highest) {
            let end = if encrypted { seq } else { seq.wrapping_add(1) };
            let mut missing = self.highest.wrapping_add(1);
            while missing != end {
                if missing != 0 && !self.naked.contains_key(&missing) {
                    let k = self.keys.next_key();
                    self.naked.insert(missing, k);
                }
                missing = missing.wrapping_add(1);
            }
            self.highest = seq;
        }
        if encrypted {
            Verdict::Accept(Some(key.unwrap_or_else(|| self.keys.next_key())))
        } else {
            Verdict::Accept(None)
        }
    }

    /// `SharedNet::HandleEmptyAck`.
    pub fn forget(&mut self, ids: &[u32]) {
        for id in ids {
            self.naked.remove(id);
        }
    }

    /// The NAK list (lowest first) or ACK due at `now`, per the 0.6 s / 2 s cadence.
    pub fn due(&mut self, now: f64) -> Option<ac2_net::OptionalHeader> {
        let since = now - self.ack_stamp;
        let header = if self.naked.is_empty() {
            (2.0 <= since).then_some(ac2_net::OptionalHeader::Ack(self.highest))
        } else {
            (0.6 < since).then(|| {
                ac2_net::OptionalHeader::Nak(self.naked.keys().take(MAX_IDS).copied().collect())
            })
        }?;
        self.ack_stamp = now;
        Some(header)
    }
}

/// One reliable packet the double sent: its key and its body segments (each fragment's header
/// and data are summed apart, as `CNetLayerPacket` sums them).
#[derive(Debug, Clone)]
pub struct Stored {
    pub key: u32,
    pub segments: Vec<Vec<u8>>,
    /// Header flags without `RETRANSMISSION` and `ENCRYPTED_CHECKSUM`.
    pub flags: u32,
}

/// The double's reliable packets kept for resending (`SentPacketStore`).
#[derive(Debug, Default)]
pub struct SentStore {
    pub packets: BTreeMap<u32, Stored>,
}

impl SentStore {
    /// `SentPacketStore::Flush`: the client acknowledged everything below `highest`.
    pub fn retire(&mut self, highest: u32) {
        self.packets.retain(|&seq, _| !newer(highest, seq));
    }
}
