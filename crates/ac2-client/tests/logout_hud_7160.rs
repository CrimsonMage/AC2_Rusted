//! The logout key on the HUD (`gmGamePlayUI::ListenToMessage`, `LOGOUT_KEY` = Shift+Escape, UI
//! action `0x4100000e`): the key press asks, the next UI loop runs `EndSession(1)`, which raises
//! the please-wait dialog and asks for the game options to go up and the character to log off
//! (`gmCWInterface::UploadGameOptions`, then `Proto_UI::LogOffCharacter`). The server's
//! `CharExitGame` is the session's end. Driven through the HUD rig over the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter).
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_ui::LOGOUT_ACTION;
use ac2_weenie::SessionEvent;
use hud::Rig;

#[test]
fn the_logout_key_ends_the_session_once_and_logs_the_character_off() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    // The HUD builds on worker loads; the key goes down and comes up once it is up.
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    // The key goes down and comes up: the action is a press, then its release.
    rig.frame(&[(LOGOUT_ACTION, 1.0, true)]);
    rig.frame(&[(LOGOUT_ACTION, 1.0, false)]);
    let mut left = false;
    for _ in 0..200 {
        left |= rig.frame(&[]).contains(&SessionEvent::LeftWorld);
        if left {
            break;
        }
    }
    let asked = rig.asked.clone();
    let count = |command: &Command| asked.iter().filter(|c| *c == command).count();
    assert_eq!(count(&Command::LogOffCharacter), 1, "{asked:?}");
    assert_eq!(count(&Command::UploadGameOptions), 1, "{asked:?}");
    let upload = asked
        .iter()
        .position(|c| *c == Command::UploadGameOptions)
        .unwrap();
    let logoff = asked
        .iter()
        .position(|c| *c == Command::LogOffCharacter)
        .unwrap();
    assert!(upload < logoff, "the options go up before the logoff");
    assert!(left, "the server answered CharExitGame");

    // Ended once: a second press asks nothing more.
    rig.frame(&[(LOGOUT_ACTION, 1.0, true)]);
    rig.frame(&[(LOGOUT_ACTION, 1.0, false)]);
    assert_eq!(
        rig.asked
            .iter()
            .filter(|c| **c == Command::LogOffCharacter)
            .count(),
        1
    );
}
