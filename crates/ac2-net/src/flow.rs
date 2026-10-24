//! `FlowQueue` / `ClientFlowQueue`: everything one recipient (a server connection) is sent.
//!
//! Optional headers wait in a min-heap keyed by mask and blobs in a min-heap keyed by priority;
//! `CoalesceData` packs them into waiting packets (headers first, each into the first packet it
//! fits, then each blob's fragments), and `TransmitNewPackets` sends the waiting packets in
//! order. A reliable packet takes the next sequence and ISAAC key and is stored, without its
//! unreliable headers, until the peer acknowledges past it; resends happen only when the peer
//! NAKs (`TransmitAcks`). The client's flow control (`CanSend`) always allows sending.

use std::collections::{BTreeSet, VecDeque};

use crate::blob::{fragmentize, sequence_newer};
use crate::header::{flags, PacketHeader};
use crate::isaac::Isaac;
use crate::optional::{NetErrorId, OptionalHeader, EXCLUSIVE, MAX_SEQUENCE_LIST, SINGLE_TRY};
use crate::packet::{OutHeader, OutPacket, MAX_PACKET_BODY, PACKET_ENCRYPTED, PACKET_RELIABLE};

/// Seconds per local interval (`FlowQueue::Empty`).
pub(crate) const INTERVAL_SECONDS: f64 = 0.5;
/// Intervals between time sync and echo requests (`ClientFlowQueue::IncrementLocalInterval`).
const TIME_SYNC_INTERVALS: u32 = 6;
/// Intervals between CICMD headers to the server's second port.
const CICMD_INTERVALS: u32 = 0xDC;
/// Blob priority `NetBlob::Send` uses for `Proto_UI` sends.
pub(crate) const PROTO_UI_PRIORITY: u32 = 5;

/// Where a datagram goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    /// The address the connection came from.
    Peer,
    /// The same host, port plus one (`SendConnectAck`, the CICMD header).
    PeerPortPlusOne,
}

/// The connection facts a flush needs from its `ReceiverData` and the clock.
pub(crate) struct FlushContext<'a> {
    pub local_time: f64,
    /// `Timer::get_elapsed_time() + Timer::base_time`, for time-sensitive payloads.
    pub server_time: f64,
    pub cur_time: f64,
    pub iteration: u16,
    pub outgoing: &'a mut Isaac,
    /// `cs_DisconnectReceived`, with the seconds since the state changed.
    pub disconnect_received: Option<f64>,
}

#[derive(Debug)]
struct OutBlob {
    id: u64,
    queue: u16,
    data: Vec<u8>,
}

/// `SentPacketStore`: reliable packets kept for resending, in send order.
#[derive(Debug, Default)]
pub(crate) struct SentPacketStore {
    packets: VecDeque<OutPacket>,
}

impl SentPacketStore {
    fn contains(&self, seq: u32) -> bool {
        self.packets.iter().any(|p| p.sequence == seq)
    }

    /// `SentPacketStore::Flush`: drop packets older than `highest` (or all).
    fn flush(&mut self, highest: u32, everything: bool) {
        while let Some(front) = self.packets.front() {
            if !everything && !sequence_newer(highest, front.sequence) {
                return;
            }
            self.packets.pop_front();
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.packets.len()
    }
}

/// `ClientFlowQueue`.
#[derive(Debug)]
pub(crate) struct FlowQueue {
    /// `ReceiverData::m_NetID`: the `recID_` this client sends.
    net_id: u16,
    /// `CurLocalInterval_.intervalID_`, from 1.
    interval: u16,
    interval_time: f64,
    highest_sent: u32,
    waiting: VecDeque<OutPacket>,
    special: crate::pqueue::PQueueArray<u32, OutHeader>,
    blobs: crate::pqueue::PQueueArray<u32, OutBlob>,
    /// `m_ackList`: NAKed sequences to resend, ascending.
    ack_list: Vec<u32>,
    /// `m_emptyAckList` and `empties_`: NAKed sequences there is nothing to resend for.
    empty_acks: VecDeque<u32>,
    empty_set: BTreeSet<u32>,
    /// `emptyAcks_[0]`: the count compiled by the previous `CompileEmptyAcks`.
    compiled_empty: usize,
    sent: SentPacketStore,
    pub packets_sent: u32,
    /// Headers and blobs ever queued, so a caller can tell a flush queued more.
    enqueued: u64,
}

impl FlowQueue {
    /// `FlowQueue::FlowQueue` and `ClientFlowQueue::ClientFlowQueue`.
    pub fn new(net_id: u16, local_time: f64) -> Self {
        Self {
            net_id,
            interval: 1,
            interval_time: local_time,
            highest_sent: 1,
            waiting: VecDeque::new(),
            special: crate::pqueue::PQueueArray::new(),
            blobs: crate::pqueue::PQueueArray::new(),
            ack_list: Vec::new(),
            empty_acks: VecDeque::new(),
            empty_set: BTreeSet::new(),
            compiled_empty: 0,
            sent: SentPacketStore::default(),
            packets_sent: 0,
            enqueued: 0,
        }
    }

