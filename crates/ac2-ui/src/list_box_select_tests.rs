//! `UIElement_ListBox`'s selection on a hand-built layout.

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::{UiTree, MSG_NEW_SELECTION};

pub(crate) fn state(id: u32, attrs: Vec<(&str, AttributeValue)>) -> StateDesc {
    StateDesc {
        id,
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
    }
}

pub(crate) fn element(
    id: &str,
    kind: &str,
    x1: i16,
    y1: i16,
    states: Vec<StateDesc>,
) -> Arc<ElementDesc> {
    Arc::new(ElementDesc {
        id: tokenize(id),
        kind: tokenize(kind),
        default_state: STATELESS,
        rect: Rect {
            x0: 0,
            y0: 0,
            x1,
            y1,
        },
        z_level: 0,
        edges: [0; 4],
        states,
    })
}

/// A 20 x 20 click-select list (two 10-pixel rows shown) of `Item`s with normal / highlight
/// states, inside a root field (a press on a root's direct child stops there).
fn tree() -> (UiTree, Arc<LayoutDesc>, usize) {
    let list = element(
        "List",
        "listbox",
        19,
        19,
        vec![state(
            STATELESS,
            vec![
                ("click_select", AttributeValue::Bool(true)),
                ("selected_item_state_change", AttributeValue::Bool(true)),
                ("item_normal_state", AttributeValue::Id(tokenize("normal"))),
                (
                    "item_selected_state",
                    AttributeValue::Id(tokenize("highlight")),
                ),
            ],
        )],
    );
    let item = element(
        "Item",
        "field",
        19,
        9,
        vec![
            state(STATELESS, vec![]),
            state(tokenize("normal"), vec![]),
            state(tokenize("highlight"), vec![]),
        ],
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

pub(crate) fn items(
    tree: &mut UiTree,
    layout: &Arc<LayoutDesc>,
    list: usize,
    n: usize,
) -> Vec<usize> {
    let desc = Arc::clone(layout.element(tokenize("Item")).unwrap());
    let items: Vec<usize> = (0..n)
        .map(|_| {
            tree.create_element(Arc::clone(layout), Arc::clone(&desc), false)
                .unwrap()
        })
        .collect();
    for &item in &items {
        tree.list_add_item(list, item);
    }
    items
}

pub(crate) fn selections(tree: &mut UiTree, list: usize) -> Vec<i32> {
    tree.take_messages()
        .into_iter()
        .filter(|m| m.element == Some(list) && m.message == MSG_NEW_SELECTION)
        .map(|m| m.data)
        .collect()
}

#[test]
fn selecting_an_item_highlights_it_and_broadcasts_its_index() {
    let (mut tree, layout, list) = tree();
    let items = items(&mut tree, &layout, list, 3);
    tree.take_messages();
    tree.list_set_selected_index(list, Some(1), true);
    assert_eq!(tree.list_selected_index(list), Some(1));
    assert_eq!(tree.get(items[1]).unwrap().state(), tokenize("highlight"));
    assert_eq!(selections(&mut tree, list), vec![1]);
    // The same item again changes nothing; another returns the old one to normal.
    tree.list_set_selected_index(list, Some(1), true);
    assert!(selections(&mut tree, list).is_empty());
    tree.list_set_selected_index(list, Some(2), false);
    assert_eq!(tree.get(items[1]).unwrap().state(), tokenize("normal"));
    assert!(selections(&mut tree, list).is_empty(), "not broadcast");
    // Row 2 is below the two shown rows: the view scrolls one row down.
    assert_eq!(tree.list_top_row(list), 1);
    // Past the end selects nothing (index -1).
    tree.list_set_selected_index(list, Some(9), true);
    assert_eq!(tree.list_selected_item(list), None);
    assert_eq!(selections(&mut tree, list), vec![-1]);
}

#[test]
fn a_left_press_selects_the_item_under_the_pointer() {
    let (mut tree, layout, list) = tree();
    let items = items(&mut tree, &layout, list, 2);
    tree.start_input(crate::UI_POINTER_X, 5.0);
    tree.start_input(crate::UI_POINTER_Y, 15.0);
    tree.take_messages();
    tree.start_input(0x4000_0004, 1.0);
    assert_eq!(tree.list_selected_item(list), Some(items[1]));
    assert_eq!(selections(&mut tree, list), vec![1]);
    tree.stop_input(0x4000_0004);
    // Flushing forgets the selection, so a new item in the freed slot is selected afresh.
    tree.list_flush(list);
    assert_eq!(tree.list_selected_item(list), None);
    let again = self::items(&mut tree, &layout, list, 2);
    tree.take_messages();
    tree.list_set_selected_index(list, Some(1), true);
    assert_eq!(tree.list_selected_item(list), Some(again[1]));
    assert_eq!(selections(&mut tree, list), vec![1]);
}
