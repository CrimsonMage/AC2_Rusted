//! The store's calls with the player's script on the VM (`AC2_DAT_DIR`). A store request is the
//! IFun's stub, which the VM's `CStoreSystem` sends only with a shopper open (the server's
//! `ViewStore` opens one; `CStoreSystem` 0x10 and 0x1C), so a request before that goes nowhere.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::Spawn;

const ITEM: u64 = 0x8000_0002_0000_0021;
/// A StoreTemplate's WState DID in portal.dat (the `0x6F000000..` range).
const STORE_TEMPLATE: u32 = 0x6F00_33A3;
const VENDOR: u64 = 0x8000_0002_0000_0042;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

#[test]
fn enter_store_without_an_open_shopper_sends_nothing() {
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
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    let input = FrameInput {
        commands: vec![Command::EnterStore(0x7000_1234), Command::LeaveStore],
        ..FrameInput::default()
    };
    step(&mut client, &server, &input);
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(server.borrow().script_requests.is_empty());
    assert!(client.take_script_problems().is_empty());
    // The VM answers the quantity from the store it has: none open, so nothing to buy.
    assert_eq!(client.max_purchase_quantity(ITEM), Some(0));
}

#[test]
fn store_calls_do_nothing_without_the_script_vm() {
    let (mut client, server) = entered(spawn());
    let input = FrameInput {
        commands: vec![
            Command::EnterStore(0x7000_1234),
            Command::PurchaseItem {
                item: ITEM,
                quantity: 1,
                slot: 0,
            },
            Command::RequestSell {
                item: ITEM,
                quantity: 1,
                price: 10,
            },
        ],
        ..FrameInput::default()
    };
    let idle = FrameInput::default();
    step(&mut client, &server, &input);
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(server.borrow().script_requests.is_empty());
    assert_eq!(client.max_purchase_quantity(ITEM), None);
}

/// The script with the player attached, after the login's idle frames.
fn with_script() -> Option<(
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
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
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server))
}

/// Four frames after `open`, the notices the session raised in them, in order.
fn notices_after(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) -> Vec<GameplayNotice> {
    let idle = FrameInput::default();
    let mut notices = Vec::new();
    for _ in 0..4 {
        for event in step(client, server, &idle) {
            if let SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
    }
    notices
}

/// The server's ViewStore for a StoreTemplate, run on the script. The forwarder takes the store as
/// a reference, and the shop build (`CStoreSystem` 0x1170E8's branch at 0x117D64) does not
/// reach ViewStore yet: its `GetPackageReference(did, 0x3810)` gets the StoreTemplate
/// function's return (1) as the DID, which is no WState, so the script answers LeaveStore. This
/// pins that outcome; the test changes when the shop build is understood.
#[test]
fn view_store_of_a_store_template_stops_before_the_shop() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    server.borrow_mut().open_store(STORE_TEMPLATE, VENDOR);
    let notices = notices_after(&mut client, &server);
    assert_eq!(
        notices,
        [
            GameplayNotice::LeaveConsignment,
            GameplayNotice::LeaveStore,
            GameplayNotice::LeaveConsignment,
            GameplayNotice::LeaveCatalog,
            GameplayNotice::LeaveStore,
        ]
    );
    assert!(client.take_script_problems().is_empty());
}

/// A sell with no shopper open: `CStoreSystem`'s sell body checks the shopper first (`CPlayer`
/// 0x1EC) and returns, so no request goes out and the VM reports no problem.
#[test]
fn sell_without_a_shopper_sends_nothing() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let input = FrameInput {
        commands: vec![Command::RequestSell {
            item: ITEM,
            quantity: 1,
            price: 10,
        }],
        ..FrameInput::default()
    };
    step(&mut client, &server, &input);
    let idle = FrameInput::default();
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(server.borrow().script_requests.is_empty());
    assert!(client.take_script_problems().is_empty());
}
