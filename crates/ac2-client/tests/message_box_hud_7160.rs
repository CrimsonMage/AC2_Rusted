//! A server message box (`CM_Character::SendNotice_DisplayMessage` into `gmGamePlayUI::
//! RecvNotice_DisplayMessage`) that arrives before the HUD's dialogs exist: retail registers the
//! notice handler in `gmGamePlayUI::SetupChildren` (run by `Init` at login), so the box is shown as
//! soon as the HUD is up. The HUD builds on workers here, so the box waits for it and is shown
//! once, not twice. Driven through the HUD rig over the Legions server double (`AC2_DAT_DIR`, a
//! GPU adapter).
#![cfg(feature = "window")]

mod hud;

use ac2_net::protocol::StringInfo;
use ac2_ui::{tokenize, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent};
use hud::Rig;

/// The shown message box's root (`MessageField`), when one is up.
fn message_box(tree: &UiTree) -> Option<usize> {
    tree.roots().iter().copied().find(|&root| {
        tree.get(root)
            .is_some_and(|e| e.id() == tokenize("MessageField") && e.visible())
    })
}

/// How many message boxes are up as roots.
fn message_boxes(tree: &UiTree) -> usize {
    tree.roots()
        .iter()
        .filter(|&&root| {
            tree.get(root)
                .is_some_and(|e| e.id() == tokenize("MessageField"))
        })
        .count()
}

/// The message box's text (`TextField`).
fn shown_text(tree: &UiTree) -> Option<String> {
    let root = message_box(tree)?;
    let text = tree.child_recursive(root, tokenize("TextField"))?;
    tree.get(text)?.text()
}

fn display(text: &str, topmost: bool) -> SessionEvent {
    SessionEvent::Notice(GameplayNotice::DisplayMessage {
        text: StringInfo::literal(text),
        topmost,
    })
}

#[test]
fn a_message_box_before_the_hud_is_built_shows_once_it_is() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    // The HUD builds on a worker and arrives in `update`, which the rig has not run yet: the
    // message comes before the HUD exists.
    assert!(rig.view.ui_tree().is_none());
    let frame = rig.client.frame();
    rig.view.frameworks(
        &mut rig.client,
        &frame,
        &[display("Your sword is ready.", false)],
        &[],
    );

    rig.wait("the message box", |r| {
        r.view.ui_tree().and_then(message_box).is_some()
    });
    let tree = rig.view.ui_tree().expect("the HUD");
    assert_eq!(shown_text(tree).as_deref(), Some("Your sword is ready."));
    assert_eq!(message_boxes(tree), 1, "one box, not one per frame");

    // Its OK closes it, and nothing shows it again.
    let ok = rig
        .view
        .ui_element_center("OkButton")
        .expect("the OK button");
    rig.click(ok);
    rig.idle(8);
    let tree = rig.view.ui_tree().expect("the HUD");
    assert!(message_box(tree).is_none());
    assert_eq!(message_boxes(tree), 0);
}
