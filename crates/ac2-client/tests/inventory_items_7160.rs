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
use ac2_weenie::GameplayNotice;
use common::*;
use legions_server_double::{
    Spawn, State, PACK, PLAYER, POTIONS, PRIMARY_HAND, RATIONS, SECONDARY_HAND, SWORD,
};

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
    assert_eq!(
        (inventory.pack.items.as_slice(), inventory.pack.capacity),
        (&[POTIONS, SWORD][..], 24)
    );
    let pack = inventory
        .containers
        .iter()
        .find(|c| c.id == PACK)
        .unwrap_or_else(|| panic!("{inventory:?}"));
    assert_eq!((pack.items.as_slice(), pack.capacity), (&[RATIONS][..], 24));
    assert!(inventory.open.contains(&PACK));
    let ids: Vec<u64> = inventory.items.iter().map(|i| i.id).collect();
    assert_eq!(ids, [PACK, POTIONS, SWORD, RATIONS]);
    assert_eq!(inventory.money, 250);

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
    // The sword goes in either hand, the primary when dropped on the doll's figure.
    assert_eq!(
        (sword.stats.valid_locations, sword.stats.preferred_location),
        (PRIMARY_HAND | SECONDARY_HAND, PRIMARY_HAND)
    );
    assert!(ac2_weenie::EQUIP_LOCATIONS.contains(&PRIMARY_HAND));
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
    // The tables' qualities (the pack's capacity, the stack's limit) and the `gmWeenieDesc`
    // fields the repository reads first (`CBaseQualities::Inq*`: quantity, icon, name,
    // durability).
    assert_eq!(client.container_capacity(PACK), Ok(24));
    let potions = client.item_display(POTIONS).unwrap();
    assert_eq!((potions.stackable, potions.max_stack), (true, 10));
    assert_eq!(potions.value, Some(20));
    assert_eq!((potions.quantity, potions.icon), (5, 0x0600_1001));
    assert_eq!(potions.durability, None);
    let sword = client.item_display(SWORD).unwrap();
    assert!(!sword.stackable);
    assert_eq!((sword.quantity, sword.durability), (1, Some((40, 50))));
    // The names: "5 <plural>" for the stack (its count and plural), the sword's as sent.
    assert_eq!(potions.name.string_id, 0xFE4D_81DA);
    assert_eq!(
        potions.name.variables[0],
        (0x9D56_6800, StringVariable::Word { kind: 2, value: 5 })
    );
    assert!(matches!(
        &potions.name.variables[1],
        (0x8615_ECB3, StringVariable::Nested(n)) if n.literal.as_deref() == Some("Healing Potions")
    ));
    assert_eq!(sword.name.literal.as_deref(), Some("Practice Sword"));
    // The backpack's pages: `Container` 0x3D / 0x3F over the player's items and its one
    // segment (the double's listing: 24 slots, two items).
    assert_eq!(client.contents_info(0), Ok((vec![POTIONS, SWORD], 0, 2)));
    assert_eq!(client.contents_info(1), Ok((vec![POTIONS, SWORD], 2, 0)));
    assert_eq!(client.container_segment_info(0), Ok((0, 2)));
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}

/// One command, then frames until the double's answers are in; the frame's inventory and the
/// notices on the way.
fn run(
    client: &mut Client,
    server: &Rc<RefCell<State>>,
    command: Command,
) -> (ac2_client::InventoryFrame, Vec<GameplayNotice>) {
    let input = FrameInput {
        commands: vec![command],
        ..FrameInput::default()
    };
    let mut events = step(client, server, &input);
    for _ in 0..6 {
        events.extend(step(client, server, &FrameInput::default()));
    }
    let notices = events
        .into_iter()
        .filter_map(|e| match e {
            ac2_weenie::SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect();
    (client.frame().inventory, notices)
}

#[test]
fn drags_and_drops_refresh_the_frame() {
    let (mut client, server) = entered(spawn());
    open_the_pack(&mut client, &server);
    let container = |inv: &ac2_client::InventoryFrame, id| {
        let c = std::iter::once(&inv.pack)
            .chain(&inv.containers)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("{inv:?}"));
        c.items.clone()
    };
    let refreshed = |notices: &[GameplayNotice], id| {
        notices.iter().any(
            |n| matches!(n, GameplayNotice::UpdateContainer { container, .. } if *container == id),
        )
    };
    // Within the pack: the sword to the first slot (a reorganization).
    let (inv, notices) = run(
        &mut client,
        &server,
        Command::MoveItem {
            item: SWORD,
            from: PLAYER,
            from_slot: 1,
            target: PLAYER,
            target_slot: 0,
            quantity: 1,
        },
    );
    assert_eq!(container(&inv, PLAYER), [SWORD, POTIONS]);
    assert!(refreshed(&notices, PLAYER), "{notices:?}");
    // Pack to side pack: the potions after the rations.
    let (inv, notices) = run(
        &mut client,
        &server,
        Command::MoveItem {
            item: POTIONS,
            from: PLAYER,
            from_slot: 1,
            target: PACK,
            target_slot: -1,
            quantity: 5,
        },
    );
    assert_eq!(container(&inv, PLAYER), [SWORD]);
    assert_eq!(container(&inv, PACK), [RATIONS, POTIONS]);
    assert!(refreshed(&notices, PACK), "{notices:?}");
    // Pack to the primary hand, and back.
    let (inv, notices) = run(
        &mut client,
        &server,
        Command::EquipItem {
            item: SWORD,
            location: PRIMARY_HAND,
        },
    );
    assert_eq!(inv.equipped, [(PRIMARY_HAND, SWORD)]);
    assert!(container(&inv, PLAYER).is_empty());
    assert!(notices
        .iter()
        .any(|n| matches!(n, GameplayNotice::UpdateInventory { .. })));
    let (inv, _) = run(
        &mut client,
        &server,
        Command::UnequipItem {
            item: SWORD,
            target_slot: 0,
        },
    );
    assert!(inv.equipped.is_empty());
    assert_eq!(container(&inv, PLAYER), [SWORD]);
    // The money bag: 50 out (no coin item made), then the rations in.
    assert_eq!(inv.money, 250);
    let (inv, notices) = run(
        &mut client,
        &server,
        Command::DragFromMoneyBag {
            quantity: 50,
            to_slot: -1,
            to_container: PLAYER,
        },
    );
    assert_eq!(inv.money, 200);
    assert!(
        notices.contains(&GameplayNotice::UpdateMoney(200)),
        "{notices:?}"
    );
    assert!(notices.contains(&GameplayNotice::GhostMoneyBag(false)));
    let (inv, _) = run(
        &mut client,
        &server,
        Command::DragToMoneyBag {
            item: RATIONS,
            quantity: 3,
            from_slot: 0,
            from_container: PACK,
        },
    );
    assert_eq!(inv.money, 203);
    assert_eq!(container(&inv, PACK), [POTIONS]);
    // More than the player has is refused before sending.
    let (_, _) = run(
        &mut client,
        &server,
        Command::DragFromMoneyBag {
            quantity: 1000,
            to_slot: -1,
            to_container: PLAYER,
        },
    );
    let calls: Vec<u32> = server
        .borrow()
        .script_calls
        .iter()
        .map(|c| c.function)
        .collect();
    assert_eq!(calls, [0x820F_0001, 0x820F_0002]);
}
