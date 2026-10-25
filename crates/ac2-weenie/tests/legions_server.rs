//! The 7160 session against the Legions server double: logon, world entry, a walk and logoff,
//! with the client's datagrams checked byte for byte.

use std::time::Duration;

use ac2_net::protocol::{ClientEvent, ClientPosition};
use ac2_net::{flags, Credential, Isaac, OptionalHeader, PacketHeader};
use ac2_weenie::{
    BodyState, CellPosition, CharacterChoice, OnlineConfig, OnlineSession, SessionEvent,
};
use legions_server_double::{
    addr, datagram, double, fragment_header, words, Spawn, ACK_PORT, CELL, CLIENT_SEED, NET_ID,
    PLAYER, REAL, SERVER,
};

fn config() -> OnlineConfig {
    OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    }
}

#[test]
fn a_7160_session_logs_on_enters_walks_and_logs_off() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    let entry = session.wait_for_world(Duration::from_secs(10)).unwrap();
    assert_eq!((entry.player, entry.instance), (PLAYER, 9));
    assert_eq!(entry.position.cell, CELL);
    let events = session.poll();
    assert!(events.contains(&SessionEvent::LoginFinished), "{events:?}");
    assert!(events.iter().any(|e| matches!(e, SessionEvent::CharGenVerification { response: 1, identity, result: 0x11 } if identity.id == PLAYER && identity.name == "Explorer")));
    session.scene_ready();
    // Walk north for half a second at 64 frames a second, then stand.
    let mut reports = Vec::new();
    for frame in 0..48 {
        let t = frame as f32 / 64.0;
        let walking = frame < 32;
        let body = BodyState {
            position: CellPosition {
                cell: CELL,
                origin: [59.0, 240.0 + 2.0 * t.min(0.5), -44.5],
            },
            heading_degrees: 0.0,
            turn_to_heading: None,
            motion: if walking { [0.0, 1.0, 0.0] } else { [0.0; 3] },
            times_jumped: 0,
            velocity: [0.0; 3],
            actively_moving: walking,
        };
        reports.push((session.cur_time(), body));
        session.report_body(&body);
        session.poll();
        state.borrow_mut().elapsed += 1.0 / 64.0;
    }
    assert!(session.log_off(Duration::from_secs(3)));
    drop(session);
    let state = state.borrow();
    assert!(state.logged_off);

    // The logon: `ClientNet::SendLoginRequest`, alone, sequence 0, the 7160 version.
    let mut payload = vec![4, 0, 0x4B, 0x81, 0xE7, 0xE6, 0, 0]; // "7160", obfuscated
    words(&mut payload, &[28, 1, 0, REAL as u32]);
    payload.extend([5, 0, 0xFE, 0x64, 0xB2, 0xF3, 0xC6, 0]); // "local"
    words(&mut payload, &[0, 0]);
    let logon = datagram(
        PacketHeader {
            flags: flags::LOGON,
            ..Default::default()
        },
        &[&payload],
        None,
    );
    assert_eq!(state.sent[0], (addr(SERVER), logon));
    // The connect-ack to port + 1 echoes the cookie.
    let ack = OptionalHeader::ConnectAck(0x0102_0304_0506_0708).encode();
    let connect_ack = datagram(
        PacketHeader {
            flags: flags::CONNECT_ACK,
            recipient: NET_ID as u16,
            iteration: 1,
            ..Default::default()
        },
        &[&ack],
        None,
    );
    assert_eq!(state.sent[1], (addr(ACK_PORT), connect_ack));

    // The client's blobs, in order: EnterWorld, SceneRenderingComplete, the movement reports,
    // LogOffCharacter.
    let (_, queue, enter) = &state.blobs[0];
    let (_, enter_world) = ClientEvent::EnterWorld {
        character: PLAYER,
        account: "local".into(),
    }
    .encode();
    assert_eq!((*queue, enter), (4, &enter_world));
    // Its datagram, rebuilt: the client's first reliable packet (sequence 2, the first key of
    // its ISAAC stream), the blob alone with its unordered `SendBuff` id.
    let (to, packet) = state
        .sent
        .iter()
        .find(|(_, d)| PacketHeader::decode(d).unwrap().sequence == 2)
        .unwrap();
    let header = PacketHeader::decode(packet).unwrap();
    let rebuilt = datagram(
        PacketHeader {
            sequence: 2,
            flags: flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            recipient: NET_ID as u16,
            interval: header.interval,
            iteration: 1,
            ..Default::default()
        },
        &[
            &fragment_header(0x0300_0001_0000_0000, enter_world.len() as u16 + 16, 4),
            &enter_world,
        ],
        Some(Isaac::new(CLIENT_SEED).next_key()),
    );
    assert_eq!((to, packet), (&addr(SERVER), &rebuilt));
    assert_eq!(state.blobs[1].1, 3);
    assert_eq!(state.blobs[1].2, [0xD1, 0, 1, 0]);
    let positions: Vec<_> = state
        .blobs
        .iter()
        .filter(|(_, _, b)| b[..4] == 0x0001_008Eu32.to_le_bytes())
        .collect();
    // Reports: the first frame; then, while walking, the first frame more than 0.2 s on
    // (`Timer::cur_time` is the time of the frame before, so 13 frames of 1/64 s later: 14, 27);
    // then the stop at once (a motion change).
    let expected: Vec<Vec<u8>> = [0usize, 14, 27, 32]
        .iter()
        .map(|&frame| {
            let (time, body) = reports[frame];
            ClientEvent::Position(ClientPosition {
                time,
                position: body.position,
                heading_degrees: 0.0,
                motion: body.motion,
                cancel_moveto: body.actively_moving,
                instance_stamp: 9,
                teleport_stamp: 5,
                force_position_stamp: 6,
                moveto_stamp: 7,
                jump_velocity: None,
            })
            .encode()
            .1
        })
        .collect();
    let got: Vec<Vec<u8>> = positions.iter().map(|(_, _, b)| b.clone()).collect();
    assert_eq!(got, expected);
    let (_, queue, last) = state.blobs.last().unwrap();
    assert_eq!(
        (*queue, last.clone()),
        ClientEvent::LogOffCharacter { character: PLAYER }.encode()
    );

    // Logoff last: `SendOptionalHeader`, sequence 0, interval 0, our net id.
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
    assert_eq!(state.sent.last().unwrap(), &(addr(SERVER), logoff));
}
