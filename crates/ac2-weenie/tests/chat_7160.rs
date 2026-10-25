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
        (0x4100_0004, 0xD85E_9379)
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
