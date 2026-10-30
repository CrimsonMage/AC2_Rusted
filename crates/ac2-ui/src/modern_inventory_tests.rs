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

const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;

/// The player's four pages: the main pack holds a potion stack and a sword.
struct Pack;

impl crate::InventorySource for Pack {
    fn item(&self, item: u64) -> Option<crate::ItemSlot> {
        let name = match item {
            POTIONS => "Healing Potions",
            SWORD => "Practice Sword",
            _ => return None,
        };
        Some(crate::ItemSlot {
            icon: 0x0600_1001,
            icon_layers: Vec::new(),
            name: name.into(),
            quantity: 1,
            stackable: false,
            max_stack: 0,
        })
    }
    fn contents_info(&self, _: u64, segment: u32) -> (Vec<u64>, u32, u32) {
        let size = [35, 14, 21, 28][segment.min(3) as usize];
        let items = if segment == 0 {
            vec![POTIONS, SWORD]
        } else {
            Vec::new()
        };
        (items, 0, size)
    }
    fn max_container_size(&self, _: u64) -> u32 {
        98
    }
    fn selection(&self) -> u64 {
        0
    }
    fn preferred_location(&self, item: u64) -> u32 {
        // The sword goes in the primary hand (`PrimaryHand_InvLoc`).
        if item == SWORD {
            0x40
        } else {
            0
        }
    }
}

/// The modern HUD with the player's backpack made over the drawer's restyled slots.
fn with_backpack() -> Option<(GameplayHud, crate::BackpackUi)> {
    let mut hud = hud(HudStyle::Modern)?;
    let layout = hud.inventory_layout.clone().unwrap();
    let pack = crate::BackpackUi::create(&mut hud.tree, &layout, &Pack, 0x2000_0000_0000_0001);
    hud.toolbar.follow(&mut hud.tree);
    hud.toolbar.drawer_tick(&mut hud.tree, &pack, &Pack);
    Some((hud, pack))
}

#[test]
fn each_tab_shows_its_pages_count_and_the_footer_names_the_page_shown() {
    let Some((hud, pack)) = with_backpack() else {
        return;
    };
    let tree = &hud.tree;
    let text = |name: &str| tree.el(named(tree, name).unwrap()).text().unwrap();
    for (i, page) in pack.packs.iter().enumerate() {
        let used = if i == 0 { 2 } else { 0 };
        assert_eq!(
            text(&format!("ModernPackCount{i}")),
            format!("{used}/{}", page.max_size())
        );
    }
    assert_eq!(
        text("ModernDrawerFoot"),
        format!("Main Pack \u{b7} 2/{}", pack.packs[0].max_size())
    );
}

/// The search veils the shown page's slots whose item's name lacks the text; the slots, their
/// order and their items stay.
#[test]
fn the_search_veils_the_slots_it_leaves_out() {
    let Some((mut hud, pack)) = with_backpack() else {
        return;
    };
    let tree = &mut hud.tree;
    let field = named(tree, "ModernDrawerSearch").unwrap();
    tree.set_text(field, "SWORD");
    hud.toolbar.drawer_tick(tree, &pack, &Pack);
    let veiled = |tree: &UiTree, item: u64| {
        let slot = pack.packs[0].items.iter().find(|d| d.item == item).unwrap();
        tree.child(slot.background, tokenize("ModernSlotVeil"))
            .is_some_and(|v| tree.el(v).visible)
    };
    assert!(veiled(tree, POTIONS));
    assert!(!veiled(tree, SWORD));
    let slots: Vec<u64> = pack.packs[0].items.iter().map(|d| d.item).collect();
    assert_eq!(slots, [POTIONS, SWORD]);
    tree.set_text(field, "");
    hud.toolbar.drawer_tick(tree, &pack, &Pack);
    assert!(!veiled(tree, POTIONS));
}

/// A press on the Arms chip veils the slots whose item does not equip in a hand; All shows them
/// again.
#[test]
fn a_filter_chip_veils_the_items_that_do_not_equip_where_it_says() {
    let Some((mut hud, pack)) = with_backpack() else {
        return;
    };
    let tree = &mut hud.tree;
    let veiled = |tree: &UiTree, item: u64| {
        let slot = pack.packs[0].items.iter().find(|d| d.item == item).unwrap();
        tree.child(slot.background, tokenize("ModernSlotVeil"))
            .is_some_and(|v| tree.el(v).visible)
    };
    let press = |tree: &mut UiTree, toolbar: &crate::ToolbarUi, name: &str| {
        let chip = named(tree, name).unwrap();
        toolbar.message(
            tree,
            &crate::mouse::UiMessage {
                element_id: tokenize(name),
                element: Some(chip),
                message: crate::token::MSG_MOUSE_PRESS_TOP,
                data: 0,
                item: None,
            },
        );
    };
    press(tree, &hud.toolbar, "ModernFilterChipArms");
    hud.toolbar.drawer_tick(tree, &pack, &Pack);
    assert!(veiled(tree, POTIONS) && !veiled(tree, SWORD));
    let arms = named(tree, "ModernFilterChipArms").unwrap();
    assert_eq!(tree.el(arms).image().unwrap().file, art::CHIP_ON);
    press(tree, &hud.toolbar, "ModernFilterChipAll");
    hud.toolbar.drawer_tick(tree, &pack, &Pack);
    assert!(!veiled(tree, POTIONS) && !veiled(tree, SWORD));
}
