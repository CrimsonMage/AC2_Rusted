//! The pregame UI (`ac2_ui::pregame::PregameUi`) joined to the session: the login events become
//! the UI's notices (`UIPersistantData::RecvNotice_CharacterSet`, `gmUIFlow::
//! RecvNotice_CharacterError` / `RecvNotice_ServerDied`, `RecvNotice_CharGenVerification`), and
//! the UI's requests go out through the session (`Proto_UI::SendEnterWorld`,
//! `SendDeleteCharacter`, `SendCharGenResult`). Run the session with
//! `CharacterChoice::Wait` so it stops at the character set.

use ac2_net::protocol::{CharacterIdentity, VisualDesc};
use ac2_ui::pregame::{PregameRequest, PregameUi};
use ac2_weenie::{OnlineSession, SessionEvent};

use crate::client::Notice;

/// What carrying out the UI's requests asks of the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PregameOutcome {
    /// `Device::Done`: close the client.
    pub quit: bool,
}

/// The session's character error text, `"character error N"` (`SessionEvent::Rejected`).
fn character_error(text: &str) -> Option<u32> {
    text.strip_prefix("character error ")?.parse().ok()
}

/// One session event into the UI.
pub fn pregame_event(ui: &mut PregameUi, event: &SessionEvent) {
    match event {
        SessionEvent::Characters(set) => ui.character_set(set.clone()),
        SessionEvent::Rejected(text) => {
            if let Some(error) = character_error(text) {
                ui.character_error(error, None);
            }
        }
        SessionEvent::CharGenVerification {
            response,
            character,
            name,
            result,
        } => {
            // The event carries the identity's id and name; its appearance and greyed time
            // are not passed on yet.
            let identity = CharacterIdentity {
                id: *character,
                name: name.clone(),
                seconds_greyed_out: 0,
                visual: VisualDesc::default(),
            };
            ui.char_gen_verification(*response, Some(&identity), *result);
        }
        SessionEvent::Disconnected(_) => ui.server_died(),
        _ => {}
    }
}

/// The client's notices into the UI (`ConnectionLost` is `RecvNotice_ServerDied`).
pub fn pregame_notice(ui: &mut PregameUi, notice: Notice) {
    if notice == Notice::ConnectionLost {
        ui.server_died();
    }
}

/// The UI's requests carried out on the session. A refused entry is reported as the error
/// retail's server would send (`CHAR_ERROR_ENTER_GAME_GENERIC`).
pub fn pregame_requests(ui: &mut PregameUi, session: &mut OnlineSession) -> PregameOutcome {
    let mut outcome = PregameOutcome::default();
    for request in ui.take_requests() {
        match request {
            PregameRequest::EnterWorld(character) => {
                if let Err(error) = session.enter_world(character) {
                    eprintln!("ac2: enter world: {error}");
                    ui.character_error(ac2_ui::pregame::char_error::ENTER_GAME_GENERIC, None);
                }
            }
            PregameRequest::DeleteCharacter(character) => session.delete_character(character),
            PregameRequest::CreateCharacter(result) => session.create_character(result),
            PregameRequest::RestoreCharacter { character, .. } => {
                eprintln!("ac2: restoring {character:#x} is an admin request; not sent");
            }
            PregameRequest::Quit => outcome.quit = true,
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::character_error;

    #[test]
    fn the_session_s_character_error_text_gives_the_error() {
        assert_eq!(character_error("character error 11"), Some(11));
        assert_eq!(character_error("no character matching First"), None);
    }
}
