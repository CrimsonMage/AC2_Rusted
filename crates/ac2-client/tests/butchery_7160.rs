//! Butchering end to end against the Legions server double (`AC2_DAT_DIR`): the player kills a
//! creature, uses the butchery tool on its corpse and receives the skin. The tool waits for a
//! target (`CUsageSystem` 0x9 keeps it in the usage list, `Usage_NumberOfEntitiesUsedWith` and
//! `Usage_InventoryRequired` on the tool), using the corpse completes it (`CUsageSystem` 0x3)
//! and `WM_Usage` 0x22 goes out. A failure comes back as `WM_Usage` 0x23 and shows the string
//! `CUsageSystem` 0x2 names for the error.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::ScriptRequest;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, BUTCHER_KNIFE, BUTCHER_SKIN, CORPSE, CREATURE_ONE, PLAYER};

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// Seconds between a fight's exchanges, and the exchanges that kill the creature.
const HIT_INTERVAL: f64 = 2.0;
/// `Death_IsBeingButchered`.
const IS_BEING_BUTCHERED: u32 = 0x125;

/// The text a usage line renders to (`StringInfo::GetString` over the DAT's string tables).
fn text(info: &ac2_net::protocol::StringInfo) -> String {
    let dataset = ac2_dat::test_support::env_dataset().expect("dataset");
    let store = ac2_db::ResourceStore::new(dataset);
    let tables = ac2_db::StoreStringTables::new(store, 0x5800_0000);
    let rendered = ac2_db::get_string(&tables, &ac2_client::chat_text::render_info(info), false);
    String::from_utf16_lossy(&rendered)
}

/// The usage lines (`DisplayStringInfo`, kind 8) the events raised, as text.
fn lines(events: &[SessionEvent]) -> Vec<String> {
    notices(events)
        .iter()
        .filter_map(|n| match n {
            GameplayNotice::DisplayStringInfo { text: info, .. } => Some(text(info)),
            _ => None,
        })
        .collect()
}

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

fn events_for(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
    frames: usize,
) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut events = Vec::new();
    for i in 0..frames {
        let frame = if i == 0 { &input } else { &idle };
        events.extend(step(client, server, frame));
    }
    events
}

fn run(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
) -> Vec<SessionEvent> {
    events_for(client, server, commands, 10)
}

fn notices(events: &[SessionEvent]) -> Vec<GameplayNotice> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

/// The script with the player attached, the starter items and the butchery tool in the pack, and
/// the creature killed: its corpse lies where it died.
fn with_corpse() -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset.clone(), store.clone());
    data.preload_tables();
    let names = DatScriptData::new(dataset, store);
    let (mut client, server) = entered(spawn());
    client
        .session_mut()
        .set_string_tables(ac2_client::string_table_ids(&names));
    client
        .session_mut()
        .set_chat_commands(ac2_client::chat_commands(&names));
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    server.borrow_mut().spawn_radar_objects();
    server.borrow_mut().start_fight();
    server.borrow_mut().give_butcher_tool();
    events_for(&mut client, &server, Vec::new(), 64);
    run(
        &mut client,
        &server,
        vec![Command::Select(Some(CREATURE_ONE))],
    );
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    events_for(
        &mut client,
        &server,
        Vec::new(),
        (3.0 * HIT_INTERVAL * 64.0) as usize + 64,
    );
    assert!(server.borrow().fight.dead.contains(&CREATURE_ONE));
    Some((client, server))
}

/// The uses that reached the double, in order.
fn uses(server: &Shared) -> Vec<(u64, u64)> {
    server
        .borrow()
        .script_requests
        .iter()
        .filter_map(|r| match r {
            ScriptRequest::Use(d) => Some((d.item, d.target)),
            _ => None,
        })
        .collect()
}

/// Selects the tool and uses it, then selects the corpse and uses it (the targeting click).
fn butcher(client: &mut ac2_client::Client, server: &Shared) -> Vec<SessionEvent> {
    let mut events = run(
        client,
        server,
        vec![Command::Select(Some(BUTCHER_KNIFE)), Command::UseSelection],
    );
    events.extend(use_corpse(client, server));
    events
}

/// The targeting click: select the corpse and use the selection.
fn use_corpse(client: &mut ac2_client::Client, server: &Shared) -> Vec<SessionEvent> {
    run(
        client,
        server,
        vec![Command::Select(Some(CORPSE)), Command::UseSelection],
    )
}

#[test]
fn butchering_a_corpse_gives_its_skin() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    // Using the tool alone only asks for a target: nothing goes to the server yet.
    let first = run(
        &mut client,
        &server,
        vec![Command::Select(Some(BUTCHER_KNIFE)), Command::UseSelection],
    );
    assert_eq!(
        lines(&first),
        vec!["Select a target for the Butchering Knife.\n"]
    );
    assert!(notices(&first).contains(&GameplayNotice::SetTargetingPointer(true)));
    assert!(uses(&server).is_empty());
    // Using the corpse completes it: the desc names the tool and the corpse.
    let second = use_corpse(&mut client, &server);
    assert_eq!(uses(&server), vec![(BUTCHER_KNIFE, CORPSE)]);
    let said = lines(&second);
    assert_eq!(said[0], "Using the Butchering Knife with the Corpse.\n");
    assert!(
        said.contains(&"Item use succeeds.\n".to_string()),
        "{said:?}"
    );
    // The corpse is marked and the skin is in the pack.
    assert!(second.iter().any(|e| matches!(
        e,
        SessionEvent::QualityChanged { object, quality, .. } if *object == CORPSE && *quality == IS_BEING_BUTCHERED
    )));
    let pack = client.frame().inventory.pack.items.clone();
    assert!(pack.contains(&BUTCHER_SKIN), "{pack:?}");
    assert!(client.take_script_problems().is_empty());
}

/// A second butchering of the same corpse is refused (`Usage_Butchery_AlreadyButchered`), as is
/// one of a corpse with no skin (`_Empty`) and one with no room in the pack (`_NoRoom`): the
/// double's `WM_Usage` 0x23 reaches `CUsageSystem` 0x2, whose text is shown.
#[test]
fn a_refused_butchering_shows_its_error() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    butcher(&mut client, &server);
    let again = use_corpse(&mut client, &server);
    assert_eq!(
        lines(&again).last().map(String::as_str),
        Some("The Corpse has already been butchered.\n")
    );
    assert_eq!(uses(&server).len(), 2);
    let pack = client.frame().inventory.pack.items.clone();
    assert_eq!(pack.iter().filter(|i| **i == BUTCHER_SKIN).count(), 1);
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn an_empty_corpse_and_a_full_pack_show_their_errors() {
    let Some((mut client, server)) = with_corpse() else {
        return;
    };
    server.borrow_mut().set_corpse_skinless(true);
    let empty = butcher(&mut client, &server);
    assert_eq!(
        lines(&empty).last().map(String::as_str),
        Some("The Corpse contains nothing of value.\n")
    );
    assert!(!client.frame().inventory.pack.items.contains(&BUTCHER_SKIN));
    server.borrow_mut().set_corpse_skinless(false);
    server
        .borrow_mut()
        .script_world
        .capacities
        .insert(PLAYER, 3);
    // The pointer is armed again after the refusal: the next click is on the corpse.
    let full = use_corpse(&mut client, &server);
    assert_eq!(
        lines(&full).last().map(String::as_str),
        Some("You\u{2019}re carrying too much to butcher the Corpse.\n")
    );
    assert!(!client.frame().inventory.pack.items.contains(&BUTCHER_SKIN));
    assert!(client.take_script_problems().is_empty());
}
