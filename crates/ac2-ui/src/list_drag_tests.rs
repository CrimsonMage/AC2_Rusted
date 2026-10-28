//! `UIElement_ListBox`'s drag selection, drag rollover and smooth movement on a hand-built
//! layout (`list_drag.rs`, `list_animate.rs`).

use std::sync::Arc;

use crate::layout::{AttributeValue, LayoutDesc};
use crate::list_box_select_tests::{element, items, selections, state};
use crate::token::{tokenize, STATELESS};
use crate::UiTree;

const LEFT: u32 = 0x4000_0004;

/// A 20 x 20 list (two 10-pixel rows shown) with `attrs`, of `Item`s with normal, highlight
/// and the two rollover states, inside a root field.
fn tree(attrs: Vec<(&str, AttributeValue)>) -> (UiTree, Arc<LayoutDesc>, usize) {
    let mut list_attrs = vec![
        ("selected_item_state_change", AttributeValue::Bool(true)),
        ("item_normal_state", AttributeValue::Id(tokenize("normal"))),
        (
            "item_selected_state",
            AttributeValue::Id(tokenize("highlight")),
        ),
    ];
    list_attrs.extend(attrs);
    let list = element(
        "List",
        "listbox",
        19,
        19,
        vec![state(STATELESS, list_attrs)],
    );
    let item = element(
        "Item",
        "field",
        19,
        9,
        [
            "normal",
            "highlight",
            "drag_rollover_accept",
            "drag_rollover_reject",
        ]
        .into_iter()
        .map(|name| state(tokenize(name), vec![]))
        .chain([state(STATELESS, vec![])])
        .collect(),
    );
    let mut root_state = state(STATELESS, vec![]);
    root_state.children.push(list);
    let root = element("Root", "field", 99, 99, vec![root_state]);
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 100,
        elements: vec![root, item],
    });
    let mut tree = UiTree::new(100, 100);
    let root = tree
        .create_root(Arc::clone(&layout), tokenize("Root"))
        .unwrap();
    let list = tree.child_recursive(root, tokenize("List")).unwrap();
    (tree, layout, list)
}

fn point(tree: &mut UiTree, x: f32, y: f32) {
    tree.start_input(crate::UI_POINTER_X, x);
    tree.start_input(crate::UI_POINTER_Y, y);
}

fn view(tree: &UiTree, list: usize) -> (i32, i32) {
    tree.get(list).unwrap().list.as_ref().unwrap().view
}

#[test]
fn a_drag_select_list_selects_under_the_pointer_until_the_button_comes_up() {
    let (mut tree, layout, list) = tree(vec![
        ("click_select", AttributeValue::Bool(true)),
        ("drag_select", AttributeValue::Bool(true)),
    ]);
    let items = items(&mut tree, &layout, list, 2);
    point(&mut tree, 5.0, 5.0);
    tree.take_messages();
    tree.start_input(LEFT, 1.0);
    assert_eq!(tree.list_selected_item(list), Some(items[0]));
    point(&mut tree, 5.0, 15.0);
    assert_eq!(tree.list_selected_item(list), Some(items[1]));
    assert_eq!(selections(&mut tree, list), vec![0, 1]);
    tree.stop_input(LEFT);
    point(&mut tree, 5.0, 5.0);
    assert_eq!(
        tree.list_selected_item(list),
        Some(items[1]),
        "the selection ended"
    );
    assert!(selections(&mut tree, list).is_empty());
}

#[test]
fn without_drag_select_a_press_selects_once() {
    let (mut tree, layout, list) = tree(vec![("click_select", AttributeValue::Bool(true))]);
    let items = items(&mut tree, &layout, list, 2);
    point(&mut tree, 5.0, 5.0);
    tree.start_input(LEFT, 1.0);
    point(&mut tree, 5.0, 15.0);
    assert_eq!(tree.list_selected_item(list), Some(items[0]));
    tree.stop_input(LEFT);
}

#[test]
fn a_smooth_list_moves_to_its_goal_over_the_duration() {
    let (mut tree, layout, list) = tree(vec![
        ("smooth_movement", AttributeValue::Bool(true)),
        ("smooth_movement_duration", AttributeValue::Float(1.0)),
    ]);
    items(&mut tree, &layout, list, 5);
    tree.global_loop(10.0);
    tree.list_line_step(list, true, 1);
    assert_eq!(view(&tree, list), (0, 0), "the step only sets the goal");
    assert_eq!(tree.list_int(list, "goal_y_view"), Some(10));
    tree.global_loop(10.25);
    assert_eq!(view(&tree, list), (0, 2), "a quarter of the way, truncated");
    // A second step while moving goes on from the goal, not the view.
    tree.list_line_step(list, true, 1);
    assert_eq!(tree.list_int(list, "goal_y_view"), Some(20));
    tree.global_loop(12.0);
    assert_eq!(view(&tree, list), (0, 20));
    assert_eq!(tree.list_int(list, "goal_y_view"), Some(-1_000_000));
    assert!(tree
        .get(list)
        .unwrap()
        .list
        .as_ref()
        .unwrap()
        .anim
        .is_none());
}

