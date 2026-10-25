//! The mouse over hand-built layouts: hit tests, presses, buttons and the world's wheel.

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::mouse::{UiMessage, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_IN};
use crate::token::{
    tokenize, BUTTON, BUTTON_BOOLEAN, BUTTON_HIGHLIGHTED, BUTTON_ROLLOVER, MOUSE_INVISIBLE,
    MSG_BUTTON_RELEASE_ON, MSG_MOUSE_OVER_TOP, MSG_MOUSE_PRESS, MSG_MOUSE_PRESS_TOP, STATELESS,
    STATE_HIGHLIGHT_ROLLOVER, STATE_NORMAL, STATE_NORMAL_PRESSED, STATE_NORMAL_ROLLOVER,
};
use crate::UiTree;

const LEFT_DOWN: u32 = 0x4000_0004;

fn bool_attr(id: u32) -> AttributeDesc {
    AttributeDesc {
        id,
        value: AttributeValue::Bool(true),
    }
}

fn element(
    name: &str,
    kind: u32,
    rect: (i16, i16, i16, i16),
    z: u16,
    attributes: Vec<AttributeDesc>,
    children: Vec<ElementDesc>,
) -> ElementDesc {
    ElementDesc {
        id: tokenize(name),
        kind,
        default_state: if kind == BUTTON {
            STATE_NORMAL
        } else {
            STATELESS
        },
        rect: Rect {
            x0: rect.0,
            y0: rect.1,
            x1: rect.2,
            y1: rect.3,
        },
        z_level: z,
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
            z_level: z,
            attributes,
            media: Vec::new(),
            transitions: Vec::new(),
            children: children.into_iter().map(Arc::new).collect(),
        }],
    }
}

const FIELD: u32 = 0x4DA7_2ECC;

/// A full-screen world field below a panel on the right holding a rollover boolean button and,
/// over its lower half, a mouse-invisible overlay.
fn tree() -> (UiTree, [usize; 4]) {
    let button = element(
        "button",
        BUTTON,
        (5, 5, 14, 14),
        0,
        vec![bool_attr(BUTTON_ROLLOVER), bool_attr(BUTTON_BOOLEAN)],
        vec![],
    );
    let overlay = element(
        "overlay",
        FIELD,
        (0, 25, 39, 49),
        0,
        vec![bool_attr(MOUSE_INVISIBLE)],
        vec![],
    );
    let panel = element(
        "panel",
        FIELD,
        (60, 0, 99, 49),
        1,
        vec![],
        vec![button, overlay],
    );
    let world = element("world", FIELD, (0, 0, 99, 49), 10, vec![], vec![]);
    let layout = Arc::new(LayoutDesc {
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 50,
        elements: vec![Arc::new(world), Arc::new(panel)],
    });
    let mut tree = UiTree::new(100, 50);
    let world = tree
        .create_root(Arc::clone(&layout), tokenize("world"))
        .unwrap();
    let panel = tree.create_root(layout, tokenize("panel")).unwrap();
    let button = tree.child_recursive(panel, tokenize("button")).unwrap();
    let overlay = tree.child_recursive(panel, tokenize("overlay")).unwrap();
    tree.take_messages();
    (tree, [world, panel, button, overlay])
}

fn point(tree: &mut UiTree, x: i32, y: i32) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

fn has(messages: &[UiMessage], element: usize, message: u32, data: i32) -> bool {
    messages
        .iter()
        .any(|m| m.element == Some(element) && m.message == message && m.data == data)
}

#[test]
fn the_button_rolls_over_presses_and_toggles_and_ends_the_press() {
    let (mut tree, [world, _panel, button, _]) = tree();
    point(&mut tree, 70, 10);
    assert_eq!(tree.mouse_top(), Some(button));
    let messages = tree.take_messages();
    assert!(has(&messages, button, MSG_MOUSE_OVER_TOP, 1));
    assert_eq!(tree.get(button).unwrap().state(), STATE_NORMAL_ROLLOVER);

    tree.start_input(LEFT_DOWN, 1.0);
    let messages = tree.take_messages();
    assert!(has(
        &messages,
        button,
        MSG_MOUSE_PRESS_TOP,
        LEFT_DOWN as i32
    ));
    assert!(has(&messages, button, MSG_MOUSE_PRESS, LEFT_DOWN as i32));
    assert!(
        !messages.iter().any(|m| m.element == Some(world)
            && matches!(m.message, MSG_MOUSE_PRESS | MSG_MOUSE_PRESS_TOP)),
        "the button ends the press before the world"
    );
    assert_eq!(tree.get(button).unwrap().state(), STATE_NORMAL_PRESSED);

    tree.stop_input(LEFT_DOWN);
    let messages = tree.take_messages();
    assert!(has(
        &messages,
        button,
        MSG_BUTTON_RELEASE_ON,
        LEFT_DOWN as i32
    ));
    // A boolean button released over it toggles its highlight.
    assert!(tree.bool_attribute(button, BUTTON_HIGHLIGHTED));
    assert_eq!(tree.get(button).unwrap().state(), STATE_HIGHLIGHT_ROLLOVER);
}

#[test]
fn the_wheel_reaches_the_world_only_where_nothing_covers_it() {
    let (mut tree, [world, panel, _, _]) = tree();
    point(&mut tree, 20, 20);
    assert_eq!(tree.mouse_top(), Some(world));
    tree.take_messages();
    tree.start_input(UI_WHEEL_IN, 1.0);
    let messages = tree.take_messages();
    assert!(has(
        &messages,
        world,
        MSG_MOUSE_PRESS_TOP,
        UI_WHEEL_IN as i32
    ));

    point(&mut tree, 90, 20);
    assert_eq!(tree.mouse_top(), Some(panel));
    tree.take_messages();
    tree.start_input(UI_WHEEL_IN, 1.0);
    let messages = tree.take_messages();
    assert!(!has(
        &messages,
        world,
        MSG_MOUSE_PRESS_TOP,
        UI_WHEEL_IN as i32
    ));
}

#[test]
fn a_mouse_invisible_element_lets_the_pointer_through() {
    let (mut tree, [_, panel, _, overlay]) = tree();
    point(&mut tree, 70, 40);
    assert_ne!(tree.mouse_top(), Some(overlay));
    assert_eq!(tree.mouse_top(), Some(panel));
}