    #[cfg(test)]
    pub fn interval(&self) -> u16 {
        self.interval
    }

    #[cfg(test)]
    pub fn highest_sent(&self) -> u32 {
        self.highest_sent
    }

    #[cfg(test)]
    pub fn stored(&self) -> usize {
        self.sent.len()
    }

    #[cfg(test)]
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }

    pub fn enqueued(&self) -> u64 {
        self.enqueued
    }

    /// `FlowQueue::EnqueueOptionalHeader`.
    pub fn enqueue_header(&mut self, header: OptionalHeader, local_time: f64) {
        let header = OutHeader::new(header, local_time);
        self.special.insert(header.mask(), header);
        self.enqueued += 1;
    }

    /// `ClientFlowQueue::Enqueue`.
    pub fn enqueue_blob(&mut self, id: u64, queue: u16, data: Vec<u8>, priority: u32) {
        self.blobs.insert(priority, OutBlob { id, queue, data });
        self.enqueued += 1;
    }

    /// `FlowQueue::EnqueueEmptyAck`.
    fn enqueue_empty_ack(&mut self, seq: u32) {
        if self.empty_set.insert(seq) {
            self.empty_acks.push_back(seq);
        }
    }

    /// `FlowQueue::EnqueueAcks`: merge a NAK list (ascending) into the resend list. Entries the
    /// list passes over were received and leave; ids no longer stored are answered empty.
    pub fn enqueue_acks(&mut self, ids: &[u32]) {
        let mut cur = 0;
        let mut i = 0;
        while i < ids.len() && cur < self.ack_list.len() {
            let seq = ids[i];
            let existing = self.ack_list[cur];
            if seq == existing {
                cur += 1;
                i += 1;
            } else if sequence_newer(existing, seq) {
                if self.sent.contains(seq) {
                    self.ack_list.insert(cur, seq);
                    cur += 1;
                } else {
                    self.enqueue_empty_ack(seq);
                }
                i += 1;
            } else {
                self.ack_list.remove(cur);
            }
        }
        if cur >= self.ack_list.len() {
            for &seq in &ids[i..] {
                if self.sent.contains(seq) {
                    self.ack_list.push(seq);
                } else {
                    self.enqueue_empty_ack(seq);
                }
            }
        }
    }

    /// `FlowQueue::FlushSentPackets`.
    pub fn flush_sent(&mut self, id: u32, everything: bool) {
        while let Some(&front) = self.ack_list.first() {
            if !everything && (front == id || !sequence_newer(id, front)) {
                break;
            }
            self.ack_list.remove(0);
        }
        self.sent.flush(id, everything);
    }

    /// `FlowQueue::Empty`: empty acks, resends, new packets, the disconnect reply, then the
    /// local interval.
    pub fn empty(&mut self, ctx: &mut FlushContext<'_>) -> Vec<(Destination, Vec<u8>)> {
        let mut out = Vec::new();
        self.transmit_naks(ctx.local_time);
        self.transmit_acks(ctx, &mut out);
        self.transmit_new_packets(ctx, &mut out);
        if let Some(duration) = ctx.disconnect_received {
            if self.waiting.is_empty() || 10.0 < duration {
                self.enqueue_header(
                    OptionalHeader::NetErrorDisconnect(NetErrorId::NONE),
                    ctx.local_time,
                );
            }
        }
        let mut steps = 0u32;
        if self.interval_time < ctx.local_time {
            let mut t = self.interval_time;
            loop {
                t += INTERVAL_SECONDS;
                steps += 1;
                if t >= ctx.local_time {
                    break;
                }
            }
            self.interval_time = t;
            if steps != 0 {
                self.increment_local_interval(steps, ctx, &mut out);
            }
        }
        out
    }

