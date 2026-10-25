//! The store's calls with the player's script on the VM (`AC2_DAT_DIR`). A store request is the
//! IFun's stub, which the VM's `CStoreSystem` sends only with a shopper open (the server's
//! `ViewStore` opens one; `CStoreSystem` 0x10 and 0x1C), so a request before that goes nowhere.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_db::Ac2PropertyValue;
use ac2_net::protocol::{GameSale, StoreView, StringInfo};
use ac2_script::data::entity::string_info_property;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::native::NativeObject;
use ac2_script::reference::{HeapObject, PackedEntry, PackedReference};
use ac2_script::vm::{Program, ScriptData};
use ac2_script::wire::string_info_to_net;
use ac2_script::{load_wlib, load_wstate, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::Spawn;

const ITEM: u64 = 0x8000_0002_0000_0021;
/// A StoreTemplate's WState DID in portal.dat (the `0x6F000000..` range).
const STORE_TEMPLATE: u32 = 0x6F00_33A3;
const VENDOR: u64 = 0x8000_0002_0000_0042;
/// The first object a purchase gives the player (the double's `PURCHASED_BASE` + 1).
const PURCHASED: u64 = 0x8000_0000_0000_0301;
/// The entity property holding its name (a `StringInfo`), as the sale build's `InqStringInfo` asks.
const NAME_PROPERTY: u32 = 0x4100_0143;
/// The entity property holding its icon (a DID), as the sale build's `InqDataID` asks.
const ICON_PROPERTY: u32 = 0x4100_020A;

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

/// What the StoreTemplate's WState holds, read from the DAT: its name and description (the
/// StringInfos its functions 7 and 0x14 return: slots 2 and 3) and its sales (the list in slot 4
/// of `SaleTemplate`s: slot 1 the item's DID, slot 3 the price as a float, slot 5 the cost's DID).
/// The slot meanings are inferred from the template's shapes; the server's conversion of a
/// template sale to a `GameSaleProfile` is not traced.
struct Template {
    name: StringInfo,
    desc: StringInfo,
    sales: Vec<TemplateSale>,
}

struct TemplateSale {
    item_did: u32,
    price: f32,
    cost_did: u32,
}

fn heap(reference: &PackedReference, id: u32) -> Option<&HeapObject> {
    let index = reference.ids.iter().position(|i| *i == id)?;
    match reference.entries.get(index)? {
        PackedEntry::Heap { object, .. } => Some(object),
        _ => None,
    }
}

fn read_template(store: &ac2_db::ResourceStore) -> Option<Template> {
    let wstate = load_wstate(store, DataId::new(STORE_TEMPLATE)).ok()??;
    let reference = wstate.references.as_ref().ok()?;
    let root = heap(reference, *reference.ids.first()?)?;
    let text = |id: u32| match heap(reference, id)?.native.as_ref()? {
        NativeObject::StringInfo(text) => Some(string_info_to_net(text)),
        _ => None,
    };
    let list = match heap(reference, root.slots[4])?.native.as_ref()? {
        NativeObject::List(list) => list,
        _ => return None,
    };
    let sales = list
        .items
        .iter()
        .map(|id| {
            let sale = heap(reference, *id as u32)?;
            (sale.pkg_id == 0x380F && sale.slots.len() == 7).then(|| TemplateSale {
                item_did: sale.slots[1],
                price: f32::from_bits(sale.slots[3]),
                cost_did: sale.slots[5],
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Template {
        name: text(root.slots[2])?,
        desc: text(root.slots[3])?,
        sales,
    })
}

/// An entity's name and icon as the DAT holds them: the `StringInfo` of its name property and its
/// icon DID (read through the script data source, as the client's inquiries read them).
fn entity_look(data: &DatScriptData, did: u32) -> (StringInfo, u32) {
    let props: Vec<Ac2PropertyValue> = data
        .entity_properties(did)
        .map(|p| p.to_vec())
        .unwrap_or_default();
    let name = props
        .iter()
        .find(|p| p.name == NAME_PROPERTY)
        .and_then(|p| string_info_property(&p.bytes))
        .map(|info| string_info_to_net(&info))
        .unwrap_or_default();
    let icon = props
        .iter()
        .find(|p| p.name == ICON_PROPERTY)
        .and_then(|p| p.word())
        .unwrap_or(0);
    (name, icon)
}

/// The `StoreView` the server opens for the template: its sales in list order, the item IIDs
/// the server gives them (`ITEM` and up, sale `k` is `ITEM + k`), and the store's size and
/// position. Member 2 is the store's sale count and member 3 the first sale shown, 0 when the
/// shop opens (`gUIShop::SetStoreSize` and `SetPosition` feed `gmShopUI::UpdatePosition`, whose
/// next button is off once `position + 7 >= size`).
/// The sale's item description and icon are the item's own name and icon (the server's sale
/// profile members 0x1B and 0x2D), read from the item's entity record.
fn store_view(template: &Template, data: &DatScriptData) -> StoreView {
    StoreView {
        store: STORE_TEMPLATE,
        store_size: template.sales.len() as u32,
        position: 0,
        sales: template
            .sales
            .iter()
            .enumerate()
            .map(|(k, sale)| {
                let (item_desc, icon_did) = entity_look(data, sale.item_did);
                GameSale {
                    item_did: sale.item_did,
                    item: ITEM + k as u64,
                    icon_did,
                    item_desc,
                    cost: sale.price,
                    cost_did: sale.cost_did,
                    sold: 0,
                }
            })
            .collect(),
    }
}

/// The script data source over the test dataset, with its tables preloaded (the worker's part).
fn script_data() -> Option<DatScriptData> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let data = DatScriptData::new(dataset.clone(), ac2_db::ResourceStore::new(dataset));
    data.preload_tables();
    Some(data)
}

/// The template as the DAT holds it (`None` without `AC2_DAT_DIR`).
fn load_template() -> Option<Template> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    read_template(&ac2_db::ResourceStore::new(dataset))
}

/// The store the server opens: `store_view` from the template's WState, with the storekeeper.
fn open_template_store(
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) -> Option<Template> {
    let template = load_template()?;
    let data = script_data()?;
    server
        .borrow_mut()
        .open_store(store_view(&template, &data), VENDOR);
    Some(template)
}

/// The server's ViewStore for a StoreTemplate, run on the script: the shop build takes the
/// StoreView's sales and the template's name and description, and the notice carries them with
/// each sale's stock. The stock is the `GameSaleProfile`'s constant (`SaleProfile` 0x28 answers
/// -1, unlimited), so every sale reads -1.
#[test]
fn view_store_opens_the_template_store_with_its_sales() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let Some(template) = open_template_store(&server) else {
        return;
    };
    let notices = notices_after(&mut client, &server);
    // The shop body first closes the open consignment, store and catalog (`LeaveConsignment`,
    // `LeaveStore`, `LeaveConsignment`, `LeaveCatalog`), then sends `ViewStore`.
    assert_eq!(notices.len(), 5, "{notices:?}");
    assert_eq!(
        notices[..4],
        [
            GameplayNotice::LeaveConsignment,
            GameplayNotice::LeaveStore,
            GameplayNotice::LeaveConsignment,
            GameplayNotice::LeaveCatalog,
        ]
    );
    let data = script_data().unwrap();
    let GameplayNotice::ViewStore(shop) = &notices[4] else {
        panic!("{notices:?}");
    };
    assert_eq!(shop.name, template.name);
    assert_eq!(shop.desc, template.desc);
    assert_eq!(shop.sales.len(), template.sales.len());
    // The view's size (its sale count) and position (zero), as the double sends them.
    assert_eq!(
        (shop.store_size, shop.position),
        (template.sales.len() as i32, 0)
    );
    for (k, (sale, t)) in shop.sales.iter().zip(&template.sales).enumerate() {
        // The store build sets no ordinal (`UIShopSale::SetOrdinal` is the consignment build's),
        // so every sale is 0 and `UIShop::AddSale` keeps them in list order.
        assert_eq!(sale.ordinal, 0);
        assert_eq!(sale.item, ITEM + k as u64);
        assert_eq!(sale.item_did, t.item_did);
        assert_eq!(sale.cost_quantity, t.price);
        assert_eq!(sale.stock, -1);
        assert_eq!(sale.status, 0);
        // The item's name and icon (from its entity record), and the cost's (the cost entity,
        // read by the sale build's `InqEntityDescFromDataID`, `InqStringInfo` and `InqDataID`).
        let (name, icon) = entity_look(&data, t.item_did);
        let (cost_name, cost_icon) = entity_look(&data, t.cost_did);
        assert_eq!(sale.item_desc, name, "sale {k}");
        assert_eq!(sale.icon_did, icon, "sale {k}");
        assert_ne!(icon, 0, "sale {k} has an icon");
        assert_eq!(sale.cost_desc, cost_name, "sale {k}");
        assert_eq!(sale.cost_icon, cost_icon, "sale {k}");
        assert!(
            !rendered(&data, &sale.item_desc).is_empty(),
            "sale {k} has a name"
        );
        assert!(
            !rendered(&data, &sale.cost_desc).is_empty(),
            "sale {k} has a cost name"
        );
    }
    assert!(client.take_script_problems().is_empty());
}

/// The text a `StringInfo` renders to through its string table (empty without one).
fn rendered(data: &DatScriptData, info: &StringInfo) -> String {
    let native = ac2_script::wire::string_info_from_net(info);
    let Some(table) = data.string_table(native.table_id) else {
        return String::new();
    };
    data.render_string(&table, &native)
        .map(|text| String::from_utf16_lossy(&text))
        .unwrap_or_default()
}

/// The server changes the open store (`UpdateStore`, `CPlayer` 0x1E2): the body needs the
/// shopper `ViewStore` opened, then rebuilds the shop from the new `StoreView` and raises
/// `UpdateStore` with it (no leave notices: that prologue belongs to `ViewStore`).
#[test]
fn update_store_replaces_the_open_shops_sales() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let Some(template) = open_template_store(&server) else {
        return;
    };
    notices_after(&mut client, &server);
    let mut view = store_view(&template, &script_data().unwrap());
    view.sales.pop();
    server.borrow_mut().update_store(view, VENDOR);
    let notices = notices_after(&mut client, &server);
    let [GameplayNotice::UpdateStore(shop)] = &notices[..] else {
        panic!("{notices:?}");
    };
    assert_eq!(shop.sales.len(), template.sales.len() - 1);
    assert_eq!(shop.sales[0].item, ITEM);
    assert_eq!(shop.sales[0].item_did, template.sales[0].item_did);
    assert!(client.take_script_problems().is_empty());
}

