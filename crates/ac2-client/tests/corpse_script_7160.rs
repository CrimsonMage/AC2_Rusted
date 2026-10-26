//! Corpse looting against the Legions server double (`AC2_DAT_DIR`): a monster's corpse (its
//! `Corpse` package, 0x182, with its weenie type, int 20) and its loot, a trophy skin
//! (`TrophyMimbuSkin`) and an item, open as an external container. Loot All runs
//! `gmCWInterface::LootAllFromContainer`'s IFun (0xB400E8): the wlib counts the corpse's lists
//! (`Container` 0x2B: slots 15 and 19), which the open answer fills. The answer reaches the
//! player's script (`CPlayer` 0x8C, then `CInventorySystem`'s static, which finds the corpse's
//! instance by IID with `GetInternalWeenieObject`); the corpse has its process from its
//! `CreateObject`. The take-all descriptor (`WM_Inventory` 0x68) goes out byte for byte, and the
//! double answers it by moving every item into the pack, the skin included; a single item comes
//! off the corpse with a move. Another character's corpse is refused in the open answer
//! (`NotMine`), so its lists stay empty and the take-all is never sent.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::ScriptRequest;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{
    Spawn, State, CORPSE, CORPSE_ITEM, CORPSE_SKIN, CORPSE_WEENIE_TYPE, FOREIGN_CORPSE,
    FOREIGN_ITEM, PLAYER,
};

/// `WM_Inventory` 0x68: the take-all request (`ScriptRequest::LootAll`).
const LOOT_ALL: u32 = 0x820E_0068;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

fn notices(events: Vec<SessionEvent>) -> Vec<GameplayNotice> {
    events
        .into_iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect()
}

/// Steps the client with `command` (if any) and then idle frames; the notices they raised.
fn run(
    client: &mut Client,
    server: &Rc<RefCell<State>>,
    command: Option<Command>,
) -> Vec<GameplayNotice> {
    let input = FrameInput {
        commands: command.into_iter().collect(),
        ..FrameInput::default()
    };
    let mut events = step(client, server, &input);
    for _ in 0..6 {
        events.extend(step(client, server, &FrameInput::default()));
    }
    notices(events)
}

/// The client logged in with the scripts, and the corpse spawned in the world.
fn with_corpse() -> Option<(Client, Rc<RefCell<State>>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    run(&mut client, &server, None);
    server.borrow_mut().spawn_corpse();
    run(&mut client, &server, None);
    Some((client, server))
}

/// The items container `id` lists in the frame (the player's pack for the player).
fn listed(client: &Client, id: u64) -> Option<Vec<u64>> {
    let inventory = client.frame().inventory;
    if inventory.pack.id == id {
        return Some(inventory.pack.items);
    }
    inventory
        .containers
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.items.clone())
}

/// The packed argument bytes of every `function` the client sent, in order.
fn sent(server: &Rc<RefCell<State>>, function: u32) -> Vec<Vec<u8>> {
    server
        .borrow()
        .script_bytes
        .iter()
        .filter(|(f, _)| *f == function)
        .map(|(_, b)| b.clone())
        .collect()
}

fn open(client: &mut Client, server: &Rc<RefCell<State>>) {
    run(
        client,
        server,
        Some(Command::SetContainerOpen {
            container: CORPSE,
            open: true,
        }),
    );
}

/// The take-all descriptor as `CInventorySystem` 0x1C's stub packs it for the corpse and the
/// player (`WM_Inventory` 0x68): the wlib's own values, its object tags and state id included.
const STUB_TAKE_ALL: &[u8] = &[
    5, 0, 0, 0, 1, 0, 0, 0, 2, 103, 0, 0, 0, 1, 0, 68, 0, 0, 0, 0, 0, 0, 239, 32, 14, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 0, 0, 0, 0, 0, 128,
    255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 1, 0, 0, 0, 0, 0, 0, 32, 2, 2,
    2, 2, 2, 2, 132, 205, 1, 2, 2, 1, 132, 205, 0, 0,
];

