//! Disconnects: the server closing the connection, a server gone silent, and a client stall.
//! `FlowQueue::Empty` queues the `NetErrorDisconnect` reply once nothing waits to go out (or 10 s
//! passed); `TransmitNewPackets` sending it makes the recipient `cs_DisconnectSent`;
//! `ClientNet::ProcessConnection` follows on the receiver the next frame and removes it the
//! frame after, which tells the game the server died.

use super::*;

/// The client's reply: reliable, the next sequence and key, no reason.
fn disconnect_reply(sequence: u32, interval: u16) -> Vec<u8> {
    let reply = OptionalHeader::NetErrorDisconnect(crate::NetErrorId::NONE).encode();
    datagram(
        client_header(
            sequence,
            flags::NET_ERROR_DISCONNECT | flags::ENCRYPTED_CHECKSUM,
            interval,
        ),
        &[&reply],
        Some(client_key(sequence)),
    )
}

fn interval_of(datagram: &[u8]) -> u16 {
    PacketHeader::decode(datagram).unwrap().interval
}

#[test]
fn a_server_disconnect_is_reported_answered_with_the_next_ack_then_removed() {
    let (mut net, mut server) = logged_on();
    let reason = crate::NetErrorId {
        string_id: 0x0100_0005,
        table_id: 0x0E00_0002,
    };
    // Sequence 3, at the server's interval 3: the client owes a flow header for interval 1.
    let packet = server.reliable(3, OptionalHeader::NetErrorDisconnect(reason));
    let (out, events) = net.use_time(1.0, REAL + 1, vec![(addr(SERVER), packet)]);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(
        events,
        vec![NetEvent::ConnectionError {
            world: false,
            error: crate::NetError::Server {
                string_id: 0x0100_0005,
                table_id: 0x0E00_0002,
            },
        }]
    );
    assert_eq!(
        net.connection_state(SLOT),
        Some(ConnectionState::DisconnectReceived)
    );
    // The flow header waits alone for company, so the reply is not queued yet.
    for t in [1.25, 1.5, 1.75, 2.0] {
        let (out, events) = net.use_time(t, REAL + 2, vec![]);
        assert!(
            out.is_empty() && events.is_empty(),
            "{t}: {out:?} {events:?}"
        );
    }
    // 2.25: the ACK takes the flow header out (sequence 2); the queue is then empty, so the
    // reply is queued and leaves in the same pass (sequence 3).
    let (out, events) = net.use_time(2.25, REAL + 2, vec![]);
    assert!(events.is_empty());
    assert_eq!(out.len(), 2, "{out:?}");
    let flow = OptionalHeader::Flow {
        bytes: 52 + 40,
        interval: 1,
    }
    .encode();
    let ack_flow = datagram(
        client_header(
            2,
            flags::ACK | flags::FLOW | flags::ENCRYPTED_CHECKSUM,
            interval_of(&out[0].1),
        ),
        &[&3u32.to_le_bytes(), &flow],
        Some(client_key(2)),
    );
    assert_eq!(out[0], (addr(SERVER), ack_flow));
    let reply = disconnect_reply(3, interval_of(&out[1].1));
    assert_eq!(out[1], (addr(SERVER), reply));
    // Next frame the receiver follows the recipient to `cs_DisconnectSent`.
    let (out, events) = net.use_time(2.5, REAL + 2, vec![]);
    assert!(out.is_empty() && events.is_empty(), "{out:?} {events:?}");
    assert_eq!(
        net.connection_state(SLOT),
        Some(ConnectionState::DisconnectSent)
    );
    // Then `RemoveConnection`: the logon connection is gone, nothing more on the wire.
    let (out, events) = net.use_time(2.75, REAL + 2, vec![]);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(
        events,
        vec![NetEvent::Disconnected(crate::NetError::ServerDied)]
    );
    assert_eq!(net.connection_state(SLOT), None);
    assert!(net.use_time(5.0, REAL + 5, vec![]).0.is_empty());
}

