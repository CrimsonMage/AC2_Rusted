//! Disconnect and recovery over the lossy network: the server closing the connection
//! mid-stream, the server going silent, and the client logging off mid-stream. Every datagram
//! the client sent is checked by the oracle.

mod harness;
mod oracle;

use ac2_net::{flags, NetError, NetErrorId, NetEvent, OptionalHeader, PacketHeader};
use harness::{logoff_datagram, lossy, payload, Session};
use legions_server_double::{addr, SERVER};

/// Five seconds of blobs both ways.
fn traffic(session: &mut Session) {
    for n in 0..320u32 {
        if n % 2 == 0 {
            session.state.borrow_mut().send_event(&payload(n, 24));
        }
        if n % 3 == 0 {
            session.client.send(4, payload(n, 16)).expect("queued");
        }
        let events = session.frame();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, NetEvent::Disconnected(_))),
            "{events:?}"
        );
    }
}

/// The events after logon that are not blobs or time syncs.
fn notices(session: &Session) -> Vec<(f64, NetEvent)> {
    session
        .events
        .iter()
        .filter(|(_, e)| {
            !matches!(
                e,
                NetEvent::Blob(_)
                    | NetEvent::TimeSync(_)
                    | NetEvent::Connected { .. }
                    | NetEvent::LoggedOn
            )
        })
        .cloned()
        .collect()
}

/// Frames until the client reports the session over, at most `limit` seconds.
fn until_disconnected(session: &mut Session, limit: f64) -> f64 {
    let end = session.now() + limit;
    while session.now() < end {
        let now = session.now();
        if session
            .frame()
            .iter()
            .any(|e| matches!(e, NetEvent::Disconnected(_)))
        {
            return now;
        }
    }
    panic!("still connected after {limit} s");
}

#[test]
fn a_server_disconnect_mid_stream_is_reported_answered_and_ends_the_session() {
    for seed in [3, 17, 0xD15C] {
        let mut session = Session::new(seed, lossy());
        session.log_on();
        traffic(&mut session);
        let reason = NetErrorId {
            string_id: 0x0100_0005,
            table_id: 0x0E00_0002,
        };
        session
            .state
            .borrow_mut()
            .reliable_header(OptionalHeader::NetErrorDisconnect(reason));
        let ended = until_disconnected(&mut session, 30.0);
        // `Net_LoginConnectionError` with the reason, once, then the server died
        // (`RemoveConnection` -> `ECM_Login::SendNotice_ServerDied`); nothing after.
        let notices = notices(&session);
        let kinds: Vec<&NetEvent> = notices.iter().map(|(_, e)| e).collect();
        assert_eq!(
            kinds,
            [
                &NetEvent::ConnectionError {
                    world: false,
                    error: NetError::Server {
                        string_id: 0x0100_0005,
                        table_id: 0x0E00_0002,
                    },
                },
                &NetEvent::Disconnected(NetError::ServerDied),
            ],
            "{seed}"
        );
        assert_eq!(notices[1].0, ended);
        let frames = session.frames();
        oracle::check(&frames);
        // One reply, first sent no earlier than the reason arrived.
        let replies: Vec<f64> = frames
            .iter()
            .flat_map(|f| f.outbound.iter().map(move |(_, d)| (f.time, d)))
            .filter_map(|(t, d)| {
                let h = PacketHeader::decode(d).unwrap();
                (h.flags & flags::NET_ERROR_DISCONNECT != 0 && h.flags & flags::RETRANSMISSION == 0)
                    .then_some(t)
            })
            .collect();
        assert_eq!(replies.len(), 1, "{seed}: {replies:?}");
        assert!(notices[0].0 <= replies[0] && replies[0] < ended, "{seed}");
        // Silence once the session ended.
        let count = frames.len();
        for _ in 0..64 {
            assert!(session.frame().is_empty());
        }
        assert_eq!(session.frames().len(), count);
    }
}

#[test]
fn a_server_gone_silent_mid_stream_is_given_up_140_seconds_after_its_last_data() {
    let mut session = Session::new(5, lossy());
    session.log_on();
    traffic(&mut session);
    session.state.borrow_mut().silent = true;
    let ended = until_disconnected(&mut session, 200.0);
    let frames = session.frames();
    oracle::check(&frames);
    let last_data = frames
        .iter()
        .rev()
        .find(|f| !f.inbound.is_empty())
        .unwrap()
        .time;
    // `ClientNet::ProcessConnection`: more than 140 s without data, then the disconnect reply
    // goes with the next ACK or NAK (at most 2 s), then two frames to removal.
    let silence = ended - last_data;
    assert!((140.0 - 0.5..143.0).contains(&silence), "{silence}");
    let notices = notices(&session);
    assert_eq!(
        notices,
        [(ended, NetEvent::Disconnected(NetError::ServerDied))]
    );
}

#[test]
fn a_logoff_mid_stream_goes_once_and_the_client_falls_silent() {
    let mut session = Session::new(11, lossy());
    session.log_on();
    traffic(&mut session);
    // Blobs still queued are not sent: `ClientNet::SendBuff` sends nothing after the logoff.
    session.client.send(4, payload(9999, 16)).expect("queued");
    session.client.disconnect();
    let frames = session.frames();
    oracle::check(&frames);
    let last = frames.last().unwrap();
    assert_eq!(
        last.outbound.last(),
        Some(&(addr(SERVER), logoff_datagram()))
    );
    let logoffs = frames
        .iter()
        .flat_map(|f| f.outbound.iter())
        .filter(|(_, d)| PacketHeader::decode(d).unwrap().flags & flags::LOGOFF != 0)
        .count();
    assert_eq!(logoffs, 1);
    for _ in 0..64 {
        assert!(session.frame().is_empty());
    }
    assert_eq!(session.frames().len(), frames.len());
}
