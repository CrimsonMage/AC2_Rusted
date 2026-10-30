//! The modern chat windows (`modern_chat.rs`) on the real layouts (`AC2_DAT_DIR`): the design's
//! parts laid out over each shown window, the opacity slider setting the window's opacity and
//! saving it on release as retail's slider did, and the classic windows left as retail's.

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

struct Names;

impl crate::ui_string::UiStrings for Names {
    fn render_string(&self, _: &crate::ui_string::UiString) -> String {
        "Chat".into()
    }
}

fn part(tree: &UiTree, root: usize, name: &str) -> usize {
    tree.child_recursive(root, tokenize(name))
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
fn a_shown_window_takes_the_designs_header_log_and_entry() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let chat = hud.chat.as_mut().unwrap();
    chat.modern_tick(tree, &Names);
    let root = chat.roots()[0];
    let at = |tree: &UiTree, name: &str| tree.screen_box(part(tree, root, name));
    let window = tree.screen_box(root);
    let (header, log, entry, menu) = (
        at(tree, "ModernChatHeader"),
        at(tree, "ChatLog"),
        at(tree, "ChatEntry"),
        at(tree, "OptionsButton"),
    );
    assert_eq!(
        (header.x0, header.y0, header.x1),
        (window.x0, window.y0, window.x1)
    );
    assert!(header.y1 < log.y0 && log.y1 < entry.y0 && entry.y1 <= window.y1);
    assert!(menu.x1 <= window.x1 && menu.y1 <= header.y1, "{menu:?}");
    // The tab names the window by its key.
    let key = part(tree, root, "ModernChatKeyText");
    assert_eq!(tree.el(key).text().unwrap(), "F1");
    // Retail's frame and options field give way.
    assert!(!tree.el(part(tree, root, "OptionsField")).visible);
}

#[test]
fn the_opacity_slider_sets_the_windows_opacity_and_saves_it_on_release() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let chat = hud.chat.as_mut().unwrap();
    chat.modern_tick(tree, &Names);
    let root = chat.roots()[0];
    let track = part(tree, root, "ModernChatOpTrack");
    let at = tree.screen_box(track);
    tree.pointer = (at.x0 + at.width() / 2, (at.y0 + at.y1) / 2);
    let modifiers = crate::chat::Modifiers::default();
    chat.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(track)), modifiers);
    let half = tree.el(root).alpha_value;
    assert!((half - 0.5).abs() < 0.05, "{half}");
    tree.pointer.0 = at.x1;
    chat.message(tree, &message(GLOBAL_MOUSE_MOVE, None), modifiers);
    chat.message(tree, &message(GLOBAL_MOUSE_UP, None), modifiers);
    let saved = chat.take_opacity_requests();
    assert_eq!(saved.len(), 1);
    assert!(saved[0].1 > 0.95, "{saved:?}");
    assert!(tree.el(root).alpha_value > 0.95);
}

#[test]
fn the_classic_windows_keep_retails_frame() {
    let Some(mut hud) = hud(HudStyle::Classic) else {
        return;
    };
    let tree = &mut hud.tree;
    let chat = hud.chat.as_mut().unwrap();
    chat.modern_tick(tree, &Names);
    let root = chat.roots()[0];
    assert!(tree
        .child_recursive(root, tokenize("ModernChatHeader"))
        .is_none());
}

fn release(tree: &UiTree, element: usize) -> UiMessage {
    UiMessage {
        element_id: tree.el(element).id(),
        element: Some(element),
        message: crate::token::MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    }
}

fn root_named(tree: &UiTree, name: &str) -> usize {
    *tree
        .roots()
        .iter()
        .find(|&&r| tree.el(r).id() == tokenize(name))
        .unwrap_or_else(|| panic!("{name}"))
}

/// The options button (`=`) opens the window's settings over it; its chips change the window
/// (the lock stops a move, the tint and font step are kept in the HUD's layout), Hide hides it.
#[test]
fn the_settings_popover_changes_the_window_and_is_kept_in_the_layout() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let chat = hud.chat.as_mut().unwrap();
    chat.modern_tick(tree, &Names);
    let root = chat.roots()[0];
    let modifiers = crate::chat::Modifiers::default();
    let button = part(tree, root, "OptionsButton");
    chat.message(tree, &release(tree, button), modifiers);
    let popover = root_named(tree, "ModernChatSettings");
    assert!(tree.el(popover).visible);
    let (p, b) = (tree.screen_box(popover), tree.screen_box(button));
    assert!(
        p.x1 <= b.x1 + 1 && (p.y1 < b.y0 || p.y0 > b.y1),
        "{p:?} {b:?}"
    );
    let press = |tree: &mut UiTree, chat: &mut crate::ChatUi, name: &str| {
        let chip = part(tree, popover, name);
        chat.message(tree, &message(MSG_MOUSE_PRESS_TOP, Some(chip)), modifiers);
    };
    press(tree, chat, "ModernChatFontL");
    press(tree, chat, "ModernChatTint");
    press(tree, chat, "ModernChatLock");
    let mut layout = crate::hud_layout::HudLayout::default();
    assert!(chat.export_layout(&mut layout));
    assert_eq!(layout.number("chat.1.font", 0), 1);
    assert_eq!(layout.number("chat.1.tint", 0), 1);
    assert!(layout.flag("chat.1.locked", false));
    // Locked: the move bar's press does not move the window.
    let bar = part(tree, root, "MoveBar");
    let before = tree.screen_box(root);
    tree.start_movement(bar);
    assert!(tree.movement.is_none(), "a locked window does not move");
    assert_eq!(tree.screen_box(root), before);
    // The layout read back gives the same choices.
    let mut back = crate::hud_layout::HudLayout::parse(&layout.to_text());
    chat.apply_layout(tree, &back);
    assert!(!chat.export_layout(&mut back));
    press(tree, chat, "ModernChatHide");
    assert!(!tree.el(root).visible);
    assert!(!tree.el(popover).visible);
}

