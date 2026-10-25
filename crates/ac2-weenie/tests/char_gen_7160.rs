//! Character creation and deletion against the Legions server double: the
//! `Proto_UI::SendCharGenResult` blob byte for byte (queue 8, the account, then
//! `gmCharGenResult::Pack`), the server's verification, and a deletion confirmed.

use std::time::Duration;

use ac2_net::protocol::CharGenResult;
use ac2_net::{pack_string, pack_wide_string, Credential};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};
use legions_server_double::{addr, double, words, Spawn, CREATED, SERVER};

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
fn a_character_is_created_and_deleted() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    let result = CharGenResult {
        name: "Birch".into(),
        entity_did: 0x1F00_0023,
        start_area: 1,
        race: 2,
        sex: 1,
        class: 3,
        implement: 4,
        physical: vec![(7, 0.5), (2, 0.75)],
    };
    session.create_character(result.clone());
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    assert!(
        events.iter().any(|e| matches!(e, SessionEvent::CharGenVerification { response: 1, identity, result: 0x11 } if identity.id == CREATED && identity.name == "Birch")),
        "{events:?}"
    );
    session.delete_character(CREATED);
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    assert!(
        events.contains(&SessionEvent::CharacterDeleted(CREATED)),
        "{events:?}"
    );

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    // The server reads the sliders in table order.
    let read = CharGenResult {
        physical: vec![(2, 0.75), (7, 0.5)],
        ..result
    };
    assert_eq!(s.char_gen_results, vec![("local".to_string(), read)]);
    assert_eq!(s.deletions, vec![CREATED]);
    let (_, queue, data) = s
        .blobs
        .iter()
        .find(|(_, _, data)| data[..4] == [0x2A, 0, 1, 0])
        .unwrap();
    assert_eq!(*queue, 8);
    let mut expected = 0x0001_002Au32.to_le_bytes().to_vec();
    pack_string(&mut expected, 0, b"local");
    pack_wide_string(&mut expected, 0, "Birch");
    words(&mut expected, &[0x1F00_0023, 1, 2, 1, 3, 4, 2 | 2 << 24]);
    // Buckets 2 then 7 of 47.
    words(&mut expected, &[2]);
    expected.extend(0.75f32.to_le_bytes());
    words(&mut expected, &[7]);
    expected.extend(0.5f32.to_le_bytes());
    assert_eq!(data, &expected);
}
