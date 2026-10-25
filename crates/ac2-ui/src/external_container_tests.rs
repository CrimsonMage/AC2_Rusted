//! `gmExternalContainerUI` on the real gameplay and toolbar layouts (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::{build_gameplay_hud, ExternalContainerRequest, InventorySource, ItemSlot};

const CHEST: u64 = 0x8000_0000_0000_0A04;
const GEM: u64 = 0x8000_0000_0000_0105;
const SCROLL: u64 = 0x8000_0000_0000_0106;
const PACK: u64 = 0x8000_0000_0000_0020;

struct World;

impl InventorySource for World {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, name) = match item {
            GEM => (0x0600_1004, "Gem"),
            SCROLL => (0x0600_1005, "Scroll"),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            name: name.into(),
            quantity: 1,
            ..ItemSlot::default()
        })
    }

    fn contents_info(&self, container: u64, _: u32) -> (Vec<u64>, u32, u32) {
        match container {
            CHEST => (vec![GEM, SCROLL], 0, 12),
            _ => (Vec::new(), 0, 0),
        }
    }

    fn max_container_size(&self, _: u64) -> u32 {
        12
    }

    fn selection(&self) -> u64 {
        0
    }
}

fn button(name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

#[test]
fn an_opened_chest_shows_its_items_over_it_until_closed() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let panel = hud
        .external_container
        .as_mut()
        .expect("the external container");
    let field = panel.field().expect("ExternalContainerField");
    assert!(!tree.get(field).unwrap().visible);

    assert!(panel
        .container_state_update(tree, &World, CHEST, true)
        .is_empty());
    assert!(tree.get(field).unwrap().visible);
    let display = panel.display().unwrap();
    assert_eq!(display.container, CHEST);
    assert_eq!(
        display.items.iter().map(|d| d.item).collect::<Vec<_>>(),
        [GEM, SCROLL]
    );

    // Centred on the chest's projected centre (half sizes truncated), kept on the display.
    assert_eq!(panel.wants_position(), Some(CHEST));
    let rect = tree.get(field).unwrap().rect;
    let (w, h) = (rect.x1 - rect.x0 + 1, rect.y1 - rect.y0 + 1);
    let placed = |tree: &crate::UiTree| {
        let p = tree
            .placements()
            .into_iter()
            .find(|p| p.root == field)
            .unwrap();
        (p.x, p.y)
    };
    panel.position_window(tree, Some((500, 300)));
    assert_eq!(panel.wants_position(), None);
    let expect = |c: i32, size: i32| (f64::from(c) - f64::from(size) * 0.5) as i32;
    assert_eq!(placed(tree), (expect(500, w), expect(300, h)));
    panel.position_window(tree, None);
    assert_eq!(placed(tree), (expect(512, w), expect(384, h)));
    panel.position_window(tree, Some((1020, 2)));
    assert_eq!(placed(tree), (1024 - w, 0));

    // Loot All asks for the chest; Close asks it shut and hides the panel.
    let (request, _) = panel.message(tree, &World, &button("LootAllButton"));
    assert_eq!(request, Some(ExternalContainerRequest::LootAll(CHEST)));
    let (request, _) = panel.message(tree, &World, &button("ExternalContainerCloseButton"));
    assert_eq!(request, Some(ExternalContainerRequest::Close(CHEST)));
    assert!(!tree.get(field).unwrap().visible);
    assert!(panel.display().is_none());

    // Any container closing hides it (the panel listens to every object).
    panel.container_state_update(tree, &World, CHEST, true);
    panel.container_state_update(tree, &World, PACK, false);
    assert!(!tree.get(field).unwrap().visible);
    // The listed container going hides it too.
    panel.container_state_update(tree, &World, CHEST, true);
    panel.being_deleted(tree, GEM);
    assert!(tree.get(field).unwrap().visible);
    panel.being_deleted(tree, CHEST);
    assert!(!tree.get(field).unwrap().visible);
}
