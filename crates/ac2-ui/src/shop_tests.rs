//! `gmShopUI` on the real gameplay layout (`AC2_DAT_DIR`): the sales' list, a purchase through the
//! stack dialog and the confirmation, a drag from the shop into the pack, and the store's close.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::StringInfo;
use ac2_weenie::{Shop, ShopSale};

use crate::dialog::{DialogOutcome, DIALOG_CONFIRMATION};
use crate::mouse::UiMessage;
use crate::shop::{ShopRequest, ShopSource, ShopStackCall, ShopUi};
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, GameplayHud, StackUi, UiTree};

const ITEMS: [u64; 3] = [
    0x8000_0002_0000_0021,
    0x8000_0002_0000_0022,
    0x8000_0002_0000_0023,
];
const PACK_ITEM: u64 = 0x8000_0000_0000_0301;
const ROW_PRESS: i32 = 0x4000_0004;

/// Each string as its id and variables (`Int`, `UInt`, `Text`, a nested string), and a wire string
/// as its literal.
struct Strings;

fn render_var(value: &SkillVar) -> String {
    match value {
        SkillVar::UInt(n) => n.to_string(),
        SkillVar::Int(n) => n.to_string(),
        SkillVar::Text(t) => t.clone(),
        SkillVar::Float { value, .. } => value.to_string(),
        SkillVar::Info(inner) => Strings.render_string(inner),
        _ => "?".into(),
    }
}

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        let vars: Vec<String> = string
            .vars
            .iter()
            .map(|(name, value)| format!("{name:08x}={}", render_var(value)))
            .collect();
        format!("{:08x}({})", string.string_id, vars.join(","))
    }
}

impl ShopSource for Strings {
    fn render_wire(&self, info: &StringInfo) -> String {
        info.literal.clone().unwrap_or_default()
    }
}

fn literal(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}

/// Three sales: a whole price, a stock of two, and a fractional cost (the float path).
fn store() -> Shop {
    let sale = |k: usize, price: f32, stock: i32| ShopSale {
        ordinal: 0,
        item: ITEMS[k],
        item_did: 0x0600_1000 + k as u32,
        icon_did: 0x0600_2000 + k as u32,
        item_desc: literal(&format!("Item {k}")),
        cost_desc: StringInfo::default(),
        cost_icon: 0,
        cost_quantity: price,
        stock,
        status: 0,
    };
    Shop {
        id: 1,
        name: literal("Sheila's Wares"),
        desc: literal("Potions and such"),
        sorter: StringInfo::default(),
        portrait: 0,
        sales: vec![sale(0, 350.0, -1), sale(1, 25.0, 2), sale(2, 12.5, -1)],
        position: 0,
        store_size: 3,
    }
}

/// The gameplay HUD with its shop panel taken out, so the panel and the HUD can both be borrowed.
fn setup() -> Option<(GameplayHud, ShopUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let shop = hud.shop.take().expect("the shop panel");
    Some((hud, shop))
}

