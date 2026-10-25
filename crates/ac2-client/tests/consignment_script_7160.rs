//! The consignment calls with the player's script on the VM (`AC2_DAT_DIR`). Entering and leaving
//! the consignment are the IFuns' stubs, which the VM packs (the storekeeper, one long int) and the
//! session sends, but only with a shopper open (the store the storekeeper opened). The sale itself
//! (`RequestSell`) is refused outside a consignment: the VM's sell body raises a message and sends
//! nothing. The consignment's answers (`ViewConsignment`, `UpdateConsignment`) are not run yet (see
//! the catalog notes).

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{pack_values, PackValue, ScriptRequest, StoreView};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, PLAYER, POTIONS};

/// The storekeeper the shopper is open with.
const VENDOR: u64 = 0x8000_0002_0000_0042;
/// `RequestEnterConsignment`'s stub (WM_Store 0x46) and `RequestLeaveConsignment`'s (0x45).
const ENTER_CONSIGNMENT: u32 = 0xB83A_0046;
const LEAVE_CONSIGNMENT: u32 = 0xB83A_0045;
/// `RequestSell`'s stub (WM_Store 0x43).
const SELL: u32 = 0xB83A_0043;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The script with the player attached, after the login's idle frames.
fn with_script() -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server))
}

/// Runs the commands for four frames; the notices the frames raised, in order.
fn run(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut notices = Vec::new();
    for i in 0..4 {
        let frame = if i == 0 { &input } else { &idle };
        for event in step(client, server, frame) {
            if let SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
    }
    notices
}

/// Opens the vendor store with the storekeeper `VENDOR` (no sales), as the server does, and drains
/// the notices the shop body raises.
fn open_shopper(client: &mut ac2_client::Client, server: &Shared) {
    server.borrow_mut().open_store(
        StoreView {
            store: 0x6F00_33A3,
            store_size: 0,
            position: 0,
            sales: Vec::new(),
        },
        VENDOR,
    );
    run(client, server, Vec::new());
}

/// The packed bytes the VM sent for function `function`, in order.
fn sent(server: &Shared, function: u32) -> Vec<Vec<u8>> {
    server
        .borrow()
        .script_bytes
        .iter()
        .filter(|(f, _)| *f == function)
        .map(|(_, bytes)| bytes.clone())
        .collect()
}

/// Without a shopper open the enter stub sends nothing (the VM's shopper check), and the charge is
/// the script's answer: zero with no consignment.
#[test]
fn enter_consignment_without_a_shopper_sends_nothing() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    run(
        &mut client,
        &server,
        vec![Command::EnterConsignment, Command::LeaveConsignment],
    );
    assert!(sent(&server, ENTER_CONSIGNMENT).is_empty());
    assert!(sent(&server, LEAVE_CONSIGNMENT).is_empty());
    assert!(server.borrow().script_requests.is_empty());
    assert_eq!(client.consignment_charge(), Some(0.0));
    assert!(client.take_script_problems().is_empty());
}

/// With the shopper open, entering and leaving send the storekeeper as one long int: the VM's
/// packed bytes are the stub's PackBuffer, and the double decodes them as the requests.
#[test]
fn enter_and_leave_consignment_send_the_storekeeper() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    open_shopper(&mut client, &server);
    run(&mut client, &server, vec![Command::EnterConsignment]);
    assert_eq!(
        sent(&server, ENTER_CONSIGNMENT),
        vec![pack_values(&[PackValue::LongInt(VENDOR)])],
        "the enter stub's PackBuffer"
    );
    run(&mut client, &server, vec![Command::LeaveConsignment]);
    assert_eq!(
        sent(&server, LEAVE_CONSIGNMENT),
        vec![pack_values(&[PackValue::LongInt(VENDOR)])],
        "the leave stub's PackBuffer"
    );
    let requests = server.borrow().script_requests.clone();
    assert!(requests.contains(&ScriptRequest::EnterConsignment(VENDOR)));
    assert!(requests.contains(&ScriptRequest::LeaveConsignment(VENDOR)));
    assert!(client.take_script_problems().is_empty());
}

/// A sale outside a consignment is refused by the VM's sell body: it raises one message and sends
/// no `RequestSell` (the stub is not reached).
#[test]
fn sell_outside_a_consignment_is_refused_by_the_script() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    open_shopper(&mut client, &server);
    // The refusal's message comes a few frames after the command (the VM's sell body runs on).
    let idle = FrameInput::default();
    let input = FrameInput {
        commands: vec![Command::RequestSell {
            item: POTIONS,
            quantity: 1,
            price: 10,
        }],
        ..FrameInput::default()
    };
    let mut notices = Vec::new();
    for i in 0..10 {
        let frame = if i == 0 { &input } else { &idle };
        for event in step(&mut client, &server, frame) {
            if let SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
    }
    assert!(sent(&server, SELL).is_empty(), "no RequestSell sent");
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        matches!(notices[0], GameplayNotice::DisplayStringInfo { .. }),
        "the refusal's message: {notices:?}"
    );
    assert!(client.take_script_problems().is_empty());
}
