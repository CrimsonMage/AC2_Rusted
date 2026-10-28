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

#[test]
fn line_and_page_steps_stay_within_the_paper() {
    // Five 10 pixel items in a 20 pixel list: 30 pixels of scroll.
    let (mut tree, list, _, _) = panel();
    tree.list_line_step(list, true, 1);
    assert_eq!(view_y(&tree, list), 10);
    // A page is the list's height less one item.
    tree.list_page_step(list, true, true);
    assert_eq!(view_y(&tree, list), 20);
    tree.list_page_step(list, true, true);
    assert_eq!(view_y(&tree, list), 30);
    tree.list_line_step(list, true, 1);
    assert_eq!(view_y(&tree, list), 30);
    tree.list_page_step(list, true, false);
    assert_eq!(view_y(&tree, list), 20);
    tree.list_page_step(list, true, false);
    tree.list_page_step(list, true, false);
    assert_eq!(view_y(&tree, list), 0);
    tree.list_line_step(list, true, -1);
    assert_eq!(view_y(&tree, list), 0);
}

#[test]
fn columns_percentage_and_selection_by_id() {
    let (mut tree, list, _, _) = panel();
    // One column, so every item is in column 0; an empty answer is -1.
    assert_eq!(tree.list_col(list, Some(3)), 0);
    assert_eq!(tree.list_col(list, None), -1);
    assert_eq!(tree.list_left_col(list), 0);
    // The view may reach the paper's width (`ConstrainViewValue` limits it to the paper).
    tree.list_scroll_to_col(list, 1);
    assert_eq!(tree.list_left_col(list), 1);
    tree.list_set_view(list, None, Some(20));
    assert!((tree.list_percentage(list, true) - 20.0 / 30.0).abs() < 1e-6);
    // The paper is no wider than the list: no horizontal share.
    assert_eq!(tree.list_percentage(list, false), 0.0);
    let item = tree.list_items(list)[2];
    let id = tree.get(item).unwrap().id();
    tree.list_set_selected_item_by_id(list, id, true);
    assert_eq!(tree.list_selected_index(list), Some(0));
    tree.list_set_selected_item_by_id(list, 0xdead_beef, true);
    assert_eq!(tree.list_selected_index(list), None);
}

/// A 30 x 20 `listboxmsize` of one column over items 10, 30 and 10 pixels tall (a 50 pixel paper).
fn sized_panel() -> (UiTree, usize) {
    let list = element(
        "List",
        tokenize("listboxmsize"),
        rect(0, 0, 29, 19),
        vec![("max_columns", AttributeValue::Int(1))],
        vec![],
    );
    let panel = element("Panel", 0x4DA7_2ECC, rect(0, 0, 59, 19), vec![], vec![list]);
    let small = element("Small", 0x4DA7_2ECC, rect(0, 0, 9, 9), vec![], vec![]);
    let tall = element("Tall", 0x4DA7_2ECC, rect(0, 0, 19, 29), vec![], vec![]);
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 60,
        height: 20,
        elements: vec![Arc::new(panel), Arc::new(small), Arc::new(tall)],
    });
    let mut tree = UiTree::new(100, 100);
    let root = tree
        .create_root(Arc::clone(&layout), tokenize("Panel"))
        .unwrap();
    let list = tree.child_recursive(root, tokenize("List")).unwrap();
    for name in ["Small", "Tall", "Small"] {
        let desc = Arc::clone(layout.element(tokenize(name)).unwrap());
        let item = tree
            .create_element(Arc::clone(&layout), desc, false)
            .unwrap();
        tree.list_add_item(list, item);
    }
    (tree, list)
}

#[test]
fn a_sized_list_places_items_by_their_running_sizes() {
    let (mut tree, list) = sized_panel();
    let state = tree.get(list).unwrap().list.as_ref().unwrap();
    assert_eq!(state.row_heights, vec![10, 30, 10]);
    assert_eq!(state.col_widths, vec![20]);
    assert_eq!(state.paper, (20, 50));
    let items = tree.list_items(list).to_vec();
    assert_eq!(tree.get(items[1]).unwrap().rect.y0, 10);
    assert_eq!(tree.get(items[2]).unwrap().rect.y0, 40);
    // A point 15 pixels down is in the tall item.
    let top = tree.screen_box(list).y0;
    assert_eq!(tree.list_index_at(list, 5, top + 15), Some(1));
    // The view is not snapped to cells, only kept within the paper less the list.
    tree.list_set_view(list, None, Some(17));
    assert_eq!(view_y(&tree, list), 17);
    tree.list_set_view(list, None, Some(99));
    assert_eq!(view_y(&tree, list), 30);
}

#[test]
fn a_sized_list_steps_by_the_cell_at_the_view_edge() {
    let (mut tree, list) = sized_panel();
    // View 0 is in the first (10 tall) row, then 10 is still in it (its end reaches 10).
    tree.list_line_step(list, true, 1);
    assert_eq!(view_y(&tree, list), 10);
    // At 10 the cell is still row 0, so the step is 10 again, within 30 (50 - 20).
    tree.list_line_step(list, true, 1);
    assert_eq!(view_y(&tree, list), 20);
    // 20 is in the tall row: a line is 30, clamped to 30.
    tree.list_line_step(list, true, 1);
    assert_eq!(view_y(&tree, list), 30);
    tree.list_line_step(list, true, -1);
    assert_eq!(view_y(&tree, list), 0);
    tree.list_scroll_to_row(list, 1);
    assert_eq!(view_y(&tree, list), 10);
}

#[test]
fn a_page_message_pages_the_list_the_bar_drives() {
    let (mut tree, list, bar, _) = panel();
    tree.scrollbar_page(bar, true, true);
    // A page is the list's 20 pixels less one 10 pixel item.
    assert_eq!(view_y(&tree, list), 10);
    // A horizontal page message does not move a list bound by its vertical bar.
    tree.scrollbar_page(bar, false, true);
    assert_eq!(view_y(&tree, list), 10);
    tree.scrollbar_page(bar, true, false);
    assert_eq!(view_y(&tree, list), 0);
}

#[test]
fn get_child_finds_a_direct_child_by_id_only() {
    let (tree, list, bar, _) = panel();
    let root = tree.get(list).unwrap().parent_index().unwrap();
    assert_eq!(tree.child(root, tokenize("Bar")), Some(bar));
    // The list's items are its children, not the panel's.
    assert_eq!(tree.child(root, tokenize("Item")), None);
    assert!(tree.child_recursive(root, tokenize("Item")).is_some());
}
