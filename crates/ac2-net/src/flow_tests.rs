use super::{standalone, Destination, FlowQueue, FlushContext, PROTO_UI_PRIORITY};
use crate::checksum::checksum32;
use crate::header::{flags, PacketHeader};
use crate::isaac::Isaac;
use crate::optional::{NetErrorId, OptionalHeader};
use crate::packet::split_packet;

fn context(outgoing: &mut Isaac, local_time: f64) -> FlushContext<'_> {
    FlushContext {
        local_time,
        server_time: 100.0 + local_time,
        cur_time: 100.0 + local_time,
        iteration: 1,
        outgoing,
        disconnect_received: None,
        disconnect_sent: false,
    }
}

/// The body checksum a receiver recovers: the header's sum less the header part, XORed with
/// the key when encrypted.
fn body_sum(bytes: &[u8], key: Option<u32>) -> u32 {
    let header = PacketHeader::decode(bytes).unwrap();
    let raw = header.checksum.wrapping_sub(header.header_checksum());
    key.map_or(raw, |key| raw ^ key)
}

#[test]
fn a_blob_and_an_ack_share_one_reliable_packet_and_take_sequence_two() {
    let mut keys = Isaac::new(7);
    let mut expected = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    flow.enqueue_header(OptionalHeader::Ack(5), 0.0);
    flow.enqueue_blob(
        0x0300_0001_0000_0000,
        4,
        vec![1, 2, 3, 4],
        PROTO_UI_PRIORITY,
    );
    let sent = flow.empty(&mut context(&mut keys, 0.0));
    assert_eq!(sent.len(), 1);
    let (to, bytes) = &sent[0];
    assert_eq!(*to, Destination::Peer);
    let header = PacketHeader::decode(bytes).unwrap();
    assert_eq!(header.sequence, 2);
    assert_eq!(
        header.flags,
        flags::ACK | flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM
    );
    assert_eq!(
        (header.recipient, header.interval, header.iteration),
        (9, 1, 1)
    );
    assert_eq!(header.size, 4 + 16 + 4);
    let key = expected.next_key();
    let parsed = split_packet(bytes, body_sum(bytes, Some(key)), true).unwrap();
    assert_eq!(parsed.optional, vec![OptionalHeader::Ack(5)]);
    assert_eq!(parsed.fragments[0].data, [1, 2, 3, 4]);
    // The stored copy lost its unreliable ACK.
    assert_eq!(flow.stored(), 1);
}

#[test]
fn an_ack_alone_carries_the_highest_sequence_sent_and_no_encryption() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    flow.enqueue_header(OptionalHeader::Ack(3), 0.0);
    let sent = flow.empty(&mut context(&mut keys, 0.0));
    let header = PacketHeader::decode(&sent[0].1).unwrap();
    assert_eq!((header.sequence, header.flags), (1, flags::ACK));
    assert_eq!(body_sum(&sent[0].1, None), checksum32(&3u32.to_le_bytes()));
    assert_eq!(flow.stored(), 0);
}

#[test]
fn time_sync_and_echo_wait_for_company_then_ride_with_the_next_ack() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    // Intervals 1 -> 7 cross 1 (mod 6): the time sync and echo request are queued.
    assert!(flow.empty(&mut context(&mut keys, 3.0)).is_empty());
    assert_eq!(flow.interval(), 7);
    assert!(flow.empty(&mut context(&mut keys, 3.0)).is_empty());
    assert_eq!(flow.waiting(), 1);
    flow.enqueue_header(OptionalHeader::Ack(2), 3.25);
    let sent = flow.empty(&mut context(&mut keys, 3.25));
    assert_eq!(sent.len(), 1);
    let header = PacketHeader::decode(&sent[0].1).unwrap();
    assert_eq!(
        header.flags,
        flags::ACK | flags::TIME_SYNC | flags::ECHO_REQUEST | flags::ENCRYPTED_CHECKSUM
    );
    assert_eq!((header.sequence, header.interval), (2, 7));
    let mut expected = Isaac::new(7);
    let parsed = split_packet(
        &sent[0].1,
        body_sum(&sent[0].1, Some(expected.next_key())),
        true,
    )
    .unwrap();
    // Time-sensitive payloads are written when sent.
    assert_eq!(
        parsed.optional,
        vec![
            OptionalHeader::Ack(2),
            OptionalHeader::TimeSync(103.25),
            OptionalHeader::EchoRequest(3.25),
        ]
    );
}

