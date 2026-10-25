//! `gmStackUI` on the real gameplay layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON};
use crate::{
    build_gameplay_hud, StackUiDone, UiString, UiStrings, UiTree, UI_POINTER_X, UI_POINTER_Y,
};

struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        format!("{:08x}{:?}", string.string_id, string.vars)
    }
}

fn release(tree: &UiTree, field: usize, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: tree.child_recursive(field, tokenize(name)),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

fn text(tree: &UiTree, field: usize, name: &str) -> String {
    let element = tree.child_recursive(field, tokenize(name)).unwrap();
    tree.get(element).unwrap().text().unwrap_or_default()
}

#[test]
fn the_stack_dialog_asks_for_an_amount_up_to_the_most() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let stack = hud.stack.as_mut().expect("the stack dialog");
    let field = stack.field().expect("StackableField");
    assert!(!tree.get(field).unwrap().visible);

    // Shown centred on the pointer, kept on the display.
    tree.start_input(UI_POINTER_X, 2.0);
    tree.start_input(UI_POINTER_Y, 2.0);
    stack.init(tree, 7, "How many?", 5, 9, 0x8000_0001, 0x2000_0001);
    assert!(tree.get(field).unwrap().visible);
    assert_eq!(stack.shown(), Some(7));
    let placement = tree
        .placements()
        .into_iter()
        .find(|p| p.root == field)
        .unwrap();
    assert_eq!((placement.x, placement.y), (0, 0));
    assert_eq!(text(tree, field, "Label"), "How many?");
    assert_eq!(text(tree, field, "AmountEntry"), "5");

    // The toggle sets half (rounded down), then all.
    let toggle = tree
        .child_recursive(field, tokenize("AmountToggleButton"))
        .unwrap();
    tree.set_bool(toggle, BUTTON_HIGHLIGHTED, true);
    let message = release(tree, field, "AmountToggleButton");
    assert_eq!(stack.message(tree, &message, &Strings), None);
    assert_eq!(text(tree, field, "AmountEntry"), "4");
    tree.set_bool(toggle, BUTTON_HIGHLIGHTED, false);
    stack.message(tree, &message, &Strings);
    assert_eq!(text(tree, field, "AmountEntry"), "9");

    // Typing keeps only digits; more than the most is refused with the reason.
    let entry = tree
        .child_recursive(field, tokenize("AmountEntry"))
        .unwrap();
    assert!(tree.activate(entry), "the entry takes typing");
    // End, then the characters.
    tree.character(0, 0x23, false);
    for ch in "2x".encode_utf16() {
        tree.character(ch, 0, false);
    }
    assert_eq!(text(tree, field, "AmountEntry"), "92");
    let ok = release(tree, field, "StackOkButton");
    assert_eq!(stack.message(tree, &ok, &Strings), None);
    assert_eq!(
        text(tree, field, "ErrorText"),
        format!(
            "{:08x}[({}, UInt(9))]",
            tokenize("ID_Stack_MaxQuantity"),
            tokenize("size")
        )
    );
    // A leading zero reads as octal: "010" is 8.
    tree.set_text(entry, "010");
    assert_eq!(
        stack.message(tree, &ok, &Strings),
        Some(StackUiDone {
            context: 7,
            amount: 8
        })
    );
    assert!(!tree.get(field).unwrap().visible);
    assert_eq!(stack.shown(), None);

    // Its source container closing closes it without an answer.
    stack.init(tree, 8, "", 1, 3, 0x8000_0001, 0x2000_0001);
    stack.container_state_update(tree, 0x2000_0001, false);
    assert_eq!(stack.shown(), Some(8));
    stack.container_state_update(tree, 0x8000_0001, false);
    assert_eq!(stack.shown(), None);
}