    /// `FlowQueue::CompileEmptyAcks` and `FlowQueue::TransmitNaks`: up to `0x72` empty acks per
    /// header; after a header is built the next call compiles nothing.
    fn transmit_naks(&mut self, local_time: f64) {
        let n = self.empty_acks.len();
        if n == 0 || self.compiled_empty != 0 {
            self.compiled_empty = 0;
            return;
        }
        let n = n.min(MAX_SEQUENCE_LIST);
        let ids: Vec<u32> = (0..n)
            .filter_map(|_| {
                let seq = self.empty_acks.pop_front()?;
                self.empty_set.remove(&seq);
                Some(seq)
            })
            .collect();
        self.compiled_empty = ids.len();
        self.enqueue_header(OptionalHeader::EmptyAck(ids), local_time);
    }

    /// `FlowQueue::DequeueAck` and `FlowQueue::TransmitAcks`: resend each NAKed sequence with
    /// `RETRANSMISSION | ENCRYPTED_CHECKSUM`, under its original key.
    fn transmit_acks(&mut self, ctx: &mut FlushContext<'_>, out: &mut Vec<(Destination, Vec<u8>)>) {
        while !self.ack_list.is_empty() {
            let seq = self.ack_list.remove(0);
            let Some(packet) = self.sent.packets.iter_mut().find(|p| p.sequence == seq) else {
                continue;
            };
            if packet.size != 0 {
                packet.update_time_sensitive(ctx.local_time, ctx.server_time);
                if packet.checksum == 0 {
                    packet.compute_checksum();
                }
                if packet.flags & PACKET_ENCRYPTED == 0 && packet.flags & PACKET_RELIABLE != 0 {
                    packet.checksum ^= packet.crypto_key;
                    packet.flags |= PACKET_ENCRYPTED;
                }
            }
            if packet.fragments.is_empty() && packet.headers.is_empty() {
                let seq = packet.sequence;
                self.enqueue_empty_ack(seq);
                continue;
            }
            let mut bits = flags::RETRANSMISSION | flags::ENCRYPTED_CHECKSUM;
            if !packet.fragments.is_empty() {
                bits |= flags::BLOB_FRAGMENTS;
            }
            let mut header = PacketHeader {
                flags: bits | packet.header_masks(),
                recipient: self.net_id,
                interval: self.interval,
                iteration: ctx.iteration,
                ..Default::default()
            };
            header.sequence = packet.sequence;
            header.checksum = packet.checksum;
            header.size = packet.size as u16;
            out.push((Destination::Peer, packet.datagram(header)));
            self.packets_sent += 1;
        }
    }

    /// `NetPacket::AddToHead` (header packets) or `AddToTail` (fragment packets).
    fn enqueue_packet(&mut self, packet: OutPacket) {
        if packet.headers.is_empty() {
            self.waiting.push_back(packet);
        } else if self
            .waiting
            .front()
            .is_some_and(|head| head.flags & PACKET_ENCRYPTED != 0)
        {
            self.waiting.insert(1, packet);
        } else {
            self.waiting.push_front(packet);
        }
    }

    /// `FlowQueue::CoalesceData`.
    fn coalesce(&mut self) {
        while let Some((_, header)) = self.special.remove_min() {
            let retail = header.retail_flags();
            let mut header = Some(header);
            if retail & EXCLUSIVE == 0 {
                for packet in self.waiting.iter_mut() {
                    let masks = packet.header_masks();
                    let h = header.as_ref().expect("unplaced header");
                    if packet.flags & PACKET_ENCRYPTED == 0
                        && packet.size + h.bytes.len() <= MAX_PACKET_BODY
                        && h.mask() & masks == 0
                        && !packet.closed()
                    {
                        packet.add_header(header.take().expect("unplaced header"));
                        break;
                    }
                    if masks == 0 || retail & SINGLE_TRY != 0 {
                        break;
                    }
                }
            }
            if let Some(header) = header {
                let mut packet = OutPacket::new();
                packet.add_header(header);
                self.enqueue_packet(packet);
            }
            if retail & SINGLE_TRY != 0 {
                break;
            }
        }
        while let Some((_, blob)) = self.blobs.remove_min() {
            for fragment in fragmentize(blob.id, blob.queue, &blob.data) {
                let fits = self.waiting.iter_mut().find(|packet| {
                    packet.flags & PACKET_ENCRYPTED == 0
                        && fragment.size() + packet.size <= MAX_PACKET_BODY
                        && !packet.closed()
                });
                match fits {
                    Some(packet) => packet.add_fragment(fragment),
                    None => {
                        let mut packet = OutPacket::new();
                        packet.add_fragment(fragment);
                        self.enqueue_packet(packet);
                    }
                }
            }
        }
    }

