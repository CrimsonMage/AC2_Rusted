//! The shop panel (`gmShopUI`) through the server double with the player's script on the VM
//! (`AC2_DAT_DIR`): the store the double opens shows its sales in the panel, a purchase pressed on
//! the panel asks the server through the client's commands, the confirmation's yes is answered
//! through the dialogs, and the item reaches the player's pack. The setup is the store script
//! test's (`store_script_7160.rs`).

mod common;

use std::sync::Arc;

use ac2_client::{shop_commands, Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{GameSale, StoreView, StringInfo};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::native::NativeObject;
use ac2_script::reference::{HeapObject, PackedEntry, PackedReference};
use ac2_script::vm::Program;
use ac2_script::{load_wlib, load_wstate, LEGIONS_WLIB_DID};
use ac2_ui::shop_notice;
use ac2_ui::{
    build_gameplay_hud, tokenize, ShopRequest, ShopSource, ShopUi, UiMessage, UiString, UiStrings,
    UiTree, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS,
};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::Spawn;

const ITEM: u64 = 0x8000_0002_0000_0021;
/// A StoreTemplate's WState DID in portal.dat (the `0x6F000000..` range).
const STORE_TEMPLATE: u32 = 0x6F00_33A3;
const VENDOR: u64 = 0x8000_0002_0000_0042;
/// The first object a purchase gives the player (the double's `PURCHASED_BASE` + 1).
const PURCHASED: u64 = 0x8000_0000_0000_0301;
/// The panel's press on a sale's row (select) and the purchase button's release.
const ROW_PRESS: i32 = 0x4000_0004;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
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

/// The notices the session raised in four frames after a command (or none), in order.
fn notices_after_events(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let first = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let idle = FrameInput::default();
    let mut events = step(client, server, &first);
    for _ in 0..4 {
        events.extend(step(client, server, &idle));
    }
    events
        .into_iter()
        .filter_map(|event| match event {
            SessionEvent::Notice(notice) => Some(notice),
            _ => None,
        })
        .collect()
}

/// What the StoreTemplate's WState holds (its sales: the item's DID, the price as a float and the
/// cost's DID), read from the DAT.
struct Template {
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
    Some(Template { sales })
}

/// The `StoreView` the server opens for the template: sale `k` is `ITEM + k`.
fn store_view(template: &Template) -> StoreView {
    StoreView {
        store: STORE_TEMPLATE,
        store_size: template.sales.len() as u32,
        position: 0,
        sales: template
            .sales
            .iter()
            .enumerate()
            .map(|(k, sale)| GameSale {
                item_did: sale.item_did,
                item: ITEM + k as u64,
                icon_did: 0,
                item_desc: StringInfo::default(),
                cost: sale.price,
                cost_did: sale.cost_did,
                sold: 0,
            })
            .collect(),
    }
}

/// The store the server opens (the template's sales, the storekeeper `VENDOR`).
fn open_template_store(
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) -> Option<Template> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let template = read_template(&ac2_db::ResourceStore::new(dataset))?;
    server
        .borrow_mut()
        .open_store(store_view(&template), VENDOR);
    Some(template)
}

/// The panel's strings for this test: a wire string as its literal, a UI string as its id and
/// variables (enough to read a price back).
struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        let vars: Vec<String> = string
            .vars
            .iter()
            .map(|(name, value)| format!("{name:08x}={value:?}"))
            .collect();
        format!("{:08x}({})", string.string_id, vars.join(","))
    }
}

impl ShopSource for Strings {
    fn render_wire(&self, info: &StringInfo) -> String {
        info.literal.clone().unwrap_or_default()
    }
}

