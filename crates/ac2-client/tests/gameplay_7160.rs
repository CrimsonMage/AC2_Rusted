//! The UI's gameplay commands through the client core against the Legions 7160 server double:
//! `Command::Select` goes out as the `WM_Player` target call, the double's `SelectionInfo`
//! comes back, and the frame's selection shows the object's name and health; `Command::Say`
//! comes back as the chat line the frame's events carry.

mod common;

use ac2_client::{Command, FrameInput, SelectionFrame};
use ac2_net::protocol::{ScriptRequest, SelectionInfo};
use ac2_weenie::{GameplayNotice, SessionEvent, DISPLAY_SPEECH};
use common::*;
use legions_server_double::{create_object_bytes, ObjectSpawn, Spawn};

const NPC: u64 = 0x7000_0000_0000_0042;

#[test]
fn selecting_an_object_fills_the_frame_selection() {
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    let info = SelectionInfo {
        kind: SelectionInfo::CREATURE,
        cur_health: 7,
        max_health: 9,
        ..SelectionInfo::default()
    };
    server
        .borrow_mut()
        .script_world
        .selection_infos
        .insert(NPC, info);
    server
        .borrow_mut()
        .send_event(&create_object_bytes(&ObjectSpawn {
            id: NPC,
            instance: 3,
            visual: 0x1F00_0023,
            cell: CELL,
            origin: [80.0, 70.0, GROUND],
            rotation: [1.0, 0.0, 0.0, 0.0],
            stamps: [10, 1, 1, 0],
            name: "Birch",
        }));
    let idle = FrameInput::default();
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert_eq!(client.frame().selection, None);

    let select = FrameInput {
        commands: vec![Command::Select(Some(NPC))],
        ..FrameInput::default()
    };
    let events = step(&mut client, &server, &select);
    assert!(
        events.contains(&SessionEvent::Notice(GameplayNotice::SelectionChanged(NPC))),
        "{events:?}"
    );
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(step(&mut client, &server, &idle));
    }
    assert!(
        events.contains(&SessionEvent::Notice(GameplayNotice::UpdateSelectionInfo {
            id: NPC,
            info
        })),
        "{events:?}"
    );
    let frame = client.frame();
    assert_eq!(
        frame.selection,
        Some(SelectionFrame {
            id: NPC,
            name: client.session().objects().get(NPC).unwrap().name.clone(),
            info: Some(info),
        })
    );

    let clear = FrameInput {
        commands: vec![Command::Select(None)],
        ..FrameInput::default()
    };
    step(&mut client, &server, &clear);
    assert_eq!(client.frame().selection, None);
    // The call leaves with the next frame's network turn.
    step(&mut client, &server, &idle);
    assert_eq!(
        server.borrow().script_requests,
        vec![ScriptRequest::SetTarget(NPC), ScriptRequest::SetTarget(0)]
    );
}

#[test]
fn saying_something_comes_back_as_a_chat_line() {
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    let say = FrameInput {
        commands: vec![Command::Say("Well met".into())],
        ..FrameInput::default()
    };
    let mut events = step(&mut client, &server, &say);
    let idle = FrameInput::default();
    for _ in 0..4 {
        events.extend(step(&mut client, &server, &idle));
    }
    let line = events.iter().find_map(|e| match e {
        SessionEvent::Notice(GameplayNotice::DisplayStringInfo { kind, text }) => {
            Some((*kind, text.string_id))
        }
        _ => None,
    });
    assert_eq!(line, Some((DISPLAY_SPEECH, 0xD85E_9379)), "{events:?}");
}
