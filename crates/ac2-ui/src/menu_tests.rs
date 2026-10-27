//! `UIElement_Menu` on a hand-built layout: a drop-down over a list of text items in a popup. The
//! real friends page's drop-down is tested with the friends panel (`ac2-client`'s friends test).

use std::sync::Arc;

use ac2_assets::Ac2Font;

use crate::element::UiBox;
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::mouse::{UI_POINTER_X, UI_POINTER_Y};
use crate::text_tests::font;
use crate::token::{tokenize, STATELESS, TEXT};
use crate::{UiTextData, UiTree, MSG_MENU_NEW_SELECTION};

const PANEL: &str = "MenuPanel";
const DROP: &str = "MenuDrop";
const DISPLAY: &str = "MenuDisplay";
const POPUP: &str = "MenuPopup";
const LIST: &str = "MenuList";
const ITEM: &str = "MenuItem";
const BUTTON: u32 = 0x4000_0004;
/// A kind that is not a text element (a text template of this kind is refused).
const FIELD: u32 = 0x4DA7_2ECC;

fn element(id: &str, kind: u32, r: Rect, attrs: Vec<(&str, AttributeValue)>) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind,
        default_state: STATELESS,
        rect: r,
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: Rect {
                x0: -1,
                y0: -1,
                x1: -1,
                y1: -1,
            },
            z_level: 0,
            attributes: attrs
                .into_iter()
                .map(|(name, value)| AttributeDesc {
                    id: tokenize(name),
                    value,
                })
                .collect(),
            media: Vec::new(),
            transitions: Vec::new(),
            children: Vec::new(),
        }],
    }
}

fn with_children(mut desc: ElementDesc, children: Vec<ElementDesc>) -> ElementDesc {
    desc.states[0].children = children.into_iter().map(Arc::new).collect();
    desc
}

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

/// A panel (200 x 100) holding the drop-down (80 x 20 at (10, 10)): its display text and its menu.
/// The popup is 100 x 100 over an 80 x 80 list of 80 x 20 items, so its margin is 20 a side.
/// `item_kind` is the item template's kind (a text element unless a test says otherwise).
fn layout(item_kind: u32) -> Arc<LayoutDesc> {
    // The display text is mouse-invisible, as the drop-downs' are, so the menu is the top.
    let display = element(
        DISPLAY,
        TEXT,
        rect(0, 0, 99, 19),
        vec![("mouse_invisible", AttributeValue::Bool(true))],
    );
    let drop = with_children(
        element(
            DROP,
            tokenize("menu"),
            rect(10, 10, 89, 29),
            vec![
                ("popup", AttributeValue::Id(tokenize(POPUP))),
                ("listbox", AttributeValue::Id(tokenize(LIST))),
                ("text_item", AttributeValue::Id(tokenize(ITEM))),
                ("selection_display", AttributeValue::Id(tokenize(DISPLAY))),
                ("open_up", AttributeValue::Bool(false)),
            ],
        ),
        vec![display],
    );
    let panel = with_children(
        element(PANEL, FIELD, rect(0, 0, 199, 99), vec![]),
        vec![drop],
    );
    let list = element(
        LIST,
        tokenize("listbox"),
        rect(0, 0, 79, 79),
        vec![("click_select", AttributeValue::Bool(true))],
    );
    let popup = with_children(
        element(POPUP, FIELD, rect(0, 0, 99, 99), vec![]),
        vec![list],
    );
    let item = element(ITEM, item_kind, rect(0, 0, 79, 19), vec![]);
    Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 200,
        height: 100,
        elements: vec![Arc::new(panel), Arc::new(popup), Arc::new(item)],
    })
}

/// Text for a string id: "string N" for any id (the menu's string items are the table's).
struct Strings;

impl UiTextData for Strings {
    fn font(&self, _did: u32) -> Option<Arc<Ac2Font>> {
        Some(font())
    }

    fn default_font(&self) -> Option<Arc<Ac2Font>> {
        Some(font())
    }

    fn string(&self, _table: u32, id: u32) -> Option<Vec<u16>> {
        Some(format!("string {id}").encode_utf16().collect())
    }
}

