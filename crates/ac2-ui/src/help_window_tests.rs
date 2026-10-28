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

/// The pointer at `(x, y)`, then a press on the book's text there (what the text element sends).
fn press_text(
    tree: &mut crate::UiTree,
    help: &mut crate::help_window::HelpWindow,
    pages: &HelpPages,
    at: (i32, i32),
) -> bool {
    tree.start_input(crate::UI_POINTER_X, at.0 as f32);
    tree.start_input(crate::UI_POINTER_Y, at.1 as f32);
    let text = help.element(tree, "BookClientAreaText");
    let press = UiMessage {
        element: text,
        ..press(tokenize("BookClientAreaText"))
    };
    let press = UiMessage {
        message: crate::token::MSG_MOUSE_PRESS_TOP,
        ..press
    };
    help.message(tree, Some(pages), &press)
}

#[test]
fn a_press_on_a_link_follows_it_and_a_press_on_tabs_checks_them() {
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
    let mut followed = false;
    let mut tabbed = false;
    // The topic pages, then the default page (token 0), whose tabs are the page's own.
    let tokens: Vec<(u32, &str)> = HELP_TOPICS
        .iter()
        .map(|topic| (help_token(topic), topic.id))
        .chain(std::iter::once((0, "default page")))
        .collect();
    for (token, name) in tokens {
        help.open(tree, Some(&pages), token);
        if !followed {
            if let Some((at, link)) = help.link_point(tree, false) {
                let before = help.page().map(str::to_owned);
                let Some(href) = link.href.clone() else {
                    continue;
                };
                assert!(
                    press_text(tree, help, &pages, at),
                    "{}: the press is the text's",
                    name
                );
                // A link without a target opens its page in the window. A link into a frame keeps
                // the page; only the frame changes, so the page name is the same.
                if link.target.as_deref().unwrap_or("").is_empty() {
                    assert_ne!(
                        help.page(),
                        before.as_deref(),
                        "{name}: link {href} target {:?} at {at:?}",
                        link.target
                    );
                    assert_eq!(
                        help.page().map(|p| crate::help::page_key(p).to_owned()),
                        Some(crate::help::page_key(&href).to_owned()),
                        "{} follows {href}",
                        name
                    );
                }
                followed = true;
            }
        }
        if !tabbed {
            if let Some((at, tab)) = help.link_point(tree, true) {
                let before = help.text();
                assert!(press_text(tree, help, &pages, at), "{}: a tab press", name);
                assert!(
                    help.text() != before,
                    "{name}: tab {:?} changes the text",
                    tab.href
                );
                tabbed = true;
            }
        }
        help.close(tree);
        if followed && tabbed {
            break;
        }
    }
    assert!(followed, "a shipped help page has a link to press");
    assert!(tabbed, "a shipped help page has tabs to press");
}

#[test]
fn the_wheel_over_the_help_text_scrolls_the_page() {
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
    // The longest topic page: the text overflows and the slider scrolls it.
    let topic = HELP_TOPICS
        .iter()
        .max_by_key(|topic| {
            help.open(tree, Some(&pages), help_token(topic));
            help.line_count()
        })
        .unwrap();
    help.open(tree, Some(&pages), help_token(topic));
    assert!(
        help.line_count() > 30,
        "{} has {} lines",
        topic.id,
        help.line_count()
    );
    let slider = help.element(tree, "BookSliderBar").expect("the slider");
    let text = help.element(tree, "BookClientAreaText").expect("the text");
    let at = {
        let rect = tree.screen_box(text);
        ((rect.x0 + rect.x1) / 2, (rect.y0 + rect.y1) / 2)
    };
    tree.start_input(crate::UI_POINTER_X, at.0 as f32);
    tree.start_input(crate::UI_POINTER_Y, at.1 as f32);
    let start = tree.scrollbar_position(slider);
    tree.start_input(crate::UI_WHEEL_OUT, 1.0);
    for message in tree.take_messages() {
        help.message(tree, Some(&pages), &message);
    }
    let down = tree.scrollbar_position(slider);
    assert!(
        down > start,
        "a wheel turn out scrolls the text: {start} -> {down}; text {:?} pointer {:?} lines {} slider {:?} msgs {:?}",
        tree.get(text).map(|e| e.rect()),
        tree.pointer(),
        help.line_count(),
        tree.get(slider).map(|e| e.rect()),
        {
            tree.start_input(crate::UI_WHEEL_OUT, 1.0);
            tree.take_messages().iter().map(|m| (m.element_id, m.message, m.data)).collect::<Vec<_>>()
        }
    );
    tree.start_input(crate::UI_WHEEL_IN, 1.0);
    for message in tree.take_messages() {
        help.message(tree, Some(&pages), &message);
    }
    let up = tree.scrollbar_position(slider);
    assert!(up < down, "a wheel turn in scrolls it back: {down} -> {up}");
}
