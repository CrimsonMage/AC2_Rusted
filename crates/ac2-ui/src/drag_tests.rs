//! Drag and drop on a hand-built layout.

use std::sync::Arc;

use crate::drag::{DragCheck, LEFT, MSG_STOP_DRAG_ITEM};
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
        did: 0,
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

/// Drags the icon onto the target and returns the `stop_drag_item` (element, data).
fn drag_to_target(tree: &mut UiTree) -> (Option<usize>, i32) {
    point(tree, 15, 15);
    tree.start_input(LEFT, 1.0);
    point(tree, 20, 15);
    assert!(tree.dragged().is_some(), "a drag");
    point(tree, 120, 20);
    tree.take_messages();
    tree.stop_input(LEFT);
    let drop = tree
        .take_messages()
        .into_iter()
        .find(|m| m.message == MSG_STOP_DRAG_ITEM)
        .expect("stop_drag_item");
    (drop.element, drop.data)
}

/// `UIElement::CatchDroppedItem`: the container's callback refuses what it does not take, and
/// the drop then reaches the owner as one outside (`DragAndDropComplete`).
#[test]
fn a_refusing_drop_callback_sends_the_drop_back_to_the_owner() {
    for (check, attributes, accepted) in [
        (DragCheck::ItemDrop, vec![], false),
        (DragCheck::ItemDrop, vec![(0x4000_0001, 0)], false),
        (DragCheck::ItemDrop, vec![(0x4000_0002, 0)], true),
        // `gmIngredientUI_DDCB` wants the item to come from a container.
        (DragCheck::IngredientDrop, vec![(0x4000_0002, 0)], false),
        (DragCheck::IngredientDrop, vec![(0x4000_0002, 0x55)], true),
        // `gm3DItems_DDCB` too; `ContainerDisplay_DDCB` does not read it.
        (DragCheck::WorldDrop, vec![(0x4000_0002, 0)], false),
        (DragCheck::WorldDrop, vec![(0x4000_0002, 0x55)], true),
        (DragCheck::ContainerDrop, vec![(0x4000_0002, 0)], true),
        (DragCheck::ContainerDrop, vec![(0x4000_0003, 0)], false),
    ] {
        let (mut tree, icon, target) = tree();
        tree.set_drag_drop_check(target, Some(check));
        for &(kind, from) in &attributes {
            tree.set_attribute_id(icon, tokenize("Token_ShortcutType"), kind);
            if from != 0 {
                tree.set_instance_attribute(icon, tokenize("FromID"), from);
            }
        }
        let owner = tree.get(icon).unwrap().parent;
        let (element, data) = drag_to_target(&mut tree);
        if accepted {
            assert_eq!(
                (element, data),
                (Some(target), 1),
                "{check:?} {attributes:?}"
            );
        } else {
            assert_eq!((element, data), (owner, 0), "{check:?} {attributes:?}");
        }
    }
}

/// `UIElement::DragItem`: the owner's drag-out callback can refuse the drag.
#[test]
fn a_refusing_drag_out_callback_keeps_the_icon_in_place() {
    let (mut tree, icon, _) = tree();
    let owner = tree.get(icon).unwrap().parent.unwrap();
    // The shortcut bar's callback wants a `ShortcutIndex` below 100.
    tree.set_drag_out_check(owner, Some(DragCheck::ShortcutOut));
    point(&mut tree, 15, 15);
    tree.start_input(LEFT, 1.0);
    point(&mut tree, 20, 15);
    assert!(tree.dragged().is_none(), "no ShortcutIndex, no drag out");
    tree.stop_input(LEFT);
    tree.set_attribute_id(icon, tokenize("ShortcutIndex"), 0);
    tree.set_drag_out_check(owner, None);
    point(&mut tree, 15, 15);
    tree.start_input(LEFT, 1.0);
    point(&mut tree, 20, 15);
    assert!(tree.dragged().is_some(), "with no callback the drag starts");
}
