//! `UIElement_Meter` and `VitalDisplay` on a hand-built layout.

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::{UiTree, VitalDisplay};

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

fn element(
    id: &str,
    kind: &str,
    r: Rect,
    attrs: Vec<(&str, AttributeValue)>,
    children: Vec<ElementDesc>,
) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind: tokenize(kind),
        default_state: STATELESS,
        rect: r,
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: rect(-1, -1, -1, -1),
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

/// A 200 x 50 field with a 100 x 10 meter at (10, 5) whose fill is as large as the meter.
fn tree(fill: &str, attrs: Vec<(&str, AttributeValue)>) -> (UiTree, usize, usize) {
    let fill = element(fill, "field", rect(0, 0, 99, 9), vec![], vec![]);
    let meter = element("Meter", "meter", rect(10, 5, 109, 14), attrs, vec![fill]);
    let field = element("Field", "field", rect(0, 0, 199, 49), vec![], vec![meter]);
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 200,
        height: 50,
        elements: vec![Arc::new(field)],
    });
    let mut tree = UiTree::new(640, 480);
    let root = tree.create_root(layout, tokenize("Field")).unwrap();
    let meter = tree.child_recursive(root, tokenize("Meter")).unwrap();
    let fill = tree.children(meter)[0];
    (tree, meter, fill)
}

fn rect_of(tree: &UiTree, index: usize) -> (i32, i32, i32, i32) {
    let r = tree.get(index).unwrap().rect;
    (r.x0, r.y0, r.width(), r.height())
}

const GOAL: u32 = tokenize("goal_position");

#[test]
fn a_moving_fill_slides_to_the_goal_at_once() {
    let (mut tree, meter, fill) = tree(
        "left_fill",
        vec![
            ("smooth_movement", AttributeValue::Bool(false)),
            ("move_fill", AttributeValue::Bool(true)),
            ("goal_position", AttributeValue::Float(0.0)),
        ],
    );
    // Empty: the fill sits a whole width to the left, clipped away.
    assert_eq!(rect_of(&tree, fill), (-100, 0, 100, 10));
    tree.set_float(meter, GOAL, 0.25);
    assert_eq!(rect_of(&tree, fill), (-75, 0, 100, 10));
    // The offset truncates toward zero: 0.33 as a float is a little over 0.33, so
    // -((1 - 0.33) * 100) is -66.99... and lands on -66.
    tree.set_float(meter, GOAL, 0.33);
    assert_eq!(rect_of(&tree, fill).0, -66);
}

#[test]
fn a_right_fill_shows_what_is_missing() {
    let (mut tree, meter, fill) = tree(
        "right_fill",
        vec![
            ("smooth_movement", AttributeValue::Bool(false)),
            ("move_fill", AttributeValue::Bool(true)),
        ],
    );
    let mut vital = VitalDisplay::new(&tree, Some(meter), None);
    vital.update(&mut tree, 30, 120, &|_, _| String::new());
    // 30 of 120 leaves 0.75 missing: the right fill starts 75 pixels in.
    assert_eq!(rect_of(&tree, fill), (75, 0, 100, 10));
    // More than the maximum shows full.
    vital.update(&mut tree, 500, 120, &|_, _| String::new());
    assert_eq!(rect_of(&tree, fill).0, 0);
}

#[test]
fn a_smooth_meter_moves_over_its_duration() {
    let (mut tree, meter, fill) = tree(
        "left_fill",
        vec![
            ("smooth_movement", AttributeValue::Bool(true)),
            ("smooth_movement_duration", AttributeValue::Float(1.0)),
            ("move_fill", AttributeValue::Bool(true)),
        ],
    );
    tree.global_loop(10.0);
    tree.set_float(meter, GOAL, 1.0);
    assert_eq!(rect_of(&tree, fill).0, -100);
    tree.global_loop(10.5);
    assert_eq!(rect_of(&tree, fill).0, -50);
    tree.global_loop(11.25);
    assert_eq!(rect_of(&tree, fill).0, 0);
}

#[test]
fn a_meter_switches_its_fill_states_as_it_passes_them() {
    let (mut tree, meter, fill) = tree(
        "left_fill",
        vec![
            ("smooth_movement", AttributeValue::Bool(true)),
            ("smooth_movement_duration", AttributeValue::Float(1.0)),
            ("move_fill", AttributeValue::Bool(true)),
        ],
    );
    let (low, mid) = (tokenize("state_low"), tokenize("state_mid"));
    tree.meter_set_state_at_percentage(meter, low, 0.0);
    tree.meter_set_state_at_percentage(meter, mid, 0.5);
    tree.global_loop(0.0);
    tree.set_float(meter, GOAL, 1.0);
    tree.global_loop(0.25);
    assert_eq!(tree.get(fill).unwrap().state(), low);
    tree.global_loop(0.75);
    assert_eq!(tree.get(fill).unwrap().state(), mid);
}

#[test]
fn a_cut_fill_is_placed_at_the_meters_corner_in_its_surface() {
    // Without `move_fill` retail cuts the fill and moves it to the meter's own place in the
    // surface, inside the meter (no DAT meter does this).
    let (mut tree, meter, fill) = tree(
        "left_fill",
        vec![("smooth_movement", AttributeValue::Bool(false))],
    );
    tree.set_float(meter, GOAL, 0.5);
    assert_eq!(rect_of(&tree, fill), (10, 5, 50, 10));
}
