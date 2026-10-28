//! The chat against real data (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::*;
use crate::chat_window::{TEXT_TYPE_CHAT_ENTRY, TEXT_TYPE_ERROR, TEXT_TYPE_SAY};

const ENTER: u32 = 0x4000_0018;

/// The tree's messages to the chat until it broadcasts no more.
fn pump(tree: &mut UiTree, chat: &mut ChatUi) {
    for _ in 0..8 {
        let messages = tree.take_messages();
        if messages.is_empty() {
            return;
        }
        for message in &messages {
            chat.message(tree, message, Modifiers::default());
        }
    }
}

fn key(tree: &mut UiTree, chat: &mut ChatUi, action: u32) {
    tree.start_input(action, 1.0);
    tree.stop_input(action);
    pump(tree, chat);
}

fn hud() -> Option<(UiTree, ChatUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    Some((
        hud.tree,
        hud.chat.expect("the toolbar layout has the chat windows"),
    ))
}

#[test]
fn the_first_chat_window_starts_shown_and_greets_on_the_first_settings() {
    let Some((mut tree, mut chat)) = hud() else {
        return;
    };
    let roots = chat.roots();
    assert_eq!(roots.len(), 4);
    let shown: Vec<bool> = roots
        .iter()
        .map(|&r| tree.get(r).unwrap().visible)
        .collect();
    assert_eq!(shown, vec![true, false, false, false]);
    // No filters yet: the log greets, then explains the empty filter.
    chat.reload_chat_settings(&mut tree, &ChatSettings::default());
    let log = chat.logs()[0].unwrap();
    let text = tree.get(log).unwrap().text().unwrap();
    eprintln!("log: {text:?}");
    assert!(!text.is_empty());
    // Say passes window 1's filter once set; it reaches the log on the UI loop.
    let mut settings = ChatSettings::default();
    settings.filters.insert(1, TEXT_TYPE_SAY);
    chat.reload_chat_settings(&mut tree, &settings);
    chat.display(TEXT_TYPE_SAY, "Gnome says, \"hello\"\n");
    chat.display(0x20, "not for this window\n");
    chat.global_loop(&mut tree, 0.0);
    let text = tree.get(log).unwrap().text().unwrap();
    assert!(text.ends_with("Gnome says, \"hello\"\n"), "{text:?}");
    assert!(!text.contains("not for this window"));
    // The chat entry type is always shown.
    chat.display(TEXT_TYPE_CHAT_ENTRY, "entry\n");
    chat.global_loop(&mut tree, 0.0);
    assert!(tree.get(log).unwrap().text().unwrap().ends_with("entry\n"));
}

#[test]
fn enter_opens_the_entry_line_and_sends_what_was_typed() {
    let Some((mut tree, mut chat)) = hud() else {
        return;
    };
    let entry = chat.entries()[0].unwrap();
    assert!(!tree.text_mode());
    key(&mut tree, &mut chat, ENTER);
    assert_eq!(tree.active(), Some(entry));
    assert!(tree.text_mode());
    for ch in "hi there".encode_utf16() {
        tree.character(ch, 0, crate::chat::Modifiers::default());
    }
    pump(&mut tree, &mut chat);
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some("hi there"));
    key(&mut tree, &mut chat, ENTER);
    assert_eq!(
        chat.take_requests(),
        vec![ChatRequest::Line("hi there".into())]
    );
    assert_eq!(tree.active(), None);
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some(""));
    // An empty line sends nothing.
    key(&mut tree, &mut chat, ENTER);
    key(&mut tree, &mut chat, ENTER);
    assert!(chat.take_requests().is_empty());
}