/// The notices and the frame's money after a command, four frames on.
fn command_notices(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut events = step(client, server, &input);
    events.extend(notices_after_events(client, server));
    events
        .into_iter()
        .filter_map(|event| match event {
            SessionEvent::Notice(notice) => Some(notice),
            _ => None,
        })
        .collect()
}

fn notices_after_events(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    (0..4).flat_map(|_| step(client, server, &idle)).collect()
}

/// The player leaves the open store. The request goes out (`RequestLeaveStore`, the storekeeper's
/// long); the leave IFun has already closed the shopper (`CPlayer` 0x1D5 runs the store's close,
/// `CPlayer` 0x1EB's shopper goes to -1), so the storekeeper's `LeaveStore` answer finds no
/// shopper (`CPlayer` 0x1DD -> `CStoreSystem` 0x14 returns) and raises no notice.
#[test]
fn leaving_the_open_store_sends_the_storekeeper_and_closes_the_shop() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let Some(_) = open_template_store(&server) else {
        return;
    };
    notices_after(&mut client, &server);
    let notices = command_notices(&mut client, &server, vec![Command::LeaveStore]);
    assert_eq!(notices, [], "the open store closes without a notice");
    assert_eq!(
        server.borrow().script_requests,
        [ac2_net::protocol::ScriptRequest::LeaveStore(VENDOR)]
    );
    assert!(client.take_script_problems().is_empty());
}

