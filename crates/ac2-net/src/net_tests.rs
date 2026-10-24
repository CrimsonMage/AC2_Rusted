//! Scripted sessions against a synthetic server: every datagram the client emits is checked
//! field by field and byte for byte against an expectation built from raw segments.

use std::net::SocketAddr;

use super::{tokenize, ClientNet, NetEvent};
use crate::checksum::checksum32;
use crate::handshake::{Credential, Logon};
use crate::header::{flags, PacketHeader};
use crate::isaac::Isaac;
use crate::optional::{Connect, OptionalHeader};
use crate::receiver::ConnectionState;

const SERVER: &str = "127.0.0.1:9000";
const ACK_PORT: &str = "127.0.0.1:9001";
/// The server's recipient slot (`ProtoHeader::recID_` of its sends) and the net id it gives us.
const SLOT: u16 = 3;
const NET_ID: u32 = 0x2A;
const OUT_SEED: u32 = 0x1357_9BDF;
const IN_SEED: u32 = 0x2468_ACE0;
const REAL: i64 = 1_700_000_000;

fn logon() -> Logon {
    Logon {
        version: "7160".into(),
        account: "local".into(),
        credential: Credential::Account,
        connection_sequence: REAL as u32,
    }
}

fn addr(text: &str) -> SocketAddr {
    text.parse().unwrap()
}

/// A datagram from raw checksum segments: header fields, body segments, optional key.
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

fn fragment_header(id: u64, count: u16, size: u16, index: u16, queue: u16) -> [u8; 16] {
    let mut out = [0; 16];
    out[0..8].copy_from_slice(&id.to_le_bytes());
    out[8..10].copy_from_slice(&count.to_le_bytes());
    out[10..12].copy_from_slice(&size.to_le_bytes());
    out[12..14].copy_from_slice(&index.to_le_bytes());
    out[14..16].copy_from_slice(&queue.to_le_bytes());
    out
}

/// The synthetic server: its sequence counter and its outgoing ISAAC generator.
struct Server {
    keys: Isaac,
    sequence: u32,
}

impl Server {
    fn new() -> Self {
        Self {
            keys: Isaac::new(OUT_SEED),
            sequence: 1,
        }
    }

    fn connect(&self, interval: u16) -> Vec<u8> {
        let connect = OptionalHeader::Connect(Connect {
            server_time: 5000.0,
            cookie: 0x0102_0304_0506_0708,
            net_id: NET_ID,
            outgoing_seed: OUT_SEED,
            incoming_seed: IN_SEED,
            padding: 0,
        })
        .encode();
        let header = PacketHeader {
            flags: flags::CONNECT,
            recipient: SLOT,
            interval,
            iteration: 1,
            ..Default::default()
        };
        datagram(header, &[&connect], None)
    }

    /// A reliable packet carrying one single-fragment blob on `queue`.
    fn blob(&mut self, interval: u16, id: u64, queue: u16, data: &[u8]) -> Vec<u8> {
        self.sequence += 1;
        let header = PacketHeader {
            sequence: self.sequence,
            flags: flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            recipient: SLOT,
            interval,
            iteration: 1,
            ..Default::default()
        };
        let fragment = fragment_header(id, 1, data.len() as u16 + 16, 0, queue);
        datagram(header, &[&fragment, data], Some(self.keys.next_key()))
    }

    /// A reliable packet of one header.
    fn reliable(&mut self, interval: u16, header: OptionalHeader) -> Vec<u8> {
        self.sequence += 1;
        let mask = header.mask();
        let bytes = header.encode();
        let packet = PacketHeader {
            sequence: self.sequence,
            flags: mask | flags::ENCRYPTED_CHECKSUM,
            recipient: SLOT,
            interval,
            iteration: 1,
            ..Default::default()
        };
        datagram(packet, &[&bytes], Some(self.keys.next_key()))
    }

    /// An unreliable packet of one header.
    fn plain(&self, interval: u16, header: OptionalHeader) -> Vec<u8> {
        let mask = header.mask();
        let bytes = header.encode();
        let packet = PacketHeader {
            sequence: self.sequence,
            flags: mask,
            recipient: SLOT,
            interval,
            iteration: 1,
            ..Default::default()
        };
        datagram(packet, &[&bytes], None)
    }
}

fn client_header(sequence: u32, bits: u32, interval: u16) -> PacketHeader {
    PacketHeader {
        sequence,
        flags: bits,
        recipient: NET_ID as u16,
        interval,
        iteration: 1,
        ..Default::default()
    }
}