    /// `FlowQueue::TransmitNewPackets`.
    fn transmit_new_packets(
        &mut self,
        ctx: &mut FlushContext<'_>,
        out: &mut Vec<(Destination, Vec<u8>)>,
    ) {
        self.coalesce();
        while let Some(packet) = self.waiting.front_mut() {
            let old_flags = packet.flags;
            let mut bits = if packet.fragments.is_empty() {
                0
            } else {
                flags::BLOB_FRAGMENTS
            };
            bits |= packet.header_masks();
            // A packet of only piggyback headers (time sync, echoes, flow) waits for company.
            if bits & 0x00FF_FFFF == 0 && self.waiting.len() == 1 {
                return;
            }
            let packet = self.waiting.front_mut().expect("front packet");
            packet.update_time_sensitive(ctx.local_time, ctx.server_time);
            if packet.checksum == 0 {
                packet.compute_checksum();
            }
            if packet.flags & PACKET_RELIABLE == 0 {
                packet.sequence = self.highest_sent;
            } else {
                if packet.flags & PACKET_ENCRYPTED == 0 {
                    let key = if packet.sequence == 0 || old_flags & PACKET_ENCRYPTED == 0 {
                        self.highest_sent = self.highest_sent.wrapping_add(1);
                        if self.highest_sent == 0 {
                            self.highest_sent = 1;
                        }
                        packet.sequence = self.highest_sent;
                        ctx.outgoing.next_key()
                    } else {
                        packet.crypto_key
                    };
                    packet.checksum ^= key;
                    packet.flags |= PACKET_ENCRYPTED;
                    packet.crypto_key = key;
                }
                bits |= flags::ENCRYPTED_CHECKSUM;
            }
            let mut header = PacketHeader {
                flags: bits,
                recipient: self.net_id,
                interval: self.interval,
                iteration: ctx.iteration,
                ..Default::default()
            };
            header.sequence = packet.sequence;
            header.checksum = packet.checksum;
            header.size = packet.size as u16;
            out.push((Destination::Peer, packet.datagram(header)));
            self.packets_sent += 1;
            let mut sent = self.waiting.pop_front().expect("front packet");
            if sent.flags & PACKET_RELIABLE != 0 {
                sent.remove_disposable();
                self.sent.packets.push_back(sent);
            }
        }
    }

    /// `ClientFlowQueue::IncrementLocalInterval`: a time sync and echo request each time the
    /// interval crosses `1 (mod 6)`; a CICMD header to port plus one each time it crosses
    /// `1 (mod 220)`.
    fn increment_local_interval(
        &mut self,
        steps: u32,
        ctx: &FlushContext<'_>,
        out: &mut Vec<(Destination, Vec<u8>)>,
    ) {
        let crosses = |old: u32, period: u32| {
            let next = (old + steps) % period;
            next == 1 || steps > period - 1 || (next != 0 && next < old % period)
        };
        let old = u32::from(self.interval);
        if crosses(old, TIME_SYNC_INTERVALS) {
            self.enqueue_header(OptionalHeader::TimeSync(ctx.cur_time), ctx.local_time);
            self.enqueue_header(
                OptionalHeader::EchoRequest(ctx.local_time as f32),
                ctx.local_time,
            );
        }
        if crosses(old, CICMD_INTERVALS) {
            out.push((
                Destination::PeerPortPlusOne,
                standalone(OptionalHeader::Cicmd { cmd: 1, param: 0 }, 0, 0),
            ));
        }
        self.interval = self.interval.wrapping_add(steps as u16);
    }
}

/// `SharedNet::SendOptionalHeader`: one header alone at sequence 0 and interval 0, under the
/// receiver's net id and iteration (or zeros when there is no receiver).
pub(crate) fn standalone(header: OptionalHeader, recipient: u16, iteration: u16) -> Vec<u8> {
    let mut packet = OutPacket::new();
    let mask = header.mask();
    packet.add_header(OutHeader::new(header, 0.0));
    packet.datagram(PacketHeader {
        sequence: 0,
        flags: mask,
        checksum: packet.checksum,
        recipient,
        interval: 0,
        size: packet.size as u16,
        iteration,
    })
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod tests;
