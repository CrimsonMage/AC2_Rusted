//! The logout (`gmGamePlayUI::EndSession` -> `Proto_UI::LogOffCharacter`) against the Legions
//! 7160 server double: the player's character goes out on queue 4, the server answers
//! `CharExitGame` (`SessionEvent::LeftWorld`), the connection stays up, and the character set the
//! session kept is still there for character selection.

mod common;

use ac2_client::FrameInput;
use ac2_net::protocol::ClientEvent;
use ac2_weenie::SessionEvent;
use common::*;
use legions_server_double::Spawn;

#[test]
fn logging_off_sends_the_character_and_leaves_the_world_but_keeps_the_list() {
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (mut client, server) = entered(spawn);
    client.take_notices();
    let player = client.frame().player.as_ref().expect("the player").id;
    let set = client
        .session()
        .characters()
        .cloned()
        .expect("the character set");
    assert!(client.frame().in_world);

    client.session_mut().log_off_character();
    let idle = FrameInput::default();
    let mut left = 0;
    for _ in 0..200 {
        let events = step(&mut client, &server, &idle);
        left += events
            .iter()
            .filter(|e| **e == SessionEvent::LeftWorld)
            .count();
        if left > 0 {
            break;
        }
    }

    // `Proto_UI::LogOffCharacter`: the last blob the client sent, on queue 4.
    let (_, queue, bytes) = server.borrow().blobs.last().cloned().expect("a blob");
    assert_eq!(
        (queue, bytes),
        ClientEvent::LogOffCharacter { character: player }.encode()
    );
    assert_eq!(left, 1, "CharExitGame once");
    assert!(!client.frame().in_world);
    assert!(!client.session().is_closed(), "the connection stays up");
    assert_eq!(client.session().characters(), Some(&set));
}

#[test]
fn a_second_logoff_after_the_character_left_sends_nothing() {
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (mut client, server) = entered(spawn);
    client.take_notices();
    let idle = FrameInput::default();
    client.session_mut().log_off_character();
    for _ in 0..200 {
        step(&mut client, &server, &idle);
    }
    // The character has left: its id is gone, so a second logoff sends nothing.
    assert!(!client.frame().in_world);
    let after_first = server.borrow().blobs.len();
    client.session_mut().log_off_character();
    for _ in 0..50 {
        step(&mut client, &server, &idle);
    }
    assert_eq!(server.borrow().blobs.len(), after_first);
}
