//! The modern inventory drawer (`modern_inventory.rs`) on the real toolbar layout
//! (`AC2_DAT_DIR`): the drawer on the right edge under the radar, its parts in the design's
//! order, the grid seven slots wide, the drawer kept on its edge when the display grows, and the
//! classic style left as retail's panel.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::modern_art as art;
use crate::modern_inventory::{doll_frame, restyle};
use crate::token::tokenize;
use crate::{build_styled_hud, GameplayHud, HudStyle, UiTree};

fn hud(style: HudStyle) -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    Some(build_styled_hud(&store, 1365, 768, style).unwrap())
}

fn named(tree: &UiTree, name: &str) -> Option<usize> {
    let id = tokenize(name);
    tree.roots().iter().find_map(|&r| {
        if tree.el(r).id() == id {
            Some(r)
        } else {
            tree.child_recursive(r, id)
        }
    })
}

#[test]
fn the_drawer_sits_on_the_right_edge_with_its_parts_in_order() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    hud.toolbar.follow(tree);
    let panel = tree.screen_box(named(tree, "RightSidePanel").unwrap());
    assert_eq!((panel.x1, panel.y0, panel.width()), (1364, 172, 316));
    assert_eq!(panel.y1, 768 - 1 - 68, "{panel:?}");
    let at = |name: &str| tree.screen_box(named(tree, name).unwrap_or_else(|| panic!("{name}")));
    let (help, close, money) = (
        at("InventoryHelpButton"),
        at("InventoryCloseButton"),
        at("MoneyBag"),
    );
    assert!(help.x1 < panel.x0 + 40 && close.x0 > panel.x1 - 40);
    let doll = tree.screen_box(doll_frame(tree).unwrap());
    let rail = at("PackMainButton");
    let grid = at("PackMainContainer");
    assert!(money.y1 < doll.y0 && doll.y1 < rail.y0 && rail.y1 < grid.y0);
    assert_eq!(
        tree.el(doll_frame(tree).unwrap()).image().unwrap().file,
        art::DOLL_FRAME
    );
    // The page's list is seven slots wide.
    let list = tree
        .child_recursive(
            named(tree, "PackMainContainer").unwrap(),
            tokenize("ContainerListBox"),
        )
        .unwrap();
    assert!(tree.el(list).rect.width() >= 7 * art::ITEM_PITCH);
}

#[test]
fn a_resized_display_keeps_the_drawer_on_its_edge() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    hud.toolbar.follow(tree);
    tree.set_display(2276, 1280);
    hud.toolbar.follow(tree);
    let panel = tree.screen_box(named(tree, "RightSidePanel").unwrap());
    assert_eq!((panel.x1, panel.y0, panel.width()), (2275, 172, 316));
    assert_eq!(panel.y1, 1280 - 1 - 68, "{panel:?}");
}

#[test]
fn the_classic_style_keeps_retails_inventory() {
    let Some(hud) = hud(HudStyle::Classic) else {
        return;
    };
    assert!(doll_frame(&hud.tree).is_none());
    assert!(hud.inventory_layout.is_none());
}

/// The restyle keeps every element of the layout and changes only the item slot's size and art.
#[test]
fn the_item_slot_restyle_keeps_every_element() {
    let Some(hud) = hud(HudStyle::Modern) else {
        return;
    };
    let (Some(before), Some(after)) = (hud.toolbar_layout.as_ref(), hud.inventory_layout.as_ref())
    else {
        return;
    };
    assert_eq!(restyle(before).elements.len(), before.elements.len());
    let slot = after.element(tokenize("ItemBackground")).unwrap();
    assert_eq!(slot.rect.x1 as i32 + 1, art::ITEM_PITCH);
    assert!(slot.states.iter().flat_map(|s| &s.media).any(
        |m| matches!(m, crate::layout::MediaDesc::Image { file, .. } if *file == art::ITEM_WELL)
    ));
    assert_eq!(after.elements.len(), before.elements.len());
}
