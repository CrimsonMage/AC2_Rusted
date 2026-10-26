//! `gmPaperDollUI` on the real toolbar layout (`AC2_DAT_DIR`): the worn items in their slots,
//! their requests, and the doll shows with the inventory panel.

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::tokenize;
use crate::{
    build_gameplay_hud, BackpackUi, InventorySource, ItemRequest, ItemSlot, PaperDollUi,
    ToolbarChild, UiTree, UI_POINTER_X, UI_POINTER_Y,
};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const HELM: u64 = 0x8000_0000_0000_0012;
const SWORD: u64 = 0x8000_0000_0000_0013;
const HEAD: u32 = 0x1;
const TORSO: u32 = 0x2;
const PRIMARY_HAND: u32 = 0x40;
const OTHER: u64 = 0x2000_0000_0000_0002;
const LEFT: u32 = 0x4000_0004;
const MSG_MOUSE_PRESS_TOP: u32 = crate::token::MSG_MOUSE_PRESS_TOP;

struct Player;

impl InventorySource for Player {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, name) = match item {
            HELM => (0x0600_1003, "Leather Cap"),
            SWORD => (0x0600_1004, "Practice Sword"),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            icon_layers: Vec::new(),
            name: name.into(),
            quantity: 1,
            stackable: false,
            max_stack: 0,
        })
    }

    fn contents_info(&self, _: u64, segment: u32) -> (Vec<u64>, u32, u32) {
        let size = if segment == 0 { 24 } else { 0 };
        (vec![SWORD], 0, size)
    }

    fn max_container_size(&self, _: u64) -> u32 {
        24
    }

    fn selection(&self) -> u64 {
        0
    }

    fn durability(&self, item: u64) -> Option<(i32, i32)> {
        (item == HELM).then_some((30, 100))
    }

    fn durability_tooltip(&self, current: i32, max: i32) -> Option<String> {
        Some(format!("{current} / {max}"))
    }

    fn preferred_location(&self, item: u64) -> u32 {
        if item == SWORD {
            PRIMARY_HAND
        } else {
            0
        }
    }
}

fn find(tree: &UiTree, name: &str) -> usize {
    let id = tokenize(name);
    tree.roots()
        .iter()
        .find_map(|&r| {
            if tree.get(r).unwrap().id() == id {
                Some(r)
            } else {
                tree.child_recursive(r, id)
            }
        })
        .unwrap_or_else(|| panic!("{name}"))
}

fn centre(tree: &UiTree, element: usize) -> (i32, i32) {
    let b = tree.screen_box(element);
    ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
}

/// A drag from `from` to `to` with the left button: press, move there, let go.
fn drag(tree: &mut UiTree, from: (i32, i32), to: (i32, i32)) -> Vec<UiMessage> {
    point(tree, from);
    tree.start_input(LEFT, 1.0);
    point(tree, (from.0 + 6, from.1 + 6));
    point(tree, to);
    tree.stop_input(LEFT);
    tree.take_messages()
}

fn point(tree: &mut UiTree, (x, y): (i32, i32)) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

