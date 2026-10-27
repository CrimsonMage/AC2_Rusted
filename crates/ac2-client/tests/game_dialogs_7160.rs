//! `gmGamePlayUI`'s message boxes and please-wait dialog on the real gameplay layout
//! (`AC2_DAT_DIR`): a message box shows its text in `MessageField`'s `TextField` and its `OkButton`
//! answers it, which removes its context from the list; the please-wait dialog shows
//! `ID_Wait_PleaseWait` once and its close cancels the wait.

use ac2_ui::{
    build_gameplay_hud, tokenize, DialogOutcome, GameplayDialogs, UiMessage, UiString, UiStrings,
    UiTree, DIALOG_MESSAGE, DIALOG_WAIT, MSG_BUTTON_RELEASE_ON,
};

/// The dialogs' strings: the rendered id, so the text can be read back.
struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        format!("{:08x}/{:08x}", string.table_enum, string.string_id)
    }
}

fn release(tree: &UiTree, root: usize, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: tree.child_recursive(root, tokenize(name)),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

fn root_of(tree: &UiTree, field: &str) -> usize {
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == tokenize(field))
        .unwrap_or_else(|| panic!("the {field} root"))
}

#[test]
fn a_message_box_shows_its_text_and_its_ok_answers_it() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ui_store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let mut game = GameplayDialogs::default();

    let context = game.display_message(&mut hud.tree, &mut hud.dialogs, "hello", false);
    assert_eq!(hud.dialogs.shown(), Some((context, DIALOG_MESSAGE)));
    assert_eq!(game.message_contexts(), &[context]);
    let root = root_of(&hud.tree, "MessageField");
    let text = hud
        .tree
        .child_recursive(root, tokenize("TextField"))
        .and_then(|t| hud.tree.get(t)?.text())
        .expect("the message's text");
    assert_eq!(text, "hello");

    let press = release(&hud.tree, root, "OkButton");
    hud.dialogs.message(&mut hud.tree, &press);
    let outcomes = hud.dialogs.take_outcomes();
    assert_eq!(outcomes, [DialogOutcome::Message { context }]);
    assert!(hud.dialogs.shown().is_none());
    game.close_message(context);
    assert!(game.message_contexts().is_empty());
}

#[test]
fn the_please_wait_dialog_is_made_once_and_its_close_cancels_the_wait() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ui_store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let mut game = GameplayDialogs::default();

    game.make_please_wait(&mut hud.tree, &mut hud.dialogs, &Strings);
    let context = game.wait_context();
    assert_eq!(hud.dialogs.shown(), Some((context, DIALOG_WAIT)));
    let root = root_of(&hud.tree, "WaitField");
    let text = hud
        .tree
        .child_recursive(root, tokenize("TextField"))
        .and_then(|t| hud.tree.get(t)?.text())
        .expect("the wait's text");
    // `ID_Wait_PleaseWait` of the UI table (`0x41000018`), rendered by the test's strings.
    let id = tokenize("ID_Wait_PleaseWait");
    assert_eq!(text, format!("{:08x}/{id:08x}", 0x4100_0018u32));

    game.make_please_wait(&mut hud.tree, &mut hud.dialogs, &Strings);
    assert_eq!(game.wait_context(), context);
    assert!(hud.dialogs.queued().is_empty());

    hud.dialogs.close_dialog(&mut hud.tree, context);
    assert!(game.recv_close_wait_dialog(context));
    assert_eq!(game.wait_context(), 0);
}
