//! The classic inventory's vertical resize (`inventory_resize.rs`) on the real HUD
//! (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::token::tokenize;
use crate::window_locations::UI_LEFT_BUTTON;
use crate::{build_gameplay_hud, UiTree, UI_POINTER_X, UI_POINTER_Y};

#[test]
fn the_grips_id_is_its_name() {
    assert_eq!(
        crate::inventory_resize::GRIP,
        tokenize("InventoryResizeBar")
    );
}

fn hud() -> Option<UiTree> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 2560, 1440).unwrap().tree;
    let inventory = named(&tree, "InventoryField");
    tree.set_visible(inventory, true);
    tree.sync_inventory_grip();
    Some(tree)
}

fn named(tree: &UiTree, name: &str) -> usize {
    let id = tokenize(name);
    (0..tree.slots_len())
        .find(|&i| tree.get(i).is_some_and(|e| e.id() == id))
        .unwrap_or_else(|| panic!("{name}"))
}

fn point(tree: &mut UiTree, (x, y): (i32, i32)) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

/// The press on the grip (the window's top edge), a pull up by `dy`, the release.
fn pull_up(tree: &mut UiTree, dy: i32) {
    let grip = named(tree, "InventoryResizeBar");
    let b = tree.screen_box(grip);
    let at = ((b.x0 + b.x1) / 2, b.y0 + 2);
    point(tree, at);
    assert_eq!(tree.mouse_top(), Some(grip), "the grip is on top at {at:?}");
    tree.start_input(UI_LEFT_BUTTON, 1.0);
    for step in 1..=5 {
        point(tree, (at.0, at.1 - dy * step / 5));
    }
    tree.stop_input(UI_LEFT_BUTTON);
}

fn height(tree: &UiTree, name: &str) -> i32 {
    tree.screen_box(named(tree, name)).height()
}

#[test]
fn the_inventory_grows_by_pack_rows_and_the_rest_keeps_its_size() {
    let Some(mut tree) = hud() else { return };
    let root = named(&tree, "RightSidePanel");
    let before = (
        tree.screen_box(root),
        height(&tree, "PackMainContainer"),
        height(&tree, "PackMainButton"),
    );
    // 80 pixels is two rows and a bit: it snaps to two (72).
    pull_up(&mut tree, 80);
    let after = tree.screen_box(root);
    assert_eq!(after.height(), before.0.height() + 72, "two rows taller");
    assert_eq!(after.y1, before.0.y1, "the bottom edge stays");
    assert_eq!(
        height(&tree, "PackMainContainer"),
        before.1 + 72,
        "the list's rows"
    );
    assert_eq!(
        height(&tree, "PackMainButton"),
        before.2,
        "the tabs keep their size"
    );
    // Not shorter than the layout's own height.
    pull_up(&mut tree, -500);
    assert_eq!(tree.screen_box(root).height(), before.0.height());
}

#[test]
fn the_grip_is_there_only_while_the_inventory_is() {
    let Some(mut tree) = hud() else { return };
    let (grip, inventory) = (
        named(&tree, "InventoryResizeBar"),
        named(&tree, "InventoryField"),
    );
    assert!(tree.get(grip).unwrap().visible());
    tree.set_visible(inventory, false);
    tree.sync_inventory_grip();
    assert!(!tree.get(grip).unwrap().visible());
}
