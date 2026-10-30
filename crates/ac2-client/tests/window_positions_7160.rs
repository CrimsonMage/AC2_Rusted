//! The window exception (`AGENTS.md`) on the HUD rig over the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter): Alt + drag moves a visible chat window, its place is written to
//! the player's file when the drag ends, a rebuilt HUD (a relog) puts it back where it was left,
//! and the save_loc window still uploads its locations to the server. A drag at a UI scale moves
//! the window by UI pixels.

#![cfg(feature = "window")]

mod hud;

use std::path::PathBuf;

use ac2_ui::UiTree;
use hud::Rig;

/// `WM_Player` 0x32, `UploadUISettings` (`ac2_net` `calls.rs`).
const UPLOAD_UI_SETTINGS: u32 = 0x8212_0031;

/// A private directory for the test's position files, under the system temp directory.
fn test_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ac2-window-positions-{}-{name}",
        std::process::id()
    ))
}

/// The radar's root (`RadarField`), a window of the gameplay HUD since the window exception covers
/// the chrome too.
fn radar_root(tree: &UiTree) -> usize {
    let id = ac2_ui::tokenize("RadarField");
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).is_some_and(|e| e.id() == id))
        .expect("the radar is a root")
}

/// The first visible save_loc window: its root and its screen box.
fn shown_window(tree: &UiTree) -> (usize, ac2_ui::UiBox, u32) {
    let saved = tree.inq_locations();
    let window = saved
        .iter()
        .find(|s| s.location.shown)
        .expect("a shown window that saves its location");
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == window.element)
        .expect("the window is a root");
    (root, tree.screen_box(root), window.element)
}

fn uploads(rig: &Rig) -> usize {
    rig.server
        .borrow()
        .script_bytes
        .iter()
        .filter(|(f, _)| *f == UPLOAD_UI_SETTINGS)
        .count()
}

/// Alt + drag of the window `root` by `by` window pixels, pressed on a point of the window that is
/// the topmost element there (chrome can cover a window). `scale` is the window's pixels per UI
/// pixel; the window's UI box is read at the press.
fn alt_drag(rig: &mut Rig, root: usize, scale: f32, by: (i32, i32)) {
    let b = rig.view.hud_tree().unwrap().screen_box(root);
    let mut press = None;
    'search: for y in (b.y0 + 4..b.y1 - 4).step_by(6) {
        for x in (b.x0 + 4..b.x1 - 4).step_by(6) {
            let at = (
                (x as f32 * scale).round() as i32,
                (y as f32 * scale).round() as i32,
            );
            rig.point(at);
            let tree = rig.view.hud_tree().unwrap();
            if tree
                .mouse_top()
                .is_some_and(|top| tree.root_of(top) == root)
            {
                press = Some(at);
                break 'search;
            }
        }
    }
    let from = press.expect("a point of the window is on top");
    // The press follows the search's frame directly: a fresh pointer frame could move the chrome
    // (the radar's blips) over the point again.
    rig.alt = true;
    rig.button(true);
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        rig.point((
            from.0 + (by.0 as f32 * t).round() as i32,
            from.1 + (by.1 as f32 * t).round() as i32,
        ));
    }
    rig.release();
    rig.alt = false;
}

