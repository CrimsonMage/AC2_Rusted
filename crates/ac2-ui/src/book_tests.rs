//! `gmBookUI` on the real gameplay layout (`AC2_DAT_DIR`): the notice shows the book, its pages
//! and their numbers, the page turns and the slider, the close, help and copy buttons. The page
//! split itself is checked on [`segments`](super::book::segments) without the layout.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::book::{segments, BookRequest, BookUi};
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, load_layout_by_enum};

/// Renders the page label as `current/last` from its variables, and anything else as its id.
struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        let value = |name: &str| {
            string
                .vars
                .iter()
                .find(|(id, _)| *id == tokenize(name))
                .and_then(|(_, var)| match var {
                    SkillVar::Int(v) => Some(v.to_string()),
                    _ => None,
                })
                .unwrap_or_default()
        };
        format!("{}/{}", value("CURRENT"), value("LAST"))
    }
}

fn press(element_id: u32, message: u32) -> UiMessage {
    UiMessage {
        element_id,
        element: None,
        message,
        data: 0,
        item: None,
    }
}

/// `n` one-line pages, each `<BR>`-ended.
fn pages_of(n: usize) -> String {
    (0..n)
        .map(|i| format!("Page {i} text"))
        .collect::<Vec<_>>()
        .join("<BR>")
}

#[test]
fn a_piece_splits_into_its_header_and_body_with_a_newline() {
    let got = segments("<H>Chapter One</H>\nIt began.<BR>Plain.<BR>");
    assert_eq!(
        got,
        vec![
            (Some("Chapter One".to_owned()), "It began.\n".to_owned()),
            (None, "Plain.\n".to_owned()),
        ]
    );
    // A header that never closes is body text.
    assert_eq!(segments("<H>open"), vec![(None, "<H>open\n".to_owned())]);
}

#[test]
fn the_book_is_hidden_until_a_notice_and_the_close_button_hides_it_again() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let tree = &mut hud.tree;
    let mut book = BookUi::create(tree, &gameplay).expect("the Book template");
    assert!(!book.is_shown());
    book.recv_use_book(
        tree,
        &Strings,
        true,
        tokenize("stateless"),
        "Tome",
        "Words",
        0,
    );
    assert!(book.is_shown());
    assert!(tree.get(book.field()).unwrap().visible);
    let title = tree
        .child_recursive(book.field(), tokenize("Panel_Title_Book"))
        .unwrap();
    assert_eq!(tree.get(title).unwrap().text().as_deref(), Some("Tome"));
    let close = press(tokenize("BookCloseButton"), MSG_BUTTON_RELEASE_ON);
    assert_eq!(book.message(tree, &Strings, &close), None);
    assert!(!book.is_shown());
    assert!(!tree.get(book.field()).unwrap().visible);
}

#[test]
fn a_long_book_pages_and_the_page_turns_follow_the_book() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let tree = &mut hud.tree;
    let mut book = BookUi::create(tree, &gameplay).expect("the Book template");
    // Each one-line page is padded to the client area's lines per page: one page apiece.
    book.recv_use_book(
        tree,
        &Strings,
        true,
        tokenize("stateless"),
        "Tome",
        &pages_of(12),
        0,
    );
    assert_eq!(book.last_page(), 12);
    assert_eq!(book.current_page(), 1);
    let label = tree
        .child_recursive(book.field(), tokenize("PageNumberText"))
        .unwrap();
    assert_eq!(tree.get(label).unwrap().text().as_deref(), Some("1/12"));

    let foreground = tokenize("BookForeground");
    let turn = press(foreground, MSG_MOUSE_PRESS_TOP);
    assert_eq!(book.message(tree, &Strings, &turn), None);
    assert_eq!(book.current_page(), 2);
    let right = press(tokenize("BookSliderRight"), MSG_BUTTON_RELEASE_ON);
    book.message(tree, &Strings, &right);
    assert_eq!(book.current_page(), 3);
    let left = press(tokenize("BookSliderLeft"), MSG_BUTTON_RELEASE_ON);
    book.message(tree, &Strings, &left);
    book.message(tree, &Strings, &left);
    assert_eq!(book.current_page(), 1);
    assert_eq!(tree.get(label).unwrap().text().as_deref(), Some("1/12"));

    // The slider at its middle: (12 - 1) * 0.5 = 5.5, rounded half up, not past .5: page 5.
    let slider = tree
        .child_recursive(book.field(), tokenize("BookSliderBar"))
        .unwrap();
    tree.scrollbar_set_position(slider, 0.5);
    let mut moved = press(tokenize("SliderBar"), crate::scrollbar::MSG_NEW_POSITION);
    moved.element = Some(slider);
    book.message(tree, &Strings, &moved);
    assert_eq!(book.current_page(), 5);

    // The page turn past the last page closes the book.
    for _ in 0..7 {
        book.message(tree, &Strings, &turn);
    }
    assert_eq!(book.current_page(), 12);
    book.message(tree, &Strings, &turn);
    assert!(!book.is_shown());
}

#[test]
fn help_asks_for_the_help_page_and_copy_gives_the_text() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let tree = &mut hud.tree;
    let mut book = BookUi::create(tree, &gameplay).expect("the Book template");
    book.recv_use_book(
        tree,
        &Strings,
        true,
        tokenize("stateless"),
        "Tome",
        "<H>Ch</H>\nAlpha<BR>Beta",
        0,
    );
    let help = press(tokenize("BookHelpButton"), MSG_BUTTON_RELEASE_ON);
    assert_eq!(
        book.message(tree, &Strings, &help),
        Some(BookRequest::OpenHelp)
    );
    let copy = press(tokenize("BookCopyButton"), MSG_BUTTON_RELEASE_ON);
    let Some(BookRequest::Copy(text)) = book.message(tree, &Strings, &copy) else {
        panic!("copy gives the book's text");
    };
    assert!(text.starts_with("Tome\n\n"), "{text:?}");
    assert!(text.contains("Alpha"), "{text:?}");
    assert!(text.contains("Beta"), "{text:?}");
}

/// The copy button puts the book's text on the clipboard (`SendBookToClipboard`), not back to
/// the client as a request.
#[test]
fn the_copy_button_sends_the_book_to_the_clipboard() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let tree = &mut hud.tree;
    let clipboard = Arc::new(crate::MemoryClipboard::default());
    tree.set_text_clipboard(clipboard.clone());
    let mut book = BookUi::create(tree, &gameplay).expect("the Book template");
    book.recv_use_book(
        tree,
        &Strings,
        true,
        tokenize("stateless"),
        "Tome",
        "<H>Ch</H>\nAlpha<BR>Beta",
        0,
    );
    let copy = press(tokenize("BookCopyButton"), MSG_BUTTON_RELEASE_ON);
    assert!(crate::book_pass(&mut book, tree, &Strings, &[], &[copy]).is_empty());
    let text = crate::TextClipboard::text(&*clipboard).expect("the copied text");
    assert!(text.starts_with("Tome\n\n"), "{text:?}");
    assert!(text.contains("Alpha") && text.contains("Beta"), "{text:?}");
}
