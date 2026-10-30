//! Keyboard use of the active element (`crate::activation`): the real dialogs'
//! `activate_on_show` buttons take Enter.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::activation::UI_USE_ACTIVE;
use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_MESSAGE, LAYOUT_DIALOGS};
use crate::mouse::GLOBAL_KEY_PRESS;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::{build_gameplay_hud, UiTree};

fn hud_and_dialogs(store: &Arc<ResourceStore>) -> (UiTree, DialogFactory) {
    let tree = build_gameplay_hud(store, 1024, 768).unwrap().tree;
    let layout = crate::hud::load_layout_by_enum(store, LAYOUT_DIALOGS)
        .unwrap()
        .unwrap();
    (tree, DialogFactory::new(Some(layout)))
}

#[test]
fn a_message_dialogs_ok_button_takes_enter_as_its_click() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut dialogs) = hud_and_dialogs(&store);
    tree.take_messages();
    let context = dialogs.make_dialog(&mut tree, "Hello", DIALOG_MESSAGE, false);
    let ok = tree.active().expect("the OK button activates as it shows");
    assert_eq!(tree.get(ok).unwrap().id(), tokenize("OkButton"));
    tree.take_messages();
    tree.start_input(UI_USE_ACTIVE, 1.0);
    tree.stop_input(UI_USE_ACTIVE);
    let messages = tree.take_messages();
    // The button took the key: no global key press (the chat's Enter) went out.
    assert!(!messages
        .iter()
        .any(|m| m.message == GLOBAL_KEY_PRESS && m.data == UI_USE_ACTIVE as i32));
    let release = messages
        .iter()
        .find(|m| m.message == MSG_BUTTON_RELEASE_ON)
        .expect("Enter releases on the button");
    assert_eq!(release.element, Some(ok));
    dialogs.message(&mut tree, release);
    assert_eq!(
        dialogs.take_outcomes(),
        [DialogOutcome::Message { context }]
    );
}

#[test]
fn a_dialog_shown_while_enter_is_held_does_not_activate() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut dialogs) = hud_and_dialogs(&store);
    tree.start_input(UI_USE_ACTIVE, 1.0);
    dialogs.make_dialog(&mut tree, "Hello", DIALOG_MESSAGE, false);
    assert_eq!(tree.active(), None);
    // The held Enter goes out as the global key press it was.
    tree.stop_input(UI_USE_ACTIVE);
    assert!(dialogs.take_outcomes().is_empty());
}
