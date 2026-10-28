//! `gmGamePlayUI`'s death panel in the HUD: the frame's notices are run by `ac2_ui::death_notices`,
//! and the death dialog's button asks the respawn (`RecvNotice_CloseResurrectDialog` calls
//! `gmCWInterface::RequestResurrect`). The panel is [`ac2_ui::DeathPanel`].

use ac2_ui::{DeathPanel, DialogOutcome};

use crate::gameplay::Command;

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
