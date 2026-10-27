//! The world objects a player uses, against the Legions server double (`AC2_DAT_DIR`): a door and
//! a lever are selected and used (`Command::UseSelection`, one `WM_Usage` with no target), and the
//! server answers the use with success; a ground item used is taken into the player's main pack
//! (the server's `CONTAIN` for it, the pack's list, the use's answer), so the client's object is
//! contained and the server holds it in the pack. As in `use_world_7160.rs`, the client only sends
//! the use and shows what the server sends back.

mod common;

use ac2_client::{Client, Command, FrameInput};
use ac2_net::protocol::{ScriptRequest, UsageDesc};
use common::*;
use legions_server_double::{Spawn, State, DOOR, GROUND_ITEM, LEVER, PLAYER};

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Server = std::rc::Rc<std::cell::RefCell<State>>;

/// Steps the client with `commands` on the first frame, then idle frames to let the answers in.
fn play(client: &mut Client, server: &Server, commands: Vec<Command>) {
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
fn uses(server: &Server) -> Vec<UsageDesc> {
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

/// Whether the server lists `item` in the player's main pack.
fn in_pack(server: &Server, item: u64) -> bool {
    server
        .borrow()
        .script_world
        .contents
        .get(&PLAYER)
        .is_some_and(|(_, items)| items.contains(&item))
}

/// A client in the world with the double's door, lever and ground item placed.
fn world() -> (Client, Server) {
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.send_init_player();
        state.give_starter_items();
        state.spawn_radar_objects();
        state.spawn_world_objects();
    }
    step(&mut client, &server, &FrameInput::default());
    (client, server)
}

#[test]
fn a_door_and_a_lever_are_used_and_the_server_answers_success() {
    let (mut client, server) = world();

    play(&mut client, &server, vec![Command::Select(Some(DOOR))]);
    play(&mut client, &server, vec![Command::UseSelection]);
    assert_eq!(uses(&server), vec![UsageDesc::new(PLAYER, DOOR, 0)]);

    play(&mut client, &server, vec![Command::Select(Some(LEVER))]);
    play(&mut client, &server, vec![Command::UseSelection]);
    assert_eq!(
        uses(&server),
        vec![
            UsageDesc::new(PLAYER, DOOR, 0),
            UsageDesc::new(PLAYER, LEVER, 0),
        ]
    );
    // The answers came back as the double's success (no error), and nothing was left unread.
    assert!(server.borrow().script_errors.is_empty());
    let object = client
        .session()
        .objects()
        .get(DOOR)
        .expect("the door is in the world");
    assert!(!object.contained, "a door stays where it is");
}

#[test]
fn a_ground_item_used_is_taken_into_the_main_pack() {
    let (mut client, server) = world();
    let before = client
        .session()
        .objects()
        .get(GROUND_ITEM)
        .expect("the item is in the world");
    assert!(!before.contained);
    assert!(
        !in_pack(&server, GROUND_ITEM),
        "not in the pack before the use"
    );

    play(
        &mut client,
        &server,
        vec![Command::Select(Some(GROUND_ITEM))],
    );
    play(&mut client, &server, vec![Command::UseSelection]);
    assert_eq!(uses(&server), vec![UsageDesc::new(PLAYER, GROUND_ITEM, 0)]);

    // The item left the landblock for the inventory (`RecvEvt_Contain`) and is in the pack.
    let after = client
        .session()
        .objects()
        .get(GROUND_ITEM)
        .expect("the item is known");
    assert!(after.contained, "the client's object is contained");
    assert!(
        in_pack(&server, GROUND_ITEM),
        "in the main pack after the use"
    );
    assert!(server.borrow().script_errors.is_empty());
}
