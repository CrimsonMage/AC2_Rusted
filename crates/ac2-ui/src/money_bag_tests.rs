//! `gmMoneyBagUI` and the coin field on the real toolbar layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::backpack::find;
use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::ui_string::{UiString, UiStrings};
use crate::{
    build_gameplay_hud, BackpackUi, GameplayHud, InventorySource, ItemRequest, ItemSlot,
    MoneyBagUi, UiTree,
};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;

/// The player: `money`, the coin stack's `coin_max`, the bypass option, the potions' count and
/// whether the potions melt.
struct Purse {
    money: u32,
    coin_max: u32,
    bypass: bool,
    potions: u32,
    melts: bool,
}

impl Default for Purse {
    fn default() -> Self {
        Self {
            money: 250,
            coin_max: u32::MAX,
            bypass: false,
            potions: 5,
            melts: false,
        }
    }
}

impl InventorySource for Purse {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, name, quantity, stackable) = match item {
            POTIONS => (0x0600_1001, "Healing Potions", self.potions, true),
            SWORD => (0x0600_1002, "Practice Sword", 1, false),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            icon_layers: Vec::new(),
            name: name.into(),
            quantity,
            stackable,
            max_stack: if stackable { 10 } else { 0 },
        })
    }

    fn contents_info(&self, _: u64, segment: u32) -> (Vec<u64>, u32, u32) {
        let size = if segment == 0 { 24 } else { 0 };
        (vec![POTIONS, SWORD], 0, size)
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

    fn money(&self) -> u32 {
        self.money
    }

    fn coin_max_stack(&self) -> u32 {
        self.coin_max
    }

    fn melts_to_money(&self, item: u64) -> bool {
        self.melts && item == POTIONS
    }

    fn bypass_melting_confirmation(&self) -> bool {
        self.bypass
    }
}

/// `ID_Inventory_Amount(value=250)` and the like: the string's id and its variables, as text.
struct Strings;

