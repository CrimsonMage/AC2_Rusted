//! `UIElement_ListBox` on a hand-built layout.

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::UiTree;

fn element(id: &str, r: Rect, attrs: Vec<(&str, AttributeValue)>) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind: tokenize("listbox"),
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

/// A 30 x 20 list of 10 x 10 items, three to a row when horizontal.
fn tree(horizontal: bool) -> (UiTree, Arc<LayoutDesc>, usize) {
    let list = element(
        "List",
        Rect {
            x0: 0,
            y0: 0,
            x1: 29,
            y1: 19,
        },
        vec![
            ("max_columns", AttributeValue::Int(3)),
            ("horizontal", AttributeValue::Bool(horizontal)),
        ],
    );
    let item = element(
        "Item",
        Rect {
            x0: 0,
            y0: 0,
            x1: 9,
            y1: 9,
        },
        vec![],
    );
    let layout = Arc::new(LayoutDesc {
        sub_ids: Vec::new(),
        string_table: 0,
        width: 30,
        height: 20,
        elements: vec![Arc::new(list), Arc::new(item)],
    });
    let mut tree = UiTree::new(100, 100);
    let root = tree
        .create_root(Arc::clone(&layout), tokenize("List"))
        .unwrap();
    (tree, layout, root)
}

fn item(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> usize {
    let desc = Arc::clone(layout.element(tokenize("Item")).unwrap());
    tree.create_element(Arc::clone(layout), desc, false)
        .unwrap()
}

fn at(tree: &UiTree, i: usize) -> (i32, i32) {
    let r = tree.get(i).unwrap().rect;
    (r.x0, r.y0)
}

#[test]
fn a_horizontal_list_fills_rows_and_scrolls_by_whole_rows() {
    let (mut tree, layout, list) = tree(true);
    let items: Vec<usize> = (0..5).map(|_| item(&mut tree, &layout)).collect();
    tree.list_multi_op_start(list);
    for &i in &items {
        tree.list_add_item(list, i);
    }
    tree.list_multi_op_complete(list);
    let placed: Vec<_> = items.iter().map(|&i| at(&tree, i)).collect();
    assert_eq!(placed, vec![(0, 0), (10, 0), (20, 0), (0, 10), (10, 10)]);
    assert_eq!(tree.list_index_at(list, 15, 12), Some(4));
    assert_eq!(tree.list_index_at(list, 25, 12), None, "an empty cell");
    // A view snaps to the nearest row.
    tree.list_set_view(list, None, Some(7));
    assert_eq!(tree.list_top_row(list), 1);
    assert_eq!(at(&tree, items[3]), (0, 0));
    assert_eq!(
        tree.take_messages().last().map(|m| m.message),
        Some(tokenize("new_view"))
    );
    // Removing an item closes the gap; deleting one frees it.
    tree.list_home(list);
    assert_eq!(tree.list_remove_item(list, 1), Some(items[1]));
    assert_eq!(at(&tree, items[2]), (10, 0));
    assert!(tree.list_delete_item(list, items[2]));
    assert!(tree.get(items[2]).is_none());
    assert_eq!(tree.list_what_num(list, items[4]), Some(2));
    tree.list_flush(list);
    assert!(tree.list_items(list).is_empty());
}

#[test]
fn a_vertical_list_fills_columns() {
    let (mut tree, layout, list) = tree(false);
    let items: Vec<usize> = (0..4).map(|_| item(&mut tree, &layout)).collect();
    for &i in &items {
        tree.list_add_item(list, i);
    }
    // Three columns, two rows: down each column.
    let placed: Vec<_> = items.iter().map(|&i| at(&tree, i)).collect();
    assert_eq!(placed, vec![(0, 0), (0, 10), (10, 0), (10, 10)]);
}
