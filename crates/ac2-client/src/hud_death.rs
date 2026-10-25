//! `gmGamePlayUI`'s death panel in the HUD: the frame's `UpdateDeathState` notices raise and close
//! the resurrect dialog, and its button asks the respawn (`RecvNotice_CloseResurrectDialog` calls
//! `gmCWInterface::RequestResurrect`). The panel is [`ac2_ui::DeathPanel`].

use ac2_ui::{DeathPanel, DialogFactory, DialogOutcome, UiStrings, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::gameplay::Command;

/// `RecvNotice_UpdateDeathState` for the frame's notices, in order.
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
        if let SessionEvent::Notice(GameplayNotice::UpdateDeathState(dead)) = event {
            death.notice(tree, dialogs, strings, *dead);
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