/// The tree with the panel as a root, its menu and its display.
fn tree_with(item_kind: u32) -> (UiTree, usize, usize, usize) {
    let layout = layout(item_kind);
    let mut tree = UiTree::new(300, 200);
    tree.set_text_data(Arc::new(Strings));
    let panel = tree
        .create_root(Arc::clone(&layout), tokenize(PANEL))
        .unwrap();
    let menu = tree.child_recursive(panel, tokenize(DROP)).unwrap();
    let display = tree.child_recursive(menu, tokenize(DISPLAY)).unwrap();
    (tree, panel, menu, display)
}

/// The menu's popup element (a root only while the menu is open).
fn popup_of(tree: &UiTree, menu: usize) -> usize {
    tree.menu_popup(menu).expect("the menu's popup")
}

fn point_at(tree: &mut UiTree, x: i32, y: i32) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

fn click(tree: &mut UiTree, x: i32, y: i32) {
    point_at(tree, x, y);
    tree.start_input(BUTTON, 1.0);
}

fn release(tree: &mut UiTree) {
    tree.stop_input(BUTTON);
}

fn centre(b: UiBox) -> (i32, i32) {
    ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
}

fn fill(tree: &mut UiTree, menu: usize) {
    assert!(tree.menu_insert_text(menu, "Yes", 0).is_some());
    assert!(tree.menu_insert_text(menu, "No", 1).is_some());
}

#[test]
fn a_press_opens_the_popup_under_the_menu_and_a_second_press_closes_it() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    let popup = popup_of(&tree, menu);
    let drop = tree.screen_box(menu);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    assert!(tree.menu_is_open(menu));
    assert!(tree.roots().contains(&popup));
    // Sized to the widest item (80) plus the margin (20), the items' height (40) plus the margin.
    let popup_box = tree.screen_box(popup);
    assert_eq!((popup_box.width(), popup_box.height()), (100, 60));
    // Under the menu (retail's y1 - y0 of the menu: its bottom row), centred on it (the popup is
    // wider by 20, so 10 out each side).
    assert_eq!(popup_box.y0, drop.y0 + drop.height() - 1);
    assert_eq!(
        popup_box.x0,
        drop.x0 - (popup_box.width() - drop.width()) / 2
    );
    release(&mut tree);
    // A release over the menu keeps the popup open.
    assert!(tree.menu_is_open(menu));
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    assert!(!tree.menu_is_open(menu));
    assert!(!tree.roots().contains(&popup));
}

#[test]
fn a_press_outside_closes_the_open_popup() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    let drop = tree.screen_box(menu);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    assert!(tree.menu_is_open(menu));
    // A press on the bare display (outside the menu and the popup) closes it.
    click(&mut tree, 250, 150);
    release(&mut tree);
    assert!(!tree.menu_is_open(menu));
}

#[test]
fn a_release_in_the_popup_margin_keeps_it_open_but_one_on_a_item_closes_it() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    let popup = popup_of(&tree, menu);
    let drop = tree.screen_box(menu);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    let popup_box = tree.screen_box(popup);
    // The margin is past the list's 80 x 40 items: over the popup, not over the list.
    click(&mut tree, popup_box.x0 + 90, popup_box.y0 + 50);
    release(&mut tree);
    assert!(tree.menu_is_open(menu));
    assert_eq!(tree.menu_selected_index(menu), None);
    // Over the second item: pressing selects it and releasing there closes the popup.
    let second = tree.menu_item(menu, 1).unwrap();
    let (x, y) = centre(tree.screen_box(second));
    click(&mut tree, x, y);
    release(&mut tree);
    assert_eq!(tree.menu_selected_index(menu), Some(1));
    assert!(!tree.menu_is_open(menu));
}

