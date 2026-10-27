//! A list box and its scroll bar, and the list's keyboard stepping (`list_scroll.rs`).

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::{UiTree, UI_WHEEL_OUT};

fn element(
    id: &str,
    kind: u32,
    rect: Rect,
    attributes: Vec<(&str, AttributeValue)>,
    children: Vec<ElementDesc>,
) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind,
        default_state: STATELESS,
        rect,
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
            attributes: attributes
                .into_iter()
                .map(|(name, value)| AttributeDesc {
                    id: tokenize(name),
                    value,
                })
                .collect(),
            media: Vec::new(),
            transitions: Vec::new(),
            children: children.into_iter().map(Arc::new).collect(),
        }],
    }
}

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

/// A panel holding a 30 x 20 list of one column of 10 x 10 items, and beside it the scroll bar
/// the list names as its `vertical_scrollbar`. Five items make a paper 50 pixels tall, so the
/// list scrolls 30 pixels (`paper - height`).
/// Returns the tree, the list, the scroll bar and the layout.
fn panel() -> (UiTree, usize, usize, Arc<LayoutDesc>) {
    let list = element(
        "List",
        tokenize("listbox"),
        rect(0, 0, 29, 19),
        vec![
            ("max_columns", AttributeValue::Int(1)),
            ("vertical_scrollbar", AttributeValue::Id(tokenize("Bar"))),
        ],
        vec![],
    );
    let bar = element(
        "Bar",
        tokenize("scrollbar"),
        rect(40, 0, 49, 19),
        vec![],
        vec![],
    );
    let panel = element(
        "Panel",
        0x4DA7_2ECC,
        rect(0, 0, 59, 19),
        vec![],
        vec![list, bar],
    );
    let item = element("Item", 0x4DA7_2ECC, rect(0, 0, 9, 9), vec![], vec![]);
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 60,
        height: 20,
        elements: vec![Arc::new(panel), Arc::new(item)],
    });
    let mut tree = UiTree::new(100, 100);
    let root = tree
        .create_root(Arc::clone(&layout), tokenize("Panel"))
        .unwrap();
    let list = tree.child_recursive(root, tokenize("List")).unwrap();
    let bar = tree.child_recursive(root, tokenize("Bar")).unwrap();
    let item_desc = Arc::clone(layout.element(tokenize("Item")).unwrap());
    for _ in 0..5 {
        let item = tree
            .create_element(Arc::clone(&layout), Arc::clone(&item_desc), false)
            .unwrap();
        tree.list_add_item(list, item);
    }
    (tree, list, bar, layout)
}

fn view_y(tree: &UiTree, list: usize) -> i32 {
    tree.get(list).unwrap().list.as_ref().unwrap().view.1
}

#[test]
fn the_scroll_bar_and_the_list_view_follow_each_other() {
    let (mut tree, list, bar, _) = panel();
    // Half way down the 30 pixels of scroll (the 50 pixel paper less the list's 20 pixels):
    // 15, snapped to the row boundary at 20. A bare `SetAttribute_Float` only lays the widget
    // out; the list follows the bar's move.
    tree.scrollbar_set_position(bar, 0.5);
    assert_eq!(view_y(&tree, list), 0);
    tree.lists_follow_scrollbar(bar);
    assert_eq!(view_y(&tree, list), 20);
    // Scrolling the list to the end of its 30 pixels of scroll moves the bar to the bottom.
    tree.list_set_view(list, None, Some(30));
    assert_eq!(view_y(&tree, list), 30);
    let share = tree.scrollbar_position(bar);
    assert!((share - 1.0).abs() < 1e-6, "bar at {share}");
    tree.list_set_view(list, None, Some(20));
    let share = tree.scrollbar_position(bar);
    assert!((share - 20.0 / 30.0).abs() < 1e-6, "bar at {share}");
}