/// A purchase with the price covered: `PurchaseItem` raises `ConfirmPurchase` locally (the
/// product, the quantity, the trade name and the price from the sale), `PurchaseConfirmed`
/// sends the `TransactionBlob` (item IID, quantity, storekeeper), and the server takes the price
/// off the player's money and gives the item. The money is the server's (`give_money`), so
/// its answer is the double's own rule (see `answer_purchase`).
#[test]
fn purchase_confirmed_takes_the_price_and_gives_the_item() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    server.borrow_mut().give_starter_items();
    server.borrow_mut().give_money(350);
    let Some(template) = open_template_store(&server) else {
        return;
    };
    notices_after(&mut client, &server);
    let price = template.sales[0].price as u32;
    let notices = command_notices(
        &mut client,
        &server,
        vec![Command::PurchaseItem {
            item: ITEM,
            quantity: 1,
            slot: 0,
        }],
    );
    assert!(
        matches!(
            notices[..],
            [GameplayNotice::ConfirmPurchase { quantity: 1, cost, .. }] if cost == price
        ),
        "{notices:?}"
    );
    assert!(server.borrow().script_requests.is_empty());
    let notices = command_notices(&mut client, &server, vec![Command::PurchaseConfirmed(true)]);
    let requests = server.borrow().script_requests.clone();
    let [ac2_net::protocol::ScriptRequest::Purchase(blob)] = &requests[..] else {
        panic!("{requests:?}");
    };
    let purchase = ac2_net::protocol::Purchase::from_reference(blob).unwrap();
    assert_eq!(
        purchase,
        ac2_net::protocol::Purchase {
            item: ITEM,
            quantity: 1,
            vendor: VENDOR,
        }
    );
    assert!(
        notices.contains(&GameplayNotice::UpdateMoney(600 - price)),
        "{notices:?}"
    );
    assert_eq!(client.frame().inventory.money, 600 - price as i32);
    // The item the server gave: the double's purchased object, listed in the player's pack.
    let frame = client.frame();
    assert!(
        frame.inventory.pack.items.contains(&PURCHASED),
        "{:?}",
        frame.inventory.pack.items
    );
    assert!(client.take_script_problems().is_empty());
}

/// A sell of an item the player owns (the starter potions) to the open vendor store is refused
/// by the sell body before its request, as retail does: `gmCWInterface::RequestSell` is raised
/// only by the consignment's new-sale dialog (`gmShopNewSaleUI`, admin 0x41A7B0), not by the
/// vendor shop's drag. The body's first failing check is the shopper context's buy list (slot 0
/// must be a StoreGroup, `GetPackageReference(slot 0, StoreGroup)`), which only the server's
/// 0xB83A0052 flow sets (`CPlayer` 0x1F3, body 0x117748, which first closes the consignment).
/// No `RequestSell` goes out. (An item the player does not own sends nothing at all.)
#[test]
fn sell_to_the_open_store_is_refused_without_a_request() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    server.borrow_mut().give_starter_items();
    let Some(_) = open_template_store(&server) else {
        return;
    };
    notices_after(&mut client, &server);
    let notices = command_notices(
        &mut client,
        &server,
        vec![Command::RequestSell {
            item: legions_server_double::POTIONS,
            quantity: 1,
            price: 10,
        }],
    );
    assert!(
        matches!(notices[..], [GameplayNotice::DisplayStringInfo { .. }]),
        "{notices:?}"
    );
    assert!(server.borrow().script_requests.is_empty());
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
