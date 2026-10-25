//! `ReceiverData`: what the client knows about one server's sends: the highest sequence seen,
//! the sequences it NAKed (with the ISAAC key reserved for each), both checksum generators, the
//! acknowledgement timer and the remote interval.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use crate::blob::sequence_newer;
use crate::header::{flags, PacketHeader};
use crate::isaac::Isaac;
use crate::optional::{OptionalHeader, MAX_SEQUENCE_LIST};

/// `ConnectionState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionState {
    Disconnected = 0,
    AwaitingWorldAuth = 1,
    AuthSent = 2,
    ConnectionRequestSent = 3,
    ConnectionRequestAcked = 4,
    Connected = 5,
    DisconnectReceived = 6,
    DisconnectSent = 7,
}

/// Seconds between acknowledgements while nothing is missing (`SharedNet::EnqueuePak`).
pub(crate) const ACK_SECONDS: f64 = 2.0;
/// Seconds between NAK lists while sequences are missing (`SharedNet::EnqueueNaks`).
pub(crate) const NAK_SECONDS: f64 = 0.6;
/// NAKed sequences beyond which packets are refused (`SharedNet::SeqIDSanityCheck`).
const MAX_NAKED: usize = 40_000;

#[derive(Debug)]
pub(crate) struct Receiver {
    /// `m_NetID`: the `recID_` the client sends under (`CConnectHeader::NetID`).
    pub net_id: u16,
    pub iteration: u16,
    pub addr: SocketAddr,
    pub highest_received: u32,
    /// `m_SeqIDsWeNAKed`: missing sequence -> the key reserved for it.
    pub naked: BTreeMap<u32, u32>,
    incoming: Isaac,
    pub outgoing: Isaac,
    /// `timeStamp_` (`cur_time`): when the last ACK or NAK list was queued.
    time_stamp: f64,
    pub remote_interval: u16,
    pub bytes_received: u32,
    pub state: ConnectionState,
    /// `m_TimeLastConnectionStateChanged` (`local_time`).
    pub state_changed: f64,
    /// `m_qwReferralCookie`.
    pub cookie: u64,
    /// `lastSentHandshake_` (`local_time`).
    pub last_sent_handshake: f64,
    /// `m_LocalTimeLastGotData`.
    pub last_got_data: f64,
    /// `RoundTripLatency_`.
    pub round_trip: f32,
}

