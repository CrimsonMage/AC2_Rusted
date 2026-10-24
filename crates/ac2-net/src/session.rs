//! One connected session: reliable sequenced datagrams, ISAAC checksum keys, blob fragmentation
//! and reassembly, ordered delivery, acknowledgement and retransmission.
//!
//! Sequences start at 2: `FlowQueue` initializes the highest id sent to 1 and increments before
//! each new reliable packet, and `ReceiverData` starts the highest id received at 1. One ISAAC key
//! is consumed per new sequence in order; retransmissions reuse the stored datagram.
//!
//! Blob ids follow `Proto_UI::SendBuff` and `NetBlobIDUtils`: the high word carries the ordering
//! type in bits 24..28 (1 ordered, 3 unordered), the server bits 16..23 and a 16-bit stamp; the
//! low word is the running non-ephemeral sequence. Queues 3 and 7 share one ordered stamp counter.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::handshake::Connect;
use crate::header::{flags, PacketHeader};
use crate::isaac::Isaac;
use crate::packet::{decode_packet, encode_packet, Fragment};
use crate::NetError;

/// Furthest a received sequence may run ahead of the contiguous one.
const WINDOW: u32 = 512;
/// Largest blob assembled or sent.
const MAX_BLOB: usize = 64 * 1024;
/// Payload bytes per sent fragment.
const FRAGMENT_DATA: usize = 448;
/// Unacknowledged sent datagrams allowed at once.
const MAX_PENDING: usize = 256;
/// Blobs assembling at once.
const MAX_ASSEMBLIES: usize = 32;
/// Ordered blobs held back waiting for an earlier stamp.
const MAX_HELD: usize = 512;
/// Retransmissions of one datagram before the session fails.
const MAX_RETRIES: u8 = 8;
/// Idle time after which an echo request keeps the session alive.
const KEEPALIVE: Duration = Duration::from_secs(5);

/// Ordering type of a blob id (`NetBlobIDUtils::GetOrderingType`, bits 56..60).
fn ordering_type(id: u64) -> u64 {
    (id >> 56) & 0x1F
}

/// `NetBlobIDUtils::GetOrderingStamp`.
fn ordering_stamp(id: u64) -> u16 {
    (id >> 32) as u16
}

/// A completed blob: its id, the queue it arrived on and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    pub id: u64,
    pub queue: u16,
    pub data: Vec<u8>,
}

/// What one received datagram produced.
#[derive(Debug, Default)]
pub struct Inbound {
    /// Completed blobs in delivery order.
    pub blobs: Vec<Blob>,
    /// Datagrams to send back now: acknowledgements, echo responses, requested retransmissions.
    pub replies: Vec<Vec<u8>>,
    pub logoff: bool,
    /// The server's disconnect error, if it closed the session.
    pub disconnect: Option<NetError>,
    pub time_sync: Option<f64>,
}

struct Assembly {
    queue: u16,
    pieces: Vec<Option<Vec<u8>>>,
    bytes: usize,
}

struct Pending {
    bytes: Arc<[u8]>,
    sent: Instant,
    retries: u8,
}

pub struct Session {
    recipient: u16,
    iteration: u16,
    incoming: Isaac,
    outgoing: Isaac,
    started: Instant,
    // Receive state.
    generated: u32,
    contiguous: u32,
    seen: BTreeSet<u32>,
    keys: BTreeMap<u32, u32>,
    assemblies: BTreeMap<u64, Assembly>,
    completed: VecDeque<u64>,
    next_stamp: u32,
    held: BTreeMap<u16, Blob>,
    // Send state.
    next_sequence: u32,
    pending: BTreeMap<u32, Pending>,
    ordered_counter: u16,
    unordered_counter: u16,
    non_ephemeral: u32,
    last_sent: Instant,
}