#[test]
fn a_nak_resends_under_the_original_key_and_unknown_ids_are_empty_acked() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    flow.enqueue_blob(0x0300_0001_0000_0000, 4, vec![9; 8], PROTO_UI_PRIORITY);
    let first = flow.empty(&mut context(&mut keys, 0.0));
    flow.enqueue_acks(&[2, 40]);
    let resent = flow.empty(&mut context(&mut keys, 0.1));
    // The resend goes out first, then the empty-ack header alone, unreliable, at the highest
    // sequence sent.
    let header = PacketHeader::decode(&resent[0].1).unwrap();
    assert_eq!(header.sequence, 2);
    assert_eq!(
        header.flags,
        flags::RETRANSMISSION | flags::ENCRYPTED_CHECKSUM | flags::BLOB_FRAGMENTS
    );
    assert_eq!(&resent[0].1[20..], &first[0].1[20..]);
    let mut expected = Isaac::new(7);
    let key = expected.next_key();
    assert!(split_packet(&resent[0].1, body_sum(&resent[0].1, Some(key)), true).is_ok());
    let header = PacketHeader::decode(&resent[1].1).unwrap();
    assert_eq!((header.flags, header.sequence), (flags::EMPTY_ACK, 2));
    let parsed = split_packet(&resent[1].1, body_sum(&resent[1].1, None), true).unwrap();
    assert_eq!(parsed.optional, vec![OptionalHeader::EmptyAck(vec![40])]);
}

#[test]
fn acknowledgement_retires_only_older_sequences() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    for _ in 0..3 {
        flow.enqueue_blob(0x0300_0001_0000_0000, 4, vec![1; 460], PROTO_UI_PRIORITY);
    }
    // Each blob is a full fragment and a 28-byte tail; the tails share one packet.
    assert_eq!(flow.empty(&mut context(&mut keys, 0.0)).len(), 4);
    assert_eq!((flow.stored(), flow.highest_sent()), (4, 5));
    flow.flush_sent(4, false);
    assert_eq!(flow.stored(), 2);
}

#[test]
fn the_cicmd_header_goes_to_the_second_port_every_220_intervals() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    let sent = flow.empty(&mut context(&mut keys, 110.0));
    assert_eq!(flow.interval(), 221);
    let cicmd: Vec<_> = sent
        .iter()
        .filter(|(to, _)| *to == Destination::PeerPortPlusOne)
        .collect();
    assert_eq!(cicmd.len(), 1);
    assert_eq!(
        cicmd[0].1,
        standalone(OptionalHeader::Cicmd { cmd: 1, param: 0 }, 0, 0)
    );
    let header = PacketHeader::decode(&cicmd[0].1).unwrap();
    assert_eq!(
        (
            header.sequence,
            header.flags,
            header.recipient,
            header.iteration
        ),
        (0, flags::CICMD, 0, 0)
    );
}

#[test]
fn reliable_sequences_wrap_past_zero_and_acks_retire_across_the_wrap() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    flow.set_highest_sent(0xFFFF_FFFE);
    // Three whole-fragment blobs: one packet each.
    for _ in 0..3 {
        flow.enqueue_blob(0x0300_0001_0000_0000, 4, vec![1; 0x1C0], PROTO_UI_PRIORITY);
    }
    let sent = flow.empty(&mut context(&mut keys, 0.0));
    let sequences: Vec<u32> = sent
        .iter()
        .map(|(_, b)| PacketHeader::decode(b).unwrap().sequence)
        .collect();
    // `TransmitNewPackets`: 0 is never a sequence.
    assert_eq!(sequences, [0xFFFF_FFFF, 1, 2]);
    // An ACK of 2 retires everything older across the wrap.
    flow.flush_sent(2, false);
    assert_eq!(flow.stored(), 1);
    // A NAK of the pre-wrap sequence finds nothing left: empty-acked.
    flow.enqueue_acks(&[0xFFFF_FFFF]);
    let resent = flow.empty(&mut context(&mut keys, 0.1));
    let header = PacketHeader::decode(&resent[0].1).unwrap();
    assert_eq!((header.flags, header.sequence), (flags::EMPTY_ACK, 2));
}

#[test]
fn a_received_disconnect_is_answered_once_the_queue_drains_and_marks_it_sent() {
    let mut keys = Isaac::new(7);
    let mut flow = FlowQueue::new(9, 0.0);
    flow.enqueue_blob(0x0300_0001_0000_0000, 4, vec![1; 0x1C0], PROTO_UI_PRIORITY);
    flow.enqueue_blob(0x0300_0001_0000_0001, 4, vec![2; 0x1C0], PROTO_UI_PRIORITY);
    let mut ctx = context(&mut keys, 0.0);
    ctx.disconnect_received = Some(0.0);
    // Both blobs went out, so nothing waits: the reply is queued (after `TransmitNewPackets`).
    assert_eq!(flow.empty(&mut ctx).len(), 2);
    assert!(!ctx.disconnect_sent);
    let mut ctx = context(&mut keys, 0.1);
    ctx.disconnect_received = Some(0.1);
    let sent = flow.empty(&mut ctx);
    assert!(ctx.disconnect_sent);
    let header = PacketHeader::decode(&sent[0].1).unwrap();
    // `NetErrorDisconnect` is reliable (`m_Flags` 2): the next sequence and key.
    assert_eq!(
        (header.sequence, header.flags),
        (4, flags::NET_ERROR_DISCONNECT | flags::ENCRYPTED_CHECKSUM)
    );
    let mut expected = Isaac::new(7);
    let key = (0..3).map(|_| expected.next_key()).last().unwrap();
    let parsed = split_packet(&sent[0].1, body_sum(&sent[0].1, Some(key)), true).unwrap();
    assert_eq!(
        parsed.optional,
        vec![OptionalHeader::NetErrorDisconnect(NetErrorId::NONE)]
    );
}
