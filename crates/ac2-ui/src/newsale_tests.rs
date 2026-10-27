//! `gmShopNewSaleUI` on the real gameplay and toolbar layouts (`AC2_DAT_DIR`): the item the dialog
//! opens with, the drop of an item on its slot (one is chosen at once, more ask how many), the
//! price checks and the fee's confirmation, the close button and the global loop.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::dialog::{DialogOutcome, DIALOG_CONFIRMATION, DIALOG_MESSAGE};
use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::mouse::UiMessage;
use crate::newsale::{consignment_fee, parse_price, NewSaleRequest, NewSaleUi};
use crate::shop::ShopStackCall;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, BackpackUi, GameplayHud, InventorySource, ItemSlot, UiTree};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;

/// The player's pack: five potions (a stack) and a sword.
struct Pack;

impl InventorySource for Pack {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, name, quantity, stackable) = match item {
            POTIONS => (0x0600_1001, "Healing Potions", 5, true),
            SWORD => (0x0600_1002, "Practice Sword", 1, false),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            name: name.into(),
            quantity,
            stackable,
            max_stack: if stackable { 10 } else { 0 },
            ..ItemSlot::default()
        })
    }

    fn contents_info(&self, container: u64, _: u32) -> (Vec<u64>, u32, u32) {
        match container {
            PLAYER => (vec![POTIONS, SWORD], 0, 24),
            _ => (Vec::new(), 0, 0),
        }
    }

    fn max_container_size(&self, _: u64) -> u32 {
        24
    }

    fn selection(&self) -> u64 {
        0
    }

    fn player(&self) -> u64 {
        PLAYER
    }
}

/// `ID(name=value)` for each rendered string.
impl UiStrings for Pack {
    fn render_string(&self, string: &UiString) -> String {
        let vars: Vec<String> = string
            .vars
            .iter()
            .map(|(name, value)| match value {
                SkillVar::UInt(n) => format!("{name}={n}"),
                SkillVar::Text(t) => format!("{name}={t}"),
                _ => format!("{name}=?"),
            })
            .collect();
        format!("{}({})", string.string_id, vars.join(","))
    }
}

fn setup() -> Option<(GameplayHud, NewSaleUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.new_sale.take().expect("the new sale panel");
    Some((hud, panel))
}

fn release(name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

fn text_of(tree: &UiTree, field: usize, name: &str) -> Option<String> {
    let element = tree.child_recursive(field, tokenize(name))?;
    tree.get(element)?.text()
}

fn set_price(tree: &mut UiTree, field: usize, text: &str) {
    let price = tree
        .child_recursive(field, tokenize("NewSalePrice"))
        .expect("the price field");
    tree.set_text(price, text);
}

fn open(hud: &mut GameplayHud, panel: &mut NewSaleUi, item: u64, quantity: u32) {
    panel.recv_init_new_sale(&mut hud.tree, &mut hud.dialogs, &Pack, item, quantity, 0.0);
}

#[test]
fn opening_with_an_item_shows_its_name_and_clears_the_price() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, POTIONS, 5);
    let field = panel.field().expect("the dialog's root");
    assert!(panel.is_shown());
    assert_eq!(panel.chosen_item(), POTIONS);
    assert_eq!(panel.chosen_quantity(), 5);
    assert_eq!(
        text_of(&hud.tree, field, "NewSaleDesc").as_deref(),
        Some("Healing Potions")
    );
    assert_eq!(
        text_of(&hud.tree, field, "NewSalePrice").as_deref(),
        Some("")
    );
}

#[test]
fn an_empty_dialog_clears_the_item_and_its_name() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, POTIONS, 5);
    open(&mut hud, &mut panel, 0, 0);
    let field = panel.field().expect("the dialog's root");
    assert_eq!(panel.chosen_item(), 0);
    assert_eq!(
        text_of(&hud.tree, field, "NewSaleDesc").as_deref(),
        Some("")
    );
}

#[test]
fn a_dropped_single_item_is_chosen_at_once_and_a_stack_asks_how_many() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    let layout = hud.toolbar_layout.clone().unwrap();
    let backpack = BackpackUi::create(&mut hud.tree, &layout, &Pack, PLAYER);
    let copy_of = |item: u64| {
        backpack.packs[0]
            .items
            .iter()
            .find(|slot| slot.item == item)
            .expect("the item in the pack")
            .icon
    };
    open(&mut hud, &mut panel, 0, 0);
    let field = panel.field().expect("the dialog's root");
    let slot = hud
        .tree
        .child_recursive(field, tokenize("NewSale"))
        .unwrap();
    let drop = |copy: usize| UiMessage {
        element_id: tokenize("NewSale"),
        element: Some(slot),
        message: MSG_STOP_DRAG_ITEM,
        data: 1,
        item: Some(copy),
    };

    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &drop(copy_of(SWORD)),
        1.0,
        &mut || 0.0,
    );
    assert_eq!(panel.chosen_item(), SWORD);
    assert_eq!(panel.chosen_quantity(), 1, "one is chosen at once");
    assert!(panel.take_stack_calls().is_empty());

    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &drop(copy_of(POTIONS)),
        1.0,
        &mut || 0.0,
    );
    let calls = panel.take_stack_calls();
    let context = match calls[..] {
        [ShopStackCall::Init {
            context, max: 5, ..
        }] => context,
        ref other => panic!("the stack dialog asks how many: {other:?}"),
    };
    panel.recv_stack_ui_done(&mut hud.tree, &Pack, context, 3);
    assert_eq!(panel.chosen_item(), POTIONS);
    assert_eq!(panel.chosen_quantity(), 3, "the stack's amount");
    assert_eq!(
        text_of(&hud.tree, field, "NewSaleDesc").as_deref(),
        Some("Healing Potions")
    );
}