impl Session {
    /// A session for the identity a connect named. The client's incoming generator takes the
    /// connect's outgoing seed and its outgoing generator the incoming seed
    /// (`ClientNet::InitCrypto`, `ReceiverData::InitCrypto`).
    pub fn new(connect: &Connect, recipient: u16, iteration: u16, now: Instant) -> Self {
        Self {
            recipient,
            iteration,
            incoming: Isaac::new(connect.outgoing_seed),
            outgoing: Isaac::new(connect.incoming_seed),
            started: now,
            generated: 1,
            contiguous: 1,
            seen: BTreeSet::new(),
            keys: BTreeMap::new(),
            assemblies: BTreeMap::new(),
            completed: VecDeque::new(),
            next_stamp: 1,
            held: BTreeMap::new(),
            next_sequence: 2,
            pending: BTreeMap::new(),
            ordered_counter: 0,
            unordered_counter: 0,
            non_ephemeral: 0,
            last_sent: now,
        }
    }

    pub fn recipient(&self) -> u16 {
        self.recipient
    }

    /// Whether sent data still awaits acknowledgement.
    pub fn has_unacknowledged(&self) -> bool {
        !self.pending.is_empty()
    }

    fn control(&mut self, optional: &[(u32, Vec<u8>)], now: Instant) -> Vec<u8> {
        self.last_sent = now;
        encode_packet(
            PacketHeader {
                recipient: self.recipient,
                iteration: self.iteration,
                ..Default::default()
            },
            optional,
            &[],
            None,
        )
    }

    fn seconds(&self, now: Instant) -> f32 {
        now.saturating_duration_since(self.started).as_secs_f32()
    }

    /// Processes one datagram from the server.
    pub fn receive(&mut self, bytes: &[u8], now: Instant) -> Result<Inbound, NetError> {
        let header = PacketHeader::decode(bytes).ok_or(NetError::Truncated)?;
        if header.recipient != self.recipient || header.iteration != self.iteration {
            return Err(NetError::Sequence);
        }
        let sequenced = header.sequence != 0;
        let key = if header.flags & flags::ENCRYPTED_CHECKSUM != 0 {
            let seq = header.sequence;
            if seq < 2
                || seq > self.contiguous.saturating_add(WINDOW)
                || seq.saturating_add(WINDOW) < self.contiguous
            {
                return Err(NetError::Sequence);
            }
            while self.generated < seq {
                self.generated += 1;
                self.keys.insert(self.generated, self.incoming.next_key());
            }
            Some(*self.keys.get(&seq).ok_or(NetError::Sequence)?)
        } else {
            None
        };
        let packet = decode_packet(bytes, key)?;
        let mut inbound = Inbound {
            logoff: packet.logoff,
            time_sync: packet.time_sync,
            disconnect: packet
                .disconnect
                .map(|(string_id, table_id)| NetError::Server {
                    string_id,
                    table_id,
                }),
            ..Default::default()
        };
        // `HandleNak` runs before `HandlePak`: resend what the server is missing, then retire
        // everything at or below its acknowledgement.
        for seq in packet.naks.iter().copied().collect::<BTreeSet<_>>() {
            if let Some(pending) = self.pending.get_mut(&seq) {
                if pending.retries >= MAX_RETRIES {
                    return Err(NetError::RetryLimit);
                }
                pending.retries += 1;
                pending.sent = now;
                inbound.replies.push(pending.bytes.to_vec());
            }
        }
        if let Some(ack) = packet.ack {
            self.pending.retain(|&seq, _| seq > ack);
        }
        if let Some(echo) = packet.echo_request {
            let mut body = echo.to_le_bytes().to_vec();
            body.extend(self.seconds(now).to_le_bytes());
            let reply = self.control(&[(flags::ECHO_RESPONSE, body)], now);
            inbound.replies.push(reply);
        }
        if sequenced {
            let seq = header.sequence;
            let duplicate = seq <= self.contiguous || self.seen.contains(&seq);
            if !duplicate {
                let mut ordered = Vec::new();
                for fragment in packet.fragments {
                    if let Some(blob) = self.assemble(fragment)? {
                        if ordering_type(blob.id) == 1 {
                            ordered.push(blob);
                        } else {
                            inbound.blobs.push(blob);
                        }
                    }
                }
                ordered.sort_by_key(|blob| ordering_stamp(blob.id));
                for blob in ordered {
                    self.deliver_ordered(blob, &mut inbound.blobs)?;
                }
                self.seen.insert(seq);
                while self.seen.remove(&(self.contiguous + 1)) {
                    self.contiguous += 1;
                }
                let floor = self.contiguous.saturating_sub(WINDOW);
                self.keys.retain(|&id, _| id >= floor);
            }
            let ack = self.control(&[(flags::ACK, self.contiguous.to_le_bytes().to_vec())], now);
            inbound.replies.push(ack);
        }
        Ok(inbound)
    }

