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
    hud.toolbar.drawer_tick(&mut hud.tree, &pack, None, &Pack);
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
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
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
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
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
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
    assert!(veiled(tree, POTIONS) && !veiled(tree, SWORD));
    let arms = named(tree, "ModernFilterChipArms").unwrap();
    assert_eq!(tree.el(arms).image().unwrap().file, art::CHIP_ON);
    press(tree, &hud.toolbar, "ModernFilterChipAll");
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
    assert!(!veiled(tree, POTIONS) && !veiled(tree, SWORD));
}

fn press(tree: &mut UiTree, toolbar: &crate::ToolbarUi, name: &str) {
    let element = named(tree, name).unwrap();
    toolbar.message(
        tree,
        &crate::mouse::UiMessage {
            element_id: tokenize(name),
            element: Some(element),
            message: crate::token::MSG_MOUSE_PRESS_TOP,
            data: 0,
            item: None,
        },
    );
}

/// The All tab shows the four pages at once, stacked under their headings in the view; a pack
/// tab's `ShowPack` leaves it, the pages back in the pack area.
#[test]
fn the_all_tab_shows_every_page_until_a_pack_tab_is_pressed() {
    let Some((mut hud, pack)) = with_backpack() else {
        return;
    };
    let tree = &mut hud.tree;
    press(tree, &hud.toolbar, "ModernAllTab");
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
    let view = named(tree, "ModernAllView").unwrap();
    let pages: Vec<usize> = [
        "PackMainContainer",
        "PackOneContainer",
        "PackTwoContainer",
        "PackThreeContainer",
    ]
    .iter()
    .map(|n| named(tree, n).unwrap())
    .collect();
    assert!(tree.el(view).visible);
    let mut last = i32::MIN;
    for &page in &pages {
        let el = tree.el(page);
        assert!(el.visible && el.parent_index() == Some(view));
        assert!(el.rect.y0 > last, "stacked in order");
        last = el.rect.y0;
    }
    let foot = tree
        .el(named(tree, "ModernDrawerFoot").unwrap())
        .text()
        .unwrap();
    assert!(foot.starts_with("All packs"), "{foot}");
    if let Ok(out) = std::env::var("ZZ_OUT") {
        let dataset = ac2_dat::test_support::env_dataset().unwrap();
        let store = Arc::new(ResourceStore::new(dataset));
        let mut images = crate::load_ui_images(&store, tree).unwrap();
        let icons = crate::load_images(&store, &art::SOURCE_ICONS).unwrap();
        art::bake(&mut images, &icons);
        let panel = named(tree, "RightSidePanel").unwrap();
        tree.set_visible(panel, true);
        let field = named(tree, "InventoryField").unwrap();
        tree.set_visible(field, true);
        let _ = tree.compose(&images);
        let s = tree.compose_root(panel, &images);
        let f = std::fs::File::create(out).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), s.width, s.height);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&s.rgba).unwrap();
    }
    // A pack tab (`ShowPack`) leaves the view.
    pack.show_pack(tree, 1);
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
    assert!(!tree.el(view).visible);
    let area = tree.el(pages[0]).parent_index();
    assert!(pages
        .iter()
        .all(|&p| tree.el(p).parent_index() == area && area != Some(view)));
    assert!(tree.el(pages[1]).visible && !tree.el(pages[0]).visible);
}

/// [`Pack`] with `0` its selection.
struct Selected(u64);

impl crate::InventorySource for Selected {
    fn item(&self, item: u64) -> Option<crate::ItemSlot> {
        Pack.item(item)
    }
    fn contents_info(&self, container: u64, segment: u32) -> (Vec<u64>, u32, u32) {
        Pack.contents_info(container, segment)
    }
    fn max_container_size(&self, container: u64) -> u32 {
        Pack.max_container_size(container)
    }
    fn selection(&self) -> u64 {
        self.0
    }
    fn preferred_location(&self, item: u64) -> u32 {
        Pack.preferred_location(item)
    }
}

/// The selected item's ring fans out of its slot inside the grid; Use answers its press, a
/// Split that does not apply (one item, no stack) is dimmed and takes none, and Equip applies to
/// the sword only.
#[test]
fn the_item_ring_opens_on_the_selected_slot_with_its_actions() {
    let Some((mut hud, pack)) = with_backpack() else {
        return;
    };
    let tree = &mut hud.tree;
    for name in ["RightSidePanel", "InventoryField"] {
        let e = named(tree, name).unwrap();
        tree.set_visible(e, true);
    }
    hud.toolbar.drawer_tick(tree, &pack, None, &Selected(SWORD));
    let buttons: Vec<usize> = (0..5)
        .map(|i| named(tree, &format!("ModernItemRing{i}")).unwrap())
        .collect();
    let list = tree
        .child_recursive(
            named(tree, "PackMainContainer").unwrap(),
            tokenize("ContainerListBox"),
        )
        .unwrap();
    let grid = tree.screen_box(list);
    for &b in &buttons {
        let at = tree.screen_box(b);
        assert!(tree.el(b).visible);
        assert!(
            at.x0 >= grid.x0 && at.x1 <= grid.x1 && at.y0 >= grid.y0 && at.y1 <= grid.y1,
            "{at:?} in {grid:?}"
        );
    }
    let press = |tree: &UiTree, i: usize| {
        hud.toolbar.drawer_ring_press(&crate::mouse::UiMessage {
            element_id: tree.el(buttons[i]).id(),
            element: Some(buttons[i]),
            message: crate::token::MSG_MOUSE_PRESS_TOP,
            data: 0,
            item: None,
        })
    };
    use crate::modern_item_ring::RingAction;
    assert_eq!(press(tree, 0), Some((RingAction::Use, SWORD)));
    assert_eq!(press(tree, 2), None, "no stack to split");
    assert_eq!(press(tree, 4), Some((RingAction::Equip, SWORD)));
    if let Ok(out) = std::env::var("ZZ_OUT") {
        let dataset = ac2_dat::test_support::env_dataset().unwrap();
        let store = Arc::new(ResourceStore::new(dataset));
        let mut images = crate::load_ui_images(&store, tree).unwrap();
        let icons = crate::load_images(&store, &art::SOURCE_ICONS).unwrap();
        art::bake(&mut images, &icons);
        let panel = named(tree, "RightSidePanel").unwrap();
        let _ = tree.compose(&images);
        let s = tree.compose_root(panel, &images);
        let f = std::fs::File::create(out).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), s.width, s.height);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&s.rgba).unwrap();
    }
    // Nothing selected: the ring hides.
    hud.toolbar.drawer_tick(tree, &pack, None, &Pack);
    assert!(buttons.iter().all(|&b| !tree.el(b).visible));
}

