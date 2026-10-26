//! `gm3DItemsUI` on the real gameplay layout (`AC2_DAT_DIR`), over a stub player.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::tokenize;
use crate::{build_gameplay_hud, InventorySource, ItemRequest, ItemSlot, Items3dUi, UiTree};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;
const CREATURE: u64 = 0x8000_0000_0000_0A01;

struct Player;

impl InventorySource for Player {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, quantity, stackable) = match item {
            POTIONS => (0x0600_1001, 5, true),
            SWORD => (0x0600_1002, 1, false),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            quantity,
            stackable,
            max_stack: if stackable { 10 } else { 0 },
            ..ItemSlot::default()
        })
    }

    fn contents_info(&self, _: u64, _: u32) -> (Vec<u64>, u32, u32) {
        (vec![POTIONS, SWORD], 0, 24)
    }

    fn max_container_size(&self, _: u64) -> u32 {
        24
    }

    fn selection(&self) -> u64 {
        SWORD
    }

    fn money(&self) -> u32 {
        250
    }

    fn coin_max_stack(&self) -> u32 {
        100
    }
}

fn setup() -> Option<(UiTree, Items3dUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut tree = hud.tree;
    let items = Items3dUi::create(&mut tree, PLAYER);
    Some((tree, items))
}

/// A stand-in for the dragged copy: an element carrying the item the way an item icon does.
fn copy(tree: &mut UiTree, item: u64, from: u64, equipped: bool, slot: i32) -> usize {
    let icon = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("ItemIcon")))
        .expect("an item icon");
    let attributes = &mut tree.el_mut(icon).attributes;
    attributes.insert(
        tokenize("Token_ShortcutData"),
        AttributeValue::InstanceId(item),
    );
    attributes.insert(tokenize("FromID"), AttributeValue::InstanceId(from));
    attributes.insert(tokenize("Token_OldIndex"), AttributeValue::Int(slot));
    tree.set_bool(icon, tokenize("Equipped"), equipped);
    icon
}

fn field_message(tree: &UiTree, message: u32, data: i32, item: Option<usize>) -> UiMessage {
    let field = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == tokenize("ItemSelectionField"))
        .unwrap();
    UiMessage {
        element_id: tokenize("ItemSelectionField"),
        element: Some(field),
        message,
        data,
        item,
    }
}

/// `copy` dropped on the field (`stop_drag_item`, dropped), the pointer at (400, 300).
fn drop_on_field(tree: &mut UiTree, items: &mut Items3dUi, copy: usize) {
    tree.start_input(crate::UI_POINTER_X, 400.0);
    tree.start_input(crate::UI_POINTER_Y, 300.0);
    let message = field_message(tree, MSG_STOP_DRAG_ITEM, 1, Some(copy));
    items.message(tree, &Player, &message);
}

fn move_to(target: u64, quantity: u32) -> ItemRequest {
    ItemRequest::Move {
        item: SWORD,
        from: PLAYER,
        from_slot: 1,
        target,
        target_slot: 0,
        quantity,
    }
}

#[test]
fn a_sword_from_the_pack_goes_to_the_object_under_the_pointer() {
    let Some((mut tree, mut items)) = setup() else {
        return;
    };
    let sword = copy(&mut tree, SWORD, PLAYER, false, 1);
    assert_eq!(items.wants_pick(), None);
    drop_on_field(&mut tree, &mut items, sword);
    // `StartFind3DTarget`: the search waits at the pointer.
    assert_eq!(items.wants_pick(), Some((400, 300)));
    // `FinishFind3DTarget` and `MidItemDrop`: not a stack, so `FinishItemDrop` at once.
    assert_eq!(
        items.finish_pick(&mut tree, &Player, Some(CREATURE)),
        Some(move_to(CREATURE, 1))
    );
    assert_eq!(items.wants_pick(), None);
    // Nothing under the pointer, or the player itself: the ground.
    for picked in [None, Some(PLAYER)] {
        drop_on_field(&mut tree, &mut items, sword);
        assert_eq!(
            items.finish_pick(&mut tree, &Player, picked),
            Some(move_to(0, 1))
        );
    }
}