    fn assemble(&mut self, fragment: Fragment) -> Result<Option<Blob>, NetError> {
        if self.completed.contains(&fragment.blob_id) {
            return Ok(None);
        }
        if fragment.count > 256 || fragment.data.len() > MAX_BLOB {
            return Err(NetError::Fragment);
        }
        if !self.assemblies.contains_key(&fragment.blob_id)
            && self.assemblies.len() >= MAX_ASSEMBLIES
        {
            return Err(NetError::Capacity);
        }
        let assembly = self
            .assemblies
            .entry(fragment.blob_id)
            .or_insert_with(|| Assembly {
                queue: fragment.queue,
                pieces: vec![None; usize::from(fragment.count)],
                bytes: 0,
            });
        if assembly.queue != fragment.queue || assembly.pieces.len() != usize::from(fragment.count)
        {
            return Err(NetError::Fragment);
        }
        let slot = &mut assembly.pieces[usize::from(fragment.index)];
        match slot {
            Some(old) if *old != fragment.data => return Err(NetError::Fragment),
            Some(_) => return Ok(None),
            None => {}
        }
        assembly.bytes += fragment.data.len();
        if assembly.bytes > MAX_BLOB {
            return Err(NetError::Capacity);
        }
        *slot = Some(fragment.data);
        if assembly.pieces.iter().any(Option::is_none) {
            return Ok(None);
        }
        let assembly = self
            .assemblies
            .remove(&fragment.blob_id)
            .expect("assembly present");
        let data = assembly.pieces.into_iter().flatten().flatten().collect();
        self.completed.push_back(fragment.blob_id);
        if self.completed.len() > 1024 {
            self.completed.pop_front();
        }
        Ok(Some(Blob {
            id: fragment.blob_id,
            queue: assembly.queue,
            data,
        }))
    }

    /// Ordered blobs are released in stamp order; a later stamp waits for the missing one.
    fn deliver_ordered(&mut self, blob: Blob, out: &mut Vec<Blob>) -> Result<(), NetError> {
        let stamp = u32::from(ordering_stamp(blob.id));
        if stamp < self.next_stamp {
            return Ok(());
        }
        if stamp > self.next_stamp {
            if self.held.len() >= MAX_HELD {
                return Err(NetError::Capacity);
            }
            self.held.insert(stamp as u16, blob);
            return Ok(());
        }
        out.push(blob);
        self.next_stamp += 1;
        while let Some(next) = self.held.remove(&(self.next_stamp as u16)) {
            out.push(next);
            self.next_stamp += 1;
        }
        Ok(())
    }

