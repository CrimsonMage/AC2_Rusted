//! `gmGamePlayUI`'s message boxes and the resurrection offer's confirmation in the HUD. The frame's
//! `DisplayMessage` notices raise a message box ([`ac2_ui::GameplayDialogs::display_message`]);
//! the dialogs' answers close the box's context, and the offer's answers respawn or clear the
//! offer and make the death dialog again ([`ac2_ui::DeathPanel`]). The panel is kept by the HUD
//! as `game_dialogs`.

use ac2_ui::{
    DeathPanel, DialogFactory, DialogOutcome, GameplayDialogs, OfferAnswer, UiMessage, UiStrings,
    UiTree,
};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

/// `RecvNotice_DisplayMessage` for the frame's notices, in order, and `Init`'s dialog step on the
/// login. The message's text is rendered from its `StringInfo` on the player's tables.
pub(crate) fn game_dialog_notices(
    game: &mut GameplayDialogs,
    tree: &mut UiTree,
    dialogs: &mut Option<DialogFactory>,
    player: &ClientPlayer<'_>,
    events: &[SessionEvent],
) {
    let Some(dialogs) = dialogs.as_mut() else {
        return;
    };
    for event in events {
        match event {
            SessionEvent::Notice(GameplayNotice::DisplayMessage { text, topmost }) => {
                let text = player.render_info(text);
                game.display_message(tree, dialogs, &text, *topmost);
            }
            // `gmGamePlayUI::RecvNotice_LoginFinished` runs `Init`, which closes the please-wait
            // dialog the login left up.
            SessionEvent::LoginFinished => game.init(tree, dialogs),
            _ => {}
        }
    }
}

/// `gmGamePlayUI::ListenToMessage` for the frame's messages: the logout key (`LOGOUT_KEY`,
/// global key press `0x4100000e`) asks for the end of the session on the next loop.
pub(crate) fn game_dialog_message(game: &mut GameplayDialogs, message: &UiMessage) {
    if message.element.is_none()
        && message.message == ac2_ui::GLOBAL_KEY_PRESS
        && message.data as u32 == ac2_ui::LOGOUT_ACTION
    {
        game.logout_key();
    }
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

/// The dialogs' answers for the gameplay panel: a message box that was answered leaves the
/// list, and the resurrection offer's answer respawns (yes) or clears the offer and makes the
/// death dialog again (no).
pub(crate) fn game_dialog_answers(
    game: &mut GameplayDialogs,
    death: &mut DeathPanel,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    outcomes: &[DialogOutcome],
    commands: &mut Vec<Command>,
) {
    for outcome in outcomes {
        match *outcome {
            DialogOutcome::Message { context } => game.close_message(context),
            DialogOutcome::Confirmation { context, yes } => {
                match death.recv_close_confirmation(context, yes) {
                    OfferAnswer::None => {}
                    OfferAnswer::Declined => {
                        commands.push(Command::ClearResurrectionRequest);
                        death.make_you_died(tree, dialogs, player);
                    }
                    OfferAnswer::Accepted => commands.push(Command::RequestResurrect),
                }
            }
            DialogOutcome::Resurrect { .. } => {}
        }
    }
}
