//! The consignment's new-sale dialog (`gmShopNewSaleUI`) driven the way a player drives it, through
//! the HUD rig over the Legions server double with the player's script on the VM (`AC2_DAT_DIR`,
//! a GPU adapter). The player enters the storekeeper's consignment, presses the consignment's new
//! button (`CM_Store::SendNotice_InitNewSale`, an empty dialog), drops the sword on the sale slot,
//! types a price, presses Add Sale and answers the confirmation's Yes. The answer is
//! `gmCWInterface::RequestSell(item, quantity, price)`, which the script packs and sends as
//! `RequestSell` (`WM_Store` 0x43): the store, price, quantity, item and storekeeper, as the
//! double receives them.
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_net::protocol::{pack_values, ConsignmentList, PackValue, StoreCatalog, StoreView};
use ac2_ui::Modifiers;
use hud::Rig;
use legions_server_double::SWORD;

/// The storekeeper the shopper is open with.
const VENDOR: u64 = 0x8000_0002_0000_0042;
/// A StoreTemplate's WState DID in portal.dat, and a StoreGroup WState with its one store.
const STORE_TEMPLATE: u32 = 0x6F00_33A3;
const STORE_GROUP: u32 = 0x6F00_3558;
const GROUP_STORE: u32 = 0x6F00_321C;
/// `RequestSell`'s stub (WM_Store 0x43).
const SELL: u32 = 0xB83A_0043;
/// The price the player types and the quantity the sword's single drop gives.
const PRICE: &str = "10";
const QUANTITY: u32 = 1;

/// The packed bytes the VM sent for function `function`, in order.
fn sent(rig: &Rig, function: u32) -> Vec<Vec<u8>> {
    rig.server
        .borrow()
        .script_bytes
        .iter()
        .filter(|(f, _)| *f == function)
        .map(|(_, bytes)| bytes.clone())
        .collect()
}

/// The shopper open on the storekeeper's catalog, and the player's consignment entered: the
/// server's `ViewConsignment` answer, empty, opens the panel with its new button.
fn enter_consignment(rig: &mut Rig) {
    {
        let mut server = rig.server.borrow_mut();
        server.open_store(
            StoreView {
                store: STORE_TEMPLATE,
                store_size: 0,
                position: 0,
                sales: Vec::new(),
            },
            VENDOR,
        );
        server.open_catalog(
            StoreCatalog {
                group: STORE_GROUP,
                store_sizes: vec![(GROUP_STORE, 5)],
            },
            VENDOR,
        );
    }
    rig.idle(4);
    rig.command(Command::EnterConsignment);
    rig.idle(2);
    rig.server.borrow_mut().open_consignment(
        ConsignmentList {
            consignments: Vec::new(),
        },
        VENDOR,
    );
    rig.wait("the consignment's new button", |r| {
        r.view.ui_element_center("NewConsignmentButton").is_some()
    });
}

#[test]
fn a_sale_entered_in_the_new_sale_dialog_sends_the_request_sell_bytes() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.open_inventory();
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });

    enter_consignment(&mut rig);

    // The new button opens the sale dialog empty: its sale slot and price field are shown.
    let new = rig
        .view
        .ui_element_center("NewConsignmentButton")
        .expect("the new button");
    rig.click(new);
    rig.wait("the new-sale dialog", |r| {
        r.view.ui_element_center("NewSale").is_some()
    });

    // One sword dropped on the sale slot is the sale at once (no stack dialog for one).
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .expect("the sword");
    let slot = rig.view.ui_element_center("NewSale").expect("sale slot");
    rig.drag(from, slot);

    // The price field takes the typing once it is clicked.
    let price = rig
        .view
        .ui_element_center("NewSalePrice")
        .expect("price field");
    rig.click(price);
    let typed: Vec<(u16, u32)> = PRICE.encode_utf16().map(|c| (c, 0)).collect();
    rig.view.text_input(&[], &typed, Modifiers::default());
    rig.idle(2);

    // Add Sale asks the confirmation; its Yes sends the sale and hides the dialog.
    let add = rig
        .view
        .ui_element_center("AddSaleButton")
        .expect("add sale");
    rig.click(add);
    rig.wait("the sale's confirmation", |r| {
        r.view.ui_element_center("YesButton").is_some()
    });
    let yes = rig.view.ui_element_center("YesButton").expect("yes");
    rig.click(yes);
    rig.wait("the RequestSell", |r| !sent(r, SELL).is_empty());

    // The packed RequestSell: the store, price, quantity (ints), the item and the storekeeper
    // (long ints), in the stub's order.
    assert_eq!(
        sent(&rig, SELL),
        vec![pack_values(&[
            PackValue::Int(GROUP_STORE),
            PackValue::Int(PRICE.parse().unwrap()),
            PackValue::Int(QUANTITY),
            PackValue::LongInt(SWORD),
            PackValue::LongInt(VENDOR),
        ])],
        "the RequestSell stub's PackBuffer"
    );
    assert!(rig.view.ui_element_center("NewSale").is_none());
}
