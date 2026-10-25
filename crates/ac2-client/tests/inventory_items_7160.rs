//! The double's starter inventory: the pack in the player's contents, its items listed when it
//! opens, and what the inventory panels show of each, in the frame (the `gmEntity` getters over
//! the item's qualities) and as the item's own process answers `ItemDisplay::Update`'s
//! questions on the script VM (`AC2_DAT_DIR`).

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::StringVariable;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{Spawn, State, PACK, PLAYER, POTIONS, SWORD};

/// Gives the starter items and opens the pack.
fn open_the_pack(client: &mut Client, server: &Rc<RefCell<State>>) {
    let idle = FrameInput::default();
    step(client, server, &idle);
    server.borrow_mut().give_starter_items();
    for _ in 0..6 {
        step(client, server, &idle);
    }
    server.borrow_mut().send_init_player();
    for _ in 0..6 {
        step(client, server, &idle);
    }
    let open = FrameInput {
        commands: vec![Command::SetContainerOpen {
            container: PACK,
            open: true,
        }],
        ..FrameInput::default()
    };
    step(client, server, &open);
    for _ in 0..6 {
        step(client, server, &idle);
    }
}

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

#[test]
fn the_frame_carries_what_the_panels_show() {
    let (mut client, server) = entered(spawn());
    open_the_pack(&mut client, &server);
    let frame = client.frame();
    let inventory = &frame.inventory;
    assert_eq!(inventory.pack.id, PLAYER);
    assert_eq!(inventory.pack.packs, [PACK]);
    let pack = inventory
        .containers
        .iter()
        .find(|c| c.id == PACK)
        .unwrap_or_else(|| panic!("{inventory:?}"));
    assert_eq!(
        (pack.items.as_slice(), pack.capacity),
        (&[POTIONS, SWORD][..], 24)
    );
    assert!(inventory.open.contains(&PACK));
    let ids: Vec<u64> = inventory.items.iter().map(|i| i.id).collect();
    assert_eq!(ids, [PACK, POTIONS, SWORD]);

    // A stack of five potions, stackable to ten: "5 <plural>" (the count and the plural).
    let potions = &inventory.items[1];
    let stats = &potions.stats;
    assert_eq!(
        (stats.quantity, stats.stackable, stats.max_stack, stats.icon),
        (5, true, Some(10), 0x0600_1001)
    );
    assert_eq!((stats.value, stats.durability), (20, None));
    assert_eq!(stats.name.literal.as_deref(), Some("Healing Potion"));
    assert_eq!(potions.name.string_id, 0xFE4D_81DA);
    assert_eq!(
        potions.name.variables[0],
        (0x9D56_6800, StringVariable::Word { kind: 2, value: 5 })
    );
    assert!(matches!(
        &potions.name.variables[1],
        (0x8615_ECB3, StringVariable::Nested(n)) if n.literal.as_deref() == Some("Healing Potions")
    ));
    // A sword: one, not stackable, durability 40 of 50, its name as sent.
    let sword = &inventory.items[2];
    assert_eq!((sword.stats.quantity, sword.stats.stackable), (1, false));
    assert_eq!(
        (sword.stats.durability, sword.stats.value),
        (Some((40, 50)), 50)
    );
    assert_eq!(sword.name.literal.as_deref(), Some("Practice Sword"));
}

#[test]
fn the_items_processes_answer_from_their_tables() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    open_the_pack(&mut client, &server);
    // The qualities the description does not carry: the pack's capacity, the stack's limit.
    // (Quantity, icon, name and durability are `gmWeenieDesc` fields, which the VM's
    // repository does not read yet: see the coord file.)
    assert_eq!(client.container_capacity(PACK), Ok(24));
    let potions = client.item_display(POTIONS).unwrap();
    assert_eq!((potions.stackable, potions.max_stack), (true, 10));
    assert_eq!(potions.value, Some(20));
    let sword = client.item_display(SWORD).unwrap();
    assert!(!sword.stackable);
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}
