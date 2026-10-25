//! `gmBackpackUI` on the real toolbar layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::{build_gameplay_hud, BackpackUi, InventorySource, ItemRequest, ItemSlot, UiTree};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;

/// The player's pack; `potions` is the potion stack's quantity (stackable to 10).
struct Player {
    potions: u32,
}

const ONE_POTION: Player = Player { potions: 1 };

impl InventorySource for Player {
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
        SWORD
    }
}

#[test]
fn the_backpack_shows_the_players_items_in_their_slots() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let mut backpack = BackpackUi::create(tree, &layout, &ONE_POTION, PLAYER);
    let main = &backpack.packs[0];
    eprintln!("pages {}, slots {}", backpack.packs.len(), main.max_size());
    assert!(main.max_size() > 2);
    assert_eq!(
        main.items.iter().map(|d| d.item).collect::<Vec<_>>(),
        vec![POTIONS, SWORD]
    );
    let potions = &main.items[0];
    assert_eq!(
        tree.get(potions.icon).unwrap().image().unwrap().file,
        0x0600_1001
    );
    // The selection's icon is `selected`.
    assert_eq!(
        tree.get(main.items[1].icon).unwrap().state(),
        tokenize("selected")
    );
    // A double click uses the potions.
    let press = UiMessage {
        element_id: tree.get(potions.icon).unwrap().id(),
        element: Some(potions.icon),
        message: MSG_MOUSE_PRESS_TOP,
        data: 0x4000_0007,
        item: None,
    };
    assert_eq!(
        backpack.message(tree, &ONE_POTION, &press),
        Some(ItemRequest::Use(POTIONS))
    );
}

/// The potions dragged from the main page's first slot to its sixth: the backpack's requests.
fn drag_potions(player: &Player) -> Option<(crate::GameplayHud, BackpackUi, Vec<ItemRequest>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let tree = &mut hud.tree;
    let mut backpack = BackpackUi::create(tree, &layout, player, PLAYER);
    // The inventory panel shown (its field and every parent).
    let potions = backpack.packs[0].items[0].icon;
    let mut at = Some(potions);
    while let Some(e) = at {
        tree.set_visible(e, true);
        at = tree.get(e).unwrap().parent;
    }
    let slot = |tree: &UiTree, i: usize| {
        let list = tree
            .get(potions)
            .unwrap()
            .parent
            .and_then(|b| tree.get(b).unwrap().parent)
            .unwrap();
        let b = tree.screen_box(tree.list_items(list)[i]);
        ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
    };
    let (x, y) = slot(tree, 0);
    let (tx, ty) = slot(tree, 5);
    tree.start_input(crate::UI_POINTER_X, x as f32);
    tree.start_input(crate::UI_POINTER_Y, y as f32);
    tree.start_input(0x4000_0004, 1.0);
    tree.start_input(crate::UI_POINTER_X, tx as f32);
    tree.start_input(crate::UI_POINTER_Y, ty as f32);
    assert!(tree.dragged().is_some(), "the potions are dragged");
    tree.stop_input(0x4000_0004);
    let requests: Vec<ItemRequest> = tree
        .take_messages()
        .iter()
        .filter_map(|m| backpack.message(tree, player, m))
        .collect();
    Some((hud, backpack, requests))
}

#[test]
fn an_item_dragged_to_another_slot_asks_to_move_there() {
    let Some((_, _, requests)) = drag_potions(&ONE_POTION) else {
        return;
    };
    assert!(
        requests.contains(&ItemRequest::Move {
            item: POTIONS,
            from: PLAYER,
            from_slot: 0,
            target: PLAYER,
            target_slot: 5,
            quantity: 1,
        }),
        "{requests:?}"
    );
}

#[test]
fn a_stack_dragged_asks_how_many_then_moves_that_many() {
    let player = Player { potions: 5 };
    let Some((_, mut backpack, requests)) = drag_potions(&player) else {
        return;
    };
    // The press selects; `ContainerDisplay::HandleListBoxDrop` asks with the whole stack as
    // default and most.
    assert_eq!(
        requests,
        [
            ItemRequest::Select(POTIONS),
            ItemRequest::Split {
                context: 1,
                item: POTIONS,
                target: PLAYER,
                amount: 5,
            }
        ]
    );
    // Another dialog's answer is not this drop's.
    assert_eq!(backpack.stack_ui_done(&player, 2, 2), None);
    assert_eq!(
        backpack.stack_ui_done(&player, 1, 2),
        Some(ItemRequest::Move {
            item: POTIONS,
            from: PLAYER,
            from_slot: 0,
            target: PLAYER,
            target_slot: 5,
            quantity: 2,
        })
    );
    assert_eq!(backpack.stack_ui_done(&player, 1, 2), None);
}
