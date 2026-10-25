//! The chat command line against the Legions server double: lines the chat window enters
//! (`CWInterface::ProcessText`) go out as the `WM_Communication` stubs and the chat client
//! pack them, byte for byte; the rooms come from the server's calls.

use std::time::Duration;

use ac2_net::protocol::{HeapObject, PackValue, PackedEntry, PackedReference, ScriptRequest};
use ac2_net::{pack_wide_string, Credential};
use ac2_weenie::chat_handler::{EMOTE, FELLOWSHIP, RETELL, SAY, TELL};
use ac2_weenie::{
    CharacterChoice, ChatCommands, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent,
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

/// The commands as the DATs name them (`chat_commands` resolves them on the loader side).
fn commands() -> ChatCommands {
    let mut c = ChatCommands::new("say");
    c.register("say, s, c", SAY, true);
    c.register("tell, t, send", TELL, true);
    c.register("retell, rt", RETELL, true);
    c.register("emote, em, me, e", EMOTE, true);
    c.register("f, group, g", FELLOWSHIP, true);
    c.add_emote("wave", 0x1D);
    c
}

/// A literal `StringInfo` argument (see `chat_7160`).
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
fn chat_lines_send_what_their_commands_send() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.set_chat_commands(commands());
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    let mut events = Vec::new();
    for line in [
        "Hello there",
        "/t Old Man, psst now",
        "/rt again",
        "/me grins",
        "/Wave",
        "/f hi",
        "/nonsense",
    ] {
        events.extend(session.process_text(line));
    }
    // The fellowship and its room (`WM_Fellowship` 0x2B: a `Fellowship`, room in slot 3).
    let fellowship = PackedReference {
        ids: vec![1],
        entries: vec![PackedEntry::Heap {
            flags: 0x4400_0100,
            object: HeapObject {
                pkg_state_id: 0,
                pkg_id: 0x1F0,
                native: None,
                slots: vec![0, 0, 0, 0x42, 0],
                tags: vec![2; 5],
            },
        }],
    };
    state
        .borrow_mut()
        .send_script_call(0x820C_002B, &[PackValue::Reference(fellowship)]);
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    events.extend(session.process_text("/g hi team"));
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }

    // "/f" before the fellowship: "You are not in a Fellowship."; "/nonsense": unknown.
    let lines: Vec<u32> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. })
                if text.literal.is_none() =>
            {
                Some(text.string_id)
            }
            _ => None,
        })
        .collect();
    assert!(lines.starts_with(&[0x6B9F_0584, 0xA968_48DC]), "{lines:?}");

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    let sent: Vec<&Vec<u8>> = s
        .blobs
        .iter()
        .filter(|(_, queue, data)| *queue == 3 && data[..4] == [0x53, 0, 1, 0])
        .map(|(_, _, data)| data)
        .collect();
    let event = |function: u32, args: Vec<u8>| {
        let mut b = Vec::new();
        words(&mut b, &[0x0001_0053, function, args.len() as u32]);
        b.extend(args);
        b
    };
    let tell = |text: &str, to: &str| {
        let mut b = Vec::new();
        words(&mut b, &[1, 0]);
        b.extend(literal_arg(text));
        b.extend(literal_arg(to));
        event(0x8205_0002, b)
    };
    let mut say = Vec::new();
    words(&mut say, &[1, 0]);
    say.extend(literal_arg("Hello there"));
    let mut emote_text = Vec::new();
    words(&mut emote_text, &[5, 1, 1, 0x4400_0100, 0, 0xFFFF_0076]);
    pack_wide_string(&mut emote_text, 0, "grins");
    words(&mut emote_text, &[0]);
    let mut wave = Vec::new();
    words(&mut wave, &[1, 0x1D]);
    let expected = [
        event(0x8205_0006, say),
        tell("psst now", "Old Man"),
        tell("again", "Old Man"),
        event(0x8205_0004, emote_text),
        event(0x8205_000B, wave),
    ];
    for want in &expected {
        assert!(sent.contains(&want), "{want:02x?} not among {sent:02x?}");
    }
    assert!(s
        .script_requests
        .iter()
        .any(|r| matches!(r, ScriptRequest::EmoteText(t) if t == "grins")));

    // "/g hi team": room 0x42 with the fellowship's chat flag 1, context 1.
    let (_, queue, data) = s
        .blobs
        .iter()
        .find(|(_, _, data)| data[..4] == [0xD3, 0, 1, 0])
        .unwrap();
    assert_eq!(*queue, 4);
    let mut payload = Vec::new();
    words(&mut payload, &[1, 2, 2, 0x42]);
    payload.push(7);
    for unit in "hi team".encode_utf16() {
        payload.extend(unit.to_le_bytes());
    }
    words(&mut payload, &[16, 1, 0, 1, 0]);
    let mut packet = Vec::new();
    words(&mut packet, &[3, 2, 1, 0, 0, 0, 0, payload.len() as u32]);
    packet.extend(payload);
    let mut expected = Vec::new();
    words(&mut expected, &[0x0001_00D3, packet.len() as u32]);
    expected.extend(packet);
    assert_eq!(data, &expected);
}
