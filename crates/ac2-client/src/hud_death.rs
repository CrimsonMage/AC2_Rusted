//! `gmGamePlayUI`'s death panel in the HUD: the frame's `UpdateDeathState` notices raise and close
//! the resurrect dialog, and its button asks the respawn (`RecvNotice_CloseResurrectDialog` calls
//! `gmCWInterface::RequestResurrect`). The panel is [`ac2_ui::DeathPanel`].

use ac2_ui::{DeathPanel, DialogFactory, DialogOutcome, UiStrings, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::gameplay::Command;

/// `RecvNotice_UpdateDeathState` and `RecvNotice_ResurrectionRequestedByOther` for the frame's
/// notices, in order. The offer's prompt is rendered with the resurrecter's name.
pub(crate) fn death_notices(
    death: &mut DeathPanel,
    tree: &mut UiTree,
    dialogs: &mut Option<DialogFactory>,
    strings: &dyn UiStrings,
    events: &[SessionEvent],
) {
    let Some(dialogs) = dialogs.as_mut() else {
        return;
    };
    for event in events {
        match event {
            SessionEvent::Notice(GameplayNotice::UpdateDeathState(dead)) => {
                death.notice(tree, dialogs, strings, *dead);
            }
            SessionEvent::Notice(GameplayNotice::ResurrectionRequestedByOther(name)) => {
                let prompt = strings.render_string(&ac2_ui::resurrection_prompt(name));
                death.recv_resurrection_requested(tree, dialogs, &prompt);
            }
            _ => {}
        }
    }
}

/// The dialogs' answers: the death dialog's button asks the respawn.
pub(crate) fn death_answers(
    death: &mut DeathPanel,
    outcomes: &[DialogOutcome],
    commands: &mut Vec<Command>,
) {
    if death.take_resurrect(outcomes) {
        commands.push(Command::RequestResurrect);
    }
}