#[test]
fn a_silent_server_is_given_up_after_140_seconds() {
    let (mut net, _) = logged_on();
    // The last data came at 0.75. One frame a second; at 140.75 exactly 140 s passed, which
    // is not more than 140.
    for step in 1..=140 {
        let t = 0.75 + f64::from(step);
        let (_, events) = net.use_time(t, REAL + step as i64, vec![]);
        assert!(events.is_empty(), "{t}: {events:?}");
        assert_eq!(net.connection_state(SLOT), Some(ConnectionState::Connected));
    }
    // 141.75: `cs_DisconnectReceived` (no referral: the cookie was cleared once connected);
    // nothing waits, so the reply leaves in this frame's flush. It is the first reliable
    // packet: two-interval steps from an even interval never cross `1 (mod 6)`
    // (`ClientFlowQueue::IncrementLocalInterval`), so no time sync went out at this cadence.
    let (out, events) = net.use_time(141.75, REAL + 141, vec![]);
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(
        net.connection_state(SLOT),
        Some(ConnectionState::DisconnectReceived)
    );
    assert_eq!(out.len(), 1, "{out:?}");
    let reply = disconnect_reply(2, interval_of(&out[0].1));
    assert_eq!(out[0], (addr(SERVER), reply));
    // 142.75: the receiver follows to `cs_DisconnectSent`, and (falling through to the
    // connected case) the 2 s ACK still goes, unreliable at the highest sequence sent.
    let (out, events) = net.use_time(142.75, REAL + 142, vec![]);
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(
        net.connection_state(SLOT),
        Some(ConnectionState::DisconnectSent)
    );
    let ack = datagram(
        client_header(2, flags::ACK, interval_of(&out[0].1)),
        // The server's highest sequence: its first blob, 2.
        &[&2u32.to_le_bytes()],
        None,
    );
    assert_eq!(out, vec![(addr(SERVER), ack)]);
    let (out, events) = net.use_time(143.75, REAL + 143, vec![]);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(
        events,
        vec![NetEvent::Disconnected(crate::NetError::ServerDied)]
    );
    assert_eq!(net.connection_state(SLOT), None);
}

#[test]
fn a_client_stall_of_140_seconds_spares_the_connection_for_one_frame() {
    let (mut net, _) = logged_on();
    // `cur_time - lastDidUseTime_ < 140` fails: the client itself was away, so the silence
    // check is skipped this frame.
    let (_, events) = net.use_time(200.0, REAL + 200, vec![]);
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(net.connection_state(SLOT), Some(ConnectionState::Connected));
    // The next frame it runs.
    let (_, events) = net.use_time(200.1, REAL + 200, vec![]);
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(
        net.connection_state(SLOT),
        Some(ConnectionState::DisconnectReceived)
    );
    // The 200.0 time sync and echo request wait alone, so no reply is queued. A second time
    // sync (201.0) cannot join their packet (same mask), so the first packet leaves alone at
    // 201.25 and the second waits; the 2 s ACK (202.0) takes it out, the queue is empty, and
    // the reply follows in the same pass, a local interval later.
    let mut sent = Vec::new();
    let mut t = 200.25;
    while t <= 202.0 {
        let (out, events) = net.use_time(t, REAL + t as i64, vec![]);
        assert!(events.is_empty(), "{t}: {events:?}");
        for (to, d) in out {
            assert_eq!(to, addr(SERVER));
            let h = PacketHeader::decode(&d).unwrap();
            sent.push((t, h.sequence, h.flags, h.interval, d));
        }
        t += 0.25;
    }
    let sync = flags::TIME_SYNC | flags::ECHO_REQUEST | flags::ENCRYPTED_CHECKSUM;
    let summary: Vec<(f64, u32, u32, u16)> = sent.iter().map(|s| (s.0, s.1, s.2, s.3)).collect();
    assert_eq!(
        summary,
        [
            (201.25, 2, sync, 403),
            (202.0, 3, sync | flags::ACK, 404),
            (
                202.0,
                4,
                flags::NET_ERROR_DISCONNECT | flags::ENCRYPTED_CHECKSUM,
                405
            ),
        ]
    );
    assert_eq!(sent[2].4, disconnect_reply(4, 405));
    let (out, events) = net.use_time(202.25, REAL + 202, vec![]);
    assert!(out.is_empty() && events.is_empty(), "{out:?} {events:?}");
    let (out, events) = net.use_time(202.5, REAL + 202, vec![]);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(
        events,
        vec![NetEvent::Disconnected(crate::NetError::ServerDied)]
    );
}