    /// Queues one blob for the server and returns its datagrams.
    pub fn send(
        &mut self,
        queue: u16,
        data: &[u8],
        now: Instant,
    ) -> Result<Vec<Vec<u8>>, NetError> {
        if data.is_empty() || data.len() > MAX_BLOB {
            return Err(NetError::Length);
        }
        let count = data.len().div_ceil(FRAGMENT_DATA);
        if self.pending.len() + count > MAX_PENDING {
            return Err(NetError::Capacity);
        }
        let high = if matches!(queue, 3 | 7) {
            self.ordered_counter = self.ordered_counter.wrapping_add(1);
            0x0100_0000 | u32::from(self.ordered_counter)
        } else {
            self.unordered_counter = self.unordered_counter.wrapping_add(1);
            0x0300_0000 | u32::from(self.unordered_counter)
        };
        self.non_ephemeral = self.non_ephemeral.wrapping_add(1);
        let id = (u64::from(high) << 32) | u64::from(self.non_ephemeral);
        let mut datagrams = Vec::with_capacity(count);
        for (index, part) in data.chunks(FRAGMENT_DATA).enumerate() {
            let fragment = Fragment {
                blob_id: id,
                count: count as u16,
                index: index as u16,
                queue,
                data: part.to_vec(),
            };
            let sequence = self.next_sequence;
            self.next_sequence += 1;
            let key = self.outgoing.next_key();
            let bytes = encode_packet(
                PacketHeader {
                    sequence,
                    recipient: self.recipient,
                    iteration: self.iteration,
                    ..Default::default()
                },
                &[],
                &[fragment],
                Some(key),
            );
            self.pending.insert(
                sequence,
                Pending {
                    bytes: Arc::from(bytes.as_slice()),
                    sent: now,
                    retries: 0,
                },
            );
            datagrams.push(bytes);
        }
        self.last_sent = now;
        Ok(datagrams)
    }

    /// Timed work: retransmit unacknowledged datagrams on a doubling delay, and keep an idle
    /// session alive with an echo request.
    pub fn poll(&mut self, now: Instant) -> Result<Vec<Vec<u8>>, NetError> {
        let mut out = Vec::new();
        for pending in self.pending.values_mut() {
            let delay = Duration::from_millis(500u64 << pending.retries.min(3));
            if now.saturating_duration_since(pending.sent) >= delay {
                if pending.retries >= MAX_RETRIES {
                    return Err(NetError::RetryLimit);
                }
                pending.retries += 1;
                pending.sent = now;
                out.push(pending.bytes.to_vec());
            }
        }
        if !out.is_empty() {
            self.last_sent = now;
        } else if now.saturating_duration_since(self.last_sent) >= KEEPALIVE {
            let body = self.seconds(now).to_le_bytes().to_vec();
            out.push(self.control(&[(flags::ECHO_REQUEST, body)], now));
        }
        Ok(out)
    }

    /// `ClientNet::LogOffServer`: a plain empty logoff header at sequence 0.
    pub fn logoff_datagram(&mut self, now: Instant) -> Vec<u8> {
        self.control(&[(flags::LOGOFF, Vec::new())], now)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Blob, Session};
    use crate::handshake::Connect;
    use crate::header::{flags, PacketHeader};
    use crate::isaac::Isaac;
    use crate::packet::{decode_packet, encode_packet, Fragment};

    fn connect() -> Connect {
        Connect {
            server_time: 0.0,
            cookie: 1,
            net_id: 5,
            outgoing_seed: 0x1111,
            incoming_seed: 0x2222,
        }
    }

    /// A stand-in server: sequenced, ISAAC-keyed fragment datagrams from the connect's outgoing
    /// seed, the same way the server's send path builds them.
    struct Server {
        keys: Isaac,
        sequence: u32,
    }

    impl Server {
        fn new() -> Self {
            Self {
                keys: Isaac::new(connect().outgoing_seed),
                sequence: 1,
            }
        }
        fn datagram(&mut self, fragments: Vec<Fragment>) -> Vec<u8> {
            self.sequence += 1;
            encode_packet(
                PacketHeader {
                    sequence: self.sequence,
                    recipient: 5,
                    iteration: 1,
                    ..Default::default()
                },
                &[],
                &fragments,
                Some(self.keys.next_key()),
            )
        }
    }

    fn blob_fragment(id: u64, index: u16, count: u16, data: &[u8]) -> Fragment {
        Fragment {
            blob_id: id,
            count,
            index,
            queue: 1,
            data: data.to_vec(),
        }
    }