#[test]
fn a_stack_asks_how_many_and_the_answer_moves_that_many() {
    let Some((mut tree, mut items)) = setup() else {
        return;
    };
    let potions = copy(&mut tree, POTIONS, PLAYER, false, 0);
    drop_on_field(&mut tree, &mut items, potions);
    let Some(ItemRequest::DropSplit {
        context,
        item,
        amount,
    }) = items.finish_pick(&mut tree, &Player, Some(CREATURE))
    else {
        panic!("a stack of five asks");
    };
    // The stack's size, up to its maximum (10), is the default and the most.
    assert_eq!((item, amount), (POTIONS, 5));
    assert_eq!(items.contexts(), [context, 0]);
    // Another dialog's answer is not ours.
    assert_eq!(items.stack_ui_done(context + 1, 3), None);
    assert_eq!(
        items.stack_ui_done(context, 3),
        Some(ItemRequest::Move {
            item: POTIONS,
            from: PLAYER,
            from_slot: 0,
            target: CREATURE,
            target_slot: 0,
            quantity: 3,
        })
    );
    assert_eq!(items.stack_ui_done(context, 3), None, "answered once");
}

#[test]
fn only_an_unworn_item_of_the_players_own_pack_is_dropped() {
    let Some((mut tree, mut items)) = setup() else {
        return;
    };
    // Worn: it comes off first.
    let worn = copy(&mut tree, SWORD, PLAYER, true, 0);
    drop_on_field(&mut tree, &mut items, worn);
    assert_eq!(items.wants_pick(), None);
    // In a side pack (`FromID` the pack).
    let packed = copy(&mut tree, SWORD, 0x8000_0000_0000_0101, false, 0);
    drop_on_field(&mut tree, &mut items, packed);
    assert_eq!(items.wants_pick(), None);
    // A drop that was refused (`data` 0: it went to the owner) is no drop.
    let sword = copy(&mut tree, SWORD, PLAYER, false, 1);
    let message = field_message(&tree, MSG_STOP_DRAG_ITEM, 0, Some(sword));
    items.message(&mut tree, &Player, &message);
    assert_eq!(items.wants_pick(), None);
}

#[test]
fn the_coin_field_asks_how_much_and_leaves_the_bag_for_the_target() {
    let Some((mut tree, mut items)) = setup() else {
        return;
    };
    let coins = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("CoinsField")))
        .expect("the coin field");
    drop_on_field(&mut tree, &mut items, coins);
    // `MidMoneyDrop`: the money, up to the coin stack's maximum, is the default and the most.
    let Some(ItemRequest::MoneyStack {
        context,
        target,
        amount,
    }) = items.finish_pick(&mut tree, &Player, Some(CREATURE))
    else {
        panic!("the money asks");
    };
    assert_eq!((target, amount), (0, 100));
    assert_eq!(
        items.stack_ui_done(context, 40),
        Some(ItemRequest::DragFromMoneyBag {
            quantity: 40,
            to_slot: 0,
            to_container: CREATURE,
        })
    );
}

#[test]
fn a_press_on_the_world_with_a_selection_drags_the_selected_item() {
    let Some((mut tree, mut items)) = setup() else {
        return;
    };
    let message = field_message(&tree, tokenize("not_dragable"), 0, None);
    items.message(&mut tree, &Player, &message);
    // `Start3DDrag`: the field's icon shows the selection, dragable, as an item shortcut.
    let icon = tree
        .roots()
        .iter()
        .find_map(|&r| {
            let field = tree.get(r)?;
            (field.id() == tokenize("ItemSelectionField"))
                .then(|| tree.child_recursive(r, tokenize("ItemIcon")))?
        })
        .expect("the field's icon");
    assert_eq!(tree.get(icon).unwrap().image().unwrap().file, 0x0600_1002);
    assert!(tree.bool_attribute(icon, tokenize("dragable")));
    let attribute = |name: &str| tree.get(icon).unwrap().attribute(tokenize(name)).cloned();
    assert_eq!(
        attribute("Token_ShortcutType"),
        Some(AttributeValue::Id(0x4000_0002))
    );
    assert_eq!(
        attribute("Token_ShortcutData"),
        Some(AttributeValue::InstanceId(SWORD))
    );
}