#[test]
fn a_view_set_from_elsewhere_stops_the_movement() {
    let (mut tree, layout, list) = tree(vec![
        ("smooth_movement", AttributeValue::Bool(true)),
        ("smooth_movement_duration", AttributeValue::Float(1.0)),
    ]);
    items(&mut tree, &layout, list, 5);
    tree.global_loop(0.0);
    tree.list_line_step(list, true, 1);
    tree.global_loop(0.5);
    assert_eq!(view(&tree, list), (0, 5));
    tree.list_set_view(list, None, Some(0));
    tree.global_loop(1.0);
    assert_eq!(view(&tree, list), (0, 0), "stopped");
    assert_eq!(tree.list_int(list, "goal_y_view"), Some(-1_000_000));
}

#[test]
fn a_list_without_smooth_movement_jumps_to_the_goal() {
    let (mut tree, layout, list) = tree(vec![]);
    items(&mut tree, &layout, list, 5);
    tree.list_line_step(list, true, 1);
    assert_eq!(view(&tree, list), (0, 10));
    assert_eq!(tree.list_int(list, "goal_y_view"), Some(-1_000_000));
}

#[test]
fn letting_go_of_the_scroll_bar_goes_to_the_nearest_item() {
    let (mut tree, layout, list) = tree(vec![
        ("smooth_movement", AttributeValue::Bool(true)),
        ("smooth_movement_duration", AttributeValue::Float(1.0)),
    ]);
    items(&mut tree, &layout, list, 5);
    tree.list_set_view(list, None, Some(16));
    tree.global_loop(0.0);
    tree.list_scrollbar_widget_release(list);
    assert_eq!(tree.list_int(list, "goal_y_view"), Some(20));
    tree.global_loop(1.0);
    assert_eq!(view(&tree, list), (0, 20));
}

#[test]
fn a_drag_over_a_container_list_rolls_over_its_items() {
    let (mut tree, layout, list) = tree(vec![
        ("container", AttributeValue::Bool(true)),
        ("drag_rollover", AttributeValue::Bool(true)),
    ]);
    let items = items(&mut tree, &layout, list, 2);
    for &item in &items {
        tree.el_mut(item).mouse_visible = false;
    }
    let dragged = items[1];
    point(&mut tree, 50.0, 50.0);
    tree.drag.element = Some(dragged);
    // Coming over the top starts the rollover; the items change on the next move.
    point(&mut tree, 5.0, 5.0);
    assert_eq!(tree.get(items[0]).unwrap().state(), STATELESS);
    point(&mut tree, 5.0, 6.0);
    let accept = tokenize("drag_rollover_accept");
    assert_eq!(tree.get(items[0]).unwrap().state(), accept);
    point(&mut tree, 5.0, 15.0);
    assert_eq!(tree.get(items[0]).unwrap().state(), STATELESS, "put back");
    assert_eq!(tree.get(items[1]).unwrap().state(), accept);
    // `disallow_drag_in` rejects.
    tree.el_mut(list)
        .attributes
        .insert(tokenize("disallow_drag_in"), AttributeValue::Bool(true));
    point(&mut tree, 5.0, 5.0);
    assert_eq!(
        tree.get(items[0]).unwrap().state(),
        tokenize("drag_rollover_reject")
    );
    // Any button coming up ends the rollover and puts the item back.
    tree.stop_input(LEFT);
    assert_eq!(tree.get(items[0]).unwrap().state(), STATELESS);
    tree.drag.element = None;
}

#[test]
fn a_list_steps_by_the_buttons_it_names() {
    let (mut tree, layout, list) = tree(vec![]);
    items(&mut tree, &layout, list, 5);
    let down = tree
        .child_recursive(tree.root_of(list), tokenize("List"))
        .unwrap();
    // The list itself stands in for a button: only its id matters to the list.
    tree.el_mut(list).attributes.insert(
        tokenize("down_button"),
        AttributeValue::Id(tokenize("List")),
    );
    tree.list_hot_click(down);
    assert_eq!(view(&tree, list), (0, 10));
    tree.el_mut(list)
        .attributes
        .insert(tokenize("up_button"), AttributeValue::Id(tokenize("List")));
    tree.el_mut(list)
        .attributes
        .remove(&tokenize("down_button"));
    tree.list_hot_click(down);
    assert_eq!(view(&tree, list), (0, 0));
}