#[test]
fn the_paper_doll_wears_the_equipped_items_and_follows_their_requests() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let mut doll = PaperDollUi::create(tree, &layout, &Player, PLAYER, &[(HEAD, HELM)]);
    let viewport = doll.viewport().expect("PaperDollViewport");
    // The doll shows with the inventory panel.
    doll.follow_panel(tree);
    assert!(!tree.get(viewport).unwrap().visible());
    hud.toolbar.show(tree, ToolbarChild::Inventory, true);
    doll.follow_panel(tree);
    assert!(tree.get(viewport).unwrap().visible());
    tree.global_loop(0.0);

    // The cap in the head slot, its icon the item's.
    assert_eq!(doll.item_at(HEAD), Some(HELM));
    assert_eq!(doll.item_at(TORSO), None);
    let head = find(tree, "Head_Slot");
    let icon = tree.child(head, tokenize("ItemIcon")).unwrap();
    assert_eq!(tree.get(icon).unwrap().image().unwrap().file, 0x0600_1003);

    // A press on the worn cap selects it (`ItemDisplay`'s icon).
    point(tree, {
        let b = tree.screen_box(icon);
        ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
    });
    tree.start_input(LEFT, 1.0);
    let presses: Vec<UiMessage> = tree
        .take_messages()
        .into_iter()
        .filter(|m| m.message == MSG_MOUSE_PRESS_TOP && m.element == Some(icon))
        .collect();
    assert!(!presses.is_empty(), "the press reaches the icon");
    let asked: Vec<_> = presses
        .iter()
        .filter_map(|m| doll.message(tree, &Player, m))
        .collect();
    assert_eq!(asked, vec![ItemRequest::Select(HELM)]);
    tree.stop_input(LEFT);
    let _ = tree.take_messages();

    // A request on the cap begins and fails; the icon is pending while it waits.
    doll.update_inventory(tree, &Player, 0x4000_0005, HELM, HEAD, &[]);
    assert_eq!(doll.item_at(HEAD), Some(HELM));
    doll.update_inventory(tree, &Player, 0x4000_0004, HELM, HEAD, &[]);
    assert_eq!(doll.item_at(HEAD), Some(HELM));

    // A cap added to the torso slot, and the sword replacing it.
    doll.update_inventory(tree, &Player, 0x4000_0002, SWORD, TORSO, &[]);
    assert_eq!(doll.item_at(TORSO), Some(SWORD));
    doll.update_inventory(tree, &Player, 0x4000_0002, HELM, TORSO, &[]);
    assert_eq!(doll.item_at(TORSO), Some(HELM));

    // The cap taken off: its slot empties.
    doll.update_inventory(tree, &Player, 0x4000_0003, HELM, TORSO, &[]);
    assert_eq!(doll.item_at(TORSO), None);
    assert_eq!(doll.item_at(HEAD), Some(HELM));
    doll.update_inventory(tree, &Player, 0x4000_0003, HELM, HEAD, &[]);
    assert_eq!(doll.item_at(HEAD), None);

    // The frame's list is the whole equipment: what it no longer lists comes off.
    doll.update_inventory(tree, &Player, 0x4000_0001, 0, 0, &[(HEAD, HELM)]);
    assert_eq!(doll.item_at(HEAD), Some(HELM));
    assert_eq!(doll.item_at(TORSO), None);
    // The frame's list refills the doll.
    doll.update_inventory(
        tree,
        &Player,
        0x4000_0001,
        0,
        0,
        &[(HEAD, HELM), (TORSO, SWORD)],
    );
    assert_eq!(doll.item_at(HEAD), Some(HELM));
    assert_eq!(doll.item_at(TORSO), Some(SWORD));
}

#[test]
fn a_drop_on_a_slot_or_the_body_equips_the_item() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let mut backpack = BackpackUi::create(tree, &layout, &Player, PLAYER);
    let mut doll = PaperDollUi::create(tree, &layout, &Player, PLAYER, &[(HEAD, HELM)]);
    let viewport = doll.viewport().expect("PaperDollViewport");
    hud.toolbar.show(tree, ToolbarChild::Inventory, true);
    doll.follow_panel(tree);
    tree.global_loop(0.0);
    let sword = backpack.packs[0].items[0].icon;

    // The sword from the pack onto the torso slot: equipped at the slot's location.
    let torso = centre(tree, find(tree, "Torso_Slot"));
    let messages = drag(tree, centre(tree, sword), torso);
    let asked: Vec<_> = messages
        .iter()
        .filter_map(|m| doll.message(tree, &Player, m))
        .collect();
    assert_eq!(
        asked,
        vec![ItemRequest::Equip {
            item: SWORD,
            location: TORSO
        }]
    );

    // Onto the doll's body: the item's preferred location (the primary hand).
    tree.global_loop(1.0);
    let messages = drag(tree, centre(tree, sword), centre(tree, viewport));
    let asked: Vec<_> = messages
        .iter()
        .filter_map(|m| doll.message(tree, &Player, m))
        .collect();
    assert_eq!(
        asked,
        vec![ItemRequest::Equip {
            item: SWORD,
            location: PRIMARY_HAND
        }]
    );

    // The cap dragged from its slot back onto the pack comes off into the slot it lands on.
    tree.global_loop(2.0);
    let icon = tree
        .child(find(tree, "Head_Slot"), tokenize("ItemIcon"))
        .unwrap();
    // The backpack's list (the external container panel has one too).
    let list = tree
        .get(backpack.packs[0].items[0].background)
        .unwrap()
        .parent
        .unwrap();
    let slot = tree.list_items(list)[3];
    let messages = drag(tree, centre(tree, icon), centre(tree, slot));
    let asked: Vec<_> = messages
        .iter()
        .filter_map(|m| backpack.message(tree, &Player, m))
        .collect();
    assert_eq!(
        asked,
        vec![ItemRequest::Unequip {
            item: HELM,
            target_slot: 3
        }]
    );
}