fn release(tree: &UiTree, root: Option<usize>, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: root.and_then(|r| tree.child_recursive(r, tokenize(name))),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

/// The notices the panel takes, routed as the frames route them.
fn route(shop: &mut ShopUi, hud: &mut ac2_ui::GameplayHud, notices: &[GameplayNotice], now: f64) {
    for notice in notices {
        shop_notice(shop, &mut hud.tree, &mut hud.dialogs, &Strings, now, notice);
    }
}

#[test]
fn the_open_store_shows_its_sales_and_a_purchase_from_the_panel_reaches_the_pack() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    server.borrow_mut().give_starter_items();
    server.borrow_mut().give_money(350);
    let Some(template) = open_template_store(&server) else {
        return;
    };
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ui_store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let mut shop = hud.shop.take().expect("the shop panel");

    // The server's ViewStore (and its leaves) open the panel with the double's sales.
    let opened = notices_after_events(&mut client, &server, Vec::new());
    assert!(
        opened
            .iter()
            .any(|n| matches!(n, GameplayNotice::ViewStore(_))),
        "{opened:?}"
    );
    route(&mut shop, &mut hud, &opened, 0.0);
    assert!(shop.is_shown());
    let items: Vec<u64> = (0..template.sales.len() as u64).map(|k| ITEM + k).collect();
    assert_eq!(shop.sale_items(), items);
    let row = shop.sale_element(ITEM).expect("the first sale's row");
    let price = shop_text(&hud.tree, row, "CostQuantity");
    assert!(
        price.contains(&format!("{}", template.sales[0].price as i64)),
        "{price}"
    );

    // The player selects the first sale and presses the purchase button; the panel asks how
    // many it may buy and the commands go to the server. `GetMaxPurchaseQuantity` answers 0 for
    // this sale here (the double does not spawn the sale's item as an object, which the script's
    // answer reads), so the test gives the answer a retail server gives for an unlimited sale: one.
    let most = 1;
    let press = UiMessage {
        element_id: tokenize("Sale"),
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data: ROW_PRESS,
        ..UiMessage::default()
    };
    shop.message(
        &mut hud.tree,
        &Strings,
        &mut hud.dialogs,
        &press,
        0.0,
        &mut |_| most,
    );
    let field = shop.field().expect("the panel's root");
    let buy = release(&hud.tree, Some(field), "PurchaseButton");
    shop.message(
        &mut hud.tree,
        &Strings,
        &mut hud.dialogs,
        &buy,
        0.0,
        &mut |_| most,
    );
    let requests = shop.take_requests();
    assert_eq!(
        requests.last(),
        Some(&ShopRequest::Purchase {
            item: ITEM,
            quantity: 1,
            slot: 0,
        }),
        "{requests:?}"
    );
    let asked = notices_after_events(&mut client, &server, shop_commands(requests));
    route(&mut shop, &mut hud, &asked, 0.0);
    assert!(
        asked
            .iter()
            .any(|n| matches!(n, GameplayNotice::ConfirmPurchase { quantity: 1, .. })),
        "{asked:?}"
    );

    // The confirmation's yes answers the server, which gives the item.
    let (context, _) = hud.dialogs.shown().expect("the confirmation");
    let root = hud
        .tree
        .roots()
        .iter()
        .copied()
        .find(|&r| hud.tree.get(r).unwrap().id() == tokenize("ConfirmationField"))
        .expect("the confirmation dialog");
    let yes = release(&hud.tree, Some(root), "YesButton");
    hud.dialogs.message(&mut hud.tree, &yes);
    let outcomes = hud.dialogs.take_outcomes();
    assert_eq!(
        outcomes,
        [ac2_ui::DialogOutcome::Confirmation { context, yes: true }]
    );
    shop.dialog_outcomes(&outcomes);
    let answered = notices_after_events(&mut client, &server, shop_commands(shop.take_requests()));
    route(&mut shop, &mut hud, &answered, 0.0);
    let requests = server.borrow().script_requests.clone();
    assert!(
        matches!(
            requests.last(),
            Some(ac2_net::protocol::ScriptRequest::Purchase(_))
        ),
        "{requests:?}"
    );
    let frame = client.frame();
    assert!(
        frame.inventory.pack.items.contains(&PURCHASED),
        "{:?}",
        frame.inventory.pack.items
    );
    assert!(client.take_script_problems().is_empty());
}

fn shop_text(tree: &UiTree, row: usize, name: &str) -> String {
    let element = tree
        .child_recursive(row, tokenize(name))
        .unwrap_or_else(|| panic!("{name} under the sale"));
    tree.get(element).and_then(|e| e.text()).unwrap_or_default()
}