impl UiStrings for Strings {
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

/// The gameplay HUD with the player's backpack and money bag made over it, the inventory shown.
fn setup(purse: &Purse) -> Option<(GameplayHud, BackpackUi, MoneyBagUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let backpack = BackpackUi::create(tree, &layout, purse, PLAYER);
    let bag = MoneyBagUi::create(tree);
    let potions = backpack.packs[0].items[0].icon;
    let mut at = Some(potions);
    while let Some(e) = at {
        tree.set_visible(e, true);
        at = tree.get(e).unwrap().parent;
    }
    Some((hud, backpack, bag))
}

/// `ID_Inventory_Amount` with `value` as the money.
fn amount_text(money: u32) -> String {
    format!(
        "{}({}={money})",
        tokenize("ID_Inventory_Amount"),
        tokenize("value")
    )
}

fn label(tree: &UiTree) -> usize {
    let bag = find(tree, "MoneyBag").expect("the MoneyBag element");
    tree.child_recursive(bag, tokenize("AmountLabel"))
        .expect("the AmountLabel")
}

/// The centre of the `index`th slot of the main page.
fn slot_centre(tree: &UiTree, backpack: &BackpackUi, index: usize) -> (i32, i32) {
    let potions = backpack.packs[0].items[0].icon;
    let list = tree
        .get(potions)
        .unwrap()
        .parent
        .and_then(|b| tree.get(b).unwrap().parent)
        .unwrap();
    let b = tree.screen_box(tree.list_items(list)[index]);
    ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
}

/// A drop of `copy` on the money bag `bag`, as the drag system sends it.
fn bag_drop(bag: usize, copy: usize) -> UiMessage {
    UiMessage {
        element_id: tokenize("MoneyBag"),
        element: Some(bag),
        message: MSG_STOP_DRAG_ITEM,
        data: 1,
        item: Some(copy),
    }
}

#[test]
fn the_amount_label_reads_the_money_on_each_update() {
    let purse = Purse::default();
    let Some((mut hud, _, mut bag)) = setup(&purse) else {
        return;
    };
    let tree = &mut hud.tree;
    bag.set_money(tree, &Strings, 250);
    assert_eq!(bag.money(), 250);
    assert_eq!(
        tree.get(label(tree)).unwrap().text().as_deref(),
        Some(amount_text(250).as_str())
    );
    bag.set_money(tree, &Strings, 300);
    assert_eq!(
        tree.get(label(tree)).unwrap().text().as_deref(),
        Some(amount_text(300).as_str())
    );
}

#[test]
fn a_coin_field_dropped_on_a_pack_slot_asks_how_much_then_takes_that_out() {
    // The coin stack's maximum (100) is the less than the money (250), so it is the limit.
    let purse = Purse {
        coin_max: 100,
        ..Purse::default()
    };
    let Some((mut hud, mut backpack, _)) = setup(&purse) else {
        return;
    };
    let tree = &mut hud.tree;
    let coin = tree
        .child_recursive(find(tree, "MoneyBag").unwrap(), tokenize("CoinsField"))
        .expect("the CoinsField");
    // The coin field is dropped with the pointer over the sixth slot of the main page: the
    // list box gets the copy (the coin field itself, `stop_drag_item`).
    let potions = backpack.packs[0].items[0].icon;
    let list = tree
        .get(potions)
        .unwrap()
        .parent
        .and_then(|b| tree.get(b).unwrap().parent)
        .unwrap();
    let (tx, ty) = slot_centre(tree, &backpack, 5);
    tree.start_input(crate::UI_POINTER_X, tx as f32);
    tree.start_input(crate::UI_POINTER_Y, ty as f32);
    let drop = UiMessage {
        element_id: tree.get(list).unwrap().id(),
        element: Some(list),
        message: MSG_STOP_DRAG_ITEM,
        data: 1,
        item: Some(coin),
    };
    let request = backpack.message(tree, &purse, &drop);
    let Some(ItemRequest::MoneyStack {
        context,
        target,
        amount,
    }) = request
    else {
        panic!("the stack dialog asks how much: {request:?}");
    };
    assert_eq!((target, amount), (PLAYER, 100));
    // The answer is the drag from the money bag, into the slot the coins were dropped on.
    assert_eq!(
        backpack.stack_ui_done(&purse, context, 40),
        Some(ItemRequest::DragFromMoneyBag {
            quantity: 40,
            to_slot: 5,
            to_container: PLAYER,
        })
    );
    // An answer again is nobody's.
    assert_eq!(backpack.stack_ui_done(&purse, context, 40), None);
}

/// The copy of an item's slot (`ItemIcon`, which carries the item as its shortcut).
fn slot_copy(backpack: &BackpackUi, index: usize) -> usize {
    backpack.packs[0].items[index].icon
}

#[test]
fn an_item_dropped_on_the_bag_asks_for_the_confirmation_then_melts() {
    let purse = Purse::default();
    let Some((mut hud, backpack, mut bag)) = setup(&purse) else {
        return;
    };
    let tree = &mut hud.tree;
    let bag_element = find(tree, "MoneyBag").unwrap();
    let mut dialogs = DialogFactory::new(None);
    // The sword is not a stack: the confirmation asks, then the answer melts it from slot 1.
    let sword = slot_copy(&backpack, 1);
    let request = bag.message(
        tree,
        &purse,
        &Strings,
        &mut dialogs,
        &bag_drop(bag_element, sword),
        false,
    );
    assert_eq!(request, None);
    let (context, kind) = dialogs.shown().expect("the confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert_eq!(
        bag.take_outcome(&[DialogOutcome::Confirmation {
            context,
            yes: false,
        }]),
        None
    );
    dialogs.close_dialog(tree, context);
    bag.message(
        tree,
        &purse,
        &Strings,
        &mut dialogs,
        &bag_drop(bag_element, sword),
        false,
    );
    let (context, _) = dialogs.shown().expect("the confirmation again");
    assert_eq!(
        bag.take_outcome(&[DialogOutcome::Confirmation { context, yes: true }]),
        Some(ItemRequest::DragToMoneyBag {
            item: SWORD,
            quantity: 1,
            from_slot: 1,
            from_container: PLAYER,
        })
    );
}

#[test]
fn shift_or_the_bypass_option_melts_without_the_confirmation() {
    let Some((mut hud, backpack, mut bag)) = setup(&Purse::default()) else {
        return;
    };
    let tree = &mut hud.tree;
    let bag_element = find(tree, "MoneyBag").unwrap();
    let sword = slot_copy(&backpack, 1);
    let mut dialogs = DialogFactory::new(None);
    let melt = ItemRequest::DragToMoneyBag {
        item: SWORD,
        quantity: 1,
        from_slot: 1,
        from_container: PLAYER,
    };
    // Shift held.
    let request = bag.message(
        tree,
        &Purse::default(),
        &Strings,
        &mut dialogs,
        &bag_drop(bag_element, sword),
        true,
    );
    assert_eq!(request, Some(melt));
    // The bypass option.
    let bypass = Purse {
        bypass: true,
        ..Purse::default()
    };
    let request = bag.message(
        tree,
        &bypass,
        &Strings,
        &mut dialogs,
        &bag_drop(bag_element, sword),
        false,
    );
    assert_eq!(request, Some(melt));
    assert_eq!(dialogs.shown(), None, "no confirmation was asked");
}

#[test]
fn a_stack_asks_how_many_then_the_confirmation_melts_that_many() {
    let purse = Purse {
        potions: 5,
        ..Purse::default()
    };
    let Some((mut hud, backpack, mut bag)) = setup(&purse) else {
        return;
    };
    let tree = &mut hud.tree;
    let bag_element = find(tree, "MoneyBag").unwrap();
    let potions = slot_copy(&backpack, 0);
    let mut dialogs = DialogFactory::new(None);
    let request = bag.message(
        tree,
        &purse,
        &Strings,
        &mut dialogs,
        &bag_drop(bag_element, potions),
        false,
    );
    let Some(ItemRequest::MoneyBagSplit {
        context,
        item,
        from,
        amount,
    }) = request
    else {
        panic!("the stack asks how many: {request:?}");
    };
    assert_eq!((item, from, amount), (POTIONS, PLAYER, 5));
    // Three out of five: the confirmation asks, and yes melts three.
    assert_eq!(
        bag.stack_ui_done(tree, &purse, &Strings, &mut dialogs, context, 3, false),
        None
    );
    let (confirm, kind) = dialogs.shown().expect("the confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert_eq!(
        bag.take_outcome(&[DialogOutcome::Confirmation {
            context: confirm,
            yes: true,
        }]),
        Some(ItemRequest::DragToMoneyBag {
            item: POTIONS,
            quantity: 3,
            from_slot: 0,
            from_container: PLAYER,
        })
    );
}

#[test]
fn a_melting_item_goes_in_at_once_and_an_equipped_one_is_refused() {
    let purse = Purse {
        melts: true,
        ..Purse::default()
    };
    let Some((mut hud, backpack, mut bag)) = setup(&purse) else {
        return;
    };
    let tree = &mut hud.tree;
    let bag_element = find(tree, "MoneyBag").unwrap();
    let potions = slot_copy(&backpack, 0);
    let mut dialogs = DialogFactory::new(None);
    assert_eq!(
        bag.message(
            tree,
            &purse,
            &Strings,
            &mut dialogs,
            &bag_drop(bag_element, potions),
            false,
        ),
        Some(ItemRequest::DragToMoneyBag {
            item: POTIONS,
            quantity: 5,
            from_slot: 0,
            from_container: PLAYER,
        })
    );
    // Worn items are not taken.
    tree.set_bool(potions, tokenize("Equipped"), true);
    assert_eq!(
        bag.message(
            tree,
            &purse,
            &Strings,
            &mut dialogs,
            &bag_drop(bag_element, potions),
            false,
        ),
        None
    );
    // A press on a slot is still a press, not a drop.
    let press = UiMessage {
        element_id: tree.get(potions).unwrap().id(),
        element: Some(potions),
        message: MSG_MOUSE_PRESS_TOP,
        data: 0x4000_0004,
        item: None,
    };
    assert_eq!(
        bag.message(tree, &purse, &Strings, &mut dialogs, &press, false),
        None
    );
}

#[test]
fn a_coin_field_dropped_on_a_page_asks_for_the_page_s_first_slot() {
    let purse = Purse {
        coin_max: 100,
        ..Purse::default()
    };
    let Some((mut hud, mut backpack, _)) = setup(&purse) else {
        return;
    };
    let tree = &mut hud.tree;
    let coin = tree
        .child_recursive(find(tree, "MoneyBag").unwrap(), tokenize("CoinsField"))
        .expect("the CoinsField");
    // Dropped on the main page's tab: `gmBackpackUI::ListenToMessage`'s coin branch takes the
    // first slot after the page's items.
    let page = find(tree, "PackMainButton").expect("the main page's tab");
    let drop = UiMessage {
        element_id: tree.get(page).unwrap().id(),
        element: Some(page),
        message: MSG_STOP_DRAG_ITEM,
        data: 1,
        item: Some(coin),
    };
    let request = backpack.message(tree, &purse, &drop);
    let Some(ItemRequest::MoneyStack {
        context,
        target,
        amount,
    }) = request
    else {
        panic!("the stack dialog asks how much: {request:?}");
    };
    assert_eq!((target, amount), (0, 100));
    assert_eq!(
        backpack.stack_ui_done(&purse, context, 7),
        Some(ItemRequest::DragFromMoneyBag {
            quantity: 7,
            // After the page's two items.
            to_slot: 2,
            to_container: 0,
        })
    );
}

/// `gmMoneyBagUI_DDCB`: an item the player has (`DoIHaveItem`) and does not wear
/// (`IsOnInventory`) is taken from any pack, a side pack too; one the player does not have, or
/// wears, is not.
#[test]
fn the_bag_takes_an_item_from_a_side_pack_but_not_a_worn_or_foreign_one() {
    /// The purse, its potions melting; `worn` is the item `IsOnInventory` reports, `foreign`
    /// one `DoIHaveItem` denies.
    struct Packs {
        purse: Purse,
        worn: u64,
        foreign: u64,
    }
    impl InventorySource for Packs {
        fn item(&self, item: u64) -> Option<ItemSlot> {
            self.purse.item(item)
        }
        fn contents_info(&self, container: u64, segment: u32) -> (Vec<u64>, u32, u32) {
            self.purse.contents_info(container, segment)
        }
        fn max_container_size(&self, container: u64) -> u32 {
            self.purse.max_container_size(container)
        }
        fn selection(&self) -> u64 {
            0
        }
        fn player(&self) -> u64 {
            PLAYER
        }
        fn melts_to_money(&self, _: u64) -> bool {
            true
        }
        fn has_item(&self, item: u64) -> bool {
            item != self.foreign && self.purse.item(item).is_some()
        }
        fn is_on_inventory(&self, item: u64) -> bool {
            item == self.worn
        }
    }
    const SIDE_PACK: u64 = 0x8000_0000_0000_0020;
    let packs = Packs {
        purse: Purse::default(),
        worn: 0,
        foreign: 0,
    };
    let Some((mut hud, backpack, mut bag)) = setup(&packs.purse) else {
        return;
    };
    let tree = &mut hud.tree;
    let bag_element = find(tree, "MoneyBag").unwrap();
    let potions = slot_copy(&backpack, 0);
    tree.el_mut(potions).attributes.insert(
        tokenize("FromID"),
        crate::layout::AttributeValue::InstanceId(SIDE_PACK),
    );
    let mut dialogs = DialogFactory::new(None);
    let mut drop = |tree: &mut UiTree, packs: &Packs| {
        bag.message(
            tree,
            packs,
            &Strings,
            &mut dialogs,
            &bag_drop(bag_element, potions),
            false,
        )
    };
    assert_eq!(
        drop(tree, &packs),
        Some(ItemRequest::DragToMoneyBag {
            item: POTIONS,
            quantity: 5,
            from_slot: 0,
            from_container: SIDE_PACK,
        }),
        "a side pack's item is taken"
    );
    let worn = Packs {
        worn: POTIONS,
        ..packs
    };
    assert_eq!(drop(tree, &worn), None, "a worn item is not");
    let foreign = Packs {
        worn: 0,
        foreign: POTIONS,
        ..worn
    };
    assert_eq!(
        drop(tree, &foreign),
        None,
        "nor one the player does not have"
    );
}