/// The modern stack popover: a drag on its slider writes the amount into the entry, typing moves
/// the knob, and OK answers with what the entry holds, as retail's dialog does.
#[test]
fn the_stack_sliders_drag_sets_the_amount_and_ok_answers_it() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let stack = hud.stack.as_mut().unwrap();
    stack.init(tree, 7, "Split Healing Potions: how many?", 10, 10, 0, 0);
    let track = named(tree, "ModernStackTrack").unwrap();
    let knob = named(tree, "ModernStackKnob").unwrap();
    let entry = named(tree, "AmountEntry").unwrap();
    let at = tree.screen_box(track);
    // At ten, the knob sits at the track's right end.
    let k = tree.screen_box(knob);
    assert!(((k.x0 + k.x1) / 2 - at.x1).abs() <= 2, "{k:?} {at:?}");
    let global = |message, element: Option<usize>| crate::mouse::UiMessage {
        element_id: 0,
        element,
        message,
        data: 0,
        item: None,
    };
    tree.pointer = (at.x0 + at.width() * 3 / 10, (at.y0 + at.y1) / 2);
    let strings = NoStrings;
    stack.message(
        tree,
        &global(crate::token::MSG_MOUSE_PRESS_TOP, Some(track)),
        &strings,
    );
    assert_eq!(tree.el(entry).text().unwrap(), "3");
    tree.pointer.0 = at.x0 + at.width() * 6 / 10;
    stack.message(
        tree,
        &global(crate::mouse::GLOBAL_MOUSE_MOVE, None),
        &strings,
    );
    stack.message(tree, &global(crate::mouse::GLOBAL_MOUSE_UP, None), &strings);
    assert_eq!(tree.el(entry).text().unwrap(), "6");
    // Typing moves the knob.
    tree.set_text(entry, "1");
    stack.message(
        tree,
        &global(crate::mouse::GLOBAL_MOUSE_MOVE, None),
        &strings,
    );
    let k = tree.screen_box(knob);
    assert!(
        ((k.x0 + k.x1) / 2 - (at.x0 + at.width() / 10)).abs() <= 2,
        "{k:?}"
    );
    if let Ok(out) = std::env::var("ZZ_OUT") {
        let dataset = ac2_dat::test_support::env_dataset().unwrap();
        let store = Arc::new(ResourceStore::new(dataset));
        let mut images = crate::load_ui_images(&store, tree).unwrap();
        let icons = crate::load_images(&store, &art::SOURCE_ICONS).unwrap();
        art::bake(&mut images, &icons);
        let field = stack.field().unwrap();
        let _ = tree.compose(&images);
        let s = tree.compose_root(field, &images);
        let f = std::fs::File::create(out).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), s.width, s.height);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&s.rgba).unwrap();
    }
    let ok = named(tree, "StackOkButton").unwrap();
    let release = crate::mouse::UiMessage {
        element_id: tokenize("StackOkButton"),
        element: Some(ok),
        message: crate::token::MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    };
    let done = stack.message(tree, &release, &strings).unwrap();
    assert_eq!((done.context, done.amount), (7, 1));
}

/// Renders every string as nothing.
struct NoStrings;

impl crate::ui_string::UiStrings for NoStrings {
    fn render_string(&self, _: &crate::ui_string::UiString) -> String {
        String::new()
    }
}

/// The modern loot window: the design's size, the list four slots wide, both buttons and the
/// close inside it; the classic panel keeps retail's size.
#[test]
fn the_loot_window_takes_the_designs_size_and_grid() {
    let (Some(modern), Some(classic)) = (hud(HudStyle::Modern), hud(HudStyle::Classic)) else {
        return;
    };
    let tree = &modern.tree;
    let root = named(tree, "ExternalContainerField").unwrap();
    let at = tree.screen_box(root);
    assert_eq!((at.width(), at.height()), (214, 262));
    let list = tree.child_recursive(root, 0x397F_2C39).unwrap();
    assert_eq!(
        tree.el(list).attribute(tokenize("max_columns")),
        Some(&crate::layout::AttributeValue::Int(4))
    );
    for name in [
        "LootAllButton",
        "TransmuteAllButton",
        "ExternalContainerCloseButton",
    ] {
        let b = tree.screen_box(named(tree, name).unwrap());
        assert!(
            b.x0 >= at.x0 && b.x1 <= at.x1 && b.y1 <= at.y1,
            "{name} {b:?}"
        );
    }
    let tree = &classic.tree;
    let root = named(tree, "ExternalContainerField").unwrap();
    let at = tree.screen_box(root);
    assert_eq!((at.width(), at.height()), (138, 165));
}