#[test]
fn loot_all_takes_every_item_of_a_corpse() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    // The corpse's weenie type (int 20): `Corpse`, which the loot check's `(type & 3) == 3` passes.
    let corpse = client.session().objects().get(CORPSE).expect("the corpse");
    assert_eq!(corpse.qualities.ints.get(&20), Some(&CORPSE_WEENIE_TYPE));
    open(&mut client, &server);
    // Open: the corpse lists its skin and item, and the external container shows it open.
    assert_eq!(
        listed(&client, CORPSE),
        Some(vec![CORPSE_SKIN, CORPSE_ITEM])
    );
    assert!(client.frame().inventory.open.contains(&CORPSE));

    // Loot All: the IFun's stub sends the take-all descriptor (the wlib's bytes); the double
    // answers it, moving the skin and the item into the pack.
    run(&mut client, &server, Some(Command::LootAll(CORPSE)));
    assert_eq!(sent(&server, LOOT_ALL), vec![STUB_TAKE_ALL.to_vec()]);
    let asked = server.borrow().script_requests.clone();
    let Some(ScriptRequest::LootAll(taken)) = asked
        .iter()
        .find(|r| matches!(r, ScriptRequest::LootAll(_)))
        .cloned()
    else {
        panic!("no take-all request: {asked:?}");
    };
    assert_eq!((taken.container(), taken.player()), (CORPSE, PLAYER));
    assert_eq!(listed(&client, CORPSE), Some(Vec::new()));
    let pack = listed(&client, PLAYER).unwrap_or_default();
    assert!(
        pack.contains(&CORPSE_SKIN) && pack.contains(&CORPSE_ITEM),
        "{pack:?}"
    );

    run(
        &mut client,
        &server,
        Some(Command::SetContainerOpen {
            container: CORPSE,
            open: false,
        }),
    );
    assert!(!client.frame().inventory.open.contains(&CORPSE));
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}

#[test]
fn a_single_item_comes_off_a_corpse_and_the_rest_stays() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    open(&mut client, &server);
    // The trophy skin taken out of the corpse into the pack (the inventory's move path).
    run(
        &mut client,
        &server,
        Some(Command::MoveItem {
            item: CORPSE_SKIN,
            from: CORPSE,
            from_slot: 0,
            target: PLAYER,
            target_slot: -1,
            quantity: 1,
        }),
    );
    assert_eq!(listed(&client, CORPSE), Some(vec![CORPSE_ITEM]));
    assert!(listed(&client, PLAYER).is_some_and(|pack| pack.contains(&CORPSE_SKIN)));
    // No Loot All was asked, so none went out.
    assert!(sent(&server, LOOT_ALL).is_empty());
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}

#[test]
fn loot_all_needs_the_corpse_opened_first() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    // Unopened, the corpse's lists are empty: the wlib's count refuses, so no take-all is sent.
    run(&mut client, &server, Some(Command::LootAll(CORPSE)));
    assert!(sent(&server, LOOT_ALL).is_empty());
    assert!(listed(&client, PLAYER).is_some_and(|pack| !pack.contains(&CORPSE_ITEM)));
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}

#[test]
fn another_characters_corpse_does_not_open_or_loot_for_the_player() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    server.borrow_mut().spawn_foreign_corpse();
    run(&mut client, &server, None);
    // The open answer refuses it: nothing is listed, and the corpse is not open on the player.
    run(
        &mut client,
        &server,
        Some(Command::SetContainerOpen {
            container: FOREIGN_CORPSE,
            open: true,
        }),
    );
    assert_eq!(listed(&client, FOREIGN_CORPSE), None);
    assert!(!client.frame().inventory.open.contains(&FOREIGN_CORPSE));
    // Loot All: the refused corpse's lists are empty, so the wlib sends no take-all.
    run(&mut client, &server, Some(Command::LootAll(FOREIGN_CORPSE)));
    assert!(sent(&server, LOOT_ALL).is_empty());
    assert!(listed(&client, PLAYER).is_some_and(|pack| !pack.contains(&FOREIGN_ITEM)));
    // The player's own corpse still opens and loots.
    open(&mut client, &server);
    assert_eq!(
        listed(&client, CORPSE),
        Some(vec![CORPSE_SKIN, CORPSE_ITEM])
    );
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}
