//! The character selection screen's session against the Legions server double: it waits at
//! the character set, creates a character there, and enters the world only when asked.

use std::time::Duration;

use ac2_net::protocol::{CharGenResult, ClientEvent};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};
use legions_server_double::{addr, double, Spawn, CREATED, PLAYER, SERVER};

fn config() -> OnlineConfig {
    OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::Wait,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    }
}

fn settle(
    session: &mut OnlineSession,
    state: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    events
}

#[test]
fn the_session_waits_at_the_character_set_until_asked_to_enter() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    let set = session
        .wait_for_characters(Duration::from_secs(10))
        .unwrap();
    assert_eq!(set.characters[0].id, PLAYER);
    assert_eq!(session.characters(), Some(&set));
    // Nothing enters by itself.
    let events = settle(&mut session, &state);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SessionEvent::Entering(_))),
        "{events:?}"
    );
    assert!(state
        .borrow()
        .blobs
        .iter()
        .all(|(_, _, b)| b[..4] != 0x0001_002Bu32.to_le_bytes()));

    // A character is created at the selection screen.
    session.create_character(CharGenResult {
        name: "Birch".into(),
        entity_did: 0x1F00_0023,
        start_area: 1,
        race: 2,
        sex: 1,
        class: 3,
        implement: 4,
        physical: vec![(7, 0.5)],
    });
    let events = settle(&mut session, &state);
    assert!(events.contains(&SessionEvent::CharGenVerification {
        response: 1,
        character: CREATED,
        name: "Birch".into(),
        result: 0x11,
    }));

    // Only a listed character enters; then the world entry runs as usual.
    assert!(session.enter_world(0x1234).is_err());
    session.enter_world(PLAYER).unwrap();
    let entry = session.wait_for_world(Duration::from_secs(10)).unwrap();
    assert_eq!(entry.player, PLAYER);
    assert!(session.poll().contains(&SessionEvent::Entering(PLAYER)));
    assert!(session.enter_world(PLAYER).is_err());

    // The `Proto_UI::SendEnterWorld` blob, byte for byte, on queue 4.
    let s = state.borrow();
    let (_, enter) = ClientEvent::EnterWorld {
        character: PLAYER,
        account: "local".into(),
    }
    .encode();
    let sent: Vec<_> = s
        .blobs
        .iter()
        .filter(|(_, _, b)| b[..4] == 0x0001_002Bu32.to_le_bytes())
        .collect();
    assert_eq!(sent.len(), 1);
    assert_eq!((sent[0].1, &sent[0].2), (4, &enter));
}
