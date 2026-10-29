//! `gmShopConsignmentUI` on the real gameplay and toolbar layouts (`AC2_DAT_DIR`): the sales the
//! server opens it with, the selection and collect, the new button, the drop of an item on the
//! list (a sale asks how many, then the new-sale dialog; a restock asks for its confirmation), the
//! close button and the global loop.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::StringInfo;
use ac2_weenie::{Shop, ShopSale};

use crate::consignment::{ConsignmentRequest, ConsignmentSource, ConsignmentUi};
use crate::dialog::{DialogOutcome, DIALOG_CONFIRMATION};
use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::item_display::InventorySource;
use crate::mouse::UiMessage;
use crate::shop::{ShopSource, ShopStackCall};
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, BackpackUi, GameplayHud, ItemSlot};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;
const POTION_DID: u32 = 0x0500_0001;
const SWORD_SALE: u32 = 2;
const POTION_SALE: u32 = 1;

/// The player's pack (five potions and a sword) and the strings they render.
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

impl ShopSource for Pack {
    fn render_wire(&self, info: &StringInfo) -> String {
        info.literal.clone().unwrap_or_default()
    }
}

impl ConsignmentSource for Pack {
    fn item_did(&self, item: u64) -> Option<u32> {
        (item == POTIONS).then_some(POTION_DID)
    }
}

fn literal(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}

/// Two consigned sales: potions (active) and a sword (sold).
fn shop() -> Shop {
    Shop {
        id: 0x6F00_0001,
        name: literal("Sheila's Consignment"),
        desc: literal("Goods on consignment"),
        sorter: StringInfo::default(),
        portrait: 0,
        sales: vec![
            ShopSale {
                ordinal: POTION_SALE,
                item: POTIONS,
                item_did: POTION_DID,
                stock: 5,
                ..ShopSale::default()
            },
            ShopSale {
                ordinal: SWORD_SALE,
                item: SWORD,
                stock: 1,
                status: 3,
                ..ShopSale::default()
            },
        ],
        position: 0,
        store_size: 0,
    }
}

fn setup() -> Option<(GameplayHud, ConsignmentUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.consignment.take().expect("the consignment panel");
    Some((hud, panel))
}

fn release(name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

fn press_row(row: usize, data: i32) -> UiMessage {
    UiMessage {
        element_id: tokenize("Sale"),
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data,
        ..UiMessage::default()
    }
}

fn open(hud: &mut GameplayHud, panel: &mut ConsignmentUi) {
    panel.recv_view_consignment(&mut hud.tree, &mut hud.dialogs, &Pack, &shop(), 0.0);
}

fn row_of(panel: &ConsignmentUi, sale: u32) -> usize {
    panel
        .rows
        .iter()
        .find(|row| row.sale == sale)
        .map(|row| row.element)
        .expect("the sale's row")
}

/// The buttons the gameplay layout's `Consignment` carries under their retail token names
/// (`gmShopConsignmentUI::Create`'s `ID_NewButton`, `ID_CollectButton`, `ID_NextButton`,
/// `ID_PrevButton`, `ID_CloseButton` and `ID_HelpButton`).
#[test]
fn the_layout_carries_the_buttons_under_their_retail_names() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    let field = panel.field().expect("the consignment's root");
    for name in [
        "NewConsignmentButton",
        "CollectConsignmentButton",
        "ConsignmentNextButton",
        "ConsignmentPrevButton",
        "ConsignmentCloseButton",
        "ConsignmentHelpButton",
        "ItemList",
        "ConsignmentName",
    ] {
        assert!(
            hud.tree.child_recursive(field, tokenize(name)).is_some(),
            "{name} under Consignment"
        );
    }
    // The page buttons ghost at the list's ends: the first page has no previous page.
    let prev = hud
        .tree
        .child_recursive(field, tokenize("ConsignmentPrevButton"))
        .unwrap();
    assert!(hud.tree.bool_attribute(prev, BUTTON_GHOSTED));
}