#[test]
fn a_wheel_turn_over_the_scroll_bar_moves_the_list_a_line() {
    let (mut tree, list, bar, _) = panel();
    tree.scrollbar_mouse_wheel(bar, UI_WHEEL_OUT);
    assert_eq!(view_y(&tree, list), 10);
    tree.scrollbar_mouse_wheel(bar, crate::UI_WHEEL_IN);
    assert_eq!(view_y(&tree, list), 0);
    // A wheel turn at the top stays at the top.
    tree.scrollbar_mouse_wheel(bar, crate::UI_WHEEL_IN);
    assert_eq!(view_y(&tree, list), 0);
}

#[test]
fn a_wheel_turn_over_the_list_itself_moves_it_through_its_bar() {
    let (mut tree, list, _, _) = panel();
    // The pointer over the list's rows (not its bar): the wheel goes to the list
    // (`ListBox::MouseDown`), which turns it through its bar.
    tree.start_input(crate::UI_POINTER_X, 10.0);
    tree.start_input(crate::UI_POINTER_Y, 10.0);
    assert!(tree.get(list).unwrap().mouse_over);
    tree.start_input(UI_WHEEL_OUT, 1.0);
    assert_eq!(view_y(&tree, list), 10);
}

#[test]
fn keyboard_steps_wrap_and_show_the_selected_item() {
    let (mut tree, list, _, _) = panel();
    tree.list_step_selection(list, 1);
    assert_eq!(tree.list_selected_index(list), Some(0));
    tree.list_step_selection(list, -1);
    // Back past the first item wraps to the last, which the view then shows.
    assert_eq!(tree.list_selected_index(list), Some(4));
    assert_eq!(view_y(&tree, list), 30);
    tree.list_step_selection(list, 1);
    assert_eq!(tree.list_selected_index(list), Some(0));
    assert_eq!(view_y(&tree, list), 0);
}

#[test]
fn a_wheel_over_a_child_that_does_not_scroll_reaches_the_list() {
    // A plain element in the list's top left corner, the way a row's label sits in a list: the
    // wheel goes to it first, it does not scroll, so it passes the turn on to the list.
    let pad = element("Pad", 0x4DA7_2ECC, rect(0, 0, 9, 9), vec![], vec![]);
    let list = element(
        "List",
        tokenize("listbox"),
        rect(0, 0, 29, 19),
        vec![
            ("max_columns", AttributeValue::Int(1)),
            ("vertical_scrollbar", AttributeValue::Id(tokenize("Bar"))),
        ],
        vec![pad],
    );
    let bar = element(
        "Bar",
        tokenize("scrollbar"),
        rect(40, 0, 49, 19),
        vec![],
        vec![],
    );
    let panel = element(
        "Panel",
        0x4DA7_2ECC,
        rect(0, 0, 59, 19),
        vec![],
        vec![list, bar],
    );
    let item = element("Item", 0x4DA7_2ECC, rect(0, 0, 9, 9), vec![], vec![]);
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 60,
        height: 20,
        elements: vec![Arc::new(panel), Arc::new(item)],
    });
    let mut tree = UiTree::new(100, 100);
    let root = tree
        .create_root(Arc::clone(&layout), tokenize("Panel"))
        .unwrap();
    let list = tree.child_recursive(root, tokenize("List")).unwrap();
    let pad = tree.child_recursive(root, tokenize("Pad")).unwrap();
    let item_desc = Arc::clone(layout.element(tokenize("Item")).unwrap());
    for _ in 0..5 {
        let item = tree
            .create_element(Arc::clone(&layout), Arc::clone(&item_desc), false)
            .unwrap();
        tree.list_add_item(list, item);
    }
    tree.start_input(crate::UI_POINTER_X, 5.0);
    tree.start_input(crate::UI_POINTER_Y, 5.0);
    assert!(
        tree.get(pad).unwrap().mouse_over(),
        "the pad is under the pointer"
    );
    assert!(tree.get(list).unwrap().mouse_over(), "the list is too");
    tree.start_input(UI_WHEEL_OUT, 1.0);
    assert_eq!(view_y(&tree, list), 10);
}
