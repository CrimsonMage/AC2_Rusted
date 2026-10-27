//! The chat against real data (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::*;
use crate::chat_window::{TEXT_TYPE_CHAT_ENTRY, TEXT_TYPE_SAY};

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
    chat.global_loop(&mut tree);
    let text = tree.get(log).unwrap().text().unwrap();
    assert!(text.ends_with("Gnome says, \"hello\"\n"), "{text:?}");
    assert!(!text.contains("not for this window"));
    // The chat entry type is always shown.
    chat.display(TEXT_TYPE_CHAT_ENTRY, "entry\n");
    chat.global_loop(&mut tree);
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
    chat.global_loop(&mut tree);
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