#[test]
fn viewing_the_consignment_lists_its_sales_under_its_name() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    assert!(panel.is_shown());
    assert_eq!(panel.sale_ids(), vec![POTION_SALE, SWORD_SALE]);
    let field = panel.field().expect("the panel's root");
    let name = hud
        .tree
        .child_recursive(field, tokenize("ConsignmentName"))
        .and_then(|e| hud.tree.get(e)?.text());
    assert_eq!(name.as_deref(), Some("Sheila's Consignment"));
    // Each row carries the parts the panel fills (the `Sale` element of the layout).
    let row = row_of(&panel, POTION_SALE);
    for part in ["ItemIcon", "CostIcon", "CostQuantity"] {
        assert!(
            hud.tree.child_recursive(row, tokenize(part)).is_some(),
            "the row's {part}"
        );
    }
}

#[test]
fn selecting_a_sale_asks_for_its_item_and_collect_asks_for_the_sale() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    let row = row_of(&panel, POTION_SALE);
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &press_row(row, 0x4000_0004),
        1.0,
    );
    assert_eq!(panel.selected_item(), Some(POTIONS));
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::Select(POTIONS)]
    );

    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &release("CollectConsignmentButton"),
        1.0,
    );
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::Collect {
            target: POTION_SALE,
            profits: false,
        }]
    );

    // A sold sale collects its profits.
    let row = row_of(&panel, SWORD_SALE);
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &press_row(row, 0x4000_0004),
        1.0,
    );
    panel.take_requests();
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &release("CollectConsignmentButton"),
        1.0,
    );
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::Collect {
            target: SWORD_SALE,
            profits: true,
        }]
    );
}

#[test]
fn the_new_button_opens_the_new_sale_dialog_empty() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &release("NewConsignmentButton"),
        1.0,
    );
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::NewSale {
            item: 0,
            quantity: 0,
        }]
    );
}

#[test]
fn a_stack_dropped_on_the_list_with_no_sale_opens_the_new_sale_with_its_amount() {
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
    open(&mut hud, &mut panel);
    let field = panel.field().unwrap();
    let list = hud
        .tree
        .child_recursive(field, tokenize("ItemList"))
        .expect("the list");
    let drop = |copy: usize| UiMessage {
        element_id: tokenize("ItemList"),
        element: Some(list),
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
    );
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::NewSale {
            item: SWORD,
            quantity: 1,
        }],
        "one sword opens the new sale at once"
    );

    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &drop(copy_of(POTIONS)),
        1.0,
    );
    let context = match panel.take_stack_calls()[..] {
        [ShopStackCall::Init {
            context, max: 5, ..
        }] => context,
        ref other => panic!("the stack dialog asks how many: {other:?}"),
    };
    panel.recv_stack_ui_done(&mut hud.tree, &mut hud.dialogs, &Pack, context, 3);
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::NewSale {
            item: POTIONS,
            quantity: 3,
        }]
    );
}

#[test]
fn a_restock_asks_for_its_confirmation_and_yes_sends_it() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    // The restock's state as a drop on the sale (`HandleListBoxDrop`) leaves it.
    panel.item = POTIONS;
    panel.target = POTION_SALE;
    panel.stack_context = 7;
    panel.recv_stack_ui_done(&mut hud.tree, &mut hud.dialogs, &Pack, 7, 3);
    assert_eq!(
        hud.dialogs.shown().map(|(_, kind)| kind),
        Some(DIALOG_CONFIRMATION)
    );
    let context = panel.confirm_context;
    assert!(panel.take_requests().is_empty());
    panel.dialog_outcomes(&[DialogOutcome::Confirmation { context, yes: true }]);
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::Restock {
            item: POTIONS,
            quantity: 3,
            target: POTION_SALE,
        }]
    );
}