/// Logon at t=0, connect at t=0.25, first data at t=0.75: the client is logged on.
fn logged_on() -> (ClientNet, Server) {
    let mut net = ClientNet::new(addr(SERVER), &logon());
    let server = Server::new();
    let (out, _) = net.use_time(0.0, REAL, vec![]);
    assert_eq!(out, vec![(addr(SERVER), logon().datagram())]);
    let (out, events) = net.use_time(0.25, REAL, vec![(addr(SERVER), server.connect(1))]);
    let ack = OptionalHeader::ConnectAck(0x0102_0304_0506_0708).encode();
    let expected = datagram(
        PacketHeader {
            flags: flags::CONNECT_ACK,
            recipient: NET_ID as u16,
            iteration: 1,
            ..Default::default()
        },
        &[&ack],
        None,
    );
    assert_eq!(out, vec![(addr(ACK_PORT), expected.clone())]);
    assert_eq!(
        events,
        vec![NetEvent::Connected {
            net_id: NET_ID,
            server_time: 5000.0
        }]
    );
    // Not yet a third of a second: no repeat; then the repeat.
    assert!(net.use_time(0.5, REAL, vec![]).0.is_empty());
    assert_eq!(
        net.use_time(0.625, REAL, vec![]).0,
        vec![(addr(ACK_PORT), expected)]
    );
    let mut server = server;
    let blob = server.blob(1, 0x2300_0001_0000_0000, 5, &[0x3B, 0, 1, 0]);
    let (out, events) = net.use_time(0.75, REAL, vec![(addr(SERVER), blob)]);
    assert!(out.is_empty());
    assert_eq!(
        events,
        vec![
            NetEvent::Blob(crate::Blob {
                id: 0x2300_0001_0000_0000,
                queue: 5,
                data: vec![0x3B, 0, 1, 0]
            }),
            NetEvent::LoggedOn,
        ]
    );
    assert_eq!(net.connection_state(SLOT), Some(ConnectionState::Connected));
    (net, server)
}

#[test]
fn logon_repeats_every_two_real_seconds_twenty_times_then_times_out() {
    let mut net = ClientNet::new(addr(SERVER), &logon());
    let mut sent = 0;
    for second in 0..60 {
        let (out, events) = net.use_time(f64::from(second), REAL + i64::from(second), vec![]);
        sent += out.len();
        if !events.is_empty() {
            assert_eq!(
                events,
                vec![NetEvent::Disconnected(crate::NetError::Timeout)]
            );
            assert_eq!((sent, second), (20, 40));
            return;
        }
        assert_eq!(out.len(), usize::from(second % 2 == 0));
    }
    panic!("logon never timed out");
}

#[test]
fn world_entry_handshake_and_first_acknowledgement() {
    let (mut net, _) = logged_on();
    // The ACK timer started at the connect (cur_time 0.25): first ACK at 2.25, unreliable,
    // carrying the highest sequence sent (1) and the local interval.
    assert!(net.use_time(2.0, REAL, vec![]).0.is_empty());
    let (out, _) = net.use_time(2.25, REAL, vec![]);
    // The flow timer (0.25, then every half second) flushed at 0.25, 0.75 and 2.0: the local
    // interval became 2 at 0.75 and 5 at 2.0 (three half-second steps, to 2.25).
    let expected = datagram(
        client_header(1, flags::ACK, 5),
        &[&2u32.to_le_bytes()],
        None,
    );
    assert_eq!(out, vec![(addr(SERVER), expected)]);
}

#[test]
fn time_sync_and_echo_request_ride_with_an_ack_as_the_first_reliable_packet() {
    let (mut net, _) = logged_on();
    let mut out = Vec::new();
    // Frames every quarter second to t=4.25: intervals cross 7 at 3.25.
    let mut t = 1.0;
    while t <= 4.25 {
        out.extend(net.use_time(t, REAL, vec![]).0);
        t += 0.25;
    }
    // The 2.25 ACK went out alone; at 4.25 the next ACK carries the time sync and echo.
    assert_eq!(out.len(), 2);
    let first = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!((first.sequence, first.flags), (1, flags::ACK));
    let second = PacketHeader::decode(&out[1].1).unwrap();
    assert_eq!(second.sequence, 2);
    assert_eq!(
        second.flags,
        flags::ACK | flags::TIME_SYNC | flags::ECHO_REQUEST | flags::ENCRYPTED_CHECKSUM
    );
    let mut keys = Isaac::new(IN_SEED);
    let time_sync = net.timer().server_time_at(4.25);
    let expected = datagram(
        client_header(2, second.flags, second.interval),
        &[
            &2u32.to_le_bytes(),
            &time_sync.to_le_bytes(),
            &(net.timer().local_time() as f32).to_le_bytes(),
        ],
        Some(keys.next_key()),
    );
    assert_eq!(out[1].1, expected);
}