#[test]
fn a_drop_from_another_container_moves_the_item_into_the_player_and_equips_it() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    // The sword is in another container's pack, not the player's.
    let backpack = BackpackUi::create(tree, &layout, &Player, OTHER);
    let mut doll = PaperDollUi::create(tree, &layout, &Player, PLAYER, &[]);
    hud.toolbar.show(tree, ToolbarChild::Inventory, true);
    tree.global_loop(0.0);
    let sword = backpack.packs[0].items[0].icon;
    let torso = centre(tree, find(tree, "Torso_Slot"));
    let messages = drag(tree, centre(tree, sword), torso);
    let asked: Vec<_> = messages
        .iter()
        .filter_map(|m| doll.message(tree, &Player, m))
        .collect();
    assert_eq!(
        asked,
        // Retail returns after the move; the equip waits for another drop.
        vec![ItemRequest::Move {
            item: SWORD,
            from: OTHER,
            from_slot: 0,
            target: PLAYER,
            target_slot: 0,
            quantity: 1,
        }]
    );
}

#[test]
fn a_press_on_the_doll_waits_for_the_pick_and_selects_the_part_worn_there() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let mut doll = PaperDollUi::create(tree, &layout, &Player, PLAYER, &[(HEAD, HELM)]);
    let viewport = doll.viewport().expect("PaperDollViewport");
    hud.toolbar.show(tree, ToolbarChild::Inventory, true);
    doll.follow_panel(tree);
    tree.global_loop(0.0);

    // A press on the doll waits for the scene's pick at the pointer.
    let on_doll = centre(tree, viewport);
    point(tree, on_doll);
    tree.start_input(LEFT, 1.0);
    for message in tree.take_messages() {
        doll.message(tree, &Player, &message);
    }
    assert_eq!(doll.wants_pick(), Some(on_doll));
    // The pick hits the head (`Head_InvLoc`'s mesh type): the cap is selected.
    assert_eq!(
        doll.finish_pick(tree, Some(0x8100_0003)),
        Some(ItemRequest::Select(HELM))
    );
    assert_eq!(doll.wants_pick(), None);
    // Dragging on from the doll drags the cap's icon.
    point(tree, (on_doll.0 + 8, on_doll.1 + 8));
    for message in tree.take_messages() {
        doll.message(tree, &Player, &message);
    }
    assert!(tree.dragged().is_some(), "the cap's icon is dragged");
    tree.stop_input(LEFT);

    // A hit on the feet, where nothing is worn, selects nothing.
    tree.start_input(LEFT, 1.0);
    for message in tree.take_messages() {
        doll.message(tree, &Player, &message);
    }
    assert_eq!(doll.finish_pick(tree, Some(0x8100_0001)), None);
    tree.stop_input(LEFT);
    let _ = tree.take_messages();
}

#[test]
fn the_worn_items_durability_fills_its_meter_and_hides_without_durability() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let mut doll = PaperDollUi::create(tree, &layout, &Player, PLAYER, &[(HEAD, HELM)]);
    hud.toolbar.show(tree, ToolbarChild::Inventory, true);
    tree.global_loop(0.0);

    // The cap's meter at `1 - 30 / 100`, its field shown; the torso's field hidden.
    let durability = find(tree, "Head_Durability");
    let meter = tree
        .child_recursive(durability, tokenize("DurabilityMeter"))
        .unwrap();
    assert!(tree.get(durability).unwrap().visible());
    let position = tree.float_attribute(meter, tokenize("position")).unwrap();
    assert!((position - 0.7).abs() < 1e-6, "{position}");
    assert_eq!(tree.tooltip_text(durability).as_deref(), Some("30 / 100"));
    let torso = find(tree, "Torso_Durability");
    assert!(!tree.get(torso).unwrap().visible());

    // The cap taken off: its meter empties and hides.
    doll.update_inventory(tree, &Player, 0x4000_0003, HELM, HEAD, &[]);
    assert!(!tree.get(durability).unwrap().visible());
    let position = tree.float_attribute(meter, tokenize("position")).unwrap();
    assert!(position.abs() < 1e-6, "{position}");

    // A sword without durability in the torso slot leaves its field hidden, and an update
    // to it hides the field again (`RecvNotice_ItemUpdate`).
    doll.update_inventory(tree, &Player, 0x4000_0002, SWORD, TORSO, &[]);
    assert!(!tree.get(torso).unwrap().visible());
    doll.item_changed(tree, &Player, SWORD);
    assert!(!tree.get(torso).unwrap().visible());
}
