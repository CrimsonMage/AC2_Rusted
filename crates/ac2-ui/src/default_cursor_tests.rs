//! The pointer's cursor goes back to the default (`UIElementManager::UpdateMouse`,
//! `UseDefaultCursor`) once the pointer is off whatever set one, on the real HUD
//! (`AC2_DAT_DIR`): after a hover, a window move, a resize and the targeting pointer.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::default_cursor::default_cursor_did;
use crate::token::RESIZEBAR;
use crate::window_locations::UI_LEFT_BUTTON;
use crate::{build_gameplay_hud, set_targeting_pointer, UiTree, UI_POINTER_X, UI_POINTER_Y};

fn hud() -> Option<(Arc<ResourceStore>, UiTree)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    Some((store, tree))
}

fn point(tree: &mut UiTree, (x, y): (i32, i32)) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

/// A point where nothing sets a cursor: over no element, or over one without a cursor.
fn open_ground(tree: &mut UiTree) -> (i32, i32) {
    for y in (100..700).step_by(20) {
        for x in (100..900).step_by(20) {
            point(tree, (x, y));
            if tree.mouse_top().is_none_or(|t| tree.el(t).cursor.0 == 0) {
                return (x, y);
            }
        }
    }
    panic!("no open ground on the display");
}

/// The first visible resize grip with a cursor of its own, and a point on it where it is on top.
fn grip(tree: &mut UiTree) -> Option<((i32, i32), usize)> {
    for i in 0..tree.slots_len() {
        let Some(el) = tree.get(i) else { continue };
        if el.kind() != RESIZEBAR || !el.visible() || el.cursor.0 == 0 {
            continue;
        }
        let b = tree.screen_box(i);
        for y in (b.y0..=b.y1).step_by(2) {
            for x in (b.x0..=b.x1).step_by(2) {
                point(tree, (x, y));
                if tree.mouse_top() == Some(i) {
                    return Some(((x, y), i));
                }
            }
        }
    }
    None
}

#[test]
fn the_ui_has_the_flows_default_cursor_from_the_start() {
    let Some((store, tree)) = hud() else { return };
    let did = default_cursor_did(&store)
        .unwrap()
        .expect("the enum names it");
    assert_eq!(tree.pointer_cursor(), Some((did, 0, 0)));
}

#[test]
fn a_cursor_an_element_set_goes_when_the_pointer_leaves_it() {
    let Some((store, mut tree)) = hud() else {
        return;
    };
    let default = (default_cursor_did(&store).unwrap().unwrap(), 0, 0);
    let (on_grip, grip) = grip(&mut tree).expect("a resize grip with a cursor");
    let own = tree.el(grip).cursor;
    assert_ne!(own.0, default.0);
    point(&mut tree, on_grip);
    assert_eq!(
        tree.pointer_cursor(),
        Some(own),
        "the grip's cursor over it"
    );
    let away = open_ground(&mut tree);
    assert_eq!(
        tree.pointer_cursor(),
        Some(default),
        "the default off it at {away:?}"
    );
}

/// A resize that ends with the pointer off the grip (the grip kept the press while the window
/// followed the pointer) leaves the default cursor.
#[test]
fn the_cursor_is_the_default_after_a_resize() {
    let Some((store, mut tree)) = hud() else {
        return;
    };
    let default = (default_cursor_did(&store).unwrap().unwrap(), 0, 0);
    let (on_grip, _) = grip(&mut tree).expect("a resize grip with a cursor");
    tree.start_input(UI_LEFT_BUTTON, 1.0);
    for step in 1..=4 {
        point(&mut tree, (on_grip.0, on_grip.1 - 6 * step));
    }
    let away = open_ground(&mut tree);
    tree.stop_input(UI_LEFT_BUTTON);
    point(&mut tree, away);
    assert_eq!(tree.pointer_cursor(), Some(default));
}

/// A window moved by Alt + drag (`alt_move_mouse_down`), then the pointer over empty ground.
#[test]
fn the_cursor_is_the_default_after_a_window_move() {
    let Some((store, mut tree)) = hud() else {
        return;
    };
    let default = (default_cursor_did(&store).unwrap().unwrap(), 0, 0);
    let (on_grip, _) = grip(&mut tree).expect("a resize grip with a cursor");
    tree.set_alt(true);
    tree.start_input(UI_LEFT_BUTTON, 1.0);
    for step in 1..=4 {
        point(&mut tree, (on_grip.0 + 5 * step, on_grip.1 - 5 * step));
    }
    tree.stop_input(UI_LEFT_BUTTON);
    tree.set_alt(false);
    let away = open_ground(&mut tree);
    point(&mut tree, away);
    assert_eq!(tree.pointer_cursor(), Some(default));
}

/// `gmSelectionUI::RecvNotice_SetTargetingPointer`: the usage cursor while targeting, the old
/// one (the default) back when it ends.
#[test]
fn the_targeting_pointer_gives_the_cursor_back() {
    let Some((store, mut tree)) = hud() else {
        return;
    };
    let default = (default_cursor_did(&store).unwrap().unwrap(), 0, 0);
    let away = open_ground(&mut tree);
    set_targeting_pointer(&mut tree, true);
    point(&mut tree, away);
    let targeting = tree.pointer_cursor();
    set_targeting_pointer(&mut tree, false);
    point(&mut tree, away);
    assert_eq!(
        tree.pointer_cursor(),
        Some(default),
        "targeting showed {targeting:?}"
    );
}