#[test]
fn a_gap_naks_after_point_six_seconds_and_the_resend_completes_it() {
    let (mut net, mut server) = logged_on();
    let lost = server.blob(1, 0x0300_0001_0000_0001, 1, b"lost");
    let later = server.blob(1, 0x0300_0002_0000_0002, 1, b"next");
    // Sequence 4 arrives, 3 is missing. The ACK/NAK timer last fired at the connect (0.25), so
    // the NAK list goes out in the same frame, unreliable, at the highest sequence sent.
    let (out, events) = net.use_time(1.0, REAL, vec![(addr(SERVER), later)]);
    assert_eq!(events.len(), 1);
    let nak = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!((nak.flags, nak.sequence, nak.interval), (flags::NAK, 1, 2));
    let expected = datagram(
        client_header(1, flags::NAK, 2),
        &[&[1u32.to_le_bytes(), 3u32.to_le_bytes()].concat()],
        None,
    );
    assert_eq!(out[0].1, expected);
    // Within 0.6 s nothing more is sent.
    assert!(net.use_time(1.5, REAL, vec![]).0.is_empty());
    let mut resend = lost;
    resend[4] |= flags::RETRANSMISSION as u8;
    // The header checksum covers the flag change; rebuild it.
    let mut header = PacketHeader::decode(&resend).unwrap();
    let body = header.checksum.wrapping_sub(
        PacketHeader {
            flags: header.flags & !flags::RETRANSMISSION,
            ..header
        }
        .header_checksum(),
    );
    header.checksum = body.wrapping_add(header.header_checksum());
    resend[..20].copy_from_slice(&header.encode());
    let (_, events) = net.use_time(1.25, REAL, vec![(addr(SERVER), resend)]);
    assert_eq!(
        events,
        vec![NetEvent::Blob(crate::Blob {
            id: 0x0300_0001_0000_0001,
            queue: 1,
            data: b"lost".to_vec()
        })]
    );
}

#[test]
fn echo_requests_are_answered_with_the_holding_time() {
    let (mut net, mut server) = logged_on();
    // Echo requests are reliable (`m_Flags` 0x18), so they arrive encrypted and sequenced.
    let echo = server.reliable(1, OptionalHeader::EchoRequest(12.5));
    let (out, _) = net.use_time(1.0, REAL, vec![(addr(SERVER), echo)]);
    // Alone, the echo response is a piggyback header and waits for company.
    assert!(out.is_empty());
    let (out, _) = net.use_time(2.25, REAL, vec![]);
    let header = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!(
        header.flags,
        flags::ACK | flags::ECHO_RESPONSE | flags::ENCRYPTED_CHECKSUM
    );
    let mut keys = Isaac::new(IN_SEED);
    let response = [12.5f32.to_le_bytes(), 1.25f32.to_le_bytes()].concat();
    let expected = datagram(
        client_header(2, header.flags, header.interval),
        // The ACK names the echo request's sequence, 3.
        &[&3u32.to_le_bytes(), &response],
        Some(keys.next_key()),
    );
    assert_eq!(out[0].1, expected);
}

#[test]
fn blobs_sent_in_a_frame_leave_in_the_next_with_send_buff_ids() {
    let (mut net, _) = logged_on();
    net.enter_world();
    let first = net.send_buff(4, vec![0x2B, 0, 1, 0]).unwrap();
    let second = net.send_buff(3, vec![0xD1, 0, 1, 0]).unwrap();
    // Non-ephemeral sequence ids start at 0; unordered and ordered stamps from 1.
    assert_eq!(first, 0x0300_0001_0000_0000);
    assert_eq!(second, 0x0100_0001_0000_0001);
    let (out, _) = net.use_time(1.0, REAL, vec![]);
    assert_eq!(out.len(), 1);
    let header = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!(header.sequence, 2);
    let mut keys = Isaac::new(IN_SEED);
    // Equal priorities leave the blob heap in heap order: here first, then second.
    let expected = datagram(
        client_header(
            2,
            flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            header.interval,
        ),
        &[
            &fragment_header(first, 1, 20, 0, 4),
            &[0x2B, 0, 1, 0],
            &fragment_header(second, 1, 20, 0, 3),
            &[0xD1, 0, 1, 0],
        ],
        Some(keys.next_key()),
    );
    assert_eq!(out[0].1, expected);
}

