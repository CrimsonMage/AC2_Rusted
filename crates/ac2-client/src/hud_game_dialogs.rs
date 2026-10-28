//! `gmGamePlayUI`'s message boxes and the resurrection offer's confirmation in the HUD. The frame's
//! dialog notices and answers run in [`ac2_ui::GameplayDialogs`] (`frame_notices`, `frame_answers`);
//! this file keeps the player's text ([`ac2_ui::DialogText`]) and the requests as gameplay
//! commands. A script's yes/no question (`gmCWInterface::PopupYesNoDialog`) is a confirmation whose
//! answer resumes the script ([`Command::PopupAnswer`]). The panel is kept by the HUD as
//! `game_dialogs`.

use ac2_ui::{
    DeathPanel, DialogFactory, DialogOutcome, DialogRequest, DisplayNotice, GameplayDialogs,
    PopupQuestion, UiStrings, UiTree,
};
use ac2_weenie::SessionEvent;

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

/// `RecvNotice_DisplayMessage` for the backlog's message boxes, in order, and `Init`'s dialog step
/// on the login (`GameplayDialogs::frame_notices`).
pub(crate) fn game_dialog_notices(
    game: &mut GameplayDialogs,
    tree: &mut UiTree,
    dialogs: &mut Option<DialogFactory>,
    player: &ClientPlayer<'_>,
    events: &[SessionEvent],
    popup_backlog: &mut Vec<PopupQuestion>,
    display_backlog: &mut Vec<DisplayNotice>,
) {
    let Some(dialogs) = dialogs.as_mut() else {
        return;
    };
    game.frame_notices(
        tree,
        dialogs,
        player,
        events,
        popup_backlog,
        display_backlog,
    );
}

/// `gmGamePlayUI::ListenToMessage` for the global loop: a pending logout runs `EndSession(1)`,
/// which closes the death dialog and raises the please-wait dialog. Then the game options go up
/// (`gmCWInterface::UploadGameOptions`) and the character logs off (`Proto_UI::LogOffCharacter`),
/// in that order.
pub(crate) fn game_dialog_logout(
    game: &mut GameplayDialogs,
    death: &mut DeathPanel,
    tree: &mut UiTree,
    dialogs: &mut Option<DialogFactory>,
    strings: &dyn UiStrings,
    commands: &mut Vec<Command>,
) {
    let Some(dialogs) = dialogs.as_mut() else {
        return;
    };
    if game.end_session_loop(tree, dialogs, strings, death) {
        commands.push(Command::UploadGameOptions);
        commands.push(Command::LogOffCharacter);
    }
}

/// The dialogs' answers for the gameplay panel (`GameplayDialogs::frame_answers`), as commands.
pub(crate) fn game_dialog_answers(
    game: &mut GameplayDialogs,
    death: &mut DeathPanel,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    outcomes: &[DialogOutcome],
    commands: &mut Vec<Command>,
) {
    for request in game.frame_answers(death, tree, dialogs, player, outcomes) {
        commands.push(match request {
            DialogRequest::PopupAnswer {
                object,
                context,
                yes,
            } => Command::PopupAnswer {
                object,
                context,
                yes,
            },
            DialogRequest::ClearResurrection => Command::ClearResurrectionRequest,
            DialogRequest::RequestResurrect => Command::RequestResurrect,
        });
    }
}
