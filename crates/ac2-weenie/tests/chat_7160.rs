//! Chat against the Legions server double: say, tell and emote go out as the
//! `WM_Communication` stubs pack them, byte for byte, and what comes back is the chat line
//! `CCommunicationSystem` builds for `CM_Communication::SendNotice_DisplayStringInfo`.

use std::time::Duration;

use ac2_net::protocol::{ScriptRequest, StringInfo, StringVariable};
use ac2_net::{pack_wide_string, Credential};
use ac2_weenie::{
    CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent, DISPLAY_SPEECH,
    DISPLAY_TELL,
};
use legions_server_double::{addr, double, words, Spawn, SERVER};

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

fn lines(events: &[SessionEvent]) -> Vec<(u32, StringInfo)> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::DisplayStringInfo { kind, text }) => {
                Some((*kind, text.clone()))
            }
            _ => None,
        })
        .collect()
}

/// A literal `StringInfo` as one loaded native heap object: tag 5, one id, the entry, the
/// object (no package, type 0x6A), `StringInfo::Pack`, no slots.
fn literal_arg(text: &str) -> Vec<u8> {
    let mut b = Vec::new();
    words(
        &mut b,
        &[5, 1, 1, 0x4400_0100, 0, 0xFFFF_006A, 0, 0, 0x0001_0000],
    );
    pack_wide_string(&mut b, 0, text);
    words(&mut b, &[0]);
    b
}

#[test]
fn say_tell_and_emote_round_trip_through_the_double() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    // The session names its tables by DID, as the caller resolved them (a test DID here).
    let mut tables = ac2_weenie::StringTableIds::default();
    tables.insert(ac2_weenie::TABLE_ENUM_COMMUNICATION, 0x2500_7004);
    session.set_string_tables(tables);
    let entry = session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    session.say("Hello there");
    session.tell("Explorer", "psst");
    session.emote(0x1D);
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    let lines = lines(&events);
    assert_eq!(lines.len(), 2, "{events:?}");
    // The player's own speech: "You say" with the text only.
    assert_eq!(lines[0].0, DISPLAY_SPEECH);
    assert_eq!(
        (lines[0].1.table_id, lines[0].1.string_id),
        (0x2500_7004, 0xD85E_9379)
    );
    let StringVariable::Nested(said) = &lines[0].1.variables[0].1 else {
        panic!("{:?}", lines[0].1);
    };
    assert_eq!(said.literal.as_deref(), Some("Hello there"));
    // The tell back from the double (kind 0): the teller's id, name and the text.
    assert_eq!(lines[1].0, DISPLAY_TELL);
    assert_eq!(lines[1].1.string_id, 0x980B_A847);
    let StringVariable::Nested(iid) = &lines[1].1.variables[0].1 else {
        panic!();
    };
    assert_eq!(
        iid.literal.as_deref(),
        Some(ac2_weenie::to_iid(entry.player).as_str())
    );

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    // Queue 3 is ordered by stamp at the receiver; the double records completion order.
    assert!(s
        .script_requests
        .iter()
        .any(|r| matches!(r, ScriptRequest::Emote(0x1D))));
    let sent = |function: u32| {
        s.blobs
            .iter()
            .find(|(_, queue, data)| {
                *queue == 3 && data[..4] == [0x53, 0, 1, 0] && data[4..8] == function.to_le_bytes()
            })
            .map(|(_, _, data)| data.clone())
            .unwrap()
    };
    let event = |function: u32, args: Vec<u8>| {
        let mut b = Vec::new();
        words(&mut b, &[0x0001_0053, function, args.len() as u32]);
        b.extend(args);
        b
    };
    let mut say = Vec::new();
    words(&mut say, &[1, 0]);
    say.extend(literal_arg("Hello there"));
    assert_eq!(sent(0x8205_0006), event(0x8205_0006, say));
    let mut tell = Vec::new();
    words(&mut tell, &[1, 0]);
    tell.extend(literal_arg("psst"));
    tell.extend(literal_arg("Explorer"));
    assert_eq!(sent(0x8205_0002), event(0x8205_0002, tell));
    let mut emote = Vec::new();
    words(&mut emote, &[1, 0x1D]);
    assert_eq!(sent(0x8205_000B), event(0x8205_000B, emote));
}

#[test]
fn room_chat_goes_through_the_chat_client_and_comes_back() {
    use ac2_net::protocol::GunChatExtraInfo;
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    assert_eq!(session.send_to_room(0x10, "", 0), None);
    assert_eq!(session.send_to_room(0, "Hi all", 0), None);
    assert_eq!(session.send_to_room(0x10, "Hi all", 4), Some(1));
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    let notices: Vec<&GameplayNotice> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(
        notices,
        [
            &GameplayNotice::SendToRoomResult {
                context: 1,
                result: 0
            },
            &GameplayNotice::HearFromRoom {
                room: 0x10,
                speaker: "Explorer".into(),
                text: "Hi all".into(),
                extra: Some(GunChatExtraInfo::new(4, String::new())),
            },
        ]
    );
    // Byte for byte: `CChatEvent` around `IChatClientImpl::SendNetPacket`'s packet: the
    // header (binary request, method 2, target type 1), then the payload: context 1, method 2
    // twice, the room, the text (compressed length, UTF-16), the `GunChatExtraInfo` blob.
    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    let (_, queue, data) = s
        .blobs
        .iter()
        .find(|(_, _, data)| data[..4] == [0xD3, 0, 1, 0])
        .unwrap();
    assert_eq!(*queue, 4);
    let mut payload = Vec::new();
    words(&mut payload, &[1, 2, 2, 0x10]);
    payload.push(6);
    for unit in "Hi all".encode_utf16() {
        payload.extend(unit.to_le_bytes());
    }
    words(&mut payload, &[16, 1, 0, 4, 0]);
    let mut packet = Vec::new();
    words(&mut packet, &[3, 2, 1, 0, 0, 0, 0, payload.len() as u32]);
    packet.extend(payload);
    let mut expected = Vec::new();
    words(&mut expected, &[0x0001_00D3, packet.len() as u32]);
    expected.extend(packet);
    assert_eq!(data, &expected);
}
