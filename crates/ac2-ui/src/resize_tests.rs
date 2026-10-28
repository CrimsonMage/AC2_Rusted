//! Resize grips: the border a grip drags, the box a drag gives (its limits and the display), and
//! the grips of the real gameplay HUD and of the chat strip's layout (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;

use crate::element::{UiBox, UiTree};
use crate::hud::build_gameplay_hud;
use crate::layout::load_layout;
use crate::mouse::{UI_POINTER_X, UI_POINTER_Y};
use crate::resize::{border_of, resized_box, Border, SizeLimits};
use crate::window_positions::WindowPosition;

const LEFT_DOWN: u32 = 0x4000_0004;

/// The gameplay HUD's window with the grip under its top edge (layout `0x22000001`).
const STRIP_WINDOW: u32 = 0x9877_4723;
const STRIP_TOP_GRIP: u32 = 0xc9a6_f0d2;
/// A text element of the strip, which the window's resize re-anchors.
const STRIP_TEXT: u32 = 0xea2a_0b6d;
/// The chat strip (layout `0x22000005`): a window whose bottom grip sets its height.
const CHAT_WINDOW: u32 = 0xa0f7_92c9;
const CHAT_BOTTOM_GRIP: u32 = 0x7122_c1ba;

#[test]
fn a_grip_drags_the_edges_its_attributes_name() {
    // `UIElement_Resizebar::MouseDown`: the right edge decides first, then the top and bottom.
    assert_eq!(border_of(false, false, false, false), None);
    assert_eq!(border_of(false, false, false, true), Some(Border::Bottom));
    assert_eq!(border_of(false, false, true, false), Some(Border::Top));
    assert_eq!(border_of(false, false, true, true), Some(Border::Top));
    assert_eq!(border_of(true, false, false, false), Some(Border::Left));
    assert_eq!(border_of(true, false, false, true), Some(Border::LowerLeft));
    assert_eq!(border_of(true, false, true, false), Some(Border::UpperLeft));
    assert_eq!(border_of(false, true, false, false), Some(Border::Right));
    assert_eq!(
        border_of(false, true, false, true),
        Some(Border::LowerRight)
    );
    assert_eq!(
        border_of(false, true, true, false),
        Some(Border::UpperRight)
    );
    // A right edge takes the grip whatever its left edge says.
    assert_eq!(border_of(true, true, false, false), Some(Border::Right));
    assert_eq!(border_of(true, true, true, true), Some(Border::UpperRight));
    // The enum's values (`BorderLocation`).
    assert_eq!(Border::LowerLeft as u32, 7);
    assert_eq!(Border::Left as u32, 8);
}

fn sized_box() -> UiBox {
    UiBox::new(100, 100, 199, 199)
}

#[test]
fn a_top_edge_keeps_its_height_within_the_limits() {
    let limits = SizeLimits {
        min_height: Some(60),
        max_height: Some(120),
        ..SizeLimits::default()
    };
    let display = (1024, 768);
    // Up 50: 150 tall, past the 120 maximum, so the top stops at 80 (120 tall).
    let up = resized_box(Border::Top, sized_box(), (0, -50), limits, display);
    assert_eq!((up.y0, up.y1, up.height()), (80, 199, 120));
    // Down 90: past the 60 minimum, so the top stops at 140 (60 tall).
    let down = resized_box(Border::Top, sized_box(), (0, 90), limits, display);
    assert_eq!((down.y0, down.y1, down.height()), (140, 199, 60));
    // A top edge never moves the width.
    assert_eq!((up.x0, up.x1), (100, 199));
}

#[test]
fn a_right_and_a_bottom_edge_clamp_the_far_side() {
    let limits = SizeLimits {
        min_width: Some(50),
        max_width: Some(110),
        min_height: Some(15),
        max_height: Some(200),
    };
    let display = (1024, 768);
    let wide = resized_box(Border::Right, sized_box(), (30, 0), limits, display);
    assert_eq!((wide.x0, wide.x1), (100, 209), "the maximum width of 110");
    let narrow = resized_box(Border::Right, sized_box(), (-80, 0), limits, display);
    assert_eq!(
        (narrow.x0, narrow.x1),
        (100, 149),
        "the minimum width of 50"
    );
    let taller = resized_box(Border::Bottom, sized_box(), (0, 40), limits, display);
    assert_eq!((taller.y0, taller.y1), (100, 239));
    let tallest = resized_box(Border::Bottom, sized_box(), (0, 400), limits, display);
    assert_eq!(
        (tallest.y0, tallest.y1),
        (100, 299),
        "the maximum height of 200"
    );
    let shortest = resized_box(Border::Bottom, sized_box(), (0, -200), limits, display);
    assert_eq!(shortest.height(), 15, "the minimum height of 15");
}

#[test]
fn a_corner_moves_both_of_its_edges_and_the_display_keeps_the_box() {
    let display = (1024, 768);
    let none = SizeLimits::default();
    let corner = resized_box(Border::LowerRight, sized_box(), (5, 7), none, display);
    assert_eq!(
        (corner.x0, corner.y0, corner.x1, corner.y1),
        (100, 100, 204, 206)
    );
    let upper_left = resized_box(Border::UpperLeft, sized_box(), (-10, -10), none, display);
    assert_eq!((upper_left.x0, upper_left.y0), (90, 90));
    assert_eq!((upper_left.x1, upper_left.y1), (199, 199));
    // Past the display's right and top edges the box stops on the display.
    let edge = UiBox::new(900, 10, 1000, 300);
    let off = resized_box(Border::UpperRight, edge, (200, -50), none, display);
    assert_eq!((off.x1, off.y0), (1023, 0));
}