/// A button's release (`ButtonMsg_mouse_release_on`), with its element when the panel has one.
fn release(tree: &UiTree, root: Option<usize>, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: root.and_then(|r| tree.child_recursive(r, tokenize(name))),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

/// A press on the sale in `row` (`ElementMsg_mouse_press`, select).
fn press_sale(row: usize) -> UiMessage {
    UiMessage {
        element_id: tokenize("Sale"),
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data: ROW_PRESS,
        ..UiMessage::default()
    }
}

fn text_of(tree: &UiTree, row: usize, name: &str) -> String {
    let element = tree
        .child_recursive(row, tokenize(name))
        .unwrap_or_else(|| panic!("{name} under the sale"));
    tree.get(element).and_then(|e| e.text()).unwrap_or_default()
}

fn shop_text(tree: &UiTree, field: usize, name: &str) -> String {
    let element = tree.child_recursive(field, tokenize(name)).unwrap();
    tree.get(element).and_then(|e| e.text()).unwrap_or_default()
}

fn no_max(_: u64) -> u32 {
    1
}

#[test]
fn view_store_lists_the_sales_in_order_with_their_prices() {
    let Some((mut hud, mut shop)) = setup() else {
        return;
    };
    let tree = &mut hud.tree;
    let s = Strings;
    shop.recv_view_store(tree, &mut hud.dialogs, &s, &store(), 0.0);
    assert!(shop.is_shown());
    let field = shop.field().expect("the panel's root");
    assert_eq!(shop.sale_items(), ITEMS, "the sales in the order they came");
    assert_eq!(shop_text(tree, field, "ShopName"), "Sheila's Wares");
    assert_eq!(shop_text(tree, field, "ShopDesc"), "Potions and such");
    let price = |quantity: SkillVar| {
        s.render_string(&UiString::ui(
            tokenize("ID_Store_Quantity"),
            vec![(tokenize("Quantity"), quantity)],
        ))
    };
    let rows: Vec<usize> = ITEMS
        .iter()
        .map(|item| shop.sale_element(*item).expect("the sale's row"))
        .collect();
    assert_eq!(
        text_of(tree, rows[0], "CostQuantity"),
        price(SkillVar::Int(350))
    );
    assert_eq!(
        text_of(tree, rows[2], "CostQuantity"),
        price(SkillVar::Float {
            value: 12.5,
            precision: 2
        })
    );
    assert_eq!(text_of(tree, rows[1], "ProductDesc"), "Item 1");
    // Two in stock, unlimited, and the sale's stock text.
    assert_eq!(
        text_of(tree, rows[1], "StockQuantity"),
        s.render_string(&UiString::ui(
            tokenize("ID_Store_Quantity"),
            vec![(tokenize("Quantity"), SkillVar::Int(2))],
        ))
    );
    assert_eq!(
        text_of(tree, rows[0], "StockQuantity"),
        s.render_string(&UiString::ui(tokenize("ID_Store_Unlimited"), Vec::new()))
    );
}

#[test]
fn a_purchase_asks_the_stack_for_a_stack_and_then_buys_it() {
    let Some((mut hud, mut shop)) = setup() else {
        return;
    };
    let s = Strings;
    shop.recv_view_store(&mut hud.tree, &mut hud.dialogs, &s, &store(), 0.0);
    let row = shop.sale_element(ITEMS[0]).unwrap();
    // The sale is selected (the selection goes to the player) and the purchase asks for five.
    shop.message(
        &mut hud.tree,
        &s,
        &mut hud.dialogs,
        &press_sale(row),
        0.0,
        &mut |_| 5,
    );
    let field = shop.field().unwrap();
    let buy = release(&hud.tree, Some(field), "PurchaseButton");
    shop.message(&mut hud.tree, &s, &mut hud.dialogs, &buy, 0.0, &mut |_| 5);
    let calls = shop.take_stack_calls();
    let [ShopStackCall::Init { context, text, max }] = &calls[..] else {
        panic!("the stack dialog: {calls:?}");
    };
    let (context, max) = (*context, *max);
    assert_eq!(max, 5);
    assert_eq!(
        *text,
        s.render_string(&UiString::ui(tokenize("ID_Store_ShopStack"), Vec::new()))
    );
    // The stack dialog takes its default and its most, then all (the toggle) and OK.
    let stack: &mut StackUi = hud.stack.as_mut().expect("the stack dialog");
    stack.init(&mut hud.tree, context, text, 0, max, 0, 0);
    let field = stack.field().unwrap();
    let toggle = hud
        .tree
        .child_recursive(field, tokenize("AmountToggleButton"))
        .unwrap();
    hud.tree.set_bool(toggle, BUTTON_HIGHLIGHTED, false);
    let all = release(&hud.tree, Some(field), "AmountToggleButton");
    stack.message(&mut hud.tree, &all, &s);
    let ok = release(&hud.tree, Some(field), "StackOkButton");
    let done = stack
        .message(&mut hud.tree, &ok, &s)
        .expect("the stack's answer");
    shop.recv_stack_ui_done(done.context, done.amount);
    assert_eq!(
        shop.take_requests(),
        [
            ShopRequest::Select(ITEMS[0]),
            ShopRequest::Purchase {
                item: ITEMS[0],
                quantity: 5,
                slot: 0,
            },
        ]
    );
}

#[test]
fn a_single_item_is_bought_at_once_and_a_sold_out_one_is_refused() {
    let Some((mut hud, mut shop)) = setup() else {
        return;
    };
    let s = Strings;
    shop.recv_view_store(&mut hud.tree, &mut hud.dialogs, &s, &store(), 0.0);
    let row = shop.sale_element(ITEMS[2]).unwrap();
    shop.message(
        &mut hud.tree,
        &s,
        &mut hud.dialogs,
        &press_sale(row),
        0.0,
        &mut no_max,
    );
    let field = shop.field().unwrap();
    let buy = release(&hud.tree, Some(field), "PurchaseButton");
    shop.message(&mut hud.tree, &s, &mut hud.dialogs, &buy, 0.0, &mut no_max);
    assert!(shop.take_stack_calls().is_empty());
    assert_eq!(
        shop.take_requests().last(),
        Some(&ShopRequest::Purchase {
            item: ITEMS[2],
            quantity: 1,
            slot: 0,
        })
    );
    // Nothing left to buy: the purchase is refused with the reason in the chat.
    shop.message(&mut hud.tree, &s, &mut hud.dialogs, &buy, 0.0, &mut |_| 0);
    assert!(shop.take_requests().is_empty());
    assert_eq!(
        shop.take_messages(),
        [s.render_string(&UiString::ui(tokenize("ID_Store_CantAfford"), Vec::new()))]
    );
}

#[test]
fn confirm_purchase_shows_the_confirmation_and_yes_sends_purchase_confirmed() {
    let Some((mut hud, mut shop)) = setup() else {
        return;
    };
    let s = Strings;
    shop.recv_view_store(&mut hud.tree, &mut hud.dialogs, &s, &store(), 0.0);
    shop.recv_confirm_purchase(
        &mut hud.tree,
        &mut hud.dialogs,
        &s,
        &literal("Item 0"),
        1,
        &literal("gold"),
        350,
    );
    let (context, kind) = hud.dialogs.shown().expect("the confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert_eq!(shop.confirm_context(), context);
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
        [DialogOutcome::Confirmation { context, yes: true }]
    );
    shop.dialog_outcomes(&outcomes);
    assert_eq!(shop.take_requests(), [ShopRequest::PurchaseConfirmed(true)]);
}

#[test]
fn a_drag_from_the_shop_buys_into_the_slot_and_a_pack_item_is_not_bought() {
    let Some((mut hud, mut shop)) = setup() else {
        return;
    };
    let s = Strings;
    shop.recv_view_store(&mut hud.tree, &mut hud.dialogs, &s, &store(), 0.0);
    let context = shop.context;
    // A sale dragged from the shop's context into pack slot 7 is bought there.
    shop.recv_handle_drag_from(&mut hud.tree, &s, ITEMS[1], 7, context, &mut no_max);
    assert_eq!(
        shop.take_requests(),
        [ShopRequest::Purchase {
            item: ITEMS[1],
            quantity: 1,
            slot: 7,
        }]
    );
    // An item from the pack is not one of the shop's sales: the store buys nothing and sells
    // nothing (`gmShopUI` has no sell; `RequestSell` is the consignment's).
    shop.recv_handle_drag_from(&mut hud.tree, &s, PACK_ITEM, 3, context, &mut no_max);
    assert!(shop.take_requests().is_empty());
    assert!(shop.take_stack_calls().is_empty());
}

#[test]
fn leave_store_closes_the_panel_and_a_sold_out_sale_is_unavailable() {
    let Some((mut hud, mut shop)) = setup() else {
        return;
    };
    let s = Strings;
    shop.recv_view_store(&mut hud.tree, &mut hud.dialogs, &s, &store(), 0.0);
    shop.recv_update_sale(&mut hud.tree, &s, ITEMS[1], 0);
    let row = shop.sale_element(ITEMS[1]).unwrap();
    assert!(hud.tree.bool_attribute(row, tokenize("unavailable")));
    let field = shop.field().unwrap();
    shop.recv_leave_store(&mut hud.tree, &mut hud.dialogs, 1.0);
    assert!(!shop.is_shown());
    assert!(!hud.tree.get(field).unwrap().visible());
}
