//! The window exception (`AGENTS.md`): Alt + drag moves a HUD window that has no drag bar, the
//! window stays on the display, a resize keeps it there, and the saved places round-trip through
//! the file's text and apply to the windows of a fresh HUD. The HUD cases need `AC2_DAT_DIR`.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::element::{UiBox, UiTree};
use crate::hud::build_gameplay_hud;
use crate::mouse::{UI_POINTER_X, UI_POINTER_Y};
use crate::token::{DRAGBAR, DRAGBAR_DISABLED};
use crate::window_positions::{parse, to_text, WindowPosition};
use crate::GameplayHud;

const LEFT_DOWN: u32 = 0x4000_0004;

fn sample() -> Vec<WindowPosition> {
    vec![
        WindowPosition {
            layout: 0x4100_0009,
            element: 0x0123_4567,
            x0: -0.5,
            y0: 0.1,
            x1: -0.1,
            y1: 0.7,
        },
        WindowPosition {
            layout: 0x2200_0005,
            element: 0x89AB_CDEF,
            x0: 0.333_333_34,
            y0: -1e-7,
            x1: 0.9,
            y1: 1.0,
        },
    ]
}

#[test]
fn the_positions_file_round_trips_exactly() {
    let saved = sample();
    let text = to_text(&saved);
    assert!(text.starts_with('#'), "a header comment: {text}");
    assert_eq!(parse(&text).unwrap(), saved);
}

#[test]
fn a_malformed_positions_line_is_an_error_and_comments_are_skipped() {
    let text = "# header\n\nwindow 41000009 1234 0 0 1 1\n";
    assert_eq!(parse(text).unwrap().len(), 1);
    assert!(parse("window 41000009 1234 0 0 1").is_err());
    assert!(parse("panel 41000009 1234 0 0 1 1").is_err());
    assert!(parse("window zz 1234 0 0 1 1").is_err());
}

fn load(store: &Arc<ResourceStore>) -> GameplayHud {
    build_gameplay_hud(store, 1024, 768).unwrap()
}

/// Every window in the HUD has a drag bar. The test takes a visible window and disables its drag
/// bars (`DRAGBAR_DISABLED`, as `dragbar_mouse_down` reads it): a window with no working bar.
fn bar_less_window(tree: &mut UiTree) -> usize {
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.is_window_root(r) && tree.get(r).is_some_and(|el| el.visible()))
        .expect("a visible window");
    disable_drag_bars(tree, root);
    root
}

fn disable_drag_bars(tree: &mut UiTree, index: usize) {
    for child in tree.children(index).to_vec() {
        if tree.get(child).is_some_and(|el| el.kind() == DRAGBAR) {
            tree.set_bool(child, DRAGBAR_DISABLED, true);
        }
        disable_drag_bars(tree, child);
    }
}

fn left_press(tree: &mut UiTree, at: (i32, i32)) {
    tree.start_input(UI_POINTER_X, at.0 as f32);
    tree.start_input(UI_POINTER_Y, at.1 as f32);
    tree.start_input(LEFT_DOWN, 1.0);
}

fn pointer_to(tree: &mut UiTree, at: (i32, i32)) {
    tree.start_input(UI_POINTER_X, at.0 as f32);
    tree.start_input(UI_POINTER_Y, at.1 as f32);
}

fn left_release(tree: &mut UiTree) {
    tree.stop_input(LEFT_DOWN);
}

#[test]
fn alt_drag_moves_a_window_without_a_drag_bar() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = load(&store);
    let tree = &mut hud.tree;
    let root = bar_less_window(tree);
    let start = tree.screen_box(root);
    // The press lands on the window itself: the window is the top element under the pointer.
    let at = (start.x0 + 10, start.y0 + 10);
    tree.set_alt(true);
    pointer_to(tree, at);
    tree.update_mouse();
    let top = tree.mouse_top().expect("something under the pointer");
    assert_eq!(tree.root_of(top), root, "the press lands on the window");
    // Toward the middle of the display, so the move is not clamped at an edge.
    let by = (
        if start.x0 < 300 { 40 } else { -40 },
        if start.y0 < 300 { 30 } else { -30 },
    );
    left_press(tree, at);
    pointer_to(tree, (at.0 + by.0, at.1 + by.1));
    left_release(tree);
    let moved = tree.screen_box(root);
    assert_eq!((moved.x0 - start.x0, moved.y0 - start.y0), by);
    assert!(tree.take_positions_changed(), "the drag ended with a move");
    assert!(!tree.take_positions_changed(), "taken once");
}