impl Receiver {
    /// `ReceiverData::Init` / `SharedInit` (the map key is `m_RecID`): the highest sequence starts at 1; the incoming
    /// generator takes the connect's outgoing seed and the outgoing generator its incoming seed.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        addr: SocketAddr,
        net_id: u16,
        iteration: u16,
        remote_interval: u16,
        seed_incoming: u32,
        seed_outgoing: u32,
        cur_time: f64,
        local_time: f64,
    ) -> Self {
        Self {
            net_id,
            iteration,
            addr,
            highest_received: 1,
            naked: BTreeMap::new(),
            incoming: Isaac::new(seed_incoming),
            outgoing: Isaac::new(seed_outgoing),
            time_stamp: cur_time,
            remote_interval,
            bytes_received: 0,
            state: ConnectionState::Disconnected,
            state_changed: local_time,
            cookie: 0,
            last_sent_handshake: 0.0,
            last_got_data: local_time,
            round_trip: 0.0,
        }
    }

    /// `ReceiverData::SetConnectionState`.
    pub fn set_state(&mut self, state: ConnectionState, local_time: f64) {
        self.state = state;
        self.state_changed = local_time;
    }

    /// `ReceiverData::AddNakked`: reserve the next incoming key for a missing sequence unless
    /// one is given.
    pub fn add_nakked(&mut self, seq: u32, key: Option<u32>) {
        if self.naked.get(&seq).is_some_and(|&k| k != 0) {
            return;
        }
        let key = key.unwrap_or_else(|| self.incoming.next_key());
        self.naked.insert(seq, key);
    }

    /// `SharedNet::SeqIDSanityCheck`: no more than `0x7fff` ahead, and a bounded NAK list.
    fn sanity_check(&self, seq: u32) -> bool {
        let limit = self.highest_received.wrapping_add(0x7FFF);
        if seq != limit && sequence_newer(seq, limit) {
            return false;
        }
        self.naked.len() <= MAX_NAKED
    }

    /// `SharedNet::ProcessNewSeqNum` and `ProcessNewestSeqNum`: the decryption key for a
    /// sequenced packet, `None` to drop it. A newer sequence NAKs every gap first (keys are
    /// drawn in sequence order); an older encrypted one must be a NAKed sequence and takes its
    /// reserved key. An unencrypted packet carries the sender's highest reliable sequence,
    /// so a newer one NAKs that sequence as well.
    pub fn process_sequence(&mut self, header: &PacketHeader) -> Option<Option<u32>> {
        let seq = header.sequence;
        if !self.sanity_check(seq) {
            return None;
        }
        let encrypted = header.flags & flags::ENCRYPTED_CHECKSUM != 0;
        let mut key = None;
        if encrypted && !sequence_newer(seq, self.highest_received) {
            let reserved = self.naked.remove(&seq).filter(|&k| k != 0)?;
            key = Some(reserved);
        }
        if sequence_newer(seq, self.highest_received) {
            let end = if encrypted { seq } else { seq.wrapping_add(1) };
            let mut missing = self.highest_received.wrapping_add(1);
            while missing != end {
                if missing != 0 {
                    self.add_nakked(missing, None);
                }
                missing = missing.wrapping_add(1);
            }
            self.highest_received = seq;
        }
        if encrypted {
            Some(Some(key.unwrap_or_else(|| self.incoming.next_key())))
        } else {
            Some(None)
        }
    }

    /// `ReceiverData::GetNaks`: the lowest `0x72` missing sequences.
    fn naks(&self) -> Vec<u32> {
        self.naked.keys().take(MAX_SEQUENCE_LIST).copied().collect()
    }

    /// `SharedNet::ProcessConnection` -> `EnqueuePak` / `EnqueueNaks`: an ACK of the highest
    /// sequence every 2 s while nothing is missing, else the NAK list every 0.6 s.
    pub fn acknowledgement(&mut self, cur_time: f64) -> Option<OptionalHeader> {
        let since = cur_time - self.time_stamp;
        let header = if self.naked.is_empty() {
            (ACK_SECONDS <= since).then(|| OptionalHeader::Ack(self.highest_received))
        } else {
            (NAK_SECONDS < since).then(|| OptionalHeader::Nak(self.naks()))
        }?;
        self.time_stamp = cur_time;
        Some(header)
    }

    /// `SharedNet::HandleEmptyAck`: sequences the server will not resend.
    pub fn forget(&mut self, ids: &[u32]) {
        for id in ids {
            self.naked.remove(id);
        }
    }

    /// `SharedNet::ProcessNewRemoteInterval`: when the server's interval advances, report the
    /// bytes received during the previous one in a flow header.
    pub fn remote_interval(&mut self, interval: u16) -> Option<OptionalHeader> {
        let flow = (self.remote_interval != 0).then_some(OptionalHeader::Flow {
            bytes: self.bytes_received,
            interval: self.remote_interval,
        });
        self.remote_interval = interval;
        self.bytes_received = 0;
        flow
    }
}

#[cfg(test)]
mod tests {
    use super::Receiver;
    use crate::header::{flags, PacketHeader};
    use crate::isaac::Isaac;
    use crate::optional::OptionalHeader;

    fn receiver() -> Receiver {
        Receiver::new(
            "127.0.0.1:9000".parse().unwrap(),
            5,
            1,
            0,
            0x1111,
            0x2222,
            0.0,
            0.0,
        )
    }

    fn sequenced(sequence: u32, encrypted: bool) -> PacketHeader {
        PacketHeader {
            sequence,
            flags: if encrypted {
                flags::ENCRYPTED_CHECKSUM
            } else {
                0
            },
            ..Default::default()
        }
    }