#[test]
fn the_server_updates_the_sales_and_leaves_the_panel_shut() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    let mut updated = shop();
    updated.sales.truncate(1);
    panel.recv_update_consignment(&mut hud.tree, &Pack, &updated);
    assert_eq!(panel.sale_ids(), vec![POTION_SALE]);
    panel.recv_leave_consignment(&mut hud.tree, &mut hud.dialogs, 5.0);
    assert!(!panel.is_shown());
    panel.tick(&mut hud.tree, 100.0);
    assert!(panel.field().is_some(), "still waiting to be destroyed");
    panel.tick(&mut hud.tree, 126.0);
    assert!(
        panel.field().is_none(),
        "destroyed after two minutes hidden"
    );
}

#[test]
fn the_close_button_leaves_the_consignment() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &release("ConsignmentCloseButton"),
        2.0,
    );
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::LeaveConsignment]
    );
    assert!(!panel.is_shown());
}

/// `gmShopConsignmentUI::RecvNotice_SelectionChanged`: the selected sale stays selected while the
/// player's selection is its item, and clears when the selection is another.
#[test]
fn another_selection_clears_the_selected_sale() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    let row = row_of(&panel, POTION_SALE);
    panel.message(
        &mut hud.tree,
        &mut hud.dialogs,
        &Pack,
        &press_row(row, 0x4000_0004),
        1.0,
    );
    panel.take_requests();
    panel.recv_selection_changed(&mut hud.tree, POTIONS);
    assert_eq!(panel.selected_item(), Some(POTIONS));
    panel.recv_selection_changed(&mut hud.tree, SWORD);
    assert_eq!(panel.selected_item(), None);
    panel.recv_update_mount_state(true);
    assert!(panel.mounted);
}

/// `gmShopConsignmentUI::RecvNotice_HandleDragFrom`: a row's icon is a drag source of its own
/// context, and that context dropped on the pack collects the row's sale (the profits when sold).
#[test]
fn a_row_dragged_into_the_pack_collects_its_sale() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    open(&mut hud, &mut panel);
    let row = panel
        .rows
        .iter()
        .find(|r| r.sale == SWORD_SALE)
        .copied()
        .unwrap();
    let icon = hud
        .tree
        .child_recursive(row.element, crate::token::tokenize("ItemIcon"))
        .unwrap();
    assert!(hud
        .tree
        .bool_attribute(icon, crate::token::tokenize("dragable")));
    panel.take_requests();
    panel.recv_handle_drag_from(row.context + 1000);
    assert!(panel.take_requests().is_empty(), "not a row's context");
    panel.recv_handle_drag_from(row.context);
    assert_eq!(
        panel.take_requests(),
        vec![ConsignmentRequest::Collect {
            target: SWORD_SALE,
            profits: true,
        }]
    );
}

#[test]
fn a_stack_restocks_a_sale_of_its_kind_unless_the_sale_is_sold() {
    let Some((mut hud, mut panel)) = setup() else {
        return;
    };
    // `CanStackWith`: the potions stack with their own sale (same definition, stackable).
    open(&mut hud, &mut panel);
    let mut active = shop();
    assert!(panel.can_stack_with(&Pack, POTION_SALE, POTIONS));
    // Not with a sale of another item, nor an item that does not stack.
    assert!(!panel.can_stack_with(&Pack, SWORD_SALE, POTIONS));
    assert!(!panel.can_stack_with(&Pack, POTION_SALE, SWORD));
    // An expired sale (2) still takes it; a sold one (3) does not (Legions 0x41b4b9).
    active.sales[0].status = 2;
    panel.recv_update_consignment(&mut hud.tree, &Pack, &active);
    assert!(panel.can_stack_with(&Pack, POTION_SALE, POTIONS));
    active.sales[0].status = 3;
    panel.recv_update_consignment(&mut hud.tree, &Pack, &active);
    assert!(!panel.can_stack_with(&Pack, POTION_SALE, POTIONS));
}