#[test]
fn without_alt_a_window_without_a_drag_bar_stays_put() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = load(&store);
    let tree = &mut hud.tree;
    let root = bar_less_window(tree);
    let start = tree.screen_box(root);
    let at = (start.x0 + 10, start.y0 + 10);
    left_press(tree, at);
    pointer_to(tree, (at.0 + 40, at.1 + 30));
    left_release(tree);
    assert_eq!(tree.screen_box(root), start);
    assert!(!tree.take_positions_changed());
}

#[test]
fn a_window_moved_past_the_display_edge_is_clamped_on_it() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = load(&store);
    let tree = &mut hud.tree;
    let root = bar_less_window(tree);
    let start = tree.screen_box(root);
    let at = (start.x0 + 10, start.y0 + 10);
    tree.set_alt(true);
    left_press(tree, at);
    pointer_to(tree, (5000, 5000));
    left_release(tree);
    let (w, h) = (1024, 768);
    let b = tree.screen_box(root);
    assert!(b.x0 >= 0 && b.y0 >= 0, "{b:?}");
    assert!(b.x1 < w && b.y1 < h, "{b:?}");
    // Dragged past the top-left corner: clamped there too.
    left_press(tree, (b.x0 + 5, b.y0 + 5));
    pointer_to(tree, (-500, -500));
    left_release(tree);
    let b = tree.screen_box(root);
    assert_eq!((b.x0, b.y0), (0, 0));
}

#[test]
fn a_resized_display_keeps_the_window_on_screen() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = load(&store);
    let tree = &mut hud.tree;
    let root = bar_less_window(tree);
    let start = tree.screen_box(root);
    let at = (start.x0 + 10, start.y0 + 10);
    tree.set_alt(true);
    left_press(tree, at);
    pointer_to(tree, (1000, 740));
    left_release(tree);
    tree.set_display(640, 480);
    let b: UiBox = tree.screen_box(root);
    assert!(b.x0 >= 0 && b.y0 >= 0, "{b:?}");
    assert!(b.x1 < 640 && b.y1 < 480, "{b:?}");
}

#[test]
fn saved_positions_apply_to_the_windows_of_a_fresh_hud_and_skip_the_overlays() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let first = load(&store);
    let saved = first.tree.window_positions();
    assert!(!saved.is_empty(), "the HUD has windows");
    assert!(
        saved
            .iter()
            .any(|p| p.element == crate::tokenize("RadarField")),
        "the radar keeps its place"
    );
    for overlay in ["TargettingReticle", "HeadingField", "NameTextTemplate"] {
        assert!(
            saved.iter().all(|p| p.element != crate::tokenize(overlay)),
            "{overlay} is an overlay and keeps no place"
        );
    }
    let mut second = load(&store);
    let target = saved[0];
    let root = second
        .tree
        .roots()
        .iter()
        .copied()
        .find(|&r| second.tree.get(r).unwrap().id() == target.element)
        .expect("the window is a root");
    second.tree.move_to(root, 5, 5);
    second.tree.set_window_positions(&saved);
    let again = second.tree.window_positions();
    assert_eq!(again.len(), saved.len());
    for (a, b) in saved.iter().zip(again.iter()) {
        assert_eq!((a.layout, a.element), (b.layout, b.element));
        assert!((a.x0 - b.x0).abs() < 1e-3, "{a:?} {b:?}");
        assert!((a.y0 - b.y0).abs() < 1e-3, "{a:?} {b:?}");
    }
}

#[test]
fn alt_drag_moves_the_radar_as_a_window() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = load(&store);
    let tree = &mut hud.tree;
    let radar = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == crate::tokenize("RadarField"))
        })
        .expect("the radar is a root");
    let start = tree.screen_box(radar);
    // A press at the radar's centre: the radar is the window under the pointer.
    let at = ((start.x0 + start.x1) / 2, (start.y0 + start.y1) / 2);
    tree.set_alt(true);
    pointer_to(tree, at);
    tree.update_mouse();
    left_press(tree, at);
    pointer_to(tree, (at.0 - 30, at.1 + 20));
    left_release(tree);
    let moved = tree.screen_box(radar);
    assert_eq!((moved.x0 - start.x0, moved.y0 - start.y0), (-30, 20));
    assert!(tree.take_positions_changed(), "the radar's place is kept");
    let saved = tree.window_positions();
    assert!(saved
        .iter()
        .any(|p| p.element == crate::tokenize("RadarField")));
}
