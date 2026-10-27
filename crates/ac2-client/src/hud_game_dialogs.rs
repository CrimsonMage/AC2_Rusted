//! `gmGamePlayUI`'s message boxes and the resurrection offer's confirmation in the HUD. The frame's
//! `DisplayMessage` notices raise a message box ([`ac2_ui::GameplayDialogs::display_message`]);
//! the dialogs' answers close the box's context, and the offer's answers respawn or clear the
//! offer and make the death dialog again ([`ac2_ui::DeathPanel`]). The panel is kept by the HUD
//! as `game_dialogs`.

use ac2_ui::{DeathPanel, DialogFactory, DialogOutcome, GameplayDialogs, OfferAnswer, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

/// `RecvNotice_DisplayMessage` for the frame's notices, in order. The message's text is rendered
/// from its `StringInfo` on the player's tables.
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
        if let SessionEvent::Notice(GameplayNotice::DisplayMessage { text, topmost }) = event {
            let text = player.render_info(text);
            game.display_message(tree, dialogs, &text, *topmost);
        }
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
