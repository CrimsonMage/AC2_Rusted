//! `UIElement_Scrollbar` on a hand-built layout: a slider with stops and step buttons.

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::{UiTree, MSG_NEW_POSITION, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_IN};

fn element(
    id: &str,
    kind: &str,
    r: (i16, i16, i16, i16),
    attrs: Vec<(&str, AttributeValue)>,
    children: Vec<Arc<ElementDesc>>,
) -> Arc<ElementDesc> {
    Arc::new(ElementDesc {
        id: tokenize(id),
        kind: tokenize(kind),
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
            children,
        }],
    })
}

/// A root with a 100 x 10 slider at (100, 0) (five stops, a 20-pixel widget) and its two step
/// buttons.
fn tree() -> (UiTree, usize) {
    tree_with_widget("field")
}

/// The same slider, its widget an element of kind `widget_kind` (a `field`, or a `button` as a
/// layout may make it).
fn tree_with_widget(widget_kind: &str) -> (UiTree, usize) {
    let slider = element(
        "Slider",
        "scrollbar",
        (100, 0, 199, 9),
        vec![
            ("horizontal", AttributeValue::Bool(true)),
            ("move_to_touched_position", AttributeValue::Bool(true)),
            ("stop_locations", AttributeValue::Bool(true)),
            ("num_stop_locations", AttributeValue::Int(5)),
            ("buttons_interact", AttributeValue::Bool(true)),
            ("left_button", AttributeValue::Id(tokenize("Left"))),
            ("right_button", AttributeValue::Id(tokenize("Right"))),
        ],
        vec![element(
            "widget",
            widget_kind,
            (0, 0, 19, 9),
            vec![],
            vec![],
        )],
    );
    let hot = || vec![("hot_button", AttributeValue::Bool(true))];
    let left = element("Left", "button", (0, 20, 9, 29), hot(), vec![]);
    let right = element("Right", "button", (20, 20, 29, 29), hot(), vec![]);
    let root = element(
        "Root",
        "field",
        (0, 0, 299, 99),
        vec![],
        vec![slider, left, right],
    );
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 300,
        height: 100,
        elements: vec![root],
    });
    let mut tree = UiTree::new(300, 100);
    let root = tree.create_root(layout, tokenize("Root")).unwrap();
    let slider = tree.child_recursive(root, tokenize("Slider")).unwrap();
    (tree, slider)
}

fn point(tree: &mut UiTree, x: i32, y: i32) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

fn moves(tree: &mut UiTree) -> usize {
    tree.take_messages()
        .iter()
        .filter(|m| m.message == MSG_NEW_POSITION)
        .count()
}

fn widget_x(tree: &UiTree, slider: usize) -> i32 {
    let widget = tree.child_recursive(slider, tokenize("widget")).unwrap();
    tree.get(widget).unwrap().rect.x0
}

#[test]
fn the_slider_jumps_snaps_drags_and_steps() {
    let (mut tree, slider) = tree();
    assert_eq!(tree.scrollbar_position(slider), 0.0);
    // A press on the track at 160: the widget's middle there, (160 - 10 - 100) / 80 = 0.625,
    // snapped down to the 0.5 stop; the widget at 40.
    point(&mut tree, 160, 5);
    tree.take_messages();
    tree.start_input(0x4000_0004, 1.0);
    assert_eq!(tree.scrollbar_position(slider), 0.5);
    assert_eq!(widget_x(&tree, slider), 40);
    assert_eq!(moves(&mut tree), 1);
    // The drag keeps the pointer 20 pixels into the widget: at 150 the widget is at 30,
    // 30 / 80 = 0.375, snapped to 0.25.
    point(&mut tree, 150, 5);
    assert_eq!(tree.scrollbar_position(slider), 0.25);
    tree.stop_input(0x4000_0004);
    moves(&mut tree);
    point(&mut tree, 60, 5);
    assert_eq!(tree.scrollbar_position(slider), 0.25, "no longer dragged");
    // The right button steps one stop on, the left one back.
    point(&mut tree, 25, 25);
    tree.start_input(0x4000_0004, 1.0);
    tree.stop_input(0x4000_0004);
    assert_eq!(tree.scrollbar_position(slider), 0.5);
    assert_eq!(moves(&mut tree), 1);
    point(&mut tree, 5, 25);
    for _ in 0..3 {
        tree.start_input(0x4000_0004, 1.0);
        tree.stop_input(0x4000_0004);
    }
    assert_eq!(tree.scrollbar_position(slider), 0.0);
    assert_eq!(moves(&mut tree), 2, "nothing past the first stop");
}

