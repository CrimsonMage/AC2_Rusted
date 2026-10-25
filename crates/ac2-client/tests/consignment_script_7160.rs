//! The consignment calls with the player's script on the VM (`AC2_DAT_DIR`). Entering and leaving
//! the consignment are the IFuns' stubs, which the VM packs (the storekeeper, one long int) and the
//! session sends, but only with a shopper open (the store the storekeeper opened). The sale itself
//! (`RequestSell`) is refused outside a consignment: the VM's sell body raises a message and sends
//! nothing. The consignment's answers (`ViewConsignment`, `UpdateConsignment`) carry an `RList` of
//! `Consignment`s and raise their notices with the shop built from them.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{
    pack_values, Consignment, ConsignmentList, GameSale, PackValue, ScriptRequest, StoreCatalog,
    StoreView,
};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::native::NativeObject;
use ac2_script::reference::{HeapObject, PackedEntry};
use ac2_script::vm::Program;
use ac2_script::{load_wlib, load_wstate, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, PLAYER, POTIONS};

/// The items the consignments sell (the server's IIDs for them).
const ITEM: u64 = 0x8000_0002_0000_0051;
/// The StoreTemplate the shopper opens with (portal.dat WState).
const STORE_TEMPLATE: u32 = 0x6F00_33A3;
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

/// The first sale of the store template the shopper opens with (`SaleTemplate`, package 0x380F:
/// slot 1 the item's DID, slot 3 the price, slot 5 the cost's DID, as the store test reads them),
/// as a sale the server would consign.
fn template_sale(item: u64) -> Option<GameSale> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset);
    let wstate = load_wstate(&store, DataId::new(STORE_TEMPLATE)).ok()??;
    let reference = wstate.references.as_ref().ok()?;
    let heap = |id: u32| -> Option<&HeapObject> {
        let index = reference.ids.iter().position(|i| *i == id)?;
        match reference.entries.get(index)? {
            PackedEntry::Heap { object, .. } => Some(object),
            _ => None,
        }
    };
    let root = heap(*reference.ids.first()?)?;
    let NativeObject::List(list) = heap(root.slots[4])?.native.as_ref()? else {
        return None;
    };
    let sale = heap(*list.items.first()? as u32)?;
    Some(GameSale {
        item_did: sale.slots[1],
        item,
        icon_did: 0,
        item_desc: Default::default(),
        cost: f32::from_bits(sale.slots[3]),
        cost_did: sale.slots[5],
        sold: 0,
    })
}

/// `ViewConsignment` (`WM_Store` 0x53) opens the storekeeper's consignments: the shop holds one
/// sale per consignment with stock, in the consignment's ordinal, stock and expiry; one with no
/// stock is left out. `UpdateConsignment` (0x57) replaces them.
#[test]
fn view_and_update_consignment_show_the_consigned_sales() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let (Some(first), Some(second)) = (template_sale(ITEM), template_sale(ITEM + 1)) else {
        panic!("the store template's sale");
    };
    open_shopper(&mut client, &server);
    let consignments = ConsignmentList {
        consignments: vec![
            Consignment {
                sale: first.clone(),
                ordinal: 0,
                stock: 2,
                expired: false,
            },
            Consignment {
                sale: second.clone(),
                ordinal: 1,
                stock: 1,
                expired: true,
            },
            Consignment {
                sale: first.clone(),
                ordinal: 2,
                stock: 0,
                expired: false,
            },
        ],
    };
    server.borrow_mut().open_consignment(consignments, VENDOR);
    let notices = run(&mut client, &server, Vec::new());
    let shop = notices
        .iter()
        .find_map(|n| match n {
            GameplayNotice::ViewConsignment(shop) => Some(shop.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("ViewConsignment: {notices:?}"));
    let shown: Vec<(u32, u64, i32, u32)> = shop
        .sales
        .iter()
        .map(|s| (s.ordinal, s.item, s.stock, s.status))
        .collect();
    assert_eq!(
        shown,
        vec![(0, ITEM, 2, 0), (1, ITEM + 1, 1, 2)],
        "{shop:?}"
    );
    assert_eq!(shop.sales[0].item_did, first.item_did);
    assert_eq!(shop.sales[0].cost_quantity, first.cost);

    server.borrow_mut().update_consignment(
        ConsignmentList {
            consignments: vec![Consignment {
                sale: second,
                ordinal: 0,
                stock: 3,
                expired: false,
            }],
        },
        VENDOR,
    );
    let notices = run(&mut client, &server, Vec::new());
    let shop = notices
        .iter()
        .find_map(|n| match n {
            GameplayNotice::UpdateConsignment(shop) => Some(shop.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("UpdateConsignment: {notices:?}"));
    let shown: Vec<(u32, u64, i32)> = shop
        .sales
        .iter()
        .map(|s| (s.ordinal, s.item, s.stock))
        .collect();
    assert_eq!(shown, vec![(0, ITEM + 1, 3)]);
    assert!(client.take_script_problems().is_empty());
}

/// A `StoreGroup` WState in portal.dat (package 0x3858) and one of its stores.
const STORE_GROUP: u32 = 0x6F00_3558;
const GROUP_STORE: u32 = 0x6F00_321C;

/// `ViewCatalog` (`WM_Store` 0x52) opens the storekeeper's catalog: the group's stores, each
/// with the size the hash gives it (`gUIShop::SetStoreSize`; a store the hash does not name shows
/// 0), after the open store, consignment and catalog are closed.
#[test]
fn view_catalog_lists_the_groups_stores_with_their_sizes() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    open_shopper(&mut client, &server);
    server.borrow_mut().open_catalog(
        StoreCatalog {
            group: STORE_GROUP,
            store_sizes: vec![(GROUP_STORE, 5)],
        },
        VENDOR,
    );
    let notices = run(&mut client, &server, Vec::new());
    let leaves: Vec<_> = notices
        .iter()
        .filter(|n| {
            matches!(
                n,
                GameplayNotice::LeaveStore
                    | GameplayNotice::LeaveConsignment
                    | GameplayNotice::LeaveCatalog
            )
        })
        .collect();
    assert_eq!(leaves.len(), 3, "{notices:?}");
    let shops = notices
        .iter()
        .find_map(|n| match n {
            GameplayNotice::ViewCatalog { shops, .. } => Some(shops.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("ViewCatalog: {notices:?}"));
    let sized = shops
        .iter()
        .find(|shop| shop.id == GROUP_STORE)
        .expect("the group's store");
    assert_eq!(sized.store_size, 5);
    assert!(shops
        .iter()
        .filter(|shop| shop.id != GROUP_STORE)
        .all(|shop| shop.store_size == 0));
    assert!(client.take_script_problems().is_empty());
}
