//! `gmBackpackUI` on the real toolbar layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::{build_gameplay_hud, BackpackUi, InventorySource, ItemRequest, ItemSlot};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;

struct Player;

impl InventorySource for Player {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, name) = match item {
            POTIONS => (0x0600_1001, "5 Healing Potions"),
            SWORD => (0x0600_1002, "Practice Sword"),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            icon_layers: Vec::new(),
            name: name.into(),
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
    let backpack = BackpackUi::create(tree, &layout, &Player, PLAYER);
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
        backpack.message(tree, &press),
        Some(ItemRequest::Use(POTIONS))
    );
}
