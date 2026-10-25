//! Drag and drop on a hand-built layout.

use std::sync::Arc;

use crate::drag::{LEFT, MSG_STOP_DRAG_ITEM};
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::{UiTree, UI_POINTER_X, UI_POINTER_Y};

fn element(
    id: &str,
    r: (i16, i16, i16, i16),
    attrs: Vec<(&str, AttributeValue)>,
    children: Vec<ElementDesc>,
) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind: tokenize("field"),
        default_state: STATELESS,
        rect: Rect {
            x0: r.0,
            y0: r.1,
            x1: r.2,
            y1: r.3,
        },
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
            children: children.into_iter().map(Arc::new).collect(),
        }],
    }
}

/// A 200 x 100 root: `Source` holding a dragable 20 x 20 `Icon` at (10, 10), and a `Target`
/// container at (100, 0).
fn tree() -> (UiTree, usize, usize) {
    let icon = element(
        "Icon",
        (10, 10, 29, 29),
        vec![("dragable", AttributeValue::Bool(true))],
        vec![],
    );
    let source = element("Source", (0, 0, 49, 49), vec![], vec![icon]);
    let target = element(
        "Target",
        (100, 0, 149, 49),
        vec![("container", AttributeValue::Bool(true))],
        vec![],
    );
    let root = element("Root", (0, 0, 199, 99), vec![], vec![source, target]);
    let layout = Arc::new(LayoutDesc {
        sub_ids: Vec::new(),
        string_table: 0,
        width: 200,
        height: 100,
        elements: vec![Arc::new(root)],
    });
    let mut tree = UiTree::new(640, 480);
    let root = tree.create_root(layout, tokenize("Root")).unwrap();
    let icon = tree.child_recursive(root, tokenize("Icon")).unwrap();
    let target = tree.child_recursive(root, tokenize("Target")).unwrap();
    (tree, icon, target)
}

fn point(tree: &mut UiTree, x: i32, y: i32) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

#[test]
fn an_icon_dragged_onto_a_container_is_caught() {
    let (mut tree, icon, target) = tree();
    point(&mut tree, 15, 15);
    tree.start_input(LEFT, 1.0);
    tree.take_messages();
    // Three pixels is not a drag yet; five is.
    point(&mut tree, 18, 15);
    assert!(tree.dragged().is_none());
    point(&mut tree, 20, 15);
    let copy = tree.dragged().expect("a drag");
    assert_ne!(copy, icon);
    assert_eq!(tree.get(copy).unwrap().id(), tokenize("Icon"));
    // The copy follows the pointer; the original stays.
    point(&mut tree, 120, 20);
    let placement = tree
        .placements()
        .into_iter()
        .find(|p| p.root == copy)
        .unwrap();
    assert_eq!((placement.x, placement.y), (120, 20));
    assert!(tree.get(icon).is_some());
    tree.take_messages();
    tree.stop_input(LEFT);
    let drop = tree
        .take_messages()
        .into_iter()
        .find(|m| m.message == MSG_STOP_DRAG_ITEM)
        .expect("stop_drag_item");
    assert_eq!(
        (drop.element, drop.data, drop.item),
        (Some(target), 1, Some(copy))
    );
    assert!(tree.dragged().is_none());
    // The copy is readable until the next global loop.
    assert!(tree.get(copy).is_some());
    tree.global_loop(1.0);
    assert!(tree.get(copy).is_none());
}

#[test]
fn a_drop_on_nothing_tells_the_owner() {
    let (mut tree, icon, _) = tree();
    tree.global_loop(0.0);
    point(&mut tree, 15, 15);
    tree.start_input(LEFT, 1.0);
    // Held long enough without moving.
    tree.global_loop(0.4);
    let copy = tree.dragged().expect("a drag after 0.3 seconds");
    point(&mut tree, 60, 80);
    tree.take_messages();
    tree.stop_input(LEFT);
    let drop = tree
        .take_messages()
        .into_iter()
        .find(|m| m.message == MSG_STOP_DRAG_ITEM)
        .expect("stop_drag_item");
    let owner = tree.get(icon).unwrap().parent;
    assert_eq!((drop.element, drop.data, drop.item), (owner, 0, Some(copy)));
}