#[test]
fn logoff_goes_alone_to_every_connection_and_silences_the_client() {
    let (mut net, _) = logged_on();
    net.log_off_server();
    let out = net.take_datagrams();
    let expected = datagram(
        PacketHeader {
            flags: flags::LOGOFF,
            recipient: NET_ID as u16,
            iteration: 1,
            ..Default::default()
        },
        &[],
        None,
    );
    assert_eq!(out, vec![(addr(SERVER), expected)]);
    assert!(net.use_time(5.0, REAL, vec![]).0.is_empty());
}

#[test]
fn net_error_tokens_match_tokenize() {
    // Tokenize("A"): 0x41 << 25 = 0x82000000.
    assert_eq!(tokenize(b"A"), 0x8200_0000);
    // Second character shifted right by 7.
    assert_eq!(tokenize(b"AB"), 0x8200_0000 ^ (0x8400_0000 >> 7));
}

#[test]
fn a_new_remote_interval_reports_the_previous_interval_in_a_flow_header() {
    let (mut net, mut server) = logged_on();
    // The first data packet (interval 1, 20 + 16 + 4 bytes) came after the connect (52 bytes):
    // both count toward interval 1. A packet at interval 2 closes it.
    let next = server.blob(2, 0x0300_0002_0000_0002, 1, b"abcd");
    let (out, _) = net.use_time(1.0, REAL, vec![(addr(SERVER), next)]);
    // The flow header is a piggyback header: alone it waits.
    assert!(out.is_empty());
    let (out, _) = net.use_time(2.25, REAL, vec![]);
    let header = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!(
        header.flags,
        flags::ACK | flags::FLOW | flags::ENCRYPTED_CHECKSUM
    );
    let flow = OptionalHeader::Flow {
        bytes: 52 + 40,
        interval: 1,
    }
    .encode();
    let mut keys = Isaac::new(IN_SEED);
    let expected = datagram(
        client_header(2, header.flags, header.interval),
        &[&3u32.to_le_bytes(), &flow],
        Some(keys.next_key()),
    );
    assert_eq!(out[0].1, expected);
}

#[test]
fn duplicates_are_dropped_and_a_server_nak_resends_the_stored_packet() {
    let (mut net, mut server) = logged_on();
    let packet = server.blob(1, 0x0300_0002_0000_0002, 1, b"once");
    let (_, events) = net.use_time(1.0, REAL, vec![(addr(SERVER), packet.clone())]);
    assert_eq!(events.len(), 1);
    let (_, events) = net.use_time(1.25, REAL, vec![(addr(SERVER), packet)]);
    assert!(events.is_empty());
    // The client sends a blob (sequence 2), then the server NAKs it.
    net.send_buff(4, vec![0x2B, 0, 1, 0]).unwrap();
    let (sent, _) = net.use_time(1.5, REAL, vec![]);
    assert_eq!(PacketHeader::decode(&sent[0].1).unwrap().sequence, 2);
    let nak = server.plain(1, OptionalHeader::Nak(vec![2]));
    let (out, _) = net.use_time(1.625, REAL, vec![(addr(SERVER), nak)]);
    let resend = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!(resend.sequence, 2);
    assert_eq!(
        resend.flags,
        flags::RETRANSMISSION | flags::ENCRYPTED_CHECKSUM | flags::BLOB_FRAGMENTS
    );
    assert_eq!(&out[0].1[20..], &sent[0].1[20..]);
    // A NAK for a sequence never sent is answered with an empty ack.
    let nak = server.plain(1, OptionalHeader::Nak(vec![9]));
    let (out, _) = net.use_time(1.75, REAL, vec![(addr(SERVER), nak)]);
    let empty = PacketHeader::decode(&out[0].1).unwrap();
    assert_eq!(empty.flags, flags::EMPTY_ACK);
    let expected = datagram(
        client_header(2, flags::EMPTY_ACK, empty.interval),
        &[&[1u32.to_le_bytes(), 9u32.to_le_bytes()].concat()],
        None,
    );
    assert_eq!(out[0].1, expected);
}