#[test]
fn a_price_below_one_is_refused_with_a_message() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, POTIONS, 5);
    let field = panel.field().unwrap();
    set_price(&mut hud.tree, field, "0");
    panel.add_sale(&mut hud.tree, &mut hud.dialogs, &Pack, &mut || 0.5);
    assert_eq!(
        hud.dialogs.shown().map(|(_, kind)| kind),
        Some(DIALOG_MESSAGE)
    );
    assert_eq!(panel.confirm_context, 0, "no sale is asked");
    assert!(panel.take_requests().is_empty());
}

#[test]
fn a_sale_without_an_item_or_a_quantity_is_refused() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, 0, 0);
    panel.add_sale(&mut hud.tree, &mut hud.dialogs, &Pack, &mut || 0.5);
    assert_eq!(
        hud.dialogs.shown().map(|(_, kind)| kind),
        Some(DIALOG_MESSAGE)
    );
    assert_eq!(panel.confirm_context, 0);
}

#[test]
fn a_priced_sale_asks_for_its_fee_and_yes_sends_the_sell() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, POTIONS, 5);
    let field = panel.field().unwrap();
    set_price(&mut hud.tree, field, "10");
    let mut asked = 0;
    panel.add_sale(&mut hud.tree, &mut hud.dialogs, &Pack, &mut || {
        asked += 1;
        0.5
    });
    assert_eq!(asked, 1, "the charge is read once");
    assert_eq!(
        hud.dialogs.shown().map(|(_, kind)| kind),
        Some(DIALOG_CONFIRMATION)
    );
    let context = panel.confirm_context;
    assert_ne!(context, 0);
    assert!(
        panel.take_requests().is_empty(),
        "nothing is sent before the answer"
    );

    panel.dialog_outcomes(
        &mut hud.tree,
        &mut hud.dialogs,
        &[DialogOutcome::Confirmation { context, yes: true }],
        2.0,
    );
    assert_eq!(
        panel.take_requests(),
        vec![NewSaleRequest::Sell {
            item: POTIONS,
            quantity: 5,
            price: 10,
        }]
    );
    assert!(!panel.is_shown(), "the sale hides the dialog");
    assert_eq!(panel.confirm_context, 0);
}

#[test]
fn no_to_the_confirmation_sends_nothing() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, POTIONS, 5);
    let field = panel.field().unwrap();
    set_price(&mut hud.tree, field, "10");
    panel.add_sale(&mut hud.tree, &mut hud.dialogs, &Pack, &mut || 0.0);
    let context = panel.confirm_context;
    panel.dialog_outcomes(
        &mut hud.tree,
        &mut hud.dialogs,
        &[DialogOutcome::Confirmation {
            context,
            yes: false,
        }],
        2.0,
    );
    assert!(panel.take_requests().is_empty());
    assert!(panel.is_shown());
}

#[test]
fn the_close_button_leaves_the_store_and_the_loop_destroys_the_dialog() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel, POTIONS, 5);
    let field = panel.field().unwrap();
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &release("NewSaleCloseButton"),
        10.0,
        &mut || 0.0,
    );
    assert_eq!(panel.take_requests(), vec![NewSaleRequest::LeaveStore]);
    assert!(!panel.is_shown());
    panel.tick(&mut hud.tree, 100.0);
    assert_eq!(panel.field(), Some(field), "still waiting");
    panel.tick(&mut hud.tree, 131.0);
    assert_eq!(panel.field(), None, "destroyed after two minutes hidden");
}

#[test]
fn parse_price_reads_as_wcstol_with_base_zero() {
    assert_eq!(parse_price("12"), 12);
    assert_eq!(parse_price(" 7 "), 7);
    assert_eq!(parse_price("0x10"), 16);
    assert_eq!(parse_price("010"), 8);
    assert_eq!(parse_price("-3"), -3);
    assert_eq!(parse_price("0"), 0);
    assert_eq!(parse_price("abc"), 0);
}

#[test]
fn the_consignment_fee_is_rounded_up_and_at_least_one() {
    assert_eq!(consignment_fee(10, 5, 0.5), 25);
    assert_eq!(consignment_fee(7, 3, 0.5), 11, "10.5 rounds up");
    assert_eq!(consignment_fee(1, 1, 0.0), 1, "never less than one");
    assert_eq!(consignment_fee(3, 1, 0.1), 1);
}