/// The top-level element with `id` in the tree.
fn root_by_id(tree: &UiTree, id: u32) -> usize {
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).is_some_and(|e| e.id() == id))
        .expect("the window is a root")
}

fn child_by_id(tree: &UiTree, parent: usize, id: u32) -> usize {
    tree.child_recursive(parent, id).expect("the child")
}

fn pointer_to(tree: &mut UiTree, at: (i32, i32)) {
    tree.start_input(UI_POINTER_X, at.0 as f32);
    tree.start_input(UI_POINTER_Y, at.1 as f32);
}

/// A grip drag: the press at `from`, the pointer walked to `from + by` in steps, the release.
fn grip_drag(tree: &mut UiTree, from: (i32, i32), by: (i32, i32)) {
    pointer_to(tree, from);
    tree.start_input(LEFT_DOWN, 1.0);
    for step in 1..=4 {
        pointer_to(tree, (from.0 + by.0 * step / 4, from.1 + by.1 * step / 4));
    }
    tree.stop_input(LEFT_DOWN);
}

#[test]
fn a_top_grip_resizes_the_strip_within_its_height_limits_and_relays_its_children() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let window = root_by_id(tree, STRIP_WINDOW);
    let text = child_by_id(tree, window, STRIP_TEXT);
    let grip = child_by_id(tree, window, STRIP_TOP_GRIP);
    assert_eq!(
        tree.screen_box(grip).y0,
        tree.screen_box(window).y0,
        "the grip is on the top edge"
    );
    let start = tree.screen_box(window);
    let text_start = tree.screen_box(text).height();
    assert_eq!(start.height(), 44, "the strip opens at its minimum height");

    // Up 60 from the top strip: 104 tall, inside the 44 to 172 limits.
    let press = (start.x0 + 100, start.y0 + 2);
    pointer_to(tree, press);
    tree.update_mouse();
    let top = tree.mouse_top().expect("the grip is under the pointer");
    assert_eq!(tree.root_of(top), window, "the press lands on the strip");
    grip_drag(tree, press, (0, -60));
    let up = tree.screen_box(window);
    assert_eq!((up.y1, up.height()), (start.y1, 104));
    // The text under the grip keeps its top and gains the strip's extra height at its bottom.
    assert_eq!(tree.screen_box(text).height(), text_start + 60);
    assert!(
        tree.take_positions_changed(),
        "the resize ended with a change"
    );

    // Down past the minimum: the strip stops at 44 tall with its bottom edge where it was.
    let press = (up.x0 + 100, up.y0 + 2);
    grip_drag(tree, press, (0, 200));
    assert_eq!(tree.screen_box(window).height(), 44);
    // Up past the maximum: 172 tall.
    let press = (start.x0 + 100, tree.screen_box(window).y0 + 2);
    grip_drag(tree, press, (0, -400));
    let tallest = tree.screen_box(window);
    assert_eq!((tallest.height(), tallest.y1), (172, start.y1));
}

#[test]
fn a_resized_window_keeps_its_size_in_the_saved_places_and_opens_at_it_again() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let window = root_by_id(tree, STRIP_WINDOW);
    let start = tree.screen_box(window);
    let press = (start.x0 + 100, start.y0 + 2);
    grip_drag(tree, press, (0, -60));
    let resized = tree.screen_box(window);
    let saved: Vec<WindowPosition> = tree.window_positions();
    assert!(
        saved.iter().any(|p| p.element == STRIP_WINDOW),
        "the window's place (its size included) is saved"
    );

    // A fresh HUD applies the saved place: the strip is 104 tall again, where it was left.
    let mut fresh = build_gameplay_hud(&store, 1024, 768).unwrap();
    fresh.tree.set_window_positions(&saved);
    let again = fresh.tree.screen_box(root_by_id(&fresh.tree, STRIP_WINDOW));
    assert_eq!(
        (again.x0, again.y0, again.x1, again.y1),
        (resized.x0, resized.y0, resized.x1, resized.y1)
    );
}

#[test]
fn the_chat_strips_bottom_grip_sets_its_height_between_its_limits() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let layout = load_layout(&store, DataId::new(0x2200_0005))
        .unwrap()
        .expect("the chat strip's layout");
    let mut tree = UiTree::new(1024, 768);
    let window = tree
        .create_root(Arc::new((*layout).clone()), CHAT_WINDOW)
        .expect("the chat window");
    let grip = child_by_id(&tree, window, CHAT_BOTTOM_GRIP);
    let start = tree.screen_box(window);
    assert_eq!(tree.screen_box(grip).y1, start.y1);
    assert_eq!(start.height(), 200);
    let press = (start.x0 + 50, start.y1 - 2);
    grip_drag(&mut tree, press, (0, -100));
    let shorter = tree.screen_box(window);
    assert_eq!(
        (shorter.width(), shorter.height()),
        (200, 100),
        "the width is fixed at 200"
    );
    assert_eq!(
        (shorter.x0, shorter.y0),
        (start.x0, start.y0),
        "the top edge stays"
    );
    // Down past the maximum of 200: back to 200 tall.
    let press = (shorter.x0 + 50, shorter.y1 - 2);
    grip_drag(&mut tree, press, (0, 300));
    assert_eq!(tree.screen_box(window).height(), 200);
}
