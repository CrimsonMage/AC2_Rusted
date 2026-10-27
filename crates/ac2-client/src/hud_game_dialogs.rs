//! `gmGamePlayUI`'s message boxes and the resurrection offer's confirmation in the HUD. The frame's
//! `DisplayMessage` notices raise a message box ([`ac2_ui::GameplayDialogs::display_message`]);
//! the dialogs' answers close the box's context, and the offer's answers respawn or clear the
//! offer and make the death dialog again ([`ac2_ui::DeathPanel`]). A script's yes/no question
//! (`gmCWInterface::PopupYesNoDialog`) is a confirmation whose answer resumes the script
//! ([`Command::PopupAnswer`]). The panel is kept by the HUD as `game_dialogs`.

use ac2_ui::{
    DeathPanel, DialogFactory, DialogOutcome, GameplayDialogs, OfferAnswer, UiMessage, UiStrings,
    UiTree,
};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

/// A script's yes/no question as the notice carried it, until the HUD's dialogs show it
/// ([`HudView::popup_backlog`](crate::hud_view::HudView)).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PopupQuestion {
    pub text: ac2_net::protocol::StringInfo,
    pub object: u64,
    pub context: u32,
}

/// A message box as the notice carried it (`GameplayNotice::DisplayMessage`), until the HUD's
/// dialogs show it ([`HudView::display_backlog`](crate::hud_view::HudView)).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DisplayNotice {
    pub text: ac2_net::protocol::StringInfo,
    pub topmost: bool,
}

/// The frame's message boxes (`GameplayNotice::DisplayMessage`), in order. Retail's handler is
/// registered by `gmGamePlayUI::SetupChildren`, so a box that comes before the HUD waits for it.
pub(crate) fn display_notices(events: &[SessionEvent]) -> Vec<DisplayNotice> {
    events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::Notice(GameplayNotice::DisplayMessage { text, topmost }) => {
                Some(DisplayNotice {
                    text: text.clone(),
                    topmost: *topmost,
                })
            }
            _ => None,
        })
        .collect()
}

/// The frame's script questions (`GameplayNotice::PopupYesNo`), in order.
pub(crate) fn popup_questions(events: &[SessionEvent]) -> Vec<PopupQuestion> {
    events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::Notice(GameplayNotice::PopupYesNo {
                object,
                context,
                text,
            }) => Some(PopupQuestion {
                text: text.clone(),
                object: *object,
                context: *context,
            }),
            _ => None,
        })
        .collect()
}

/// `RecvNotice_DisplayMessage` for the backlog's message boxes, in order, and `Init`'s dialog step
/// on the login. The message's text is rendered from its `StringInfo` on the player's tables. The
/// backlog's script questions are asked first, as they were raised before this frame's notices.
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
    // `gmCWInterface::PopupYesNoDialog`: the question's confirmation, answered later.
    for question in popup_backlog.drain(..) {
        let text = player.render_info(&question.text);
        game.ask_popup(tree, dialogs, &text, question.object, question.context);
    }
    for notice in display_backlog.drain(..) {
        let text = player.render_info(&notice.text);
        game.display_message(tree, dialogs, &text, notice.topmost);
    }
    for event in events {
        // `gmGamePlayUI::RecvNotice_LoginFinished` runs `Init`, which closes the please-wait
        // dialog the login left up.
        if let SessionEvent::LoginFinished = event {
            game.init(tree, dialogs);
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
                if let Some(popup) = game.take_popup(context) {
                    commands.push(Command::PopupAnswer {
                        object: popup.object,
                        context: popup.context,
                        yes,
                    });
                    continue;
                }
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
