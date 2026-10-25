//! The death panel (`gmGamePlayUI`'s resurrect dialog) on the real gameplay layouts
//! (`AC2_DAT_DIR`): the death notice raises the dialog with the `ID_Death_YouDied` text in its
//! `TextField`, and its `ResurrectButton` closes it and asks the respawn. The notice and the
//! button come from the frames the server double drives (`death_7160.rs`); this test checks the
//! dialog's layout names and its answer.

use std::sync::Arc;

use ac2_ui::{
    build_gameplay_hud, tokenize, DeathPanel, UiMessage, UiString, UiStrings, UiTree,
    DIALOG_RESURRECT, MSG_BUTTON_RELEASE_ON,
};

/// The panel's strings: the rendered id, so the text can be read back.
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

#[test]
fn the_death_dialog_shows_the_death_text_and_its_button_asks_the_respawn() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ui_store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let mut death = DeathPanel::default();

    death.notice(&mut hud.tree, &mut hud.dialogs, &Strings, true);
    let (context, kind) = hud.dialogs.shown().expect("the death dialog");
    assert_eq!((context, kind), (death.context(), DIALOG_RESURRECT));
    let root = hud
        .tree
        .roots()
        .iter()
        .copied()
        .find(|&r| hud.tree.get(r).unwrap().id() == tokenize("ResurrectField"))
        .expect("the resurrect dialog's root");
    let text = hud
        .tree
        .child_recursive(root, tokenize("TextField"))
        .and_then(|t| hud.tree.get(t)?.text())
        .expect("the dialog's text");
    assert_eq!(text, ac2_ui::you_died_string(&Strings));

    // The button closes the dialog and asks the respawn once.
    let press = release(&hud.tree, root, "ResurrectButton");
    hud.dialogs.message(&mut hud.tree, &press);
    let outcomes = hud.dialogs.take_outcomes();
    assert!(death.take_resurrect(&outcomes), "{outcomes:?}");
    assert!(hud.dialogs.shown().is_none());
    assert!(!death.take_resurrect(&outcomes));
}