    #[test]
    fn gaps_reserve_keys_in_sequence_order() {
        let mut expected = Isaac::new(0x1111);
        let keys: Vec<u32> = (0..4).map(|_| expected.next_key()).collect();
        let mut r = receiver();
        // Sequence 4 arrives first: 2 and 3 take keys 0 and 1, 4 takes key 2.
        assert_eq!(r.process_sequence(&sequenced(4, true)), Some(Some(keys[2])));
        assert_eq!(r.naked.keys().copied().collect::<Vec<_>>(), [2, 3]);
        assert_eq!(r.process_sequence(&sequenced(3, true)), Some(Some(keys[1])));
        // A duplicate of a delivered sequence is dropped.
        assert_eq!(r.process_sequence(&sequenced(3, true)), None);
        assert_eq!(r.process_sequence(&sequenced(2, true)), Some(Some(keys[0])));
        assert_eq!(r.process_sequence(&sequenced(5, true)), Some(Some(keys[3])));
        assert!(r.naked.is_empty());
    }

    #[test]
    fn an_unencrypted_packet_naks_the_reliable_sequence_it_names() {
        let mut r = receiver();
        assert_eq!(r.process_sequence(&sequenced(2, false)), Some(None));
        assert_eq!(r.naked.keys().copied().collect::<Vec<_>>(), [2]);
        assert_eq!(r.highest_received, 2);
    }

    #[test]
    fn ack_every_two_seconds_nak_every_point_six() {
        let mut r = receiver();
        assert_eq!(r.acknowledgement(1.9), None);
        assert_eq!(r.acknowledgement(2.0), Some(OptionalHeader::Ack(1)));
        r.process_sequence(&sequenced(4, true));
        assert_eq!(r.acknowledgement(2.5), None);
        assert_eq!(
            r.acknowledgement(2.75),
            Some(OptionalHeader::Nak(vec![2, 3]))
        );
        r.forget(&[2, 3]);
        assert_eq!(r.acknowledgement(4.5), None);
        assert_eq!(r.acknowledgement(4.75), Some(OptionalHeader::Ack(4)));
    }

    #[test]
    fn sequences_wrap_past_zero_which_is_never_naked() {
        let mut expected = Isaac::new(0x1111);
        let keys: Vec<u32> = (0..3).map(|_| expected.next_key()).collect();
        let mut r = receiver();
        r.highest_received = 0xFFFF_FFFE;
        // 2 arrives: 0xFFFFFFFF then 1 take keys in sequence order; 0 is skipped.
        assert_eq!(r.process_sequence(&sequenced(2, true)), Some(Some(keys[2])));
        assert_eq!(r.naked.get(&0xFFFF_FFFF), Some(&keys[0]));
        assert_eq!(r.naked.get(&1), Some(&keys[1]));
        // `GetNaks` walks the AVL tree in key order.
        assert_eq!(r.naks(), [1, 0xFFFF_FFFF]);
        assert_eq!(
            r.process_sequence(&sequenced(0xFFFF_FFFF, true)),
            Some(Some(keys[0]))
        );
        assert_eq!(r.process_sequence(&sequenced(1, true)), Some(Some(keys[1])));
        assert!(r.naked.is_empty());
        assert_eq!(r.highest_received, 2);
    }

    #[test]
    fn more_than_40000_missing_sequences_refuse_every_packet() {
        let mut r = receiver();
        // At most `0x7fff` ahead per packet: two jumps leave 2..=40001 missing. 40,000 missing is
        // still sane (`SeqIDSanityCheck`: refused above 40,000).
        assert_eq!(r.process_sequence(&sequenced(0x8000, false)), Some(None));
        assert_eq!(r.process_sequence(&sequenced(40_001, false)), Some(None));
        assert_eq!(r.naked.len(), 40_000);
        assert_eq!(r.process_sequence(&sequenced(40_003, false)), Some(None));
        assert_eq!(r.naked.len(), 40_002);
        // Now even a missing sequence's resend is refused.
        assert_eq!(r.process_sequence(&sequenced(2, true)), None);
        assert_eq!(r.process_sequence(&sequenced(40_004, false)), None);
        r.forget(&[2, 3]);
        assert!(r.process_sequence(&sequenced(4, true)).is_some());
    }

    #[test]
    fn far_future_sequences_fail_the_sanity_check() {
        let mut r = receiver();
        assert_eq!(r.process_sequence(&sequenced(1 + 0x8000, true)), None);
        assert!(r.process_sequence(&sequenced(1 + 0x7FFF, false)).is_some());
    }
}
