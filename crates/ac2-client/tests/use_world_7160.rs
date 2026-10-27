//! Using world objects against the Legions server double (`AC2_DAT_DIR`): the object is selected
//! (`Command::Select`, `gmCWInterface::SetSelection`), then the use key runs
//! `gmCWInterface::UseSelection`, which sends `WM_Usage` 0x22 with a `UsageDesc` naming the
//! player, the object and no target. The server decides what the use does (a chest opens, an NPC
//! starts its conversation, a ground item is taken); the client only sends the use and shows the
//! outcome. Nothing selected sends nothing.

mod common;

use ac2_client::{Command, FrameInput};
use ac2_net::protocol::{ScriptRequest, UsageDesc};
use common::*;
use legions_server_double::{Spawn, CHEST, NPC, PLAYER};

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// Steps the client with `commands` on the first frame, then idle frames to let the answers in.
fn play(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    commands: Vec<Command>,
) {
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    step(client, server, &input);
    for _ in 0..8 {
        step(client, server, &FrameInput::default());
    }
}

/// The `Use` requests the double heard, in order.
fn uses(server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>) -> Vec<UsageDesc> {
    server
        .borrow()
        .script_requests
        .iter()
        .filter_map(|request| match request {
            ScriptRequest::Use(desc) => Some(desc.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_selected_world_object_is_used_with_no_target() {
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.send_init_player();
        state.spawn_radar_objects();
        state.spawn_chest();
    }
    step(&mut client, &server, &FrameInput::default());

    // The chest is selected, then used: one `UsageDesc` for the chest.
    play(&mut client, &server, vec![Command::Select(Some(CHEST))]);
    play(&mut client, &server, vec![Command::UseSelection]);
    assert_eq!(uses(&server), vec![UsageDesc::new(PLAYER, CHEST, 0)]);

    // The NPC is the same kind of use; the server's answer, not the client, decides what follows.
    play(&mut client, &server, vec![Command::Select(Some(NPC))]);
    play(&mut client, &server, vec![Command::UseSelection]);
    assert_eq!(
        uses(&server),
        vec![
            UsageDesc::new(PLAYER, CHEST, 0),
            UsageDesc::new(PLAYER, NPC, 0),
        ]
    );
    assert!(server.borrow().script_errors.is_empty());
}

#[test]
fn use_with_nothing_selected_sends_nothing() {
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.send_init_player();
        state.spawn_radar_objects();
        state.spawn_chest();
    }
    step(&mut client, &server, &FrameInput::default());
    play(&mut client, &server, vec![Command::UseSelection]);
    assert!(uses(&server).is_empty());
}
