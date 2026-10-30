//! The modern Edit layout mode (`modern_layout_edit.rs`) on the real layouts (`AC2_DAT_DIR`): the
//! option panel's button, a drag on the shield moving a pinned part by its offset (kept in the
//! HUD's layout), and the bar's lock stopping every move.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::mouse::{UiMessage, GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP};
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::{build_styled_hud, GameplayHud, HudStyle, UiTree};

fn hud(style: HudStyle) -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    Some(build_styled_hud(&store, 1365, 768, style).unwrap())
}

fn named(tree: &UiTree, name: &str) -> usize {
    let id = tokenize(name);
    tree.roots()
        .iter()
        .find_map(|&r| {
            if tree.el(r).id() == id {
                Some(r)
            } else {
                tree.child_recursive(r, id)
            }
        })
        .unwrap_or_else(|| panic!("{name}"))
}

fn message(message: u32, element: Option<usize>) -> UiMessage {
    UiMessage {
        element_id: 0,
        element,
        message,
        data: 0,
        item: None,
    }
}

#[test]
fn the_option_panel_has_the_edit_layout_button_in_the_modern_style_only() {
    let (Some(modern), Some(classic)) = (hud(HudStyle::Modern), hud(HudStyle::Classic)) else {
        return;
    };
    let id = tokenize("ModernEditLayoutButton");
    let has = |tree: &UiTree| {
        tree.roots()
            .iter()
            .any(|&r| tree.child_recursive(r, id).is_some())
    };
    assert!(has(&modern.tree));
    assert!(!has(&classic.tree));
    assert!(classic.modern_edit.is_none());
}

#[test]
fn a_drag_on_the_shield_moves_the_vitals_and_is_kept_in_the_layout() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let edit = hud.modern_edit.as_ref().unwrap();
    edit.set_on(tree, true);
    let shield = named(tree, "ModernEditShield");
    assert!(tree.el(shield).visible);
    let vitals = named(tree, "StatusbarField");
    let before = tree.screen_box(vitals);
    tree.pointer = (before.x0 + 20, before.y0 + 8);
    edit.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(shield)));
    tree.pointer = (before.x0 + 70, before.y0 + 38);
    edit.message(tree, &message(GLOBAL_MOUSE_MOVE, None));
    edit.message(tree, &message(GLOBAL_MOUSE_UP, None));
    assert_eq!(tree.hud_offset("vitals"), (50, 30));
    hud.vitals.modern_tick(tree);
    let after = tree.screen_box(vitals);
    assert_eq!((after.x0 - before.x0, after.y0 - before.y0), (50, 30));
    // Kept in the layout and read back.
    let mut layout = crate::hud_layout::HudLayout::default();
    crate::modern_layout_edit::export(tree, &mut layout);
    assert_eq!(layout.get("offset.vitals"), Some("50,30"));
    tree.hud_offsets.clear();
    crate::modern_layout_edit::apply(tree, &layout);
    assert_eq!(tree.hud_offset("vitals"), (50, 30));
}

#[test]
fn the_lock_stops_every_move_and_leaves_the_mode() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let edit = hud.modern_edit.as_ref().unwrap();
    edit.set_on(tree, true);
    let lock = named(tree, "ModernEditLock");
    edit.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(lock)));
    assert!(tree.hud_locked && !edit.is_on());
    let chat = named(tree, "ChatWindow1");
    tree.start_movement(chat);
    assert!(tree.movement.is_none());
    // The mode releases the lock.
    edit.set_on(tree, true);
    assert!(!tree.hud_locked);
}

#[test]
fn the_targeting_banner_shows_at_the_top_centre_while_the_pointer_is_on() {
    let (Some(mut modern), Some(classic)) = (hud(HudStyle::Modern), hud(HudStyle::Classic)) else {
        return;
    };
    assert!(classic.modern_banner.is_none());
    let banner = modern.modern_banner.clone().expect("banner");
    let tree = &mut modern.tree;
    let root = named(tree, "ModernTargetBanner");
    banner.tick(tree);
    assert!(!tree.el(root).visible);
    crate::selection_notices::set_targeting_pointer(tree, true);
    banner.tick(tree);
    let at = tree.screen_box(root);
    assert!(tree.el(root).visible);
    assert_eq!(at.y0, 6);
    assert!((at.x0 + at.x1 - 1365).abs() <= 2, "{at:?}");
    crate::selection_notices::set_targeting_pointer(tree, false);
    banner.tick(tree);
    assert!(!tree.el(root).visible);
}

#[test]
fn reset_to_default_puts_the_chat_windows_back() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let chat = named(tree, "ChatWindow1");
    let home = tree.screen_box(chat);
    tree.move_to(chat, home.x0 + 200, home.y0 - 150);
    let moved = tree.screen_box(chat);
    assert_ne!(moved, home);
    tree.set_window_positions(&tree.window_positions());
    crate::modern_layout_edit::reset_windows(tree);
    assert_eq!(tree.screen_box(chat), home);
    // Its saved place is forgotten: a later window position pass leaves it home.
    let saved = tree.window_positions();
    tree.set_window_positions(&saved);
    assert_eq!(tree.screen_box(chat), home);
}

/// The bar's second row: Save as names the layout from the field, the chosen chip steps through
/// the saved layouts and Load asks for the chosen one; a saved layout's chat window places come
/// back when it loads.
#[test]
fn the_presets_row_saves_and_loads_a_named_layout() {
    use crate::modern_layout_edit::{apply_window_lines, window_lines, EditAction};
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let edit = hud.modern_edit.as_ref().unwrap();
    edit.set_on(tree, true);
    let field = named(tree, "ModernEditPresetName");
    tree.set_text(field, "  Raid  ");
    let save = named(tree, "ModernEditSave");
    assert_eq!(
        edit.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(save))),
        (true, Some(EditAction::SavePreset("Raid".into())))
    );
    // The chat window's place as saved, then moved, then the saved place loaded back.
    let chat = named(tree, "ChatWindow1");
    let home = tree.screen_box(chat);
    let lines = window_lines(tree);
    assert!(!lines.is_empty());
    let mut layout = crate::hud_layout::HudLayout::default();
    layout.save_preset("Raid", &lines);
    layout.save_preset("Solo", &[]);
    edit.set_presets(tree, layout.presets(), Some("Raid"));
    tree.move_to(chat, home.x0 + 150, home.y0 - 100);
    let load = named(tree, "ModernEditLoad");
    let EditAction::LoadPreset(name) = edit
        .message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(load)))
        .1
        .unwrap()
    else {
        panic!("load");
    };
    assert_eq!(name, "Raid");
    apply_window_lines(tree, &layout.load_preset(&name).unwrap());
    assert_eq!(tree.screen_box(chat), home);
    // The chosen chip steps to the next saved layout.
    let chosen = named(tree, "ModernEditChosen");
    edit.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(chosen)));
    assert_eq!(
        edit.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(load)))
            .1,
        Some(EditAction::LoadPreset("Solo".into()))
    );
}
