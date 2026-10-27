//! The help window on the real gameplay layout and the shipped pages (`AC2_DAT_DIR`, and the
//! content directory `AC2_CONTENT_DIR` or the workspace's `payload/ProgramFiles/Content`): each
//! topic opens its page in the window, token 0 opens the default page, and the close button hides
//! the window and forgets the page.

use std::path::{Path, PathBuf};

use ac2_db::ResourceStore;

use crate::build_gameplay_hud;
use crate::help::{help_token, load_help_pages, HelpPages, HELP_TOPICS};
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

fn content_dir() -> PathBuf {
    match std::env::var_os("AC2_CONTENT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../payload/ProgramFiles/Content"),
    }
}

fn press(element_id: u32) -> UiMessage {
    UiMessage {
        element_id,
        element: None,
        message: MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    }
}

#[test]
fn each_help_topic_opens_its_page_in_the_window() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let content = content_dir();
    if !content.is_dir() {
        return;
    }
    let store = ResourceStore::new(dataset);
    let pages: HelpPages = load_help_pages(&store, &content)
        .unwrap()
        .expect("the help table enum resolves");
    assert!(pages.len() > 100, "{} pages parsed", pages.len());
    let store = std::sync::Arc::new(store);
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let help = hud
        .help
        .as_mut()
        .expect("the help window is made over the Book");
    assert!(!help.is_shown());
    for topic in HELP_TOPICS {
        help.open(tree, Some(&pages), help_token(topic));
        assert!(help.is_shown(), "{} shows the window", topic.id);
        let page = pages
            .page_for_token(help_token(topic))
            .unwrap_or_else(|| panic!("{} names a page", topic.id));
        assert_eq!(help.page(), Some(page), "{} opens its page", topic.id);
        assert!(help.line_count() > 0, "{} has text", topic.id);
        assert!(
            !help.text().starts_with("The help page"),
            "{}: {}",
            topic.id,
            help.text()
        );
        help.close(tree);
    }
    help.open(tree, Some(&pages), 0);
    assert!(help.is_shown());
    assert_eq!(help.page(), pages.page_for_token(0));
    assert!(help.line_count() > 0);
}

#[test]
fn the_close_button_hides_the_window_and_forgets_its_page() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let content = content_dir();
    if !content.is_dir() {
        return;
    }
    let store = ResourceStore::new(dataset);
    let pages = load_help_pages(&store, &content).unwrap().unwrap();
    let store = std::sync::Arc::new(store);
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let help = hud.help.as_mut().unwrap();
    help.open(tree, Some(&pages), tokenize("ID_Help_Book"));
    assert!(help.is_shown());
    let element = help.element(tree, "BookCloseButton");
    let close = UiMessage {
        element,
        ..press(tokenize("BookCloseButton"))
    };
    assert!(element.is_some());
    assert!(help.message(tree, Some(&pages), &close));
    assert!(!help.is_shown());
    assert_eq!(help.page(), None);
}

#[test]
fn without_pages_the_window_says_help_is_not_installed() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let help = hud.help.as_mut().unwrap();
    help.open(tree, None, tokenize("ID_Help_Book"));
    assert!(help.is_shown());
    assert!(help.text().contains("not installed"), "{}", help.text());
}
