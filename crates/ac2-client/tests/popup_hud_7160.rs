//! A script's yes/no question on the HUD (`gmCWInterface::PopupYesNoDialog` through
//! `YesNoDialogAwakener`): the question is a confirmation with the question's text, its Yes or No
//! button answers it, and the answer goes back to the script as `Resume_PopupYesNoDialog`
//! (`Command::PopupAnswer`). A question raised before the HUD's dialogs exist waits for them
//! rather than being dropped. Driven through the HUD rig over the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter).
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_net::protocol::StringInfo;
use ac2_ui::{tokenize, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent};
use hud::Rig;

const OBJECT: u64 = 0x5000_0009;
const CONTEXT: u32 = 0x44;

/// The shown confirmation's root (`ConfirmationField`), when one is up.
fn confirmation(tree: &UiTree) -> Option<usize> {
    tree.roots().iter().copied().find(|&root| {
        tree.get(root)
            .is_some_and(|e| e.id() == tokenize("ConfirmationField") && e.visible())
    })
}

fn question(text: &str, context: u32) -> SessionEvent {
    SessionEvent::Notice(GameplayNotice::PopupYesNo {
        object: OBJECT,
        context,
        text: StringInfo::literal(text),
    })
}

/// The confirmation's question text, as its `QuestionText` shows it.
fn shown_text(tree: &UiTree) -> Option<String> {
    let root = confirmation(tree)?;
    let text = tree.child_recursive(root, tokenize("QuestionText"))?;
    tree.get(text)?.text()
}

#[test]
fn a_script_question_is_asked_and_its_answers_go_back_to_the_script() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    // The question comes before the HUD is up when the HUD builds on workers: it waits in the
    // backlog, and the first frames with the HUD ask it.
    rig.notify(&[question("Accept the quest?", CONTEXT)]);
    rig.wait("the question's confirmation", |r| {
        r.view.ui_tree().and_then(confirmation).is_some()
    });
    let tree = rig.view.ui_tree().expect("the HUD");
    assert_eq!(shown_text(tree).as_deref(), Some("Accept the quest?"));

    // Yes: the confirmation closes and the answer goes back to the script.
    let yes = rig
        .view
        .ui_element_center("YesButton")
        .expect("the Yes button");
    rig.click(yes);
    rig.idle(4);
    assert!(confirmation(rig.view.ui_tree().unwrap()).is_none());
    assert!(
        rig.asked.contains(&Command::PopupAnswer {
            object: OBJECT,
            context: CONTEXT,
            yes: true,
        }),
        "{:?}",
        rig.asked
    );

    // No, to a second question: the answer is no, and the confirmation closes the same way.
    rig.notify(&[question("Leave the instance?", CONTEXT + 1)]);
    rig.wait("the second question", |r| {
        r.view.ui_tree().and_then(confirmation).is_some()
    });
    let no = rig
        .view
        .ui_element_center("NoButton")
        .expect("the No button");
    rig.click(no);
    rig.idle(4);
    assert!(confirmation(rig.view.ui_tree().unwrap()).is_none());
    assert!(rig.asked.contains(&Command::PopupAnswer {
        object: OBJECT,
        context: CONTEXT + 1,
        yes: false,
    }));
}

/// The HUD key (`0x4100000C`): a press and a release, as the window sends it.
const HUD_KEY: u32 = 0x4100_000C;

#[test]
fn a_popup_with_the_gameplay_panels_hidden_still_has_its_buttons() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    // The HUD key hides the gameplay panels (`gmGamePlayUI::Show(0)`).
    rig.frame(&[(HUD_KEY, 1.0, true)]);
    rig.frame(&[(HUD_KEY, 1.0, false)]);
    assert!(!rig.view.gameplay_shown());

    rig.notify(&[question("Accept the quest?", CONTEXT)]);
    rig.wait("the question's confirmation", |r| {
        r.view.ui_tree().and_then(confirmation).is_some()
    });
    let yes = rig
        .view
        .ui_element_center("YesButton")
        .expect("the Yes button");
    rig.click(yes);
    rig.idle(4);
    assert!(
        rig.asked.contains(&Command::PopupAnswer {
            object: OBJECT,
            context: CONTEXT,
            yes: true,
        }),
        "{:?}",
        rig.asked
    );
}