#[test]
fn a_clock_a_minute_ahead_of_the_server_for_a_minute_reports_a_speedhack() {
    let (mut net, mut server) = logged_on();
    // The server's clock is 110 s behind ours: `Timer::set_time` ignores it, and
    // `ClientNet::HandleTimeSynch` notes when the lag began.
    let sync = server.reliable(1, OptionalHeader::TimeSync(40.0));
    let (out, events) = net.use_time(150.0, REAL + 150, vec![(addr(SERVER), sync)]);
    assert_eq!(events, vec![NetEvent::TimeSync(40.0)]);
    assert_eq!(net.timer().base_time(), 0.0);
    assert!(out
        .iter()
        .all(|(_, d)| { PacketHeader::decode(d).unwrap().flags & flags::NET_ERROR == 0 }));
    // Still behind 61 s later at 211: not yet more than a minute since 150.
    let sync = server.reliable(1, OptionalHeader::TimeSync(100.0));
    let (out, _) = net.use_time(210.0, REAL + 210, vec![(addr(SERVER), sync)]);
    assert!(out
        .iter()
        .all(|(_, d)| { PacketHeader::decode(d).unwrap().flags & flags::NET_ERROR == 0 }));
    // At 211 it is: `ID_ConnectionError_RunningSpeedhack` (table 0x40000005) goes alone
    // (`NetError` is unreliable and exclusive), at the highest sequence sent.
    let sync = server.reliable(1, OptionalHeader::TimeSync(101.0));
    let (out, _) = net.use_time(211.0, REAL + 211, vec![(addr(SERVER), sync)]);
    let errors: Vec<&Vec<u8>> = out
        .iter()
        .map(|(_, d)| d)
        .filter(|d| PacketHeader::decode(d).unwrap().flags & flags::NET_ERROR != 0)
        .collect();
    assert_eq!(errors.len(), 1, "{out:?}");
    let header = PacketHeader::decode(errors[0]).unwrap();
    let error = OptionalHeader::NetError(crate::NetErrorId {
        string_id: tokenize(b"ID_ConnectionError_RunningSpeedhack"),
        table_id: 0x4000_0005,
    })
    .encode();
    let expected = datagram(
        client_header(header.sequence, flags::NET_ERROR, header.interval),
        &[&error],
        None,
    );
    assert_eq!(errors[0], &expected);
}

#[test]
fn a_server_net_error_after_logon_logs_off_and_the_server_died() {
    let (mut net, server) = logged_on();
    // `ClientNet::ProcessOptionalHeader`, `NetError` outside the logon: `LogOffServer` (the
    // logoff alone to the connection), then `SendNotice_ServerDied`; the reason is not used.
    let error = OptionalHeader::NetError(crate::NetErrorId {
        string_id: 0x0100_0007,
        table_id: 0x4000_0005,
    });
    let packet = server.plain(1, error);
    let (out, events) = net.use_time(1.0, REAL + 1, vec![(addr(SERVER), packet)]);
    assert_eq!(
        events,
        vec![NetEvent::Disconnected(crate::NetError::ServerDied)]
    );
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
    assert_eq!(out, vec![(addr(SERVER), logoff)]);
    assert!(net.use_time(2.0, REAL + 2, vec![]).0.is_empty());
}
