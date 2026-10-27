//! Saved window locations over the gameplay HUD built from real data (`AC2_DAT_DIR`): the
//! windows that save their location are filed, a drag bar moves its window and raises
//! `location_changed`, and a saved location applies to a fresh HUD at the same place.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::hud::build_gameplay_hud;
use crate::mouse::{UI_POINTER_X, UI_POINTER_Y};
use crate::token::{tokenize, DRAGBAR};
use crate::GameplayHud;

const LEFT_DOWN: u32 = 0x4000_0004;

fn load(store: &Arc<ResourceStore>) -> GameplayHud {
    build_gameplay_hud(store, 1024, 768).unwrap()
}

#[test]
fn the_hud_saves_its_windows_and_a_saved_location_applies_to_a_fresh_hud() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let first = load(&store);
    let saved = first.tree.inq_locations();
    assert!(
        !saved.is_empty(),
        "the HUD has windows that save their location"
    );
    assert!(
        saved.iter().all(|s| s.layout != 0),
        "layouts are keyed by DID"
    );

    // A fresh HUD with one saved window moved away: the saved locations put every window back.
    let mut second = load(&store);
    let target = saved[0];
    let root = second
        .tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            let el = second.tree.get(r).unwrap();
            el.id() == target.element
        })
        .expect("the saved window is a root");
    second.tree.move_to(root, 5, 5);
    assert!(second.tree.apply_locations(&saved));
    let again = second.tree.inq_locations();
    assert_eq!(again.len(), saved.len());
    for (a, b) in saved.iter().zip(again.iter()) {
        assert_eq!((a.layout, a.element), (b.layout, b.element));
        assert_eq!(a.location.shown, b.location.shown);
        assert!((a.location.x0 - b.location.x0).abs() < 1e-3, "{a:?} {b:?}");
        assert!((a.location.y0 - b.location.y0).abs() < 1e-3, "{a:?} {b:?}");
    }
}

#[test]
fn a_drag_bar_moves_its_window_and_raises_location_changed() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = load(&store);
    // The chat windows' drag bars; the one a press at its centre reaches is the one that moves.
    let bars: Vec<(usize, usize)> = hud
        .tree
        .roots()
        .iter()
        .copied()
        .filter_map(|r| {
            hud.tree
                .child_recursive(r, tokenize("MoveBar"))
                .map(|b| (r, b))
        })
        .collect();
    assert!(!bars.is_empty(), "the chat windows have drag bars");
    let (window, bar, cx, cy) = bars
        .iter()
        .find_map(|&(window, bar)| {
            let centre = hud.tree.screen_box(bar);
            let (cx, cy) = ((centre.x0 + centre.x1) / 2, (centre.y0 + centre.y1) / 2);
            hud.tree.start_input(UI_POINTER_X, cx as f32);
            hud.tree.start_input(UI_POINTER_Y, cy as f32);
            (hud.tree.mouse_top() == Some(bar)).then_some((window, bar, cx, cy))
        })
        .expect("a drag bar is under the pointer");
    assert_eq!(
        hud.tree.get(bar).unwrap().kind(),
        DRAGBAR,
        "MoveBar is a drag bar"
    );
    // The drag bar moves its parent (`UIElement_Dragbar::MouseDown` -> `StartMovement`).
    assert_eq!(hud.tree.get(bar).unwrap().parent_index(), Some(window));
    hud.tree.take_locations_changed();

    let before = hud.tree.screen_box(window);
    hud.tree.start_input(LEFT_DOWN, 0.0);
    hud.tree.start_input(UI_POINTER_X, (cx + 20) as f32);
    hud.tree.start_input(UI_POINTER_Y, (cy + 10) as f32);
    hud.tree.stop_input(LEFT_DOWN);
    let after = hud.tree.screen_box(window);
    assert_eq!(after.x0 - before.x0, 20, "the window follows the pointer");
    assert_eq!(after.y0 - before.y0, 10);
    assert!(hud.tree.take_locations_changed());
    assert!(!hud.tree.take_locations_changed(), "taken once");
}
