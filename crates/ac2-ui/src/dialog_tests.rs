//! `DialogFactory` on a hand-built dialogs layout.

use std::sync::Arc;

use crate::layout::{ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::{
    DialogFactory, DialogOutcome, UiTree, DIALOG_CONFIRMATION, DIALOG_WAIT, UI_POINTER_X,
    UI_POINTER_Y,
};

fn element(
    id: &str,
    kind: &str,
    r: (i16, i16, i16, i16),
    children: Vec<Arc<ElementDesc>>,
) -> Arc<ElementDesc> {
    Arc::new(ElementDesc {
        id: tokenize(id),
        kind: tokenize(kind),
        default_state: STATELESS,
        rect: Rect {
            x0: r.0,
            y0: r.1,
            x1: r.2,
            y1: r.3,
        },
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: Rect {
                x0: -1,
                y0: -1,
                x1: -1,
                y1: -1,
            },
            z_level: 0,
            attributes: Vec::new(),
            media: Vec::new(),
            transitions: Vec::new(),
            children,
        }],
    })
}

fn layout() -> Arc<LayoutDesc> {
    let confirmation = element(
        "ConfirmationField",
        "field",
        (0, 0, 99, 49),
        vec![
            element("QuestionText", "text", (0, 0, 99, 19), vec![]),
            element("YesButton", "button", (0, 20, 49, 49), vec![]),
            element("NoButton", "button", (50, 20, 99, 49), vec![]),
        ],
    );
    let wait = element(
        "WaitField",
        "field",
        (0, 0, 99, 49),
        vec![element("TextField", "text", (0, 0, 99, 19), vec![])],
    );
    Arc::new(LayoutDesc {
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 100,
        elements: vec![confirmation, wait],
    })
}

fn click(tree: &mut UiTree, x: i32, y: i32) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
    tree.start_input(0x4000_0004, 1.0);
    tree.stop_input(0x4000_0004);
}

/// The text element exists (without fonts the text itself is dropped, as retail's glyph list
/// does with no font).
fn has_text(tree: &UiTree, root: usize, name: &str) -> bool {
    tree.child_recursive(root, tokenize(name))
        .and_then(|i| tree.get(i))
        .is_some_and(|el| el.text().is_some())
}

#[test]
fn dialogs_queue_show_one_at_a_time_and_report_the_answer() {
    let mut tree = UiTree::new(100, 100);
    let mut dialogs = DialogFactory::new(Some(layout()));
    let first = dialogs.make_dialog(&mut tree, "Delete?", DIALOG_CONFIRMATION, false);
    let second = dialogs.make_dialog(&mut tree, "Please wait", DIALOG_WAIT, false);
    assert_eq!((first, second), (1, 2));
    assert_eq!(dialogs.shown(), Some((1, DIALOG_CONFIRMATION)));
    assert_eq!(dialogs.queued(), vec![2]);
    assert!(!dialogs.can_mouse_look());
    let root = tree.roots()[0];
    assert!(has_text(&tree, root, "QuestionText"));
    // No: the confirmation reports and closes; the wait dialog shows next.
    click(&mut tree, 75, 30);
    for message in tree.take_messages() {
        dialogs.message(&mut tree, &message);
    }
    assert_eq!(
        dialogs.take_outcomes(),
        vec![DialogOutcome::Confirmation {
            context: 1,
            yes: false
        }]
    );
    assert_eq!(dialogs.shown(), Some((2, DIALOG_WAIT)));
    assert_eq!(tree.roots().len(), 1);
    let root = tree.roots()[0];
    assert!(has_text(&tree, root, "TextField"));
    dialogs.close_dialog(&mut tree, 2);
    assert!(dialogs.shown().is_none() && tree.roots().is_empty());
    assert!(dialogs.can_mouse_look());
}

#[test]
fn a_topmost_dialog_puts_the_shown_one_back_in_front_of_the_queue() {
    let mut tree = UiTree::new(100, 100);
    let mut dialogs = DialogFactory::new(Some(layout()));
    let wait = dialogs.make_dialog(&mut tree, "wait", DIALOG_WAIT, false);
    let queued = dialogs.make_dialog(&mut tree, "later", DIALOG_WAIT, false);
    let top = dialogs.make_dialog(&mut tree, "now?", DIALOG_CONFIRMATION, true);
    assert_eq!(dialogs.shown(), Some((top, DIALOG_CONFIRMATION)));
    assert_eq!(dialogs.queued(), vec![wait, queued]);
    assert_eq!(tree.roots().len(), 1);
    // Closing a queued dialog drops it; yes on the shown one brings back the first.
    dialogs.close_dialog(&mut tree, queued);
    click(&mut tree, 25, 30);
    for message in tree.take_messages() {
        dialogs.message(&mut tree, &message);
    }
    assert_eq!(
        dialogs.take_outcomes(),
        vec![DialogOutcome::Confirmation {
            context: top,
            yes: true
        }]
    );
    assert_eq!(dialogs.shown(), Some((wait, DIALOG_WAIT)));
    assert!(dialogs.queued().is_empty());
}