#[test]
fn the_command_key_starts_a_line_and_window_keys_toggle_windows() {
    let Some((mut tree, mut chat)) = hud() else {
        return;
    };
    let entry = chat.entries()[0].unwrap();
    key(&mut tree, &mut chat, 0x4100_0010);
    assert_eq!(tree.active(), Some(entry));
    let text = tree.get(entry).unwrap().text().unwrap();
    eprintln!("command prefix: {text:?}");
    assert_eq!(text, "/");
    // Escape deactivates on the next UI loop.
    key(&mut tree, &mut chat, 0x4000_001E);
    assert_eq!(tree.active(), Some(entry));
    chat.global_loop(&mut tree, 0.0);
    assert_eq!(tree.active(), None);
    let second = chat.roots()[1];
    key(&mut tree, &mut chat, 0x4100_0042);
    assert!(tree.get(second).unwrap().visible);
    key(&mut tree, &mut chat, 0x4100_0042);
    assert!(!tree.get(second).unwrap().visible);
    // History keys go to the client.
    key(&mut tree, &mut chat, 0x4000_0015);
    assert_eq!(
        chat.take_requests(),
        vec![ChatRequest::History {
            history: ChatHistory::Command,
            prev: true
        }]
    );
}

#[test]
fn error_lines_show_in_the_spew_box_and_leave_after_their_decay_time() {
    let Some((mut tree, mut chat)) = hud() else {
        return;
    };
    assert!(
        chat.spew_box().is_some(),
        "the chat layout has the spew box"
    );
    chat.display(TEXT_TYPE_SAY, "not an error");
    chat.global_loop(&mut tree, 100.0);
    assert!(chat.spew_box().unwrap().shown().is_empty());
    // `TEXT_TYPE_ERROR` queues the line; the next loop shows it with its element's decay_time
    // (6 s in the chat layout's `SpewBoxMessage`).
    chat.display(TEXT_TYPE_ERROR, "You are hit.  ");
    chat.global_loop(&mut tree, 100.0);
    let shown = chat.spew_box().unwrap().shown().to_vec();
    assert_eq!(shown.len(), 1);
    let line = shown[0];
    assert_eq!(
        tree.get(line.element).unwrap().text().as_deref(),
        Some("You are hit.")
    );
    assert_eq!(line.removal_time, 106.0);
    chat.global_loop(&mut tree, 105.9);
    assert_eq!(chat.spew_box().unwrap().shown().len(), 1);
    chat.global_loop(&mut tree, 106.0);
    assert!(chat.spew_box().unwrap().shown().is_empty());
    assert!(
        tree.get(line.element).is_none(),
        "an expired line is deleted"
    );
}

#[test]
fn a_press_on_the_log_scroll_bar_holds_the_log_until_release() {
    let Some((mut tree, mut chat)) = hud() else {
        return;
    };
    // The client applies the player's settings at login (`build.rs`): window 1 shows say lines.
    let mut settings = ChatSettings::default();
    settings.filters.insert(1, TEXT_TYPE_SAY);
    chat.reload_chat_settings(&mut tree, &settings);
    let root = chat.roots()[0];
    let scrollbar = tree
        .child_recursive(root, tokenize("ChatLogScrollbar"))
        .expect("the log has its scroll bar");
    let log = tree.child_recursive(root, tokenize("ChatLog")).unwrap();
    let message = |tree: &mut UiTree, chat: &mut ChatUi, what: u32| {
        let element_id = tree.get(scrollbar).unwrap().id();
        let m = UiMessage {
            element_id,
            element: Some(scrollbar),
            message: what,
            data: 0,
            item: None,
        };
        chat.message(tree, &m, Modifiers::default());
    };
    for n in 0..60 {
        chat.display(TEXT_TYPE_SAY, &format!("line {n}"));
    }
    chat.global_loop(&mut tree, 0.0);
    let view = |tree: &UiTree| tree.el(log).text.as_ref().map_or(0, |text| text.view.1);
    let at_end = view(&tree);
    assert_ne!(at_end, 0);
    assert!(at_end > 0, "the log scrolls to its end without a lock");
    tree.scroll_to_home(log);
    message(&mut tree, &mut chat, tokenize("widget_press"));
    chat.display(TEXT_TYPE_SAY, "held");
    chat.global_loop(&mut tree, 0.0);
    assert_eq!(view(&tree), 0, "a locked log keeps its place");
    message(&mut tree, &mut chat, tokenize("widget_release"));
    chat.display(TEXT_TYPE_SAY, "released");
    chat.global_loop(&mut tree, 0.0);
    assert!(view(&tree) > 0, "the release unlocks the log");
}