#[test]
fn an_alt_drag_keeps_a_window_where_it_was_left_and_still_uploads_its_locations() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    let dir = test_dir("keep");
    rig.client.set_window_positions_dir(Some(dir.clone()));
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    let (root, _, element) = shown_window(rig.view.hud_tree().unwrap());
    let before = uploads(&rig);

    // Into bounds first: the window starts partly off the display, and a move keeps it on it.
    alt_drag(&mut rig, root, 1.0, (-475, -20));
    let start = rig.view.hud_tree().unwrap().screen_box(root);
    alt_drag(&mut rig, root, 1.0, (60, 40));
    let moved = rig.view.hud_tree().unwrap().screen_box(root);
    let (dx, dy) = (moved.x0 - start.x0, moved.y0 - start.y0);
    assert!(
        (dx - 60).abs() <= 1 && (dy - 40).abs() <= 1,
        "moved {dx},{dy}"
    );

    // The player's file holds every window's place, this one included.
    let player = rig.client.frame().player.expect("the player").id;
    let kept = rig
        .client
        .local_window_positions(player, ac2_ui::HudStyle::Classic)
        .unwrap();
    let expected = rig.view.hud_tree().unwrap().window_positions();
    assert_eq!(kept.len(), expected.len());
    let mine = kept
        .iter()
        .find(|p| p.element == element)
        .expect("the window");
    let here = expected.iter().find(|p| p.element == element).unwrap();
    assert!((mine.x0 - here.x0).abs() < 1e-6 && (mine.y0 - here.y0).abs() < 1e-6);

    // The save_loc window still uploads to the server (`UploadUISettings`).
    assert!(uploads(&rig) > before, "the moved save_loc window uploads");

    // A relog builds the HUD again: the window opens where it was left.
    rig.rebuild_hud();
    let (_, again, _) = shown_window(rig.view.hud_tree().unwrap());
    assert_eq!(
        (again.x0, again.y0),
        (moved.x0, moved.y0),
        "the window is where it was left"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_alt_drag_moves_a_window_by_ui_pixels_at_a_scaled_window() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    // A 1280 x 960 window lays the UI out at 1024 x 768, scaled by 1.25.
    rig.view.resize(1280, 960);
    rig.idle(4);
    let dir = test_dir("scale");
    rig.client.set_window_positions_dir(Some(dir.clone()));
    let (root, _, _) = shown_window(rig.view.hud_tree().unwrap());
    // Into bounds (UI pixels), then 125 window pixels: 100 UI pixels.
    alt_drag(&mut rig, root, 1.25, (-594, -25));
    let start = rig.view.hud_tree().unwrap().screen_box(root);
    alt_drag(&mut rig, root, 1.25, (125, 100));
    let moved = rig.view.hud_tree().unwrap().screen_box(root);
    let (dx, dy) = (moved.x0 - start.x0, moved.y0 - start.y0);
    assert!(
        (dx - 100).abs() <= 1 && (dy - 80).abs() <= 1,
        "moved {dx},{dy}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_alt_drag_moves_the_radar_and_its_place_is_kept() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    let dir = test_dir("radar");
    rig.client.set_window_positions_dir(Some(dir.clone()));
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    let root = radar_root(rig.view.hud_tree().unwrap());
    let start = rig.view.hud_tree().unwrap().screen_box(root);
    // Toward the middle of the display, so the move stays on it.
    let by = (
        if start.x0 < 300 { 40 } else { -40 },
        if start.y0 < 300 { 30 } else { -30 },
    );
    alt_drag(&mut rig, root, 1.0, by);
    let moved = rig.view.hud_tree().unwrap().screen_box(root);
    let (dx, dy) = (moved.x0 - start.x0, moved.y0 - start.y0);
    assert!(
        (dx - by.0).abs() <= 1 && (dy - by.1).abs() <= 1,
        "moved {dx},{dy}"
    );
    let player = rig.client.frame().player.expect("the player").id;
    let kept = rig
        .client
        .local_window_positions(player, ac2_ui::HudStyle::Classic)
        .unwrap();
    assert!(
        kept.iter()
            .any(|p| p.element == ac2_ui::tokenize("RadarField")),
        "the radar's place is in the player's file"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_fresh_login_puts_a_moved_window_back_where_it_was_left() {
    let dir = test_dir("login");
    // The first session moves the radar and ends; its place is in the player's file.
    let left = {
        let Some(mut rig) = Rig::new_in(Some(dir.clone())) else {
            return;
        };
        rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
        let root = radar_root(rig.view.hud_tree().unwrap());
        let start = rig.view.hud_tree().unwrap().screen_box(root);
        let by = (
            if start.x0 < 300 { 40 } else { -40 },
            if start.y0 < 300 { 30 } else { -30 },
        );
        alt_drag(&mut rig, root, 1.0, by);
        rig.view.hud_tree().unwrap().screen_box(root)
    };
    // A fresh login (a new client over the same character's file) opens the radar there.
    let Some(mut rig) = Rig::new_in(Some(dir.clone())) else {
        let _ = std::fs::remove_dir_all(&dir);
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    let again = rig
        .view
        .hud_tree()
        .unwrap()
        .screen_box(radar_root(rig.view.hud_tree().unwrap()));
    assert_eq!(
        (again.x0, again.y0),
        (left.x0, left.y0),
        "the radar opens where it was left"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The paper doll's 3D viewport (`PaperDollViewport`), shown with the inventory panel.
fn doll_viewport(rig: &Rig) -> ac2_ui::SceneViewport {
    rig.view
        .paper_doll_viewport()
        .expect("the doll shows with the inventory panel")
}

#[test]
fn an_alt_drag_of_the_inventory_carries_the_paper_doll_viewport_by_the_same_offset() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    let dir = test_dir("doll");
    rig.client.set_window_positions_dir(Some(dir.clone()));
    rig.open_inventory();
    rig.wait("the doll's viewport", |r| {
        r.view.paper_doll_viewport().is_some()
    });
    // The inventory window: the root of the head slot, the doll's docking window.
    let tree = rig.view.hud_tree().unwrap();
    let slot = ac2_ui::tokenize("Head_Slot");
    let inventory = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.child_recursive(r, slot).is_some())
        .expect("the inventory window holds the head slot");
    let window_before = tree.screen_box(inventory);
    let viewport_before = doll_viewport(&rig);

    // Into bounds first, then the drag to measure.
    alt_drag(&mut rig, inventory, 1.0, (-200, -40));
    let start_w = rig.view.hud_tree().unwrap().screen_box(inventory);
    let start_v = doll_viewport(&rig);
    alt_drag(&mut rig, inventory, 1.0, (60, 40));
    let moved_w = rig.view.hud_tree().unwrap().screen_box(inventory);
    let moved_v = doll_viewport(&rig);
    let (dx, dy) = (moved_w.x0 - start_w.x0, moved_w.y0 - start_w.y0);
    assert!(
        (dx - 60).abs() <= 1 && (dy - 40).abs() <= 1,
        "the inventory moved {dx},{dy}"
    );
    assert_eq!(
        (moved_v.x - start_v.x, moved_v.y - start_v.y),
        (dx, dy),
        "the doll's viewport moved with the inventory"
    );
    // The docking offset holds: the viewport is where it was relative to the window.
    assert_eq!(
        (start_v.x - start_w.x0, start_v.y - start_w.y0),
        (
            viewport_before.x - window_before.x0,
            viewport_before.y - window_before.y0
        ),
    );
    let _ = std::fs::remove_dir_all(&dir);
}
