//! The store's paging, restock, collect and leave-catalog requests with the player's script on the
//! VM (`AC2_DAT_DIR`). Each request's IFun stub packs its values and the session sends them as the
//! `WM_Store` event the double decodes; the storekeeper comes from the shopper the server opened.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{pack_values, PackValue, ScriptRequest, StoreView};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::SessionEvent;
use common::*;
use legions_server_double::{Spawn, POTIONS};

/// The StoreTemplate the shopper opens with (portal.dat WState).
const STORE_TEMPLATE: u32 = 0x6F00_33A3;
/// The storekeeper the shopper is open with.
const VENDOR: u64 = 0x8000_0002_0000_0042;
/// The `WM_Store` stubs: `RequestSales` (0x70), `RequestNextSales` (0x2F), `RequestPrevSales`
/// (0x30), `RequestLeaveCatalog` (0x41), `RequestRestock` (0x5A) and `RequestCollect` (0x4B).
const SALES: u32 = 0xB83A_0070;
const NEXT_SALES: u32 = 0xB83A_002F;
const PREV_SALES: u32 = 0xB83A_0030;
const LEAVE_CATALOG: u32 = 0xB83A_0041;
const RESTOCK: u32 = 0xB83A_005A;
const COLLECT: u32 = 0xB83A_004B;

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
    let program: Arc<Program> = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(legions_server_double::PLAYER);
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

/// Runs the commands for four frames.
fn run(client: &mut ac2_client::Client, server: &Shared, commands: Vec<Command>) {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    for i in 0..4 {
        let frame = if i == 0 { &input } else { &idle };
        let _: Vec<SessionEvent> = step(client, server, frame);
    }
}

/// Opens the vendor store with the storekeeper `VENDOR` (no sales), as the server does.
fn open_shopper(client: &mut ac2_client::Client, server: &Shared) {
    server.borrow_mut().open_store(
        StoreView {
            store: STORE_TEMPLATE,
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

/// Without a shopper open the paging and leave stubs send nothing (the VM's shopper check).
#[test]
fn store_pages_without_a_shopper_send_nothing() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    run(
        &mut client,
        &server,
        vec![
            Command::RequestSales(0),
            Command::NextSales,
            Command::PrevSales,
            Command::LeaveCatalog,
        ],
    );
    for function in [SALES, NEXT_SALES, PREV_SALES, LEAVE_CATALOG] {
        assert!(sent(&server, function).is_empty(), "{function:#x}");
    }
    assert!(client.take_script_problems().is_empty());
}

/// With the shopper open, the paging requests send the page index (`RequestSales`, an int) and the
/// storekeeper (a long int); next and previous send the storekeeper only.
#[test]
fn sales_paging_sends_the_page_and_the_storekeeper() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    open_shopper(&mut client, &server);
    run(&mut client, &server, vec![Command::RequestSales(3)]);
    assert_eq!(
        sent(&server, SALES),
        vec![pack_values(&[
            PackValue::Int(3),
            PackValue::LongInt(VENDOR)
        ])],
        "the RequestSales stub's PackBuffer"
    );
    run(&mut client, &server, vec![Command::NextSales]);
    assert_eq!(
        sent(&server, NEXT_SALES),
        vec![pack_values(&[PackValue::LongInt(VENDOR)])],
        "the RequestNextSales stub's PackBuffer"
    );
    run(&mut client, &server, vec![Command::PrevSales]);
    assert_eq!(
        sent(&server, PREV_SALES),
        vec![pack_values(&[PackValue::LongInt(VENDOR)])],
        "the RequestPrevSales stub's PackBuffer"
    );
    let requests = server.borrow().script_requests.clone();
    assert!(requests.contains(&ScriptRequest::StoreSales {
        index: 3,
        vendor: VENDOR
    }));
    assert!(requests.contains(&ScriptRequest::NextSales(VENDOR)));
    assert!(requests.contains(&ScriptRequest::PrevSales(VENDOR)));
    assert!(client.take_script_problems().is_empty());
}

/// The restock needs the item in the player's pack (the starter potions), or the VM refuses it.
/// `RequestRestock` packs the target (int), the quantity (int), the item (long int) and the
/// storekeeper; `RequestCollect` packs the profits flag (int), the target (int) and the
/// storekeeper. `RequestLeaveCatalog` packs the storekeeper.
#[test]
fn restock_collect_and_leave_catalog_send_their_values() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    open_shopper(&mut client, &server);
    run(
        &mut client,
        &server,
        vec![Command::RequestRestock {
            item: POTIONS,
            quantity: 2,
            target: 3,
        }],
    );
    assert_eq!(
        sent(&server, RESTOCK),
        vec![pack_values(&[
            PackValue::Int(3),
            PackValue::Int(2),
            PackValue::LongInt(POTIONS),
            PackValue::LongInt(VENDOR),
        ])],
        "the RequestRestock stub's PackBuffer"
    );
    run(
        &mut client,
        &server,
        vec![Command::RequestCollect {
            target: 3,
            profits: true,
        }],
    );
    assert_eq!(
        sent(&server, COLLECT),
        vec![pack_values(&[
            PackValue::Int(1),
            PackValue::Int(3),
            PackValue::LongInt(VENDOR),
        ])],
        "the RequestCollect stub's PackBuffer"
    );
    run(&mut client, &server, vec![Command::LeaveCatalog]);
    assert_eq!(
        sent(&server, LEAVE_CATALOG),
        vec![pack_values(&[PackValue::LongInt(VENDOR)])],
        "the RequestLeaveCatalog stub's PackBuffer"
    );
    let requests = server.borrow().script_requests.clone();
    assert!(requests.contains(&ScriptRequest::Restock {
        item: POTIONS,
        quantity: 2,
        target: 3,
        vendor: VENDOR,
    }));
    assert!(requests.contains(&ScriptRequest::Collect {
        target: 3,
        profits: 1,
        vendor: VENDOR,
    }));
    assert!(requests.contains(&ScriptRequest::LeaveCatalog(VENDOR)));
    assert!(client.take_script_problems().is_empty());
}