#[test]
fn a_selection_shows_the_item_and_broadcasts_its_element_id() {
    let (mut tree, _panel, menu, display) = tree_with(TEXT);
    fill(&mut tree, menu);
    tree.take_messages();
    let second = tree.menu_item(menu, 1).unwrap();
    tree.menu_set_selected_index(menu, Some(1));
    assert_eq!(tree.menu_selected_index(menu), Some(1));
    assert_eq!(tree.menu_selected_item(menu), Some(second));
    assert_eq!(tree.get(display).unwrap().text().as_deref(), Some("No"));
    let messages: Vec<_> = tree
        .take_messages()
        .into_iter()
        .filter(|m| m.message == MSG_MENU_NEW_SELECTION && m.element == Some(menu))
        .collect();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].element, Some(menu));
    assert_eq!(messages[0].data, tree.get(second).unwrap().id() as i32);
    // No selection: the display is cleared and the broadcast carries no id.
    tree.menu_set_selected_index(menu, None);
    assert_eq!(tree.get(display).unwrap().text().as_deref(), Some(""));
    let cleared: Vec<_> = tree
        .take_messages()
        .into_iter()
        .filter(|m| m.message == MSG_MENU_NEW_SELECTION && m.element == Some(menu))
        .collect();
    assert_eq!(cleared.len(), 1);
    assert_eq!(cleared[0].data, 0);
}

#[test]
fn ghosting_keeps_the_menu_shut_and_closes_it_when_set() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    tree.set_bool(menu, tokenize("ghosted"), true);
    assert_eq!(tree.get(menu).unwrap().state(), tokenize("ghosted"));
    let drop = tree.screen_box(menu);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    assert!(!tree.menu_is_open(menu));
    tree.set_bool(menu, tokenize("ghosted"), false);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    assert!(tree.menu_is_open(menu));
    tree.set_bool(menu, tokenize("ghosted"), true);
    assert!(!tree.menu_is_open(menu));
}

#[test]
fn the_state_follows_the_open_popup_and_the_pointer_over_the_top() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    let drop = tree.screen_box(menu);
    point_at(&mut tree, drop.x0 + 2, drop.y0 + 2);
    assert_eq!(tree.get(menu).unwrap().state(), tokenize("rollover"));
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    assert_eq!(tree.get(menu).unwrap().state(), tokenize("open"));
    release(&mut tree);
    // Open wins over the pointer; a press outside closes it and the state is closed again.
    point_at(&mut tree, 250, 150);
    assert_eq!(tree.get(menu).unwrap().state(), tokenize("open"));
    click(&mut tree, 250, 150);
    release(&mut tree);
    assert_eq!(tree.get(menu).unwrap().state(), tokenize("closed"));
}

#[test]
fn hiding_the_menu_closes_its_popup() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    let popup = popup_of(&tree, menu);
    let drop = tree.screen_box(menu);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    tree.set_visible(menu, false);
    assert!(!tree.menu_is_open(menu));
    assert!(!tree.roots().contains(&popup));
}

#[test]
fn removing_the_menu_removes_its_popup() {
    let (mut tree, panel, menu, _display) = tree_with(TEXT);
    fill(&mut tree, menu);
    let popup = popup_of(&tree, menu);
    let drop = tree.screen_box(menu);
    click(&mut tree, drop.x0 + 2, drop.y0 + 2);
    release(&mut tree);
    tree.remove_root(panel);
    assert!(tree.get(popup).is_none());
    assert!(!tree.roots().contains(&popup));
}

#[test]
fn a_text_item_is_refused_unless_its_template_is_a_text_element() {
    let (mut tree, _panel, menu, _display) = tree_with(FIELD);
    assert_eq!(tree.menu_insert_text(menu, "Yes", 0), None);
    assert!(tree.menu_items(menu).is_empty());
}

#[test]
fn items_are_inserted_at_their_place_and_flushed() {
    let (mut tree, _panel, menu, _display) = tree_with(TEXT);
    let first = tree.menu_insert_string(menu, 0x4100_0018, 7, 0).unwrap();
    let second = tree.menu_insert_text(menu, "Second", 1).unwrap();
    let inserted_front = tree.menu_insert_text(menu, "Front", 0).unwrap();
    assert_eq!(tree.menu_items(menu), [inserted_front, first, second]);
    assert_eq!(tree.get(first).unwrap().text().as_deref(), Some("string 7"));
    assert_eq!(tree.menu_item(menu, 2), Some(second));
    tree.menu_set_selected_item(menu, Some(second));
    assert_eq!(tree.menu_selected_index(menu), Some(2));
    tree.menu_flush(menu);
    assert!(tree.menu_items(menu).is_empty());
    assert_eq!(tree.menu_selected_item(menu), None);
}
