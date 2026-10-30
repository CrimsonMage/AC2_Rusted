//! The modern toolbar (`modern_toolbar.rs`) on the real toolbar layout (`AC2_DAT_DIR`): the strip's
//! and the dock's parts laid out, the fan opening at the orb and the panels' wells lit, the
//! panels' keys still toggling them, and the classic style left as retail's.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::modern_art as art;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS, GLOBAL_MOUSE_DOWN, GLOBAL_MOUSE_MOVE};
use crate::token::tokenize;
use crate::{build_styled_hud, GameplayHud, HudStyle, ToolbarChild, UiTree};

fn hud(style: HudStyle) -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    Some(build_styled_hud(&store, 1365, 768, style).unwrap())
}

fn named(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .find_map(|&r| {
            if tree.el(r).id() == tokenize(name) {
                Some(r)
            } else {
                tree.child_recursive(r, tokenize(name))
            }
        })
        .unwrap_or_else(|| panic!("{name}"))
}

fn global(message: u32, data: i32) -> UiMessage {
    UiMessage {
        element_id: 0,
        element: None,
        message,
        data,
        item: None,
    }
}

#[test]
fn the_dock_folds_at_rest_and_opens_while_the_orb_is_pressed() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    hud.toolbar.follow(tree);
    let inventory = named(tree, "InventoryButton");
    let orb = named(tree, "ModernDockOrb");
    assert!(!tree.el(inventory).visible(), "the fan rests folded");
    assert!(tree.el(orb).visible());
    // A press on the orb pins the fan open; the pointer leaving keeps it open.
    let at = tree.screen_box(orb);
    tree.pointer = (at.x0 + 29, at.y0 + 29);
    hud.toolbar.message(tree, &global(GLOBAL_MOUSE_DOWN, 0));
    tree.pointer = (10, 10);
    hud.toolbar.message(tree, &global(GLOBAL_MOUSE_MOVE, 0));
    assert!(tree.el(inventory).visible(), "pinned open");
    // Pressed again, it unpins and folds once the pointer is away.
    tree.pointer = (at.x0 + 29, at.y0 + 29);
    hud.toolbar.message(tree, &global(GLOBAL_MOUSE_DOWN, 0));
    tree.pointer = (10, 10);
    hud.toolbar.message(tree, &global(GLOBAL_MOUSE_MOVE, 0));
    assert!(!tree.el(inventory).visible(), "folded again");
}

#[test]
fn a_shown_panel_lights_its_well_and_pip_and_its_key_still_toggles_it() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    hud.toolbar.follow(tree);
    let index = ToolbarChild::ALL
        .iter()
        .position(|&c| c == ToolbarChild::Inventory)
        .unwrap();
    let well = named(tree, &format!("ModernPanelWell{index}"));
    let pip = named(tree, &format!("ModernDockPip{index}"));
    assert_eq!(tree.el(well).image().unwrap().file, art::PANEL_WELL);
    // The inventory's key (`gmToolbarUI::ChildKeyPress`) shows it.
    hud.toolbar.message(
        tree,
        &global(GLOBAL_KEY_PRESS, ToolbarChild::Inventory.key() as i32),
    );
    hud.toolbar.follow(tree);
    assert!(hud.toolbar.shown(tree, ToolbarChild::Inventory));
    assert_eq!(tree.el(well).image().unwrap().file, art::PANEL_WELL_ON);
    assert_eq!(tree.el(pip).image().unwrap().file, art::PIP_ON);
}

#[test]
fn the_strip_lays_out_the_attack_buttons_the_set_chip_and_the_bar_in_order() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    hud.toolbar.follow(tree);
    let start = tree.screen_box(named(tree, "StartAttackButton"));
    let stop = tree.screen_box(named(tree, "StopAttackButton"));
    let bar = tree.screen_box(named(tree, "ShortcutField"));
    let fade = tree.screen_box(named(tree, "ModernStripFade"));
    assert!(start.x1 < stop.x1 && stop.x1 - art::GLOW_ROOM < bar.x0);
    assert!(fade.x0 <= start.x0 + art::GLOW_ROOM && fade.x1 >= bar.x1);
    assert_eq!(bar.width(), 10 * art::SLOT_PITCH);
    // Side by side: the two attack buttons share a row.
    assert_eq!(start.y0, stop.y0);
}

#[test]
fn the_classic_style_keeps_retails_bar() {
    let Some(mut hud) = hud(HudStyle::Classic) else {
        return;
    };
    let tree = &mut hud.tree;
    hud.toolbar.follow(tree);
    assert!(tree
        .roots()
        .iter()
        .all(|&r| tree.child_recursive(r, tokenize("ModernDockOrb")).is_none()));
    let field = named(tree, "ToolbarField");
    assert_eq!(
        tree.el(field).image().unwrap().file,
        0x4100_02E7,
        "retail's bar art"
    );
}