/// A wheel turn over the widget (the thumb) reaches its bar, as one over the track does: the
/// widget is a child that takes the pointer over the bar (`UIElement_Scrollbar::MouseDown` runs
/// for the bar under it, `HandleMouseWheel`). Both a plain widget and a button widget.
fn wheel_over_the_thumb_steps_the_bar(widget_kind: &str) {
    let (mut tree, slider) = tree_with_widget(widget_kind);
    tree.scrollbar_set_position(slider, 0.5);
    let widget = tree.child_recursive(slider, tokenize("widget")).unwrap();
    // The thumb is at x 40..59 in the track (100..199): its middle is at 150.
    let thumb = tree.screen_box(widget);
    point(&mut tree, (thumb.x0 + thumb.x1) / 2, 5);
    assert_eq!(
        tree.mouse_top(),
        Some(widget),
        "the pointer is over the thumb"
    );
    tree.take_messages();
    tree.start_input(UI_WHEEL_IN, 1.0);
    assert_eq!(
        tree.scrollbar_position(slider),
        0.25,
        "a turn in over the thumb steps the bar back one stop ({widget_kind})"
    );
    assert_eq!(moves(&mut tree), 1);
}

#[test]
fn the_wheel_over_the_thumb_reaches_the_bar() {
    wheel_over_the_thumb_steps_the_bar("field");
    wheel_over_the_thumb_steps_the_bar("button");
}

/// `UIElement_Button::StartUseActive` clicks at once; `Global_Loop` repeats the click from
/// 0.3 s on, then every 0.1 s (counted from the click due), until the release.
#[test]
fn a_held_hot_button_repeats_its_click() {
    let (mut tree, slider) = tree();
    tree.global_loop(10.0);
    // Stop locations 0, 0.25 .. 1: put the bar at 1 and hold the left button (a step back).
    tree.scrollbar_set_position(slider, 1.0);
    point(&mut tree, 4, 24);
    tree.start_input(0x4000_0004, 1.0);
    assert_eq!(
        tree.scrollbar_position(slider),
        0.75,
        "the press clicks once"
    );
    tree.global_loop(10.2);
    assert_eq!(
        tree.scrollbar_position(slider),
        0.75,
        "before the first interval"
    );
    tree.global_loop(10.31);
    assert_eq!(tree.scrollbar_position(slider), 0.5);
    tree.global_loop(10.39);
    assert_eq!(
        tree.scrollbar_position(slider),
        0.5,
        "the next click is due at 10.4"
    );
    tree.global_loop(10.41);
    assert_eq!(tree.scrollbar_position(slider), 0.25);
    tree.stop_input(0x4000_0004);
    tree.global_loop(11.0);
    assert_eq!(
        tree.scrollbar_position(slider),
        0.25,
        "released: no more clicks"
    );
}

/// `SetAttribute_Bool` / `_Float` run `UpdateLayout`: a bar set `disabled` with
/// `hide_when_disabled` hides, enabled again it shows; a `proportional` bar's widget follows a
/// new `x_widget_size`.
#[test]
fn a_changed_layout_attribute_lays_the_bar_out_again() {
    let (mut tree, slider) = tree();
    let widget = tree.child_recursive(slider, tokenize("widget")).unwrap();
    tree.set_bool(slider, tokenize("hide_when_disabled"), true);
    tree.set_bool(slider, tokenize("disabled"), true);
    assert!(!tree.get(slider).unwrap().visible);
    tree.set_bool(slider, tokenize("disabled"), false);
    assert!(tree.get(slider).unwrap().visible);
    tree.set_bool(slider, tokenize("proportional"), true);
    tree.set_float(slider, tokenize("x_widget_size"), 0.5);
    let rect = tree.get(widget).unwrap().rect;
    assert_eq!(rect.x1 - rect.x0 + 1, 50);
    tree.set_int(slider, tokenize("min_widget_width"), 70);
    let rect = tree.get(widget).unwrap().rect;
    assert_eq!(rect.x1 - rect.x0 + 1, 70);
}