    #[test]
    fn reassembles_orders_by_stamp_and_acknowledges_the_contiguous_sequence() {
        let now = Instant::now();
        let mut session = Session::new(&connect(), 5, 1, now);
        let mut server = Server::new();
        let ordered = |stamp: u64| (0x0100_0000u64 | stamp) << 32 | stamp;
        let first = server.datagram(vec![blob_fragment(ordered(1), 0, 2, b"he")]);
        let later = server.datagram(vec![blob_fragment(ordered(2), 0, 1, b"next")]);
        let rest = server.datagram(vec![blob_fragment(ordered(1), 1, 2, b"llo")]);
        assert!(session.receive(&first, now).unwrap().blobs.is_empty());
        assert!(session.receive(&later, now).unwrap().blobs.is_empty());
        let inbound = session.receive(&rest, now).unwrap();
        let data: Vec<_> = inbound
            .blobs
            .iter()
            .map(|b: &Blob| b.data.clone())
            .collect();
        assert_eq!(data, vec![b"hello".to_vec(), b"next".to_vec()]);
        let ack = decode_packet(inbound.replies.last().unwrap(), None).unwrap();
        assert_eq!(ack.ack, Some(4));
        // A duplicate is acknowledged again but not delivered twice.
        let again = session.receive(&rest, now).unwrap();
        assert!(again.blobs.is_empty());
        assert_eq!(decode_packet(&again.replies[0], None).unwrap().ack, Some(4));
    }

    #[test]
    fn sent_blobs_carry_send_buff_ids_and_retire_on_acknowledgement() {
        let now = Instant::now();
        let mut session = Session::new(&connect(), 5, 1, now);
        let mut server_keys = Isaac::new(connect().incoming_seed);
        let datagrams = session.send(3, &[7; 900], now).unwrap();
        assert_eq!(datagrams.len(), 3);
        for (offset, datagram) in datagrams.iter().enumerate() {
            let packet = decode_packet(datagram, Some(server_keys.next_key())).unwrap();
            assert_eq!(packet.header.sequence, 2 + offset as u32);
            assert_eq!(packet.fragments[0].blob_id, (0x0100_0001u64 << 32) | 1);
            assert_eq!(packet.fragments[0].queue, 3);
        }
        let unordered = session.send(4, b"x", now).unwrap();
        let packet = decode_packet(&unordered[0], Some(server_keys.next_key())).unwrap();
        assert_eq!(packet.fragments[0].blob_id, (0x0300_0001u64 << 32) | 2);
        assert!(session.has_unacknowledged());
        let ack = encode_packet(
            PacketHeader {
                recipient: 5,
                iteration: 1,
                ..Default::default()
            },
            &[(flags::ACK, 5u32.to_le_bytes().to_vec())],
            &[],
            None,
        );
        session.receive(&ack, now).unwrap();
        assert!(!session.has_unacknowledged());
    }

    #[test]
    fn unacknowledged_datagrams_retransmit_and_idle_sessions_echo() {
        let now = Instant::now();
        let mut session = Session::new(&connect(), 5, 1, now);
        let sent = session.send(4, b"x", now).unwrap();
        assert!(session.poll(now).unwrap().is_empty());
        let resent = session.poll(now + Duration::from_millis(600)).unwrap();
        assert_eq!(resent, sent);
        let mut idle = Session::new(&connect(), 5, 1, now);
        let echo = idle.poll(now + Duration::from_secs(6)).unwrap();
        assert!(decode_packet(&echo[0], None)
            .unwrap()
            .echo_request
            .is_some());
    }

    #[test]
    fn wrong_identity_and_far_future_sequences_are_rejected_without_state_change() {
        let now = Instant::now();
        let mut session = Session::new(&connect(), 5, 1, now);
        let mut server = Server::new();
        let mut packet = server.datagram(vec![blob_fragment(1, 0, 1, b"a")]);
        packet[12] = 9;
        assert!(session.receive(&packet, now).is_err());
        let far = encode_packet(
            PacketHeader {
                sequence: 10_000,
                recipient: 5,
                iteration: 1,
                ..Default::default()
            },
            &[],
            &[blob_fragment(1, 0, 1, b"a")],
            Some(1),
        );
        assert!(session.receive(&far, now).is_err());
        let good = Server::new().datagram(vec![blob_fragment(3, 0, 1, b"ok")]);
        assert_eq!(session.receive(&good, now).unwrap().blobs[0].data, b"ok");
    }
}
