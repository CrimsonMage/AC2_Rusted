//! The logout (`gmGamePlayUI::EndSession` -> `Proto_UI::LogOffCharacter`) against the Legions
//! 7160 server double: the player's character goes out on queue 4, the server answers
//! `CharExitGame` (`SessionEvent::LeftWorld`), the connection stays up, and the character set the
//! session kept is still there for character selection.

mod common;

use ac2_client::FrameInput;
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

    // `Proto_UI::LogOffCharacter` (Legions 0x004BE2F0): 12 bytes, opcode 0x27 (`CharExitGame`),
    // header word 1, the player's id as 8 little-endian bytes, sent on queue 4.
    let (_, queue, bytes) = server.borrow().blobs.last().cloned().expect("a blob");
    let mut retail = vec![0x27, 0x00, 0x01, 0x00];
    retail.extend_from_slice(&player.to_le_bytes());
    assert_eq!((queue, bytes), (4, retail));
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

#[test]
fn the_character_set_sent_after_logoff_carries_the_look_from_the_world() {
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (mut client, server) = entered(spawn);
    client.take_notices();
    let before = client.session().characters().cloned().expect("the set");
    assert_eq!(before.characters[0].visual.scale, None);
    // While the character is in the world its look changes (a larger figure, say).
    let changed = ac2_net::protocol::VisualDesc {
        parent: Some(0x1F00_0023),
        scale: Some([1.25, 1.25, 1.25]),
        ..Default::default()
    };
    server
        .borrow_mut()
        .set_explorer_visual(legions_server_double::visual_desc_bytes(&changed));

    client.session_mut().log_off_character();
    let idle = FrameInput::default();
    let (mut left, mut sets) = (false, 0);
    for _ in 0..200 {
        for event in step(&mut client, &server, &idle) {
            left |= event == SessionEvent::LeftWorld;
            sets += usize::from(matches!(event, SessionEvent::Characters(_)) && left);
        }
        if sets > 0 {
            break;
        }
    }
    assert!(left, "CharExitGame");
    assert_eq!(sets, 1, "the characters again after leaving");
    // The set character selection resumes with (`back_to_character_selection` reads it) has the
    // new look: the select screen builds its figure from this `VisualDesc`.
    let after = client.session().characters().expect("the set");
    assert_eq!(after.characters[0].visual.scale, Some([1.25, 1.25, 1.25]));
}
